//! The table's glue: the block entity Part, the click and menu Parts, the
//! declarations, the deferred experience charge and the `/disenchanting_table
//! info` readout.
//!
//! Every rule lives in [`crate::runtime`] (the retired bundle's `runtime.rs`,
//! byte-identical); this file reads the game, calls it and writes the result
//! back. Its tests pin what it *declares* (three slots and the menu, the panel
//! geometry), which component an item's enchantments live in, the exact patches
//! a strip and an output book carry, and the readout's field formatting.
use std::sync::{Arc, Mutex, PoisonError};

use mod_api::{
    BlockClick, BlockEntityEvent, BlockEntityPart, BlockEntityRef, BlockEntityTypeKey, BlockPart,
    ClientHost, CommandArgumentValue, ComponentId, Compound, Host, HostTransaction, Identifier,
    InteractionResult, ItemResource, ItemStack, LevelRef, LevelSide, MenuPart, MenuSlotRulesEvent,
    ModContainer, PlayerRef, Registrar, Screen, ScreenDrawEvent, ScreenEvent, ServerHost, Tag,
    Text, UseItemOnEvent, UseWithoutItemEvent,
};

use crate::runtime::{
    BOOK_SLOT, Charge, DEFAULT_SETTINGS, ENCHANTED_BOOK_ITEM, INPUT_SLOT, OUTPUT_SLOT, Plan,
    SLOT_COUNT, TABLE_MENU, affordable_with, charge, is_book, plan as plan_take, slot_rule,
    total_points,
};

/// The block entity type's path, in this mod's namespace.
pub(crate) const TABLE_TYPE: &str = "disenchanting_table";

/// `minecraft:enchantments`: an item's applied enchantments.
const KEY_ENCHANTMENTS: &str = "minecraft:enchantments";
/// `minecraft:stored_enchantments`: an enchanted book's own ones.
const KEY_STORED_ENCHANTMENTS: &str = "minecraft:stored_enchantments";
/// `minecraft:repair_cost`: the anvil's prior-work penalty (`resets_repair_cost`).
const KEY_REPAIR_COST: &str = "minecraft:repair_cost";

/// The fields the readout prints for one block: the three slots, the cost, the
/// player's experience and whether a click would dis-enchant.
pub(crate) struct TableSlots {
    input: Option<ItemStack>,
    book: Option<ItemStack>,
    output: Option<ItemStack>,
}

// ---------------------------------------------------------------------------
// Declarations
// ---------------------------------------------------------------------------

/// The block entity type declared for the table, built in `Hooks::register`.
pub(crate) fn declare(registrar: &mut dyn Registrar) -> BlockEntityTypeKey {
    registrar
        .block_entity_type(TABLE_TYPE)
        .blocks(&[TABLE_TYPE])
        .build()
}

/// The table's container: three slots every face reaches — input 0, book 1,
/// output 2 — with the input and book filling and the input and output
/// emptying, and the table's menu. Upstream's `DisEnchantingStrategy
/// .canOutputToHopper` lets a hopper put an input and a book in and take a
/// dis-enchanted item out.
pub(crate) fn container() -> ModContainer {
    let all = [INPUT_SLOT, BOOK_SLOT, OUTPUT_SLOT];
    ModContainer {
        slots: SLOT_COUNT,
        faces: std::array::from_fn(|_| all.to_vec()),
        inputs: vec![INPUT_SLOT, BOOK_SLOT],
        outputs: vec![INPUT_SLOT, OUTPUT_SLOT],
        menu: Some(TABLE_MENU.to_owned()),
    }
}

/// The table's menu (`native_screen`, API 19 at the retired pin):
/// `DisEnchantingMenu`'s 27 / 76 / 134 at y 47, in a 176 × 166 screen.
///
/// The jar's GUI texture is carried but not drawn: Horizon's plain panel and
/// slot frames are used (a gap, see `notes.md`).
pub(crate) fn menu() -> Screen {
    Screen {
        width: 176.0,
        height: 166.0,
        slots: vec![Some([27.0, 47.0]), Some([76.0, 47.0]), Some([134.0, 47.0])],
        inventory_y: 84.0,
        inventory_x: 8.0,
        own_labels: false,
        own_background: false,
        ..Screen::default()
    }
}

// ---------------------------------------------------------------------------
// Enchantments: reading a stack, writing a component
// ---------------------------------------------------------------------------

/// The component an item's enchantments live in: `crafting_enchantments` keeps
/// a book's own ones in `stored_enchantments` and every other item's applied
/// ones in `enchantments`.
fn enchantment_component(item: &str) -> &'static str {
    if crate::runtime::is_enchanted_book(item) {
        KEY_STORED_ENCHANTMENTS
    } else {
        KEY_ENCHANTMENTS
    }
}

/// A stack's enchantments as `(id, level)`, namespaced, in the engine's own
/// order. Reads the item's effective component (its own, else the item's
/// default) and accepts the pre-1.21.5 `{levels: {…}}` wrapper the engine still
/// reads, so an item carried from an older world dis-enchants too.
fn levels_of(host: &dyn Host, stack: &ItemStack) -> Vec<(String, u32)> {
    let item = stack.item().id(host);
    let Some(component) = ComponentId::new(enchantment_component(&item)) else {
        return Vec::new();
    };
    let Some(Tag::Compound(map)) = ItemResource::of(stack).value(host, &component) else {
        return Vec::new();
    };
    let wrapped = map.iter().find_map(|(key, value)| match value {
        Tag::Compound(inner) if key.to_string_lossy() == "levels" => Some(inner),
        _ => None,
    });
    wrapped
        .unwrap_or(&map)
        .iter()
        .filter_map(|(key, value)| {
            let Tag::Int(level) = value else {
                return None;
            };
            if !(1..=255).contains(level) {
                return None;
            }
            let id = key.to_string_lossy();
            let id = if id.contains(':') {
                id
            } else {
                format!("minecraft:{id}")
            };
            Some((id, u32::try_from(*level).ok()?))
        })
        .collect()
}

/// The level map the engine writes for `minecraft:enchantments` and
/// `minecraft:stored_enchantments` in 26.3: `{id: level}` with no wrapper.
fn levels_component(levels: &[(String, u32)]) -> Tag {
    Tag::compound(levels.iter().map(|(id, level)| {
        (
            id.as_str(),
            Tag::Int(i32::try_from(*level).unwrap_or(i32::MAX)),
        )
    }))
}

/// The component patch that makes an item the stripped (or split) input of
/// `plan`: the enchantments removed, or a book's first kept, and the
/// prior-work penalty cleared (`resets_repair_cost`).
fn strip_patch(item: &str, plan: &Plan) -> Compound {
    /// The engine's explicit removal marker for one component: `!id` with an
    /// empty compound, which `apply_components` reads as "this stack has none",
    /// not as "leave the item's default".
    fn remove(patch: &mut Compound, key: &str) {
        patch.insert(
            format!("!{key}").as_str().into(),
            Tag::Compound(Compound::new()),
        );
    }
    let mut patch = Compound::new();
    match plan {
        Plan::Whole { .. } => remove(&mut patch, enchantment_component(item)),
        // A book keeps its first enchantment (`onTake`'s book branch).
        Plan::Split { kept, .. } => {
            patch.insert(
                KEY_STORED_ENCHANTMENTS.into(),
                levels_component(std::slice::from_ref(kept)),
            );
        }
    }
    if DEFAULT_SETTINGS.resets_repair_cost {
        remove(&mut patch, KEY_REPAIR_COST);
    }
    patch
}

/// A new enchanted book carrying `moved`.
fn book_with(host: &dyn Host, moved: &[(String, u32)]) -> Option<ItemStack> {
    let key = host.item_key(&Identifier::parse(ENCHANTED_BOOK_ITEM).ok()?)?;
    let mut book = ItemStack::of(key);
    let Tag::Compound(patch) = Tag::compound([(KEY_STORED_ENCHANTMENTS, levels_component(moved))])
    else {
        return None;
    };
    book.apply_components(&patch).ok()?;
    Some(book)
}

// ---------------------------------------------------------------------------
// The dis-enchant
// ---------------------------------------------------------------------------

/// A take paid in points, waiting for the server tick that can charge them: the
/// console is the only surface that takes points (`give_experience_points` only
/// gives them, and `commands()` refuses inside a player's hook). The result is
/// granted *after* the charge runs, so no output ever exists before it is paid
/// for.
pub(crate) struct PendingTake {
    name: String,
    uuid: u128,
    pos: [i32; 3],
    plan: Plan,
    moved: Vec<(String, u32)>,
    points: u32,
}

/// The takes a click queued and the points they already promised, by player:
/// the bar has not moved yet, so a second click would otherwise pass the same
/// affordability check.
#[derive(Default)]
pub(crate) struct Charges {
    pending: Mutex<Vec<PendingTake>>,
    reserved: Mutex<Vec<(u128, u64)>>,
}

impl Charges {
    fn reserve(&self, uuid: u128, points: u32) {
        let mut guard = self.reserved.lock().unwrap_or_else(PoisonError::into_inner);
        match guard.iter_mut().find(|(id, _)| *id == uuid) {
            Some((_, total)) => *total += u64::from(points),
            None => guard.push((uuid, u64::from(points))),
        }
    }

    fn promised(&self, uuid: u128) -> u64 {
        let guard = self.reserved.lock().unwrap_or_else(PoisonError::into_inner);
        guard
            .iter()
            .find(|(id, _)| *id == uuid)
            .map_or(0, |(_, total)| *total)
    }

    fn release(&self, uuid: u128, points: u32) {
        let mut guard = self.reserved.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some((_, total)) = guard.iter_mut().find(|(id, _)| *id == uuid) {
            *total = total.saturating_sub(u64::from(points));
        }
        guard.retain(|(_, total)| *total > 0);
    }

    fn queue(&self, take: PendingTake) {
        self.pending
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(take);
    }

    fn take_pending(&self) -> Vec<PendingTake> {
        let mut guard = self.pending.lock().unwrap_or_else(PoisonError::into_inner);
        std::mem::take(&mut *guard)
    }
}

/// A right click on the table with an eligible input, a book and an empty
/// output slot: upstream's `onTake` in one step — strip or split the input,
/// spend one book, put the produced enchanted book in the output slot, charge
/// the experience — and consume the click. Anything else answers `None`, so the
/// click falls through to the template's use path and opens the menu, and every
/// refusal leaves all three slots exactly as they were.
pub(crate) fn disenchant(
    charges: &Charges,
    host: &mut dyn ServerHost,
    click: &BlockClick<'_>,
) -> Option<InteractionResult> {
    let player = host.player(click.player)?;
    let level = player.level(&*host)?;
    let entity = host.block_entity(level, click.pos)?;
    let slots = read_slots(&*host, entity)?;
    // The produced book waits in the output slot; a click then opens the menu.
    if slots.output.is_some() {
        return None;
    }
    // The book slot must hold an actual book, not merely nothing: a hopper can
    // put anything in, and the take spends whatever sits there.
    let book = slots.book?;
    if !is_book(&book.item().id(&*host)) {
        return None;
    }
    let input = slots.input?;
    let levels = levels_of(&*host, &input);
    let view: Vec<(&str, u32)> = levels
        .iter()
        .map(|(id, level)| (id.as_str(), *level))
        .collect();
    let plan = plan_take(&input.item().id(&*host), &view)?;
    let moved = match &plan {
        Plan::Whole { moved } | Plan::Split { moved, .. } => moved.clone(),
    };
    let cost = charge(
        DEFAULT_SETTINGS,
        player.is_creative(&*host).unwrap_or(false),
    );
    let (xp_level, xp_progress) = player.experience(&*host).map_or((0, 0.0), |experience| {
        (experience.level, experience.progress)
    });
    // A take already queued this tick has promised its points but not spent
    // them, so the bar still reads the old balance.
    if !affordable_with(cost, xp_level, xp_progress, charges.promised(player.uuid())) {
        let _ = player.action_bar(host, Text::literal("Not enough experience to dis-enchant"));
        return None;
    }
    match cost {
        // Paid in points: charge first, next tick, then hand the result over.
        Charge::Points(points) => {
            let name = player.name(&*host)?;
            charges.reserve(player.uuid(), points);
            charges.queue(PendingTake {
                name,
                uuid: player.uuid(),
                pos: click.pos,
                plan,
                moved,
                points,
            });
            let _ = player.action_bar(host, Text::literal("Dis-enchanting costs 25 experience"));
            Some(InteractionResult::Consume)
        }
        // Levels come off here, where the acting player is reachable.
        Charge::Levels(levels) => {
            let levels = i32::try_from(levels).unwrap_or(i32::MAX);
            if !player.give_levels(host, -levels) {
                return None;
            }
            if !finish(host, level, click.pos, &plan, &moved) {
                // The write did not happen: give the levels back.
                let _ = player.give_levels(host, levels);
            }
            Some(InteractionResult::Consume)
        }
        Charge::Nothing => {
            finish(host, level, click.pos, &plan, &moved).then_some(InteractionResult::Consume)
        }
    }
}

/// Strips or splits the input, spends one book and puts the produced book in the
/// output slot — upstream's `onTake` in one step. Every slot write is in the
/// caller's transaction, so a refusal anywhere leaves all three as they were.
fn finish(
    host: &mut dyn ServerHost,
    level: LevelRef,
    pos: [i32; 3],
    plan: &Plan,
    moved: &[(String, u32)],
) -> bool {
    host.transaction(|host, tx| -> Result<bool, ()> {
        let Some(entity) = host.block_entity(level, pos) else {
            return Ok(false);
        };
        let Ok(input) = entity.item_slot(&*host, INPUT_SLOT as u32) else {
            return Ok(false);
        };
        let Ok(book) = entity.item_slot(&*host, BOOK_SLOT as u32) else {
            return Ok(false);
        };
        let Ok(output) = entity.item_slot(&*host, OUTPUT_SLOT as u32) else {
            return Ok(false);
        };
        if output.is_some_and(|stack| stack.count() > 0) {
            return Ok(false);
        }
        let Some(input) = input.filter(|stack| stack.count() > 0) else {
            return Ok(false);
        };
        // The book is spent here, so it must still be one: a hopper, a save, or
        // the tick between the click and the charge may have put something else
        // there.
        let Some(book) =
            book.filter(|stack| stack.count() > 0 && is_book(&stack.item().id(&*host)))
        else {
            return Ok(false);
        };
        let item = input.item().id(&*host);
        let patch = strip_patch(&item, plan);
        let mut stripped = input;
        stripped.apply_components(&patch).map_err(|_| ())?;
        let produced = book_with(&*host, moved).ok_or(())?;
        let left = book.count().saturating_sub(1);
        let mut book_left = book;
        let book_left = (left > 0 && book_left.set_count(left).is_ok()).then_some(book_left);
        entity
            .set_item_slot(host, tx, INPUT_SLOT as u32, Some(stripped))
            .map_err(|_| ())?;
        entity
            .set_item_slot(host, tx, BOOK_SLOT as u32, book_left)
            .map_err(|_| ())?;
        entity
            .set_item_slot(host, tx, OUTPUT_SLOT as u32, Some(produced))
            .map_err(|_| ())?;
        // Vanilla's `setChanged`: comparators and any open menu re-read the
        // slots, as upstream's own slot writes notify.
        let _ = entity.set_changed(host);
        Ok(true)
    })
    .unwrap_or(false)
}

/// End of a running server tick (`server_tick`): run the charges the clicks
/// queued. `commands().run_command` is the only surface that takes experience
/// points, and it refuses inside a player's hook, so the click cannot charge
/// itself.
pub(crate) fn run_charges(charges: &Charges, host: &mut dyn ServerHost) {
    for take in charges.take_pending() {
        let line = format!("xp add {} -{} points", take.name, take.points);
        if !host.commands().run_command(&line) {
            // Nothing was charged and nothing is granted; the promise goes.
            charges.release(take.uuid, take.points);
            continue;
        }
        let level = host
            .player(take.uuid)
            .and_then(|player| player.level(&*host));
        let applied =
            level.is_some_and(|level| finish(host, level, take.pos, &take.plan, &take.moved));
        if !applied && let Some(player) = host.player(take.uuid) {
            // The table or its output slot is gone: refund, so a paid-for take
            // never vanishes.
            let _ = player.give_experience_points(host, take.points);
        }
        charges.release(take.uuid, take.points);
    }
}

// ---------------------------------------------------------------------------
// The readout
// ---------------------------------------------------------------------------

/// The `/disenchanting_table info <pos>` line: one line with the input, book and
/// output slots and their enchantments, the cost, the player's points and
/// level, and whether the slots hold what a dis-enchant needs (the experience is
/// reported beside it, not folded in).
pub(crate) fn readout(
    host: &mut dyn ServerHost,
    pos: [i32; 3],
    player: Option<PlayerRef>,
) -> String {
    let level = player
        .and_then(|player| player.level(&*host))
        .or_else(|| host.level(host.dimension()));
    let Some(level) = level else {
        return "minecraft:air: not a dis-enchanting table".to_owned();
    };
    let block = level
        .block(&*host, pos)
        .map_or_else(|| "minecraft:air".to_owned(), |block| block.id(&*host));
    if !crate::runtime::is_table(&block) {
        return format!("{block}: not a dis-enchanting table");
    }
    let Some(slots) = host
        .block_entity(level, pos)
        .and_then(|entity| read_slots(&*host, entity))
    else {
        return format!("{block}: not a dis-enchanting table");
    };
    let cost = match charge(DEFAULT_SETTINGS, false) {
        Charge::Points(points) => u64::from(points),
        Charge::Levels(levels) => u64::from(levels),
        Charge::Nothing => 0,
    };
    let (points, experience_level) =
        player
            .and_then(|player| player.experience(&*host))
            .map_or((0, 0), |experience| {
                (
                    total_points(experience.level, experience.progress),
                    experience.level,
                )
            });
    let ready = slots.output.is_none()
        && slots.input.as_ref().is_some_and(|input| {
            let levels = levels_of(&*host, input);
            let view: Vec<(&str, u32)> = levels
                .iter()
                .map(|(id, level)| (id.as_str(), *level))
                .collect();
            crate::runtime::can_disenchant(&input.item().id(&*host), &view)
        })
        && slots
            .book
            .as_ref()
            .is_some_and(|book| is_book(&book.item().id(&*host)));
    let name = |stack: Option<&ItemStack>| {
        stack.map_or_else(|| "none".to_owned(), |stack| stack.item().id(&*host))
    };
    let enchantments = |stack: Option<&ItemStack>| {
        stack.map_or_else(
            || "none".to_owned(),
            |stack| enchant_list(&levels_of(&*host, stack)),
        )
    };
    format!(
        "{block}: input={} input_ench={} book={} output={} output_ench={} cost={cost} xp={points} \
         level={experience_level} ready={ready}",
        name(slots.input.as_ref()),
        enchantments(slots.input.as_ref()),
        name(slots.book.as_ref()),
        name(slots.output.as_ref()),
        enchantments(slots.output.as_ref()),
    )
}

/// `minecraft:sharpness:3,minecraft:unbreaking:2`, or `none` when there are no
/// enchantments.
fn enchant_list(levels: &[(String, u32)]) -> String {
    if levels.is_empty() {
        return "none".to_owned();
    }
    levels
        .iter()
        .map(|(id, level)| format!("{id}:{level}"))
        .collect::<Vec<_>>()
        .join(",")
}

// ---------------------------------------------------------------------------
// Shared reads
// ---------------------------------------------------------------------------

/// The table's three slots, cloned out of its block entity.
fn read_slots(host: &dyn Host, entity: BlockEntityRef) -> Option<TableSlots> {
    let slot = |index: usize| {
        entity
            .item_slot(host, index as u32)
            .ok()
            .flatten()
            .filter(|stack| stack.count() > 0)
    };
    Some(TableSlots {
        input: slot(INPUT_SLOT),
        book: slot(BOOK_SLOT),
        output: slot(OUTPUT_SLOT),
    })
}

/// The command's `<pos>` argument, resolved by the server's dispatcher.
pub(crate) fn block_pos(argument: Option<&CommandArgumentValue>) -> Option<[i32; 3]> {
    match argument? {
        CommandArgumentValue::BlockPos(pos) => Some(*pos),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Parts
// ---------------------------------------------------------------------------

/// The table's declaration parts: the block entity and its screen.
pub(crate) struct TablePart;

#[mod_api::horizon_part]
impl BlockPart for TablePart {}

#[mod_api::horizon_part]
impl BlockEntityPart for TablePart {
    /// `EntityBlock.newBlockEntity`: the slots are its whole state, so it saves
    /// nothing of its own and a reverted write inside a tick has nothing to lose.
    fn block_entity(
        &self,
        _level: LevelSide,
        _event: &mut BlockEntityEvent<'_>,
    ) -> Option<Compound> {
        Some(Compound::new())
    }

    /// The table's three-slot layout on an open menu.
    fn screen(&self, _host: &dyn Host, _event: &mut ScreenEvent<'_>) -> Option<Screen> {
        Some(menu())
    }

    /// The cost under the book slot.
    fn screen_draw(&self, _host: &dyn ClientHost, event: &mut ScreenDrawEvent<'_>) {
        let cost = match charge(DEFAULT_SETTINGS, false) {
            Charge::Points(points) => format!("{points} XP"),
            Charge::Levels(levels) => format!("{levels} levels"),
            Charge::Nothing => "free".to_owned(),
        };
        let [x, y] = event.origin;
        event
            .draw
            .text(&cost, [x + 8.0, y + 68.0], 0xFF40_4040, false);
    }
}

/// The click: a right click with a hand, either empty or full.
pub(crate) struct ClickPart {
    pub(crate) charges: Arc<Charges>,
}

#[mod_api::horizon_part]
impl BlockPart for ClickPart {
    fn use_item_on(
        &self,
        host: &mut dyn ServerHost,
        event: &mut UseItemOnEvent<'_>,
    ) -> Option<InteractionResult> {
        disenchant(&self.charges, host, event.click)
    }

    /// The same click with an empty hand: the SDK asks `use_without_item` for
    /// the main hand when the block's `use_item_on` asks for it, and an empty
    /// hand must dis-enchant too.
    fn use_without_item(
        &self,
        host: &mut dyn ServerHost,
        event: &mut UseWithoutItemEvent<'_>,
    ) -> Option<InteractionResult> {
        disenchant(&self.charges, host, event.click)
    }
}

/// The menu's slot rules.
pub(crate) struct MenuRulesPart;

#[mod_api::horizon_part]
impl MenuPart for MenuRulesPart {
    fn menu_slot_rules(&self, host: &dyn Host, event: &mut MenuSlotRulesEvent<'_>) {
        let item = event.stack.item().id(host);
        let levels = levels_of(host, event.stack);
        let view: Vec<(&str, u32)> = levels
            .iter()
            .map(|(id, level)| (id.as_str(), *level))
            .collect();
        slot_rule(
            event.menu,
            event.slot,
            &item,
            &view,
            &mut event.may_place,
            &mut event.max_stack,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::TABLE_BLOCK;

    #[test]
    fn the_table_declares_three_slots_and_its_menu() {
        let container = container();
        assert_eq!(container.slots, SLOT_COUNT, "input, book and output");
        assert_eq!(container.inputs, vec![INPUT_SLOT, BOOK_SLOT]);
        assert_eq!(
            container.outputs,
            vec![INPUT_SLOT, OUTPUT_SLOT],
            "upstream's canOutputToHopper"
        );
        assert_eq!(
            container.faces,
            std::array::from_fn(|_| vec![INPUT_SLOT, BOOK_SLOT, OUTPUT_SLOT]),
            "a hopper reaches all three from every face"
        );
        assert_eq!(container.menu.as_deref(), Some(TABLE_MENU));
    }

    #[test]
    fn the_declared_type_is_the_table_s_own_block() {
        assert_eq!(
            format!("disenchanting_table:{TABLE_TYPE}"),
            TABLE_BLOCK,
            "the type's blocks are bare names in the mod's namespace"
        );
    }

    #[test]
    fn the_screen_is_upstream_s_panel_with_three_slots() {
        let screen = menu();
        assert_eq!([screen.width, screen.height], [176.0, 166.0]);
        assert_eq!(
            screen.slots,
            vec![Some([27.0, 47.0]), Some([76.0, 47.0]), Some([134.0, 47.0])],
            "DisEnchantingMenu's input, book and output"
        );
        assert_eq!([screen.inventory_x, screen.inventory_y], [8.0, 84.0]);
        assert!(!screen.own_labels && !screen.own_background);
    }

    #[test]
    fn the_enchantment_component_follows_the_item() {
        assert_eq!(
            enchantment_component("minecraft:diamond_sword"),
            KEY_ENCHANTMENTS
        );
        assert_eq!(
            enchantment_component(ENCHANTED_BOOK_ITEM),
            KEY_STORED_ENCHANTMENTS,
            "crafting_enchantments keeps a book's own ones"
        );
    }

    #[test]
    fn the_output_component_is_the_direct_level_map_the_engine_writes() {
        let moved = vec![("minecraft:sharpness".to_owned(), 3)];
        let Tag::Compound(map) = levels_component(&moved) else {
            panic!("the level map is a compound");
        };
        assert_eq!(map.len(), 1);
        assert_eq!(
            Tag::Compound(map).get("minecraft:sharpness"),
            Some(&Tag::Int(3)),
            "26.3 writes the map directly, with no `levels` wrapper"
        );
    }

    #[test]
    fn an_item_gives_every_enchantment_and_keeps_none() {
        let plan = plan_take("minecraft:diamond_sword", &[("minecraft:sharpness", 3)])
            .expect("an enchanted item has a plan");
        let patch = Tag::Compound(strip_patch("minecraft:diamond_sword", &plan));
        assert_eq!(
            patch.get("!minecraft:enchantments"),
            Some(&Tag::Compound(Compound::new())),
            "the component is removed, not left empty"
        );
        assert_eq!(
            patch.get("!minecraft:repair_cost"),
            Some(&Tag::Compound(Compound::new())),
            "resets_repair_cost clears the prior-work penalty"
        );
    }

    #[test]
    fn a_book_keeps_its_first_enchantment() {
        let plan = plan_take(
            ENCHANTED_BOOK_ITEM,
            &[("minecraft:sharpness", 3), ("minecraft:unbreaking", 2)],
        )
        .expect("a two-enchantment book has a plan");
        let patch = Tag::Compound(strip_patch(ENCHANTED_BOOK_ITEM, &plan));
        let Some(Tag::Compound(kept)) = patch.get(KEY_STORED_ENCHANTMENTS) else {
            panic!("a split sets the book's own component");
        };
        assert_eq!(kept.len(), 1);
        assert_eq!(
            Tag::Compound(kept.clone()).get("minecraft:sharpness"),
            Some(&Tag::Int(3))
        );
    }

    #[test]
    fn the_readout_lists_enchantments_compactly() {
        let levels = vec![
            ("minecraft:sharpness".to_owned(), 3),
            ("minecraft:unbreaking".to_owned(), 2),
        ];
        assert_eq!(
            enchant_list(&levels),
            "minecraft:sharpness:3,minecraft:unbreaking:2"
        );
        assert_eq!(enchant_list(&[]), "none");
    }
}

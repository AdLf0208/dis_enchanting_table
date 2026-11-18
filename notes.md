# Dis-Enchanting Table → Horizon

Port of **Dis-Enchanting Table 5.0.2** (`Mods-For-Lupin/Dis-Enchanting-Redux`,
**Unlicense** per the jar's `META-INF/neoforge.mods.toml`), Minecraft Java
1.21.1 NeoForge, from the exact ATM10 jar
`disenchanting_table-merged-1.21.1-5.0.2.jar`, **306,197 B**, SHA-1
`c054a8bef63addff4d4a9520ec4c7a915c3fc1d1` (= the pack record, Modrinth version
`UzZnJKSR`), SHA-256 `bdb4f294…dc9c` (provenance:
[`../_shared/carry-work/disenchanting_table/SOURCE.md`](../_shared/carry-work/disenchanting_table/SOURCE.md),
jar kept there). The upstream source was read for behaviour only, at
`Mods-For-Lupin/Dis-Enchanting-Redux` commit `22b228f` (branch 1.21.6 — the
repo has **no 1.21.1 branch or tag**, see `FINDINGS.md`); the Rust is a
clean-room rewrite and the jar was not decompiled. Local-only until publishing
is approved (ARCHITECTURE.md §8).

> **Current state: version 0.2.0, one flat SDK `.hmod`** (mod API 0.2, 2026-11-18).
> [`## Migration to the SDK format`](#migration-to-the-sdk-format-version-020-2026-11-18)
> is the current shape — file map, helper→hook table, measured ladder and the traps.
> The `.hnmod` sections below (the AE2 embedded-content half, the staged
> `tools/horizon.py` ladder, the 0.1.0 e2e logs) describe the **retired** format and
> are kept as history and as the behaviour's in-game evidence.

## Approach (from the pin's `docs/modpacks.md`)

- **Class `content`, tier `machines`** (ledger row `disenchanting_table`;
  provenance `ports/_shared/source-map/atm10-sources.csv:144`). A machines mod
  is a block entity plus a container/menu/screen, so this is a native slice
  with a content half (the jar's block, models, textures, lang, recipe, loot
  and tags), not a data-only carry and not a plain block with a use handler.
- **Licence:** Unlicense → **rank 0**. The source map's
  `LicenseRef-All-Rights-Reserved` and the ledger's first copy were catalogue
  errors; the jar's own `license` field wins (ARCHITECTURE.md §8, "the jar wins
  over every catalogue"). Publishable with maintainer approval; local-only
  until then.
- **Why now:** the licence-first queue, core before add-ons, mechanics before
  UI. `precheck-mod.py disenchanting_table` reports `content · machines · core
  · P0`, licence `Unlicense` rank 0, verdict `DISPATCHABLE`.
- **Carry vs agent port:** the blockstate, both models, the three textures, the
  lang, the block's two tags and 26.3 re-spellings of the recipe and the loot
  table are carried from the jar (mechanical); the eligibility rule, the
  result plan, the slot rules, the experience cost model, the block entity, the
  container/menu/screen, the click and the readout are the agent port.

## Migration to the SDK format (version 0.2.0, 2026-11-18)

Horizon's mod-API merge (#16941, launcher ≥ 0.2.70, PR #17992) retired `.hnmod`, and the
registry closed every `.hnmod` submission — this port's was **#881** — with one reason.
**0.2.0 supersedes #881.** The unit is now **one flat `.hmod`**: content, native code, or both.
Pins for this migration: nightly **`0b14f8acfd97fa2c750e42795249dcc981178ca4`**, mixins
**`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`**, launcher 0.2.78, SDK slice cached at
`ports/_shared/sdk/0b14f8a/`. The recipe followed is `ports/imfast/notes.md` (first migration)
with `ports/charginggadgets/notes.md` for the block-entity/container/screen shape and
`ports/naturesaura/notes.md` for `use_item_on` + commands.

**What changed for `api = "0.2"`, in one line:** the nine per-hook injection helpers became
three typed `Part`s and a `Registrar` declaration, `perform_command`'s free-form words became a
typed `<pos:block_pos>` argument, and the AE2 embedded-content helper disappeared because the
package now carries its own `assets/` and `data/`. Every rule, number and line of readout text
is unchanged: `src/runtime.rs` did not change by a byte.

### The line that makes this a migration and not a rewrite

`src/runtime.rs` is `mixins/disenchanting_table/runtime.rs` **byte-identical** (SHA-256
`a43b472b3070fbe3a56334433515f1c548790f5105958f72aa163596e2797738`; `cmp` exit 0, and
`unzip -p disenchanting_table.hmod src/runtime.rs | sha256sum` is the same again). Its **16
tests were green before any glue existed** (`sdk-runtime-first.log`: `running 16 tests … 16
passed; 0 failed`, with `src/lib.rs` holding only `pub mod runtime;`), and they still pass with
the glue (**24 passed**). Evidence: `ports/_shared/evidence/disenchanting_table/sdk-*.log`.

> The 0.1.0 ladder and its e2e logs (`e2e-default.log`, `e2e-embedded.log`,
> `e2e-embedded-selftest.log`, `e2e-modless.log`, `e2e-repair.log`, `package-0.1.0.log`,
> `runtime-*.log`, `embed-ladder-*.log`) are **historical**: they describe the retired `.hnmod`
> and the staged `tools/horizon.py` ladder, which no longer exist. They remain the in-game
> evidence for a `runtime.rs` and a scenario that did not change.

### File map

| 0.2.0 path | What it is |
|---|---|
| `horizon-mod.toml` | the only manifest and the only identity: `id`, `version = "0.2.0"`, `api = "0.2"`, `side = "both"`, `capabilities = ["registries"]`, `license`/`homepage`/`issues`, and `[native]` game 26.3 + the two 40-hex commits |
| `Cargo.toml` | one crate, edition 2024, its only dependency `horizon-mod-api` from `.horizon/sdk/` |
| `src/lib.rs` | the one `#[mod_api::horizon_mod] impl Hooks`: `register`, `server_tick`, `register_commands`, `run_mod_command` |
| `src/runtime.rs` | the pure engine, **byte-identical** (16 tests) |
| `src/table.rs` | the glue: `TablePart` (a `BlockEntityPart`), `ClickPart` (a `BlockPart`), `MenuRulesPart` (a `MenuPart`), `declare`/`container`/`menu`, `Charges`, `disenchant`, `finish`, `run_charges`, `readout`, and the tests of the declarations and the patches |
| `assets/**`, `data/**` | the retired `content/assets` and `content/data`, moved up **unchanged** (15 files, diffed file by file against the retired package: `sdk-carried-content.log`) |
| `LICENSE`, `LICENSE-NOTICE.md`, `README.md` | inside the package |
| `tools/e2e.sh` | the same scenario, rewritten for a launcher-built game (`GAME=`, `CONTROL_GAME=`) |
| `tools/carry_content.py` | now writes `assets/` + `data/` at the package root and writes no manifest |
| `notes.md`, `FINDINGS.md` | the record (not packed) |

**Moved to `ports/_shared/evidence/disenchanting_table/retired-hnmod-0.1.0/`** (moved, not
deleted): `mixins/` (the `horizon-mixin.toml`, `runtime.rs`, `support.rs`, nine hook helpers and
the embedded content archive), the `disenchanting_table.hnmod` (43,796 B), `content/`
(`horizon-mod.toml` and the 0.1.0 `disenchanting_table.hmod`), `compatibility.json`,
`rust-toolchain.toml`, the 0.1.0 `Cargo.toml`/`Cargo.lock`/`.gitignore`, the port-local
`AGENTS.md`, `PLAN.md`, `VALIDATION.md`, and the retired tools (`horizon.py`, `package.py`,
`package.sh`, `add-notices.py`, `e2e-modless.sh`, `e2e-embedded-legs.sh`).

### Helper → shape table

| 0.1.0 injection | 0.2.0 shape | kept / gap |
|---|---|---|
| `block_entity` (API 15) | `BlockEntityPart::block_entity` on `TablePart`, attached to the type built by `block_entity_type("disenchanting_table").blocks(&["disenchanting_table"]).build()` | **kept** — returns `Some(Compound::new())`: the three slots are the whole state |
| `block_entity_container` (API 16/17) | `Registrar::block_container(&block, ModContainer)` in `Hooks::register` | **kept** — the same three slots, the same face/input/output lists |
| `screen` (API 19) | `BlockEntityPart::screen` | **kept** — 176 × 166, slots at (27, 47)/(76, 47)/(134, 47) |
| `screen_draw` (API 19) | `BlockEntityPart::screen_draw` | **kept** — the cost under the book slot, same text and colour |
| `right_click_block` (API 21) | `BlockPart::use_item_on` **and** `use_without_item` on `ClickPart` | **kept, two methods**: the SDK splits the hands, and an empty hand must dis-enchant too |
| `server_tick` (API 4) | `Hooks::server_tick` (`ServerTickPhase::ServerTickEnd { world_running }`) | **kept** — the deferred points charge |
| `register_commands` (API 8) | `Hooks::register_commands`, pushing `Command::new("disenchanting_table info <pos:block_pos>")` | **kept, typed**: the same player-visible usage, parsed by the dispatcher instead of by hand |
| `perform_command` (API 8) | `Hooks::run_mod_command`, reading `CommandArgumentValue::BlockPos` | **kept**: the readout line is byte-identical to 0.1.0 |
| `menu_slot_rules` (API 130) | `MenuPart::menu_slot_rules` on `MenuRulesPart`, attached with `Registrar::menu_part(TABLE_MENU, …)` | **kept** — the same pure `slot_rule` |
| `embedded-content` (raw `content_root` target) | — | **gone by design, not a gap**: one `.hmod` carries `assets/`, `data/` and `src/`, so the helper that wrote an embedded archive out has nothing left to work around |

Host calls, 0.1.0 → 0.2.0:

| 0.1.0 (`Game`) | 0.2.0 |
|---|---|
| `game.native_block(pos)` | `level.block(&*host, pos)` → `BlockKey::id(&*host)` |
| `game.world.block_entities.get(&(x,y,z))` + `entity.items[i]` | `host.block_entity(level, pos)` → `BlockEntityRef::item_slot`/`set_item_slot`/`data` |
| `game.native_with_mod_block_entity(pos, f)` (read + write in one call) | one `host.transaction(|host, tx| …)` reading `item_slot` and writing `set_item_slot` per slot — atomic, which 0.1.0's sequential writes were not |
| `game.native_acting_player()`, `player.xp_level`, `player.xp_progress`, `player.game_mode` | `host.player(click.player)` → `PlayerRef` handle: `player.experience(&*host)`, `player.is_creative(&*host)`, `player.name(&*host)`, `player.level(&*host)` |
| `game.native_give_levels(uuid, n)` | `player.give_levels(host, n)` |
| `game.native_give_experience(uuid, points)` (refund) | `player.give_experience_points(host, points)` |
| `game.native_run_command(line)` | `host.commands().run_command(&line)` |
| `game.native_feedback(Text)` | `host.commands().feedback(Text)` |
| `game.native_action_bar(uuid, Text)` | `player.action_bar(host, Text)` |
| `stack.crafting_enchantments()` / `set_crafting_enchantments(Levels)` | the `minecraft:enchantments` / `minecraft:stored_enchantments` component: read through `ItemResource::value(host, &ComponentId)`, written with `ItemStack::apply_components` |
| `input.data_mut().repair_cost = 0; input.tidy()` | the `!minecraft:repair_cost` removal marker in the same patch |
| `static PENDING`/`static RESERVED` in `support.rs` | `Charges` behind one `Arc`, owned by the mod struct and shared with `ClickPart` (`Hooks: Any + Send + Sync`) |
| `horizon_client::native_data!` + `current_dir()` cache write | — (no embedded content) |

### Build and check (the SDK ladder, measured)

`build-slot.sh` speaks the retired `tools/horizon.py`, so it is used as the **slot guard**, not
as the driver; every step ran with `BUILD_LOCAL=1` (the remote box's cargo cache lacks
`unicode_names2` at this pin) and `CARGO_INCREMENTAL=0`, from the port directory.

| Check | Result | Evidence |
|---|---|---|
| runtime carried over | `cmp` exit 0, SHA-256 `a43b472b…7738`, packaged entry identical | `sdk-build.log`, §above |
| runtime tests before glue | **16 passed**, 0 failed (`src/lib.rs` = `pub mod runtime;` only) | `sdk-runtime-first.log` |
| the glue's tests can fail | 7 of 8 new tests `todo!()` → **17 passed, 7 failed**, each `not yet implemented` | `sdk-glue-red.log` |
| all tests | **24 passed**, 0 failed | `sdk-test.log` |
| lints | `cargo clippy --all-targets -- -D warnings` exit 0, **0 warnings** | `sdk-clippy.log` |
| formatting | `cargo fmt --check` clean (no `--all`) | `sdk-fmt.log` |
| carried content | 15 files, every one byte-identical to the retired content package | `sdk-carried-content.log` |
| package | `cargo horizon-mod build` → **53,938 B**, SHA-256 `b4ebc477746b44a5a849d4f0d604da65d7510d230e8762ff4393f3b37752efc1`, **24 entries**, no `Cargo.lock` | `sdk-build.log` |
| determinism | two consecutive builds **byte-identical**, same SHA-256 | `sdk-build-second.log` |
| registry check | `disenchanting_table 0.2.0 (native-mod, format native-1/0b14f8acfd97fa2c750e42795249dcc981178ca4/e59da9094f0c2fdda6c627fe9eff22dec6a906f0, for 26.3)`, 53,938 bytes, unsigned, **EXIT 0** | `sdk-check.log` |
| what it uses | **350 surface entries** — 167 type, 82 handle, 34 method, 32 hook, 26 trait, 5 part, 2 macro, 1 capability, 1 fn; the ten hooks are `block_entity`, `menu_slot_rules`, `register_commands`, `run_mod_command`, `screen`, `screen_draw`, `server_tick`, `use_item_on`, `use_without_item` (+ `blocks`) | `sdk-uses.json`, `sdk-uses-summary.log` |
| fits the pinned build | `0b14f8a made for`, **EXIT 0** | `sdk-fits.log` |
| skills line | `python3 ports/tools/check-skills.py --notes ports/disenchanting_table --strict` | §Skills below |

The nine stamped hooks and the one target injection are therefore **9 mapped, 0 recorded as
gaps**; the tenth (the embedded-content target) has no successor because the format removed the
reason it existed.

### Traps this migration hit (the next port must know)

1. **A player's reads live on the `PlayerRef` handle, not on the host.** `host.player(uuid)` is
   on the host, but `host.player_level(…)`, `host.player_experience(…)`, `host.player_is_creative(…)`
   and `host.player_name(…)` do **not** resolve on `&mut dyn ServerHost` (seven E0599 errors):
   the shape is `player.level(&*host)`, `player.experience(&*host)`, `player.is_creative(&*host)`,
   `player.name(&*host)`. The same holds for `PlayerRef::action_bar`, `give_levels` and
   `give_experience_points`.
2. **One click is two methods.** `use_item_on` is the held-item answer and `use_without_item`
   the empty-hand one; the retired bundle's `right_click_block` covered both, so the Part
   implements both with one body (as `naturesaura` does). Answering `None` lets the template
   (the barrel) open the menu.
3. **`Some(InteractionResult::Consume)` ends the click; 0.1.0 did not.** `canceled = true` with
   the default `Pass` still ran the *held item's* own use after the table's; the SDK answer ends
   the click, so a successful take no longer also eats the food in hand. That is the recorded
   intent ("consume the click"), and it is the one observable behaviour that changed.
4. **The three slot writes go in one `host.transaction(…)`.** 0.1.0 charged, then wrote the
   input, the book and the output one after another; a refused later write left a spent book.
   Now any refusal aborts the transaction and every slot goes back — the review repair of
   PR #545 ("payment is not atomic with granting the result") is structural here, not a guard.
5. **A mod cannot write `ItemEnchantments`.** It is only handed to `get_enchantment_level`
   listeners; the port reads the *component* (`ItemResource::value(host, &ComponentId)`) and
   writes it with `apply_components`. 26.3's value is the **direct** `{id: level}` map (the
   engine's `levels_tag`, `horizon-items/src/item_data.rs:477`); a pre-1.21.5 `{levels: {…}}`
   wrapper is still read, as `read_levels` does, so an older item dis-enchants too. Stripping
   uses the `!minecraft:enchantments` removal marker, because `ItemStack::remove` wants a typed
   `Component`.
6. **`player_set_experience_points` is not "take points".** It sets the *progress* bar and keeps
   the level and total; `give_experience_points` only gives; `give_levels` takes levels only. So
   the points charge stays deferred to `server_tick` + `commands().run_command("xp add … points")`,
   exactly as 0.1.0 needed — the API gap the port's notes already recorded is still open.
7. **`PlayerExperience` is `#[non_exhaustive]`.** Read `level`/`progress` out of the value;
   constructing one in a `map_or` default does not compile (E0639).
8. **A readout built from handle reads must be formatted in one place.** The three slots' names
   and enchantment lists are resolved before the `format!`, because `stack.item().id(host)` owns
   its `String`; a helper returning `&str` from it cannot exist.
9. **The panel's slot indices in `menu_slot_rules` are the backing container's.** 0..2 are the
   table's slots and 3.. the player's inventory's, so the pure `slot_rule`'s `INPUT_SLOT`/
   `BOOK_SLOT`/`OUTPUT_SLOT` checks stay exactly as they were — and a rule attached to *our*
   menu type can never touch another mod's menu.
10. **`cargo test … | tee` reports `tee`'s status.** The first forwarded run said `exit 0` on a
    compile failure. `set -o pipefail` inside the wrapped command (`bash -c 'set -o pipefail; …'`)
    is required for every logged step.
11. **`build-slot.sh run` forwards to the far box, which `cd`s into the synced port.** The
    brief's `bash -c 'cd ports/<id> && …'` shape therefore fails when it forwards (`cd: no such
    directory`). `[ -f Cargo.toml ] || cd ports/<id>` before the command works in both places.
    The far box also lacks `unicode_names2` in its cargo cache at this pin, so `--offline` fails
    there (`no matching package named unicode_names2`): use `BUILD_LOCAL=1` for SDK packages.
12. **`license`/`homepage`/`issues` are *not* stripped at this tool build.** `imfast`'s trap 7
    says `build` removes them; measured here, the `horizon-mod.toml` inside the package still
    carries all three (and the comments). The artefact wins: the manifest packed is the one
    written, and the hash is stable across two builds.
13. **The retired tree must be moved, not just deleted.** `cargo horizon-mod build` reports
    `unexpected package path: mixins/disenchanting_table/runtime.rs` for a leftover helper, so
    `mixins/`, the `.hnmod`, the old `content/` manifest and the retired tools went to
    `ports/_shared/evidence/disenchanting_table/retired-hnmod-0.1.0/` before the first build —
    and `runtime.rs` was copied to `src/` first, then `cmp`-ed.
14. **No in-game run at 0.2.0.** `horizon-launcher` is not installed here, so a game carrying an
    SDK package cannot be built (an SDK package is compiled **into** the client by the launcher;
    there is no staging path for it). `tools/e2e.sh` was rewritten for `GAME=` (a launcher build
    with the package) and `CONTROL_GAME=` (one without it, the control) and was **not run**; it
    refuses to start without `GAME`. The first thing the in-game run must confirm is the one
    claim this migration cannot test offline: that the content block gets its block entity from
    the *declaration* (`block_entity_type("disenchanting_table").blocks(&["disenchanting_table"])`),
    without which every readout answers "not a dis-enchanting table".

### Gaps after the migration

The behaviour gaps of slice 1 (below, unchanged) stand: the config file, the automatic mode, the
sound/particles, the jar's GUI texture, the hopper-path `mayPlace`, and the client halves of the
merged jar. The migration adds three record-level ones:

1. **No in-game run at 0.2.0** (trap 14): `tools/e2e.sh` is written for a launcher build and
   unrun; the 0.1.0 logs are the behaviour evidence for a `runtime.rs` that did not change.
2. **The old `CONTROL=1` embedded-file leg is gone**: one package is the block, so the control is
   now a build *without* the package (`CONTROL_GAME=`), and the readout-versus-unknown-command
   assertion is the tolerant "no `: input=` table line" rather than a "not a dis-enchanting table"
   line, because a game without the package has no command to answer with.
3. **The points charge still reads the player by name** in the console line (`xp add <name> -25
   points`), which is what 0.1.0 did; a player who renames during the one tick between the click
   and the charge would be refused, and the promise is released when the command fails.

## Slice 1 — the table

- `disenchanting_table:disenchanting_table` on the `minecraft:barrel` template
  (it has a block entity and its use path opens a menu), with the jar's
  `ofFullCopy(ENCHANTING_TABLE)` behaviour: hardness 5.0, blast resistance
  1200, **light 7**, stone sounds, and the jar's 12-pixel shape
  (`Block.box(0,0,0,16,12,16)`), `occludes: false`. The jar's own data adds the
  block to `minecraft:mineable/pickaxe` and `minecraft:needs_stone_tool`, so
  the item drops only to a stone pickaxe.
- A block entity (API 15) that **saves nothing of its own**: its three slots are
  the whole state, so the pin's reverted-block-entity-write caveat
  (`moderndynamics` item 77) cannot bite — there is no field to lose.
- Three slots — input `0`, book `1`, output `2` — and the content menu
  `disenchanting_table:disenchanting_table` (API 16/17, 3 slots, output slot 2
  a `results` entry so players only take). A hopper may fill the input and book
  slots and take from the input and output slots (upstream's
  `canOutputToHopper`).
- `native_menu_slot_rules` (API 130): the input takes only an item that may be
  dis-enchanted, the book slot only `minecraft:book`, the output nothing.
- **The dis-enchant** on a right click of the table (API 21) when an eligible
  input, a book and an empty output slot are present: strip the input (or, for
  an enchanted book, keep its first enchantment), spend one book, put a new
  `minecraft:enchanted_book` carrying the moved enchantments in the output
  slot, and charge. Every refusal (no book, no experience, output occupied)
  leaves all three slots as they were.
- **Eligibility** (upstream's `canDisenchant`): any enchantments on an item; an
  **enchanted book needs at least two**, or nothing would be left.
- **The cost** is upstream's default **25 experience points**; creative pays
  nothing (vanilla clamps the points at 0 anyway). Horizon can *give* points
  and *take levels* but not take points, so the charge is queued and run on the
  next server tick as the console's `xp add <player> -25 points`
  (`native_run_command`, API 21, refused inside a player's hook).
- `/disenchanting_table info <x> <y> <z>`: the three slots with their
  enchantments, the cost, the acting player's points and level and whether the
  slots are ready (the experience is reported beside it, not folded in; the
  harness reads it, upstream has no command).
- The screen (API 19) lays the slots out at upstream's 27 / 76 / 134, y 47, in a
  176×166 screen and draws the cost under the book slot.

### Behaviour read from upstream (and where this port differs)

| Area | Upstream (`DisEnchantingMenu`) | This port |
|---|---|---|
| result | computed into a result slot; `mayPickup` gated by experience | computed and applied by one right click; a menu has no take hook here |
| take | `onTake`: strip, spend the book, charge | the same steps in one write (synchronous; the charge is one tick later) |
| book input | keeps the **first** enchantment, gives the rest | same |
| item input | all enchantments move, the item stays | same |
| repair cost | cleared on a stripped item (`resets_repair_cost`) | same |
| slots | `mayPlace`: input eligible, book only, output none | same, through `native_menu_slot_rules` |
| experience | points or levels, configurable | points by default; a level charge is implemented; the config is baked |
| auto mode | `automatic_disenchanting` every 10 ticks | not ported (default off); gap |
| sound/particles | `levelEvent 1503` + a sound | not ported; gap |
| screen | the jar's GUI texture | Horizon's plain panel + a drawn cost; the texture is carried but unused |

### Deliberate deviations

1. **The action is a right click, not a result-slot take.** The pin has no
   `mayPickup`/`onTake` hook and no per-slot take validity, so upstream's result
   slot becomes: compute + apply on the click, leave the produced book in the
   `results` output slot, and let the menu's normal click take it. A click while
   the output slot is occupied, or with an ineligible input or no book, falls
   through to the template's use path and opens the menu.
2. **The experience charge runs one tick later** (`server_tick` → console
   `xp add … points`). It is the only surface that takes points; the e2e asserts
   the exact 25-point drop, and a second click without a new book cannot charge
   twice because it produces no result.
3. **The config is baked** (upstream's defaults: `resets_repair_cost = true`,
   `requires_experience = true`, `uses_points = true`, `experience_cost = 25`,
   `automatic_disenchanting = false`). The pin's `native_config_dir` exists but
   no config API does; a `disenchanting_table-server.toml` is a later slice.
4. **`ExperienceHelper`'s thresholds are not copied.** Upstream's
   `arithmeticSeriesSum` drops the half of `n / 2` (level 15 → 294 points where
   vanilla has 315). The port uses vanilla's thresholds, which is what the
   player's own `experienceLevel`/`experienceProgress` mean.
5. **The level path charges through the engine.** Upstream's
   `deductExperienceLevels` writes the field directly; the port calls
   `Player.giveExperienceLevels` (`native_give_levels`).
6. **The command is added** (`/disenchanting_table info`) for the harness, as
   `botanypots` did; upstream has none.

## Slice boundary and gaps

**In:** the table block, item, carried content, block entity, three-slot
container and menu, slot rules, screen and cost readout, the right-click
dis-enchant for both input kinds, the 25-point charge, the deferred level path,
`/disenchanting_table info`, the `.hmod`, the `.hnmod`.

**Out (later slices / gaps):**

1. the **config file** (`automatic_disenchanting`, `automatic_disenchanting_ticks`,
   `resets_repair_cost`, `requires_experience`, `uses_points`,
   `experience_cost`) — baked defaults only;
2. the **automatic mode** (a 10-tick auto-dis-enchant) and its data slot;
3. the **dis-enchant sound** and the block's portal particles
   (`animateTick`), and the jar's GUI texture on the screen;
4. the **Fabric/Forge halves** of the merged jar (their `ClientConfig`,
   title-screen mixins and network packets are not ported: the port needs no
   client→server packet);
5. **per-item `mayPlace` in the hopper path**: `native_container_accepts` is
   **pending/unstamped**, so a hopper can put any item into the input or book
   slot; the click then refuses it and nothing is lost (the slot rule covers
   players' clicks only);
6. the **creative tab icon layout** beyond the one item, JEI/EMI/REI/Jade and
   Patchouli integration (upstream has none).

**Not asserted in this slice's e2e:** the screen's drawn cost text (the slot
layout is asserted through `ctl screen`; the text is not a widget), the hopper
paths, the GUI texture, and the config.

## Embedded content (AE2 pattern) — one `.hnmod` for both halves (retired at 0.2.0)

The registry stores **one file per project** (playbook §1.0), so a port with both
halves could not be listed faithfully: an `.hnmod` is code-only and an `.hmod`
is content. This bundle now carries its content, exactly as the merged AE2 entry
and `charginggadgets` (PR #526) do:

- **`mixins/disenchanting_table/data/disenchanting_table-content.hmod`** is the
  built `content/disenchanting_table.hmod`, byte for byte (25,628 B,
  `ad62edaf…d567`). `tools/package.sh` now builds and checks the content first,
  then syncs it into `data/`; `tools/package.py` packs `data/**` as
  `mixins/<id>/data/…` (the bundle's top-level `.rs`/`.toml` plus everything
  under `data/`), keeps the fixed zip timestamps, and **refuses** a
  `data/<id>-content.hmod` that is missing or not byte-identical to
  `content/<id>.hmod`, so a rebuilt content mod can never ship stale inside the
  bundle. Two consecutive packagings give the same SHA-256, and the guard was
  exercised: appending one byte to the `data/` copy made `tools/package.py`
  refuse (`exit 1`, `.hnmod` untouched), and restoring the file reproduced the
  same SHA-256.
- **`mixins/disenchanting_table/embedded_content.rs`** with the `embedded-content`
  `[[injection]]` (`target = "horizon_client::game::mods::content_root"`,
  `kind = "HEAD"`, `cancellable = true`, bindings `root`/`enabled`/`dev`)
  compiles the file in with `native_data!` (API 41), writes it to
  `<game folder>/.horizon-cache/disenchanting_table-embedded-content.hmod` when
  the copy on disk differs, appends it to the dev-mod list and returns its
  `content_root`. The helper calls `content_root` once more and stops at its own
  `dev.contains(&file)` guard, so the second call runs the vanilla body — the
  same one-level recursion AE2 uses. An unwritable cache logs a warning and
  `Continue`s (the code still loads; the block does not).
- **Where the file goes matters.** The game makes `--game-dir` its working
  directory (`crates/horizon-client/src/game.rs`), so `current_dir()` inside the
  helper is the **game folder**, not the launcher's cwd. The e2e's control blocks
  exactly that path (a directory where the file goes), which leaves the game's own
  `.horizon-cache` (content extraction, `mod-content/`) working.

Verified in game with **only the bundle** (`EMBEDDED=1`, no `--dev-mod`): the
whole scenario passes — the table places, the sword gives up `sharpness:3`, the
book is spent, the produced book reaches the inventory through the output slot,
the charge is exactly 25 points, and all four controls hold. The helper's file is
byte-identical to `content/disenchanting_table.hmod`, and the content is loaded
from it (the game log names `<game>/.horizon-cache/mod-content/…`). The control
(`CONTROL=1`) launches
the same binary with that one path blocked: `give` answers `Unknown item`, the
`setblock` answers `Unknown block type` and leaves `minecraft:air`, and the
readout says `minecraft:air: not a dis-enchanting table` — so the PASS is not
vacuous. `SELFTEST=1 EMBEDDED=1 CONTROL=0` still exits **1**.

The port is therefore a **single listable artifact** (`disenchanting_table.hnmod`,
39,206 B, 15 entries); `content/disenchanting_table.hmod` remains the built
content source of truth and the dev-mod form used by the default e2e. Publishing
still needs the maintainer's approval (ARCHITECTURE.md §8).

Two harness consequences followed from the embedding and are now in the scripts:

- The **default** e2e (`EMBEDDED=0`) still passes `--dev-mod content/`; because
  the bundle also embeds the same content, that run loads the same id twice (the
  pin tolerates it, as it does on `charginggadgets`). `e2e-default.log` from the
  previous slice is unchanged evidence; `e2e-embedded.log` is the sharper proof.
- `tools/e2e-modless.sh` used to prove "no `--dev-mod` ⇒ unknown ids", which is no
  longer true (the binary carries its own content). It now blocks the embedded
  file itself — a directory at
  `<game folder>/.horizon-cache/disenchanting_table-embedded-content.hmod` — so
  leg 1 still proves the ids are unknown without any content, and leg 2 proves the
  **packaged** `content/disenchanting_table.hmod` registers the block and item on
  its own.

### In-game transcript (quoted from `e2e-embedded.log`)

```
== control: the same game with no embedded content file ==
  give: give @s disenchanting_table:disenchanting_table 1<--[HERE]|Unknown item 'disenchanting_table:disenchanting_table'|…
  setblock: …|Unknown block type 'disenchanting_table:disenchanting_table'|…
  readout: minecraft:air: not a dis-enchanting table|…
== launch (offscreen renderer, bundle only, no --dev-mod) ==
  embedded content: 25624 B at /tmp/opencode/disenchanting_table-home.jS1fRI/game/.horizon-cache/disenchanting_table-embedded-content.hmod, identical to content/disenchanting_table.hmod
readout: disenchanting_table:disenchanting_table: input=minecraft:diamond_sword input_ench=minecraft:sharpness:3 book=minecraft:book output=none output_ench=none cost=25 xp=135 level=9 ready=true
experience before: 135
readout: disenchanting_table:disenchanting_table: input=minecraft:diamond_sword input_ench=none book=none output=minecraft:enchanted_book output_ench=minecraft:sharpness:3 cost=25 xp=110 level=7 ready=false
experience after: 110 (cost 25)
enchanted books held: 1
PASS: the table dis-enchants a sharpness-3 sword onto a book for exactly 25 points, spends the book and keeps the item; an unenchanted item, a missing book, no experience and a vanilla enchanting table all dis-enchant nothing and lose nothing
E2E EXIT:0
```

## Verification (the 0.1.0 `.hnmod` ladder — historical, superseded by the migration section)

Pin **`1d63e0a5a37a7a708eb0b32b49bf0dfeb28ed8a9`**, horizon-mixins
**`5a64af938ec86b00d53d16a408aabc0038fb3e57`** (schema 6). Every build, test and
e2e ran through `ports/tools/build-slot.sh` (`CARGO_INCREMENTAL=0`, no bare
cargo, no `--target-dir`); the in-game runs used the pin-matched shared
`horizon-ctl` and a port-built `horizon-client`, built and run in **one** lease
each. Evidence: [`../_shared/evidence/disenchanting_table/`](../_shared/evidence/disenchanting_table/).

| Step | Command (through the slot runner) | Result |
|---|---|---|
| runtime RED | `cargo test --locked --offline` (engine only, no impl) | `cargo_exit=101`, **58** `error[E…]` — `runtime-red.log` |
| runtime GREEN | the same | **15 passed**, 0 failed — `runtime-test.log` |
| mutation check | `can_disenchant`'s book rule `>= 2` → `>= 1`, then restored | **3 tests FAILED**, exit 101, then 15 pass — `mutation-check.log` |
| fmt + clippy | `cargo fmt --all --check`; `cargo clippy --all-targets --locked --offline -- -D warnings` | pass, **0 warnings** — `clippy.log` |
| content carry | `python3 tools/carry_content.py --check` | **16 files, 0 pending**, jar SHA-1 `c054a8be…` |
| stage | `python3 tools/horizon.py stage --horizon …/horizon-1d63e0a` | `status: ok`, **3 modules, 10 injections** (the embedded-content target included) — `embed-ladder-stage.log` |
| check | `python3 tools/horizon.py check …` | `cargo_exit: 0`, **0 warnings** — `embed-ladder-horizon-check.log` |
| embedded tests | `python3 tools/horizon.py test …` (`--lib game::disenchanting_table`) | **17 passed**, 0 failed, 1240 filtered — `embed-ladder-horizon-test.log` |
| fast build | `python3 tools/horizon.py build …` | `cargo_exit: 0` — `embed-ladder-horizon-build.log` |
| content `.hmod` | `tools/package.sh` (`cargo-horizon-mod build` + `check`) | **25,628 B**, SHA-256 `ad62edaf…d567`, **16 entries**, `disenchanting_table 0.1.0 (mod, API 0.2, both)`, "no code: content only" — `embed-ladder-package.log`, `package-0.1.0.log` |
| native `.hnmod` (embeds the content) | `tools/package.sh` (`tools/package.py`) | **39,202 B**, SHA-256 `9118b6c9…da42`, **15 entries**, including `mixins/disenchanting_table/data/disenchanting_table-content.hmod`; `horizon-native.json` pins horizon `1d63e0a5…`, mixins `5a64af93…`, game 26.3 — `embed-ladder-package.log`, `package-0.1.0.log`; two consecutive packagings give the same SHA-256 |
| `cargo horizon-mod check … --kind native-mod` | the same script | exit 0, `disenchanting_table 0.1.0 (native-mod, format native-1/1d63e0a5…/5a64af93…, for 26.3)`, 39,202 B, SHA-256 `9118b6c9…da42` — `embed-ladder-check-native.log`, `package-0.1.0.log` |
| e2e, **only the bundle** (`EMBEDDED=1`, no `--dev-mod`) + its control | `tools/e2e-embedded-legs.sh` through `build-slot.sh e2e` | **`PASS` + `E2E EXIT:0`**; control: ids unknown; content file 25,628 B identical — `e2e-embedded.log` |
| e2e, embedded selftest | `EMBEDDED=1 CONTROL=0 SELFTEST=1 …` | **`FAIL` + `E2E EXIT:1`** at `SELFTEST: the sword was stripped, so the check is alive` — `e2e-embedded-selftest.log` |
| e2e (no content, then the packaged `.hmod`) | `tools/e2e-modless.sh` | **`PASS` + `E2E EXIT:0`** — `e2e-modless.log` |
| log checker | `python3 ../tools/check-e2e-logs.py disenchanting_table --strict` | `logs=5 ok=5`, exit 0 |

### Port version (reviewer rule, playbook §1.7)

The registry version is the **port's** version, not the original mod's: the
manifest carried `5.0.2`, the reviewer's flag on earlier submissions.
`5.0.2 → 0.1.0` is the only change (no id, hook, content or behaviour), made in
`content/horizon-mod.toml` and `mixins/disenchanting_table/horizon-mixin.toml`
`[mod]`; `tools/package.sh` derives the bundle name from the manifest, so it
needed no other edit. Two consecutive packagings reproduced the same bytes
(`ad62edaf…d567` / `9118b6c9…da42`), both `cargo horizon-mod check` kinds exit
0, and the embedded legs were re-run because `tools/e2e.sh` compares the written
cache file with `content/disenchanting_table.hmod`: `PASS` + `E2E EXIT:0`, the
selftest `FAIL` + `E2E EXIT:1`, the modless leg `PASS` + `E2E EXIT:0`. Evidence:
`../_shared/evidence/disenchanting_table/{package-0.1.0,e2e-embedded,e2e-embedded-selftest,e2e-modless}.log`.

### E2E transcript (quoted from `e2e-default.log`, 0.1.0 — historical)

```
readout: disenchanting_table:disenchanting_table: input=none input_ench=none book=none output=none output_ench=none cost=25 xp=135 level=9 ready=false
readout: disenchanting_table:disenchanting_table: input=minecraft:diamond_sword input_ench=minecraft:sharpness:3 book=minecraft:book output=none output_ench=none cost=25 xp=135 level=9 ready=true
experience before: 135
readout: disenchanting_table:disenchanting_table: input=minecraft:diamond_sword input_ench=none book=none output=minecraft:enchanted_book output_ench=minecraft:sharpness:3 cost=25 xp=110 level=7 ready=false
experience after: 110 (cost 25)
enchanted books held: 1
readout: disenchanting_table:disenchanting_table: input=none input_ench=none book=none output=none output_ench=none cost=25 xp=110 level=7 ready=false
after the click: disenchanting_table:disenchanting_table: input=none input_ench=none book=none output=none output_ench=none cost=25 xp=110 level=7 ready=false
readout: disenchanting_table:disenchanting_table: input=minecraft:diamond_sword input_ench=minecraft:sharpness:3 book=none output=none output_ench=none cost=25 xp=110 level=7 ready=false
after the click: disenchanting_table:disenchanting_table: input=minecraft:diamond_sword input_ench=minecraft:sharpness:3 book=none output=none output_ench=none cost=25 xp=110 level=7 ready=false
readout at 0 points: disenchanting_table:disenchanting_table: input=minecraft:diamond_sword input_ench=minecraft:sharpness:3 book=minecraft:book output=none output_ench=none cost=25 xp=0 level=0 ready=true
after the click: disenchanting_table:disenchanting_table: input=minecraft:diamond_sword input_ench=minecraft:sharpness:3 book=minecraft:book output=none output_ench=none cost=25 xp=0 level=0 ready=true
vanilla enchanting table screen: station
the dis-enchanting table is unchanged: disenchanting_table:disenchanting_table: input=minecraft:diamond_sword input_ench=minecraft:sharpness:3 book=minecraft:book output=none output_ench=none cost=25 xp=135 level=9 ready=true
PASS: the table dis-enchants a sharpness-3 sword onto a book for exactly 25 points, spends the book and keeps the item; an unenchanted item, a missing book, no experience and a vanilla enchanting table all dis-enchant nothing and lose nothing
E2E EXIT:0
```

What the scenario proves: the 135 → 110 drop is **exactly** the 25-point cost,
charged once (the second readout of the same table cannot charge again: no
result); the sword kept its item and lost only its enchantments, the book was
spent, and the produced book carries `sharpness:3` and reaches the inventory
through the menu's `results` slot. The controls each change nothing: an
unenchanted item cannot enter the input slot; an enchanted item with no book
keeps its enchantment across the click; a player at 0 points keeps the
enchantment, the book and 0 points; the vanilla enchanting table opens its own
station menu and leaves the table beside it untouched. `SELFTEST=1` asserts the
impossible (the stripped sword kept its enchantment) and exits 1, so the
assertions can fail.

`e2e-modless.log` is the non-vacuous check of the *content*: without
`--dev-mod` the block and item are `Unknown item`/`Unknown block type`, the
readout finds `minecraft:air`, and the packaged
`content/disenchanting_table.hmod` makes all three work.

### Harness facts learned (useful for the next slice)

- **`ctl command` runs a command on the server as the player; `ctl enchant`,
  `ctl give` and `ctl xp` are client-side station controls.** `ctl enchant`
  enchants only the window's own copy, so the server's stack stays plain and the
  table correctly refuses it — enchant with `ctl command "enchant @s …"`. The
  same trap made a first e2e look like a slot-rule bug.
- `xp set <n> points` only moves the bar's progress (`xp set 0 points` left a
  level-7 player at 91 points); take points away with
  `xp add @s -1000 points` (`giveExperiencePoints` clamps at 0).
- A content block placed with `/setblock` gets its block entity (the readout
  answers immediately); no player placement is needed.
- A native mod command must be typed with `ctl chat "/cmd …"`; the usage's
  argument node decides what parses. A literal-only usage
  (`"mod id info"`) failed on some coordinate triples, so the readout registers
  one greedy `text` argument and parses the words itself.
- `data get block` says "not a block entity" for a mod block entity, and
  `data get entity @s Inventory` is the quickest way to see what the *server*
  holds (the control protocol's `state` shows the client's copy).
- A vanilla station menu reports `screen: "station"` in the control protocol; a
  mod container menu reports `container`, and its slots are numbered 300+.

## Class + tier, approach, licence (recorded)

Class **content**, tier **machines**; the shape chosen is the tier's (carried
block content + block entity + container/menu/screen + an interaction that
performs the machine's work), not a plain data block and not a deep
entity/renderer port. Licence **Unlicense → rank 0**; publishable with the
maintainer's approval, local-only until then.

Skills: superpowers/using-superpowers,brainstorming,writing-plans,test-driven-development,systematic-debugging,verification-before-completion; BeautifulRust/rust-best-practices,rust-port

## Review repairs (PR #545)

- The provenance mismatch (PR body `5.0.2`/`ab8c49…` vs the entry's
  `0.1.0`/`9118b6…`) is in the body, which the orchestrator re-writes; the
  entry keeps the hash of this repackaged artifact.
- **[P2] "The linked release source cannot reproduce the submitted
  self-contained archive … packaging only includes top-level `.rs`/`.toml`
  files and never adds the content data directory"** — the packager requires
  `content/<id>.hmod` and `mixins/<id>/data/<id>-content.hmod`, refuses a stale
  or missing embedded copy, and packages the bundle's `data/` folder; the source
  in this tree reproduces the artifact (rebuilt twice, same SHA-256).
- **[P2] "experience payment is not atomic with granting the result"** —
  **fixed**: a points take is reserved against the player's balance the moment
  it is accepted (`runtime::affordable_with`; two clicks in one tick can no
  longer spend the same points), charged at the tick's end, and only then
  applied; a failed write refunds the points. A level take is deducted before
  the result and refunded if the write fails.
- **Book slot validated** — `on_right_click` and `finish` require an actual
  `minecraft:book` (including the hopper and reload paths), so a non-book item in
  that slot is no longer consumed for an enchanted book.
- `mixins/disenchanting_table/LICENSE` and `NOTICE.md` travel inside the
  artifact.

### Verification after the repairs

- Artifact: `disenchanting_table.hnmod`, 43,796 B, SHA-256 `21b934ba239396177a7c296480a635addb740e00c67c55db3af85730db3ea01a` (rebuilt twice, identical bytes; `tools/package.py`; the embedded content equals `content/disenchanting_table.hmod`).
- `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test --locked --offline`: clean, 18 passed; the embedded run (`python3 tools/horizon.py test --horizon ports/_shared/pins/horizon-1d63e0a`) is 18 passed, 0 failed (`target/horizon-test.log`).
- `build-slot.sh e2e --job … --port ports/disenchanting_table -- tools/e2e.sh`: PASS, log `ports/_shared/evidence/disenchanting_table/e2e-repair.log` ends `EXIT:0` (`check-e2e-logs.py disenchanting_table --strict` exit 0).
- `python3 ports/tools/upstream_ready.py disenchanting_table` and `check-skills.py --notes ports/disenchanting_table`: exit 0.

//! Dis-Enchanting Table for Horizon — slice 1, the table.
//!
//! One flat SDK package (mod API 0.2): the carried block, its item, models,
//! textures, lang, loot table, recipe, menu and tags are the `assets/` and
//! `data/` here, and the behaviour is the one `impl Hooks` below. The rules
//! themselves live in [`runtime`], byte-identical to the retired `.hnmod`
//! bundle's `runtime.rs`; [`table`] is the glue.
//!
//! What the retired bundle expressed as nine injected helpers is here three
//! shapes: the block entity and its screen are a **Part** (`table::TablePart`,
//! attached to the type declared for the block), the click and the slot rules
//! are two more Parts (`table::ClickPart` on the block, `table::MenuRulesPart`
//! on the menu), the three slots are a **declaration**, and the readout is the
//! typed `register_commands`/`run_mod_command` pair. Its embedded-content helper
//! has no successor: one `.hmod` carries the content and the code, which is
//! what that helper existed to work around.
use std::sync::Arc;

use mod_api::{
    self, Command, CommandSource, Flow, Hooks, RegisterCommandsEvent, Registrar,
    RegistrationContext, RunModCommandEvent, ServerHost, ServerTickEvent, ServerTickPhase, Text,
};

use crate::runtime::TABLE_MENU;
use crate::table::{Charges, ClickPart, MenuRulesPart, TablePart};

pub mod runtime;

mod table;

#[derive(Default)]
struct Mod {
    /// The takes the clicks queued and the points they already promised: the
    /// Parts share it, so it is one state behind an `Arc` (a Part is attached
    /// once and lives for the session).
    charges: Arc<Charges>,
}

#[mod_api::horizon_mod]
impl Hooks for Mod {
    /// Declare the table: one block with a block entity, three slots and its
    /// menu, plus the Parts that give it its click and its slot rules.
    ///
    /// The type is declared with its block before the Part is attached to it
    /// (`block_entity_part` refuses a key that named no type), and a refusal is
    /// reported rather than ignored, so a broken declaration shows up at load.
    fn register(&self, registrar: &mut dyn Registrar, context: &RegistrationContext) {
        let block = registrar.block_key(crate::runtime::TABLE_BLOCK);
        registrar.block_container(&block, table::container());
        let kind = table::declare(registrar);
        if let Err(error) = registrar.block_entity_part(&kind, TablePart) {
            context.refuse(&error.to_string());
        }
        registrar.block_part(
            block,
            ClickPart {
                charges: Arc::clone(&self.charges),
            },
        );
        registrar.menu_part(TABLE_MENU, MenuRulesPart);
    }

    /// The end of a running tick: run the experience charges the clicks queued.
    /// The console is the only surface that takes points, and a player's hook
    /// cannot reach it.
    fn server_tick(&self, host: &mut dyn ServerHost, event: &mut ServerTickEvent<'_>) {
        if let ServerTickPhase::ServerTickEnd {
            world_running: true,
        } = event.phase
        {
            table::run_charges(&self.charges, host);
        }
    }

    /// `/disenchanting_table info <x> <y> <z>`: the harness readout. Upstream
    /// has no command; the usage is the one the retired bundle registered.
    fn register_commands(&self, _host: &mut dyn ServerHost, event: &mut RegisterCommandsEvent<'_>) {
        event
            .commands
            .push(Command::new("disenchanting_table info <pos:block_pos>"));
    }

    /// The readout body: the dispatcher resolves `<pos>`, so this only reads the
    /// table and answers.
    fn run_mod_command(
        &self,
        host: &mut dyn ServerHost,
        event: &mut RunModCommandEvent<'_>,
    ) -> Flow {
        let literals: Vec<&str> = event.command.literals.iter().map(String::as_str).collect();
        if literals.as_slice() != ["disenchanting_table", "info"] {
            return Flow::Continue;
        }
        let Some(pos) = table::block_pos(event.command.argument("pos")) else {
            let _ = host.commands().feedback(Text::literal(
                "usage: /disenchanting_table info <x> <y> <z>",
            ));
            return Flow::Cancel;
        };
        let player = match event.source {
            CommandSource::Player { player, .. } => Some(*player),
            _ => None,
        };
        let line = table::readout(host, pos, player);
        let _ = host.commands().feedback(Text::literal(line));
        Flow::Cancel
    }
}

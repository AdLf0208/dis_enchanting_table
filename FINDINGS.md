# Dis-Enchanting Table — upstream analysis (slice 1)

Read from the mod's own repository as an analysis aid only; the Rust below is a
clean-room rewrite. No Java was copied or translated and the jar was not
decompiled.

## Provenance

| | |
|---|---|
| Mod | Dis-Enchanting Table **5.0.2** (`merged`, NeoForge + Fabric) |
| Jar | `disenchanting_table-merged-1.21.1-5.0.2.jar`, **306,197 B** |
| SHA-1 | `c054a8bef63addff4d4a9520ec4c7a915c3fc1d1` — **equals** Modrinth's file hash for version `UzZnJKSR` (`ports/_shared/source-map/deps-cache/9BgYgXE4.versions.json`) |
| SHA-256 | `bdb4f294f968b0e5bb75fe4b9bfe65563add01da2e34df81ba6c65fb7e77dc9c` |
| Licence | `license = "Unlicense"` in `META-INF/neoforge.mods.toml` (the source map's `LicenseRef-All-Rights-Reserved` is wrong; the jar wins — ARCHITECTURE.md §8) |
| Repo | <https://github.com/Mods-For-Lupin/Dis-Enchanting-Redux> |
| Commit read | `22b228fcaf87549812ff87897dcedab38a800d9a` (branch **1.21.6**, 2025-10-03) |
| Source map row | `ports/_shared/source-map/atm10-sources.csv:144` |

**The repo has no 1.21.1 branch or tag** (`refs/heads/1.21.6`, `1.21.8`,
`1.21.9` only). The 5.0.2 jar for 1.21.1 was built from the 1.21.1 tree before
it was dropped; the closest readable source is the 1.21.6 branch, whose
`DisEnchantingHelper`/`DisEnchantingMenu`/`DisEnchantingStrategy` match the
jar's class list (`DisenchantmentHelper`, `DisenchantingTableMenu`,
`TileEntityConstants` slot/data numbers) and the 5.0.2 changelog
("Simplified block inventory to work with hoppers", "sound during
dis-enchanting", "automatic_disenchanting"). Differences the 1.21.1 build
cannot be checked against are listed under *Version risk* below.

## Files read

| File | What it gave |
|---|---|
| `core/util/DisEnchantingHelper.java` | `canDisenchant` |
| `core/disenchant/DisenchantmentHelper.java` | the same rule, 1.21.8 form |
| `core/disenchant/SlotPredicates.java` | input/extra slot predicates |
| `core/world/block/entity/function/DisEnchantingStrategy.java` | slot indices, config readers, `hasEnoughExperience` |
| `core/world/block/entity/DisEnchantingTableBlockEntity.java` | 3 slots, the data slots, the (empty) tick |
| `core/world/inventory/DisEnchantingMenu.java` | **the whole behaviour**: result, take, charge |
| `core/world/inventory/DisEnchantingTableMenu.java` | the 1.21.6 slot layout (27/76/134, y 47) |
| `core/world/block/DisEnchantingTableBlock.java` | shape `box(0,0,0,16,12,16)`, `ofFullCopy(ENCHANTING_TABLE)`, use opens the menu |
| `core/util/ExperienceHelper.java` | point/level arithmetic |
| `DisEnchantingTableServer.java` | the server config defaults |
| `core/registry/{ModBlocks,ModItems,ModMenus,ModTabs}.java` | ids and registration |

## Behaviour extracted

**Eligibility** (`canDisenchant`): enchantments on the item (book enchantments
for an enchanted book) — an **enchanted book needs at least 2**, anything else
at least 1.

**Slots** (`DisEnchantingStrategy`): `0` input (must be eligible), `1` extra
(must be `minecraft:book`), `2` output (`mayPlace` false). Hopper faces:
`canAcceptInput`/`canAcceptExtra` in, `canOutputToHopper` from slot 0 when the
item there is no longer eligible, and from slot 2.

**Result** (`createResult`, shown without consuming anything):
- input is **not** an enchanted book → the output book takes **every**
  enchantment of the input;
- input **is** an enchanted book → it **keeps the first** enchantment (the
  iteration order's first) and the output takes the **rest**.

**Take** (`onTake`): particles (`levelEvent 1503`); non-book input — clear the
repair cost when `resets_repair_cost`, remove **all** enchantments, the item
stays; book input — the input becomes a book with the kept enchantment; the
extra slot shrinks by one; charge when `requires_experience` and the player has
enough (creative always has enough); the result is recomputed if the input is
still eligible.

**Charge**: `uses_points` → `giveExperiencePoints(-25)`; else
`giveExperienceLevels(-25)`.

**Config defaults** (`disenchanting_table-server.toml`):
`automatic_disenchanting = false`, `automatic_disenchanting_ticks = 10`,
`resets_repair_cost = true`, `requires_experience = true`, `uses_points = true`,
`experience_cost = 25`.

**Block**: `ofFullCopy(Blocks.ENCHANTING_TABLE)` → hardness 5.0, blast
resistance 1200, light 7, sound group stone; shape a 16×12×16 box; a
`horizontal_facing` state the carried blockstate never selects; its data adds
the block to `minecraft:mineable/pickaxe` and `minecraft:needs_stone_tool`.

**Assets/data in the jar**: one blockstate, two models, two block textures, one
GUI texture, one lang file, a shaped recipe (obsidian + lapis + diamond), a
self-drop loot table (`copy_name`), and the two block tags.

## Version risk (1.21.1 jar vs 1.21.6 source)

| Behaviour | This port |
|---|---|
| The 1.21.1 menu may place the result differently | the port uses the 1.21.6 slot layout (27/76/134, y 47), which the jar's own GUI texture matches |
| `automatic_disenchanting` (a 10-tick auto mode) | not ported (default off); gap |
| The dis-enchant **sound** (5.0.2 changelog) | not ported; gap (`native_play_sound` exists but the sound event id is not in the jar's data) |
| `ExperienceHelper` loses the half-level term (`n/2` integer division) | the port uses vanilla's exact thresholds, which is what the player's own `experienceLevel`/`experienceProgress` mean |
| `deductExperienceLevels` writes the field directly | the port charges through the engine's `giveExperienceLevels` |

## Where the rewrite differs (API shape)

- Horizon has **no result-slot-take hook** (`mayPickup`/`onTake`): the port
  performs the dis-enchant on a **right-click of the table** while input and
  book are present and the output slot is empty, then leaves the produced book
  in the output slot, which the menu's `results` fence already makes take-only.
- Horizon has **no per-slot `mayPlace` in the menu engine**: the port answers
  through `native_menu_slot_rules` (API 130).
- Horizon can **give** experience points and **take** levels, but not take
  points: the default 25-point charge is queued and run on the next server tick
  as the console's `xp add <player> -25 points` (`native_run_command`, API 21 —
  refused inside a player's hook, so it cannot run in the click itself).

# Dis-Enchanting Table → Horizon

Port of **Dis-Enchanting Table 5.0.2** (`Mods-For-Lupin/Dis-Enchanting-Redux`,
**Unlicense** per the jar's `neoforge.mods.toml`), Minecraft 1.21.1 NeoForge,
from the exact ATM10 jar: `disenchanting_table-merged-1.21.1-5.0.2.jar`,
**306,197 B**, SHA-1 `c054a8bef63addff4d4a9520ec4c7a915c3fc1d1` (matches
Modrinth's file hash for version `UzZnJKSR`), SHA-256
`bdb4f294f968b0e5bb75fe4b9bfe65563add01da2e34df81ba6c65fb7e77dc9c`.
Provenance and the carried bytes:
[`../_shared/carry-work/disenchanting_table/SOURCE.md`](../_shared/carry-work/disenchanting_table/SOURCE.md).

**Slice 1.** The `disenchanting_table:disenchanting_table` block with its
three-slot menu (input, book, output), the screen readout, and the
enchantment transfer — an enchanted item gives its enchantments to a fresh
enchanted book, or an enchanted book keeps its first and gives the rest — for
**25 experience points**, the book spent and the item kept.

## The package (mod API 0.2, version 0.2.0)

One flat `.hmod` carries the content and the code; `.hnmod` is retired.

| Path | What it is |
|---|---|
| `horizon-mod.toml` | the only manifest and the only identity: `id`, `version = "0.2.0"`, `api = "0.2"`, `side`, `capabilities`, and `[native]` pinning game 26.3, Horizon `0b14f8a…` and mixins `e59da90…` |
| `Cargo.toml` | one crate, edition 2024; its only dependency is `horizon-mod-api` from `.horizon/sdk/` (fetched, never packed) |
| `src/lib.rs` | the one `#[mod_api::horizon_mod] impl Hooks`: `register`, `server_tick`, `register_commands`, `run_mod_command` |
| `src/runtime.rs` | the pure engine (eligibility, the result plan, the cost model, the slot rules) — byte-identical to the retired `.hnmod` bundle's copy, 16 unit tests |
| `src/table.rs` | the glue: the block entity/screen Part, the click Part, the menu Part, the declarations, the deferred charge and the readout |
| `assets/`, `data/` | the carried content — the jar's blockstate, models, textures and lang, the block's tags, a 26.3 re-spelling of the recipe and the loot table, and the port's block/menu/creative-tab declarations |
| `LICENSE`, `LICENSE-NOTICE.md`, `README.md` | the licence, its provenance and this file, inside the package |
| `tools/carry_content.py` | re-derives `assets/` + `data/` from the jar (`--check` compares) |
| `tools/e2e.sh` | the in-game scenario, for a launcher-built game (`GAME=`) |
| `notes.md`, `FINDINGS.md` | the record: migration map, hook table, gaps, upstream analysis (not packed) |

## Build and verify

```sh
# Every compile runs in a lease; BUILD_LOCAL=1 keeps it on this box.
BUILD_LOCAL=1 ../tools/build-slot.sh run --job dt-sdk --port ports/disenchanting_table --step test \
    -- bash -c '[ -f Cargo.toml ] || cd ports/disenchanting_table; cargo test --offline'
BUILD_LOCAL=1 ../tools/build-slot.sh run --job dt-sdk --port ports/disenchanting_table --step lint \
    -- bash -c '[ -f Cargo.toml ] || cd ports/disenchanting_table; cargo clippy --all-targets -- -D warnings'
cargo fmt --check                     # never --all: it would walk the fetched SDK
rm -f Cargo.lock                      # a packed lock changes the artifact
cargo horizon-mod build               # -> target/horizon-mod/disenchanting_table.hmod
cargo horizon-mod check disenchanting_table.hmod
cargo horizon-mod fits disenchanting_table.hmod --build nightly
```

The in-game check needs a launcher-built client carrying the package, which
this environment does not have:

```sh
horizon-launcher native-build . --instance <instance> --trust-native-code
GAME=<that build's horizon-client> tools/e2e.sh          # PASS + E2E EXIT:0
SELFTEST=1 GAME=… tools/e2e.sh                           # must FAIL (exit 1)
```

Local-only until publishing is approved; the Unlicense allows redistribution
with this complete source.

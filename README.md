# Bad Wither No Cookie Reloaded for Horizon

Clean-room Rust port of **Bad Wither No Cookie Reloaded 3.20.4** (`bwncr`, MIT,
Kreezxil / Eleksploded) for Minecraft 26.3 on the Horizon SDK at nightly
`0b14f8acfd97fa2c750e42795249dcc981178ca4` (mixins
`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`).

The mod silences sounds by name — the wither's spawn and death broadcast, the wandering
trader's and llama's ambient calls, the ender dragon's death and the thunder broadcast —
plus a configurable list, and offers `/bwncr listen` to discover the names to add.

## What this package is

One flat `.hmod` (SDK `api = "0.2"`): the Rust code in `src/`, the mod's own language files
in `assets/bwncr/lang/`. There is no separate content half to install.

| Path | What it is |
|---|---|
| `horizon-mod.toml` | the only manifest: `id`, version, `api = "0.2"`, and `[native]` with the exact pins |
| `Cargo.toml` | one crate; its only dependency is `horizon-mod-api` from `.horizon/sdk/` |
| `src/lib.rs` | the one `#[mod_api::horizon_mod] impl Hooks` (config, commands, `play_sound`) |
| `src/support.rs` | the glue: the switches, `/bwncr`, the heard-sound feed, the policy applied to a sound |
| `src/runtime.rs` | the pure policy and its tests — no game types, byte-identical to 0.1.0 |
| `assets/bwncr/lang/` | the mod's three language files, carried from the jar |
| `tools/e2e.sh` | the in-game scenario (needs a launcher-built game, see below) |
| `notes.md`, `FINDINGS.md`, `VALIDATION.md` | the port's history, the upstream behaviour, the ladder |

## Status

**Slice 1, version 0.2.0.** The switches, the config file, the `/bwncr` commands, the
listen→heard discovery loop and the **suppression itself** are ported: `play_sound` is
NeoForge's `PlaySoundEvent`, and setting its sound to nothing is what upstream's listener
does. Read [`notes.md`](notes.md) for the hook table, the gaps (the debug log is gone, the
feed is server-read, the config shape is the port's) and the migration traps.

Not verified in game since the SDK migration: `horizon-launcher` is not installed here, so no
game carrying this package could be built. The 0.1.0 in-game transcript in
[`VALIDATION.md`](VALIDATION.md) is historical.

## Building

```sh
cargo test --offline                      # the pure engine and the glue, 21 tests
cargo clippy --all-targets -- -D warnings # lints
cargo fmt --check                         # formatting (not --all: .horizon/sdk is a path dep)
rm -f Cargo.lock                          # the lock is never packed
cargo horizon-mod build                   # -> target/horizon-mod/bwncr.hmod
cargo horizon-mod check bwncr.hmod        # the registry's own check
```

The game that runs it is built by the launcher, which compiles the mod against its own copy
of the same pin:

```sh
horizon-launcher native-build ports/bwncr --instance <instance> --trust-native-code
```

## Licence

MIT, from the jar's `META-INF/neoforge.mods.toml` (`license="MIT"`, `authors="Kreezxil,
Eleksploded"`); see `LICENSE` and `LICENSE-NOTICE.md`, which travel inside the package. The
port stays local-only until the maintainer approves publishing (ARCHITECTURE.md §8).

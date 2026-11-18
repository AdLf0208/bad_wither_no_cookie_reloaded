# Bad Wither No Cookie - Reloaded — a Horizon port

**Bad Wither No Cookie - Reloaded** is a Minecraft mod. This repository is a **port of it to [Horizon](https://github.com/horizon-rust/horizon)**, the Rust engine for Minecraft Java 26.3. It carries the complete source of the port plus the content files it ships, and installs as a single `.hmod` package.

Upstream: [https://gitlab.com/kreezxil/BadWitherNoCookie](https://github.com/https://gitlab.com/kreezxil/BadWitherNoCookie) — licence **MIT**.

| | |
|---|---|
| Upstream mod | Bad Wither No Cookie - Reloaded ([source](https://github.com/https://gitlab.com/kreezxil/BadWitherNoCookie)) |
| Ported to | Horizon 0b14f8a, Minecraft Java 26.3, mod API `0.2` |
| Package | `bwncr.hmod` (install it with the Horizon launcher) |
| Licence | **MIT** — the upstream mod's licence; see [Licence and credit](#licence-and-credit) |

## What this port implements

- A sound about to play is replaced with nothing when its name matches
- `entity.wither.spawn` (exact) or `entity.wither.death` (case-insensitive)
- `entity.wandering_trader.ambient` or `entity.llama.ambient`
- `entity.ender_dragon.death` (exact) and `silenceDragon`;
- `entity.lightning_bolt.thunder` (exact) and `silenceLightning`;
- any name in `silenceUs` (exact, case-sensitive) — the default list holds a
- Defaults (`BWNCR_Config.java:23-47`): all four switches `true`,
- `/bwncr listen` toggles a mode that prints every heard sound name

## Not yet ported

- **The suppression itself is not carried.** The pin has no client "a sound is
- **`/bwncr listen` prints through `/bwncr heard`, not live.** Upstream sends
- **Level-event sounds are named only where vanilla's table names them**
- **`/bwncr` is a server command, not a client one.** Upstream registers it as
- **Config file shape is ours, key-compatible.** The keys (`silenceWither`,
- **No GUI.** Upstream's config screen (`ConfigurationScreen`) and the Fabric
- **The wither-griefing behaviour the brief described is impossible at the pin
- **Not verified in game.** `horizon-launcher` is not installed here, so no game carrying this

## Installing

1. Install the [Horizon launcher](https://github.com/horizon-rust/horizon/releases) (0.2.70 or newer).
2. Add `bwncr.hmod` to the instance's mods.
3. The package is pinned to a specific Horizon build; the launcher checks that pin.

## Licence and credit

Bad Wither No Cookie - Reloaded was created by its upstream authors; the code, models and assets in the original mod
remain theirs and are used here under **MIT**. The Rust in this repository is a clean-room
rewrite for Horizon, not a translation of the original source.

- `LICENSE` — the upstream licence text.
- `LICENSE-NOTICE.md` — what this port carries and where it came from.
- The upstream project: https://github.com/https://gitlab.com/kreezxil/BadWitherNoCookie

## Technical details

```
horizon-mod.toml   package manifest (id, version, api, [native] pin)
Cargo.toml         the crate; depends on `horizon-mod-api` from the fetched SDK
src/               lib.rs (the one `impl Hooks`), runtime.rs (pure logic + tests), glue
assets/ data/      the content the package ships
tools/             content generators and the e2e scenario
```

- Build: `cargo horizon-mod build` (packages; it does not compile against the game).
- Test: `cargo test`, `cargo clippy --all-targets -- -D warnings`.
- In-game: `ports/tools/launcher-game.sh run bwncr` in the campaign tree, or install the
  `.hmod` and play; the harness scenario is `tools/e2e.sh`.

This repository is generated from the campaign's port at `ports/bwncr/`; the port's own
`notes.md` is the full record (decisions, traps, evidence).

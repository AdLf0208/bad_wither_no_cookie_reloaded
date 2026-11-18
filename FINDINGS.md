# bwncr — findings (slice 1)

Ported from **Bad Wither No Cookie Reloaded** `bwncr-neoforge-1.21.1-3.20.4.jar`
(MIT, Kreezxil/Eleksploded), Minecraft Java 1.21.1 NeoForge 21.1.116, for
Horizon pin `1d63e0a5` / mixins `5a64af93`.

## 1. The jar, verified

| Item | Value |
|---|---|
| jar | `ports/_shared/licence-cache/bwncr-neoforge-1.21.1-3.20.4.jar` (kept as evidence) |
| size | 28,573 B |
| SHA-1 | `dd7920e114fbd066986ea1028aa201a5e0742875` |
| **pack record** (`licence-cache/atm10-modlist.json`, `modrinthHash`) | `dd7920e114fbd066986ea1028aa201a5e0742875` — **matches** |
| SHA-256 | `5ad29f3287e9c6f5b20638f051597918657886a0cdbc5123cf20505210584ac4` |
| declared licence | `license="MIT"` in `META-INF/neoforge.mods.toml`; the "absolute structural" licence |
| manifest version | `Specification-Version: 3.20.4`, `Built-On-Minecraft: 1.21.1`, `Implementation-Timestamp: 2026-05-24T01:26:25+0200` |
| contents | 9 classes, `assets/bwncr/lang/{en_us,es_ar,pt_br}.json`, `logo.png`, `pack.mcmeta` — no LICENSE/NOTICE file |

`python3 ports/tools/precheck-mod.py bwncr` → exit 0, `licence=MIT rank=0`,
`jar licence: MIT (rank 0) from bwncr-neoforge-1.21.1-3.20.4.jar`.

## 2. Upstream repo

- Source map (`atm10-sources.csv`): `source_url = https://gitlab.com/kreezxil/BadWitherNoCookie`,
  `repo` blank. Recovered per the AGENTS.md order — the `source_url` is the
  Modpack Index / pack page's link, and `git ls-remote` confirms it exists and is
  the mod's own project (issue tracker matches the jar's
  `issueTrackerURL = https://gitlab.com/kreezxil/BadWitherNoCookie/-/issues`).
- Branches: `1.21.1` exists; analysed at **`112fa60559f1152141eb38515c9b85ada36b3b9f`**
  (`2026-05-24 01:26:17 +0200`, "Increment version"); `gradle.properties version=3.20.4`,
  `neo_version=21.1.116` — the jar's build. Shallow clone at
  `ports/_shared/upstream/bwncr/` (deleted after writing these notes, as required).
- Files read: `common/.../SoundHandler.java`, `common/.../ProxyPlaySoundEvent.java`,
  `common/.../CommonClass.java`, `forge/.../BadwitherNoCookieNeoForge.java`,
  `forge/.../BWNCR_Config.java`, `forge/.../handler/SoundEventHandler.java`,
  `forge/.../command/NeoForgeListenCommand.java`, `common/src/main/resources/assets/bwncr/lang/*`,
  `gradle.properties`, `META-INF/neoforge.mods.toml`.

## 3. What the mod actually does — and what it does **not**

> **The brief's slice-1 premise is false for this mod version.** BWNCR 3.20.4
> does not touch wither or skull block damage, mob damage, item destruction or
> explosions. There is no wither, block-break, damage or loot code in the jar or
> in the 1.21.1 branch. The name is historical: "Bad wither, no cookie" is about
> the **server-wide sound** a wither emits.

The one behaviour (`SoundHandler.onPlaySound`, `SoundHandler.java:24-70`, called
from a `PlaySoundEvent` listener registered **client-side only** —
`BadwitherNoCookieNeoForge.java:28`, `SoundEventHandler.java:21-27`) is: when the
client is about to play a sound, replace it with `null` if it matches a rule.

| Rule | Sound names | Match | Config key | Default |
|---|---|---|---|---|
| Wither | `entity.wither.spawn`, `entity.wither.death` | spawn: `equals`; death: `equalsIgnoreCase` (upstream quirk) | `silenceWither` | `true` |
| Trader | `entity.wandering_trader.ambient`, `entity.llama.ambient` | `equalsIgnoreCase` each | `silenceTrader` | `true` |
| Dragon | `entity.ender_dragon.death` | `equals` | `silenceDragon` | `true` |
| Lightning | `entity.lightning_bolt.thunder` | `equals` | `silenceLightning` | `true` |
| Custom | any name in `silenceUs` | `equals` (case-sensitive), exact | `silenceUs` | `[""]` (a list holding one empty string; matches no real name) |
| Debug | logs every intercepted name | — | `debugMode` | `false` |

Also `/bwncr listen` (`NeoForgeListenCommand.java:17-38`) toggles a client
"what was that" mode that prints each heard sound name to the player's chat
(`SoundHandler.java:61-67`), so a player can discover names for `silenceUs`.
Registered as a **client** command (`RegisterClientCommandsEvent`).

Not present anywhere: block destruction prevention, mob damage changes, item
destruction, explosion filtering, spawning changes, entity behaviour.

## 4. Hook mapping at pin `1d63e0a5`

| Upstream behaviour | Pin hook / API | State |
|---|---|---|
| Cancel a sound before the client plays it (`PlaySoundEvent`) | **none** — `native-hooks.json` has no sound/audio hook; `play_sound_to` is still `@API:play_sound_to@` (pending, not stamped) | **blocked** |
| `/bwncr listen` sound-name feed | `client_packet` (API 408, listen): the client hears `Sound`/`SoundEntity`/`LevelEvent` before handling; names via `Game::remote_sound_name` and vanilla's global-event table (`level_particles.rs:330-334`) | ported |
| `/bwncr listen` command | `register_commands` + `perform_command` (API 8) | ported |
| Config `silenceWither` … `silenceUs`, `debugMode` | `server_about_to_start` (API 3) + `config_dir` (API 26), file `config/bwncr-common.toml` | ported |

The suppression itself is therefore **not** carried in slice 1: the pin has no
way to refuse a sound. The named-hook request is:

> A client hook where a sound is about to play (sound id, category, position,
> volume, pitch) with the power to cancel it — NeoForge's `PlaySoundEvent`.

## 5. Why the wither-griefing premise is impossible even as a fallback

If the campaign ever wanted the *protection* behaviour the brief described, the
pin could not express it either: the wither's periodic block destruction is not
an explosion. `WitherBoss.customServerAiStep` emits `MobEvent::BreakBlock`
(`mobs/wither.rs:434-472`), handled directly in `game/creatures.rs:3064` — no
hook. `destroy_block` is players-only, and `explosion_detonate` covers only the
skull's blast. Recorded here so the gap is not rediscovered.

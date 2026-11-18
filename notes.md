# Bad Wither No Cookie Reloaded — port notes (server-only/logic)

Ported from **Bad Wither No Cookie Reloaded** (`bwncr`, MIT, Kreezxil /
Eleksploded; jar `bwncr-neoforge-1.21.1-3.20.4.jar`, Minecraft Java 1.21.1
NeoForge 21.1.116) for Minecraft 26.3.

**Current package: 0.2.0, the flat SDK format** (`api = "0.2"`, nightly
`0b14f8acfd97fa2c750e42795249dcc981178ca4`, mixins `e59da909…`) — one `.hmod` carrying the
code and the lang files. See [`## Migration to the SDK format`](#migration-to-the-sdk-format)
at the end for the file map, the hook table, the traps and the ladder; **the sections below it
are the 0.1.0 history** (the retired `.hnmod` on pin `1d63e0a5`, mixins `5a64af93`) and are
kept unedited for the record, including their now-outdated claim that the suppression half has
no hook.

Status: **ported-partial — slice 1**, now with the suppression carried. The 0.1.0 slice ported
the mod's decision policy, its config, its `/bwncr` command and its `/bwncr listen` sound-name
feed, and recorded the actual behaviour — *replacing a sound with nothing before the client
plays it* — as a blocked gap because no "a sound is about to play" hook was stamped at
`1d63e0a5`. The SDK at `0b14f8a` **does** stamp it: `Hooks::play_sound` is NeoForge's
`PlaySoundEvent`, and its `sound` may be set to `None`. The migration therefore closes that gap
([`## Migration to the SDK format`](#migration-to-the-sdk-format), "The gap this migration
closes"). What is *not* proven is the audible half in game: `horizon-launcher` is not installed
here, so no game carrying this package could be built (recorded as a gap).

The carried lang files are at the package root (`assets/bwncr/lang/`), so one file installs the
content and the code. The port stays **local-only until the maintainer approves publishing**
(ARCHITECTURE.md §8, licence rank 0).

## Class and tier; the brief's premise (0.1.0, unchanged)

`server-only` / `logic` (the ledger row), core, P0, MIT (rank 0). The row's
class is about the *kind* of mod (an admin/server comfort feature); the jar's
behaviour is client-side (a `PlaySoundEvent` listener registered only when
`FMLEnvironment.dist.isClient()`, `BadwitherNoCookieNeoForge.java:28`). The tier
is right: pure logic, no machines or content.

**The brief's slice-1 premise is false and is not ported.** It described
"stop the wither (and its skulls) from destroying blocks and mobs". The jar
contains no block, damage, explosion or loot code — only five sound rules
(`SoundHandler.java:24-70`) and a config (`BWNCR_Config.java:23-47`). The name
is historical. Per the campaign rule "a brief may not invent a mod's contents
from its name / the artefact wins", the port implements what the jar does and
records the discrepancy (`FINDINGS.md` §3). The brief's own step 2 asks
"…mob damage, item destruction, **sounds?**", so the answer is "sounds only".

## Behaviour (jar + 1.21.1 source)

Analysis clone: `https://gitlab.com/kreezxil/BadWitherNoCookie` branch `1.21.1`
at `112fa60559f1152141eb38515c9b85ada36b3b9f` (`2026-05-24`, version 3.20.4 —
the jar's build); the clone was deleted after these notes.

1. A sound about to play is replaced with nothing when its name matches
   (`SoundHandler.java:27-59`):
   - `entity.wither.spawn` (exact) or `entity.wither.death` (case-insensitive)
     and `silenceWither`;
   - `entity.wandering_trader.ambient` or `entity.llama.ambient`
     (case-insensitive) and `silenceTrader`;
   - `entity.ender_dragon.death` (exact) and `silenceDragon`;
   - `entity.lightning_bolt.thunder` (exact) and `silenceLightning`;
   - any name in `silenceUs` (exact, case-sensitive) — the default list holds a
     single empty string, which matches nothing.
2. Defaults (`BWNCR_Config.java:23-47`): all four switches `true`,
   `silenceUs = [""]`, `debugMode = false`.
3. `/bwncr listen` toggles a mode that prints every heard sound name
   (`NeoForgeListenCommand.java:17-38`, `SoundHandler.java:61-67`), so a player
   can discover names for `silenceUs`.
4. `debugMode` is console logging of every intercepted name.

The port adapts the names by stripping Horizon's `minecraft:` prefix before
comparing (Horizon's sound ids are `minecraft:entity.wither.spawn` where
NeoForge's event name is `entity.wither.spawn`); the policy accepts both.

## Hooks

| Hook | Helper | What it does |
|---|---|---|
| `server_about_to_start` (v1, HEAD) | `about.rs` | Reads/creates `config/bwncr-common.toml` (API 3 + `config_dir` API 26). |
| `register_commands` (v1, HEAD) | `register.rs` | Declares `/bwncr …` usages (API 8). |
| `perform_command` (v1, HEAD) | `perform.rs` | Answers the player's `/bwncr` line (API 8). |
| `client_packet` (v1, HEAD) | `packet.rs` | Names the sound packets the server sends (`Sound`, `SoundEntity`, global `LevelEvent`), for `/bwncr listen`/`heard` (API 408). |

`client_packet` is the honest substitute for `PlaySoundEvent`: it observes the
sound as the client receives it (via `Game::remote_sound_name`, and vanilla's
global-event names from `level_particles.rs:330-334`) but cannot cancel it.

## Commands

| Command | Level | What it answers |
|---|---|---|
| `/bwncr`, `/bwncr status` | 0 | `bwncr: silenceWither=… silenceTrader=… silenceDragon=… silenceLightning=… custom=N debug=… listen=…` |
| `/bwncr check <sound>` | 0 | `bwncr: <sound> -> SILENCE (<rule>)` or `-> ALLOW` |
| `/bwncr listen` | 0 | `bwncr: listen on/off` (upstream's `/bwncr listen`) |
| `/bwncr heard [n]` | 0 | the newest heard names (`bwncr: heard k of m: …`) |
| `/bwncr silence <sound>` / `allow <sound>` | 2 | edits `silenceUs`, rewrites the config |
| `/bwncr reload` | 2 | re-reads the config |

`check` is the harness command the brief asked for: it reports the exact
decision with its rule, so the e2e asserts the policy rather than guessing.

## Pure engine (`runtime.rs`, 15 tests)

`Config` (defaults/parse/format of the upstream `[general]` keys),
`decide` (the five rules, upstream order, case quirks, `minecraft:`
normalisation), `set_custom`/`remove_custom`, `global_event_sound`
(1023/1028/1038, as `level_particles.rs` names them), and `Feed` (the newest
32 heard names). The parser reads NeoForge's one-line and multi-line string
lists.

## Verification

- **TDD:** 15 tests written before the implementation;
  `ports/_shared/evidence/bwncr/unit-tdd-red.log` shows the stubbed engine
  **11 failed, 4 passed**, and `unit-tdd-green.log` shows **15 passed** after
  the implementation (a red test also caught a real `[""]` list bug before it
  shipped). `cargo fmt --all --check` and
  `cargo clippy --all-targets --locked --offline -- -D warnings` are clean.
- **Ladder at `1d63e0a`, via `ports/tools/build-slot.sh`:** `stage → check →
  test → build`, all `cargo_exit: 0`; `check` **0 warnings**
  (`evidence/bwncr/ladder-check.log`), embedded tests
  `game::bwncr::tests` **15 passed, 0 failed** with 1240 filtered out
  (`ladder-test.log`); that log predates the final cleanup (one unused import
  in `support.rs`, outside the tested module) and the final sources were
  re-verified by `check` (0 warnings) and the full e2e — `VALIDATION.md` note
  (b). Fast build OK (`ladder-build.log`).
- **`tools/e2e.sh` PASS** (`evidence/bwncr/e2e-pass.log`): defaults, all five
  rule families (including the prefixed and upper-case forms), a custom
  add/remove with the config rewritten, the `/bwncr listen` → `/playsound` →
  `/bwncr heard` round trip (the feed really saw
  `minecraft:entity.cow.ambient`), and the carried `.hmod` loading on the
  shared unmodded game as `content from 1 mod(s)`. Transcript:
  `PASS: defaults, upstream rules, custom list, config file, listen feed, hmod
  load, unmodded control` + `EXIT:0`.
- **Controls (both must fail, and do):** `NEGATIVE=1` expects an
  `ALLOW` for `entity.wither.spawn` and exits **1**
  (`evidence/bwncr/e2e-negative.log`, `FAIL: … missing '… -> ALLOW'` +
  `EXIT:1`); `SELFTEST=1` runs the assertion against a wrong transcript and
  exits **1** (`e2e-selftest.log`). The unmodded shared game answers
  `/bwncr status` with vanilla's `Unknown or incomplete command`.
- **Log audit:** `python3 ports/tools/check-e2e-logs.py bwncr --strict` →
  `logs=6 ok=6 missing-exit=0 bad-exit=0` (exit 0), including the three
  embedded legs (`e2e-embedded.log`, `-negative`, `-selftest`).
- **Not applicable, recorded honestly:** `tools/test_template.py` and
  `tools/test_stage.py` are the *template's* self-tests for its one-injection
  example bundle (they assert exactly 1 injection and 1 module against a tiny
  fixture with no hook catalog), so they cannot pass for a four-injection
  named-hook bundle; the campaign ladder above is what was run.
- **Artifacts (version `0.1.0`, the port's version — playbook §1.7; the bytes
  below supersede the `3.20.4` ones in `VALIDATION.md`'s history):** content
  `.hmod` **2,101 B**, SHA-256
  `7d2a8895bfd9f94fb0f54b605fbe3b316af5dc505a0bb9036e2be645c1a84125`
  (`horizon-mod.toml` + the 3 upstream lang files); native `.hnmod`
  **14,483 B**, SHA-256
  `aac8cd3ec64f29a28826779d5c58264d0eb0bad15d8797d283884e5e443ca6d2`
  (`horizon-native.json` `version: 0.1.0`, `hooks: 408` + `mixins/bwncr/*.rs|toml`
  + the embedded `mixins/bwncr/data/bwncr-content.hmod`; `cargo horizon-mod check
  --kind native-mod` at the pin exits 0 — run with the pin's own prebuilt
  `target/fast/cargo-horizon-mod`, since the subcommand is on neither box's
  `PATH`). Both live in place, the `.hmod` in `content/` and the `.hnmod` beside
  it in the port root, and both rebuild to the same hashes (`package-embed.log`,
  two runs). The embedded copy is byte-identical to `content/bwncr.hmod`
  (`7d2a8895…4125`), which `tools/package.py` enforces on every run.

Skills: superpowers/using-superpowers,brainstorming,writing-plans,test-driven-development,systematic-debugging,verification-before-completion; BeautifulRust/rust-best-practices (fmt, clippy `-D warnings`, tests, owned `Action` instead of a borrow of a local, no clone-to-satisfy-borrowck)

## Embedded content (AE2 pattern)

The registry stores **one file per project** (playbook §1.0), so a port with both
halves could not be listed faithfully. This bundle now carries its content,
exactly as the merged AE2 entry does:

- **`mixins/bwncr/data/bwncr-content.hmod`** is the built `content/bwncr.hmod`,
  byte for byte (2,101 B, `7d2a8895…4125`). `tools/package.py` packs the
  bundle's top-level `.rs`/`.toml` **and** everything under `data/` as
  `mixins/<id>/data/…`, with fixed zip timestamps, and refuses a
  `data/<id>-content.hmod` that is not byte-identical to `content/<id>.hmod`.
- **`mixins/bwncr/embedded_content.rs`** with the `embedded-content`
  `[[injection]]` (`target = "horizon_client::game::mods::content_root"`,
  `kind = "HEAD"`, `cancellable = true`, bindings `root`/`enabled`/`dev`)
  compiles the file in with `native_data!` (API 41), writes it to
  `.horizon-cache/bwncr-embedded-content.hmod` when the copy on disk differs,
  appends it to the dev-mod list and returns its `content_root`. The helper
  calls `content_root` once more and stops at its own `dev.contains(&file)`
  guard (AE2's one-level recursion). An unwritable cache logs a warning and
  `Continue`s (no content) instead of failing the game.

**What "the content exists" means for this mod.** Its `.hmod` carries only the
three upstream **lang files** — no block and no item id — so the proof is the
loader's own line, `Horizon: content from 1 mod(s) in …/.horizon-cache/…`, plus
the written file being byte-identical, not an id that resolves. The control
launches the same binary with `.horizon-cache` replaced by a plain file: no
content mod loads, while `/bwncr status` still answers from the code — so the
difference between the runs is exactly the embedded content, and the PASS is not
vacuous.

This makes the `.hnmod` the single listable artifact of this port. It does not
change the recorded **gap**: the mod's actual silencing still has no hook at the
pin (`## Gaps`), so what ships is the policy, the config, the commands and the
heard-sound feed, not the cancellation.

### In-game transcript (quoted from `e2e-embedded.log`)

```
== launch (world BwncrE2E, one bundle, no --dev-mod) ==
== control: the same game with no embedded content file ==
OK: the code still answers in the control
  without the embedded file: no content mod loaded, /bwncr still answers from the code
  embedded content: 2008 B at /tmp/opencode/bwncr-home.jgyESK/.horizon-cache/bwncr-embedded-content.hmod, identical to content/bwncr.hmod, loaded as 1 content mod
OK: status reports upstream's defaults
OK: an unrelated sound is allowed
OK: the played sound reached the feed
PASS: one bundle, no --dev-mod: defaults, upstream rules, custom list, config file and listen feed all work, the embedded content loaded as 1 content mod, and the control loaded none while the code still answered
EXIT:0
```

Both fail-capable legs exit 1: `EMBEDDED=1 NEGATIVE=1`
(`e2e-embedded-negative.log`, `FAIL: NEGATIVE=1 expects an allow` +
`EXIT:1`, after the control and the full launch) and `EMBEDDED=1 SELFTEST=1`
(`e2e-embedded-selftest.log`, `FAIL: selftest: …` + `EXIT:1`).

## Version

The registry version is the **port's** version, not the original mod's
(playbook §1.7), so a partial port ships **`0.1.0`**: `content/horizon-mod.toml`
and `mixins/bwncr/horizon-mixin.toml` `[mod] version` carry it, and
`tools/package.py` copies the latter into `horizon-native.json` — entry, file and
release therefore agree. The `3.20.4` strings that remain are provenance
(comments naming the jar the port came from). `Cargo.toml`'s crate version is
still `3.20.4`; no packaging tool reads it and it is in neither artifact —
published ports already ship that way (`botanypots`, `disenchanting_table`).
It was a **manifest-only** change: no id, hook, binding, content or behaviour
file moved, so no e2e was re-run.

## Gaps

- **The suppression itself is not carried.** The pin has no client "a sound is
  about to play" hook: `native-hooks.json` has no sound/audio hook, and
  `play_sound_to` is still `@API:play_sound_to@` (pending, unstamped). The
  named hook request is NeoForge's `PlaySoundEvent` — sound id, category,
  position, volume, pitch, **cancellable**. Until it is stamped the port cannot
  silence anything; `/bwncr` reports the decisions the hook will use.
- **`/bwncr listen` prints through `/bwncr heard`, not live.** Upstream sends
  each name to the player's chat as it arrives (client-side). The port records
  the last 32 names, and the server command prints them, because the client
  `client_packet` hook has no client-chat helper in this slice. The feed reads
  the packets, so the names are the real ones.
- **Level-event sounds are named only where vanilla's table names them**
  (1023 wither spawn, 1028 dragon death, 1038 end-portal spawn). Other numeric
  level events appear as `minecraft:level_event/<id>`; they are not resolved
  through the packet's numeric id to a name.
- **`/bwncr` is a server command, not a client one.** Upstream registers it as
  a client command (`RegisterClientCommandsEvent`) for the performer only. On a
  server all players see the same config; `silence`/`allow`/`reload` need
  operator level 2, the rest level 0.
- **Config file shape is ours, key-compatible.** The keys (`silenceWither`,
  `silenceTrader`, `silenceDragon`, `silenceLightning`, `silenceUs`,
  `debugMode`) and defaults are upstream's; the comments and list layout are
  the port's own, and NeoForge's richer TOML (dotted tables, escapes beyond
  `\"`/`\\`) is not read.
- **No GUI.** Upstream's config screen (`ConfigurationScreen`) and the Fabric
  Mod Menu integration are not carried; the file and `/bwncr` are the surface.
- **The wither-griefing behaviour the brief described is impossible at the pin
  even as a separate port:** the wither's periodic block destruction is a
  direct `MobEvent::BreakBlock` (`mobs/wither.rs:434-472` → `game/creatures.rs:3064`),
  not an explosion; `destroy_block` is players-only and `explosion_detonate`
  covers only the skull's blast (`FINDINGS.md` §5).

## Licence

MIT (the jar's `META-INF/neoforge.mods.toml` `license="MIT"`; upstream
publishes no LICENSE file, so `LICENSE` carries the MIT text with the jar's
authors named and that provenance note). Rank 0 — derivatives allowed, but the
port stays local-only until the maintainer approves publishing
(ARCHITECTURE.md §8). The pack jar is kept at
`ports/_shared/licence-cache/bwncr-neoforge-1.21.1-3.20.4.jar`
(SHA-1 `dd7920e1…2875`, the pack record's hash).

## Review repairs (PR #615, 2026-11-11)

Review: `ports/_shared/horizon-mods/reviews/pr-615-bwncr.md` (CHANGES_REQUESTED, 1 item + follow-up).

| Reviewer's item | This job |
|---|---|
| [P1] artifact not staged in the registry | **orchestrator** (no write access) |
| upstream publishes no LICENSE; "supply an authoritative grant … or clearly segregate independently original permitted files" | **answered with the artefact**: the jar the pack ships (`bwncr-neoforge-1.21.1-3.20.4.jar`, `META-INF/neoforge.mods.toml`) declares `license="MIT"` with `authors="Kreezxil, Eleksploded"`. `LICENSE-NOTICE.md` now quotes that file and line, records that the upstream tree at the analysed revision has no licence file, and states that the only upstream-derived content in this port is the three language files — the block, item, model and texture are this campaign's own work |
| the archive contains no LICENSE/NOTICE | **fixed** by the packager (`mixins/bwncr/LICENSE` + `LICENSE-NOTICE.md`) |
| "the entry expressly says suppression is not implemented" | kept, and made more prominent: the description opens with "implementing it changes no sound" and names the missing `PlaySoundEvent` hook |

A residual uncertainty (the jar's `license` attribute is upstream's own declaration, but
there is no separate licence file) is stated in the notice rather than papered over. If
the reviewer still requires a signed grant, that is a maintainer decision, not a code fix.

Artifact: `bwncr.hnmod` **18,366 B** `3e8bff4626389f106d64c2bd7019b96ee7a3216883d472607960efceea5605d3`
(content `bwncr.hmod`).

## Migration to the SDK format

Migrated 2026-11-18 from the retired `.hnmod` container (Horizon's mod-API merge #16941,
launcher ≥ 0.2.70, PR #17992) to the flat SDK package at nightly
**`0b14f8acfd97fa2c750e42795249dcc981178ca4`**, mixins
**`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`**, `package_format` 6, `content_api` 0.2.
Version **0.2.0**. This is a **first publication**: the old `.hnmod` submission was never
merged, so there is no closed PR to supersede and the changelog says so.

Reference recipes: `ports/imfast/notes.md` → "Migration to the SDK format" (the mechanical
16-step recipe and its fifteen traps), `ports/comforts/src/lib.rs` for the shape of a config
declared through `register_configs`/`config_loading`, and `ports/enderstorage/src/support.rs`
for `register_commands` + `run_mod_command` + `Commands::feedback`.

**Order of work (the brief's traps 1, 5, 6 and 21):**

1. the retired paths left the project root **before the first build**, into
   `ports/_shared/evidence/bwncr/retired-hnmod-0.1.0/` (`mixins/`, `content/`,
   `compatibility.json`, `rust-toolchain.toml`, the old `Cargo.toml`, `bwncr.hnmod`, and the
   old tools) — `cargo horizon-mod build` refuses a root holding package paths it does not
   know;
2. `content/assets/` **moved** (not copied) to the package root as `assets/`, and the three
   language files were diffed against the retired copy: **unchanged** — `en_us.json` 1,200 B
   `bde40b73…`, `es_ar.json` 1,303 B `5058489c…`, `pt_br.json` 1,249 B `8daa0c2b…`;
3. `src/runtime.rs` was `cp`ed from `mixins/bwncr/runtime.rs` and proved with `cmp`;
4. `cargo test` ran on the new crate **before any glue existed**: the 15 runtime tests passed
   with `src/lib.rs` holding nothing but `pub mod runtime;`;
5. only then was the glue written — first as `todo!()`, so the six glue tests failed
   (`sdk-test-red.log`: **15 passed, 6 failed**, runtime green and glue red in the same run),
   then implemented (`sdk-test.log`: **21 passed, 0 failed**).

### The gap this migration closes (read the artefact, not the old note)

The 0.1.0 notes say the pin has no "a sound is about to play" hook, and that was true at
`1d63e0a5`. **It is not true of the SDK.** Measured from
`.horizon/sdk/api/crates/horizon-mod-vocab/src/hooks/sound_events.rs` and the generated trait:

```text
fn play_sound(&self, context: &dyn LoopContext, event: &mut PlaySoundEvent<'_>);
// PlaySoundEvent { instance: u64, starting: bool, original: &SoundPlay,
//                  sound: &mut Option<SoundPlay> }
```

and its own doc says: *"A sound is about to play (`PlaySoundEvent`, vanilla's `SoundEngine.play`)
… Edit `sound` … **or set it to `None` to play nothing**. It is asked … for every sound the
client plays … Client-side only."* — which is exactly the listener upstream registers
(`SoundHandler.java`, `BadwitherNoCookieNeoForge.java:28`, client only) and exactly what it
does with the event (sets the result to `null`).

So the port asks the policy there: [`support::apply`](../bwncr/src/support.rs) calls
`runtime::decide` on `event.original.identifier` and sets `*event.sound = None` when the answer
is `Decision::Silence(_)`. That line has its own fail-capable tests (six of them, run with
`cargo test` without a game), so the glue is not a claim. The *audible* half is still unproven
in game — no launcher here — and is recorded as a gap, not asserted.

The two `play_level_sound_at_position`/`_at_entity` hooks (both sides, `cancel: &mut bool`)
would also silence level sounds, and NeoForge's `PlayLevelSoundEvent` is their counterpart; the
port uses `play_sound` because it is the whole-engine moment upstream hooks, so a custom sound
name in `silenceUs` works for UI and level sounds alike. Recorded as the alternative shape, not
used.

### File map (old `.hnmod` → SDK package)

| Old | New |
|---|---|
| `horizon-mixin.toml` (`[mod]`, `[compatibility]`, 2 `[[module]]`, 5 `[[injection]]`) | `horizon-mod.toml` (`id`/`name`/`version`/`api`/`side`/`capabilities` + `[native]` game/horizon/mixins) |
| `Cargo.toml` (`[lib] path = "mixins/bwncr/runtime.rs"`, `[workspace] members = ["tools/audit"]`) | `Cargo.toml` (`src/lib.rs`, one dependency: `horizon-mod-api` from `.horizon/sdk/`) |
| `mixins/bwncr/runtime.rs` (crate lib + injected module `crate::game::bwncr`) | `src/runtime.rs` — **byte-identical** (`0b300267…f04b48`), `pub mod runtime;` |
| `mixins/bwncr/support.rs` (module `crate::game::bwncr_state`, 352 lines) | `src/support.rs` — the same state, `Mutex<Option<Core>>`, now `host`-driven |
| `about.rs`, `register.rs`, `perform.rs`, `packet.rs`, `embedded_content.rs` | methods of the one `impl Hooks` (no per-hook files) |
| `content/` (`horizon-mod.toml`, `assets/`, `bwncr.hmod`, `target/`) | package root `assets/bwncr/lang/`; one flat `.hmod` |
| `mixins/bwncr/data/bwncr-content.hmod` + `embedded_content.rs` (the AE2 write-out) | **gone**: code and content are one package |
| `compatibility.json`, `rust-toolchain.toml`, `horizon-native.json`-era manifest | **gone**: `[native]` in `horizon-mod.toml` is the pin |
| `tools/{horizon,package,notice,package_content,test_stage,test_template}.py`, `tools/audit/` | moved to the evidence tree; kept: `tools/e2e.sh` |

`src/lib.rs` is the one `#[mod_api::horizon_mod] impl Hooks` and nothing else; `src/support.rs`
holds the glue; `src/runtime.rs` holds every rule and every number, untouched.

`LICENSE-NOTICE.md` was rewritten for this package: the 0.1.0 copy said "the block, item,
model and texture are this campaign's own work" (this mod has none — the three language files
are the only upstream-derived content) and "it silences nothing" (no longer true). It now names
the language files, the fact that both licence files are inside the `.hmod`, and the current
behaviour.

### Helper and module → hook table (every retired injection, mapped)

| Retired injection (pin `1d63e0a5`) | SDK 0.2 | Notes |
|---|---|---|
| `embedded-content` — `content_root` HEAD, `native_data!` + a `.horizon-cache` write | **no home: deleted** | one flat `.hmod` carries the code *and* `assets/`; the write-out existed only because the registry took one file per project |
| `about-to-start` — `server_about_to_start`, hand-rolled `fs::read_to_string`/`fs::write` of `config/bwncr-common.toml`, `log::info!` | `Hooks::register_configs` (push `Config::new("bwncr", ConfigKind::Common, default)`) + `Hooks::config_loading` + `Hooks::config_reloading` | the game writes the default file and hands the text over; the file name is unchanged (`bwncr-common.toml`) |
| `register-commands` — `Vec<NativeCommand>` usages as raw strings | `Hooks::register_commands` — `Vec<mod_api::Command>` (`Command::new` / `Command::op`) | the two argument kinds are typed now: `<sound:text>` (a name holds `:`, so it must be the greedy kind) and `<count:integer>` |
| `perform-command` — `&mut Game`, `&NativeServerPlayer`, the raw `&str` line, `ran: &mut bool` | `Hooks::run_mod_command` — `&mut dyn ServerHost`, `CommandInvocation { literals, arguments }`, `Flow` | `Flow::Cancel` is the old `*ran = true`; feedback moved from `game.native_feedback` to `host.commands().feedback` |
| `client-packet` — `Clientbound::{Sound,SoundEntity,LevelEvent}` + `Game::remote_sound_name`, an **observation substitute** for the missing hook | `Hooks::play_sound` — `PlaySoundEvent { original, sound, starting }` | the real moment, and it can silence; no packet decoding, no `level_particles.rs` id table |
| module `runtime` (`crate::game::bwncr`) | `pub mod runtime` | byte-identical |
| module `state` (`crate::game::bwncr_state`) | `src/support.rs` | the same `static CORE: Mutex<Option<Core>>`: on single player the client's play-sound hook and the server's command share one process |
| `support::decision_for` — `#[expect(dead_code, reason = "the suppression hook is not stamped yet")]` | `support::apply`, called from `play_sound` | the placeholder is gone; the policy is carried out |

Every retired injection has a home except the content write-out, whose *job* — "one file
installs the content and the code" — is now the package format itself. Nothing was deleted
silently.

### Commands: raw line → typed invocation, same readout

The declared usages (levels as in 0.1.0) and the strings are unchanged, so the 0.1.0 e2e
assertions still match:

| Usage | Level |
|---|---|
| `bwncr`, `bwncr status`, `bwncr check`, `bwncr check <sound:text>`, `bwncr listen`, `bwncr heard`, `bwncr heard <count:integer>` | 0 |
| `bwncr silence`, `bwncr silence <sound:text>`, `bwncr allow`, `bwncr allow <sound:text>`, `bwncr reload` | 2 |

`bwncr check` and `bwncr check <sound:text>` are two executable usages of one prefix, not an
optional argument: the port answers the bare form with its own `usage:` line, as 0.1.0 did, and
`[count]`-style syntax is not needed. The readout (`bwncr: silenceWither=…`, `… -> SILENCE
(silenceWither)`, `bwncr: heard k of m: …`) is byte-identical to 0.1.0 — trap 17 of the
brief.

### Gaps (what this package still does not do)

1. **Not verified in game.** `horizon-launcher` is not installed here, so no game carrying this
   package exists (`native-build … --trust-native-code` is the only compile path). The 0.1.0
   transcript in `VALIDATION.md` is historical: it proves the commands, the policy and the
   *packet* feed of the old container, not this package's `play_sound`.
2. **The audible half has no in-game proof.** The decision and the silence line are unit-tested;
   whether the engine asks `play_sound` in a launcher-built game with no audio device is
   untested. `tools/e2e.sh` leaves audio enabled for that reason and says so.
3. **`debugMode` is inert.** Upstream logged every intercepted name; the SDK has no logger for
   a mod (no `log` crate, no `println!` as diagnostics), so the config key is accepted and
   reported, and does nothing. Recorded rather than faked with a chat line per sound.
4. **`/bwncr heard` is server-read.** `play_sound` runs on the client; the feed is the process's
   cell. Single player (one process) works, and a LAN host that also plays works; a *remote*
   client's hears are invisible to the server's command. 0.1.0 had the same shape (its
   `client_packet` cell), now stated plainly.
5. **`global_event_sound` is no longer consulted.** `runtime` keeps it (public, tested) for the
   numeric level events of the old packet route; `play_sound` hands over resolved identifiers,
   so the 1023/1028/1038 table is dead weight kept only because `runtime.rs` is immutable.
6. **Config shape is the port's.** The keys and defaults are upstream's; comments and list
   layout are the port's, and NeoForge's richer TOML (dotted tables, escapes beyond `\"`/`\\`)
   is not read. The file is `config/bwncr-common.toml`, the same path as 0.1.0.
7. **No GUI.** Upstream's config screen and the Mod Menu integration are not carried.
8. **The wither-griefing behaviour the 0.1.0 brief described is impossible at this pin even as
   a separate port** (`FINDINGS.md` §5: the wither's block destruction is a direct
   `MobEvent::BreakBlock`, not an explosion). Unchanged by the migration.

**No upstream defect was found in this migration.** The 0.1.0 adaptation stands and is not a
defect fix: Horizon names carry `minecraft:`, NeoForge's event names do not, and `runtime::decide`
compares without the prefix (accepting both spellings).

### Deviations (deliberate)

1. **The feed's source.** 0.1.0 read the server's sound *packets*; 0.2.0 reads the client
   engine's `play_sound` asks. That is upstream's own moment (its listener is on
   `PlaySoundEvent`), so the feed is more faithful, and it now also sees client-predicted sounds
   the server never sent.
2. **Silencing is at the engine**, client-side, like upstream — not a server command that
   cancels a broadcast.
3. **The config is declared, not read by hand.** The game creates the file when missing
   (`register_configs`), announces every load (`config_loading`) and every on-disk change
   (`config_reloading`) — NeoForge's model, replacing 0.1.0's `server_about_to_start` read and
   its manual `fs::write`.
4. **`/bwncr silence|allow` writes through `ConfigFiles`** (`host.config_files().write`) instead
   of `fs::write`, so the write is atomic and scoped to this mod's own config folder.

### Traps new to this port (each cost a run here)

1. **The brief's `cd ports/bwncr && …` is wrong once a run is forwarded.** `build-slot.sh`
   forwards compile steps to the far box, which already starts in the port
   (`remote-slot.sh` mirrors the caller's directory), so the wrapped `cd` answered `No such
   file or directory` and the lease exited 1 in 2 s. Call `build-slot.sh` **from inside the
   port** and drop the `cd`.
2. **`| tee` hides a red test.** The pipeline exits with `tee`'s status, so the `todo!()` run
   released the slot with `exit 0` while six tests had failed. Put `set -o pipefail` inside
   the wrapped `bash -c`.
3. **`/tmp/opencode` does not exist on the far box.** `tee /tmp/opencode/bwncr-test.log` made a
   green run exit 1 (`No such file or directory`). Log to a **port-relative** path
   (`sdk-test.log`); `pull_port` brings `*.log` back, and the files then move to
   `ports/_shared/evidence/bwncr/`.
4. **`cargo horizon-mod` is on neither forwarded `PATH`.** `build`, `check` and `fits` must run
   with `BUILD_LOCAL=1`; compiles (`test`, `clippy`, `fmt`) forward normally.
5. **`cargo test` writes a `Cargo.lock` in a crate with no registry dependency of its own**, and
   resolves versions *newer* than the SDK's (`libc 0.2.191` vs 0.2.190). `rm -f Cargo.lock`
   immediately before **every** `cargo horizon-mod build`, not only the first.
6. **`play_sound` is `contract = "listen"` yet it can silence.** Silencing is
   `*event.sound = None` on a `&mut Option<SoundPlay>` — there is no `cancel` flag. Reading the
   hook's contract word and stopping there would have kept the 0.1.0 gap.
7. **`starting` is false for a live channel refresh.** Recording every call would duplicate
   names in `/bwncr heard` (a looping sound refreshes); record only `starting`, and silence
   both (a refresh with `None` stops the channel — the doc says so).
8. **The engine has no game at hand.** `play_sound`'s context is `&dyn LoopContext` (side,
   registry names, settings) — no `Host`, no level, no `config_files()`. The switches must come
   from the process-wide cell the config hooks fill; anything else does not compile.
9. **A sound name is not a `word`.** `<sound:word>` stops at `:` and the command would parse
   `minecraft` only; the greedy `text` kind is the one that takes
   `minecraft:entity.wither.spawn`, and it must be the usage's last argument.
10. **Two executable usages of one prefix are allowed, one optional argument is not needed.**
    `bwncr check` + `bwncr check <sound:text>` keeps 0.1.0's bare-form `usage:` reply without
    guessing at `[count]` syntax.
11. **`uses.json` is a textual surface closure, not a call list.** It has **151** entries and
    names handles this mod never calls (`PlayerRef::feed`, `LevelRef::play_sound`,
    `EntityRef::push`) and even `hook:command`, because their identifiers appear in the source.
    Read it that way: 6 hook families (17 `hook:` entries with their fields), 7 `Hooks::`
    methods, 2 `Commands::`, 2 `ConfigFiles::`, 1 capability.
12. **The packer packs the root documents.** `README.md` was rewritten *before* the first build
    and `LICENSE`/`LICENSE-NOTICE.md` sit at the package root, so all three travel inside the
    `.hmod` (12 entries: manifest, `Cargo.toml`, three root documents, three lang files, three
    source files, `uses.json`). No `Cargo.lock`, no `.horizon/`, no `tools/`.
13. **`config_loading` hears every mod's config.** Filter on `event.config.mod_id == "bwncr"`,
    or another port's file would set the switches.
14. **`ConfigFiles` is `#[access(WorldOnly)]` but allowed in the Config phase** by name
    (`access_allowed`: `ModPhase::Config => … operation == "config_files"`). Do not move a
    `config_files()` call into `Setup`/`Registration`; `run_mod_command` runs in `World`, so the
    command path is fine.
15. **A forwarded run rewrites `ports/_shared/evidence/<mod>/` from the far box.** `pull_port`
    rsyncs `$REMOTE_ROOT/ports/_shared/evidence/<mod>/` back with no exclusions, so evidence
    written locally *after* the far box last received it is silently replaced by the stale
    remote copy — this port's evidence `README.md` was, once, and had to be rewritten. Write
    evidence after the job's last forwarded run, then re-read it; a 0-byte `sdk-fmt.log` is the
    same trap seen from the other side (`tee` of a clean run writes nothing, so the log proves
    nothing).

### Verification (SDK package)

Skills: superpowers/using-superpowers, brainstorming, writing-plans, test-driven-development, systematic-debugging, verification-before-completion; BeautifulRust/rust-best-practices, rust-port

| Check | Command | Result |
|---|---|---|
| Engine green with no glue | `cargo test --offline`, `src/lib.rs` = `pub mod runtime;` | **15 passed**, 0 failed |
| Glue red first | the same, with `apply` as `todo!()` | **15 passed, 6 failed** (`evidence/bwncr/sdk-test-red.log`) |
| Tests | `cargo test --offline` | **21 passed**, 0 failed (`sdk-test.log`) |
| Lints | `cargo clippy --all-targets -- -D warnings` | exit 0, **0 warnings** (`sdk-clippy.log`) |
| Formatting | `cargo fmt --check` (not `--all`) | clean (`sdk-fmt.log`, `FMT_EXIT:0`) |
| Package | `rm -f Cargo.lock && cargo horizon-mod build` | `Built ./target/horizon-mod/bwncr.hmod` (`sdk-build.log`) |
| Second build | the same again | **byte-identical**, `cmp` clean (`sdk-build-second.log`) |
| Registry check | `cargo horizon-mod check bwncr.hmod` | `bwncr 0.2.0 (native-mod, format native-1/0b14f8a/e59da90, for 26.3)`, **22,615 bytes**, SHA-256 `1bde8b52…25cec4`, **EXIT 0** (`sdk-check.log`) |
| Fits the pinned build | `cargo horizon-mod fits bwncr.hmod --build nightly` | `0b14f8a  made for`, **EXIT 0** (`sdk-fits.log`) |
| `runtime.rs` untouched | `sha256sum` retired vs `src/` vs `unzip -p bwncr.hmod src/runtime.rs` | `0b300267…f04b48` three times |
| Language files | `cmp` against the retired `.hmod`'s copies | **unchanged**: 1,200 / 1,303 / 1,249 B |
| `uses.json` | `unzip -p bwncr.hmod uses.json` | 151 entries, summarised in `sdk-uses-summary.txt` |
| Fail-capable e2e assertion | `SELFTEST=1 tools/e2e.sh` | `FAIL: selftest: …` + **`EXIT:1`** (`e2e-sdk-selftest.log`) |
| In-game run | — | **not run**: no launcher here (gap 1) |
| Skills gate | `python3 ports/tools/check-skills.py --notes ports/bwncr --strict` | see the report |

### Changelog (0.2.0)

> **First publication.** This package has no earlier published version, so there is no closed
> PR to supersede; the retired `.hnmod` submission was never merged.
>
> **Changed for `api = "0.2"`:** republished as the flat SDK package (launcher ≥ 0.2.70) —
> `horizon-mod.toml` is the only manifest, the mod is one `#[mod_api::horizon_mod] impl Hooks`,
> and the content is at the package root, so one `.hmod` installs the language files and the
> code together. The hooks are now typed methods: the config is declared through
> `register_configs` (the game writes and reloads `config/bwncr-common.toml`), the commands are
> `register_commands` + `run_mod_command` with parsed arguments, and — the substantive change —
> the silencing itself is carried, because the SDK stamps `play_sound` (NeoForge's
> `PlaySoundEvent`) and its sound may be set to nothing. The debug log is not carried (a mod has
> no logger here). `src/runtime.rs` is byte-identical to 0.1.0. Still local-only until the
> maintainer approves publishing (MIT, rank 0).

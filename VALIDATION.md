# bwncr slice 1 — validation

> **HISTORICAL (0.1.0, the retired `.hnmod`).** This file describes the container the mod-API
> merge retired (pin `1d63e0a5`, mixins `5a64af93`, `tools/horizon.py` + a split content
> `.hmod`). It is kept for the record and is **not** the evidence of the 0.2.0 package: that
> ladder is in [`notes.md` → "Migration to the SDK format" → Verification](notes.md#verification-sdk-package),
> with logs named `sdk-*.log` in `ports/_shared/evidence/bwncr/`. No in-game run exists for
> 0.2.0 (the launcher is not installed here).

Pin `1d63e0a5a37a7a708eb0b32b49bf0dfeb28ed8a9`, mixins
`5a64af938ec86b00d53d16a408aabc0038fb3e57`, Minecraft 26.3.
Evidence logs: `ports/_shared/evidence/bwncr/`.

## Ladder

| Step | Command | Result | Log |
|---|---|---|---|
| runtime tests | `cargo test --locked --offline` (the lib is `runtime.rs`; `support.rs` is not in it) | red **11 failed / 4 passed**, then green **15 passed / 0 failed** | `unit-tdd-red.log`, `unit-tdd-green.log` |
| fmt + clippy | `cargo fmt --all --check`, `cargo clippy --all-targets --locked --offline -- -D warnings` | clean, **0 warnings** | — |
| stage | `build-slot.sh run … python3 tools/horizon.py stage` | pin exported, plan applied | — |
| check | `build-slot.sh run … python3 tools/horizon.py check` | `cargo_exit: 0`, **0 warnings** | `ladder-check.log` |
| test | `build-slot.sh run … python3 tools/horizon.py test` | `game::bwncr::tests` **15 passed, 0 failed**, 1240 filtered | `ladder-test.log` (b) |
| build | `build-slot.sh run … python3 tools/horizon.py build` | `cargo_exit: 0`, fast profile | `ladder-build.log`, `ladder-build.json` |
| e2e | `build-slot.sh e2e --job bwncr-1d63e0a --port ports/bwncr -- tools/e2e.sh` | **PASS + EXIT:0** | `e2e-pass.log` |
| e2e (negative) | same with `NEGATIVE=1` | **FAIL + EXIT:1** (as required) | `e2e-negative.log` |
| e2e (selftest) | `SELFTEST=1 tools/e2e.sh` | **FAIL + EXIT:1** (as required) | `e2e-selftest.log` |
| log audit | `python3 ports/tools/check-e2e-logs.py bwncr --strict` | `logs=3 ok=3 missing-exit=0 bad-exit=0` | — |
| skills gate | `python3 ports/tools/check-skills.py --notes ports/bwncr --strict` | `SKILLS GATE: PASS` | — |

(b) The embedded-test log was recorded on the revision immediately before the
final cleanup: removing one unused `Rule` import from `support.rs`, a file
outside `runtime.rs` and outside the injected test module. The final sources
were verified by the staged `check` (0 warnings, `ladder-check.log`) and by the
full e2e (`e2e-pass.log`, `e2e-negative.log`). Two attempts to re-run the staged
`test` on the final bytes were terminated by the environment (SIGTERM, rc 143)
while the shared slots were saturated by other sessions' jobs; no test code
changed after the recorded run.

The template's own `tools/test_template.py` / `tools/test_stage.py` are
self-tests for its one-injection example bundle (they assert exactly 1 injection
and 1 module against a fixture with no hook catalog) and cannot describe this
four-injection named-hook bundle; they were not used as acceptance evidence.

## Packages (deterministic; rebuilt to the same hashes)

| File | Size | SHA-256 | Entries |
|---|---:|---|---|
| `content/bwncr.hmod` (version 0.1.0; was 2,008 B `b46a1e87…c799` before the bump) | 2,101 B | `7d2a8895bfd9f94fb0f54b605fbe3b316af5dc505a0bb9036e2be645c1a84125` | 4 |
| `bwncr.hnmod` (split build, **superseded** — see the embedded section below) | 11,314 B | `a2af48e7ca8a26929a05b126bab6f279a7aea5e84c0d45720befee746d953f39` | 8 |

The `.hmod` is a deterministic zip of `content/` (`tools/package_content.py`:
`horizon-mod.toml` + the three upstream lang files) and was validated by the
real game loading it: `Horizon: content from 1 mod(s)` on the shared unmodded
game (`e2e-pass.log`). The `cargo-horizon-mod build/check` route was not run:
building that tool is a full Horizon cargo build and the game load proves the
same thing (the package declares no code and loads).

**Log-capture note.** `tools/e2e.sh` tees its output to `$LOG` through a
process substitution. In the final pass run the tee file kept only its tail
(the last launch plus `OK`, `PASS` and `EXIT:0`), so `e2e-pass.log` shows the
closing assertions rather than every phase; the negative log, same script and
package, shows the full phase transcript, and the session's console output for
the pass run showed all `OK:` lines. The slot wrapper returned **0**. This is a
capture artefact, not a missing assertion: the `PASS` line lists each one.

## 2026-11-10 — the content embedded in the `.hnmod` (AE2 pattern)

The port shipped an `.hmod` and an `.hnmod`, which the registry cannot list (one
file per project). The built content now travels **inside** the `.hnmod`:
`mixins/bwncr/data/bwncr-content.hmod` (byte-identical to `content/bwncr.hmod`;
`tools/package.py` refuses a stale copy and packs `data/**` as
`mixins/<id>/data/…`) plus `mixins/bwncr/embedded_content.rs`, whose
`content_root` injection writes it to `.horizon-cache/` and appends it to the
dev-mod list.

| Step | Command | Result | Log |
|---|---|---|---|
| fmt + clippy | `cargo fmt --all -- --check`, `rustfmt --check …/embedded_content.rs`, `cargo clippy --all-targets -- -D warnings` | exit 0, **0 warnings** | `ladder-embed.log` |
| runtime tests | `cargo test --locked --offline` | **15 passed**, 0 failed | `ladder-embed.log` |
| stage / check / test | `build-slot.sh run … horizon.py stage\|check\|test` | `cargo_exit: 0` twice, `game::bwncr::tests` **15 passed**, 1240 filtered, **0 warnings** | `ladder-embed.log`, `horizon-check.log`, `horizon-test.log` |
| build | `horizon.py build --profile fast` on this port's own target | `cargo_exit: 0`, **0 warnings**, 43m 50s | `horizon-build.log` |
| package | `tools/package.py` (twice) + `cargo horizon-mod check bwncr.hnmod --kind native-mod` | `.hnmod` **14,340 B** `1749abc1…f53a1` (the pre-0.1.0 version; current bytes in the 0.1.0 section below), 10 entries incl. the data file; check exit 0 | `package-embed.log`, `check-native.log` |
| e2e, one file only | `EMBEDDED=1` (no `--dev-mod`) | **PASS + EXIT:0**; the control loaded no content mod while the code still answered | `e2e-embedded.log` |
| e2e (negative, embedded) | `EMBEDDED=1 NEGATIVE=1` | **FAIL + EXIT:1** (as required) | `e2e-embedded-negative.log` |
| e2e (selftest, embedded) | `EMBEDDED=1 SELFTEST=1` | **FAIL + EXIT:1** (as required) | `e2e-embedded-selftest.log` |
| log audit | `check-e2e-logs.py bwncr --strict` | `logs=6 ok=6 missing-exit=0 bad-exit=0` | — |
| submission gate | `upstream_ready.py bwncr` | **exit 0**, `READY` | — |

**Build note (recorded because it cost runs).** The `horizon.py build` step
failed three times with `could not parse/generate dep info … (os error 2)` on
different slots and different crates, and the slot journal showed one slot
granted to several jobs at once, so concurrent jobs were replacing each other's
`stage/modded` and writing the same target. The build above therefore ran in its
own port-local stage and target (`target/embed-{stage,cargo}`), which no other
job touches; the e2e then rebuilt into the same tree and ran its `GAME` from
there, in one lease. The port's own sources were unchanged by that.

## 2026-11-10 — the port's version in the artifacts (manifest-only)

AGENT_PLAYBOOK.md §1.7: the registry version is the **port's** version, so a
partial port ships `0.1.0` instead of inheriting the original mod's `3.20.4`.
Only version strings changed — no id, hook, binding, content or behaviour file
was touched, so **no e2e was re-run** (the previous e2e legs still describe the
same code; the embedded `.hmod` differs only by its `horizon-mod.toml` version
line).

| Step | Command | Result |
|---|---|---|
| version | `content/horizon-mod.toml` `version`, `mixins/bwncr/horizon-mixin.toml` `[mod] version` | `0.1.0` (no version constant exists in `tools/package.py` / `package_content.py`; `package.py` reads the mixin plan, `package_content.py` zips `content/`) |
| content rebuild | `python3 tools/package_content.py` | `content/bwncr.hmod` **2,101 B** `7d2a8895bfd9f94fb0f54b605fbe3b316af5dc505a0bb9036e2be645c1a84125`, 4 entries |
| embed | `cp content/bwncr.hmod mixins/bwncr/data/bwncr-content.hmod` | embedded copy **byte-identical** (`sha256sum` equal), which `tools/package.py` re-checks on every run |
| bundle, twice | `python3 tools/package.py` ×2 | `bwncr.hnmod` **14,483 B** `aac8cd3ec64f29a28826779d5c58264d0eb0bad15d8797d283884e5e443ca6d2`, 10 entries, identical bytes both runs |
| bundle manifest | `horizon-native.json` inside the zip | `{"id":"bwncr","version":"0.1.0",...}`, `hooks: 408`, `horizon 1d63e0a5…`, `mixins 5a64af93…` |
| artifact check | `ports/_shared/pins/horizon-1d63e0a/target/fast/cargo-horizon-mod horizon-mod check bwncr.hnmod --kind native-mod` | exit **0**: `bwncr 0.1.0 (native-mod, format native-1/1d63e0a5…/5a64af93…, for 26.3) 14483 bytes, SHA-256 aac8cd3e…a6d2` |
| content check (same tool) | `… horizon-mod check content/bwncr.hmod --kind mod` | exit **0**: `bwncr 0.1.0 (mod, API 0.2, both) 2101 bytes … no code: content only` |
| publish tree | `publish/bwncr/`: artifact + `content/` + `mixins/` + `tools/` synced; `registry.toml` `[[versions]]` → `version = "0.1.0"`, `sha256 = "aac8cd3e…"`, `published = "2026-11-10"` | entry, file and release agree; prose/README/icons untouched |
| entry pre-flight | `python3 ports/tools/upstream_publish.py bwncr` (dry run) | exit **0** — `bwncr: ready (MIT, bwncr.hnmod)` |
| gates | `upstream_ready.py bwncr`, `check-e2e-logs.py bwncr --strict` | both exit **0** |

`cargo horizon-mod` is not on `PATH` here (and not on the far box, so the
forwarded `--step check` came back `error: no such command: horizon-mod`); the
check was run with the **pin's own prebuilt binary**
(`ports/_shared/pins/horizon-1d63e0a/target/fast/cargo-horizon-mod`), which is
the same revision the bundle pins. The check is a read-only zip inspection — it
compiles nothing, so it needs no build slot and writes no target.

`Cargo.toml` still carries the crate version `3.20.4` (with the provenance
comment above it). It is not read by the packaging tools and is **not** in either
artifact; published ports already ship that way (`botanypots` 21.1.44,
`disenchanting_table` 5.0.2, both registry `0.1.0`). Left unchanged here because
changing it also means editing `Cargo.lock`'s root-package entry — outside this
manifest-only slice.

## What the e2e proves

1. The `/bwncr` command exists only with the mod (the unmodded game answers
   vanilla's `Unknown or incomplete command`).
2. The defaults are upstream's; all five rules fire, including the prefixed and
   upper-case forms; unrelated sounds are allowed.
3. `/bwncr silence` / `allow` edit `silenceUs` and rewrite
   `config/bwncr-common.toml` with the defaults preserved.
4. `/bwncr listen` + `/playsound` + `/bwncr heard` show the real sound packet
   (`minecraft:entity.cow.ambient`) reaching the feed.
5. The carried `.hmod` loads (`content from 1 mod(s)`).

What it does **not** prove: that a sound is silenced. No hook at the pin can
cancel one (`notes.md`, "Gaps").

#!/usr/bin/env bash
# In-game check for bwncr 0.2.0, the flat SDK package (mod API 0.2), Minecraft 26.3.
#
# What this scenario can prove:
#   1. the `/bwncr` command exists only with the mod (control below), and the
#      loaded defaults match upstream (all four switches on, empty custom list);
#   2. the policy silences exactly the upstream names, with their case quirks,
#      and leaves everything else alone;
#   3. `/bwncr silence` / `/bwncr allow` edit the custom list and rewrite
#      config/bwncr-common.toml;
#   4. `/bwncr listen` + `/bwncr heard` really observe the sounds this client
#      hears (a `/playsound` round trip), which is upstream's `/listen`
#      discovery command;
#   5. the control run on an unmodified, launcher-built game sees no `bwncr`
#      command, so the replies above come from the port.
#
# NEGATIVE=1 keeps the scenario but expects the wither spawn to be ALLOWED; the
# run MUST fail, proving the assertion is not vacuous.
# SELFTEST=1 runs the assertions against a wrong transcript; it MUST fail too.
#
# The *silencing itself* is only observable with a sound engine: `play_sound` is
# asked as the client's engine takes a sound, which needs a client that plays
# audio. This scenario therefore proves the decision (`/bwncr check`) and the
# feed (a real played sound), and the audible half is checked in the 0.1.0
# evidence only as far as the packet route went — record that honestly, do not
# assert what cannot fail. AUDIO=1 keeps HORIZON_NO_AUDIO unset so the sound
# engine starts (the default here, unlike other ports' headless scripts).
#
# Since the SDK migration (2026-11-18) the mod is a flat `.hmod` the launcher
# compiles into the game, so GAME must name a game the launcher built with it:
#   horizon-launcher native-build ports/bwncr --instance <instance> --trust-native-code
# and then `GAME=<that horizon-client> tools/e2e.sh`. There is no `--dev-mod`:
# the package carries its code and its `assets/` together.
#
# Run through the registry:
#   ports/tools/build-slot.sh e2e --job bwncr-sdk --port ports/bwncr -- tools/e2e.sh
set -euo pipefail

CLONE=/home/adlf/Desktop/all_the_mods_10_rust_rewrite
PORT=$CLONE/ports/bwncr
SHARED=$CLONE/ports/_shared/bin/horizon-1d63e0a5
CTL=${CTL:-$SHARED/horizon-ctl}
# A launcher-built game carrying this package. The old default (the retired
# `tools/horizon.py build` output) no longer exists; the script fails with
# "missing binary" rather than running an unmodded game, which is intended.
GAME=${GAME:-$PORT/target/horizon-cargo/fast/horizon-client}
CONTROL_GAME=${CONTROL_GAME:-$SHARED/horizon-client}
RES=${RES:-${HORIZON_RES:-/home/adlf/Desktop/horizon/.local/horizon-client/26.3}}
SOCK=${SOCK:-/tmp/opencode/bwncr-e2e.sock}
LOG=${LOG:-/tmp/opencode/bwncr-e2e.log}
NEGATIVE=${NEGATIVE:-0}
SELFTEST=${SELFTEST:-0}
AUDIO=${AUDIO:-1}
WORLD=${WORLD:-BwncrE2E}
CONTROL_WORLD=${CONTROL_WORLD:-BwncrControl}

mkdir -p "$(dirname "$LOG")"
exec > >(tee "$LOG") 2>&1

fail() {
    echo "FAIL: $*" >&2
    exit 1
}

ctl() { "$CTL" --socket "$SOCK" "$@"; }

SCRATCH=""
SCRATCH2=""
PIDFILE=/tmp/opencode/bwncr-e2e.pid
# Launch the game and remember its pid: the game keeps writing to this log
# while it shuts down, and EXIT:<status> must be the log's last line.
launch_game() { # launch_game <game binary> [extra args...]
    local game=$1
    shift
    local json
    json=$("$CTL" --socket "$SOCK" launch --game "$game" "$@")
    jq -r '.launch.pid // empty' <<<"$json" >"$PIDFILE"
}
cleanup() {
    local rc=$? pid
    ctl quit >/dev/null 2>&1 || true
    pid=$(cat "$PIDFILE" 2>/dev/null || true)
    if [ -n "${pid:-}" ]; then
        for _ in $(seq 1 100); do
            kill -0 "$pid" 2>/dev/null || break
            sleep 0.2
        done
    fi
    sleep 1
    [ -n "$SCRATCH" ] && rm -rf "$SCRATCH" 2>/dev/null || true
    [ -n "$SCRATCH2" ] && rm -rf "$SCRATCH2" 2>/dev/null || true
    rm -f "$PIDFILE" 2>/dev/null || true
    echo "EXIT:$rc"
}
trap cleanup EXIT

# The last N chat lines, one per line.
chat_lines() { ctl messages 40 | jq -r '.chat[]'; }

# Runs a chat line and waits for the tick that answers it.
say() {
    ctl chat "$1" >/dev/null
    ctl wait 4 >/dev/null
}

check_text() { # check_text <transcript> <substring> <why>
    if ! grep -qF -- "$2" <<<"$1"; then
        echo "--- text was ---" >&2
        echo "$1" >&2
        fail "$3: missing '$2'"
    fi
    echo "OK: $3"
}

expect_chat() { # expect_chat <substring> <why>
    check_text "$(chat_lines)" "$1" "$2"
}

if [ "$SELFTEST" = 1 ]; then
    echo "== selftest: the assertion must reject a wrong transcript =="
    check_text "bwncr: something else" "bwncr: this line is not in the chat" "selftest"
    echo "selftest did not fail"
    exit 1
fi

[ -x "$CTL" ] || fail "missing horizon-ctl: $CTL"
[ -x "$GAME" ] || fail "missing launcher-built game: $GAME (horizon-launcher is not installed here; see the header)"
[ -x "$CONTROL_GAME" ] || fail "missing unmodified game: $CONTROL_GAME"
[ -d "$RES/assets" ] || fail "missing prepared resources: $RES"
[ -f "$PORT/bwncr.hmod" ] || fail "missing package: $PORT/bwncr.hmod"
command -v jq >/dev/null || fail "missing jq"

SCRATCH=$(mktemp -d /tmp/opencode/bwncr-home.XXXXXX)
mkdir -p "$SCRATCH"
export HORIZON_HOME=$SCRATCH/game
export HORIZON_LAUNCHER_HOME=$SCRATCH/launcher
export XDG_DATA_HOME=$SCRATCH/data
export XDG_CONFIG_HOME=$SCRATCH/config
# `play_sound` is asked as the client's sound engine takes a sound, so the
# engine must start: no HORIZON_NO_AUDIO unless AUDIO=0.
if [ "$AUDIO" = 0 ]; then export HORIZON_NO_AUDIO=1; fi
export HORIZON_LAUNCHER_NO_SELF_UPDATE=1
mkdir -p "$(dirname "$SOCK")"
rm -f "$SOCK"
cd "$SCRATCH"

echo "== launch (world $WORLD, the launcher-built game) =="
launch_game "$GAME" \
    --world "$WORLD" --seed 20261109 --creative --headless --wait --resources "$RES"
ctl wait-ready 120

echo "== 1. the defaults are upstream's =="
say "/bwncr status"
expect_chat "bwncr: silenceWither=on silenceTrader=on silenceDragon=on silenceLightning=on custom=0 debug=off listen=off" \
    "status reports upstream's defaults"

echo "== 2. the policy silences the upstream names and only them =="
say "/bwncr check entity.wither.spawn"
expect_chat "bwncr: entity.wither.spawn -> SILENCE (silenceWither)" "wither spawn"
if [ "$NEGATIVE" = 1 ]; then
    # A scenario that cannot fail is not evidence: this run MUST exit 1.
    expect_chat "bwncr: entity.wither.spawn -> ALLOW" "NEGATIVE=1 expects an allow"
    echo "NEGATIVE CONTROL DID NOT FAIL"
    exit 1
fi
say "/bwncr check minecraft:entity.wither.death"
expect_chat "bwncr: minecraft:entity.wither.death -> SILENCE (silenceWither)" "wither death, prefixed"
say "/bwncr check ENTITY.WITHER.DEATH"
expect_chat "bwncr: ENTITY.WITHER.DEATH -> SILENCE (silenceWither)" "wither death, case-insensitive"
say "/bwncr check minecraft:entity.llama.ambient"
expect_chat "bwncr: minecraft:entity.llama.ambient -> SILENCE (silenceTrader)" "llama ambient"
say "/bwncr check entity.ender_dragon.death"
expect_chat "bwncr: entity.ender_dragon.death -> SILENCE (silenceDragon)" "dragon death"
say "/bwncr check minecraft:entity.lightning_bolt.thunder"
expect_chat "bwncr: minecraft:entity.lightning_bolt.thunder -> SILENCE (silenceLightning)" "thunder"
say "/bwncr check minecraft:entity.cow.ambient"
expect_chat "bwncr: minecraft:entity.cow.ambient -> ALLOW" "an unrelated sound is allowed"

echo "== 3. the custom list edits and rewrites the config =="
say "/bwncr silence entity.cow.ambient"
expect_chat "bwncr: silenceUs += entity.cow.ambient (now 1)" "custom list append"
say "/bwncr check minecraft:entity.cow.ambient"
expect_chat "bwncr: minecraft:entity.cow.ambient -> SILENCE (silenceUs)" "custom rule"
say "/bwncr allow entity.cow.ambient"
expect_chat "bwncr: silenceUs -= entity.cow.ambient (now 0)" "custom list remove"
say "/bwncr check minecraft:entity.cow.ambient"
expect_chat "bwncr: minecraft:entity.cow.ambient -> ALLOW" "custom rule removed"
CONFIG_FILE=$(find "$SCRATCH" -name 'bwncr-common.toml' -type f -print -quit 2>/dev/null || true)
[ -n "$CONFIG_FILE" ] || fail "the config file bwncr-common.toml was not written under $SCRATCH"
grep -q '^silenceWither = true$' "$CONFIG_FILE" || fail "config lost silenceWither=true"
grep -q '^silenceUs = \[""\]$' "$CONFIG_FILE" || fail "config lost the default silenceUs entry"
echo "OK: config $CONFIG_FILE carries the defaults"

echo "== 4. /bwncr listen observes the sounds this client hears =="
say "/bwncr listen"
expect_chat "bwncr: listen on" "listen on"
ctl command "playsound minecraft:entity.cow.ambient master @a 0 0 0 1 1" >/dev/null
ctl wait 6 >/dev/null
say "/bwncr heard 8"
expect_chat "minecraft:entity.cow.ambient" "the played sound reached the feed"
say "/bwncr listen"
expect_chat "bwncr: listen off" "listen off"

echo "== 5. control: the unmodified game has no bwncr command =="
ctl quit >/dev/null 2>&1 || true
sleep 2
rm -f "$SOCK"
SCRATCH2=$(mktemp -d /tmp/opencode/bwncr-control.XXXXXX)
mkdir -p "$SCRATCH2"
(
    cd "$SCRATCH2"
    export HORIZON_HOME=$SCRATCH2/game
    export HORIZON_LAUNCHER_HOME=$SCRATCH2/launcher
    export XDG_DATA_HOME=$SCRATCH2/data
    export XDG_CONFIG_HOME=$SCRATCH2/config
    export HORIZON_NO_AUDIO=1
    export HORIZON_LAUNCHER_NO_SELF_UPDATE=1
    launch_game "$CONTROL_GAME" \
        --world "$CONTROL_WORLD" --seed 20261109 --creative --headless --wait \
        --resources "$RES"
    ctl wait-ready 120
    ctl chat "/bwncr status" >/dev/null
    ctl wait 4 >/dev/null
    lines=$(chat_lines)
    echo "--- control chat ---"
    echo "$lines"
    if grep -qF -- "bwncr:" <<<"$lines"; then
        fail "the unmodded game answered a bwncr command"
    fi
    echo "OK: the unmodded game has no bwncr command"
    ctl quit >/dev/null 2>&1 || true
)
rm -rf "$SCRATCH2" 2>/dev/null || true
SCRATCH2=""

echo "PASS: defaults, upstream rules, custom list, config file, listen feed, unmodded control"

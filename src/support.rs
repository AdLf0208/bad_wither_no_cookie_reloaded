//! Game-side glue for Bad Wither No Cookie Reloaded: the config file, the `/bwncr`
//! commands, the heard-sound feed, and the one line of policy the sound hook runs.
//!
//! Upstream keeps its switches in NeoForge's COMMON config and asks them on every
//! `PlaySoundEvent` (`SoundHandler.java:21-27`), a listener registered only on the client
//! (`BadwitherNoCookieNeoForge.java:28`). The SDK has both moments: `register_configs` /
//! `config_loading` for `config/bwncr-common.toml`, and `play_sound` — the same
//! `PlaySoundEvent`, whose `sound` may be set to `None` — so the policy in
//! [`crate::runtime`] is asked about every sound the client is about to play and can
//! replace it with nothing. The rules and every number live in `runtime`; nothing here
//! decides anything.
use std::sync::{Mutex, MutexGuard};

use mod_api::{
    Command, CommandArgumentValue, CommandInvocation, Config as ModConfig, ConfigKind,
    ConfigLoadingEvent, ConfigPath, ConfigReloadingEvent, Flow, PlaySoundEvent,
    RegisterConfigsEvent, RunModCommandEvent, ServerHost, Text,
};

use crate::runtime::{
    CONFIG_FILE, Config, Decision, Feed, decide, format_config, parse_config, remove_custom,
    set_custom,
};

/// This mod's id: the resource namespace, the config owner and the command's root.
pub const MOD_ID: &str = "bwncr";

/// What the mod keeps for the whole process. On single player the window's client and the
/// server thread are two sides of one process, so the sounds the client hears are the ones
/// `/bwncr heard` reads on the server — as in 0.1.0, this is one process-wide cell rather
/// than a field on the hook object.
#[derive(Default)]
struct Core {
    /// The switches in force, from `config/bwncr-common.toml` (or the defaults).
    config: Config,
    /// Upstream's `whatWasThat`: `/bwncr listen` toggles it.
    listen: bool,
    /// The sounds heard while listening.
    feed: Feed,
}

static CORE: Mutex<Option<Core>> = Mutex::new(None);

fn core() -> MutexGuard<'static, Option<Core>> {
    CORE.lock()
        .expect("the bwncr state lock is held for one short update")
}

fn with_core<T>(update: impl FnOnce(&mut Core) -> T) -> T {
    update(core().get_or_insert_with(Core::default))
}

/// The switches in force.
fn config() -> Config {
    with_core(|core| core.config.clone())
}

/// `config/bwncr-common.toml`, relative to the game's `config/` folder.
fn config_path() -> Option<ConfigPath> {
    ConfigPath::new(CONFIG_FILE).ok()
}

fn on(value: bool) -> &'static str {
    if value { "on" } else { "off" }
}

/// The `[general]` switches in one line, for `/bwncr status` and the e2e.
fn summary(config: &Config, listen: bool) -> String {
    let custom = config
        .silence_us
        .iter()
        .filter(|entry| !entry.is_empty())
        .count();
    format!(
        "silenceWither={} silenceTrader={} silenceDragon={} silenceLightning={} custom={custom} \
         debug={} listen={}",
        on(config.silence_wither),
        on(config.silence_trader),
        on(config.silence_dragon),
        on(config.silence_lightning),
        on(config.debug_mode),
        on(listen)
    )
}

/// Declare `config/bwncr-common.toml` as this mod's COMMON config: the game writes
/// `default` there when the file is missing (NeoForge's `registerConfig`).
pub fn register_configs(event: &mut RegisterConfigsEvent<'_>) {
    let default = format_config(&Config::default());
    event
        .configs
        .push(ModConfig::new(MOD_ID, ConfigKind::Common, &default));
}

/// The game read a config: ours it is `bwncr`, so keep the switches. Every mod hears every
/// config, and `Client`/`Common` load once on their owning side.
pub fn config_loading(event: &ConfigLoadingEvent<'_>) {
    read(event.config.mod_id.as_str(), event.config.text.as_str());
}

/// The file changed on disk: apply the new switches without a restart.
pub fn config_reloading(event: &ConfigReloadingEvent<'_>) {
    read(event.config.mod_id.as_str(), event.config.text.as_str());
}

/// Keeps the parsed switches when the config is this mod's.
fn read(mod_id: &str, text: &str) {
    if mod_id == MOD_ID {
        let parsed = parse_config(text);
        with_core(|core| core.config = parsed);
    }
}

/// `/bwncr`'s usages in the server's command tree. Each has no argument kinds beyond the
/// two that need them: a sound name contains `:`, so it is the greedy `text` kind, and a
/// count is an `integer`.
pub fn declare_commands(commands: &mut Vec<Command>) {
    for usage in [
        "bwncr",
        "bwncr status",
        "bwncr check",
        "bwncr check <sound:text>",
        "bwncr listen",
        "bwncr heard",
        "bwncr heard <count:integer>",
    ] {
        commands.push(Command::new(usage));
    }
    for usage in [
        "bwncr silence",
        "bwncr silence <sound:text>",
        "bwncr allow",
        "bwncr allow <sound:text>",
        "bwncr reload",
    ] {
        commands.push(Command::op(usage, 2));
    }
}

/// The greedy `sound` argument of a command, or `None`.
fn sound_argument(invocation: &CommandInvocation) -> Option<&str> {
    match invocation.argument("sound")? {
        CommandArgumentValue::String(sound) => Some(sound.as_str()),
        _ => None,
    }
}

/// The optional `count` argument of `/bwncr heard`.
fn count_argument(invocation: &CommandInvocation) -> Option<i32> {
    match invocation.argument("count")? {
        CommandArgumentValue::Integer(count) => Some(*count),
        _ => None,
    }
}

/// One feedback line to the command's source, vanilla's silence and track-output rules
/// included.
fn say(host: &mut dyn ServerHost, line: &str) {
    let _ = host.commands().feedback(Text::literal(line));
}

/// A player's (or the console's) `/bwncr …`: answer it and take it. `Flow::Continue` for a
/// line that is not ours.
pub fn run_command(host: &mut dyn ServerHost, event: &mut RunModCommandEvent<'_>) -> Flow {
    let invocation = event.command;
    let Some(root) = invocation.literals.first().map(String::as_str) else {
        return Flow::Continue;
    };
    if root != MOD_ID {
        return Flow::Continue;
    }
    let path: Vec<&str> = invocation.literals.iter().map(String::as_str).collect();
    let line = match path.as_slice() {
        ["bwncr"] => usage(),
        ["bwncr", "status"] => status(),
        ["bwncr", "check"] => check(sound_argument(invocation).unwrap_or("")),
        ["bwncr", "silence"] => silence(host, sound_argument(invocation).unwrap_or("")),
        ["bwncr", "allow"] => allow(host, sound_argument(invocation).unwrap_or("")),
        ["bwncr", "listen"] => listen(),
        ["bwncr", "heard"] => heard(count_argument(invocation).unwrap_or(8)),
        ["bwncr", "reload"] => reload(host),
        _ => return Flow::Continue,
    };
    say(host, &line);
    Flow::Cancel
}

fn usage() -> String {
    "bwncr: /bwncr status|check <sound>|listen|heard [n]|silence <sound>|allow <sound>|reload"
        .to_owned()
}

fn status() -> String {
    let config = config();
    let listen = with_core(|core| core.listen);
    format!("bwncr: {}", summary(&config, listen))
}

/// The policy's answer for one name, as `/bwncr check` prints it.
fn check(name: &str) -> String {
    let name = name.trim();
    if name.is_empty() {
        return "bwncr: usage: /bwncr check <sound>".to_owned();
    }
    match decide(&config(), name) {
        Decision::Silence(rule) => format!("bwncr: {name} -> SILENCE ({})", rule.key()),
        Decision::Allow => format!("bwncr: {name} -> ALLOW"),
    }
}

/// Adds a name to `silenceUs` and rewrites the config file.
fn silence(host: &mut dyn ServerHost, name: &str) -> String {
    let name = name.trim();
    if name.is_empty() {
        return "bwncr: usage: /bwncr silence <sound>".to_owned();
    }
    let mut config = config();
    if !set_custom(&mut config, name) {
        return format!("bwncr: {name} is already in silenceUs");
    }
    let saved = save(host, &config);
    let count = config
        .silence_us
        .iter()
        .filter(|entry| !entry.is_empty())
        .count();
    with_core(|core| core.config = config);
    format!(
        "bwncr: silenceUs += {name} (now {count}{})",
        if saved { "" } else { ", not saved" }
    )
}

/// Removes a name from `silenceUs` and rewrites the config file.
fn allow(host: &mut dyn ServerHost, name: &str) -> String {
    let name = name.trim();
    if name.is_empty() {
        return "bwncr: usage: /bwncr allow <sound>".to_owned();
    }
    let mut config = config();
    if !remove_custom(&mut config, name) {
        return format!("bwncr: {name} is not in silenceUs");
    }
    let saved = save(host, &config);
    let count = config
        .silence_us
        .iter()
        .filter(|entry| !entry.is_empty())
        .count();
    with_core(|core| core.config = config);
    format!(
        "bwncr: silenceUs -= {name} (now {count}{})",
        if saved { "" } else { ", not saved" }
    )
}

/// Writes the switches back to `config/bwncr-common.toml`; false when it could not.
fn save(host: &mut dyn ServerHost, config: &Config) -> bool {
    let Some(path) = config_path() else {
        return false;
    };
    host.config_files()
        .write(&path, format_config(config).as_bytes())
        .is_ok()
}

fn listen() -> String {
    let listen = with_core(|core| {
        core.listen = !core.listen;
        core.listen
    });
    format!("bwncr: listen {}", on(listen))
}

/// The newest heard names, oldest first.
fn heard(count: i32) -> String {
    let count = usize::try_from(count).unwrap_or(8).clamp(1, 32);
    let (heard, empty, total) = with_core(|core| {
        (
            core.feed.recent(count),
            core.feed.is_empty(),
            core.feed.len(),
        )
    });
    if empty {
        return "bwncr: heard nothing".to_owned();
    }
    format!(
        "bwncr: heard {} of {}: {}",
        heard.len(),
        total,
        heard.join(", ")
    )
}

/// Re-reads the config file from disk.
fn reload(host: &mut dyn ServerHost) -> String {
    let loaded = match config_path() {
        Some(path) => host
            .config_files()
            .read(&path)
            .map(|bytes| parse_config(&String::from_utf8_lossy(&bytes))),
        None => Err(mod_api::ConfigFileError::InvalidPath),
    };
    let config = match loaded {
        Ok(config) => {
            let parsed = config.clone();
            with_core(|core| core.config = parsed);
            config
        }
        Err(error) => {
            say(host, &format!("bwncr: reload failed: {error}"));
            config()
        }
    };
    let listen = with_core(|core| core.listen);
    format!("bwncr: reloaded {}", summary(&config, listen))
}

/// The policy applied to one sound the client is about to play: the name to record when
/// listening, and `None` for `event.sound` when the policy says to play nothing.
///
/// The decision is [`decide`]'s; this only carries it out. A named rule or a custom entry
/// replaces the sound with nothing, which is what upstream's `PlaySoundEvent` listener does
/// (it sets the event's result to `null`). A live refresh (`starting` false) is silenced the
/// same way, which stops the channel.
fn apply(
    config: &Config,
    listen: bool,
    starting: bool,
    event: &mut PlaySoundEvent<'_>,
) -> Option<String> {
    if matches!(
        decide(config, event.original.identifier.as_str()),
        Decision::Silence(_)
    ) {
        *event.sound = None;
    }
    (listen && starting).then(|| event.original.identifier.clone())
}

/// A sound is about to play on this client: record it for `/bwncr heard` while listening
/// and replace it with nothing when the policy says so. The sound hook runs inside the
/// sound engine with no game at hand, so the switches are read from the process-wide cell.
pub fn about_to_play(event: &mut PlaySoundEvent<'_>) {
    let (config, listen) = with_core(|core| (core.config.clone(), core.listen));
    if let Some(name) = apply(&config, listen, event.starting, event) {
        with_core(|core| core.feed.push(name));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mod_api::{SoundPlay, SoundSource};

    /// A sound as the client asks for it.
    fn sound(identifier: &str) -> SoundPlay {
        SoundPlay::at(identifier, SoundSource::Hostile, [0.0; 3])
    }

    #[test]
    fn a_silenced_sound_is_replaced_with_nothing() {
        let original = sound("minecraft:entity.wither.spawn");
        let mut playing = Some(original.clone());
        let mut event = PlaySoundEvent {
            instance: 1,
            starting: true,
            original: &original,
            sound: &mut playing,
        };
        let recorded = apply(&Config::default(), false, true, &mut event);
        assert!(
            playing.is_none(),
            "the wither's spawn broadcast is silenced"
        );
        assert_eq!(recorded, None, "not listening, so nothing is recorded");
    }

    #[test]
    fn an_allowed_sound_still_plays() {
        let original = sound("minecraft:entity.cow.ambient");
        let mut playing = Some(original.clone());
        let mut event = PlaySoundEvent {
            instance: 2,
            starting: true,
            original: &original,
            sound: &mut playing,
        };
        let recorded = apply(&Config::default(), false, true, &mut event);
        assert_eq!(playing, Some(original), "an unrelated sound is left alone");
        assert_eq!(recorded, None);
    }

    #[test]
    fn listening_records_the_name_of_a_played_sound() {
        let original = sound("minecraft:entity.cow.ambient");
        let mut playing = Some(original.clone());
        let mut event = PlaySoundEvent {
            instance: 3,
            starting: true,
            original: &original,
            sound: &mut playing,
        };
        assert_eq!(
            apply(&Config::default(), true, true, &mut event),
            Some("minecraft:entity.cow.ambient".to_owned())
        );
    }

    #[test]
    fn a_live_refresh_is_not_recorded_again() {
        let original = sound("minecraft:block.note_block.harp");
        let mut playing = Some(original.clone());
        let mut event = PlaySoundEvent {
            instance: 4,
            starting: false,
            original: &original,
            sound: &mut playing,
        };
        assert_eq!(apply(&Config::default(), true, false, &mut event), None);
    }

    #[test]
    fn one_switch_off_lets_its_sound_through() {
        let config = Config {
            silence_trader: false,
            ..Config::default()
        };
        let original = sound("minecraft:entity.llama.ambient");
        let mut playing = Some(original.clone());
        let mut event = PlaySoundEvent {
            instance: 5,
            starting: true,
            original: &original,
            sound: &mut playing,
        };
        apply(&config, false, true, &mut event);
        assert_eq!(playing, Some(original));
    }

    #[test]
    fn the_custom_list_silences_a_sound_it_names() {
        let mut config = Config::default();
        assert!(set_custom(&mut config, "entity.sheep.step"));
        let original = sound("minecraft:entity.sheep.step");
        let mut playing = Some(original.clone());
        let mut event = PlaySoundEvent {
            instance: 6,
            starting: true,
            original: &original,
            sound: &mut playing,
        };
        apply(&config, false, true, &mut event);
        assert!(playing.is_none());
    }
}

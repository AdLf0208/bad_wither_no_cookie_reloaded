//! Bad Wither No Cookie Reloaded — the one `impl Hooks` at the crate root.
//!
//! The retired `.hnmod` named four stamped hooks (and one `content_root` target to embed its
//! lang files). The flat SDK package maps them like this:
//!
//! | Retired injection (pin `1d63e0a5`) | SDK 0.2 |
//! |---|---|
//! | `about-to-start` (`server_about_to_start`, read `config/bwncr-common.toml` by hand) | [`Mod::register_configs`] + [`Mod::config_loading`] + [`Mod::config_reloading`] |
//! | `register-commands` | [`Mod::register_commands`] |
//! | `perform-command` (a raw `&str` line) | [`Mod::run_mod_command`] (typed `CommandInvocation`) |
//! | `client-packet` (observed the server's sound packets) | [`Mod::play_sound`] — the `PlaySoundEvent` the mod is named for, which can also set the sound to nothing |
//! | `embedded-content` (`content_root` HEAD, the AE2 write-out) | **gone**: one flat `.hmod` carries `assets/` and the code |
//!
//! The rules and every number live in [`runtime`], byte-identical to the retired bundle's
//! `runtime.rs` and tested by `cargo test` without the game; [`support`] reads the game,
//! calls `runtime`, and writes the result back.
pub mod runtime;
pub mod support;

use mod_api::{
    self, ConfigLoadingEvent, ConfigReloadingEvent, Flow, Hooks, HostSide, LoopContext,
    PlaySoundEvent, RegisterCommandsEvent, RegisterConfigsEvent, RunModCommandEvent, ServerHost,
};

/// The hook object: it holds nothing. The switches, the listen switch and the heard names
/// live in `support`'s process-wide cell, so the client's play-sound hook and the server's
/// command read the same state on a single-player world.
#[derive(Default)]
struct Mod;

#[mod_api::horizon_mod]
impl Hooks for Mod {
    /// Declare `config/bwncr-common.toml` (NeoForge's COMMON config for this mod), written
    /// with upstream's defaults when the file is missing.
    fn register_configs(&self, _host: HostSide, event: &mut RegisterConfigsEvent<'_>) {
        support::register_configs(event);
    }

    /// The file was read: keep its switches.
    fn config_loading(&self, _host: HostSide, event: &mut ConfigLoadingEvent<'_>) {
        support::config_loading(event);
    }

    /// The file changed on disk: apply it without a restart.
    fn config_reloading(&self, _host: HostSide, event: &mut ConfigReloadingEvent<'_>) {
        support::config_reloading(event);
    }

    /// Declare `/bwncr …` in the server's command tree.
    fn register_commands(&self, _host: &mut dyn ServerHost, event: &mut RegisterCommandsEvent<'_>) {
        support::declare_commands(event.commands);
    }

    /// Answer `/bwncr status|check|listen|heard|silence|allow|reload` and take the command.
    fn run_mod_command(
        &self,
        host: &mut dyn ServerHost,
        event: &mut RunModCommandEvent<'_>,
    ) -> Flow {
        support::run_command(host, event)
    }

    /// A sound is about to play on this client: upstream's `PlaySoundEvent` listener. The
    /// policy silences the five rule families and the custom list; while `/bwncr listen` is
    /// on, the name is also recorded for `/bwncr heard`.
    fn play_sound(&self, _context: &dyn LoopContext, event: &mut PlaySoundEvent<'_>) {
        support::about_to_play(event);
    }
}

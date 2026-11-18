# Licence provenance — bwncr

- Mod: `bwncr` (Bad Wither No Cookie Reloaded 3.20.4), original source:
  <https://gitlab.com/kreezxil/BadWitherNoCookie>, branch `1.21.1`.
- **Upstream publishes no `LICENSE` file.** Checked at the analysed revision
  `112fa60559f1152141eb38515c9b85ada36b3b9f` (2026-05-24, the jar's build): the
  repository root has no licence file, and neither does the GitLab project's
  licence field.
- The authoritative declaration is in the **jar the pack ships**,
  `ports/_shared/licence-cache/bwncr-neoforge-1.21.1-3.20.4.jar`,
  `META-INF/neoforge.mods.toml`:

  ```
  license="MIT"
  [[mods]]
      modId="bwncr"
      displayName="Bad Wither No Cookie Reloaded"
      authors="Kreezxil, Eleksploded"
  ```

  Its `curseforge` page carries the same MIT declaration. `LICENSE` in this
  repository reproduces the MIT text with that authorship; the copyright holder
  is not named anywhere upstream, so none is invented.

## What this package carries from upstream

**Only the mod's own language files**: `assets/bwncr/lang/{en_us,es_ar,pt_br}.json`
(18 keys each — the config screen's labels and the autoconfig titles), carried under the
same MIT grant the jar declares. There is no block, item, model, texture, recipe or loot
table in this port, so nothing else of upstream's is present; the Rust in `src/` is this
campaign's clean-room rewrite from the behaviour recorded in `FINDINGS.md`.

## How the notices travel

The SDK package is one flat `.hmod`, and its packer includes the root documents:
`LICENSE` and this `LICENSE-NOTICE.md` are both inside `bwncr.hmod` (see
`unzip -l bwncr.hmod`), so a downloaded copy carries them.

## Status of the behaviour

The client-side silencing **is** carried (`Hooks::play_sound`, NeoForge's
`PlaySoundEvent`); the config, the `/bwncr` commands and the listen→heard feed are
carried. See `notes.md` → "Migration to the SDK format" for the gap list and the ladder.

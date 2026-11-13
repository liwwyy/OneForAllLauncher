## OneForAllLauncher 2.10.2 — local testing, not released

- Integrate stable upstream OneClient 2.10.2, commit `43815efac80072c255528d558f6325534327e4ec`, retaining fork accounts, local skin/head-preview hooks, private settings, shared-library choices and signed updater identity.
- Include upstream context-menu dismissal and overlay fixes, grid-card badge outlines, responsive wardrobe layout, instance migration and browser compatibility overrides.
- Include upstream group renaming, the Special chat tab, paginated social endpoints, settings search registration and the partial ZIP64 modpack import fix. The new `inventory` dependency comes from upstream's settings-search implementation.
- Add a pink macOS / Windows / Linux platform badge to the README.
- Adapt the translation-key audit to include registered settings-search titles and descriptions, retaining checks for genuinely missing or unused translations.

This update is held locally for testing. Published downloads remain at fork 2.10.1; do not publish or replace existing releases without owner approval. UI/game behavior and platform packages still require owner verification.

## OneForAllLauncher 2.10.1 — upstream OneClient 2.10.1

- Integrate stable upstream OneClient 2.10.1, commit `ec784c912792f619dd88c788f97d7477be361bd2`, preserving the fork's account features, private settings, shared-library choices and signed updater identity.
- Include upstream folder resource-pack discovery/toggling and shared disabled-pack handling, special characters in instance names, contrast fixes, crash-dialog log-upload controls, instance-settings ordering and the social-window header changes.
- Include the upstream Windows dedicated-GPU default and read-only dedicated-folder sync fixes.
- Retain the existing authlib/legacy skin-rendering paths and provider-specific compatibility scope.
- Fix local offline skin head previews in PolyPlus/OneConfig by supplying the selected offline player's private profile response for direct lookups. Keep other players, authenticated account kinds and remote TLS unchanged. The owner confirms offline and Ely.by skins and head previews work in game.
- Document PolyPlus showing Ely.by accounts as Microsoft as a display quirk, without maintaining a modified PolyPlus build. Add an in-game Ely.by skin screenshot to the README gallery and record maintenance rules in AGENTS.md.

Validated with the 18-version skin matrix, real Fabric loaders, Secure Skins and the shipped OneConfig head reader, alongside Rust tests and a Linux debug build. Windows GPU behavior and Windows/macOS package compatibility still require platform-specific verification. README downloads remain at the published fork 2.10.0 assets until fork 2.10.1 packages are published.

## OneForAllLauncher 2.10.0 — upstream OneClient 2.10.0

- Rebuild the fork on stable upstream OneClient 2.10.0, retaining its presentation, components and instance export behavior.
- Preserve standalone offline accounts, both name generators, optional online-name lookup, invalid-name overrides, Ely.by and custom Yggdrasil sign-in.
- Allow equal account names across different account types, preserving legacy account selections and Minecraft profile UUIDs.
- Add automatic and explicit cache-only launches, a launch-button context menu and a per-instance Always launch Offline setting. Installed game files and Java are required.
- Add an offline account wardrobe with local PNG skins. The source-built client hook uses shared authlib and legacy skin paths across versions/loaders and never loads for other account types. Minecraft 1.8.9/Ornithe has been tested in game by the owner.
- Preserve local skin loading under restricted classloaders and HTTPS-enforcing skin mods, while leaving remote TLS unchanged. Validate with real authlib, Fabric loaders and Secure Skins. The owner confirms skins now work on modern versions after this correction.
- Restore the automatic-update choice between Preferences and Summary during initial setup, preselect automatic updates, and allow Enter to confirm without changing existing installs' preferences.
- Default fresh setup to stock OneLauncher's active game folder, including configured custom locations, while preserving existing fork installs' paths and private accounts/settings. Add library choices under Launcher Settings → Folders and files; share catalogs, game files and managed Java without nesting or copying libraries. Protect reused libraries from old-copy cleanup.
- Document unsigned-installer warnings, per-app Windows/macOS opening steps and macOS compatibility checks.
- Make the scoped VanillaHUD preview workaround optional while retaining authlib-injector authentication.
- Restore upstream artwork, use theme-aware account icons, and retain fork storage, installation, protocol and signed updater identities for upgrades and side-by-side use.
- Limit fork contributions to offline accounts, Ely.by and custom authentication support. Follow upstream stable releases, usually within a day after validation.

The owner confirms shared OneClient libraries, offline launching, local skins on Ornithe and modern versions, and opening OneConfig HUD with Ely.by. Packaged builds are produced by CI. macOS hardware testing has not been available to the maintainer.
Windows installers and macOS bundles are not OS-signed/notarized; signed updater packages retain the fork's existing verification identity. See [installation guidance](https://github.com/liwwyy/OneForAllLauncher/blob/main/docs/installation-troubleshooting.md).
Older release notes remain in [GitHub Releases](https://github.com/liwwyy/OneForAllLauncher/releases).

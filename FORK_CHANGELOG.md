## OneForAllLauncher 2.13.1 — upstream OneClient 2.13.0

- Restore OneLauncher's standard **Launch** action for all accounts. Offline accounts retain their local skins and capes.
- Fix Microsoft Java installation when Microsoft republishes a build under the same archive filename.
- Add an x86_64 Linux Flatpak download.

## OneForAllLauncher 2.13.0 — upstream OneClient 2.13.0

- Update to stable OneClient 2.13.0.
- Use the Wardrobe with standalone offline accounts to import skins and custom capes, select Minecraft capes and save outfits. These appearances are visible only to you.
- Show a Local label for offline accounts and a dedicated message when opening Wardrobe with Ely.by.
- Retire the VanillaHUD preview workaround following the upstream fix.

## OneForAllLauncher 2.12.1 — upstream OneClient 2.12.1

- Update to stable OneClient 2.12.1 while retaining the fork's account features, local skins/head previews, shared-library choices and updater identity.
- Include upstream fixes for auto-hide, CurseForge category filters, session streaks, macOS touchpad scrolling, onboarding and custom-theme text colors; restore the Versions navigation button.
- Include upstream macOS background-close defaults and memory-return tuning for games using ZGC with at least 8 GB allocated.
- Pin the Nix updater's actions to immutable commit SHAs and refresh the Nix package to the published fork 2.12.0 asset.

## OneForAllLauncher 2.12.0 — upstream OneClient 2.12.0

- Update to stable OneClient 2.12.0 while retaining offline accounts, Ely.by/custom authentication, local skins and head previews, shared libraries and fork updater identity.
- Include upstream GraalVM support, bundle-channel selection, appearance and wardrobe improvements, and supported video previews in mod descriptions.
- Include upstream log-viewer performance improvements, OneDrive folder warnings, Windows updater exit handling, ARM Java selection fixes and instance/bundle maintenance fixes.

## OneForAllLauncher 2.11.0 — upstream OneClient 2.11.0

- Update to stable upstream OneClient 2.11.0 while retaining offline accounts, Ely.by/custom authentication, local skins and PolyPlus head previews.
- Add upstream's Copy Mod Files option, options.txt editor, individual mod update checks, bundle selection during instance creation, and expanded instance sharing/export controls.
- Include upstream fixes for overlapping/interrupted Java installations, download integrity, partial downloads on unstable connections, and preserving shared resource packs/shaders when deleting an instance.
- Include upstream fixes for chat focus crashes, tooltips and overlays, bundle/mod visibility, Fabric loader overrides, log privacy and macOS menu-bar icon sizing.

## OneForAllLauncher 2.10.2 — upstream OneClient 2.10.2

- Integrate stable upstream OneClient 2.10.2, commit `43815efac80072c255528d558f6325534327e4ec`, retaining fork accounts, local skin/head-preview hooks, private settings, shared-library choices and signed updater identity.
- Include upstream context-menu dismissal and overlay fixes, grid-card badge outlines, responsive wardrobe layout, instance migration and browser compatibility overrides.
- Include upstream group renaming, the Special chat tab, paginated social endpoints, settings search registration and the partial ZIP64 modpack import fix.
- Add a pink macOS / Windows / Linux platform badge to the README.

## OneForAllLauncher 2.10.1 — upstream OneClient 2.10.1

- Integrate stable upstream OneClient 2.10.1, commit `ec784c912792f619dd88c788f97d7477be361bd2`, preserving the fork's account features, private settings, shared-library choices and signed updater identity.
- Include upstream folder resource-pack discovery/toggling and shared disabled-pack handling, special characters in instance names, contrast fixes, crash-dialog log-upload controls, instance-settings ordering and the social-window header changes.
- Include the upstream Windows dedicated-GPU default and read-only dedicated-folder sync fixes.
- Retain the existing authlib/legacy skin-rendering paths and provider-specific compatibility scope.
- Fix local offline skin head previews in PolyPlus/OneConfig by supplying the selected offline player's private profile response for direct lookups. Keep other players, authenticated account kinds and remote TLS unchanged.
- Document PolyPlus showing Ely.by accounts as Microsoft as a display quirk, without maintaining a modified PolyPlus build. Add an in-game Ely.by skin screenshot to the README gallery and record maintenance rules in AGENTS.md.

## OneForAllLauncher 2.10.0 — upstream OneClient 2.10.0

- Rebuild the fork on stable upstream OneClient 2.10.0, retaining its presentation, components and instance export behavior.
- Preserve standalone offline accounts, both name generators, optional online-name lookup, invalid-name overrides, Ely.by and custom Yggdrasil sign-in.
- Allow equal account names across different account types, preserving legacy account selections and Minecraft profile UUIDs.
- Add automatic and explicit cache-only launches, a launch-button context menu and a per-instance Always launch Offline setting. Installed game files and Java are required.
- Add an offline account wardrobe with local PNG skins. The source-built client hook uses shared authlib and legacy skin paths across versions/loaders and never loads for other account types.
- Preserve local skin loading under restricted classloaders and HTTPS-enforcing skin mods, while leaving remote TLS unchanged.
- Restore the automatic-update choice between Preferences and Summary during initial setup, preselect automatic updates, and allow Enter to confirm without changing existing installs' preferences.
- Default fresh setup to stock OneLauncher's active game folder, including configured custom locations, while preserving existing fork installs' paths and private accounts/settings. Add library choices under Launcher Settings → Folders and files; share catalogs, game files and managed Java without nesting or copying libraries. Protect reused libraries from old-copy cleanup.
- Document unsigned-installer warnings, per-app Windows/macOS opening steps and macOS compatibility checks.
- Make the scoped VanillaHUD preview workaround optional while retaining authlib-injector authentication.
- Restore upstream artwork, use theme-aware account icons, and retain fork storage, installation, protocol and signed updater identities for upgrades and side-by-side use.
- Limit fork contributions to offline accounts, Ely.by and custom authentication support. Follow upstream stable releases, usually within a day after validation.

Windows installers and macOS bundles are not OS-signed/notarized; signed updater packages retain the fork's existing verification identity. See [installation guidance](https://github.com/liwwyy/OneForAllLauncher/blob/main/docs/installation-troubleshooting.md).
Older release notes remain in [GitHub Releases](https://github.com/liwwyy/OneForAllLauncher/releases).

## OneForAllLauncher 2.10.0 — pending release validation

- Rebuild the fork on stable upstream OneClient 2.10.0, retaining its presentation, components and instance export behavior.
- Preserve standalone offline accounts, both name generators, optional online-name lookup, invalid-name overrides, Ely.by and custom Yggdrasil sign-in.
- Allow equal account names across different account types, preserving legacy account selections and Minecraft profile UUIDs.
- Add automatic and explicit cache-only launches, a launch-button context menu and a per-instance Always launch Offline setting. Installed game files and Java are required.
- Add an offline account wardrobe with local PNG skins. The source-built client hook uses shared authlib and legacy skin paths across versions/loaders and never loads for other account types. Minecraft 1.8.9/Ornithe has been tested in game by the owner.
- Restore the automatic-update choice between Preferences and Summary during initial setup, preselect automatic updates, and allow Enter to confirm without changing existing installs' preferences.
- Offer OneClient's default data folder during onboarding, reusing its catalog and game files while keeping fork accounts/settings separate. Recognize existing custom libraries without creating a nested data directory.
- Document unsigned-installer warnings, per-app Windows/macOS opening steps and macOS compatibility checks.
- Make the scoped VanillaHUD preview workaround optional while retaining authlib-injector authentication.
- Restore upstream artwork, use theme-aware account icons, and retain fork storage, installation, protocol and signed updater identities for upgrades and side-by-side use.
- Limit fork contributions to offline accounts, Ely.by and custom authentication support. Follow upstream stable releases, usually within a day after validation.

The owner must complete UI, game and packaged-upgrade validation before publication.
Older release notes remain in [GitHub Releases](https://github.com/liwwyy/OneForAllLauncher/releases).

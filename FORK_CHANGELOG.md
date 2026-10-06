# OneForAllLauncher 2.6.2 — 2026-10-06

OneForAllLauncher, a fork of OneClient by Polyfrost.

- Use the supplied transparent artwork for launcher, window, tray and installer icons.
- Add an Exporting section to instance settings with Export mods.zip and Export prism instance.
- Export real mod files from instance symlinks and enabled cached mods; respect disabled mods.
- Include Minecraft options, config/defaultconfigs and OneConfig settings in Prism archives.
- Preserve the supplied Ornithe Gen2 1.8.9 Prism patches, with the instance's selected loader and name.
- Generate standard Prism metadata for Vanilla, Fabric, Quilt, Forge and NeoForge instances.
- Save ZIPs atomically so failed exports do not overwrite previous files.

## OneForAllLauncher 2.6.1 — 2026-10-06

OneForAllLauncher, a fork of OneClient by Polyfrost.

- Create and use offline accounts without a Microsoft account.
- Add offline accounts during onboarding and in Settings → Accounts.
- Separate executable, IPC, protocol, installer and data directory identities from upstream.
- Replace launcher icons and branding with the supplied artwork.
- Disable upstream automatic updates and OneClient v1 migration.
- Tolerate transient database writer contention during concurrent mod installation.

Original work copyright Polyfrost and its contributors. Fork modifications copyright
2026 OneForAllLauncher contributors. Licensed under GPL-3.0-only, without warranty.
See LICENSE and ATTRIBUTION.md. Binary distributions must provide the corresponding
source and these notices.

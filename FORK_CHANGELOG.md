# OneForAllLauncher 2.6.4 — 2026-10-06

OneForAllLauncher, a fork of OneClient by Polyfrost.

- Fix Ely.by sign-in rejecting the server's long-lived compatibility token expiry; accept non-expiring tokens and validate expiry arithmetic without an arbitrary one-year limit.
- Log Ely.by sign-in failures without logging access tokens, refresh tokens or device codes.
- Keep Add Ely.by beside the Microsoft and offline account buttons in account settings.
- Correct the header and onboarding wordmark to OneForAll, with blue One and yellow ForAll and matching glow.
- Render the original transparent launcher icon directly with smooth image sampling at small sizes.
- Add boxed Choose all, recommended for personal use and recommended for distribution presets to the Prism export picker.
- Personal exports select mods/configs, options, shader/texture/resource packs, worlds and server lists. Distribution presets select mods and configs, excluding worlds, server lists and personal options.
- Fix checkbox flicker when expanding export folders by using the selection model directly and giving file rows stable keys.
- Preserve saved custom export selections and the optional Prism import preference.

## OneForAllLauncher 2.6.3 — 2026-10-06

OneForAllLauncher, a fork of OneClient by Polyfrost.

- Replace the duplicated header emblem with the AllForOne wordmark and a warm white glow.
- Add Ely.by device-code sign-in, refresh, profile skins and Minecraft authentication through authlib-injector.
- Use the launcher's registered public Ely.by client ID `oneforalllauncher`; override at build time with `ONEFORALL_ELYBY_CLIENT_ID` if needed.
- Give Microsoft, offline and Ely.by onboarding buttons equal widths and sizes, with the supplied offline and themed Ely.by icons.
- Show Prism exporting only for Ornithe 1.8.9; use oneclient.png as the exported instance icon.
- Add an expandable file/folder picker, per-instance saved selections, and optional automatic ZIP import in Prism.
- Export in the background with filenames, file counts, bytes, rate and ZIP finalization in live notifications.
- Name exported ZIPs after the instance and Minecraft version.
- Integrate the five upstream commits through 0f21171f: opt-in bundle fixes, mods folder synchronization, URL/GPU fixes, and custom game arguments.
- Preserve the previous README as README_LOCAL.md and leave README.md empty for replacement.

## OneForAllLauncher 2.6.2 — 2026-10-06

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

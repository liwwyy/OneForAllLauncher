## OneForAllLauncher 2.6.9 — 2026-10-10

- Restore OneClient's name and stock modpack icon in instance creation, instance lists, version labels and shared modpack descriptions.
- Add an optional, initially unchecked online username lookup to offline account creation in onboarding and account settings. Matching online names and lookup failures do not prevent creating an offline account.
- Merge the 16 upstream commits through `1d6ec598`, including OneClient 2.8.0, parallel Minecraft instances, signed-out Microsoft session handling, bundled-mod fixes, notification improvements and the updated Freya/Skia fixes.

## OneForAllLauncher 2.6.8 — 2026-10-08

- Make the custom-account icon white in onboarding, account settings and the sign-in dialog.
- Update the README screenshots for onboarding, account management and offline account creation.

## OneForAllLauncher 2.6.7 — 2026-10-08

- Use the supplied custom authentication icon in onboarding, account settings and the sign-in dialog.
- Replace the two-stage custom sign-in form with an inline trust toggle and one Add account action.
- Move login/refresh paths under Options, with Freesm-style autofill that preserves custom paths.
- Compile the preview compatibility agent from Java source during Cargo and release/nightly builds; remove the checked-in prebuilt JAR.
- Document the JDK/Python build requirements and verified ASM dependency cache.

## OneForAllLauncher 2.6.6 — 2026-10-08

- Validate offline usernames while typing; add an explicit invalid-name override and random characters/username generators in onboarding and account settings.
- Add custom Yggdrasil accounts with server discovery, configurable login/refresh paths, credential destination review, profile selection, token refresh and authlib-injector launches.
- Restore the stock ONECLIENT wordmark without glow, keeping the custom launcher icon.
- Add a matching release-download badge and document the bundled Ely.by preview compatibility agent in the README.
- Limit release notes to the current version's changes.

# OneForAllLauncher 2.6.5 — 2026-10-06

OneForAllLauncher, a fork of OneClient by Polyfrost.

- Start Ely.by browser authorization only from an explicit Add Ely.by click; reopening account settings no longer replays a cached sign-in result.
- Keep sign-in progress/errors local to the requested flow, ignore cancelled attempts, and cancel active authorization when leaving its screen.
- Fix the VanillaHUD tab preview crash exposed when opening OneConfig with an Ely.by account on Minecraft 1.8.9. Resolve missing preview names through Mojang's public profile lookup, or use stable safe preview labels when unavailable.
- Scope the compatibility agent to VanillaHUD's preview profile method and Ely.by 1.8.9 launches; preserve named profiles, UUIDs, textures and normal account/server authentication.
- Include auditable Java 8 agent source, a deterministic rebuild script, console-only regression tests and the bundled ASM license.

## OneForAllLauncher 2.6.4 — 2026-10-06

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

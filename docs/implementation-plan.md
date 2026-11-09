# Proposed application changes

Status: implementation authorized and present on the local `rewrite/2.10.0`
branch. Automated validation is recorded in [validation.md](validation.md).
UI/game and packaged-upgrade acceptance checks remain with the owner. No remote
history rewrite or release has been published.

Build the agreed replacement fork history on stable upstream `oneclient-2.10.0`
(`3968fb95f98569de41ec341ef0da2ed63802cfc4`). Retain all existing account features,
upgrade paths and the technical identities listed in [AGENTS.md](../AGENTS.md).
Use upstream Freya components, styling, layout and interaction patterns throughout.

Every fork-specific feature change must directly support standalone offline
accounts, Ely.by or custom authentication servers. Keep the upstream diff minimal
and maintainable. The proposals below are limited to those account features and
their necessary integration; unrelated launcher improvements belong upstream.

## VanillaHUD compatibility

GitHub inspection on 2026-11-09 found an existing fix for the crash that motivated
the preview agent:

- [Commit f1c8c189](https://github.com/Polyfrost/VanillaHUD/commit/f1c8c189b23c1e674df9319570bf7109d480828e),
  released as VanillaHUD 3.5.8, checks for null profile names and supplies `Player`.
- The current upstream HUD bundle for Minecraft 1.8.9/Ornithe supplies
  `vanillahud-3.5.10+1.8.9.jar`, SHA-1
  `d0539f0612488a1db712e5f6906770f3f7573bfa`.
- Previous inspection in this work session verified that checksum and confirmed
  the fallback in the distributed class's bytecode. The existing preview-agent
  console suite passed on Java 25 against that JAR's transformer target. That
  suite uses simulated preview behavior; it does not run the actual HUD in game.
- A subsequent headless test invoked the distributed class's actual private
  `getProfile(UUID)` method on Java 25 with no Java agents. Minecraft/session
  services were stubbed and authlib 1.5.21 was real. Null profiles, unnamed
  profiles and session lookup exceptions all returned a named fallback with the
  original UUID; a named Ely.by-style profile passed through unchanged. All four
  checks passed. This verifies the fixed method, not the entire in-game renderer.
- Searching open issues and pull requests found no matching unresolved Ely.by,
  authlib or null-preview-name report. Open scoreboard positioning work is a
  separate issue. Recheck before filing any new report.

Keep a persisted toggle for the scoped preview workaround, separate from
authlib-injector. Preserve current behavior for old settings initially. With it
disabled, omit both its Java agent argument and bridge classpath entry. Never
disable the authentication agent as a side effect.

The owner should test an Ely.by account and a custom account on Minecraft 1.8.9,
using the current HUD bundle with the preview workaround disabled. Open OneConfig,
edit the tab-list HUD, exercise preview heads, and reopen the editor. Include an
unreachable profile service in the test. Compare behavior with the workaround
enabled if the crash recurs. No screenshots or automated visual comparisons.

If a failure is reproduced, help prepare an upstream issue with the minimal mod
set, Minecraft/loader/OneConfig/VanillaHUD/Java versions, authlib-injector version,
sanitized stack trace and precise reproduction steps. State that the preview
agent was disabled. Link the existing fix. Prepare a narrowly scoped PR if the
remaining failure can be explained and tested; do not file a duplicate report
merely because the launcher still includes its historical workaround.

## Offline launching

### Existing upstream behavior

Upstream already has offline accounts, cached metadata, transient authentication
failure fallbacks, signed-out Microsoft singleplayer launches, and a missing-file
"Launch anyway" prompt. It does not provide the requested explicit launch policy,
right-click action or per-instance always-offline setting.

`oneclient_net/src/status.rs` publishes `ServiceStatus` through a Tokio watch
channel. Its connectivity probe timeout is eight seconds. `StatusBar` subscribes
to the same status to display the no-internet banner. Probes currently run on
startup or a requested recheck; there is no idle recovery timer. Network request
timeouts are separate and can be longer, so changing only the button label will
not prevent launch delays.

### Launch policy and controls

Resolve one launch policy for every request, shared by the UI and launch pipeline:

| Condition | Effective mode | Idle launch label |
| --- | --- | --- |
| Online, instance preference off | Normal | Existing normal label |
| Existing status reports no internet | Offline | `Launch (Offline)` |
| Instance preference on | Offline | `Launch (Offline)` |
| Right-click `Launch Offline` | Offline for this launch only | Does not persist a preference |

Keep checking, waiting, running, stopping and disabled states ahead of idle labels.
Internet recovery restores normal labels unless the instance preference remains
enabled. Dismissing the no-internet banner must not change the launch policy.
An outage at Microsoft or Polyfrost alone is not proof that the internet is down.

Reuse the existing timeout, probe and subscription. Add a single bounded recovery
recheck while offline, plus rechecks on existing request/focus triggers as useful,
so the launcher notices recovery even when no other requests are being made.
Avoid a second competing connectivity detector or a per-button polling task.
Freeze the chosen mode for a launch attempt so recovery does not start downloads
or cloud synchronization halfway through preparing an offline session.

Use the existing ContextMenu for right-clicking each launch button, with a
`Launch Offline` entry and the offline icon. Right-click must not also trigger
the normal launch. Preserve existing focus, keyboard and dismissal behavior.

Under instance settings' existing **Instance** section, add a standard settings
row with the offline icon and the toggle **Always launch Offline**. Default it to
false for existing and new instances. Persist it per instance, not in a shared
game-settings profile that could change multiple instances together. Use a new
additive database migration and update the DAO, row/model and saved instance
metadata paths as needed; never edit an already-applied migration.

### Launch pipeline

Thread the policy through `Actions` and the core launch functions so it also
applies to tray, shortcut, URL-handler and pending-launch requests. Offline mode
should:

- Use the selected account's saved name and game UUID without account renewal,
  sign-in prompts or creating/replacing an account. Keep stored credentials and
  the selected account unchanged; use a launch-only credential snapshot.
- Use installed Minecraft, loader, Java, libraries, assets, bundled mods and
  cached metadata. Do not refresh loader versions, mods, modpacks or block on
  Java offers, share updates, cloud synchronization or optional remote services.
- Retain local directory ownership, unfinished-reset, dependency, native-library
  and game-file checks. An installed game should launch without a chain of
  network retries. Do not attempt network repair from the offline launch path.
- Identify missing required local files clearly. A first installation or a
  missing runtime cannot be completed without the necessary files. Reuse the
  existing incomplete-assets decision where appropriate; do not promise that
  all partially installed games can run.
- Use an already-cached, verified authlib-injector when the chosen provider
  requires it. Missing authentication/compatibility artifacts must not trigger a
  download in explicit offline mode. Review preview-agent network lookups too.
- Avoid reporting online party/invite or cloud operations as successful. Offline
  launching does not provide access to servers that require online authentication
  and does not control network activity started by Minecraft or its mods.

At the UI boundary, share the policy and menu behavior while preserving each
existing button's dimensions and styling. Audit at least:

- Home/active-instance hero (`components/active_cluster_panel.rs`).
- Instance detail header (`layout/cluster_shell.rs`).
- Instance list/grid/sidebar launch controls (`view/app/clusters/page.rs`).
- Invite launch controls and the debug launch control.
- Tray "Play last", desktop shortcuts and URL launches through IPC/pending launch.

Add translations using upstream 2.10.0's existing i18n mechanism.

### Acceptance checks

Use fake connectivity and counted/mocked requests to test online/offline/recovery
transitions without UI screenshots. Verify both manual and automatic offline
launches skip launch-path network calls, preserve account/game UUIDs, respect
busy states and default old instances to false. Cover cold startup without
internet, recovery while idle, connection loss during launch, cached versus
missing files, expired credentials and every launch entry point. The owner
manually tests the game launches, context menus, labels and settings row.

## Same-name accounts from different account types

Allow case-insensitively equal usernames across different `AccountKind` values.
Reject adding a distinct account with the same name and same kind. Reauthenticating
an existing account is an update, not a duplicate. Apply the rule consistently
to offline, Microsoft, Ely.by and custom additions; two custom servers are still
the same account type for this username rule.

The current offline insertion rejects names across all types, while other
providers do not share that username check. Centralize that narrow validation.
Account rows and switcher entries must display their provider so equal names are
distinguishable.

There is also a storage constraint: credentials are currently keyed solely by
game UUID and explicitly reject cross-provider UUID collisions. A complete
implementation must support distinct launcher account identities for colliding
providers without changing either account's Minecraft UUID. Design the storage
key and backward-compatible migration before changing this case. Preserve the
default selection, refresh isolation and provider-scoped skin/cache keys; never
silently overwrite an account to accept a duplicate name.

Tests should cover name casing, every provider pair, same-type rejection,
reauthentication, equal UUIDs across different providers, legacy `auth.json`
loading and persistence of the selected/default account after an upgrade.

## Account icons

Create the Ely.by button icon as a small monochrome SVG mark consistent with
upstream's line icons: a 24-unit coordinate space, matching stroke weight, simple
geometry and theme-controlled color. Avoid the current colored tile/background.
Add a matching pixel variant if retaining a distinct icon in Minecraft themes.

Implemented locally: custom-account buttons in onboarding, account settings and
the sign-in dialog now reuse the stock `CodeSnippet02` icon. The fork-specific
`Custom` icon variant, forced-white override and duplicate SVG are removed. The
standard icon inherits the button's theme foreground, including disabled states.
Existing button and icon sizes are preserved; the wider stock geometry replaces
the old narrow glyph. Actual launcher UI review remains with the owner.

Use a consistent upstream-style offline icon for the new menu and settings row.
The owner reviews all icon changes manually in light, dark and Minecraft themes.

### Ely.by icon design method

Untitled UI's official [icon design guidance](https://www.untitledui.com/components/icons)
specifies a 24-by-24 frame and a two-unit stroke, and recommends Figma's
[reusable icon grid tutorial](https://help.figma.com/hc/en-us/articles/18770195788951-Create-a-reusable-icon-grid)
for custom drawings. Use those construction rules with the actual launcher SVGs
as the integration reference:

1. Import `globe-01.svg`, `users-01.svg` and `code-snippet-02.svg` from the launcher's
   assets into Figma. Keep a 24-by-24 reference frame for each.
2. Create a new 24-by-24 frame with a one-unit grid and approximately a 20-by-20
   central drawing area. Use centered two-unit strokes, rounded caps/joins and
   no background. Match optical size against the reference icons rather than
   stretching the artwork to fill the frame.
3. Draw the owner's requested `E` enclosed by a box using vector paths. A starting
   geometry is an outlined rounded square at `(3, 3)`, size `18 by 18`, radius `3`,
   with an `E` spanning approximately `x=9..15`, `y=7..17`. Use the same two-unit
   stroke for box and letter. Keep the adjacent Ely.by text label. This geometry
   is a project-specific starting point, not an official Untitled UI prescription.
4. Keep an editable 24-unit master. For manual previews at 20 and 16 pixels, scale
   geometry and strokes together rather than retaining a two-pixel stroke at
   every display size. The final SVG's viewBox remains `0 0 24 24`.
5. Export SVG and normalize its presentation to `fill="none"`,
   `stroke="currentColor"`, `stroke-width="2"`, `stroke-linecap="round"` and
   `stroke-linejoin="round"`. Keep editable stroked geometry for this line icon;
   remove editor metadata and embedded images. Use the existing `IconType::Elyby`
   path and theme tint, with no hardcoded white or brand-color tile.
6. The owner manually checks 16-, 20- and 24-pixel rendering alongside the
   launcher icons in the actual account buttons. Check light/dark themes and
   hover/disabled states. Add a separate pixel variant for Minecraft themes if
   the new symbol is retained there.

## Local wardrobe for offline accounts

Update: the owner confirms the 1.8.9/Ornithe hook works in game. The implementation
now uses shared authlib APIs across loaders and legacy skin URL handling, with
no Minecraft-version injection restriction. The offline-account gate remains.
A loopback-only PNG endpoint supplies native texture downloads without external
requests. The original Ornithe renderer remains separately gated to its verified
mapping set. See [agent compatibility/tests](../distribution/offline-skin/README.md)
for the tested API/runtime matrix and remaining game-testing limits.


Allow offline accounts to open a limited wardrobe that reuses the existing skin
picker, dialogs and `PlayerModel::skin` preview path. Support selecting a local
PNG, choosing the classic/slim model, applying it and restoring the default skin.
Validate image decoding and Minecraft skin dimensions before saving. Store the
skin and model locally under the existing fork data directory, keyed by the
launcher account identity so equal names do not share a skin accidentally.

Do not initialize Microsoft/Poly+ wardrobe queries for this branch. No account
registration, token renewal, skin upload, store, purchases, cosmetics service,
cloud skin library or remote skin-distribution service is required. Clearly
describe the skin as visible only on this computer. Keep authenticated wardrobe
behavior intact for Microsoft accounts.

Gate any local game-skin hook on `AccountKind::Offline`, a selected local skin and
a supported Minecraft/loader combination. Check that condition in the launcher
before adding any hook, agent arguments or classpath entries. Microsoft, Ely.by
and custom accounts must never load this hook, including when disconnected or
using `Launch Offline` / `Always launch Offline`. Internet connectivity and the
launch-mode preference do not change an account's type. This gate does not change
the separate authlib-injector or VanillaHUD preview-agent rules.

The launcher already accepts raw skin bytes for its 3D preview, but its current
"apply skin" actions upload to Mojang and record through Poly+. No local game-skin
override was found in the audited launch pipeline. A launcher preview alone does
not apply a skin inside Minecraft.

`PlayerModel` currently calls `use_player_skin` even when explicit skin bytes are
provided. Reusing it for the local wardrobe therefore also needs a query-enabled
or local-only path that avoids remote profile/image lookups while preserving
Freya's hook ordering. Do not assume a preview override already prevents requests.

Investigate an existing client-side hook in the bundled OneClient mods first.
If none provides a private local override, add a minimal, separately scoped
client mod or Java agent. Build it from tracked source, pin/verify dependencies,
retain licenses and use tested version/loader-specific mappings. Never broaden
the historical VanillaHUD preview agent into an unrelated general patcher.
Only the local player's rendering may be changed: no server-side profile changes,
no other players' skins and no texture upload or broadcast. Unsupported game
versions must not silently claim that the skin was applied in game.

Tests must distinguish local wardrobe preview from actual in-game skin behavior.
Verify persistence across upgrades/restarts, model selection, default restore,
no service/registration requests, same-name account isolation and no effect on
other players' rendering. For every other account kind, assert that no local
skin-hook arguments or classpath entries are injected in either launch mode,
including when a saved local skin exists. The owner verifies in-game behavior on supported
Minecraft/loader combinations before the feature is described as complete.

## Suggested implementation order

1. Rebuild the retained fork features over upstream 2.10.0 with upgrade tests.
2. Add the preview compatibility toggle and perform the agent-disabled game test.
3. Fix account uniqueness and any required backward-compatible storage identity.
4. Implement offline launch policy and per-instance persistence, then all UI entry
   points and translations, with focused tests in the same behavior commits.
5. Complete account icon normalization through the stock icon/theme pipeline;
   the custom-account icon cleanup is already implemented locally.
6. Implement the local wardrobe and verify its game integration separately.
7. Complete owner UI/game testing and signed upgrade/package verification before
   publishing `oneforall-2.10.0`.

Keep each commit reviewable and avoid mixing a future feature's partial UI into
an earlier commit that cannot support its advertised behavior.

## Optional upstream library reuse

Onboarding keeps This PC as the fork's default and adds OneClient default folder
alongside Another drive. Resolve the upstream release path using the same legacy
settings marker as upstream, without changing fork configuration/account identity.
The explicit upstream choice applies the exact directory. Custom folder selection
recognizes the existing SQLite catalog read-only so a valid library is reused
without a nested OneClient folder. Do not replace upstream settings or auth files.

Sharing targets matching stable versions, with one launcher open at a time.
Preserve shared migration checksums and verify upstream → fork → upstream database
round trips with a temporary library. Automatic updates are preselected in fresh
setup, with Next focused for Enter; Next confirms the displayed preference.

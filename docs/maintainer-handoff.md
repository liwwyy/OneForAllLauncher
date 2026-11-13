# Maintainer handoff: local 2.10.2 update

Snapshot: 2026-11-13. Read [AGENTS.md](../AGENTS.md) before making changes.
The owner requested a pink supported-platform badge and integration of stable
upstream 2.10.2, with commits held locally for testing. Local commits are
authorized; do not push, dispatch release workflows or publish this update until
the owner authorizes the next step.
The earlier 2.10.1 publication authorization does not apply to this update.

## Source and publication state

- Current branch: `main`. Starting fork commit: `b6b1456d`, the README download
  refresh after the 2.10.1 release. The working version is **2.10.2**.
- Stable upstream `oneclient-2.10.2` resolves to
  `43815efac80072c255528d558f6325534327e4ec`. Its ten commits since 2.10.1
  are integrated by local merge `ee05874e`, retaining upstream ancestry.
  Local commit `a61a910c` adjusts the translation audit for registered search keys.
  A separate documentation commit adds the platform badge and update notes.
  Review the complete update with `git diff b6b1456d..HEAD`. Nothing is pushed.
- Fork 2.10.1 was published from `4e119418`. Its eight application packages and
  checksum file are linked in the README. Keep these actual downloads until
  fork 2.10.2 is published; do not fabricate links to unbuilt packages.
- Old local archive branches and remote `oneforall` were backed up and removed.
  Keep remote `main` and the active `badges` branch. Do not repeat the completed
  history rewrite, remove published tags/assets or force-push future updates.

## Changes in this update

Read [the upstream changelog](../CHANGELOG.md) and [fork notes](../FORK_CHANGELOG.md).
Upstream fixes context-menu dismissal/overlay layering, grid-card badge outlines,
wardrobe resizing and partial ZIP64 imports. It adds instance migration, browser
compatibility overrides, social group renaming, a Special tab and paginated social
endpoints. Settings search now uses upstream's `inventory` registration mechanism.
The new dependency is part of the stable upstream release, not a fork feature.

The first library-test run exposed an upstream audit assumption: search keys
registered through `inventory` were falsely reported as unused by the direct-call
translation scanner. A focused test-only adjustment includes registered titles
and descriptions in that audit. Keep missing/unused-key validation enabled;
do not remove real search translations or skip the audit to make tests pass.

The merge had no conflicts. Account storage, private paths, game-library choices,
launch/skin-agent gates, packaging identities and updater verification identity
remain unchanged. Keep the source-built version 5 local skin hook and scoped
VanillaHUD workaround. The README gains a pink supported-platform badge.
A pre-update recovery bundle is stored outside the repository under
`/home/user/.codex/backups/OneForAllLauncher/20261113T090947-2.10.2/`.

## Follow-up: PolyPlus offline head previews

The owner initially reported that body skins work with Ely.by and local skins, while the
PolyPlus head icon is missing only for the local skin. PolyPlus 2.0.2's account
and social heads call OneConfig's public `PlayerHead` component. The shipped
OneConfig 1.7.0 bridge makes a direct session-profile HTTP request, bypassing the
authlib methods the existing offline hook supplies.

The local hook now routes only the selected offline UUID's exact public
session-profile URL to a private loopback JSON response containing the same
unsigned local textures property. This uses the profile-routing approach that
authlib-injector provides for Ely.by, independently implemented without copying
its source. Other UUIDs, authentication endpoints, provider gates and remote TLS
are preserved. Version 5 agent/bridge files coexist with older running games.

The direct-profile regression failed before the fix. The new test executes the
shipped OneConfig `skinUrl`/`httpGet` bytecode and regex patterns, with game/Compose
initialization stubbed and external HTTPS blocked. It passes for the private
profile-to-PNG path. See [agent verification](../distribution/offline-skin/README.md#polyplus-head-previews).
Owner confirmation (2026-11-12): offline and Ely.by skin hooks and skin head
previews work in game with the updated build. No additional game/loader version
coverage was specified. No automated UI screenshots or visual comparisons have
been performed. This confirmation applies to 2.10.1; retest after the 2.10.2 update.

### Ely.by labels in the PolyPlus account switcher

Research only; no label integration implemented. PolyPlus source at
`276cc261fae42a9e2c9c5e9d75b3fc72c8b8d62e` exposes a Boolean `microsoft` in its
account view model and hardcodes Microsoft/Offline labels in `AccountSwitcher`.
`SessionAccounts.capture` infers an unsaved launch account's kind from whether
its UUID matches the offline UUID, classifying an Ely.by launch as Microsoft.
The inspected account code has no provider-label registration hook.

Owner decision (2026-11-12): accept and document this display quirk in the README.
Do not maintain a PolyPlus fork or add a label workaround; the extra maintenance
is unnecessary for this cosmetic issue. No provider-label integration is planned.
Do not point PolyPlus at the fork's private auth store just to supply a label:
its account writer does not preserve the fork's provider-specific fields.

Sources: [account UI](https://github.com/Polyfrost/PolyPlus/blob/276cc261fae42a9e2c9c5e9d75b3fc72c8b8d62e/src/main/kotlin/org/polyfrost/polyplus/client/gui/AccountSwitcher.kt),
[session capture](https://github.com/Polyfrost/PolyPlus/blob/276cc261fae42a9e2c9c5e9d75b3fc72c8b8d62e/src/main/kotlin/org/polyfrost/polyplus/client/launcher/SessionAccounts.kt),
[account store](https://github.com/Polyfrost/PolyPlus/blob/276cc261fae42a9e2c9c5e9d75b3fc72c8b8d62e/src/main/kotlin/org/polyfrost/polyplus/client/launcher/LauncherAccountStore.kt).

## Testing and acceptance

Current 2.10.2 checks are recorded in [validation.md](validation.md); earlier
results are historical evidence. The owner tests UI and game behavior manually.
Do not take screenshots or perform automated visual comparisons.

Retest offline/Ely.by/custom account flows and skins/head previews, the offline
launch button, account wardrobe sizing, context menus, settings search and folder
choices. Check new-instance migration, browser override prompts and ZIP64 imports
with temporary test instances. Never modify a user's live upstream accounts or
library to test compatibility. Windows GPU behavior and Windows/macOS packages
need platform-specific verification.

## Publication after owner approval

A push to `main` automatically builds and publishes an unpublished workspace
version after packaging/signing succeeds. Existing releases remain immutable.
Do not push this 2.10.2 update until authorized. Retain the fork signing identity
and updater endpoint. After publication, refresh all README package links,
architecture labels, checksum link and updater markers against the real assets.

Review managed CodeQL for the exact published source before release; a successful
analysis is not security clearance. See [security verification](security-verification.md).

[Return to the README](../README.md#maintenance).

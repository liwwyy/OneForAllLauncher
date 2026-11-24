# Local 2.12.1 integration — 2026-11-24

The owner requested local integration for manual testing and Nix action pinning
for the next push. The owner subsequently authorized commit and push on
2026-11-24. No additional manual test details were supplied; no UI automation
was performed. Starting HEAD: `fd0a4066`. Stable upstream `oneclient-2.12.1` is
pinned to `1605bf11248f8cc94eacf8ff1f86981abcab8dc8`. The integration retains stable upstream ancestry.

Keep the fork release workflow: upstream's replacement consumes changesets and
uses its Apple signing credentials/draft-release architecture, which are not the
fork's established packaging/signing pipeline. README conflicts retain the fork
entry point, gallery and deterministic next-release URLs rather than stock
OneClient installation instructions. Fork storage/auth/skin/updater identities
are retained. The optional Nix recipe uses the published fork 2.12.0 AppImage and
its published checksum; update again once fork 2.12.1 assets exist.

The Nix updater now pins checkout v5 to
`fbc6f3992d24b796d5a048ff273f7fcc4a7b6c09` and install-nix-action v31 to
`3dc140bfc76c49fbd4dd958429132e3746bcb086`, verified against their official GitHub
repositories. This prepares the alert #373 fix; resolution is not established
until the next authorized push is analyzed. No alerts were dismissed.

See docs/validation.md for build results. UI/game and platform packaging test details remain unspecified; the owner
authorized publication after the local build. Older handoffs below do not authorize publication of this
update or establish its test results.

---

# Local 2.12.0 integration — 2026-11-23

The owner initially requested local integration for manual testing, then explicitly
authorized commit and push on 2026-11-23. A push to main triggers the release
workflow. No additional manual test results were provided; no UI automation was
performed. HEAD before integration:
`cf93bd8f`. Stable upstream `oneclient-2.12.0` resolves to
`0da930e2bb99784e4715396252f1b51f6529af25`. The merge retains stable upstream ancestry.

Retain the fork's library choices, private settings/accounts, provider features,
local skin hooks and updater identity. Resolve upstream onboarding BundlePicker
alongside the fork Updates step; preserve offline cache-only Java handling.
The Windows wait-for-exit installer section is retained; upstream's stale
OneClient executable/shortcut cleanup is omitted to protect side-by-side installs.
README download URLs are prepared using existing deterministic release names.
The optional Nix recipe now points at the fork and uses its own scheme/package
identity. It remains pinned to the published 2.11.0 AppImage and published checksum;
update its version/hash after 2.12.0 assets exist. Nix execution is untested. Its updater now follows fork releases and the fork flake
attribute rather than stock OneClient releases.
See docs/validation.md for current checks. The owner authorized publication; platform/game test details remain unspecified.

The previous handoff below is historical and does not authorize publication of
this update or establish current test results.

---

# Maintainer handoff: 2.11.0 release preparation

Snapshot: 2026-11-16. Read [AGENTS.md](../AGENTS.md) before making changes.
The owner confirmed the tested 2.11.0 build works and authorized committing and
pushing this update to trigger the automatic release. No additional platform
coverage was specified.

## Source and publication state

- Current branch: `main`; starting fork commit: `ff94de8d`. Working version:
  **2.11.0**. Stable upstream `oneclient-2.11.0` is pinned to
  `85ae2298894413533fbe1f9f999b773a26ad5206` (28 commits since 2.10.2).
- The update retains upstream ancestry in a merge of the pinned stable tag.
  Review the final update against `ff94de8d`. Five conflicted files were resolved: README, component/hook
  exports, launch actions and core launch. Keep fork authentication exports and
  offline gating while passing upstream's remaining bundle budget online.
- Fork 2.10.2 was published from `ff94de8d`. Its release body copied an internal
  testing heading and status paragraph from FORK_CHANGELOG.md. The local source
  is now cleaned; the already-published GitHub release has not been edited.
- README direct downloads are prepared for 2.11.0 using the workflow's fixed
  tag/asset naming rules, per the owner's preference. No temporary publication
  notice is added. Verify these URLs against actual assets after an approved
  release; they do not establish that 2.11.0 is already published.
- Recovery bundle: `/home/user/.codex/backups/OneForAllLauncher/20261116-2.11.0/`.
  Keep published tags/assets, account data and updater signing identity intact.

## Changes and review points

Read [upstream changes](../CHANGELOG.md) and [public fork notes](../FORK_CHANGELOG.md).
Upstream adds Copy Mod Files, options.txt editing, mod update checks, bundle
selection and sharing/export controls. Fixes include Java installation races,
partial/ranged downloads, checksum verification, shared pack deletion protection,
chat focus, mod visibility, tooltip layering and log privacy.

Online launch passes upstream's bundle budget through the fork launch wrapper.
Offline launch skips update/optional-mod requests and retains its scoped network
block, token clearing, cache-only behavior and account-kind skin-agent gate.
Account serialization, private library paths, updater endpoint/key and source-built
version 5 skin/bridge remain unchanged. External lockfile package versions did
not change. A duplicate hook export left during conflict resolution was removed.

**Release notes must be publication-ready.** AGENTS.md now explicitly prohibits
internal testing headings, placeholders and status notes in public changelogs or
releases. Keep development state here and check results in validation.md. Review
`python3 distribution/release_notes.py 2.11.0` before publication. The local
changelog also removes the erroneous 2.10.2 heading and stale test/status prose.

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
been performed. The owner also tested 2.10.2 before authorizing its push. Retest the new 2.11.0 build.

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

Current results are in [validation.md](validation.md). Older results do not
establish new 2.11.0 behavior. No screenshots or visual comparisons are authorized.
Owner confirmation: the 2.11.0 test build works. Continue platform-specific
verification of offline/Ely.by/custom launches, local skins and head previews,
Copy Mod Files, bundled instance creation, settings/options editing and sharing.
Use temporary instances/libraries for deletion and migration tests; never modify
live upstream accounts or game libraries for automated verification.

## Authorized publication

The owner authorized commit and push after successful testing. A push to `main`
will automatically package,
sign and publish the new version. Review user-facing release text, managed CodeQL
for the exact release commit, updater continuity and platform packages before
publishing. Keep releases immutable unless the owner separately requests a
release-note correction. Do not replace existing tags or signing keys.

[Return to the README](../README.md#maintenance).

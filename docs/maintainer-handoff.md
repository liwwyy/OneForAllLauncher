# Maintainer handoff: 2.10.1 release preparation

Snapshot: 2026-11-12. Read [AGENTS.md](../AGENTS.md) before making changes.
The owner initially requested a local update and test build before committing.
After confirming working offline/Ely.by skins and head previews, the owner
authorized committing all pending changes, including AGENTS.md and changelog
updates, and pushing to trigger the automatic 2.10.1 release workflow.

## Source and publication state

- The existing checkout is `rewrite/2.10.0`; its name is retained. The local
  workspace version is now **2.10.1**.
- Stable upstream `oneclient-2.10.1` resolves to
  `ec784c912792f619dd88c788f97d7477be361bd2`. Its ten commits since 2.10.0 are
  integrated with upstream ancestry retained. Commit the upstream merge separately
  from the offline head-preview fix and fork documentation so each is reviewable.
- The starting fork commit is `3544820f`. Its published 2.10.0 application release
  was built from `2b5b364aec810e39069085c31160492f5c021a31`.
- The earlier history replacement is complete. Future approved updates should
  retain upstream ancestry and use a normal push; do not repeat the old history
  rewrite or reuse its obsolete expected remote SHA.
- README downloads continue to reference the published fork 2.10.0 assets. Update
  that table against actual fork 2.10.1 release assets when they exist.

## Changes in this update

Read [the upstream changelog](../CHANGELOG.md) and
[the fork notes](../FORK_CHANGELOG.md). Upstream adds folder resource-pack
handling, broader instance-name punctuation, UI contrast fixes, crash-dialog
log-upload controls, settings ordering and the social-window header changes.
It also fixes the Windows dedicated-GPU default and read-only dedicated-folder
sync. The README conflict was resolved by preserving this fork's documentation;
upstream's COPR instructions do not apply to this fork.

The account implementation, private configuration/data-path rules, package and
OS identities and updater public key remain unchanged. The follow-up head-preview
fix below extends the local-skin hook without changing its account-kind gate.
Cargo.lock changes only workspace version numbers, with no dependency upgrades.
Full source diffs and the starting tip were saved outside the repository under
`/home/user/.codex/backups/OneForAllLauncher/20261112-2.10.1/`.

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
been performed. Publication is authorized through the automatic release workflow.

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

Current 2.10.1 automated results and manual checks are recorded in
[validation.md](validation.md). Earlier 2.10.0 results are historical evidence,
not test results for this update. The owner manually tests UI/game behavior;
no screenshots or automated visual comparisons are authorized.

For the previous release, the owner confirmed shared OneClient libraries,
offline launching, local skins on Ornithe 1.8.9 and modern versions, and opening
OneConfig HUD with Ely.by. The HUD test's compatibility-agent setting was not
specified, so keep the optional preview workaround. Authlib-injector remains
separate and necessary for Ely.by/custom authentication.

Recheck the new folder resource-pack controls, shared/dedicated-folder syncing,
instance names with punctuation, contrast/onboarding, crash-upload confirmation
and the retained offline-launch/local-skin flows. Windows GPU behavior and
Windows/macOS packaging require platform-specific validation.

## Authorized publication and follow-up

A push to `main` automatically starts the release workflow for an unpublished
workspace version. It signs updater packages with the existing repository
secrets and publishes after all platform builds succeed. It also triggers managed
CodeQL and the stable-upstream sync badge. The owner has authorized this release.
After publication, refresh the README download table against the actual 2.10.1
assets, including every package, architecture, checksum link and updater marker.
Do not claim successful packaging or publication until the workflow confirms it.

CodeQL completed for the 2.10.0 release source but reported untriaged findings;
analysis success is not security clearance. A future published 2.10.1 commit
needs its own analysis. See [security verification](security-verification.md).

[Return to the README](../README.md#maintenance).

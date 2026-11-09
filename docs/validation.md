# Rewrite validation and release handoff

This checkout targets stable upstream `oneclient-2.10.0`, commit
`3968fb95f98569de41ec341ef0da2ed63802cfc4` (released 2026-11-08).
The implementation is on the local `rewrite/2.10.0` branch. Remote `main` remains
`71699a140a820538335f67924961973c501eec41`; no tags or release assets were removed.

## Automated checks (2026-11-09)

Run with Rust 1.98.0, Java 25 and Python 3 on Linux. Build/test debug information
and incremental compilation were disabled to keep resource use bounded.

- The final Linux debug build passed: `target/debug/oneforall_app` (Rust 1.98.0).
  The final application check passed as well. The binary was built without
  opening the launcher or modifying live launcher data.
- All **867 Rust library tests** passed across app, auth, common, core, database,
  Minecraft metadata, networking and PolyPlus. Rendering/screenshot tests in
  `components::markdown::image::tests` and `components::lobby::panel::tests` were
  explicitly excluded. No screenshots or automated visual comparisons were made.
- All **16 offline-account integration tests** passed separately. The app/auth
  library tests passed again after the final form cleanup (313 + 26 tests).
- Focused coverage includes legacy account deserialization/default selection,
  provider identity collisions without changing game UUIDs, preview-setting
  defaults/round trips, per-instance offline preference defaults/persistence,
  offline/online launch policy, task-local request isolation and a counted local
  server proving cache-only requests never connect.
- Source-built offline skin agent tests passed with JVM bytecode verification:
  local skin/slim model, unchanged remote players, opt-in gating and missing-file
  fallback. PNG validation and account-kind/version/loader gates have Rust tests.
- Source-built preview agent tests passed against the SHA-1-verified distributed
  VanillaHUD `3.5.10+1.8.9` JAR. Tests cover named and missing profiles, cache,
  mismatched profile responses, isolated loaders, scoped transformation and
  suppression of HTTP lookups in offline mode.
- The distributed VanillaHUD method was also tested headlessly without agents
  during planning: null profiles/names and lookup exceptions returned named
  fallbacks; an already named profile retained its name. This does not test the
  entire in-game renderer.
- Signed updater integration accepted the public signed fixture and rejected
  a modified package. Distribution release-note/updater-manifest tests passed.
- Clippy completed with 11 existing upstream diagnostics: bisect helpers, the
  PolyPlus token-cache type, notification variant names, browser arguments/sorts
  and chat test initializers. No diagnostics remain in the added account forms,
  authentication, offline launch or local skin code.
- Workflow YAML, translation-key coverage, SVG XML/currentColor, formatting of
  the new Rust modules and diff whitespace checks passed. Storage/binary/bundle
  identifiers and the updater public key match the previous fork; application
  artwork matches upstream exactly. `README_LOCAL.md` is absent.

The replacement series contains seven focused commits: installation/updates,
retained account support, the preview toggle, provider identities, offline launch,
local wardrobe and maintenance documentation. The assembled tree was checked
against the archived implementation snapshot before the validation report was
finalized. Backups are local and excluded from the replacement history.

## Follow-up: broader skins and installation guidance

- Removed the Minecraft/loader gate while retaining the offline-account-only
  gate. The owner confirms the original 1.8.9/Ornithe renderer works in game.
- Added shared authlib APIs and legacy skin-URL handling. Headless tests cover
  actual authlib generations 1.5, 2, 3, 4, 5, 6, 7, 9 and 10, with classic/slim,
  opt-in gating, other-player isolation and isolated loaders. A separately source-built bootstrap
  bridge also covers loaders without access to the system agent classpath. Authlib PNGs
  are served only over loopback; the hook performs no external skin requests.
- Tested the pinned 18-version matrix from beta 1.7.3 through 26.3 on Java 25;
  older subsets also passed Java 8, mid-range subsets Java 17 and current
  1.21.x subsets Java 21. Original beta, 1.6.4, 1.7.2 and 1.7.5 client URL call sites were audited
  headlessly. This does not replace owner testing of actual game rendering.
- Core/app regression tests passed: 357 + 313, with the same 14 rendering tests
  excluded. The Linux debug build passed; Clippy completed with existing upstream
  diagnostics. The new Rust modules were formatted; the lockfile is unchanged.
- Fixed the skipped update-consent step: Preferences now leads to Updates, and
  Summary's Back button returns to Updates. Existing installs retain their saved
  update choice; the owner should verify a fresh onboarding run.
- Added [installation troubleshooting](installation-troubleshooting.md) with
  official Microsoft/Apple per-app instructions, CPU/OS compatibility guidance
  and four clearly marked owner screenshot placeholders. No screenshots were
  captured by automation.

## Manual acceptance before release

1. Check the account screens, onboarding, approved icons and local wardrobe in
   light/dark and Minecraft themes. The README gallery deliberately uses the
   owner's existing 2.9 screenshots pending replacements.
2. Disconnect before startup and launch a fully installed game. Check the
   eight-second shared probe, all launch labels, right-click Launch Offline,
   the per-instance toggle (default off), busy-state guards and recovery online.
   Explicit offline mode should launch without a connectivity wait. Missing
   metadata, files or Java must produce an installation error, not downloads.
3. Check expired Microsoft/Ely.by/custom accounts in offline mode: preserve saved
   credentials, selected account, player name and game UUID. Offline mode does
   not enable authenticated multiplayer. Test equal names on different providers
   and reject a second equal name on the same provider.
4. Minecraft 1.8.9/Ornithe local skins have now been confirmed in game by the
   owner. Manually test Vanilla/Fabric/Forge/NeoForge/Quilt and representative old
   and new versions as available: choose a PNG, switch classic/slim, restart and
   restore the default. Verify only the local player's skin changes. Other
   account kinds must receive no hook, including when launched offline. Versions
   before 1.8 use the classic model. Check installed skin mods separately.

5. Test current VanillaHUD 3.5.10 with Ely.by and custom accounts, with the preview
   workaround disabled and enabled. Exercise the OneConfig tab-list/head editor
   and unavailable profile services. Authlib-injector must remain enabled. The
   upstream null-name fix exists; file an issue/PR only if a failure is reproduced.
6. Build all release platforms through CI and test signed upgrades from the old
   fork, custom data paths, preserved accounts/settings/instances, OS shortcuts,
   URL handlers, tray launch and simultaneous upstream/fork operation. Check
   Windows installation and both macOS architectures; these cannot be verified
   by this Linux build alone.

## Recoverable history replacement

The original fork and reviewed planning work are retained locally in
`archive/pre-rewrite-20261109`. The full pre-rewrite Git bundle is outside the
repository at:

`/home/user/.codex/backups/OneForAllLauncher/20261109T092735Z/before-rewrite.bundle`

Keep upstream history and old release tags. Review the new commit series and
`git diff oneclient-2.10.0...rewrite/2.10.0`, then record the new tip. A published
replacement must use the exact expected remote main SHA with `--force-with-lease`;
stop if another maintainer has moved it. Publish signed `oneforall-2.10.0` only
from the reviewed release commit after the owner's manual UI test result, as
required by [AGENTS.md](../AGENTS.md). Do not dispatch the release workflow from
a development snapshot.

The owner will write [offline-setup.md](offline-setup.md); it remains a clearly
marked placeholder rather than claiming unverified installation instructions.

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

The initial replacement series contains seven focused commits: installation/updates,
retained account support, the preview toggle, provider identities, offline launch,
local wardrobe and maintenance documentation. Follow-up commits cover onboarding,
shared libraries, skin-loader compatibility and user documentation. The assembled tree was checked
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

## Follow-up: shared libraries and onboarding defaults

- Added an explicit OneClient default folder choice using upstream's release path
  and legacy-location detection. Existing custom libraries are recognized through
  a read-only catalog check. Fork accounts/configuration remain independent.
- Verified upstream → fork → upstream migration/reopen behavior with temporary
  data, including instance preservation and the additive offline preference.
  No live upstream or fork library was used for these tests.
- Fresh setup preselects automatic updates and focuses Next. Next confirms the
  choice; existing installs keep their saved preference. A headless keyboard-event
  test confirmed Enter activates Next without a click, rendering or pixel capture.
- Passed 736 common/database/core/app tests (31 + 30 + 359 + 316), including the
  added keyboard-event test in the final app run. The same 14 rendering tests were
  excluded. The Linux debug build passed. Clippy completed with the same 11
  upstream diagnostics; Cargo.lock is unchanged.

## Follow-up: active upstream folder and library settings

- Corrected the selector's distinction between upstream's configuration root and
  its actual game library. Read only upstream's saved `data_dir`, including custom
  drives; unavailable drives and malformed locations produce errors rather than
  silently creating another library. No live launcher settings were changed.
- Fresh setup defaults to the OneClient library. Existing fork settings with
  `data_dir: null` or a custom path retain those locations. Accounts, launcher
  settings, themes and logs remain in the existing private fork directory.
- Added OneClient, OneForAllLauncher and custom library choices under Launcher →
  Folders and files. Selecting a library saves the path and quits; reopening binds
  paths and the catalog together. Explicit file moves remain separate, exclude
  private configuration and preserve reused sources without cleanup offers.
- Passed 363 core and 316 app library tests, with the same 14 rendering tests
  excluded. A separate temporary-library integration test passed: a ready stock
  instance and its registered Java runtime were reused, every game/cache/managed
  Java path used the library, private paths stayed separate, and selecting another
  library preserved preferences and original files until restart. These checks
  do not replace an actual game launch or owner UI verification.
- The refreshed Linux debug build passed. Clippy completed with the same 11
  upstream diagnostics and none in the new library-selection code. New/changed
  Rust sections, translation-key coverage, Markdown links and diff whitespace
  checks passed. Cargo.lock is unchanged. No screenshots or visual comparisons
  were performed.

## Owner checks and 26.3 log review

The owner confirms shared OneClient data/instances and Launch Offline work. The
game library has been moved to the stock default; the test launcher's saved path
already points there. No live settings were changed during this investigation.
Ornithe 1.8.9 skins remain confirmed, but the owner reports other versions,
including 26.3/Fabric, still render without the selected local skin.

Reviewed [the supplied 26.3 log](https://mclo.gs/K7KDjNY) before changing the hook:

- The agent and offline-account/PNG properties are present. The skin downloader
  later reports an HTTPS handshake failure for the selected profile, rather than
  a successful private HTTP skin request. The hostname is not shown in that stack.
- Mojang user-properties/Realms and PolyPlus cosmetics, WebSocket and EOS/P2P
  authorization fail with invalid sessions/401 responses. These are authenticated
  game/mod features that an offline identity does not provide; the launcher must
  not manufacture successful authentication or suppress those service checks.
- OpenGL backend creation fails, then Vulkan initializes successfully on the
  recorded AMD/Mesa setup. This is a separate graphics/native-path concern, not
  evidence of a local-skin failure or a fatal startup crash in this log.
- Missing optional integration classes, refmaps and mod version-format warnings
  originate in the modpack. The game proceeds through resource loading.

The old authlib and basic Knot tests passed before the correction. A new loader
that explicitly refuses the bridge package reproduced silent hook skipping on
real authlib 10.0.77. Calls now resolve the bootstrap bridge directly through JDK
reflection without consulting the game loader during transformation. That
regression passes for 1.8.9, 1.21.11 and 26.3; real Fabric 0.20.1 Knot and
compatibility loaders also pass with isolated classpaths. Version 3 filenames
separate the new helpers, and credential-free activation/use messages aid the
next owner test. This fixes a reproduced loader defect; it does **not** establish
that every cause of the supplied modpack's skin failure is resolved. Recheck
26.3 in game before describing it as verified or publishing a release.

The corrected 18-version matrix passed on Java 25, with the supported older cases
also passing Java 8. Ornithe/gating/missing-file and missing-bridge checks passed.
The targeted real Fabric runs passed for 1.8.9, 1.21.11 and 26.3, covering slim,
classic and disabled modes under normal and compatibility Knot loaders. The
679 core/app library tests passed (363 + 316), excluding the same 14 rendering
tests. The refreshed Linux debug build passed. Clippy completed with the same
11 upstream diagnostics and none in the changed skin-hook integration. Python
syntax, changed Markdown links, Rust formatting for the changed module and diff
whitespace checks passed. No UI screenshots or visual comparisons were performed.

## Follow-up: Secure Skins transport compatibility

The [next owner log](https://mclo.gs/TGH7I2i) confirms the version 3 hook activates
and supplies local textures, followed by the same skin-download TLS failure.
The installed pack includes Secure Skins 1.0.1. Its actual `useHttpsUrl` method
unconditionally replaces HTTP with HTTPS at the head of `downloadAndRegisterSkin`.
This changes the private HTTP skin endpoint into an HTTPS endpoint the loopback
server does not implement. The previous classloader correction was insufficient
for this pack; the new log establishes a distinct transport conflict.

Reviewed authlib-injector's scoped URL-routing design. The offline hook keeps its
independent source implementation and now normalizes the exact private capability
at URL/URI creation, after the mod's rewrite. Other HTTPS URLs, certificate checks
and unrelated local endpoints are preserved. Version 4 helper files distinguish
this correction from the version 3 test build.

A regression downloads the checksum-pinned official Secure Skins artifact and
executes its unchanged URL-rewrite method. It failed before the correction and
passes afterward, including retrieving the selected PNG and retaining HTTPS for
other endpoints. It also checks `URL(String)`, `URI(String)` and `URI.create` paths.
This is a concrete transport regression; the owner still verifies actual rendering.

The corrected 18-version matrix passed on Java 25, with nine older version cases
also passing Java 8. Real Fabric normal/compatibility loader runs passed for
1.8.9, 1.21.11 and 26.3, including the Secure Skins transport regression on 26.3.
Ornithe/fallback smoke checks and all 679 core/app tests passed; the same 14
rendering tests were excluded. The Linux debug build passed, Clippy completed
with the same 11 upstream diagnostics, and formatting/link/diff checks passed.
No screenshots or automated visual comparisons were performed.

The maintainer-provided offline-launch image is included in the README gallery.
[Security-verification notes](security-verification.md) document researched free
CodeQL, OpenSSF Scorecard, review and build-provenance options and their limits.
GitHub reported CodeQL default setup as not configured during the initial
research. It was subsequently enabled at the owner's request; see below. No
build attestation has been added.

## Follow-up: managed CodeQL setup

Enabled GitHub's managed default setup with the Extended query suite, standard
hosted runners and remote/local sources where supported. The
[initial setup run](https://github.com/liwwyy/OneForAllLauncher/actions/runs/42659907902)
includes Rust, Java/Kotlin, Python and GitHub Actions. It analyzes the published
`main` commit `71699a140a820538335f67924961973c501eec41`, not this unpublished
rewrite. No source commit, tag or release was pushed. The rewrite needs its own
scan after an authorized push. See [configuration and limits](security-verification.md#codeql-configuration).

All four analysis jobs completed successfully. GitHub confirms all four languages,
the Extended suite and a configured weekly schedule. The baseline contains
**260 open alerts**: 245 Rust, 3 Java, 11 Python and 1 Actions. GitHub's query
severity labels are 8 critical, 250 high and 2 medium; 237 findings use Rust's
path-injection rule. These alerts remain untriaged and none were dismissed.
Review the new rewrite scan before release rather than treating job success as
security clearance. This documentation-only follow-up passed new local Markdown-link
and diff whitespace checks; no application rebuild was needed.

## Manual acceptance before release

Owner update (2026-11-09): opening the OneConfig HUD with an Ely.by account
works in the owner's test. The compatibility-agent toggle state was not specified,
so this does not establish that the workaround can be removed.

Owner update (2026-11-09): offline skins now work on modern versions after the
version 4 correction, in addition to the previously confirmed Ornithe 1.8.9 setup.
Shared library operation and offline launch were already confirmed. This records
the owner's report, not automated rendering verification or full platform coverage.

1. Check the account screens, onboarding, approved icons and local wardrobe in
   light/dark and Minecraft themes. The README gallery deliberately uses the
   owner's existing 2.9 screenshots pending replacements. Check all three data
   location choices, including a legacy/custom upstream library, then restart to
   verify persistence. Verify fresh setup's OneClient default and an upgraded
   fork's unchanged old location. Switch back to the OneForAllLauncher library in
   Folders and files, then select OneClient again and verify its custom-path
   instances and installed Java are reused. Use one launcher at a time when
   sharing; verify accounts and launcher settings remain separate. On fresh
   setup, press Enter on Updates without clicking a choice; also check manual
   selection and Back/Next behavior.
2. Disconnect before startup and launch a fully installed game. Check the
   eight-second shared probe, all launch labels, right-click Launch Offline,
   the per-instance toggle (default off), busy-state guards and recovery online.
   Explicit offline mode should launch without a connectivity wait. Missing
   metadata, files or Java must produce an installation error, not downloads.
3. Check expired Microsoft/Ely.by/custom accounts in offline mode: preserve saved
   credentials, selected account, player name and game UUID. Offline mode does
   not enable authenticated multiplayer. Test equal names on different providers
   and reject a second equal name on the same provider.
4. Minecraft 1.8.9/Ornithe and modern-version local skins have been confirmed in
   game by the owner. Continue checking untested Vanilla/Fabric/Forge/NeoForge/Quilt
   setups and representative old and new versions as available: choose a PNG,
   switch classic/slim, restart and restore the default. Verify only the local
   player's skin changes. Other account kinds must receive no hook, including
   when launched offline. Versions before 1.8 use the classic model. Check
   installed skin mods separately.

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

The maintainer-approved [offline setup guide](oneclient-modpack-alternative-guide.md) now walks through
Freesm offline-account creation, installing the official OneClient Modrinth pack,
the `.mrpack` import alternative, initial downloads and authenticated-service
limitations. The README links to it. Interface labels were checked against
Freesm 2.3.2's [account controls](https://github.com/FreesmTeam/FreesmLauncher/blob/2.3.2/launcher/ui/pages/global/AccountListPage.ui)
and [import page](https://github.com/FreesmTeam/FreesmLauncher/blob/2.3.2/launcher/ui/pages/modplatform/ImportPage.h),
and the pack link against [Polyfrost's Modrinth listing](https://modrinth.com/modpack/oneclient-modpack).
These are source/documentation checks, not a claimed manual Freesm installation.

# Working on OneForAllLauncher

## Fork scope

Maintain a small, reviewable set of changes over Polyfrost OneLauncher/OneClient.
Only add or modify features directly associated with standalone offline accounts,
Ely.by and custom authentication server support. Keep modifications to OneLauncher
minimal so the fork stays easy to maintain and update. Necessary maintenance must
support this scope without introducing additional features. Direct unrelated
features and general launcher improvements upstream.
Prefer upstream implementations, UI components, artwork and icons. Keep the
existing `oneclient_*` crate names and workspace structure. Avoid broad refactors,
mechanical renames, formatting unrelated files and new dependencies without a
concrete need. Preserve LICENSE, ATTRIBUTION.md and original copyright notices.

Retain these account features:

- Standalone offline accounts, live username validation, the invalid-name
  override, both random-name generators and optional online-name lookup.
- Microsoft, Ely.by and custom Yggdrasil authentication, refresh and profile skins.
- Authlib-injector support for Ely.by/custom game launches.
- The scoped Minecraft 1.8.9 VanillaHUD preview compatibility agent until its
  replacement is verified against the distributed mod and tested in game.

Use upstream instance import/export and sharing. Do not restore the removed
fork-specific Prism/ZIP exporters, templates, presets or file-selection UI.

## Upgrade compatibility and side-by-side installs

Visible branding and operating-system identity serve different purposes. Restore
upstream presentation where agreed, while retaining the identities needed to
upgrade existing fork installs and coexist with upstream:

- Preserve `com` / `OneForAll` / `OneForAllLauncher` in
  `oneclient_common::paths`, including legacy-directory detection, custom data
  locations and the separate `OneForAllLauncher-dev` debug directory.
- Preserve settings, `auth.json`, account kinds, offline UUID generation,
  database migrations, instance metadata and stored account-provider fields.
  Test loading data from older releases when changing these formats.
- Keep the fork's executable (`oneforall_app`), package/bundle identity
  (`com.oneforall.OneForAllLauncher`), install paths, desktop entries, IPC
  endpoints and `oneforall://` / `oneforall-dev://` handlers distinct from
  upstream. Check platform-specific packaging before changing any identifier.
- Preserve the fork-owned updater endpoint, public verification key and release
  tag scheme. Existing installed versions must trust the next release. Do not
  replace the signing identity or direct fork users to upstream update packages.
- In fresh onboarding, preselect automatic updates and focus Next for Enter.
  Confirm/save that choice on Next, preserve a manual selection when revisiting
  the step, and retain existing installs' saved update preferences. Merely opening
  the page must not start updates.
- Preserve automatic-update consent and manual installation confirmation. Keep
  fork crash reporting unconfigured unless explicitly enabled for the build.
- Preserve existing installs' fork/custom game locations, including saved
  `data_dir: null`; changing fresh-setup defaults must never redirect upgrades.
  Fresh onboarding defaults to **OneClient folder**, following upstream's saved
  custom `data_dir` as well as release/legacy layouts. An unavailable configured
  drive must produce an error, not a fallback empty library or new downloads.
- Keep fork configuration, accounts, themes, logs and IPC in the existing private
  fork folder. Only game data/database, instances, caches and managed Java share
  the selected library. Offer OneClient, OneForAllLauncher and custom libraries in
  Settings → Launcher → Folders and files, alongside the private folder's opener.
  Switching libraries saves the exact path and requires reopening; do not copy,
  merge, delete or offer the previous shared library as removable leftovers.
  Keep explicit file moves separate from library selection, and exclude both
  launchers' configuration from copies. Recognize compatible existing libraries
  without nesting a new OneClient directory. Shared libraries are intended for
  matching stable releases; use one launcher at a time.
- Do not automatically import or modify upstream OneClient installations. Test
  sharing and database round trips only in temporary directories, never a user's
  live accounts or game library.

## Code and UI conventions

Follow nearby Rust 2024 code and the existing Freya component/builder patterns.
Keep UI changes consistent with OneLauncher's existing Freya components,
styling, layout and interaction patterns. Reuse its settings rows, context menus,
buttons, dialogs and wardrobe previews instead of building parallel UI systems.
Keep authentication logic in `oneclient_auth`, game-launch integration in
`oneclient_core` and UI state/presentation in `oneclient_app`. Reuse existing
settings, error, event, cancellation and network abstractions where appropriate.
Keep blocking work off the UI thread and avoid holding shared locks across
network requests. Never log passwords, access/refresh tokens or device codes.

Use the existing `Icon` / `IconType` SVG pipeline and theme colors. Account
buttons should reuse upstream icons rather than introduce unrelated icon styles
or hardcoded white tints. When integrating upstream 2.10.0, retain its pixel-icon,
theme, text-sizing and translation support in fork-modified views.

The compatibility-agent toggle must control only the VanillaHUD preview
workaround. Turning it off must omit both the preview `-javaagent` argument and
its added classpath entry, while retaining authlib-injector for authentication.
Use the existing persisted settings pattern and explicitly review the default
for old settings files. Verify the exact distributed VanillaHUD version before
claiming the workaround is redundant; a source fix alone is insufficient.

The local skin client hook must run only for `AccountKind::Offline`,
with a selected local skin. Use loader-independent authlib and legacy skin paths;
retain verified direct hooks only where necessary. Test against real authlib API
generations and keep in-game validation separate from headless tests. Gate hook/classpath/agent
injection in the launcher before starting Minecraft. Never load this hook for
Microsoft, Ely.by or custom accounts, even if internet is unavailable or offline
launch mode is selected. This restriction is separate from authlib-injector and
the VanillaHUD preview workaround, which retain their own provider-specific scope.

## Verification

Read Cargo.toml and README.md for current toolchain and build requirements. The
current workspace requires Rust 1.97+, Python 3 and JDK 17+ with `javac`.

Run checks appropriate to the change, keeping Cargo.lock fixed unless dependency
or version changes require updating it. Useful commands include:

```sh
cargo +stable build --locked -p oneclient_app --bin oneforall_app
cargo +stable test --locked -p oneclient_auth
cargo +stable test --locked -p oneclient_common -p oneclient_db
cargo +stable test --locked -p oneclient_core -p oneclient_events -p oneclient_app --lib -- --skip components::markdown::image::tests --skip components::lobby::panel::tests
cargo +stable fmt --all -- --check
cargo +stable clippy --locked -p oneclient_auth -p oneclient_core -p oneclient_app --all-targets
python3 -B -m unittest discover -s distribution -p 'test_*.py'
```

Distinguish pre-existing failures from regressions. Add focused tests for changed
behavior and upgrade compatibility; documentation-only changes need link and
diff checks rather than application builds. Agent tests and prerequisites are
documented in `distribution/elyby-preview-compat/README.md`. They do not replace
testing the actual game/mod combination with the workaround enabled and disabled.

**Do not take screenshots or perform automated visual comparisons unless the
owner explicitly requests them. The owner manually tests UI changes.** Report
which checks ran and which behaviors still need manual verification. Do not claim
historical entries in VALIDATION.md as results from the current checkout.

## Commits, upstream updates and releases

Keep upstream history intact. Build any agreed replacement fork history from a
pinned upstream release with a small series of focused commits. Separate storage
and OS identity, account features, compatibility changes, updates/distribution
and documentation where each can be reviewed meaningfully. Include relevant
tests with the behavior they verify; avoid commits that only record test counts
or publication status. Use clear imperative subjects and explain non-obvious
tradeoffs in commit bodies. Respect the existing changeset workflow.

Before an authorized published-history rewrite, make a recoverable backup and
record the old and new tips. Review both the patch series and final diff against
upstream; use an explicit expected remote SHA with `--force-with-lease` for the
agreed branch. Preserve published release tags and assets unless their removal
is separately requested. Replacing a branch does not erase commits retained by
tags. Do not reset, force-push, delete refs or publish outside the authorized
scope.

Keep fork version numbers aligned with the upstream release being integrated.
Sync the fork to stable OneLauncher/OneClient releases, rather than following
individual upstream commits or development builds. The usual update target is
within a day of an upstream stable release; do not promise an unconditional
deadline or bypass validation to meet it.
Prepare signed update artifacts and notes for the exact release commit, verify
upgrade continuity and side-by-side installation, and obtain the owner's manual
UI test result before publishing. Do not describe planned features as released.

## Documentation

README.md is the user entry point and source-build guide. Keep user-visible
instructions accurate and link to focused guides instead of duplicating them.
The proposed application changes and their acceptance checks are recorded in
`docs/implementation-plan.md`; keep planned behavior distinct from shipped
features and respect the owner's authorized implementation scope.
The maintainer writes the offline OneClient setup guide at
`docs/offline-setup.md`; keep its README link current. Do not invent installation
steps or claim a method is community-trusted without evidence. Keep upstream and
fork attribution clear. Do not recreate README_LOCAL.md.

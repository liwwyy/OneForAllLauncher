# Local validation — 2026-10-06

OneForAllLauncher, a fork of OneClient by Polyfrost.
Upstream base: `7683651ad4d5a74682c23c624341f278bb047bc8`.
Local branch: `oneforall`. Origin: https://github.com/liwwyy/OneForAllLauncher.
Initial local validation preceded publication. GitHub publication results are recorded below.

## Completed checks

- `cargo build -j 2`: passed, without warnings on the final build.
- `cargo test -p oneclient_auth -j 2`: 13 passed, including 9 offline integration tests.
- `cargo test -p oneclient_common paths:: -j 2`: 3 passed.
- `cargo test -p oneclient_core migration:: -j 2`: 16 passed.
- `cargo test -p oneclient_db -p oneclient_app --lib -j 2`: 28 database and 164 app tests passed.
- `git diff --check`: passed.
- LICENSE and ATTRIBUTION.md remain byte-for-byte identical to upstream.

The database regression test holds a writer lock for seven seconds and verifies
that a concurrent write succeeds afterward. The old five-second timeout caused
two bundled mod installs to fail during the first loaded smoke session. The
new timeout is 30 seconds, with a 60-second pool acquisition limit.

## Packages and runtime

Development builds were packaged using cargo-packager 0.11.8; these are not
optimized release installers. Symbols were stripped for the final local bundles.
The package contains the latest build, replacement icons including 512px,
LICENSE, ATTRIBUTION.md and FORK_CHANGELOG.md. Its package name is
`one-for-all-launcher`; desktop/executable names are `oneforall_app`.

Artifacts (ignored by Git):

- `target/packages/oneforall_app_2.6.1_amd64.deb`
- `target/packages/oneforall_app_2.6.1_x86_64.tar.gz`
- `target/packages/PKGBUILD` (local Arch packaging recipe)

The stripped packaged executable was compared against the latest built executable
and matched. The .deb was extracted and launched directly, without installing it
system-wide. Vulkan initialized on the AMD Radeon RX 550; no launcher startup
crash occurred. Single-instance forwarding returned exit code 0. The final
packaged build was also run with a 15-second timeout in a separate test namespace.

The interactive smoke session completed onboarding with an offline account,
persisted its default account and verified its UUID against Minecraft's algorithm.
Minecraft 1.8.9 was downloaded and a Java game process started without a Microsoft
account. Its game log reached graphics/resource/sound initialization. Bundled
online cosmetics and P2P services reported authentication failures for the offline
account; those services require an authenticated Minecraft session. An upstream
HUD mod also reported a configuration error. Those mod services were not changed.

The original interactive smoke session uses:
`target/smoke/data/OneForAll/OneForAllLauncher-dev`.
Normal launches use the platform's regular `OneForAll/OneForAllLauncher-dev`
application data folder instead. Test screenshots and extracted packages are
under `target/smoke`. The first interactive session used the earlier build; restarting with the final
executable uses the database timeout fix.

Windows/macOS installers, optimized release builds and simultaneous execution of
an upstream binary were not run in this Linux session. Namespace separation is
covered by the path/protocol tests and installer/IPC metadata review. macOS uses
the replacement ICNS; the upstream liquid-glass .icon/Assets.car pipeline was
removed because it is not needed for portable icon packaging.

## Reproduce the final local package

The generated `target/package-config.json` contains absolute resource paths and
points at the stripped executable under `target/package-bin`. It is a local
smoke-test configuration, not a tracked release configuration:

```sh
~/.cargo/bin/cargo-packager --config "$PWD/target/package-config.json" --formats deb,pacman
```

See README.md for standard release packaging, pushing the fork, and future upstream merges.

## GitHub publication and additional packaging checks

The source was pushed to `liwwyy/OneForAllLauncher` on `main` and `oneforall`.
The fork release workflow now builds unsigned Windows NSIS installers, Linux
DEB/RPM/AppImage packages, and macOS DMG/app archives for Intel and Apple Silicon.
It publishes only after all expected assets are present, including checksums.
Upstream signing credentials and updater signatures are not required.

Locally, the RPM configuration produced a package with the separate
`oneforall-launcher` identity and the original license and attribution files.
AppImage packaging succeeded with `NO_STRIP=1` (the bundled linuxdeploy strip
tool cannot read this host's RELR libraries). The AppImage opened the
OneForAllLauncher window and initialized Vulkan with only the known nonfatal
libayatana deprecation warning. AppImageLauncher desktop integration was bypassed
for the smoke test using `APPIMAGELAUNCHER_DISABLE=1`; test XDG directories were
under `target/smoke-appimage`. These local artifacts still use the development
binary; optimized release binaries are built separately by GitHub Actions.

The first optimized prerelease is published at
https://github.com/liwwyy/OneForAllLauncher/releases/tag/oneforall-2.6.1
from source commit `77f3e305efe350e0107e5408a2ddee0943de52d9`. All jobs in
https://github.com/liwwyy/OneForAllLauncher/actions/runs/37376223222
succeeded, including Windows installer layout and runtime-DLL checks. The release
contains eight packages plus SHA256SUMS.txt and GitHub's corresponding source
archives. Windows/macOS fork builds are unsigned; macOS is not notarized.

The published Linux AppImage was downloaded, checked against its published
SHA-256 checksum, and launched using isolated directories under
`target/smoke-release`. It opened a window with app ID `oneforall_app`, title
`OneForAllLauncher`, and Vulkan graphics initialized. The 25-second smoke test
ended with its expected timeout, without a startup crash. This host logged
nonfatal XKB Compose keysym compatibility warnings from the bundled older
xkbcommon library, a desktop settings portal theme-query timeout, and a notice
that disabled Sentry reporting did not start. These messages did not prevent
startup. Windows and macOS interactive GUI launches remain untested locally.


## 2.6.2 exports and transparent artwork — 2026-10-06

`cargo build -j2` passed for the entire workspace. The supplied transparent PNG
was converted into every shipped PNG, ICO and ICNS launcher icon and embedded
logo SVG. All nine PNG variants have alpha values spanning 0–255. The original
PNG, export SVG and example Prism ZIP are ignored; their production assets are tracked.
LICENSE and ATTRIBUTION.md remain unchanged.

The opt-in `export_sample` integration test opened the supplied upstream
OneClient SQLite database strictly read-only and exported its `1.8.9 OC` instance.
The resulting mods.zip contains 71 mods (241.1 MiB). Its Prism ZIP contains
256 files (265.9 MiB), including 71 mods and 169 config/OneConfig files, plus
Minecraft options, portable memory/window preferences and the supplied Ornithe
patches. Both archives passed Python zipfile CRC validation.

Prism Launcher 11.0.3 imported the ZIP into an isolated directory under
`target/prism-export-check`. All 254 payload files matched the ZIP byte-for-byte;
Prism rewrote instance.cfg and mmc-pack.json. The imported metadata retains
Minecraft 1.8.9 and Fabric loader 0.19.5. No Minecraft launch was attempted from
Prism. Other supported loader metadata is covered by unit tests.

A local development-binary AppImage was built with cargo-packager and launched
successfully using isolated data directories. The launcher initialized OpenGL
and its database without a startup crash. The local package is
`target/packages-2.6.2/oneforall_app_2.6.2_x86_64.AppImage`.

Automated GUI interactions and screenshots stopped at the user's request.
Final save-dialog behavior and visual acceptance will be confirmed manually
by the user. The local AppImage uses a development binary; GitHub release
packages use optimized release binaries.

`cargo test -p oneclient_core -p oneclient_auth -j1` passed: 195 core unit tests,
13 authentication tests and 35 enabled core integration tests (243 total).
The supplied-data export test also passed separately. Tests requiring explicit
external services/data remain ignored. The first core run exposed an inherited
GPU test that probed the host's real render node; its unavailable-driver fixture
now has no render node. Production GPU behavior is unchanged. All nine export
regression tests passed, covering symlinks, disabled/cached mods, shared folders,
Prism metadata/settings, invalid paths and failed-save cleanup.

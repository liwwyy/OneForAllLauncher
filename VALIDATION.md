# Local validation — 2026-10-06

OneForAllLauncher, a fork of OneClient by Polyfrost.
Upstream base: `7683651ad4d5a74682c23c624341f278bb047bc8`.
Local branch: `oneforall`. Origin: https://github.com/liwwyy/OneForAllLauncher.
No commits or pushes were made by the implementation session.

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

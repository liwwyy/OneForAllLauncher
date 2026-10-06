# OneForAllLauncher

OneForAllLauncher, a fork of OneClient by Polyfrost.

Modified on **2026-10-06** to support standalone offline accounts, independent
branding and side-by-side installation with upstream. Original copyright and
third-party notices are preserved in [LICENSE](LICENSE) and [ATTRIBUTION.md](ATTRIBUTION.md).
This project remains **GPL-3.0-only**, without warranty. When distributing binaries,
make the corresponding source for that exact build available alongside them.

Offline accounts work without Microsoft sign-in. They require a valid Minecraft
username (3–16 ASCII letters, digits or underscores) and use Minecraft's offline
UUID algorithm. Game downloads still require connectivity on first installation;
offline accounts can join servers that accept offline players. Authenticated
servers require a Microsoft account with Minecraft access. Bundled online services
(such as cosmetics and friend/P2P hosting) may also require an authenticated account.

## Build and run

Use Rust 1.97 or later and the native dependencies required by Freya/Skia.

```sh
cargo build
cargo test -p oneclient_auth
cargo run -p oneclient_app --bin oneforall_app
```

Workspace crate names retain the `oneclient_` prefix to keep upstream merges
manageable. The executable is `oneforall_app`, the URL scheme is `oneforall://`
(`oneforall-dev://` in debug builds), and release data lives under
`OneForAll/OneForAllLauncher` in the platform's application data directory.
Debug builds use `OneForAllLauncher-dev`. Settings, accounts, game installations,
and single-instance IPC are independent of upstream. OneClient v1 imports are disabled.

Automatic updates are disabled. Configure a fork-owned endpoint and signing key
before enabling them. Fork release notes are bundled locally. Upstream AUR/COPR
publishing configurations and macOS liquid-glass asset catalogs are removed;
macOS packaging uses the supplied ICNS icon. Upstream service endpoints are retained
for game metadata and Microsoft login. Crash reporting requires an explicit
`ONEFORALL_SENTRY_DSN` at build time.

## Export instances

Open an instance's **Settings → Exporting**:

- **Export mods.zip** saves installed mods at the ZIP root, copying actual bytes
  from symlinked/cached mods and excluding disabled database entries.
- **Export prism instance** creates a ZIP to import with Prism Launcher's
  **Add Instance → Import**. Mods are under `.minecraft/mods`; Minecraft options,
  `config`, `defaultconfigs`, `oneconfig`, and `OneConfig` files are included.
  Memory and window preferences are included in `instance.cfg`.

Ornithe Gen2 Minecraft 1.8.9 exports preserve the supplied Prism template's
custom patches and icon. Other Ornithe versions can export mods.zip; the Prism
button explains the template's version limit. Standard Vanilla, Fabric, Quilt,
Forge and NeoForge instances use their actual Minecraft and loader versions.
A loader set to Latest is resolved before exporting Prism metadata.
Close games using the instance's shared or dedicated game directory before
exporting. Choose a destination outside the game/mods folders. Exporting does not
change the instance or include launcher accounts, databases, caches, logs or worlds.
Prism may need to download Minecraft and its loader when the archive is imported.

## Package

```sh
cargo install cargo-packager --locked
cargo build --release -p oneclient_app --bin oneforall_app
cargo packager --release --packages oneclient_app --formats deb
```

For AppImage packaging on modern Linux distributions, set `NO_STRIP=1` so
linuxdeploy does not run its older strip tool on system libraries. Cargo already
strips the release executable.

Packaging metadata lives in `packages/oneclient_app/Cargo.toml`.
The original `icon.jpg`, `icon-transparent.png`, `export.svg`, and example Prism
ZIP are ignored; converted icons, embedded SVGs, and bundled Prism template files
are tracked. No generated Apple `Assets.car` is needed or shipped.

## GitHub releases

The repository is https://github.com/liwwyy/OneForAllLauncher. `origin` points to
this fork; `upstream` points to Polyfrost. Fork development lives on `main`.

In GitHub Actions, run **OneForAllLauncher Release Build** on `main`, or use:

```sh
gh workflow run oneclient_release.yml --ref main -f prerelease=true
```

The workflow builds optimized binaries and publishes a release only when all
four platform builds succeed:

- Windows x86_64: NSIS `.exe` installer with the Microsoft VC++ runtime bundled.
- Linux x86_64: `.deb`, `.rpm`, and `.AppImage`.
- macOS Apple Silicon and Intel: `.dmg` and `.app.tar.gz`.
- `SHA256SUMS.txt` and GitHub's source archives for the exact release tag.

Fork releases use tags like `oneforall-2.6.2`. Increase `workspace.package.version`
in `Cargo.toml`, update `Cargo.lock` (`cargo check`), and update `FORK_CHANGELOG.md`
before the next release. The workflow refuses to overwrite an existing release.
Use `prerelease=false` when a build has been validated for general release.

No private signing secrets are required. These builds are unsigned and macOS
builds are not notarized; Windows SmartScreen/macOS Gatekeeper may prompt or block
them. Automatic updates stay disabled. The inherited upstream signing and updater
publication steps are removed. Nightly builds are available by manual workflow
dispatch, so pushing source does not duplicate the four-platform release build.

## Keep up with upstream

Keep fork changes in commits separate from upstream, so future merges preserve
branding, isolated storage, offline account support and disabled upstream updates.
Update on a dedicated branch and test before merging:

```sh
git fetch upstream
git switch main
git switch -c sync-upstream-YYYY-MM-DD
git merge upstream/main
cargo build
cargo test -p oneclient_auth
# Also run GUI/packaging checks, then merge into your fork branch.
```

Check monthly or after an upstream security release. You can ask Codex to perform
an upstream sync, resolve conflicts, audit these fork requirements, and rerun
build, auth and packaged-launch checks. Review the resulting diff before publishing.

# Offline local skin hook

Cargo builds this agent from the tracked Java source. It reuses the preview
agent's SHA-256-pinned, relocated ASM dependency and includes both licenses in the
generated JAR. No precompiled hook is checked in. Building needs Python 3 and
JDK 17+ (`javac --release 8`); the game-side agent runs on Java 8 and newer.

## Scope and loading

The launcher injects the hook only for **`AccountKind::Offline` with a saved
64 × 64 PNG**, across Minecraft versions and loaders. Offline launch mode on
Microsoft, Ely.by or custom accounts never enables it. The launcher passes the
actual Minecraft UUID, name and classic/slim choice as launch-only properties;
no access token or other account credential is passed to the hook.

Local skins stay under `local-skins/<launcher-account-id>/` in the fork's
configuration directory. Changes apply on the next Minecraft launch. Only the
selected offline player's skin is supplied. Other players and server-visible
identity remain unchanged; no upload or registration is performed.

## Compatibility paths

- **Authlib skin APIs (Minecraft 1.7.6+):** hook the shared, unobfuscated session-service API,
  independently of Vanilla, Fabric, Forge, NeoForge, Quilt or Ornithe mappings.
  Handle legacy texture maps, `fetchProfile`, packed texture properties and
  modern texture records, including the newer `services` package. Match only
  the selected offline UUID. Local properties remain explicitly **unsigned**.
- **Legacy skin loading (through Minecraft 1.7.5):** rewrite `URL(String)` only when it addresses a known
  legacy Minecraft skin endpoint for the selected offline username. Other URLs
  and other players' skin names pass through. Both the old S3 MinecraftSkins
  endpoint and minecraft.net/skin paths are covered. Versions before 1.8 only
  render the classic model, regardless of the wardrobe's slim selection.
- **Minecraft 1.8.9/Ornithe:** retain the direct renderer hook already tested in
  game by the owner. It is enabled by a separate launch property only for that
  combination, so its version-specific mappings are never applied elsewhere.

Authlib clients download the PNG through a private HTTP endpoint bound solely
to **127.0.0.1 on an ephemeral port**, with an unpredictable path and a SHA-256
texture cache key. It serves only the selected PNG, permits GET/HEAD, rejects
other paths and uploads, and closes with the Minecraft process. It needs no
internet connection and makes no external skin-service requests. A small JDK-only
bridge is built into a separate JAR and loaded through the
agent manifest’s bootstrap path. Isolated mod loaders can resolve it without
access to the system classpath; ASM and the transformer stay outside that
bootstrap JAR. Both JARs are embedded and verified/written by the launcher.
The Ornithe direct renderer reads the PNG locally.

Authlib and legacy URL call sites resolve the bridge explicitly through the
bootstrap loader using JDK reflection. They do not ask the mod loader to load a
helper package during class transformation. This avoids a silent fallback when
loader isolation rejects that package and avoids reentrant game-class loading.
The launcher installs version 4 agent/bridge filenames so old and new running
JVMs cannot disagree about an updated helper. Brief status messages report
activation and use of local textures without exposing names, paths or credentials.

Secure Skins 1.0.1 changes all skin-download URLs from HTTP to HTTPS, including
the private loopback endpoint. The agent normalizes only that exact endpoint
back to its original local transport at `URL(String)`, `URI(String)` and
`URI.create(String)` call sites, after the mod's change. Other HTTPS URLs, other
loopback addresses/paths and certificate validation remain unchanged. A scoped
URL-routing approach was reviewed in
[authlib-injector](https://github.com/yushijinhun/authlib-injector); its authentication
and key-verification transformers are not needed for this standalone PNG hook.
The implementation here is independent; no injector source was copied.

The hook does not modify game/mod JARs on disk. Unsupported future API changes
fall back to normal skin loading. Mods that replace the native skin-loading or
rendering paths can override these integrations and need their own game testing.
“All versions” here describes the shared compatibility paths, not a claim that
every historical snapshot and third-party skin mod has been tested in game.

The retained 1.8.9 direct path uses verified Feather Gen 2 build 2 intermediary
names: player `C_23527099`, skin `m_68930603`, model `m_63406370`, local player
`C_57778754`, dynamic texture `C_66554568`, texture manager `C_82920792.m_87114534`,
and `Minecraft.m_20213497` / `m_65924382`.

## Headless verification

```sh
python3 -B distribution/offline-skin/test_agent.py
python3 -B distribution/offline-skin/test_matrix.py
python3 -B distribution/offline-skin/test_matrix.py --versions 1.8.9 1.21.11 26.3 --fabric
python3 -B distribution/offline-skin/test_matrix.py --versions 26.3 --secure-skins
```

The first suite tests the retained Ornithe renderer, opt-in gating and missing
PNG fallback. The matrix downloads checksum-pinned official Mojang version
metadata and authlib dependencies into `target/offline-skin-tests`. It exercises
**actual authlib methods**, rather than simulations of those APIs, with JVM
bytecode verification, local/remote UUID isolation, classic/slim models, the
private PNG endpoint, rejected paths/uploads, a game-library loader whose parent
excludes the system classpath, and a loader that explicitly rejects bridge classes.
The restricted-loader case failed before the bootstrap-dispatch correction.
`--fabric` adds SHA-256-pinned Fabric Loader 0.20.1 and Mixin test artifacts from
Fabric's Maven, and checks real Knot and compatibility loaders with isolated
classpaths. It does not start Minecraft, initialize game Mixins or render a UI. Legacy
cases also audit original Minecraft client classes for URL rewrites without
launching the game. No screenshots or visual comparisons are made.

`test-matrix.json` pins beta 1.7.3, 1.6.4, 1.7.2, 1.7.5, 1.7.6, 1.7.10,
1.8.9, 1.12.2, 1.16.5, 1.19.4, 1.20.1, 1.20.2, 1.20.4, 1.21.1, 1.21.11,
26.1, 26.2 and 26.3: authlib generations
1.5, 2, 3, 4, 5, 6, 7, 9 and 10, plus authlib 1.3-era and earlier legacy skin endpoints. Use Java 25+ for the
complete matrix; `--java /path/to/java`, `--versions VERSION ...`,
`--legacy-java /path/to/java8` and `--cache-dir PATH` select runtimes/subsets/cache.
The owner still tests actual rendering on game/loader combinations.

`--secure-skins` downloads the SHA-1-pinned Secure Skins 1.0.1 artifact from its
official Modrinth release. The test removes only the Mixin class's abstract flag
so its unchanged `useHttpsUrl` method can run without Minecraft. It then exercises
native URL-creation patterns, checks the returned PNG bytes and verifies that
remote HTTPS and other local URLs are preserved. This reproduces the reported
transport failure without screenshots or launching a game. Use Java 25+.

The owner confirmed Ornithe 1.8.9 rendering but reported a missing skin on 26.3
before the loader/transport corrections. That game/pack still needs an in-game recheck;
passing library/classloader tests alone is not evidence of visible rendering.

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
```

The first suite tests the retained Ornithe renderer, opt-in gating and missing
PNG fallback. The matrix downloads checksum-pinned official Mojang version
metadata and authlib dependencies into `target/offline-skin-tests`. It exercises
**actual authlib methods**, rather than simulations of those APIs, with JVM
bytecode verification, local/remote UUID isolation, classic/slim models, the
private PNG endpoint, rejected paths/uploads and a game-library loader whose parent excludes the system classpath. Legacy
cases also audit original Minecraft client classes for URL rewrites without
launching the game. No screenshots or visual comparisons are made.

`test-matrix.json` pins beta 1.7.3, 1.6.4, 1.7.2, 1.7.5, 1.7.6, 1.7.10,
1.8.9, 1.12.2, 1.16.5, 1.19.4, 1.20.1, 1.20.2, 1.20.4, 1.21.1, 1.21.11,
26.1, 26.2 and 26.3: authlib generations
1.5, 2, 3, 4, 5, 6, 7, 9 and 10, plus authlib 1.3-era and earlier legacy skin endpoints. Use Java 25+ for the
complete matrix; `--java /path/to/java`, `--versions VERSION ...`,
`--legacy-java /path/to/java8` and `--cache-dir PATH` select runtimes/subsets/cache.
The owner still tests actual rendering on game/loader combinations.

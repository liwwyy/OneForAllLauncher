# Ely.by preview compatibility

This Java 8 agent is enabled only for Ely.by launches of Minecraft 1.8.9.
It transforms VanillaHUD's `TabListManager.getProfile(UUID)` return values.
Valid named profiles pass through unchanged. Missing preview names are resolved
through Mojang's public profile API, preserving preview UUIDs and textures.
Network failures use stable `Preview_<id>` labels, preventing the legacy tab
renderer from dereferencing a null name. Lookups run on VanillaHUD's existing
background preview-loading thread and never send account credentials.

Account authentication, game profiles outside this preview method, and mod JAR
files on disk are unchanged. The agent uses pinned ASM 9.10.1; its BSD license
is included both here and inside the JAR. Agent source is covered by the
repository's original GPL-3.0 LICENSE, also included in the JAR.
ASM is relocated into `oneforall.compat.asm` to avoid conflicts with the game's
own ASM version. The agent JAR is also added to the Ely.by launch classpath so
Fabric's isolated mod loader can resolve the preview bridge.

Rebuild with a JDK supporting `javac --release 8` and Python 3:

```sh
python3 distribution/elyby-preview-compat/build.py
```

The build verifies the downloaded ASM SHA-256 and writes a deterministic JAR
to `packages/oneclient_core/assets/elyby-preview-compat.jar`. Release builds
embed that artifact, so launchers need no Java compiler or dependency download
to install this compatibility agent.

Diagnosis: https://github.com/Polyfrost/VanillaHUD/blob/main/src/main/kotlin/org/polyfrost/vanillahud/util/TabListManager.kt

Run console-only regression tests against cached Minecraft 1.8.9 libraries and
the installed VanillaHUD JAR (no Minecraft launch or GUI interaction):

```sh
python3 distribution/elyby-preview-compat/test.py \
  --libraries /path/to/metadata/libraries \
  --vanillahud /path/to/vanillahud-3.5.7+1.8.9.jar \
  --java /path/to/java8/bin/java --java /path/to/java25/bin/java
```

The tests mock HTTPS profile responses, so they never contact Mojang or Ely.by.
They check the actual installed preview class and use the real authlib 1.5.21
GameProfile/property classes for the transformed headless fixture.

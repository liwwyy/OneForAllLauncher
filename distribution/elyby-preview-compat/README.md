# Ely.by preview compatibility

This Java 8 agent is enabled only for Ely.by/custom-account launches of Minecraft 1.8.9.
Both use authlib-injector, which can leave VanillaHUD's Mojang preview profiles unnamed.
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

Cargo automatically compiles this agent into its `OUT_DIR` and embeds it during
launcher builds. Install Python 3 and a JDK 17+ supporting `javac --release 8`.
There is no checked-in prebuilt agent JAR. Release/nightly CI installs these tools.

To build a standalone JAR for the tests below:

```sh
python3 distribution/elyby-preview-compat/build.py
```

The standalone output is `target/elyby-preview-compat/elyby-preview-compat.jar`.
`--output /path/to/agent.jar` selects another output; Cargo uses this to write to
its build directory. The build verifies ASM's pinned SHA-256 on every use and
caches the dependency at `target/elyby-preview-compat/asm-9.10.1.jar`.
The first build needs network access; later builds reuse this verified cache.
For offline builds, `ONEFORALL_ASM_JAR` may point to the pinned dependency.
`PYTHON`, `JAVA_HOME` and `JAVAC` can select the build tools.

The compiled JAR is embedded into the launcher, so end users need no Python,
Java compiler or dependency download to install this compatibility agent.
Cargo tracks changes to Java source, build script, licenses and tool overrides;
unchanged builds reuse the generated artifact.

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

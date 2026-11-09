"""Headless integration tests for the source-built offline-only Java agent."""
import importlib.util
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('preview_build', ROOT.parent / 'elyby-preview-compat/build.py')
builder = importlib.util.module_from_spec(spec)
spec.loader.exec_module(builder)

class OfflineSkinAgentTest(unittest.TestCase):
    def test_real_instrumentation_is_local_only_and_opt_in(self):
        with tempfile.TemporaryDirectory() as directory:
            work = Path(directory)
            agent = work / 'offline-skin.jar'
            subprocess.run([sys.executable, str(ROOT / "build.py"), "--output", str(agent)], check=True, capture_output=True)
            sources = {
                'net/minecraft/unmapped/C_22168528.java': '''package net.minecraft.unmapped;
public class C_22168528 { public final String value; public C_22168528(String value) { this.value = value; } }''',
                'net/minecraft/unmapped/C_66554568.java': '''package net.minecraft.unmapped;
public class C_66554568 { public C_66554568(java.awt.image.BufferedImage image) {} }''',
                'net/minecraft/unmapped/C_82920792.java': '''package net.minecraft.unmapped;
public class C_82920792 { public C_22168528 m_87114534(String name, C_66554568 image) { return new C_22168528(name); } }''',
                'net/minecraft/unmapped/C_23527099.java': '''package net.minecraft.unmapped;
public class C_23527099 {
public C_22168528 m_68930603() { if (System.nanoTime() == 0) return null; return new C_22168528("default"); }
public String m_63406370() { return "default"; }
}''',
                'net/minecraft/unmapped/C_57778754.java': '''package net.minecraft.unmapped;
public class C_57778754 extends C_23527099 {}''',
                'net/minecraft/client/Minecraft.java': '''package net.minecraft.client;
public class Minecraft {
public static Minecraft m_20213497() { return new Minecraft(); }
public net.minecraft.unmapped.C_82920792 m_65924382() { return new net.minecraft.unmapped.C_82920792(); }
}''',
                'TestSkin.java': '''import net.minecraft.unmapped.*;
public class TestSkin {
public static void main(String[] args) throws Exception {
boolean expected = Boolean.parseBoolean(args[0]);
if (!args[1].equals("missing")) javax.imageio.ImageIO.write(new java.awt.image.BufferedImage(64,64,2), "png", new java.io.File(args[1]));
C_23527099 remote = new C_23527099();
if (!remote.m_68930603().value.equals("default") || !remote.m_63406370().equals("default")) throw new AssertionError("Changed another player's skin");
C_57778754 local = new C_57778754();
String skin = local.m_68930603().value;
if (!skin.equals(expected ? "oneforall-local-skin" : "default")) throw new AssertionError("Local skin: " + skin);
if (!local.m_63406370().equals(expected ? "slim" : "default")) throw new AssertionError("Local model");
}
}''',
            }
            for name, source in sources.items():
                path = work / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(source)
            subprocess.run([builder.javac(), '--release', '8', '-d', str(work), *[str(work / name) for name in sources]], check=True)
            java = str(Path(builder.javac()).with_name('java')) if Path(builder.javac()).is_absolute() else 'java'
            # Prepare the PNG before premain starts the private texture server.
            subprocess.run([java, '-Djava.awt.headless=true', '-cp', str(work), 'TestSkin', 'false', str(work / 'skin.png')], check=True)
            for enabled, exists in [(True, True), (False, True), (True, False)]:
                path = str(work / 'skin.png') if exists else 'missing'
                subprocess.run([java, '-Xverify:all', '-Djava.awt.headless=true', f'-javaagent:{agent}',
                                f'-Doneforall.localSkin.offlineAccount={str(enabled).lower()}',
                                f'-Doneforall.localSkin.path={path}', '-Doneforall.localSkin.slim=true',
                                '-Doneforall.localSkin.uuid=12345678-1234-1234-1234-123456789abc',
                                '-Doneforall.localSkin.username=Player', '-Doneforall.localSkin.ornithe189=true',
                                '-cp', os.pathsep.join([str(work), str(agent)]), 'TestSkin', str(enabled and exists).lower(), path], check=True)

            # A damaged/missing optional bridge must fall back instead of aborting the JVM.
            agent.with_name('offline-skin-bridge-3.jar').unlink()
            subprocess.run([java, '-Xverify:all', '-Djava.awt.headless=true', f'-javaagent:{agent}',
                            '-Doneforall.localSkin.offlineAccount=true',
                            f'-Doneforall.localSkin.path={work / "skin.png"}',
                            '-Doneforall.localSkin.uuid=12345678-1234-1234-1234-123456789abc',
                            '-Doneforall.localSkin.username=Player', '-Doneforall.localSkin.ornithe189=true',
                            '-cp', os.pathsep.join([str(work), str(agent)]), 'TestSkin', 'false', str(work / 'skin.png')], check=True)

if __name__ == '__main__':
    unittest.main()

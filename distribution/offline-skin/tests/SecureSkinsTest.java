import java.lang.reflect.*;
import java.net.*;
import java.nio.file.*;
import java.util.zip.ZipFile;
import org.objectweb.asm.*;

/** Execute the actual Secure Skins rewrite before the native URL-creation path. */
public final class SecureSkinsTest {
    private static final class FixtureLoader extends ClassLoader {
        Class<?> define(byte[] bytes) { return defineClass(null, bytes, 0, bytes.length); }
    }

    public static void main(String[] args) throws Exception {
        URL local = AuthlibTest.checkAuthlib(SecureSkinsTest.class.getClassLoader(), true,
                Boolean.parseBoolean(args[2]), args[0]);
        byte[] original;
        try (ZipFile jar = new ZipFile(args[1])) {
            java.io.InputStream input = jar.getInputStream(jar.getEntry(
                    "dev/microcontrollers/skinfix/mixin/SkinTextureDownloaderMixin.class"));
            java.io.ByteArrayOutputStream bytes = new java.io.ByteArrayOutputStream();
            byte[] buffer = new byte[4096]; int count;
            try { while ((count = input.read(buffer)) != -1) bytes.write(buffer, 0, count); }
            finally { input.close(); }
            original = bytes.toByteArray();
        }
        ClassWriter writer = new ClassWriter(0);
        new ClassReader(original).accept(new ClassVisitor(Opcodes.ASM9, writer) {
            @Override public void visit(int version, int access, String name, String signature,
                                        String parent, String[] interfaces) {
                // The Mixin has no abstract methods. Remove only its abstract
                // class flag so its unchanged URL-rewrite method can be invoked.
                super.visit(version, access & ~Opcodes.ACC_ABSTRACT, name, signature, parent, interfaces);
            }
        }, 0);
        Class<?> type = new FixtureLoader().define(writer.toByteArray());
        Object mod = type.getConstructor().newInstance();
        Method rewrite = type.getDeclaredMethod("useHttpsUrl", String.class);
        rewrite.setAccessible(true);
        String upgraded = (String) rewrite.invoke(mod, local.toString());
        require(upgraded.startsWith("https://"), "Fixture did not execute the mod's HTTPS rewrite");
        URL download = URI.create(upgraded).toURL();
        require(download.equals(local), "Secure Skins changed the private endpoint to unusable HTTPS");
        require(new URI(upgraded).toURL().equals(local), "URI constructor did not preserve the private endpoint");
        require(new URL(upgraded).equals(local), "URL constructor did not preserve the private endpoint");
        try (java.io.InputStream input = download.openStream()) {
            java.io.ByteArrayOutputStream bytes = new java.io.ByteArrayOutputStream();
            byte[] buffer = new byte[4096]; int count;
            while ((count = input.read(buffer)) != -1) bytes.write(buffer, 0, count);
            require(java.util.Arrays.equals(bytes.toByteArray(), Files.readAllBytes(Paths.get(args[0]))),
                    "The rewritten request did not return the selected PNG");
        }
        for (String external : new String[]{"http://textures.minecraft.net/texture/remote",
                "http://example.org/skin.png", "http://localhost:12345/skin.png"}) {
            String secure = (String) rewrite.invoke(mod, external);
            require(URI.create(secure).toString().equals(secure), "Changed another HTTPS endpoint");
        }
        require(URI.create(upgraded + "?other").toString().equals(upgraded + "?other"),
                "Accepted a URL other than the exact private capability");
        System.out.println("PASS: real Secure Skins rewrite, selected PNG transport, remote HTTPS unchanged");
    }

    private static void require(boolean value, String message) {
        if (!value) throw new AssertionError(message);
    }
}

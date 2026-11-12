import java.io.*;
import java.lang.reflect.*;
import java.net.*;
import java.nio.file.*;
import java.util.*;
import java.util.zip.ZipFile;
import org.objectweb.asm.*;

/** Exercise the shipped PlayerHead profile reader without Minecraft or rendering. */
public final class OneConfigHeadTest {
    private static final String OWNER = "org/polyfrost/oneconfig/api/ui/v1/internal/OneConfigUIPlayerHeadBridgeImpl";
    private static final class FixtureLoader extends ClassLoader {
        Class<?> define(byte[] bytes) { return defineClass(OWNER.replace('/', '.'), bytes, 0, bytes.length); }
    }

    public static void main(String[] args) throws Exception {
        // Test-only tripwire: a regression must fail before contacting Mojang.
        URL.setURLStreamHandlerFactory(protocol -> protocol.equals("https") ? new URLStreamHandler() {
            @Override protected URLConnection openConnection(URL url) {
                throw new AssertionError("Head reader attempted an external HTTPS request");
            }
            @Override protected URLConnection openConnection(URL url, java.net.Proxy proxy) {
                return openConnection(url);
            }
        } : null);
        URL expected = AuthlibTest.checkAuthlib(OneConfigHeadTest.class.getClassLoader(), true,
                Boolean.parseBoolean(args[2]), args[0]);
        UUID id = UUID.fromString("12345678-1234-1234-1234-123456789abc");
        // Fail before any HTTP request if the private profile route is absent.
        URI profile = new URI("https://sessionserver.mojang.com/session/minecraft/profile/"
                + id.toString().replace("-", ""));
        require(profile.getHost().equals("127.0.0.1"), "Profile would leave the computer");
        byte[] original;
        try (ZipFile jar = new ZipFile(args[1]); InputStream input = jar.getInputStream(jar.getEntry(OWNER + ".class"))) {
            original = read(input);
        }
        ClassReader reader = new ClassReader(original);
        ClassWriter writer = new ClassWriter(0);
        writer.visit(reader.readShort(6), Opcodes.ACC_PUBLIC, OWNER, null, "java/lang/Object", null);
        final Map<String, String> patterns = new LinkedHashMap<String, String>();
        final int[] copied = {0};
        reader.accept(new ClassVisitor(Opcodes.ASM9) {
            @Override public MethodVisitor visitMethod(int access, String name, String desc,
                                                      String signature, String[] exceptions) {
                if (name.equals("skinUrl") || name.equals("httpGet")) {
                    copied[0]++;
                    // Keep the actual profile request and decoding bytecode unchanged.
                    return writer.visitMethod(access, name, desc, signature, exceptions);
                }
                if (!name.equals("<clinit>")) return null;
                return new MethodVisitor(Opcodes.ASM9) {
                    private String last;
                    @Override public void visitLdcInsn(Object value) {
                        if (value instanceof String) last = (String) value;
                    }
                    @Override public void visitFieldInsn(int opcode, String owner, String name, String descriptor) {
                        if (opcode == Opcodes.PUTSTATIC && owner.equals(OWNER)
                                && (name.equals("TEXTURES_VALUE") || name.equals("SKIN_URL"))) {
                            patterns.put(name, last);
                        }
                    }
                };
            }
        }, 0);
        require(copied[0] == 2 && patterns.size() == 2, "Shipped head reader changed; inspect the fixture");
        // Stub only game/Compose initialization and the proxy, not the HTTP reader.
        MethodVisitor init = writer.visitMethod(Opcodes.ACC_PUBLIC, "<init>", "()V", null, null);
        init.visitCode();
        init.visitVarInsn(Opcodes.ALOAD, 0);
        init.visitMethodInsn(Opcodes.INVOKESPECIAL, "java/lang/Object", "<init>", "()V", false);
        init.visitInsn(Opcodes.RETURN);
        init.visitMaxs(1, 1);
        init.visitEnd();
        MethodVisitor proxy = writer.visitMethod(Opcodes.ACC_PRIVATE, "proxy", "()Ljava/net/Proxy;", null, null);
        proxy.visitCode();
        proxy.visitFieldInsn(Opcodes.GETSTATIC, "java/net/Proxy", "NO_PROXY", "Ljava/net/Proxy;");
        proxy.visitInsn(Opcodes.ARETURN);
        proxy.visitMaxs(1, 1);
        proxy.visitEnd();
        MethodVisitor clinit = writer.visitMethod(Opcodes.ACC_STATIC, "<clinit>", "()V", null, null);
        clinit.visitCode();
        for (Map.Entry<String, String> pattern : patterns.entrySet()) {
            writer.visitField(Opcodes.ACC_PRIVATE | Opcodes.ACC_STATIC, pattern.getKey(), "Lkotlin/text/Regex;", null, null).visitEnd();
            clinit.visitTypeInsn(Opcodes.NEW, "kotlin/text/Regex");
            clinit.visitInsn(Opcodes.DUP);
            clinit.visitLdcInsn(pattern.getValue());
            clinit.visitMethodInsn(Opcodes.INVOKESPECIAL, "kotlin/text/Regex", "<init>", "(Ljava/lang/String;)V", false);
            clinit.visitFieldInsn(Opcodes.PUTSTATIC, OWNER, pattern.getKey(), "Lkotlin/text/Regex;");
        }
        clinit.visitInsn(Opcodes.RETURN);
        clinit.visitMaxs(3, 0);
        clinit.visitEnd();
        writer.visitEnd();
        Class<?> fixture = new FixtureLoader().define(writer.toByteArray());
        Method lookup = fixture.getDeclaredMethod("skinUrl", UUID.class);
        lookup.setAccessible(true);
        String skin = (String) lookup.invoke(fixture.getConstructor().newInstance(), id);
        require(expected.toString().equals(skin), "Shipped PlayerHead reader did not resolve the local skin");
        try (InputStream input = new URI(skin).toURL().openStream()) {
            require(Arrays.equals(read(input), Files.readAllBytes(Paths.get(args[0]))), "Head reader received different PNG bytes");
        }
        System.out.println("PASS: shipped OneConfig 1.7.0 PlayerHead profile reader used by PolyPlus, private PNG only");
    }

    private static byte[] read(InputStream input) throws IOException {
        ByteArrayOutputStream bytes = new ByteArrayOutputStream();
        byte[] buffer = new byte[4096]; int count;
        while ((count = input.read(buffer)) != -1) bytes.write(buffer, 0, count);
        return bytes.toByteArray();
    }
    private static void require(boolean value, String message) { if (!value) throw new AssertionError(message); }
}

package oneforall.localskin;

import java.lang.instrument.ClassFileTransformer;
import java.lang.instrument.Instrumentation;
import java.security.ProtectionDomain;
import org.objectweb.asm.*;

/** Loader-independent authlib/legacy hooks, retaining the verified Ornithe renderer. */
public final class LocalSkinAgent implements ClassFileTransformer {
    private static final String PLAYER = "net/minecraft/unmapped/C_23527099";
    private static final String IDENTIFIER = "net/minecraft/unmapped/C_22168528";

    public static void premain(String arguments, Instrumentation instrumentation) {
        try {
            if (Boolean.getBoolean("oneforall.localSkin.offlineAccount")
                    && System.getProperty("oneforall.localSkin.path") != null
                    && LocalTextures.initialize()) {
                instrumentation.addTransformer(new LocalSkinAgent());
                System.err.println("[OneForAll local skin] Enabled for the selected offline account.");
            }
        } catch (Throwable error) {
            // An optional cosmetic must not abort the JVM if its bridge is unavailable.
            System.err.println("[OneForAll local skin] Skin bridge unavailable; keeping the default skin.");
        }
    }

    @Override
    public byte[] transform(ClassLoader loader, String name, Class<?> redefining,
                            ProtectionDomain domain, byte[] bytes) {
        if (loader == null || name == null || name.startsWith("java/") || name.startsWith("javax/")
                || name.startsWith("sun/") || name.startsWith("jdk/") || name.startsWith("oneforall/")) return null;
        boolean session = name.equals("com/mojang/authlib/yggdrasil/YggdrasilMinecraftSessionService")
                || name.equals("com/mojang/authlib/services/MinecraftServicesSessionService");
        boolean ornithe = PLAYER.equals(name) && Boolean.getBoolean("oneforall.localSkin.ornithe189");
        if (!session && !ornithe && !usesUrl(bytes)) return null;
        // Do not ask the game loader for bridge classes while defining another
        // class. Loader isolation/reentrant loading can otherwise skip hooks.
        if (session) return authlib(bytes);
        if (!ornithe) return urls(bytes);
        try {
            ClassReader reader = new ClassReader(bytes);
            ClassWriter writer = new ClassWriter(reader, ClassWriter.COMPUTE_MAXS);
            reader.accept(new ClassVisitor(Opcodes.ASM9, writer) {
                @Override
                public MethodVisitor visitMethod(int access, String name, String descriptor,
                                                 String signature, String[] exceptions) {
                    MethodVisitor original = super.visitMethod(access, name, descriptor, signature, exceptions);
                    final boolean skin = name.equals("m_68930603") && descriptor.equals("()L" + IDENTIFIER + ";");
                    final boolean model = name.equals("m_63406370") && descriptor.equals("()Ljava/lang/String;");
                    if (!skin && !model) return original;
                    return new MethodVisitor(Opcodes.ASM9, original) {
                        @Override
                        public void visitCode() {
                            super.visitCode();
                            visitVarInsn(Opcodes.ALOAD, 0);
                            visitMethodInsn(Opcodes.INVOKESTATIC, "oneforall/localskin/LocalSkin",
                                    skin ? "skin" : "model", "(Ljava/lang/Object;)Ljava/lang/Object;", false);
                            visitInsn(Opcodes.DUP);
                            Label fallback = new Label();
                            visitJumpInsn(Opcodes.IFNULL, fallback);
                            visitTypeInsn(Opcodes.CHECKCAST, skin ? IDENTIFIER : "java/lang/String");
                            visitInsn(Opcodes.ARETURN);
                            visitLabel(fallback);
                            visitFrame(Opcodes.F_SAME1, 0, null, 1, new Object[]{"java/lang/Object"});
                            visitInsn(Opcodes.POP);
                        }
                    };
                }
            }, 0);
            return writer.toByteArray();
        } catch (Throwable error) {
            System.err.println("[OneForAll local skin] Unsupported player class; keeping the default skin.");
            return null;
        }
    }
    private byte[] authlib(byte[] bytes) {
        try {
            ClassReader reader = new ClassReader(bytes);
            ClassWriter writer = new ClassWriter(reader, ClassWriter.COMPUTE_MAXS);
            reader.accept(new ClassVisitor(Opcodes.ASM9, writer) {
                @Override public MethodVisitor visitMethod(int access, final String name, String desc,
                                                           String signature, String[] exceptions) {
                    MethodVisitor original = super.visitMethod(access, name, desc, signature, exceptions);
                    Type[] args = Type.getArgumentTypes(desc);
                    final Type result = Type.getReturnType(desc);
                    boolean operation = name.equals("getTextures") || name.equals("getPackedTextures")
                            || name.equals("unpackTextures") || name.equals("fillProfileProperties") || name.equals("fetchProfile");
                    if (!operation || (access & Opcodes.ACC_STATIC) != 0 || args.length == 0
                            || args[0].getSort() != Type.OBJECT || result.getSort() != Type.OBJECT) return original;
                    String subject = args[0].getClassName();
                    if (!subject.equals("java.util.UUID") && !subject.equals("com.mojang.authlib.GameProfile")
                            && !subject.equals("com.mojang.authlib.properties.Property")) return original;
                    return new MethodVisitor(Opcodes.ASM9, original) {
                        @Override public void visitCode() {
                            super.visitCode();
                            visitInsn(Opcodes.ICONST_4);
                            visitTypeInsn(Opcodes.ANEWARRAY, "java/lang/Object");
                            for (int i = 0; i < 4; i++) {
                                visitInsn(Opcodes.DUP);
                                visitInsn(Opcodes.ICONST_0 + i);
                                if (i < 2) visitVarInsn(Opcodes.ALOAD, i);
                                else visitLdcInsn(i == 2 ? name : result.getClassName());
                                visitInsn(Opcodes.AASTORE);
                            }
                            invokeBridge(this, "resolve", "java/lang/Object", "java/lang/Object",
                                    "java/lang/String", "java/lang/String");
                            visitInsn(Opcodes.DUP);
                            Label fallback = new Label();
                            visitJumpInsn(Opcodes.IFNULL, fallback);
                            visitTypeInsn(Opcodes.CHECKCAST, result.getInternalName());
                            visitInsn(Opcodes.ARETURN);
                            visitLabel(fallback);
                            visitFrame(Opcodes.F_SAME1, 0, null, 1, new Object[]{"java/lang/Object"});
                            visitInsn(Opcodes.POP);
                        }
                    };
                }
            }, 0);
            return writer.toByteArray();
        } catch (RuntimeException error) {
            System.err.println("[OneForAll local skin] Unsupported authlib; keeping the default skin.");
            return null;
        }
    }

    /** The argument array is on the stack. Resolve the JDK-only bridge through
     * the bootstrap loader, independent of a mod loader's package filters. */
    private static void invokeBridge(MethodVisitor visitor, String method, String... parameters) {
        visitor.visitLdcInsn("oneforall.localskin.LocalTextures");
        visitor.visitInsn(Opcodes.ICONST_0);
        visitor.visitInsn(Opcodes.ACONST_NULL);
        visitor.visitMethodInsn(Opcodes.INVOKESTATIC, "java/lang/Class", "forName",
                "(Ljava/lang/String;ZLjava/lang/ClassLoader;)Ljava/lang/Class;", false);
        visitor.visitLdcInsn(method);
        visitor.visitInsn(Opcodes.ICONST_0 + parameters.length);
        visitor.visitTypeInsn(Opcodes.ANEWARRAY, "java/lang/Class");
        for (int i = 0; i < parameters.length; i++) {
            visitor.visitInsn(Opcodes.DUP);
            visitor.visitInsn(Opcodes.ICONST_0 + i);
            visitor.visitLdcInsn(Type.getObjectType(parameters[i]));
            visitor.visitInsn(Opcodes.AASTORE);
        }
        visitor.visitMethodInsn(Opcodes.INVOKEVIRTUAL, "java/lang/Class", "getMethod",
                "(Ljava/lang/String;[Ljava/lang/Class;)Ljava/lang/reflect/Method;", false);
        visitor.visitInsn(Opcodes.SWAP);
        visitor.visitInsn(Opcodes.ACONST_NULL);
        visitor.visitInsn(Opcodes.SWAP);
        visitor.visitMethodInsn(Opcodes.INVOKEVIRTUAL, "java/lang/reflect/Method", "invoke",
                "(Ljava/lang/Object;[Ljava/lang/Object;)Ljava/lang/Object;", false);
    }

    private static boolean usesUrl(byte[] bytes) {
        byte[] needle = {'j', 'a', 'v', 'a', '/', 'n', 'e', 't', '/', 'U', 'R'};
        outer: for (int i = 0; i <= bytes.length - needle.length; i++) {
            for (int j = 0; j < needle.length; j++) if (bytes[i + j] != needle[j]) continue outer;
            return true;
        }
        return false;
    }

    private byte[] urls(byte[] bytes) {
        // Normalize at URL creation, after skin mods have modified the argument.
        // No JDK classes or remote TLS settings are changed.
        final boolean[] changed = {false};
        ClassReader reader = new ClassReader(bytes);
        ClassWriter writer = new ClassWriter(reader, ClassWriter.COMPUTE_MAXS);
        reader.accept(new ClassVisitor(Opcodes.ASM9, writer) {
            @Override public MethodVisitor visitMethod(int access, String name, String desc,
                                                       String signature, String[] exceptions) {
                return new MethodVisitor(Opcodes.ASM9, super.visitMethod(access, name, desc, signature, exceptions)) {
                    @Override public void visitMethodInsn(int opcode, String owner, String name, String desc, boolean itf) {
                        boolean constructor = opcode == Opcodes.INVOKESPECIAL
                                && (owner.equals("java/net/URL") || owner.equals("java/net/URI"))
                                && name.equals("<init>") && desc.equals("(Ljava/lang/String;)V");
                        boolean uriFactory = opcode == Opcodes.INVOKESTATIC && owner.equals("java/net/URI")
                                && name.equals("create") && desc.equals("(Ljava/lang/String;)Ljava/net/URI;");
                        if (constructor || uriFactory) {
                            changed[0] = true;
                            // Pack the existing URL string without disturbing
                            // the pending URL constructor's uninitialized object.
                            visitInsn(Opcodes.ICONST_1);
                            visitTypeInsn(Opcodes.ANEWARRAY, "java/lang/Object");
                            visitInsn(Opcodes.DUP_X1);
                            visitInsn(Opcodes.SWAP);
                            visitInsn(Opcodes.ICONST_0);
                            visitInsn(Opcodes.SWAP);
                            visitInsn(Opcodes.AASTORE);
                            invokeBridge(this, "legacyUrl", "java/lang/String");
                            visitTypeInsn(Opcodes.CHECKCAST, "java/lang/String");
                        }
                        super.visitMethodInsn(opcode, owner, name, desc, itf);
                    }
                };
            }
        }, 0);
        return changed[0] ? writer.toByteArray() : null;
    }

}

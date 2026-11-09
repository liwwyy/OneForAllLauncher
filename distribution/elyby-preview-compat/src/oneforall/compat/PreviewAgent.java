package oneforall.compat;

import java.lang.instrument.ClassFileTransformer;
import java.lang.instrument.Instrumentation;
import java.security.ProtectionDomain;
import org.objectweb.asm.*;

/** Scoped compatibility for VanillaHUD's legacy editor preview profiles. */
public final class PreviewAgent implements ClassFileTransformer {
    public static void premain(String args, Instrumentation instrumentation) {
        instrumentation.addTransformer(new PreviewAgent());
    }

    public byte[] transform(ClassLoader loader, String name, Class<?> type,
                            ProtectionDomain domain, byte[] bytes) {
        if (!"org/polyfrost/vanillahud/util/TabListManager".equals(name)) return null;
        try {
            ClassReader reader = new ClassReader(bytes);
            ClassWriter writer = new ClassWriter(reader, 0);
            final boolean[] changed = {false};
            reader.accept(new ClassVisitor(Opcodes.ASM9, writer) {
                public MethodVisitor visitMethod(int access, String name, String descriptor,
                                                 String signature, String[] exceptions) {
                    MethodVisitor method = super.visitMethod(access, name, descriptor, signature, exceptions);
                    if (!"getProfile".equals(name) ||
                        !"(Ljava/util/UUID;)Lcom/mojang/authlib/GameProfile;".equals(descriptor)) return method;
                    return new MethodVisitor(Opcodes.ASM9, method) {
                        public void visitInsn(int opcode) {
                            if (opcode == Opcodes.ARETURN) {
                                super.visitMethodInsn(Opcodes.INVOKESTATIC, "oneforall/compat/PreviewProfiles",
                                    "resolve", "(Ljava/lang/Object;)Ljava/lang/Object;", false);
                                super.visitTypeInsn(Opcodes.CHECKCAST, "com/mojang/authlib/GameProfile");
                                changed[0] = true;
                            }
                            super.visitInsn(opcode);
                        }
                    };
                }
            }, 0);
            if (changed[0]) System.err.println("[OneForAll] Enabled Ely.by compatibility for VanillaHUD preview profiles.");
            return changed[0] ? writer.toByteArray() : null;
        } catch (Throwable failure) {
            System.err.println("[OneForAll] Could not apply VanillaHUD preview compatibility: " + failure.getClass().getSimpleName());
            return null;
        }
    }
}

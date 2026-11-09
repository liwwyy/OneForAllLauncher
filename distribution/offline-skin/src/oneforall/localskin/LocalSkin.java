package oneforall.localskin;

import java.awt.image.BufferedImage;
import java.io.File;
import javax.imageio.ImageIO;

/** Changes only the local client player. No network, account registration or uploads. */
public final class LocalSkin {
    private static Object texture;
    private static boolean attempted;

    public static synchronized Object skin(Object player) {
        try {
            ClassLoader loader = player.getClass().getClassLoader();
            Class<?> local = Class.forName("net.minecraft.unmapped.C_57778754", false, loader);
            if (!local.isInstance(player)) return null;
            if (attempted) return texture;
            attempted = true;
            BufferedImage image = ImageIO.read(new File(System.getProperty("oneforall.localSkin.path")));
            if (image == null || image.getWidth() != 64 || image.getHeight() != 64) {
                throw new IllegalArgumentException("A 64 x 64 PNG is required");
            }
            Class<?> dynamic = Class.forName("net.minecraft.unmapped.C_66554568", true, loader);
            Object imageTexture = dynamic.getConstructor(BufferedImage.class).newInstance(image);
            Class<?> minecraft = Class.forName("net.minecraft.client.Minecraft", true, loader);
            Object client = minecraft.getMethod("m_20213497").invoke(null);
            Object manager = minecraft.getMethod("m_65924382").invoke(client);
            texture = manager.getClass().getMethod("m_87114534", String.class, dynamic)
                    .invoke(manager, "oneforall-local-skin", imageTexture);
            return texture;
        } catch (Throwable error) {
            if (!attempted) attempted = true;
            System.err.println("[OneForAll local skin] Could not load the local PNG; keeping the default skin.");
            return null;
        }
    }

    public static Object model(Object player) {
        if (skin(player) == null) return null;
        return Boolean.getBoolean("oneforall.localSkin.slim") ? "slim" : "default";
    }
}

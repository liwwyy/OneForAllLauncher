import java.io.*;
import java.lang.reflect.*;
import java.net.*;
import java.nio.file.*;
import java.util.*;
import javax.imageio.ImageIO;
import java.awt.image.BufferedImage;

/** Exercises actual Mojang authlib methods with no game, GPU or internet requests. */
public final class AuthlibTest {
    private static final UUID ID = UUID.fromString("12345678-1234-1234-1234-123456789abc");

    public static void main(String[] args) throws Exception {
        if (args[0].equals("prepare")) {
            ImageIO.write(new BufferedImage(64, 64, BufferedImage.TYPE_INT_ARGB), "png", new File(args[1]));
            return;
        }
        if (args[0].equals("audit")) {
            int changed = 0;
            java.lang.instrument.ClassFileTransformer transformer = (java.lang.instrument.ClassFileTransformer)
                    Class.forName("oneforall.localskin.LocalSkinAgent").getConstructor().newInstance();
            try (java.util.zip.ZipFile jar = new java.util.zip.ZipFile(args[1])) {
                Enumeration<? extends java.util.zip.ZipEntry> entries = jar.entries();
                while (entries.hasMoreElements()) {
                    java.util.zip.ZipEntry entry = entries.nextElement();
                    if (!entry.getName().endsWith(".class")) continue;
                    ByteArrayOutputStream bytes = new ByteArrayOutputStream();
                    try (InputStream input = jar.getInputStream(entry)) {
                        byte[] buffer = new byte[4096]; int count;
                        while ((count = input.read(buffer)) != -1) bytes.write(buffer, 0, count);
                    }
                    String name = entry.getName().substring(0, entry.getName().length() - 6);
                    if (transformer.transform(AuthlibTest.class.getClassLoader(), name, null, null, bytes.toByteArray()) != null) changed++;
                }
            }
            require(changed > 0, "No real legacy skin/download classes were rewritten");
            System.out.println("PASS: rewrote URL call sites in " + changed + " original client classes");
            return;
        }
        boolean enabled = Boolean.parseBoolean(args[0]);
        boolean slim = Boolean.parseBoolean(args[1]);
        checkLegacy(enabled, args[2]);
        checkProfileLookup(enabled, args[2]);
        if (args[3].equals("legacy")) return;
        checkAuthlib(AuthlibTest.class.getClassLoader(), enabled, slim, args[2]);
        // The platform/extension parent exposes JDK libraries, but cannot see the system agent classpath.
        String[] paths = System.getProperty("java.class.path").split(java.util.regex.Pattern.quote(File.pathSeparator));
        URL[] urls = new URL[paths.length];
        for (int i = 0; i < paths.length; i++) urls[i] = new File(paths[i]).toURI().toURL();
        for (final boolean hideBridge : new boolean[]{false, true}) {
            try (URLClassLoader isolated = new URLClassLoader(urls, ClassLoader.getSystemClassLoader().getParent()) {
                @Override protected synchronized Class<?> loadClass(String name, boolean resolve) throws ClassNotFoundException {
                    // Mod loaders may refuse helper packages even when the JVM has
                    // already loaded the bridge on its bootstrap search path.
                    if (hideBridge && name.startsWith("oneforall.localskin.")) throw new ClassNotFoundException(name);
                    if (name.endsWith("YggdrasilMinecraftSessionService") || name.endsWith("MinecraftServicesSessionService")) {
                        Class<?> type = findLoadedClass(name);
                        if (type == null) type = findClass(name);
                        if (resolve) resolveClass(type);
                        return type;
                    }
                    return super.loadClass(name, resolve);
                }
            }) {
                Thread thread = Thread.currentThread();
                ClassLoader previous = thread.getContextClassLoader();
                thread.setContextClassLoader(isolated);
                try { checkAuthlib(isolated, enabled, slim, args[2]); }
                finally { thread.setContextClassLoader(previous); }
            }
        }
        System.out.println("PASS: real authlib, local UUID only, classic/slim, private PNG endpoint and isolated/restricted loaders");
    }

    private static void checkLegacy(boolean enabled, String png) throws Exception {
        URL local = new URL("http://skins.minecraft.net/MinecraftSkins/Player.png");
        require(local.getHost().equals(enabled ? "127.0.0.1" : "skins.minecraft.net"), "Legacy own skin URL");
        if (enabled) checkImage(local, png);
        require(new URL("http://skins.minecraft.net/MinecraftSkins/Other.png").getHost().equals("skins.minecraft.net"), "Changed legacy remote skin");
        require(new URL("https://example.org/MinecraftSkins/Player.png").getHost().equals("example.org"), "Changed unrelated URL");
        require(new URL("http://s3.amazonaws.com/MinecraftSkins/Player.png").getHost().equals(enabled ? "127.0.0.1" : "s3.amazonaws.com"), "Legacy S3 skin");
        require(new URL("http://www.minecraft.net/skin/Player.png").getHost().equals(enabled ? "127.0.0.1" : "www.minecraft.net"), "Legacy beta skin");
    }

    private static void checkProfileLookup(boolean enabled, String png) throws Exception {
        String endpoint = "https://sessionserver.mojang.com/session/minecraft/profile/"
                + ID.toString().replace("-", "");
        URL profile = new URI(endpoint).toURL();
        require(profile.getHost().equals(enabled ? "127.0.0.1" : "sessionserver.mojang.com"),
                "Direct head-preview profile request bypassed the local skin");
        if (enabled) {
            require(new URL(endpoint).equals(profile), "URL profile route differs from URI");
            require(URI.create(endpoint).toURL().equals(profile), "URI factory profile route differs");
            require(new URL(endpoint + "?unsigned=false").equals(profile), "Unsigned query lost local profile");
            String json;
            try (InputStream input = profile.openStream()) {
                ByteArrayOutputStream bytes = new ByteArrayOutputStream();
                byte[] buffer = new byte[4096]; int count;
                while ((count = input.read(buffer)) != -1) bytes.write(buffer, 0, count);
                json = new String(bytes.toByteArray(), "UTF-8");
            }
            require(json.contains("\"id\":\"" + ID.toString().replace("-", "") + "\""), "Wrong profile ID");
            require(!json.contains("\"signature\""), "Local profile must not pretend to be signed");
            java.util.regex.Matcher property = java.util.regex.Pattern.compile(
                    "\\\"name\\\"\\s*:\\s*\\\"textures\\\"\\s*,\\s*\\\"value\\\"\\s*:\\s*\\\"([^\\\"]+)\\\"").matcher(json);
            require(property.find(), "Head-preview reader cannot find the textures property");
            String textures = new String(Base64.getDecoder().decode(property.group(1)), "UTF-8");
            java.util.regex.Matcher skin = java.util.regex.Pattern.compile(
                    "\\\"SKIN\\\"\\s*:\\s*\\{\\s*\\\"url\\\"\\s*:\\s*\\\"([^\\\"]+)\\\"").matcher(textures);
            require(skin.find(), "Head-preview reader cannot find the skin URL");
            checkImage(new URI(skin.group(1)).toURL(), png);
            HttpURLConnection head = (HttpURLConnection) profile.openConnection();
            head.setRequestMethod("HEAD");
            require(head.getResponseCode() == 200 && head.getContentType().startsWith("application/json"), "Profile HEAD/type");
            head.disconnect();
            HttpURLConnection post = (HttpURLConnection) profile.openConnection();
            post.setRequestMethod("POST");
            require(post.getResponseCode() == 405, "Profile endpoint allowed uploads");
            post.disconnect();
        }
        for (String other : new String[]{
                endpoint.replace(ID.toString().replace("-", ""), "00000000000000000000000000000000"),
                endpoint.replace("sessionserver.mojang.com", "example.org"),
                endpoint.replace("sessionserver.mojang.com", "sessionserver.mojang.com:12345"),
                endpoint.replace("sessionserver.mojang.com", "other@sessionserver.mojang.com"),
                endpoint + "/extra", endpoint + "?other=true", endpoint + "#fragment",
                "https://api.minecraftservices.com/minecraft/profile"}) {
            require(new URI(other).toString().equals(other), "Changed an unrelated profile/service request");
        }
    }

    static URL checkAuthlib(ClassLoader loader, boolean enabled, boolean slim, String png) throws Exception {
        Class<?> profileType = Class.forName("com.mojang.authlib.GameProfile", true, loader);
        Object local = profileType.getConstructor(UUID.class, String.class).newInstance(ID, "Player");
        Object remote = profileType.getConstructor(UUID.class, String.class).newInstance(UUID.randomUUID(), "Other");
        Class<?> service;
        try { service = Class.forName("com.mojang.authlib.yggdrasil.YggdrasilMinecraftSessionService", true, loader); }
        catch (ClassNotFoundException ignored) { service = Class.forName("com.mojang.authlib.services.MinecraftServicesSessionService", true, loader); }
        Class<?> unsafeType = Class.forName("sun.misc.Unsafe");
        Field instance = unsafeType.getDeclaredField("theUnsafe");
        instance.setAccessible(true);
        Object target = unsafeType.getMethod("allocateInstance", Class.class).invoke(instance.get(null), service);
        Object texture;
        try {
            Method textures = service.getMethod("getTextures", profileType, boolean.class);
            Map<?, ?> own = (Map<?, ?>) textures.invoke(target, local, true);
            require(((Map<?, ?>) textures.invoke(target, remote, true)).isEmpty(), "Changed another profile's textures");
            if (!enabled) { require(own.isEmpty(), "Agent was enabled without opt-in"); return null; }
            texture = own.values().iterator().next();
            Object filled;
            try { filled = service.getMethod("fillProfileProperties", profileType, boolean.class).invoke(target, local, true); }
            catch (NoSuchMethodException authlib5) {
                Object fetched = service.getMethod("fetchProfile", UUID.class, boolean.class).invoke(target, ID, true);
                filled = fetched.getClass().getMethod("profile").invoke(fetched);
            }
            require(ID.equals(access(filled, "getId", "id")), "Changed the game UUID");
        } catch (NoSuchMethodException modern) {
            Method packed = service.getMethod("getPackedTextures", profileType);
            Object own = packed.invoke(target, local);
            require(packed.invoke(target, remote) == null, "Changed another profile's packed textures");
            if (!enabled) { require(own == null, "Agent was enabled without opt-in"); return null; }
            Object value = access(own, "getValue", "value");
            String json = new String(Base64.getDecoder().decode((String) value), "UTF-8");
            require(json.contains(ID.toString().replace("-", "")), "Wrong UUID in the texture property");
            Object unpacked = service.getMethod("unpackTextures", own.getClass()).invoke(target, own);
            texture = unpacked.getClass().getMethod("skin").invoke(unpacked);
            require(unpacked.getClass().getMethod("signatureState").invoke(unpacked).toString().equals("UNSIGNED"), "Local skin must not pretend to be signed");
            Object result = service.getMethod("fetchProfile", UUID.class, boolean.class).invoke(target, ID, true);
            Object fetched = result.getClass().getMethod("profile").invoke(result);
            require(ID.equals(access(fetched, "getId", "id")), "Fetched wrong profile UUID");
        }
        require(texture != null, "No local skin returned");
        URL url = new URL((String) texture.getClass().getMethod("getUrl").invoke(texture));
        require(url.getHost().equals("127.0.0.1"), "Skin endpoint is not loopback");
        try { require(texture.getClass().getMethod("getMetadata", String.class).invoke(texture, "model").equals(slim ? "slim" : "default"), "Wrong skin model"); }
        catch (NoSuchMethodException classic) { require(texture.getClass().getConstructor(String.class) != null, "Not a classic texture API"); }
        require(((String) texture.getClass().getMethod("getHash").invoke(texture)).matches("[0-9a-f]{64}"), "Unsafe texture cache key");
        checkImage(url, png);
        HttpURLConnection missing = (HttpURLConnection) new URL(url.getProtocol(), url.getHost(), url.getPort(), "/missing").openConnection();
        require(missing.getResponseCode() == 404, "Exposed a file outside the selected skin");
        missing.disconnect();
        HttpURLConnection upload = (HttpURLConnection) url.openConnection();
        upload.setRequestMethod("POST");
        require(upload.getResponseCode() == 405, "Allowed an upload");
        upload.disconnect();
        return url;
    }

    private static Object access(Object value, String oldName, String recordName) throws Exception {
        try { return value.getClass().getMethod(oldName).invoke(value); }
        catch (NoSuchMethodException ignored) { return value.getClass().getMethod(recordName).invoke(value); }
    }
    private static void checkImage(URL url, String png) throws Exception {
        ByteArrayOutputStream bytes = new ByteArrayOutputStream();
        try (InputStream input = url.openStream()) {
            byte[] buffer = new byte[4096]; int count;
            while ((count = input.read(buffer)) != -1) bytes.write(buffer, 0, count);
        }
        require(Arrays.equals(bytes.toByteArray(), Files.readAllBytes(Paths.get(png))), "Served different PNG bytes");
    }
    private static void require(boolean value, String message) { if (!value) throw new AssertionError(message); }
}

package oneforall.localskin;

import java.io.File;
import java.lang.reflect.*;
import java.net.URL;
import java.net.URLDecoder;
import java.nio.charset.StandardCharsets;
import java.util.*;

/** Authlib is not obfuscated by loaders. Reflect only at this stable library boundary. */
public final class LocalTextures {
    private static UUID id;
    private static String username;
    private static String url;
    private static String packed;
    private static boolean warned;
    private static boolean failed;
    private static boolean reported;

    public static synchronized boolean initialize() {
        if (packed != null) return true;
        if (failed) return false;
        try {
            id = UUID.fromString(System.getProperty("oneforall.localSkin.uuid"));
            username = System.getProperty("oneforall.localSkin.username");
            if (username == null) throw new IllegalArgumentException("Missing account name");
            url = new SkinServer(new File(System.getProperty("oneforall.localSkin.path"))).url();
            String json = "{\"timestamp\":" + System.currentTimeMillis() + ",\"profileId\":\""
                    + id.toString().replace("-", "") + "\",\"profileName\":\"" + escape(username)
                    + "\",\"textures\":{\"SKIN\":{\"url\":\"" + url + "\",\"metadata\":{\"model\":\""
                    + (Boolean.getBoolean("oneforall.localSkin.slim") ? "slim" : "default") + "\"}}}}";
            packed = Base64.getEncoder().encodeToString(json.getBytes(StandardCharsets.UTF_8));
            return true;
        } catch (Exception error) {
            failed = true;
            warn();
            return false;
        }
    }

    public static Object resolve(Object service, Object subject, String operation, String resultType) {
        try {
            if (!initialize()) return null;
            ClassLoader loader = service.getClass().getClassLoader();
            if (operation.equals("unpackTextures")) {
                if (subject == null || !packed.equals(access(subject, "getValue", "value"))) return null;
                return provided(textures(loader));
            }
            if (!own(subject)) return null;
            if (operation.equals("getTextures")) return provided(resultType.equals("java.util.Map") ? textureMap(loader) : textures(loader));
            if (operation.equals("getPackedTextures")) return provided(property(loader));
            Object profile = profile(loader);
            if (operation.equals("fillProfileProperties")) return profile;
            if (operation.equals("fetchProfile")) {
                return Class.forName(resultType, true, loader)
                        .getConstructor(profile.getClass()).newInstance(profile);
            }
        } catch (ReflectiveOperationException | RuntimeException error) { warn(); }
        return null;
    }

    private static boolean own(Object subject) throws ReflectiveOperationException {
        if (subject instanceof UUID) return id.equals(subject);
        return subject != null && id.equals(access(subject, "getId", "id"));
    }

    private static Object access(Object value, String oldName, String recordName) throws ReflectiveOperationException {
        try { return value.getClass().getMethod(oldName).invoke(value); }
        catch (NoSuchMethodException ignored) { return value.getClass().getMethod(recordName).invoke(value); }
    }

    private static Object property(ClassLoader loader) throws ReflectiveOperationException {
        return Class.forName("com.mojang.authlib.properties.Property", true, loader)
                .getConstructor(String.class, String.class).newInstance("textures", packed);
    }

    private static Object profile(ClassLoader loader) throws ReflectiveOperationException {
        Class<?> profile = Class.forName("com.mojang.authlib.GameProfile", true, loader);
        Class<?> map = Class.forName("com.mojang.authlib.properties.PropertyMap", true, loader);
        Object property = property(loader);
        try {
            // New authlib uses immutable properties and a record GameProfile.
            Constructor<?> constructor = profile.getConstructor(UUID.class, String.class, map);
            Class<?> multimap = Class.forName("com.google.common.collect.Multimap", true, loader);
            Object values = Class.forName("com.google.common.collect.HashMultimap", true, loader)
                    .getMethod("create").invoke(null);
            multimap.getMethod("put", Object.class, Object.class).invoke(values, "textures", property);
            return constructor.newInstance(id, username, map.getConstructor(multimap).newInstance(values));
        } catch (NoSuchMethodException ignored) {
            Object result = profile.getConstructor(UUID.class, String.class).newInstance(id, username);
            Object properties = profile.getMethod("getProperties").invoke(result);
            map.getMethod("put", Object.class, Object.class).invoke(properties, "textures", property);
            return result;
        }
    }

    @SuppressWarnings({"unchecked", "rawtypes"})
    private static Object textureMap(ClassLoader loader) throws ReflectiveOperationException {
        Class<?> texture = Class.forName("com.mojang.authlib.minecraft.MinecraftProfileTexture", true, loader);
        Class<? extends Enum> type = (Class<? extends Enum>) Class.forName(texture.getName() + "$Type", true, loader);
        Map result = new EnumMap(type);
        result.put(Enum.valueOf(type, "SKIN"), texture(loader));
        return result;
    }

    private static Object texture(ClassLoader loader) throws ReflectiveOperationException {
        Class<?> texture = Class.forName("com.mojang.authlib.minecraft.MinecraftProfileTexture", true, loader);
        Map<String, String> metadata = Collections.singletonMap("model",
                Boolean.getBoolean("oneforall.localSkin.slim") ? "slim" : "default");
        try { return texture.getConstructor(String.class, Map.class).newInstance(url, metadata); }
        catch (NoSuchMethodException classic) { return texture.getConstructor(String.class).newInstance(url); }
    }

    @SuppressWarnings({"unchecked", "rawtypes"})
    private static Object textures(ClassLoader loader) throws ReflectiveOperationException {
        Class<?> texture = Class.forName("com.mojang.authlib.minecraft.MinecraftProfileTexture", true, loader);
        Class<?> result = Class.forName("com.mojang.authlib.minecraft.MinecraftProfileTextures", true, loader);
        Class<? extends Enum> signature = (Class<? extends Enum>) Class.forName("com.mojang.authlib.SignatureState", true, loader);
        return result.getConstructor(texture, texture, texture, signature)
                .newInstance(texture(loader), null, null, Enum.valueOf(signature, "UNSIGNED"));
    }

    /** Pre-authlib clients request skins by username; change only the selected offline name. */
    public static String legacyUrl(String original) {
        try {
            if (!initialize()) return original;
            URL source = new URL(original);
            String host = source.getHost().toLowerCase(Locale.ROOT);
            String path = source.getPath();
            boolean legacy = ((host.equals("skins.minecraft.net") || host.equals("s3.amazonaws.com"))
                    && path.startsWith("/MinecraftSkins/"))
                    || ((host.equals("www.minecraft.net") || host.equals("minecraft.net")) && path.startsWith("/skin/"));
            if (!legacy || !path.endsWith(".png")) return original;
            String name = path.substring(path.lastIndexOf('/') + 1, path.length() - 4);
            if (username.equalsIgnoreCase(name) || username.equalsIgnoreCase(URLDecoder.decode(name, "UTF-8"))) return url;
        } catch (Exception ignored) {}
        return original;
    }

    private static String escape(String text) {
        StringBuilder result = new StringBuilder();
        for (char ch : text.toCharArray()) {
            if (ch == '"' || ch == '\\') result.append('\\').append(ch);
            else if (ch < 32) result.append(String.format("\\u%04x", (int) ch));
            else result.append(ch);
        }
        return result.toString();
    }

    private static synchronized void warn() {
        if (!warned) {
            warned = true;
            System.err.println("[OneForAll local skin] Could not use the local skin; keeping the default skin.");
        }
    }

    private static synchronized Object provided(Object value) {
        if (value != null && !reported) {
            reported = true;
            System.err.println("[OneForAll local skin] Providing local textures for the selected offline player.");
        }
        return value;
    }
}

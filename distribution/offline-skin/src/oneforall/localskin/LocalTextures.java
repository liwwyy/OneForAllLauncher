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
    private static String profileUrl;
    private static String packed;
    private static boolean warned;
    private static boolean failed;
    private static boolean reported;
    private static boolean reportedDownload;

    public static synchronized boolean initialize() {
        if (packed != null) return true;
        if (failed) return false;
        try {
            id = UUID.fromString(System.getProperty("oneforall.localSkin.uuid"));
            username = System.getProperty("oneforall.localSkin.username");
            if (username == null) throw new IllegalArgumentException("Missing account name");
            SkinServer server = new SkinServer(new File(System.getProperty("oneforall.localSkin.path")));
            url = server.url();
            profileUrl = server.profileUrl();
            String json = "{\"timestamp\":" + System.currentTimeMillis() + ",\"profileId\":\""
                    + id.toString().replace("-", "") + "\",\"profileName\":\"" + escape(username)
                    + "\",\"textures\":{\"SKIN\":{\"url\":\"" + url + "\",\"metadata\":{\"model\":\""
                    + (Boolean.getBoolean("oneforall.localSkin.slim") ? "slim" : "default") + "\"}}}}";
            String encoded = Base64.getEncoder().encodeToString(json.getBytes(StandardCharsets.UTF_8));
            server.setProfile("{\"id\":\"" + id.toString().replace("-", "")
                    + "\",\"name\":\"" + escape(username)
                    + "\",\"properties\":[{\"name\":\"textures\",\"value\":\"" + encoded + "\"}]}");
            packed = encoded;
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

    /** Route only the selected player's public profile and legacy skin requests. */
    public static String legacyUrl(String original) {
        try {
            if (!initialize()) return original;
            // HTTPS-enforcing skin mods can rewrite our loopback capability.
            // Restore only this exact URL; remote HTTPS and other local URLs
            // retain their original transport and validation.
            if (("https" + url.substring(4)).equals(original)
                    || ("https" + profileUrl.substring(4)).equals(original)) {
                reportDownload();
                return "http" + original.substring(5);
            }
            URL source = new URL(original);
            String host = source.getHost().toLowerCase(Locale.ROOT);
            String path = source.getPath();
            // PolyPlus PlayerHead uses OneConfig's direct session-profile lookup,
            // rather than authlib. Supply the same response shape through the
            // URL-routing boundary used by authentication injectors, scoped to
            // this offline UUID. Never redirect auth/account service endpoints.
            String scheme = source.getProtocol();
            int port = source.getPort();
            String query = source.getQuery();
            if ((scheme.equals("https") || scheme.equals("http"))
                    && host.equals("sessionserver.mojang.com")
                    && source.getUserInfo() == null && source.getRef() == null
                    && (port == -1 || port == (scheme.equals("https") ? 443 : 80))
                    && path.equals("/session/minecraft/profile/" + id.toString().replace("-", ""))
                    && (query == null || query.equals("unsigned=true") || query.equals("unsigned=false"))) {
                return profileUrl;
            }
            boolean legacy = ((host.equals("skins.minecraft.net") || host.equals("s3.amazonaws.com"))
                    && path.startsWith("/MinecraftSkins/"))
                    || ((host.equals("www.minecraft.net") || host.equals("minecraft.net")) && path.startsWith("/skin/"));
            if (!legacy || !path.endsWith(".png")) return original;
            String name = path.substring(path.lastIndexOf('/') + 1, path.length() - 4);
            if (username.equalsIgnoreCase(name) || username.equalsIgnoreCase(URLDecoder.decode(name, "UTF-8"))) return url;
        } catch (Exception ignored) {}
        return original;
    }

    private static synchronized void reportDownload() {
        if (!reportedDownload) {
            reportedDownload = true;
            System.err.println("[OneForAll local skin] Restored the private skin endpoint after an HTTPS rewrite.");
        }
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

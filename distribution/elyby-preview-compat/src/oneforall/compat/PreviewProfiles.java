package oneforall.compat;

import java.io.*;
import java.net.*;
import java.nio.charset.StandardCharsets;
import java.util.*;
import java.util.concurrent.ConcurrentHashMap;

/** Never changes valid profiles, login identities, or session authentication. */
public final class PreviewProfiles {
    private static final Map<UUID, Object> CACHE = new ConcurrentHashMap<UUID, Object>();

    public static Object resolve(Object profile) {
        if (profile == null) return null;
        try {
            Class<?> type = profile.getClass();
            String name = (String) type.getMethod("getName").invoke(profile);
            if (name != null && !name.trim().isEmpty()) return profile;
            UUID id = (UUID) type.getMethod("getId").invoke(profile);
            if (id == null) return profile;
            Object cached = CACHE.get(id);
            if (cached != null && type.isInstance(cached)) return cached;
            Object replacement = lookup(type, id);
            if (replacement == null) {
                // The editor remains usable offline or if the public lookup fails.
                replacement = type.getConstructor(UUID.class, String.class)
                    .newInstance(id, "Preview_" + id.toString().substring(0, 6));
            }
            CACHE.put(id, replacement);
            return replacement;
        } catch (ReflectiveOperationException failure) {
            return profile;
        }
    }

    private static Object lookup(Class<?> type, UUID id) {
        HttpURLConnection connection = null;
        try {
            // Public preview lookup only: never attach the Ely.by access token.
            URL url = new URL("https://sessionserver.mojang.com/session/minecraft/profile/" + id.toString().replace("-", ""));
            connection = (HttpURLConnection) url.openConnection();
            connection.setConnectTimeout(2000);
            connection.setReadTimeout(2000);
            connection.setInstanceFollowRedirects(false);
            if (connection.getResponseCode() != 200) return null;
            ByteArrayOutputStream bytes = new ByteArrayOutputStream();
            try (InputStream input = connection.getInputStream()) {
                byte[] buffer = new byte[4096];
                int count;
                while ((count = input.read(buffer)) != -1) {
                    if (bytes.size() + count > 65536) return null;
                    bytes.write(buffer, 0, count);
                }
            }
            ClassLoader loader = type.getClassLoader();
            Class<?> gsonType = Class.forName("com.google.gson.Gson", true, loader);
            Object gson = gsonType.getConstructor().newInstance();
            Map<?, ?> data = (Map<?, ?>) gsonType.getMethod("fromJson", String.class, Class.class)
                .invoke(gson, new String(bytes.toByteArray(), StandardCharsets.UTF_8), Map.class);
            Object name = data.get("name");
            if (!id.toString().replace("-", "").equalsIgnoreCase(String.valueOf(data.get("id"))) ||
                !(name instanceof String) || !((String) name).matches("[A-Za-z0-9_]{1,16}")) return null;
            Object result = type.getConstructor(UUID.class, String.class).newInstance(id, name);
            Object properties = type.getMethod("getProperties").invoke(result);
            Class<?> propertyType = Class.forName("com.mojang.authlib.properties.Property", true, loader);
            Object values = data.get("properties");
            if (values instanceof List) for (Object entry : (List<?>) values) {
                if (!(entry instanceof Map)) continue;
                Map<?, ?> value = (Map<?, ?>) entry;
                if (!"textures".equals(value.get("name")) || !(value.get("value") instanceof String)) continue;
                Object property = value.get("signature") instanceof String
                    ? propertyType.getConstructor(String.class, String.class, String.class)
                        .newInstance("textures", value.get("value"), value.get("signature"))
                    : propertyType.getConstructor(String.class, String.class)
                        .newInstance("textures", value.get("value"));
                properties.getClass().getMethod("put", Object.class, Object.class).invoke(properties, "textures", property);
            }
            return result;
        } catch (Exception failure) {
            return null;
        } finally {
            if (connection != null) connection.disconnect();
        }
    }
}

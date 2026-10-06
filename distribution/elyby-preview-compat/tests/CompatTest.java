import com.mojang.authlib.GameProfile;
import java.net.*;
import java.io.*;
import java.util.*;
import oneforall.compat.PreviewAgent;
import oneforall.compat.PreviewProfiles;
import org.polyfrost.vanillahud.util.TabListManager;
import java.util.zip.*;

public final class CompatTest {
    private static int requests;
    private static int status = 200;
    private static String response;
    public static void main(String[] args) throws Exception {
        URL.setURLStreamHandlerFactory(protocol -> "https".equals(protocol) ? new URLStreamHandler() {
            protected URLConnection openConnection(URL url) {
                if (!"sessionserver.mojang.com".equals(url.getHost())) throw new AssertionError("Unexpected host");
                return new HttpURLConnection(url) {
                    public void connect() {}
                    public void disconnect() {}
                    public boolean usingProxy() { return false; }
                    public int getResponseCode() { requests++; return status; }
                    public InputStream getInputStream() throws IOException { return new ByteArrayInputStream(response.getBytes("UTF-8")); }
                };
            }
        } : null);
        UUID id = UUID.fromString("0b4d470f-f2fb-4874-9334-1eaef8ba4804");
        response = "{\"id\":\"0b4d470ff2fb487493341eaef8ba4804\",\"name\":\"PreviewDev\",\"properties\":[{\"name\":\"textures\",\"value\":\"texture-value\",\"signature\":\"signature-value\"}]}";
        GameProfile resolved = new TabListManager().preview(id);
        require("PreviewDev".equals(resolved.getName()), "Transformed preview should receive resolved name");
        require(id.equals(resolved.getId()), "UUID must not change");
        require(resolved.getProperties().get("textures").iterator().next().getValue().equals("texture-value"), "Textures should be retained");
        require(PreviewProfiles.resolve(new GameProfile(id, null)) == resolved && requests == 1, "Repeated previews should use the cache");
        GameProfile real = new GameProfile(id, "ElyPlayer");
        require(PreviewProfiles.resolve(real) == real && requests == 1, "Valid Ely.by profile must pass through unchanged");
        status = 204;
        UUID missing = UUID.fromString("c8bf4768-af44-48cb-a259-01e42fb7bc79");
        GameProfile fallback = new TabListManager().preview(missing);
        require(fallback.getName().equals("Preview_c8bf47") && fallback.getId().equals(missing), "Missing profiles need safe stable labels");
        status = 200;
        response = "{\"id\":\"wrong\",\"name\":\"WrongUser\"}";
        UUID mismatched = UUID.randomUUID();
        require(((GameProfile) PreviewProfiles.resolve(new GameProfile(mismatched, null))).getName().startsWith("Preview_"), "Wrong UUID response must not be accepted");
        response = "{\"id\":\"0b4d470ff2fb487493341eaef8ba4804\",\"name\":\"PreviewDev\"}";
        String[] classpath = System.getProperty("java.class.path").split(java.util.regex.Pattern.quote(File.pathSeparator));
        URL[] urls = new URL[classpath.length];
        for (int i = 0; i < classpath.length; i++) urls[i] = new File(classpath[i]).toURI().toURL();
        // Exercise the bridge through an isolated mod loader, rather than only
        // the system loader used by the ordinary fixture.
        try (URLClassLoader isolated = new URLClassLoader(urls, CompatTest.class.getClassLoader()) {
            protected synchronized Class<?> loadClass(String name, boolean resolve) throws ClassNotFoundException {
                if (name.equals("org.polyfrost.vanillahud.util.TabListManager") || name.startsWith("oneforall.compat.PreviewProfiles")) {
                    Class<?> type = findLoadedClass(name);
                    if (type == null) type = findClass(name);
                    if (resolve) resolveClass(type);
                    return type;
                }
                return super.loadClass(name, resolve);
            }
        }) {
            Class<?> manager = isolated.loadClass("org.polyfrost.vanillahud.util.TabListManager");
            GameProfile isolatedProfile = (GameProfile) manager.getMethod("preview", UUID.class).invoke(manager.getConstructor().newInstance(), id);
            require("PreviewDev".equals(isolatedProfile.getName()), "Isolated mod loader must resolve the preview bridge");
        }
        try (ZipFile mod = new ZipFile(args[0])) {
            String name = "org/polyfrost/vanillahud/util/TabListManager";
            byte[] bytes;
            try (InputStream input = mod.getInputStream(mod.getEntry(name + ".class")); ByteArrayOutputStream output = new ByteArrayOutputStream()) {
                byte[] buffer = new byte[4096]; int count;
                while ((count = input.read(buffer)) != -1) output.write(buffer, 0, count);
                bytes = output.toByteArray();
            }
            PreviewAgent agent = new PreviewAgent();
            require(agent.transform(null, name, null, null, bytes) != null, "Installed VanillaHUD method must match the transformer");
            require(agent.transform(null, "com/mojang/authlib/GameProfile", null, null, bytes) == null, "Other game classes must not be transformed");
        }
        System.out.println("PASS: scoped transformation, resolved preview names/textures, unchanged Ely profiles, cache, missing/mismatched profiles, isolated mod loader and installed VanillaHUD class.");
    }
    private static void require(boolean result, String message) { if (!result) throw new AssertionError(message); }
}

import java.lang.reflect.*;
import java.nio.file.*;
import java.io.File;
import java.util.*;

/** Real Knot classpath isolation, without booting Minecraft, Mixins or a window. */
public final class FabricLoaderTest {
    public static void main(String[] args) throws Exception {
        Class<?> api = Class.forName("net.fabricmc.loader.impl.launch.knot.KnotClassLoaderInterface");
        Class<?> environment = Class.forName("net.fabricmc.api.EnvType");
        @SuppressWarnings({"unchecked", "rawtypes"})
        Object client = Enum.valueOf((Class) environment, "CLIENT");
        Method create = api.getDeclaredMethod("create", boolean.class, boolean.class, environment,
                Class.forName("net.fabricmc.loader.impl.game.GameProvider"));
        Method add = api.getDeclaredMethod("addCodeSource", Path.class);
        Method allow = api.getDeclaredMethod("setValidParentClassPath", Collection.class);
        Method get = api.getDeclaredMethod("getClassLoader");
        for (Method method : new Method[]{create, add, allow, get}) method.setAccessible(true);
        for (boolean compatibility : new boolean[]{false, true}) {
            Object delegate = create.invoke(null, compatibility, false, client, null);
            for (String path : args[0].split(java.util.regex.Pattern.quote(File.pathSeparator))) {
                add.invoke(delegate, Paths.get(path));
            }
            allow.invoke(delegate, Collections.emptyList());
            ClassLoader loader = (ClassLoader) get.invoke(delegate);
            Thread thread = Thread.currentThread();
            ClassLoader previous = thread.getContextClassLoader();
            thread.setContextClassLoader(loader);
            try {
                AuthlibTest.checkAuthlib(loader, Boolean.parseBoolean(args[1]),
                        Boolean.parseBoolean(args[2]), args[3]);
            } finally {
                thread.setContextClassLoader(previous);
            }
        }
        System.out.println("PASS: real Fabric Knot and compatibility loaders with isolated classpaths");
    }
}

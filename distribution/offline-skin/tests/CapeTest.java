import java.io.*;
import java.lang.reflect.*;
import java.net.*;
import java.nio.file.*;
import java.util.*;
import javax.imageio.ImageIO;
import java.awt.image.BufferedImage;

/** Real authlib generations: cape-only, skin+cape, disabled and URL isolation. */
public final class CapeTest {
    static final UUID ID = UUID.fromString("12345678-1234-1234-1234-123456789abc");
    public static void main(String[] args) throws Exception {
        if (args[0].equals("prepare")) {
            ImageIO.write(new BufferedImage(64,32,BufferedImage.TYPE_INT_ARGB),"png",new File(args[1]));
            return;
        }
        boolean enabled=Boolean.parseBoolean(args[0]);
        boolean withSkin=System.getProperty("oneforall.localSkin.path")!=null;
        if (args.length > 1 && args[1].equals("legacy")) {
            URL legacy=new URL("http://skins.minecraft.net/MinecraftCloaks/Player.png");
            require(legacy.getHost().equals(enabled ? "127.0.0.1" : "skins.minecraft.net"),"Legacy cape opt-in boundary");
            if (enabled) {
                try (InputStream in=legacy.openStream()) {require(Arrays.equals(read(in),Files.readAllBytes(Paths.get(System.getProperty("oneforall.localSkin.cape")))),"Wrong legacy cape");}
                require(new URL("http://s3.amazonaws.com/MinecraftCloaks/Player.png").equals(legacy),"Legacy S3 cloak not routed");
            }
            require(new URL("http://skins.minecraft.net/MinecraftCloaks/Other.png").getHost().equals("skins.minecraft.net"),"Changed remote legacy cape");
            System.out.println("PASS legacy cape transport");return;
        }
        Class<?> profile=Class.forName("com.mojang.authlib.GameProfile");
        Object own=profile.getConstructor(UUID.class,String.class).newInstance(ID,"Player");
        Object remote=profile.getConstructor(UUID.class,String.class).newInstance(UUID.randomUUID(),"Other");
        Class<?> service;
        try {service=Class.forName("com.mojang.authlib.yggdrasil.YggdrasilMinecraftSessionService");}
        catch (ClassNotFoundException e) {service=Class.forName("com.mojang.authlib.services.MinecraftServicesSessionService");}
        Class<?> unsafe=Class.forName("sun.misc.Unsafe"); Field field=unsafe.getDeclaredField("theUnsafe");field.setAccessible(true);
        Object instance=unsafe.getMethod("allocateInstance",Class.class).invoke(field.get(null),service);
        Object cape=null,skin=null;
        try {
            Method get=service.getMethod("getTextures",profile,boolean.class);
            Map<?,?> map=(Map<?,?>)get.invoke(instance,own,true);
            require(((Map<?,?>)get.invoke(instance,remote,true)).isEmpty(),"Touched remote player");
            if (!enabled) {require(map.isEmpty(),"Disabled agent supplied textures");return;}
            for (Map.Entry<?,?> e:map.entrySet()) {
                if (e.getKey().toString().equals("CAPE")) cape=e.getValue();
                if (e.getKey().toString().equals("SKIN")) skin=e.getValue();
            }
        } catch (NoSuchMethodException modern) {
            Method get=service.getMethod("getPackedTextures",profile);
            Object property=get.invoke(instance,own);
            require(get.invoke(instance,remote)==null,"Touched remote profile");
            if (!enabled) {require(property==null,"Disabled agent supplied a profile");return;}
            Object textures=service.getMethod("unpackTextures",property.getClass()).invoke(instance,property);
            cape=textures.getClass().getMethod("cape").invoke(textures);
            skin=textures.getClass().getMethod("skin").invoke(textures);
            require(textures.getClass().getMethod("signatureState").invoke(textures).toString().equals("UNSIGNED"),"Forged signature state");
        }
        require(cape!=null,"Missing cape"); require((skin!=null)==withSkin,"Changed skin selection");
        String address=(String)cape.getClass().getMethod("getUrl").invoke(cape);
        require(((String)cape.getClass().getMethod("getHash").invoke(cape)).matches("[0-9a-f]{64}"),"Unsafe cape cache key");
        URL url=new URL(address);
        byte[] expected=Files.readAllBytes(Paths.get(System.getProperty("oneforall.localSkin.cape")));
        try (InputStream in=url.openStream()) {require(Arrays.equals(expected,read(in)),"Wrong cape bytes");}
        URI rewritten=new URI("https"+address.substring(4));
        require(rewritten.getScheme().equals("http"),"Private cape HTTPS rewrite not restored");
        for (String other:new String[]{address+"/other",address+"?q=1",address.replace("127.0.0.1","localhost")}) {
            String https="https"+other.substring(4);
            require(new URI(https).toString().equals(https),"Broadened HTTP exception");
        }
        URL legacy=new URL("http://skins.minecraft.net/MinecraftCloaks/Player.png");
        require(legacy.toString().equals(address),"Legacy own cape not routed");
        require(new URL("http://skins.minecraft.net/MinecraftCloaks/Other.png").getHost().equals("skins.minecraft.net"),"Touched another cape");
        for (String other:new String[]{"http://skins.minecraft.net/MinecraftCloaks/Player.png?q=1",
                "http://skins.minecraft.net:12345/MinecraftCloaks/Player.png", "http://user@skins.minecraft.net/MinecraftCloaks/Player.png"}) {
            require(new URI(other).toString().equals(other),"Broadened legacy cloak route");
        }
        HttpURLConnection missing=(HttpURLConnection)new URL(url.getProtocol(),url.getHost(),url.getPort(),"/missing").openConnection();
        require(missing.getResponseCode()==404,"Exposed arbitrary file paths"); missing.disconnect();
        System.out.println("PASS real authlib cape"+(withSkin?" with skin":" only"));
    }
    static byte[] read(InputStream input) throws IOException {ByteArrayOutputStream out=new ByteArrayOutputStream();byte[] b=new byte[1024];int n;while((n=input.read(b))!=-1)out.write(b,0,n);return out.toByteArray();}
    static void require(boolean condition,String message) {if(!condition)throw new AssertionError(message);}
}

package oneforall.localskin;

import java.io.*;
import java.net.*;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.security.MessageDigest;
import java.util.UUID;
import javax.imageio.ImageIO;
import java.awt.image.BufferedImage;

/** Serves one skin and its unsigned profile on private loopback capability paths. */
final class SkinServer {
    private final ServerSocket server;
    private final byte[] png;
    private final byte[] cape;
    private final String path;
    private final String capePath;
    private volatile byte[] profile;

    SkinServer(File file) throws Exception { this(file, null); }

    SkinServer(File file, File capeFile) throws Exception {
        png = file == null ? null : readPng(file, 64, 64);
        cape = capeFile == null ? null : readPng(capeFile, 64, 32);
        if (png == null && cape == null) throw new IOException("No local textures selected");
        StringBuilder hash = new StringBuilder();
        for (byte value : MessageDigest.getInstance("SHA-256").digest(png != null ? png : cape)) {
            hash.append(String.format("%02x", value & 255));
        }
        path = "/" + UUID.randomUUID() + "/" + hash;
        StringBuilder capeHash = new StringBuilder();
        if (cape != null) for (byte value : MessageDigest.getInstance("SHA-256").digest(cape)) capeHash.append(String.format("%02x", value & 255));
        capePath = path + "/cape/" + capeHash;
        server = new ServerSocket();
        server.bind(new InetSocketAddress(InetAddress.getByName("127.0.0.1"), 0));
        Thread worker = new Thread(new Runnable() {
            @Override public void run() { serve(); }
        }, "OneForAll local skin");
        worker.setDaemon(true);
        worker.start();
        Runtime.getRuntime().addShutdownHook(new Thread(new Runnable() {
            @Override public void run() { try { server.close(); } catch (IOException ignored) {} }
        }));
    }

    String url() { return "http://127.0.0.1:" + server.getLocalPort() + path; }

    String capeUrl() { return "http://127.0.0.1:" + server.getLocalPort() + capePath; }

    private static byte[] readPng(File file, int width, int height) throws Exception {
        if (file.length() > 1024 * 1024) throw new IOException("Texture exceeds 1 MB");
        byte[] data = Files.readAllBytes(file.toPath());
        if (data.length < 24 || data[0] != (byte)137 || data[1] != 80 || data[2] != 78 || data[3] != 71
                || java.nio.ByteBuffer.wrap(data, 16, 8).getInt() != width
                || java.nio.ByteBuffer.wrap(data, 20, 4).getInt() != height) throw new IOException("Invalid texture dimensions");
        BufferedImage image = ImageIO.read(new ByteArrayInputStream(data));
        if (image == null || image.getWidth() != width || image.getHeight() != height) throw new IOException("Invalid PNG");
        return data;
    }

    String profileUrl() { return url() + "/profile"; }

    void setProfile(String json) { profile = json.getBytes(StandardCharsets.UTF_8); }

    private void serve() {
        while (!server.isClosed()) {
            try (Socket socket = server.accept()) {
                socket.setSoTimeout(2000);
                InputStream input = socket.getInputStream();
                ByteArrayOutputStream header = new ByteArrayOutputStream();
                int last = 0;
                while (header.size() < 8192) {
                    int next = input.read();
                    if (next < 0) break;
                    header.write(next);
                    last = (last << 8) | next;
                    if (last == 0x0d0a0d0a) break;
                }
                String request = new String(header.toByteArray(), StandardCharsets.US_ASCII);
                String[] parts = request.split("\\r\\n", 2)[0].split(" ");
                boolean head = parts.length == 3 && "HEAD".equals(parts[0]);
                boolean method = parts.length == 3 && (head || "GET".equals(parts[0]));
                byte[] body = null;
                String type = "image/png";
                if (method && path.equals(parts[1])) body = png;
                else if (method && capePath.equals(parts[1])) body = cape;
                else if (method && (path + "/profile").equals(parts[1])) {
                    body = profile;
                    type = "application/json; charset=utf-8";
                }
                boolean found = body != null;
                String status = found ? "200 OK" : method ? "404 Not Found" : "405 Method Not Allowed";
                OutputStream output = socket.getOutputStream();
                output.write(("HTTP/1.1 " + status + "\r\nContent-Type: " + type + "\r\nContent-Length: "
                        + (found ? body.length : 0) + "\r\nConnection: close\r\nCache-Control: private, max-age=86400\r\n\r\n")
                        .getBytes(StandardCharsets.US_ASCII));
                if (found && !head) output.write(body);
                output.flush();
            } catch (IOException ignored) {
                // A cancelled image request must not stop later requests.
            }
        }
    }
}

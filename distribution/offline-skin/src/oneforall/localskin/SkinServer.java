package oneforall.localskin;

import java.io.*;
import java.net.*;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.security.MessageDigest;
import java.util.UUID;
import javax.imageio.ImageIO;
import java.awt.image.BufferedImage;

/** Serves one immutable PNG on loopback. No uploads, external requests or file endpoints. */
final class SkinServer {
    private final ServerSocket server;
    private final byte[] png;
    private final String path;

    SkinServer(File file) throws Exception {
        if (file.length() > 1024 * 1024) throw new IOException("Skin exceeds 1 MB");
        png = Files.readAllBytes(file.toPath());
        BufferedImage image = ImageIO.read(new ByteArrayInputStream(png));
        if (image == null || image.getWidth() != 64 || image.getHeight() != 64) {
            throw new IOException("A 64 x 64 PNG is required");
        }
        StringBuilder hash = new StringBuilder();
        for (byte value : MessageDigest.getInstance("SHA-256").digest(png)) {
            hash.append(String.format("%02x", value & 255));
        }
        path = "/" + UUID.randomUUID() + "/" + hash;
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
                boolean found = method && path.equals(parts[1]);
                String status = found ? "200 OK" : method ? "404 Not Found" : "405 Method Not Allowed";
                OutputStream output = socket.getOutputStream();
                output.write(("HTTP/1.1 " + status + "\r\nContent-Type: image/png\r\nContent-Length: "
                        + (found ? png.length : 0) + "\r\nConnection: close\r\nCache-Control: private, max-age=86400\r\n\r\n")
                        .getBytes(StandardCharsets.US_ASCII));
                if (found && !head) output.write(png);
                output.flush();
            } catch (IOException ignored) {
                // A cancelled image request must not stop later requests.
            }
        }
    }
}

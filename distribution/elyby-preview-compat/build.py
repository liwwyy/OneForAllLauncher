#!/usr/bin/env python3
"""Rebuild the embedded Java 8 agent with a pinned ASM dependency."""
import argparse
import hashlib
import os
import shutil
import pathlib
import subprocess
import struct
import tempfile
import urllib.request
import zipfile

ROOT = pathlib.Path(__file__).resolve().parent
ASM_URL = "https://repo.maven.apache.org/maven2/org/ow2/asm/asm/9.10.1/asm-9.10.1.jar"
ASM_SHA256 = "ed825d10ab1399c8c0cb669e688cf0c8c82629b4c8399b58352b68e92ca10fcb"
DEFAULT_CACHE = ROOT.parents[1] / "target/elyby-preview-compat"

def javac():
    if os.environ.get("JAVAC"):
        return os.environ["JAVAC"]
    if os.environ.get("JAVA_HOME"):
        return str(pathlib.Path(os.environ["JAVA_HOME"]) / "bin" / ("javac.exe" if os.name == "nt" else "javac"))
    path = shutil.which("javac")
    if not path:
        raise RuntimeError("Install JDK 17+ or set JAVA_HOME/JAVAC; a Java runtime alone is not enough.")
    return path

def verified_asm(cache):
    override = os.environ.get("ONEFORALL_ASM_JAR")
    path = pathlib.Path(override) if override else cache / "asm-9.10.1.jar"
    if not path.is_file():
        if override:
            raise RuntimeError(f"ONEFORALL_ASM_JAR does not exist: {path}")
        cache.mkdir(parents=True, exist_ok=True)
        data = urllib.request.urlopen(ASM_URL, timeout=30).read()
        if hashlib.sha256(data).hexdigest() != ASM_SHA256:
            raise RuntimeError("Downloaded ASM failed its SHA-256 check")
        with tempfile.NamedTemporaryFile(dir=cache, delete=False) as temporary:
            temporary.write(data)
            download = pathlib.Path(temporary.name)
        os.replace(download, path)
    if hashlib.sha256(path.read_bytes()).hexdigest() != ASM_SHA256:
        raise RuntimeError(f"ASM failed its SHA-256 check: {path}. Remove it and rebuild, or supply the pinned dependency.")
    return path

def relocate_class(data):
    """Relocate ASM's constant-pool names so game ASM versions cannot conflict."""
    assert data[:4] == b"\xca\xfe\xba\xbe"
    output = bytearray(data[:10])
    offset = 10
    index = 1
    count = struct.unpack_from(">H", data, 8)[0]
    widths = {3: 4, 4: 4, 5: 8, 6: 8, 7: 2, 8: 2, 9: 4, 10: 4, 11: 4, 12: 4, 15: 3, 16: 2, 17: 4, 18: 4, 19: 2, 20: 2}
    while index < count:
        tag = data[offset]
        offset += 1
        output.append(tag)
        if tag == 1:
            size = struct.unpack_from(">H", data, offset)[0]
            offset += 2
            value = data[offset:offset + size].replace(b"org/objectweb/asm", b"oneforall/compat/asm").replace(b"org.objectweb.asm", b"oneforall.compat.asm")
            output.extend(struct.pack(">H", len(value)))
            output.extend(value)
            offset += size
        else:
            size = widths[tag]
            output.extend(data[offset:offset + size])
            offset += size
            if tag in (5, 6): index += 1
        index += 1
    output.extend(data[offset:])
    return bytes(output)
def build(output, cache):
    asm = verified_asm(cache)
    with tempfile.TemporaryDirectory() as directory:
        work = pathlib.Path(directory)
        classes = work / "classes"
        subprocess.run([javac(), "--release", "8", "-Xlint:-options", "-cp", str(asm), "-d", str(classes),
                        *map(str, sorted((ROOT / "src").rglob("*.java")))], check=True)
        entries = {p.relative_to(classes).as_posix(): relocate_class(p.read_bytes()) for p in classes.rglob("*.class")}
        with zipfile.ZipFile(asm) as source:
            for name in source.namelist():
                if name.startswith("org/objectweb/asm/") and name.endswith(".class"):
                    entries[name.replace("org/objectweb/asm/", "oneforall/compat/asm/")] = relocate_class(source.read(name))
        entries["META-INF/MANIFEST.MF"] = b"Manifest-Version: 1.0\r\nPremain-Class: oneforall.compat.PreviewAgent\r\n\r\n"
        entries["META-INF/LICENSE-ASM.txt"] = (ROOT / "LICENSE-ASM.txt").read_bytes()
        entries["META-INF/LICENSE-OneForAll.txt"] = (ROOT.parents[1] / "LICENSE").read_bytes()
        jar = work / "agent.jar"
        with zipfile.ZipFile(jar, "w", compression=zipfile.ZIP_DEFLATED, compresslevel=9) as archive:
            for name, content in sorted(entries.items()):
                info = zipfile.ZipInfo(name, (2026, 1, 1, 0, 0, 0))
                info.compress_type = zipfile.ZIP_DEFLATED
                info.create_system = 3
                info.external_attr = 0o644 << 16
                archive.writestr(info, content)
        output.parent.mkdir(parents=True, exist_ok=True)
        # Atomic replacement also works when build output and the temp directory use different disks.
        with tempfile.NamedTemporaryFile(dir=output.parent, delete=False) as temporary:
            temporary.write(jar.read_bytes())
            staged = pathlib.Path(temporary.name)
        os.replace(staged, output)
    return hashlib.sha256(output.read_bytes()).hexdigest()

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=pathlib.Path, default=DEFAULT_CACHE / "elyby-preview-compat.jar")
    parser.add_argument("--cache-dir", type=pathlib.Path, default=DEFAULT_CACHE)
    args = parser.parse_args()
    print(build(args.output, args.cache_dir))
    print(args.output)

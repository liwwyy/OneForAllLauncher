#!/usr/bin/env python3
"""Rebuild the embedded Java 8 agent with a pinned ASM dependency."""
import hashlib
import pathlib
import subprocess
import struct
import tempfile
import urllib.request
import zipfile

ROOT = pathlib.Path(__file__).resolve().parent
ASM_URL = "https://repo.maven.apache.org/maven2/org/ow2/asm/asm/9.10.1/asm-9.10.1.jar"
ASM_SHA256 = "ed825d10ab1399c8c0cb669e688cf0c8c82629b4c8399b58352b68e92ca10fcb"
OUTPUT = ROOT.parents[1] / "packages/oneclient_core/assets/elyby-preview-compat.jar"

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
with tempfile.TemporaryDirectory() as directory:
    work = pathlib.Path(directory)
    asm = work / "asm.jar"
    asm.write_bytes(urllib.request.urlopen(ASM_URL, timeout=30).read())
    assert hashlib.sha256(asm.read_bytes()).hexdigest() == ASM_SHA256
    classes = work / "classes"
    subprocess.run(["javac", "--release", "8", "-Xlint:-options", "-cp", str(asm), "-d", str(classes),
                    *map(str, sorted((ROOT / "src").rglob("*.java")))], check=True)
    entries = {p.relative_to(classes).as_posix(): relocate_class(p.read_bytes()) for p in classes.rglob("*.class")}
    with zipfile.ZipFile(asm) as source:
        for name in source.namelist():
            if name.startswith("org/objectweb/asm/") and name.endswith(".class"):
                entries[name.replace("org/objectweb/asm/", "oneforall/compat/asm/")] = relocate_class(source.read(name))
    entries["META-INF/MANIFEST.MF"] = b"Manifest-Version: 1.0\r\nPremain-Class: oneforall.compat.PreviewAgent\r\n\r\n"
    entries["META-INF/LICENSE-ASM.txt"] = (ROOT / "LICENSE-ASM.txt").read_bytes()
    entries["META-INF/LICENSE-OneForAll.txt"] = (ROOT.parents[1] / "LICENSE").read_bytes()
    OUTPUT.parent.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(OUTPUT, "w", compression=zipfile.ZIP_DEFLATED, compresslevel=9) as archive:
        for name, content in sorted(entries.items()):
            info = zipfile.ZipInfo(name, (2026, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_DEFLATED
            info.create_system = 3
            info.external_attr = 0o644 << 16
            archive.writestr(info, content)
print(OUTPUT)
print(hashlib.sha256(OUTPUT.read_bytes()).hexdigest())

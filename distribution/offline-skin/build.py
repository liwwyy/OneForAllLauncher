#!/usr/bin/env python3
"""Build the agent and a bootstrap bridge from auditable Java source."""
import argparse
import hashlib
import io
import os
import pathlib
import sys
import tempfile
import zipfile

ROOT = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(ROOT.parent / "elyby-preview-compat"))
from build import build as compile_agent, DEFAULT_CACHE

BRIDGE = "offline-skin-bridge-2.jar"


def archive(entries):
    output = io.BytesIO()
    with zipfile.ZipFile(output, "w", compression=zipfile.ZIP_DEFLATED, compresslevel=9) as jar:
        for name, content in sorted(entries.items()):
            info = zipfile.ZipInfo(name, (2026, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_DEFLATED
            info.create_system = 3
            info.external_attr = 0o644 << 16
            jar.writestr(info, content)
    return output.getvalue()


def atomic_write(path, content):
    with tempfile.NamedTemporaryFile(dir=path.parent, delete=False) as staged:
        staged.write(content)
        temporary = pathlib.Path(staged.name)
    os.replace(temporary, path)


def build(output, cache):
    compile_agent(output, cache, ROOT, "oneforall.localskin.LocalSkinAgent")
    with zipfile.ZipFile(output) as jar:
        entries = {name: jar.read(name) for name in jar.namelist()}
    # Only the small JDK-only bridge is placed on the bootstrap path. ASM and
    # transformer classes stay in the agent loader; no game libraries are shadowed.
    bridge = {}
    for name in list(entries):
        if name.startswith(("oneforall/localskin/LocalTextures", "oneforall/localskin/SkinServer")) or name == "oneforall/localskin/LocalSkin.class":
            bridge[name] = entries.pop(name)
    bridge["META-INF/MANIFEST.MF"] = b"Manifest-Version: 1.0\r\n\r\n"
    bridge["META-INF/LICENSE-OneForAll.txt"] = entries["META-INF/LICENSE-OneForAll.txt"]
    entries["META-INF/MANIFEST.MF"] = ("Manifest-Version: 1.0\r\nPremain-Class: oneforall.localskin.LocalSkinAgent\r\n"
                                       f"Boot-Class-Path: {BRIDGE}\r\n\r\n").encode()
    atomic_write(output.with_name(BRIDGE), archive(bridge))
    atomic_write(output, archive(entries))
    return hashlib.sha256(output.read_bytes()).hexdigest()


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=pathlib.Path, required=True)
    parser.add_argument("--cache-dir", type=pathlib.Path, default=DEFAULT_CACHE)
    args = parser.parse_args()
    print(build(args.output, args.cache_dir))

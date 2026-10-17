#!/usr/bin/env python3
"""Build cargo-packager-updater metadata for this fork's signed release assets."""
import argparse
import base64
import datetime
import json
from pathlib import Path

TARGETS = {
    "windows-x86_64": ("windows_x86_64.exe", "nsis"),
    "linux-x86_64": ("linux_x86_64.AppImage", "appimage"),
    "darwin-aarch64": ("darwin_aarch64.app.tar.gz", "app"),
    "darwin-x86_64": ("darwin_x86_64.app.tar.gz", "app"),
}


def manifest(directory, version, repository, tag, notes):
    platforms = {}
    for target, (suffix, package_format) in TARGETS.items():
        filename = f"OneForAllLauncher_{version}_{suffix}"
        artifact = directory / filename
        if not artifact.is_file() or artifact.stat().st_size == 0:
            raise ValueError(f"Missing update package: {filename}")
        signature = (directory / (filename + ".sig")).read_text().strip()
        if not signature or not base64.b64decode(signature, validate=True):
            raise ValueError(f"Invalid signature file: {filename}.sig")
        entry = {
            "url": f"https://github.com/{repository}/releases/download/{tag}/{filename}",
            "signature": signature,
            "format": package_format,
        }
        platforms[target] = entry
        if target.startswith("darwin-"):
            platforms[target.replace("darwin-", "macos-")] = entry
    return {
        "version": version,
        "notes": notes,
        "pub_date": datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "platforms": platforms,
    }


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    parser.add_argument("version")
    parser.add_argument("repository")
    parser.add_argument("tag")
    parser.add_argument("--notes", type=Path, required=True)
    args = parser.parse_args()
    result = manifest(args.directory, args.version, args.repository, args.tag, args.notes.read_text())
    (args.directory / "latest.json").write_text(json.dumps(result, indent=2) + "\n")

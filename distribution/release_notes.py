#!/usr/bin/env python3
"""Print only the requested fork version; fail rather than publish old notes."""
import pathlib
import re
import sys


def release_notes(changelog: str, version: str) -> str:
    sections = re.split(r"(?=^#{1,2} )", changelog, flags=re.MULTILINE)
    prefix = f"## OneForAllLauncher {version} — "
    matches = [section.strip() for section in sections if section.startswith(prefix)]
    if len(matches) != 1:
        raise ValueError(f"Expected one changelog section for {version}, found {len(matches)}")
    return matches[0] + "\n"


if __name__ == "__main__":
    changelog = pathlib.Path(__file__).resolve().parents[1] / "FORK_CHANGELOG.md"
    print(release_notes(changelog.read_text(), sys.argv[1]), end="")

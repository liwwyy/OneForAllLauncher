#!/usr/bin/env python3
"""Check stable OneClient ancestry and optionally publish a Shields endpoint."""
import argparse
import json
import os
import re
import subprocess
import sys
from urllib.error import HTTPError
from urllib.request import Request, urlopen

UPSTREAM = "Polyfrost/OneLauncher"
BRANCH = "badges"


def api(path, method="GET", data=None, missing_ok=False):
    headers = {"Accept": "application/vnd.github+json", "User-Agent": "OneForAllLauncher-sync"}
    if token := os.environ.get("GH_TOKEN"):
        headers["Authorization"] = f"Bearer {token}"
    body = None if data is None else json.dumps(data).encode()
    if body is not None:
        headers["Content-Type"] = "application/json"
    request = Request(f"https://api.github.com/{path}", data=body, headers=headers, method=method)
    try:
        with urlopen(request, timeout=30) as response:
            return json.load(response)
    except HTTPError as error:
        if missing_ok and error.code == 404:
            return None
        raise


def latest_release():
    page = 1
    while releases := api(f"repos/{UPSTREAM}/releases?per_page=100&page={page}"):
        for release in releases:
            if (not release["draft"] and not release["prerelease"]
                    and re.fullmatch(r"oneclient-\d+\.\d+\.\d+", release["tag_name"])):
                return release["tag_name"]
        page += 1
    raise ValueError("No stable OneClient release found")


def sync_badge():
    badge = {"schemaVersion": 1, "label": "Upstream sync"}
    try:
        tag = latest_release()
        subprocess.run(
            ["git", "fetch", "--no-tags", f"https://github.com/{UPSTREAM}.git", f"refs/tags/{tag}"],
            check=True, capture_output=True, timeout=90,
        )
        result = subprocess.run(
            ["git", "merge-base", "--is-ancestor", "FETCH_HEAD^{commit}", "HEAD"],
            capture_output=True, timeout=30,
        )
        if result.returncode not in (0, 1):
            raise ValueError("Unable to check release ancestry")
        current = result.returncode == 0
        state = "Up to date" if current else "Update available"
        badge.update(message=f"{state} ({tag.removeprefix('oneclient-')})",
                     color="f5c2e7" if current else "orange")
    except (OSError, ValueError, subprocess.SubprocessError):
        print("Upstream release check unavailable; publishing an unavailable status.", file=sys.stderr)
        badge.update(message="Check unavailable", color="lightgrey", isError=True)
    return badge


def publish(repository, badge):
    """Keep generated data off main and avoid commits when the status is unchanged."""
    if not re.fullmatch(r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+", repository):
        raise ValueError("Invalid repository")
    root = f"repos/{repository}/git"
    ref = api(f"{root}/ref/heads/{BRANCH}", missing_ok=True)
    parent = ref["object"]["sha"] if ref else None
    previous = api(f"{root}/commits/{parent}") if parent else None
    tree_data = {"tree": [{
        "path": "upstream-sync.json", "mode": "100644", "type": "blob",
        "content": json.dumps(badge, indent=2) + "\n",
    }]}
    if previous:
        tree_data["base_tree"] = previous["tree"]["sha"]
    tree = api(f"{root}/trees", "POST", tree_data)
    if previous and previous["tree"]["sha"] == tree["sha"]:
        return
    commit = api(f"{root}/commits", "POST", {
        "message": "Update stable upstream sync status", "tree": tree["sha"],
        "parents": [parent] if parent else [],
    })
    if parent:
        api(f"{root}/refs/heads/{BRANCH}", "PATCH", {"sha": commit["sha"], "force": False})
    else:
        api(f"{root}/refs", "POST", {"ref": f"refs/heads/{BRANCH}", "sha": commit["sha"]})


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--publish", action="store_true")
    args = parser.parse_args()
    result = sync_badge()
    print(json.dumps(result, indent=2))
    if args.publish:
        publish(os.environ["GITHUB_REPOSITORY"], result)

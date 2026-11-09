import subprocess
import unittest
from unittest.mock import patch

import upstream_sync


class UpstreamSyncTests(unittest.TestCase):
    def test_ignores_drafts_prereleases_and_other_package_releases(self):
        releases = [
            {"tag_name": "oneclient-3.0.0", "draft": True, "prerelease": False},
            {"tag_name": "oneclient-2.11.0", "draft": False, "prerelease": True},
            {"tag_name": "oneclient_core-4.0.0", "draft": False, "prerelease": False},
        ]
        stable = {"tag_name": "oneclient-2.10.0", "draft": False, "prerelease": False}
        with patch.object(upstream_sync, "api", side_effect=[releases, [stable]]) as api:
            self.assertEqual(upstream_sync.latest_release(), "oneclient-2.10.0")
            self.assertIn("page=2", api.call_args.args[0])

    def test_ancestry_controls_status_instead_of_fork_version_number(self):
        for status, expected in [(0, "Up to date"), (1, "Update available")]:
            with self.subTest(status=status), \
                    patch.object(upstream_sync, "latest_release", return_value="oneclient-2.10.0"), \
                    patch.object(upstream_sync.subprocess, "run", side_effect=[
                        subprocess.CompletedProcess([], 0), subprocess.CompletedProcess([], status),
                    ]) as run:
                badge = upstream_sync.sync_badge()
                self.assertEqual(badge["message"], f"{expected} (2.10.0)")
                self.assertIn("FETCH_HEAD^{commit}", run.call_args.args[0])

    def test_failed_fetch_or_invalid_history_never_reports_up_to_date(self):
        for failure in [subprocess.CalledProcessError(128, "git"), OSError("unavailable")]:
            with self.subTest(failure=type(failure).__name__), \
                    patch.object(upstream_sync, "latest_release", return_value="oneclient-2.10.0"), \
                    patch.object(upstream_sync.subprocess, "run", side_effect=failure):
                self.assertTrue(upstream_sync.sync_badge()["isError"])
        with patch.object(upstream_sync, "latest_release", return_value="oneclient-2.10.0"), \
                patch.object(upstream_sync.subprocess, "run", side_effect=[
                    subprocess.CompletedProcess([], 0), subprocess.CompletedProcess([], 128),
                ]):
            self.assertEqual(upstream_sync.sync_badge()["message"], "Check unavailable")

    def test_unchanged_status_does_not_commit(self):
        with patch.object(upstream_sync, "api", side_effect=[
            {"object": {"sha": "parent"}}, {"tree": {"sha": "same"}}, {"sha": "same"},
        ]) as api:
            upstream_sync.publish("owner/repo", {"message": "Up to date"})
            self.assertEqual(api.call_count, 3)
            self.assertEqual(api.call_args.args[2]["base_tree"], "same")

    def test_first_publication_creates_only_badges_branch(self):
        with patch.object(upstream_sync, "api", side_effect=[
            None, {"sha": "tree"}, {"sha": "commit"}, {},
        ]) as api:
            upstream_sync.publish("owner/repo", {"message": "Up to date"})
            self.assertEqual(api.call_args_list[2].args[2]["parents"], [])
            self.assertEqual(api.call_args.args[2]["ref"], "refs/heads/badges")

    def test_updates_preserve_existing_tree_and_never_force(self):
        with patch.object(upstream_sync, "api", side_effect=[
            {"object": {"sha": "parent"}}, {"tree": {"sha": "old-tree"}},
            {"sha": "new-tree"}, {"sha": "commit"}, {},
        ]) as api:
            upstream_sync.publish("owner/repo", {"message": "Update available"})
            self.assertEqual(api.call_args_list[2].args[2]["base_tree"], "old-tree")
            self.assertEqual(api.call_args.args[2], {"sha": "commit", "force": False})


if __name__ == "__main__":
    unittest.main()

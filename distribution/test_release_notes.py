import sys
import unittest
sys.dont_write_bytecode = True
from release_notes import release_notes


class ReleaseNotesTests(unittest.TestCase):
    def test_publishes_only_requested_version(self):
        changelog = "Intro\n## OneForAllLauncher 2.6.6 — today\n\n- New\n\n# OneForAllLauncher 2.6.5 — yesterday\n\n- Old\n"
        self.assertEqual(release_notes(changelog, "2.6.6"), "## OneForAllLauncher 2.6.6 — today\n\n- New\n")

    def test_missing_or_duplicate_versions_fail(self):
        entry = "## OneForAllLauncher 2.6.6 — today\n- New\n"
        for text in ["", entry + entry]:
            with self.assertRaises(ValueError):
                release_notes(text, "2.6.6")


if __name__ == "__main__":
    unittest.main()

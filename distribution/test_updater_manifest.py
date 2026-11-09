import base64
from pathlib import Path
import tempfile
import unittest

from updater_manifest import TARGETS, manifest


class UpdaterManifestTests(unittest.TestCase):
    def test_uses_the_signed_installer_formats_and_both_macos_target_names(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            signature = base64.b64encode(b"fixture signature").decode()
            for suffix, _ in TARGETS.values():
                artifact = directory / f"OneForAllLauncher_2.6.10_{suffix}"
                artifact.write_bytes(b"test package")
                artifact.with_name(artifact.name + ".sig").write_text(signature)
            result = manifest(directory, "2.6.10", "liwwyy/OneForAllLauncher", "oneforall-2.6.10", "Current changes")
            self.assertEqual(result["version"], "2.6.10")
            self.assertEqual(len(result["platforms"]), 6)
            for target, (_, package_format) in TARGETS.items():
                entry = result["platforms"][target]
                self.assertEqual(entry["format"], package_format)
                self.assertEqual(entry["signature"], signature)
                self.assertIn("/liwwyy/OneForAllLauncher/releases/download/oneforall-2.6.10/", entry["url"])
            for arch in ["aarch64", "x86_64"]:
                self.assertEqual(result["platforms"][f"macos-{arch}"], result["platforms"][f"darwin-{arch}"])
                self.assertTrue(result["platforms"][f"macos-{arch}"]["url"].endswith(".app.tar.gz"))
            artifact.with_name(artifact.name + ".sig").unlink()
            with self.assertRaises(FileNotFoundError):
                manifest(directory, "2.6.10", "liwwyy/OneForAllLauncher", "oneforall-2.6.10", "")

    def test_missing_packages_prevent_publication(self):
        with tempfile.TemporaryDirectory() as temporary:
            with self.assertRaises(ValueError):
                manifest(Path(temporary), "2.6.10", "liwwyy/OneForAllLauncher", "oneforall-2.6.10", "")


if __name__ == "__main__":
    unittest.main()

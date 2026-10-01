import os
from pathlib import Path
import stat
import tempfile
import unittest
from unittest.mock import patch

from campaign_blob import blob_path, read_blob, store_blob, verify_blob
from campaign_workspace import ACTIVE_CAMPAIGN


class CampaignBlob(unittest.TestCase):
    def setUp(self):
        self.home_context = tempfile.TemporaryDirectory()
        self.home = Path(self.home_context.name)
        self.campaign = self.home / ACTIVE_CAMPAIGN
        self.campaign.mkdir()
        self.home_patch = patch("campaign_workspace.HPC_HOME", self.home)
        self.home_patch.start()

    def tearDown(self):
        self.home_patch.stop()
        self.home_context.cleanup()

    def test_round_trip_reference_is_path_free(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            source = root / "source"
            source.write_bytes(b"sealed evidence\n")
            reference = store_blob(self.campaign, source, "parent-seal")
            self.assertNotIn("path", reference)
            self.assertEqual(read_blob(self.campaign, reference), source.read_bytes())

    def test_identical_bytes_deduplicate_across_paths_and_modes(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            one, two = root / "one", root / "two"
            one.write_bytes(b"same")
            two.write_bytes(b"same")
            one.chmod(0o600)
            two.chmod(0o755)
            first = store_blob(self.campaign, one, "legacy-evidence")
            second = store_blob(self.campaign, two, "legacy-evidence")
            self.assertEqual(first, second)
            self.assertEqual(stat.S_IMODE(blob_path(self.campaign, first).stat().st_mode), 0o444)

    def test_missing_tampered_or_writable_object_is_rejected(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            source = root / "source"
            source.write_bytes(b"evidence")
            campaign = self.campaign
            reference = store_blob(campaign, source, "legacy-evidence")
            stored = blob_path(campaign, reference)
            stored.unlink()
            with self.assertRaises(ValueError):
                verify_blob(campaign, reference)
            reference = store_blob(campaign, source, "legacy-evidence")
            stored = blob_path(campaign, reference)
            stored.chmod(0o644)
            with self.assertRaises(ValueError):
                verify_blob(campaign, reference)

    def test_source_and_object_symlinks_are_not_followed(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            source = root / "source"
            source.write_bytes(b"evidence")
            source_link = root / "source-link"
            os.symlink(source, source_link)
            with self.assertRaises((OSError, ValueError)):
                store_blob(self.campaign, source_link, "legacy-evidence")
            campaign = self.campaign
            reference = store_blob(campaign, source, "legacy-evidence")
            stored = blob_path(campaign, reference)
            payload = stored.read_bytes()
            stored.unlink()
            outside = root / "outside"
            outside.write_bytes(payload)
            outside.chmod(0o644)
            os.symlink(outside, stored)
            with self.assertRaises(ValueError):
                verify_blob(campaign, reference)
            self.assertEqual(stat.S_IMODE(outside.stat().st_mode), 0o644)

    def test_invalid_reference_role_and_boolean_size_are_rejected(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            source = root / "source"
            source.write_bytes(b"evidence")
            campaign = self.campaign
            reference = store_blob(campaign, source, "legacy-evidence")
            for bad in (None, dict(reference, role="BAD ROLE"), dict(reference, bytes=True),
                        dict(reference, path="/public/home/majj/legacy-evidence")):
                with self.assertRaises(ValueError):
                    verify_blob(campaign, bad)
            with self.assertRaises(ValueError):
                store_blob(campaign, source, "BAD ROLE")

    def test_parent_symlink_escape_is_rejected(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            source = root / "source"
            source.write_bytes(b"evidence")
            campaign = self.campaign
            outside = root / "outside"
            outside.mkdir()
            os.symlink(outside, campaign / "objects")
            with self.assertRaises(ValueError):
                store_blob(campaign, source, "legacy-evidence")


if __name__ == "__main__":
    unittest.main()

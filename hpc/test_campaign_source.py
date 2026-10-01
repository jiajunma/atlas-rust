import hashlib
import io
import os
from pathlib import Path
import shutil
import stat
import tarfile
import tempfile
import unittest
from unittest.mock import patch

from campaign_source import (ARCHIVE_FILE_MODE, SCHEMA, digest, file_manifest,
                             materialize_source_archive, object_path,
                             store_source_archive, verify_source_archive)
from campaign_workspace import ACTIVE_CAMPAIGN


class CampaignSource(unittest.TestCase):
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

    def tree(self, root, executable=True):
        (root / "nested").mkdir()
        (root / "Cargo.toml").write_text("[workspace]\n")
        script = root / "nested/tool.sh"
        script.write_text("#!/bin/sh\nexit 0\n")
        script.chmod(0o755 if executable else 0o644)
        return file_manifest(root)

    def test_round_trip_is_path_free_and_uses_canonical_modes(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            source, campaign, output = root / "source", self.campaign, root / "output"
            source.mkdir()
            expected = self.tree(source)
            reference = store_source_archive(campaign, source, expected)
            self.assertNotIn("path", reference)
            materialize_source_archive(campaign, reference, expected, output)
            self.assertEqual(file_manifest(output), expected)
            self.assertEqual(stat.S_IMODE((output / "nested/tool.sh").stat().st_mode), ARCHIVE_FILE_MODE)

    def test_identical_content_across_paths_and_modes_reuses_one_object(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            first_source, second_source, campaign = root / "first", root / "second", self.campaign
            first_source.mkdir()
            second_source.mkdir()
            first_expected = self.tree(first_source, executable=True)
            second_expected = self.tree(second_source, executable=False)
            self.assertEqual(first_expected, second_expected)
            first = store_source_archive(campaign, first_source, first_expected)
            second = store_source_archive(campaign, second_source, second_expected)
            self.assertEqual(first, second)
            self.assertEqual(len(list((campaign / "objects/sha256").rglob(first["sha256"]))), 1)

    def test_changed_source_or_manifest_is_rejected(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            source = root / "source"
            source.mkdir()
            expected = self.tree(source)
            (source / "Cargo.toml").write_text("changed\n")
            with self.assertRaises(ValueError):
                store_source_archive(self.campaign, source, expected)
            expected = file_manifest(source)
            expected["Cargo.toml"] = "0" * 64
            with self.assertRaises(ValueError):
                store_source_archive(self.campaign, source, expected)

    def test_missing_or_tampered_object_is_rejected(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            source, campaign = root / "source", self.campaign
            source.mkdir()
            expected = self.tree(source)
            reference = store_source_archive(campaign, source, expected)
            stored = object_path(campaign, reference)
            stored.unlink()
            with self.assertRaises(ValueError):
                verify_source_archive(campaign, reference)
            reference = store_source_archive(campaign, source, expected)
            stored = object_path(campaign, reference)
            stored.chmod(0o644)
            stored.write_bytes(b"tampered")
            stored.chmod(0o444)
            with self.assertRaises(ValueError):
                verify_source_archive(campaign, reference)

    def test_existing_destination_and_source_symlink_are_rejected(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            source, campaign = root / "source", self.campaign
            source.mkdir()
            expected = self.tree(source)
            reference = store_source_archive(campaign, source, expected)
            destination = root / "destination"
            destination.mkdir()
            with self.assertRaises(ValueError):
                materialize_source_archive(campaign, reference, expected, destination)
            wrong_campaign = self.home / "wrong-campaign"
            wrong_campaign.mkdir()
            clean_parent = root / "clean"
            clean_parent.mkdir()
            before = list(clean_parent.iterdir())
            with self.assertRaises(ValueError):
                materialize_source_archive(
                    wrong_campaign, reference, expected, clean_parent / "output")
            self.assertEqual(list(clean_parent.iterdir()), before)
            links = root / "links"
            links.mkdir()
            os.symlink(source / "Cargo.toml", links / "Cargo.toml")
            with self.assertRaises(ValueError):
                file_manifest(links)

    def test_object_symlink_is_not_followed_or_chmodded(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            source, campaign = root / "source", self.campaign
            source.mkdir()
            expected = self.tree(source)
            reference = store_source_archive(campaign, source, expected)
            stored = object_path(campaign, reference)
            payload = stored.read_bytes()
            stored.unlink()
            outside = root / "outside"
            outside.write_bytes(payload)
            outside.chmod(0o644)
            os.symlink(outside, stored)
            with self.assertRaises(ValueError):
                verify_source_archive(campaign, reference)
            self.assertEqual(stat.S_IMODE(outside.stat().st_mode), 0o644)
            escaped_campaign = self.home / "escaped-campaign"
            escaped_campaign.mkdir()
            escaped = root / "escaped"
            escaped.mkdir()
            os.symlink(escaped, escaped_campaign / "objects")
            with self.assertRaises(ValueError):
                store_source_archive(escaped_campaign, source, expected)

    def test_reference_metadata_and_object_mode_are_verified(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            source, campaign = root / "source", self.campaign
            source.mkdir()
            expected = self.tree(source)
            reference = store_source_archive(campaign, source, expected)
            with self.assertRaises(ValueError):
                object_path(campaign, None)
            bad = dict(reference, files=True)
            with self.assertRaises(ValueError):
                object_path(campaign, bad)
            bad = dict(reference, source_bytes=reference["source_bytes"] + 1)
            with self.assertRaises(ValueError):
                verify_source_archive(campaign, bad)
            bad = dict(reference, path="/public/home/majj/legacy-source")
            with self.assertRaises(ValueError):
                verify_source_archive(campaign, bad)
            stored = object_path(campaign, reference)
            stored.chmod(0o644)
            with self.assertRaises(ValueError):
                verify_source_archive(campaign, reference)

    def test_dot_manifest_and_noncanonical_tar_metadata_are_rejected(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            source = root / "source"
            source.mkdir()
            with self.assertRaises(ValueError):
                store_source_archive(self.campaign, source, {".": hashlib.sha256(b"").hexdigest()})
            raw = root / "noncanonical.tar"
            with tarfile.open(raw, "w") as archive:
                info = tarfile.TarInfo("safe")
                info.size = 1
                info.mode = ARCHIVE_FILE_MODE | stat.S_ISUID
                archive.addfile(info, io.BytesIO(b"x"))
            reference = dict(schema=SCHEMA, role="rust-source", sha256=digest(raw),
                             bytes=raw.stat().st_size, files=1, source_bytes=1)
            stored = object_path(self.campaign, reference)
            stored.parent.mkdir(parents=True)
            shutil.copyfile(raw, stored)
            stored.chmod(0o444)
            with self.assertRaises(ValueError):
                verify_source_archive(self.campaign, reference)

    def test_root_symlink_and_special_file_are_rejected(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            source = root / "source"
            source.mkdir()
            self.tree(source)
            root_link = root / "root-link"
            os.symlink(source, root_link)
            with self.assertRaises(ValueError):
                file_manifest(root_link)
            fifo_root = root / "fifo-root"
            fifo_root.mkdir()
            os.mkfifo(fifo_root / "pipe")
            with self.assertRaises(ValueError):
                file_manifest(fifo_root)

    def test_unsafe_archive_is_rejected_and_partial_destination_removed(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            campaign = self.campaign
            raw = root / "unsafe.tar"
            with tarfile.open(raw, "w") as archive:
                info = tarfile.TarInfo("../escape")
                info.size = 1
                info.mode = ARCHIVE_FILE_MODE
                archive.addfile(info, io.BytesIO(b"x"))
            sha = digest(raw)
            reference = dict(schema=SCHEMA, role="rust-source", sha256=sha,
                             bytes=raw.stat().st_size, files=1, source_bytes=1)
            stored = object_path(campaign, reference)
            stored.parent.mkdir(parents=True)
            shutil.copyfile(raw, stored)
            stored.chmod(0o444)
            destination = root / "destination"
            expected = {"safe": hashlib.sha256(b"x").hexdigest()}
            with self.assertRaises(ValueError):
                materialize_source_archive(campaign, reference, expected, destination)
            self.assertFalse(destination.exists())
            self.assertFalse((root / "escape").exists())
            duplicate = root / "duplicate.tar"
            with tarfile.open(duplicate, "w") as archive:
                for _ in range(2):
                    info = tarfile.TarInfo("safe")
                    info.size = 1
                    info.mode = ARCHIVE_FILE_MODE
                    archive.addfile(info, io.BytesIO(b"x"))
            duplicate_ref = dict(schema=SCHEMA, role="rust-source", sha256=digest(duplicate),
                                 bytes=duplicate.stat().st_size, files=2, source_bytes=2)
            duplicate_stored = object_path(campaign, duplicate_ref)
            duplicate_stored.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(duplicate, duplicate_stored)
            duplicate_stored.chmod(0o444)
            with self.assertRaises(ValueError):
                verify_source_archive(campaign, duplicate_ref)


if __name__ == "__main__":
    unittest.main()

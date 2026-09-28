import unittest
import hashlib
import io
from pathlib import Path
import tarfile
import tempfile
from math_suite_review import archive_source_manifest, complete_section, independently_classify


class ArchiveSourceManifestTests(unittest.TestCase):
    def manifest(self, names, entry_type=tarfile.REGTYPE):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "source.tar"
            with tarfile.open(path, "w") as archive:
                for name in names:
                    member = tarfile.TarInfo(name)
                    member.type = entry_type
                    if member.isfile():
                        member.size = 6
                        archive.addfile(member, io.BytesIO(b"source"))
                    else:
                        member.linkname = "target"
                        archive.addfile(member)
            return archive_source_manifest(path)

    def test_dot_prefix_matches_extracted_relative_paths(self):
        expected = {"crates/core.rs": hashlib.sha256(b"source").hexdigest()}
        self.assertEqual(self.manifest(["./crates/core.rs"]), expected)
        self.assertEqual(self.manifest(["crates/core.rs"]), expected)

    def test_duplicate_canonical_files_are_rejected(self):
        for names in (["a", "a"], ["a", "./a"], ["./a", "a"]):
            with self.assertRaisesRegex(ValueError, "duplicate archive file"):
                self.manifest(names)

    def test_unsafe_and_empty_file_names_are_rejected(self):
        for name in ("/tmp/a", "../a", "safe/../a", ".", "./"):
            with self.assertRaises(ValueError):
                self.manifest([name])

    def test_links_are_rejected_not_silently_ignored(self):
        for entry_type in (tarfile.SYMTYPE, tarfile.LNKTYPE):
            with self.assertRaisesRegex(ValueError, "unsupported archive member"):
                self.manifest(["link"], entry_type)

    def test_directories_do_not_become_source_files(self):
        self.assertEqual(self.manifest([".", "./crates"], tarfile.DIRTYPE), {})


class IndependentReviewTests(unittest.TestCase):
    def setUp(self):
        self.case = {"id": "A2_root_data", "expected": "accept"}
        self.entries = {e: {"exit_status": 0, "timed_out": False} for e in ("oracle", "rust")}
        self.raw = b"MATH_BEGIN A2_root_data\n[1,2,3]\nMATH_END A2_root_data\n"
        self.streams = {e: (self.raw, b"") for e in ("oracle", "rust")}

    def test_exact_full_section(self):
        self.assertEqual(complete_section(self.raw, self.case["id"]), b"[1,2,3]\n")
        self.assertEqual(independently_classify(self.case, self.entries, self.streams), "MATH_MATCH")

    def test_internal_coefficient_change(self):
        self.streams["rust"] = (self.raw.replace(b"1,2,3", b"1,8,3"), b"")
        self.assertEqual(independently_classify(self.case, self.entries, self.streams), "MATH_MISMATCH")

    def test_equal_errors_not_math_pass(self):
        for engine in self.entries:
            self.entries[engine]["exit_status"] = 1
        self.assertEqual(independently_classify(self.case, self.entries, self.streams), "ORACLE_FAILURE")

    def test_partial_and_empty_sections(self):
        self.assertIsNone(complete_section(b"MATH_BEGIN A2_root_data\nMATH_END A2_root_data\n", self.case["id"]))
        self.assertIsNone(complete_section(self.raw.replace(b"MATH_END", b"unfinished"), self.case["id"]))

    def test_failed_load_despite_successful_markers(self):
        self.streams["rust"] = (self.raw, b"Syntax error")
        self.assertEqual(independently_classify(self.case, self.entries, self.streams), "RUST_FAILURE")

    def test_signal_kept_distinct_from_timeout(self):
        self.entries["rust"]["exit_status"] = 137
        self.assertEqual(independently_classify(self.case, self.entries, self.streams), "RUST_SIGNAL_OR_RESOURCE_FAILURE")


if __name__ == "__main__":
    unittest.main()

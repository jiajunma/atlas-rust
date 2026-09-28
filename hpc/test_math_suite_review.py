import unittest
from math_suite_review import complete_section, independently_classify


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

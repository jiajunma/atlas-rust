import unittest
from pathlib import Path
from math_suite import cases, compare, observation_ok, payload


class MathSuiteTests(unittest.TestCase):
    def setUp(self):
        self.case = {"id": "G2_hodge", "expected": "accept"}
        self.good = {"exit_status": 0, "timed_out": False}
        self.stdout = b"preamble\nMATH_BEGIN G2_hodge\npolynomial [1,2,3]\nMATH_END G2_hodge\nBye.\n"

    def test_catalog_cross_product(self):
        catalog, matrix = cases(Path(__file__).resolve().parents[1])
        self.assertEqual(len(matrix), 72)
        self.assertEqual({x["family"] for x in matrix}, {"classical", "exceptional"})
        self.assertTrue(catalog["open_requirements"])
        for case in matrix:
            self.assertNotIn("@TYPE@", case["source"])
            self.assertNotIn("@RANK@", case["source"])
            self.assertNotIn("@ID@", case["source"])
            self.assertIn(case["id"], case["source"])

    def test_full_polynomial_not_just_count(self):
        changed = self.stdout.replace(b"[1,2,3]", b"[1,9,3]")
        result = compare(self.case, {"oracle": self.good, "rust": self.good},
                         {"oracle": (self.stdout, b""), "rust": (changed, b"")})
        self.assertEqual(result["status"], "MATH_MISMATCH")

    def test_equal_failure_never_passes(self):
        bad = dict(self.good, exit_status=1)
        result = compare(self.case, {"oracle": bad, "rust": bad},
                         {"oracle": (self.stdout, b"error"), "rust": (self.stdout, b"error")})
        self.assertEqual(result["status"], "ORACLE_FAILURE")

    def test_timeout_even_with_final_marker_fails(self):
        self.assertFalse(observation_ok(self.case, dict(self.good, timed_out=True), self.stdout, b""))

    def test_missing_or_duplicate_end_fails(self):
        self.assertIsNone(payload(self.stdout.replace(b"MATH_END", b"INCOMPLETE"), self.case["id"]))
        self.assertIsNone(payload(self.stdout + b"MATH_END G2_hodge\n", self.case["id"]))

    def test_loading_error_not_hidden_by_markers(self):
        self.assertFalse(observation_ok(self.case, self.good, self.stdout, b"Syntax error"))

    def test_rejected_category_and_exit_required(self):
        case = dict(self.case, expected="reject", diagnostic="Rank and rational weight size mismatch")
        bad = dict(self.good, exit_status=1)
        message = case["diagnostic"].encode()
        self.assertTrue(observation_ok(case, bad, self.stdout, message))
        self.assertFalse(observation_ok(case, self.good, self.stdout, message))
        self.assertFalse(observation_ok(case, bad, self.stdout, b"unrelated failure"))

    def test_preamble_difference_is_visible(self):
        result = compare(self.case, {"oracle": self.good, "rust": self.good},
                         {"oracle": (self.stdout, b""),
                          "rust": (self.stdout.replace(b"preamble", b"different"), b"")})
        self.assertEqual(result["status"], "MATH_MATCH")
        self.assertFalse(result["full_stdout_equal"])

    def test_rejection_does_not_hide_load_errors_or_abort(self):
        case = dict(self.case, expected="reject", diagnostic="Rank and rational weight size mismatch 2:3")
        message = case["diagnostic"].encode()
        self.assertFalse(observation_ok(case, dict(self.good, exit_status=1),
                                        self.stdout, b"Syntax error\n" + message))
        self.assertFalse(observation_ok(case, dict(self.good, exit_status=139),
                                        self.stdout, message))
        self.assertFalse(observation_ok(case, dict(self.good, exit_status=1),
                                        self.stdout, message + b"\n" + message))


if __name__ == "__main__":
    unittest.main()

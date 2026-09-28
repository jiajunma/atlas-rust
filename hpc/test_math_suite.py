import unittest
from pathlib import Path
from math_suite import cases, compare, failure_status, observation_ok, payload


class MathSuiteTests(unittest.TestCase):
    def setUp(self):
        self.case = {"id": "G2_hodge", "expected": "accept"}
        self.good = {"exit_status": 0, "timed_out": False}
        self.stdout = b"preamble\nMATH_BEGIN G2_hodge\npolynomial [1,2,3]\nMATH_END G2_hodge\nBye.\n"

    def test_catalog_cross_product(self):
        catalog, matrix = cases(Path(__file__).resolve().parents[1])
        self.assertEqual(len(matrix), 92)
        self.assertEqual(sum(x["family"] != "language" for x in matrix), 89)
        self.assertEqual({x["family"] for x in matrix}, {"classical", "exceptional", "language"})
        self.assertTrue(catalog["open_requirements"])
        for case in matrix:
            self.assertNotIn("@TYPE@", case["source"])
            self.assertNotIn("@RANK@", case["source"])
            self.assertNotIn("@ID@", case["source"])
            self.assertNotRegex(case["source"], r"@[A-Z_]+@")
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

    def test_signal_is_not_automatically_timeout(self):
        self.assertEqual(failure_status("RUST", dict(self.good, exit_status=137)),
                         "RUST_SIGNAL_OR_RESOURCE_FAILURE")
        self.assertEqual(failure_status("ORACLE", dict(self.good, exit_status=124)),
                         "ORACLE_TIMEOUT")

    def test_language_cases_do_not_move_group_indices(self):
        matrix = cases(Path(__file__).resolve().parents[1])[1]
        self.assertEqual(matrix[0]["id"], "A2_root_data")
        self.assertEqual(matrix[36]["id"], "G2_root_data")
        self.assertEqual(matrix[71]["id"], "E7_fpp_rank_rejected")
        self.assertEqual([x["id"] for x in matrix[72:75]],
                         ["Language_generic_pair", "Language_generic_identity", "Language_basic_load"])

    def test_core_klv_is_additive_and_same_parameters(self):
        matrix = cases(Path(__file__).resolve().parents[1])[1]
        self.assertEqual(matrix[75]["id"], "A2_core_klv")
        self.assertEqual(matrix[82]["id"], "E7_core_klv")
        self.assertTrue(matrix[2]["source"].startswith("<deform.at"))
        for case in matrix[75:83]:
            self.assertNotIn("<deform.at", case["source"])
            self.assertIn("two_rho(rd)/2", case["source"])
            self.assertIn("for q in [p,p*(1/2)]", case["source"])

    def test_hodge_probes_preserve_failed_bounds(self):
        matrix = cases(Path(__file__).resolve().parents[1])[1]
        self.assertIn("hodge_branch_std(p,4)", matrix[4]["source"])
        self.assertEqual([case["parameters"]["BOUND"] for case in matrix[83:89]],
                         [4, 20, 4, 20, 15, 50])
        self.assertEqual(matrix[88]["id"], "G2_hodge_complex_trace")
        for case in matrix[83:89]:
            self.assertIn('prints("STD_BACK_TRACE",back_trace)', case["source"])
            self.assertIn('prints("IRR_BACK_TRACE",back_trace)', case["source"])

    def test_d4_wall_regression_preserves_full_actions(self):
        matrix = cases(Path(__file__).resolve().parents[1])[1]
        case = matrix[89]
        self.assertEqual(case["id"], "D4_fpp_wall_regression")
        self.assertEqual(case["expected"], "accept")
        self.assertIn("[1,0,0,0]/100", case["source"])
        self.assertIn("root_permutation(w)", case["source"])
        self.assertIn("w=W_elt(rd,expected[i])", case["source"])
        self.assertIn("action[npos+j]<npos", case["source"])

    def test_partial_kl_regressions_keep_complete_polynomials(self):
        matrix = cases(Path(__file__).resolve().parents[1])[1]
        self.assertEqual([case["id"] for case in matrix[90:]],
                         ["A2_partial_kl_regression", "G2_partial_kl_regression"])
        self.assertEqual([case["parameters"]["FULL_SIZE"] for case in matrix[90:]], [4, 10])
        for case in matrix[90:]:
            self.assertIn('"POLYNOMIALS",polynomials', case["source"])
            self.assertIn("partial KL block lost Bruhat predecessors", case["source"])


if __name__ == "__main__":
    unittest.main()

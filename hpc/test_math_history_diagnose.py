import unittest

from math_kl_history_diagnose import history_records, section_difference


class HistoryDiagnosisTests(unittest.TestCase):
    def test_preserves_every_history_section(self):
        chunk = "PARAMETERp\nCOLDcEXPECTED_PARAMETERSe\nFULLfWARMw\n"
        raw = "preamble\nMATH_BEGIN id\n" + chunk * 3 + "MATH_END id\n"
        self.assertEqual(history_records(raw, "id"),
                         [dict(parameter="p", cold="c", expected="e", full="f", warm="w\n")] * 3)
        with self.assertRaises(ValueError):
            history_records(raw + "MATH_END id\n", "id")
        with self.assertRaises(ValueError):
            history_records(raw.replace(chunk, "", 1), "id")

    def test_reports_matrix_position_without_normalizing_result(self):
        left = "params\n| 1, 0 |\n| 0, 1 |\npool"
        right = left.replace("| 1, 0 |", "| 1, 2 |")
        result = section_difference(left, right)
        self.assertFalse(result["equal"])
        self.assertTrue(result["parameter_prefix_equal"])
        self.assertTrue(result["polynomial_suffix_equal"])
        self.assertEqual(result["first_matrix_differences"],
                         [dict(row=0, column=1, oracle=0, rust=2)])

    def test_equal_prefix_does_not_hide_missing_suffix(self):
        result = section_difference("abc", "abcx")
        self.assertFalse(result["equal"])
        self.assertEqual(result["first_differing_character"], 3)

    def test_locates_changed_polynomial_and_its_matrix_references(self):
        left = "params\n| 1, 2 |\n| 0, 1 |\n,[[],[1],[0,2,3]])"
        right = left.replace("[0,2,3]", "[0,4,6]")
        result = section_difference(left, right)
        self.assertEqual(result["polynomial_pool_sizes"], [3, 3])
        self.assertEqual(result["changed_polynomials"],
                         [dict(index=2, oracle=[0, 2, 3], rust=[0, 4, 6],
                               oracle_matrix_references=[[0, 1]])])


if __name__ == "__main__":
    unittest.main()

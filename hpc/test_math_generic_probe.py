from pathlib import Path
import unittest

from math_generic_probe import load_cases, observed_category


class GenericContractCaptureTests(unittest.TestCase):
    def setUp(self):
        self.case = {"id": "generic_probe"}
        self.success = {"exit_status": 0, "timed_out": False}
        self.output = b"MATH_BEGIN generic_probe\nvalue\nMATH_END generic_probe\n"

    def test_catalog_has_positive_and_rejected_cases(self):
        cases = load_cases(Path(__file__).resolve().parents[1])
        self.assertEqual(len(cases), 11)
        self.assertEqual(sum(c["intent"] == "accept" for c in cases), 5)
        self.assertEqual(len({c["id"] for c in cases}), 11)
        self.assertTrue(all(c["source"].endswith("quit\n") for c in cases))

    def test_acceptance_requires_markers_and_empty_diagnostics(self):
        self.assertEqual(observed_category(self.case, self.success, (self.output, b"")), "ACCEPTED")
        self.assertEqual(observed_category(self.case, self.success, (self.output, b"Type error")), "OTHER_FAILURE")
        self.assertEqual(observed_category(self.case, self.success, (b"value", b"")), "OTHER_FAILURE")

    def test_rejection_kinds_stay_distinct(self):
        failed = dict(self.success, exit_status=1)
        self.assertEqual(observed_category(self.case, failed, (self.output, b"Syntax error")), "REJECTED_SYNTAX")
        self.assertEqual(observed_category(self.case, failed, (self.output, b"Type error")), "REJECTED_TYPE")
        self.assertEqual(observed_category(self.case, failed, (self.output, b"unclassified")), "OTHER_FAILURE")

    def test_timeouts_and_signals_never_become_rejections(self):
        for code, expected in ((124, "TIMEOUT"), (137, "RESOURCE_OR_SIGNAL_FAILURE")):
            self.assertEqual(observed_category(self.case, dict(self.success, exit_status=code),
                                              (self.output, b"Type error")), expected)


if __name__ == "__main__":
    unittest.main()

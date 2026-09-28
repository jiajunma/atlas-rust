import unittest

from math_language_bridge import (candidate_unit_gate_passed,
                                  foundation_completion_status, oracle_execution_valid)


class BridgeExecutionTests(unittest.TestCase):
    def cases(self, *categories):
        return [{"categories": {"oracle": c}} for c in categories]

    def test_accepted_and_intended_diagnostic_runs_are_valid(self):
        self.assertTrue(oracle_execution_valid(self.cases("ACCEPTED", "REJECTED_TYPE", "REJECTED_SYNTAX")))

    def test_loader_failure_timeout_and_signal_invalidate_capture(self):
        for category in ("OTHER_FAILURE", "TIMEOUT", "RESOURCE_OR_SIGNAL_FAILURE"):
            self.assertFalse(oracle_execution_valid(self.cases("ACCEPTED", category)))

    def test_empty_capture_never_passes(self):
        self.assertFalse(oracle_execution_valid([]))

    def test_build_status_retains_unavailable_oracle_without_blessing_capture(self):
        good = {"status": "CAPTURED_NOT_LANGUAGE_ACCEPTANCE", "cases": self.cases("ACCEPTED")}
        failed = {"status": "CAPTURE_FAILED_ORACLE_EXECUTION",
                  "cases": self.cases("ACCEPTED", "RESOURCE_OR_SIGNAL_FAILURE")}
        self.assertEqual(foundation_completion_status(good), "FOUNDATION_UNITS_PASS_LANGUAGE_NOT_PORTED")
        self.assertEqual(foundation_completion_status(failed), "FOUNDATION_UNITS_PASS_ORACLE_UNAVAILABLE")
        self.assertFalse(oracle_execution_valid(failed["cases"]))
        for report in ({}, dict(good, cases=[]), dict(good, cases=failed["cases"]),
                       dict(failed, cases=good["cases"]), dict(failed, cases=[])):
            with self.assertRaises(ValueError):
                foundation_completion_status(report)

    def test_unavailable_oracle_build_needs_completed_integrity_checks(self):
        status = {"status": "FOUNDATION_UNITS_PASS_ORACLE_UNAVAILABLE"}
        self.assertFalse(candidate_unit_gate_passed(status))
        self.assertFalse(candidate_unit_gate_passed(dict(status, source_integrity_rechecked=False)))
        self.assertTrue(candidate_unit_gate_passed(dict(status, source_integrity_rechecked=True)))
        self.assertTrue(candidate_unit_gate_passed({"status": "FOUNDATION_UNITS_PASS_LANGUAGE_NOT_PORTED"}))
        for value in ("FAIL", "SUBMITTED", "CAPTURE_FAILED_ORACLE_EXECUTION"):
            self.assertFalse(candidate_unit_gate_passed({"status": value, "source_integrity_rechecked": True}))


if __name__ == "__main__":
    unittest.main()

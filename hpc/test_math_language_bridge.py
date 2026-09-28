import unittest

from math_language_bridge import oracle_execution_valid


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


if __name__ == "__main__":
    unittest.main()

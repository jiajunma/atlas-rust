#!/usr/bin/env python3
"""Focused unit checks for scalar oracle expectation validation."""

import pathlib
import sys
import unittest

sys.path.insert(0, str(pathlib.Path(__file__).parent))
from scalar_reference import validate_expectation


class ScalarReferenceValidationTest(unittest.TestCase):
    def test_diagnostic_content_does_not_impose_exit_policy(self) -> None:
        checks, passed = validate_expectation(
            {
                "value_lines": [],
                "diagnostics": ["fraction with zero denominator"],
            },
            b"",
            b"Runtime error:\n  fraction with zero denominator\n",
            timed_out=False,
        )

        self.assertTrue(checks["expected_diagnostics_exact"])
        self.assertTrue(passed)

    def test_extra_semantic_events_are_rejected(self) -> None:
        checks, passed = validate_expectation(
            {"value_lines": ["Value: 1"], "diagnostics": []},
            b"Value: 1\nValue: 999\n",
            b"Runtime error:\n  unexpected failure\nEvaluation aborted.\n",
            timed_out=False,
        )

        self.assertFalse(checks["expected_value_lines_exact"])
        self.assertFalse(checks["no_unexpected_runtime_error"])
        self.assertFalse(passed)

    def test_missing_expectation_cannot_pass_vacuously(self) -> None:
        checks, passed = validate_expectation(
            {
                "load_error": "missing expectation file",
                "value_lines": [],
                "diagnostics": [],
            },
            b"Value: anything\n",
            b"",
            timed_out=False,
        )

        self.assertFalse(checks["expectation_loaded"])
        self.assertFalse(passed)


if __name__ == "__main__":
    unittest.main()

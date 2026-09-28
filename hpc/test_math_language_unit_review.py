import unittest

from math_language_unit_review import (KNOWN_FAILURES, check_regression,
                                      check_summary, inventory, regression_contracts)


class CoreReviewTests(unittest.TestCase):
    def test_inventory_requires_both_regressions_and_other_tests(self):
        text = "\n".join(n + ": test" for n in [*KNOWN_FAILURES, "types::control"])
        self.assertEqual(len(inventory(text)), 3)
        for bad in ["types::control: test", text + "\ntypes::control: test",
                    "\n".join(n + ": test" for n in KNOWN_FAILURES)]:
            with self.assertRaises(ValueError):
                inventory(bad)

    def test_summary_requires_exact_inventory_and_no_ignored_tests(self):
        good = "test result: ok. 399 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished"
        check_summary(good, 399, 0, 2)
        for bad in [good.replace("399 passed", "398 passed"),
                    good.replace("0 ignored", "1 ignored"),
                    good.replace("2 filtered", "3 filtered"),
                    good.replace("result: ok", "result: FAILED"), good + "\n" + good]:
            with self.assertRaises(ValueError):
                check_summary(bad, 399, 0, 2)

    def test_expected_failure_must_execute_one_failure(self):
        good = "test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 400 filtered out; finished"
        check_summary(good, 0, 1, 400)
        with self.assertRaises(ValueError):
            check_summary(good.replace("1 failed", "0 failed"), 0, 1, 400)

    def test_repair_gate_requires_explicit_boolean_and_preserves_both_tests(self):
        for repaired in (False, True):
            contracts = regression_contracts(repaired)
            self.assertEqual(set(contracts), set(KNOWN_FAILURES))
            for contract in contracts.values():
                self.assertEqual(contract["exit_status"], 0 if repaired else 101)
                self.assertIn("=true" if repaired else "=false", contract["marker"])
        for bad in (None, 0, 1, "true", [], {}):
            with self.assertRaises(ValueError):
                regression_contracts(bad)

    def test_repaired_assertion_cannot_be_skipped_failed_or_report_old_marker(self):
        for name, contract in regression_contracts(True).items():
            good = (f"test {name} ... ok\n{contract['marker']}\n"
                    "test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 400 filtered out; finished")
            check_regression(good, name, True, 401)
            for bad in (good.replace("=true", "=false"), good.replace(name, "different::test"),
                        good.replace("1 passed", "0 passed"), good.replace("0 ignored", "1 ignored")):
                with self.assertRaises(ValueError):
                    check_regression(bad, name, True, 401)
            with self.assertRaises(ValueError):
                check_regression(good, name, False, 401)


if __name__ == "__main__":
    unittest.main()

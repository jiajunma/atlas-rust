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
        self.assertEqual(len(cases), 100)
        self.assertEqual(sum(c["intent"] == "accept" for c in cases), 52)
        self.assertEqual(len({c["id"] for c in cases}), 100)
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
        # Exact indentation/envelope retained from original3835190/3835224.
        name_error = (b"Error in expression ordinary_first at <standard input>:2:60-74\n"
                      b"  Undefined identifier 'ordinary_first'\n"
                      b"Error in 'set' command at <standard input>:2:0-78:\n"
                      b"Expression analysis failed\n"
                      b"  Command 'set (ordinary_first,ordinary_second)' not executed, nothing defined.\n")
        self.assertEqual(observed_category(self.case, failed, (self.output, name_error)), "REJECTED_NAME")
        self.assertEqual(observed_category(self.case, failed, (name_error, b"unclassified")), "OTHER_FAILURE")
        self.assertEqual(observed_category(self.case, self.success, (self.output, name_error)), "OTHER_FAILURE")
        self.assertEqual(observed_category(self.case, failed, (self.output, b"Undefined identifier 'missing'")), "OTHER_FAILURE")
        # Current original3836409 reports capture ambiguity without a Type or
        # Program heading inside a rejected set command. Require its envelope.
        ambiguity = (b"Error in expression captured_first at <standard input>:4:35-49\n"
                     b"  Ambiguous overloaded symbol 'captured_first': its context type (Pair<int>->int) matches\n"
                     b"  both (Pair<A>->A) and (Pair<int>->int) in overload table\n"
                     b"Error in 'set' command at <standard input>:4:0-50:\n"
                     b"Expression analysis failed\n"
                     b"  Command 'set f' not executed, nothing defined.\n")
        self.assertEqual(observed_category(self.case, failed, (self.output, ambiguity)),
                         "REJECTED_OVERLOAD_AMBIGUITY")
        for record, streams in ((self.success, (self.output, ambiguity)),
                                (failed, (ambiguity, b"unclassified")),
                                (failed, (self.output, ambiguity.split(b"\n", 1)[1])),
                                (failed, (self.output, b"  Ambiguous overloaded symbol 'x'"))):
            self.assertEqual(observed_category(self.case, record, streams), "OTHER_FAILURE")

    def test_timeouts_and_signals_never_become_rejections(self):
        for code, expected in ((124, "TIMEOUT"), (137, "RESOURCE_OR_SIGNAL_FAILURE")):
            self.assertEqual(observed_category(self.case, dict(self.success, exit_status=code),
                                              (self.output, b"Type error")), expected)

    def test_constructor_arity_is_an_explicit_distinct_rejection(self):
        failed = dict(self.success, exit_status=1)
        diagnostic = b"Type constructor 'MathArityPair' called with 1 type arguments, expected 2"
        self.assertEqual(observed_category(self.case, failed, (self.output, diagnostic)),
                         "REJECTED_TYPE_ARITY")
        self.assertEqual(observed_category(self.case, failed, (diagnostic, b"unclassified")),
                         "OTHER_FAILURE")

    def test_printed_error_words_cannot_classify_a_rejection(self):
        self.assertEqual(observed_category(self.case, dict(self.success, exit_status=1),
                                          (b"Type error", b"unclassified")), "OTHER_FAILURE")


if __name__ == "__main__":
    unittest.main()

import ast
import builtins
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import unittest
from unittest.mock import patch

import weyl_context_core_contract as contract


MISMATCH = "weyl_group_mismatch"
HIGH_WORD = "illegal_weyl_word_entry"
NEGATIVE_WORD = "negative_integer_where_unsigned_required"
HIGH_WORD_G2 = "illegal_weyl_word_entry_2"

MESSAGES = {
    MISMATCH: "Weyl group mismatch",
    HIGH_WORD: "Illegal Weyl word entry 1 (should be <1)",
    NEGATIVE_WORD: "Negative integer where unsigned is required",
    HIGH_WORD_G2: "Illegal Weyl word entry 2 (should be <2)",
}

CATALOG_SHA256 = "7fb77fecda841962cb98bdeddbdba58df46d673e076f87f2087cd234e2f1a717"
G2_CATALOG_SHA256 = "179d47814aa7526753f2a6dcdff9b23def9a09cbcabe5f96ec9d24dd2bef3bbd"

EXPECTED = {
    "weyl_context_core_cold_dual": {
        "oracle": {
            "exit_status": 0,
            "payload_lines": [
                "Variable wc_rd: RootDatum",
                "Variable wc_saved: WeylElt",
                "WC_SAME|[0]|1|[1,0]|[]",
                "Variable wc_alias: RootDatum",
                "Variable wc_alias_w: WeylElt",
                "WC_ALIAS|true|true|true|[]",
                "Variable wc_equal: RootDatum",
                "Variable wc_equal_w: WeylElt",
                "WC_EQUAL|true|true|true|[]",
                "Variable wc_dual: RootDatum",
                "Variable wc_dual_w: WeylElt",
                "WC_DUAL_COLD_OWNER|true",
                "WC_DUAL_COLD_EQ|true",
                "WC_DUAL_COLD_NEQ|false",
                "WC_DUAL_COLD_MUL|[]",
                "Variable wc_reverse_target: RootDatum",
                "Variable wc_reverse_target_w: WeylElt",
                "Variable wc_reverse_source: RootDatum",
                "Variable wc_reverse_source_w: WeylElt",
                "WC_DUAL_REVERSE_OWNER|true",
                "WC_DUAL_REVERSE_EQ|true",
                "WC_DUAL_REVERSE_NEQ|false",
                "WC_DUAL_REVERSE_MUL|[]|true",
                (
                    "Variable wc_rd: RootDatum (overriding previous instance, "
                    "which had type RootDatum)"
                ),
                "Variable wc_rebound_w: WeylElt",
                "WC_REBOUND|false|[1,0]|true|false|[0]|[1,0]",
                "WC_RECOVERY|727",
            ],
            "causes": [],
        },
        "rust": {
            "exit_status": 1,
            "payload_lines": [
                "Variable wc_rd: RootDatum",
                "Variable wc_saved: WeylElt",
                "WC_SAME|[0]|1|[1,0]|[]",
                "Variable wc_alias: RootDatum",
                "Variable wc_alias_w: WeylElt",
                "WC_ALIAS|true|true|true|[]",
                "Variable wc_equal: RootDatum",
                "Variable wc_equal_w: WeylElt",
                "WC_EQUAL|true|true|true|[]",
                "Variable wc_dual: RootDatum",
                "Variable wc_dual_w: WeylElt",
                "WC_DUAL_COLD_OWNER|true",
                "WC_DUAL_COLD_EQ|false",
                "WC_DUAL_COLD_NEQ|true",
                "Variable wc_reverse_target: RootDatum",
                "Variable wc_reverse_target_w: WeylElt",
                "Variable wc_reverse_source: RootDatum",
                "Variable wc_reverse_source_w: WeylElt",
                "WC_DUAL_REVERSE_OWNER|true",
                "WC_DUAL_REVERSE_EQ|false",
                "WC_DUAL_REVERSE_NEQ|true",
                (
                    "Variable wc_rd: RootDatum (overriding previous instance, "
                    "which had type RootDatum)"
                ),
                "Variable wc_rebound_w: WeylElt",
                "WC_REBOUND|false|[1,0]|true|false|[0]|[1,0]",
                "WC_RECOVERY|727",
            ],
            "causes": [MISMATCH, MISMATCH],
        },
    },
    "weyl_context_core_prewarmed_dual": {
        "oracle": {
            "exit_status": 1,
            "payload_lines": [
                "Variable wcn_true: RootDatum",
                "Variable wcn_false: RootDatum",
                "Variable wcn_false_w: WeylElt",
                "Variable wcn_target: RootDatum",
                "Variable wcn_target_w: WeylElt",
                "Variable wcn_dual: RootDatum",
                "Variable wcn_saved: WeylElt",
                "Variable wcn_dual_w: WeylElt",
                "WCN_AFTER_OWNER_EQ|[0]|[1,0]",
                "WCN_AFTER_OWNER_NEQ|[0]|[1,0]",
                "WCN_AFTER_OWNER_MUL|[0]|[1,0]",
                "WCN_AFTER_HIGH|[0]",
                "WCN_AFTER_NEGATIVE|[0]",
                "WCN_AFTER_DUAL_EQ|[0]|true",
                "WCN_AFTER_DUAL_NEQ|[0]|true",
                "WCN_AFTER_DUAL_MUL|[0]|[0]|true",
                "WCN_RECOVERY|733",
            ],
            "causes": [
                MISMATCH,
                MISMATCH,
                MISMATCH,
                HIGH_WORD,
                NEGATIVE_WORD,
                MISMATCH,
                MISMATCH,
                MISMATCH,
            ],
        },
        "rust": {
            "exit_status": 1,
            "payload_lines": [
                "Variable wcn_true: RootDatum",
                "Variable wcn_false: RootDatum",
                "Variable wcn_false_w: WeylElt",
                "Variable wcn_target: RootDatum",
                "Variable wcn_target_w: WeylElt",
                "Variable wcn_dual: RootDatum",
                "Variable wcn_saved: WeylElt",
                "Variable wcn_dual_w: WeylElt",
                "WCN_OWNER_EQ|false",
                "WCN_AFTER_OWNER_EQ|[0]|[1,0]",
                "WCN_OWNER_NEQ|true",
                "WCN_AFTER_OWNER_NEQ|[0]|[1,0]",
                "WCN_AFTER_OWNER_MUL|[0]|[1,0]",
                "WCN_AFTER_HIGH|[0]",
                "WCN_AFTER_NEGATIVE|[0]",
                "WCN_DUAL_PREWARM_EQ|false",
                "WCN_AFTER_DUAL_EQ|[0]|true",
                "WCN_DUAL_PREWARM_NEQ|true",
                "WCN_AFTER_DUAL_NEQ|[0]|true",
                "WCN_AFTER_DUAL_MUL|[0]|[0]|true",
                "WCN_RECOVERY|733",
            ],
            "causes": [
                MISMATCH,
                HIGH_WORD,
                NEGATIVE_WORD,
                MISMATCH,
            ],
        },
    },
}

for _case in EXPECTED.values():
    for _engine in _case.values():
        _engine["marker_lines"] = [
            line for line in _engine["payload_lines"]
            if line.startswith(("WC_", "WCN_"))
        ]


_G2_COLD_PAYLOAD = [
    "Variable wg_rd: RootDatum",
    "Variable wg_s0: WeylElt",
    "Variable wg_s1: WeylElt",
    "WG_SAME|[0]|1|1|[0,1]",
    "WG_NONCOMMUTE|[0,1]|[1,0]",
    "WG_BRAID|true|6",
    "Variable wg_alias: RootDatum",
    "Variable wg_alias_w: WeylElt",
    "WG_ALIAS|true|true",
    "Variable wg_equal: RootDatum",
    "Variable wg_equal_w: WeylElt",
    "WG_EQUAL|true|true",
    "Variable wg_dual: RootDatum",
    "Variable wg_dual_w: WeylElt",
    "WG_DUAL_OWNER|true",
    "WG_DUAL_EQ|true",
    "WG_DUAL_NEQ|false",
    "WG_DUAL_MUL|[]",
    "Variable wg_reverse_target: RootDatum",
    "Variable wg_reverse_target_w: WeylElt",
    "Variable wg_reverse_source: RootDatum",
    "Variable wg_reverse_source_w: WeylElt",
    "WG_REVERSE_OWNER|true",
    "WG_REVERSE_EQ|true",
    "WG_REVERSE_NEQ|false",
    "WG_REVERSE_MUL|[]|true",
    (
        "Variable wg_rd: RootDatum (overriding previous instance, which had "
        "type RootDatum)"
    ),
    "Variable wg_rebound_w: WeylElt",
    "WG_REBOUND|false|true|false|[0]|[1]",
    "WG_RECOVERY|727",
]

_G2_PREWARM_PAYLOAD = [
    "Variable wgn_true: RootDatum",
    "Variable wgn_false: RootDatum",
    "Variable wgn_false_w: WeylElt",
    "Variable wgn_target: RootDatum",
    "Variable wgn_target_w: WeylElt",
    "Variable wgn_dual: RootDatum",
    "Variable wgn_saved: WeylElt",
    "Variable wgn_dual_w: WeylElt",
    "WGN_AFTER_OWNER_EQ|[0]|1",
    "WGN_AFTER_OWNER_NEQ|[0]|1",
    "WGN_AFTER_OWNER_MUL|[0]|1",
    "WGN_AFTER_HIGH|[0]",
    "WGN_AFTER_NEGATIVE|[0]",
    "WGN_AFTER_DUAL_EQ|[1]|true",
    "WGN_AFTER_DUAL_NEQ|[1]|true",
    "WGN_AFTER_DUAL_MUL|[0]|[1]|true",
    "WGN_RECOVERY|733",
]

_G2_PREWARM_CAUSES = [
    MISMATCH, MISMATCH, MISMATCH, HIGH_WORD_G2, NEGATIVE_WORD,
    MISMATCH, MISMATCH, MISMATCH,
]

G2_EXPECTED = {
    "weyl_context_g2_cold_dual": {
        "oracle": {
            "exit_status": 0,
            "payload_lines": _G2_COLD_PAYLOAD,
            "causes": [],
        },
        "rust": {
            "exit_status": 0,
            "payload_lines": _G2_COLD_PAYLOAD,
            "causes": [],
        },
    },
    "weyl_context_g2_prewarmed_dual": {
        "oracle": {
            "exit_status": 1,
            "payload_lines": _G2_PREWARM_PAYLOAD,
            "causes": _G2_PREWARM_CAUSES,
        },
        "rust": {
            "exit_status": 1,
            "payload_lines": _G2_PREWARM_PAYLOAD,
            "causes": _G2_PREWARM_CAUSES,
        },
    },
}

for _case in G2_EXPECTED.values():
    for _engine in _case.values():
        _engine["marker_lines"] = [
            line for line in _engine["payload_lines"]
            if line.startswith(("WG_", "WGN_"))
        ]

ALL_EXPECTED = {**EXPECTED, **G2_EXPECTED}


def expected_catalog():
    return {
        "schema": contract.CATALOG_SCHEMA,
        "evidence_maturity": contract.CATALOG_MATURITY,
        "scope": contract.CATALOG_SCOPE,
        "cases": [dict(case) for case in contract.EXPECTED_CASES],
    }


def g2_expected_catalog():
    return {
        "schema": contract.G2_CATALOG_SCHEMA,
        "evidence_maturity": contract.CATALOG_MATURITY,
        "scope": contract.G2_CATALOG_SCOPE,
        "cases": [dict(case) for case in contract.G2_EXPECTED_CASES],
    }


def case(case_id="weyl_context_core_cold_dual"):
    return deepcopy(next(
        row
        for row in (*contract.EXPECTED_CASES, *contract.G2_EXPECTED_CASES)
        if row["id"] == case_id
    ))


def observation(engine, code):
    return {
        "engine": engine,
        "exit_status": code,
        "timed_out": False,
        "termination_uncertain": False,
        "signal": None,
        "seconds": 1.25,
        "user_cpu_seconds": 1.0,
        "system_cpu_seconds": 0.25,
        "maxrss_kb": 4096,
        "maxrss_approximate": False,
    }


def stdout(case_id, payload_lines):
    body = ["MATH_BEGIN " + case_id]
    body.extend(payload_lines)
    body.extend(["MATH_END " + case_id, "Bye."])
    return ("\n".join(body) + "\n").encode()


def stderr(engine, causes):
    if engine == "oracle":
        return "".join(
            "Runtime error:\n  " + MESSAGES[cause] + "\nEvaluation aborted.\n"
            for cause in causes
        ).encode()
    return "".join(
        f"Runtime error at <stdin>:{index}:1: {MESSAGES[cause]}\n"
        f"  | expression_{index}\n"
        "  | ^^^^^^^^^^^^\n"
        for index, cause in enumerate(causes, start=1)
    ).encode()


def run(case_id, engine):
    prediction = ALL_EXPECTED[case_id][engine]
    return {
        "engine": engine,
        "observation": observation(engine, prediction["exit_status"]),
        "stdout": stdout(case_id, prediction["payload_lines"]),
        "stderr": stderr(engine, prediction["causes"]),
        "fresh_process": True,
        "invocation_id": case_id + "-" + engine,
    }


def runs(case_id="weyl_context_core_cold_dual"):
    return [run(case_id, "rust"), run(case_id, "oracle")]


class WeylContextCoreContractTests(unittest.TestCase):
    def assert_nonaccepting(self, result):
        self.assertIn(result["status"], contract.NONACCEPTING_STATUSES)
        self.assertEqual(result["evidence_maturity"], "capture_only_unreviewed")
        self.assertFalse(result["acceptance_eligible"])
        self.assertFalse(result["math_gate_released"])
        self.assertFalse(result["cache_gate_released"])

    def test_exact_catalog_and_fixture_hashes(self):
        root = Path(__file__).resolve().parents[1] / "tests" / "math" / "generics"
        raw = (root / "weyl_context_core_catalog.json").read_bytes()
        self.assertEqual(hashlib.sha256(raw).hexdigest(), CATALOG_SHA256)
        catalog = contract.decode_catalog(raw)
        self.assertEqual(catalog, expected_catalog())
        for row in catalog["cases"]:
            self.assertEqual(
                hashlib.sha256((root / row["file"]).read_bytes()).hexdigest(),
                row["fixture_sha256"],
            )

    def test_catalog_mutations_are_rejected(self):
        mutations = []
        for mutate in (
            lambda value: value.update(schema="other"),
            lambda value: value.update(evidence_maturity="captured"),
            lambda value: value.update(extra=True),
            lambda value: value["cases"].reverse(),
            lambda value: value["cases"][0].update(fixture_sha256="0" * 64),
            lambda value: value["cases"][0].update(timeout_seconds=True),
            lambda value: value["cases"].pop(),
        ):
            changed = expected_catalog()
            mutate(changed)
            mutations.append(changed)
        for changed in mutations:
            with self.subTest(changed=changed):
                with self.assertRaisesRegex(ValueError, "catalog changed"):
                    contract.validate_catalog(changed)

    def test_json_object_key_order_is_not_semantic(self):
        catalog = expected_catalog()
        catalog = dict(reversed(list(catalog.items())))
        catalog["cases"] = [
            dict(reversed(list(row.items()))) for row in catalog["cases"]
        ]
        self.assertIsNone(contract.validate_catalog(catalog))

    def test_catalog_json_rejects_duplicate_and_nonfinite_values(self):
        for raw in (
            b'{"schema":"x","schema":"y"}',
            b'{"schema":NaN}',
            b'\xff',
        ):
            with self.subTest(raw=raw):
                with self.assertRaisesRegex(ValueError, "invalid .* catalog JSON"):
                    contract.decode_catalog(raw)

    def test_module_is_import_pure_and_stdlib_only(self):
        path = Path(contract.__file__)
        source = path.read_text()
        tree = ast.parse(source)
        imported = {
            alias.name.split(".", 1)[0]
            for node in ast.walk(tree)
            if isinstance(node, ast.Import)
            for alias in node.names
        }
        imported.update(
            node.module.split(".", 1)[0]
            for node in ast.walk(tree)
            if isinstance(node, ast.ImportFrom) and node.module
        )
        self.assertEqual(imported, {"hashlib", "json", "math", "re"})
        namespace = {"__name__": "weyl_context_core_contract_purity"}
        with patch.object(builtins, "open", side_effect=AssertionError("import I/O")):
            exec(compile(source, str(path), "exec"), namespace)
        self.assertIn("classify_capture", namespace)

    def test_both_source_predictions_are_capture_only(self):
        self.assertEqual(
            (contract.MISMATCH, contract.HIGH_WORD, contract.NEGATIVE_WORD),
            (MISMATCH, HIGH_WORD, NEGATIVE_WORD),
        )
        for case_id in ("weyl_context_core_cold_dual",
                        "weyl_context_core_prewarmed_dual"):
            with self.subTest(case_id=case_id):
                for engine in ("oracle", "rust"):
                    expected = EXPECTED[case_id][engine]
                    prediction = contract.PREDICTIONS[case_id][engine]
                    self.assertEqual(prediction["exit_status"], expected["exit_status"])
                    self.assertEqual(prediction["payload_lines"], expected["payload_lines"])
                    self.assertEqual(prediction["marker_lines"], expected["marker_lines"])
                    self.assertEqual(prediction["causes"], expected["causes"])
                result = contract.classify_capture(case(case_id), runs(case_id))
                self.assertEqual(
                    result["status"], "SOURCE_PREDICTIONS_OBSERVED_UNREVIEWED"
                )
                self.assertEqual(
                    result["source_predictions_observed"],
                    {"oracle": True, "rust": True},
                )
                self.assert_nonaccepting(result)

    def test_one_or_both_prediction_differences_are_distinct(self):
        expected = {
            (False, True): "ORIGINAL_SOURCE_PREDICTION_DIFFERED",
            (True, False): "RUST_SOURCE_PREDICTION_DIFFERED",
            (False, False): "BOTH_SOURCE_PREDICTIONS_DIFFERED",
        }
        for observed, status in expected.items():
            changed = runs()
            for index, engine in enumerate(("oracle", "rust")):
                if not observed[index]:
                    row = next(item for item in changed if item["engine"] == engine)
                    row["stdout"] = row["stdout"].replace(
                        b"WC_RECOVERY|727", b"WC_RECOVERY|728"
                    )
            result = contract.classify_capture(case(), changed)
            self.assertEqual(result["status"], status)
            self.assert_nonaccepting(result)

    def test_nonmarker_payload_change_changes_prediction(self):
        changed = runs()
        oracle = next(item for item in changed if item["engine"] == "oracle")
        oracle["stdout"] = oracle["stdout"].replace(
            b"Variable wc_rd: RootDatum", b"Variable wc_rd: Changed"
        )
        result = contract.classify_capture(case(), changed)
        self.assertTrue(result["arms"]["oracle"]["frame"]["complete"])
        self.assertEqual(result["status"], "ORIGINAL_SOURCE_PREDICTION_DIFFERED")
        self.assert_nonaccepting(result)

    def test_fresh_process_and_unique_invocations_are_required(self):
        for mutate in (
            lambda value: value[0].update(fresh_process=False),
            lambda value: value[0].update(invocation_id=""),
            lambda value: value[0].update(invocation_id=value[1]["invocation_id"]),
        ):
            changed = runs()
            mutate(changed)
            result = contract.classify_capture(case(), changed)
            self.assertEqual(result["status"], "CAPTURE_INCOMPLETE_FRESH_PROCESS")
            self.assertEqual(
                result["source_predictions_observed"],
                {"oracle": False, "rust": False},
            )
            self.assert_nonaccepting(result)

    def test_resource_timeout_signal_and_exit_are_incomplete(self):
        mutations = (
            ("timed_out", True),
            ("termination_uncertain", True),
            ("signal", 9),
            ("exit_status", 124),
            ("exit_status", -9),
        )
        for key, value in mutations:
            changed = runs()
            changed[0]["observation"][key] = value
            result = contract.classify_capture(case(), changed)
            self.assertEqual(
                result["status"], "CAPTURE_INCOMPLETE_RESOURCE_OR_TIMEOUT"
            )
            self.assertEqual(
                result["source_predictions_observed"],
                {"oracle": False, "rust": False},
            )
            arm = result["arms"][changed[0]["engine"]]
            self.assertEqual(arm["observation"][key], value)
            self.assert_nonaccepting(result)

    def test_metrics_must_be_finite_nonnegative_and_exact(self):
        mutations = (
            ("seconds", float("nan")),
            ("user_cpu_seconds", -1.0),
            ("system_cpu_seconds", float("inf")),
            ("maxrss_kb", -1),
            ("maxrss_approximate", True),
        )
        for key, value in mutations:
            changed = runs()
            changed[0]["observation"][key] = value
            result = contract.classify_capture(case(), changed)
            self.assertEqual(result["status"], "CAPTURE_INCOMPLETE_METRICS")
            self.assertEqual(
                result["source_predictions_observed"],
                {"oracle": False, "rust": False},
            )
            json.dumps(result, allow_nan=False)
            self.assert_nonaccepting(result)

        changed = runs()
        changed[0]["observation"]["seconds"] = 10 ** 1000
        result = contract.classify_capture(case(), changed)
        self.assertEqual(result["status"], "SOURCE_PREDICTIONS_OBSERVED_UNREVIEWED")
        self.assertEqual(result["arms"]["rust"]["metrics"]["seconds"], 10 ** 1000)
        json.dumps(result, allow_nan=False)

    def test_delimiters_define_completeness_but_utf8_prefix_suffix_differ(self):
        good = run("weyl_context_core_cold_dual", "oracle")["stdout"]
        begin = b"MATH_BEGIN weyl_context_core_cold_dual\n"
        end = b"MATH_END weyl_context_core_cold_dual\n"
        incomplete = (
            good.replace(begin, b""),
            begin + good,
            good.replace(end, b"MATH_END other\n"),
            end + b"payload\n" + begin + b"Bye.\n",
            good.replace(b"Variable", b"\xffVariable", 1),
        )
        for changed_stdout in incomplete:
            changed = runs()
            next(item for item in changed if item["engine"] == "oracle")["stdout"] = changed_stdout
            result = contract.classify_capture(case(), changed)
            self.assertEqual(result["status"], "CAPTURE_INCOMPLETE_STREAM")
            self.assert_nonaccepting(result)

        prediction_differences = (
            b"junk\n" + good,
            good.replace(end + b"Bye.\n", end + b"extra\nBye.\n"),
            good + b"extra\n",
        )
        for changed_stdout in prediction_differences:
            changed = runs()
            next(item for item in changed if item["engine"] == "oracle")["stdout"] = changed_stdout
            result = contract.classify_capture(case(), changed)
            self.assertEqual(result["status"], "ORIGINAL_SOURCE_PREDICTION_DIFFERED")
            self.assertTrue(result["stream_complete"])
            self.assert_nonaccepting(result)

    def test_malformed_marker_is_a_complete_prediction_difference(self):
        changed = runs()
        row = next(item for item in changed if item["engine"] == "rust")
        row["stdout"] = row["stdout"].replace(
            b"WC_SAME|[0]|1|[1,0]|[]", b"WC_SAME"
        )
        result = contract.classify_capture(case(), changed)
        self.assertEqual(result["status"], "RUST_SOURCE_PREDICTION_DIFFERED")
        self.assertTrue(result["arms"]["rust"]["frame"]["complete"])
        self.assertEqual(
            result["arms"]["rust"]["frame"]["unclassified_markers"][0]["line"],
            "WC_SAME",
        )
        self.assert_nonaccepting(result)

    def test_marker_duplicate_reorder_and_missing_change_prediction_only(self):
        row = run("weyl_context_core_cold_dual", "rust")
        first = b"WC_DUAL_COLD_OWNER|true\n"
        second = b"WC_DUAL_COLD_EQ|false\n"
        mutations = (
            row["stdout"].replace(first, b""),
            row["stdout"].replace(first, first + first),
            row["stdout"].replace(first + second, second + first),
        )
        for changed_stdout in mutations:
            changed = runs()
            rust = next(item for item in changed if item["engine"] == "rust")
            rust["stdout"] = changed_stdout
            result = contract.classify_capture(case(), changed)
            self.assertTrue(result["arms"]["rust"]["frame"]["complete"])
            self.assertEqual(result["status"], "RUST_SOURCE_PREDICTION_DIFFERED")
            self.assert_nonaccepting(result)

    def test_diagnostic_multiplicity_and_order_change_prediction(self):
        baseline = list(EXPECTED["weyl_context_core_prewarmed_dual"]["oracle"]["causes"])
        swapped = list(baseline)
        swapped[0], swapped[3] = swapped[3], swapped[0]
        for observed in (swapped, baseline[1:], [baseline[0], *baseline]):
            changed = runs("weyl_context_core_prewarmed_dual")
            oracle = next(item for item in changed if item["engine"] == "oracle")
            oracle["stderr"] = stderr("oracle", observed)
            result = contract.classify_capture(
                case("weyl_context_core_prewarmed_dual"), changed
            )
            self.assertEqual(result["status"], "ORIGINAL_SOURCE_PREDICTION_DIFFERED")
            self.assertEqual(
                [row["cause"] for row in result["arms"]["oracle"]["diagnostic_causes"]],
                observed,
            )
            self.assert_nonaccepting(result)

    def test_unknown_complete_diagnostics_differ_but_truncation_is_incomplete(self):
        for engine in ("oracle", "rust"):
            changed = runs("weyl_context_core_prewarmed_dual")
            row = next(item for item in changed if item["engine"] == engine)
            row["stderr"] += b"unexpected diagnostic text\n"
            result = contract.classify_capture(
                case("weyl_context_core_prewarmed_dual"), changed
            )
            expected = (
                "ORIGINAL_SOURCE_PREDICTION_DIFFERED"
                if engine == "oracle" else "RUST_SOURCE_PREDICTION_DIFFERED"
            )
            self.assertEqual(result["status"], expected)
            self.assertTrue(result["arms"][engine]["diagnostic_stream"]["complete"])
            self.assertTrue(result["arms"][engine]["diagnostic_stream"]["unclassified"])
            self.assert_nonaccepting(result)

        for engine in ("oracle", "rust"):
            changed = runs("weyl_context_core_prewarmed_dual")
            row = next(item for item in changed if item["engine"] == engine)
            row["stderr"] = row["stderr"].replace(
                b"Weyl group mismatch", b"unfamiliar complete diagnostic", 1
            )
            result = contract.classify_capture(
                case("weyl_context_core_prewarmed_dual"), changed
            )
            expected = (
                "ORIGINAL_SOURCE_PREDICTION_DIFFERED"
                if engine == "oracle" else "RUST_SOURCE_PREDICTION_DIFFERED"
            )
            self.assertEqual(result["status"], expected)
            causes = result["arms"][engine]["diagnostic_causes"]
            self.assertIsNone(causes[0]["cause"])
            self.assertEqual(causes[0]["message"], "unfamiliar complete diagnostic")
            self.assert_nonaccepting(result)

        incomplete = (
            lambda value: value[:-1],
            lambda value: value.replace(b"Weyl group mismatch", b"\xff", 1),
            lambda value: value.replace(
                b"  Weyl group mismatch\nEvaluation aborted.\n",
                b"  Weyl group mismatch\n",
                1,
            ),
        )
        for mutation in incomplete:
            changed = runs("weyl_context_core_prewarmed_dual")
            oracle = next(item for item in changed if item["engine"] == "oracle")
            oracle["stderr"] = mutation(oracle["stderr"])
            result = contract.classify_capture(
                case("weyl_context_core_prewarmed_dual"), changed
            )
            self.assertEqual(result["status"], "CAPTURE_INCOMPLETE_STREAM")
            self.assertFalse(result["arms"]["oracle"]["diagnostic_stream"]["complete"])
            self.assert_nonaccepting(result)

    def test_reject_case_compares_stderr_by_ordered_error_messages(self):
        # The AFTER-v4 failure: both engines rejected the same commands with
        # the same ordered messages in different envelopes.  Byte equality
        # across two diagnostic renderers is unattainable, so the reject
        # intent compares the ordered error-summary contents instead.
        baseline = list(EXPECTED["weyl_context_core_prewarmed_dual"]["oracle"]["causes"])
        changed = runs("weyl_context_core_prewarmed_dual")
        rust = next(item for item in changed if item["engine"] == "rust")
        rust["stderr"] = stderr("rust", baseline)
        self.assertNotEqual(
            next(item for item in changed if item["engine"] == "oracle")["stderr"],
            rust["stderr"],
        )
        result = contract.classify_capture(
            case("weyl_context_core_prewarmed_dual"), changed
        )
        self.assertTrue(result["full_stderr_equal"])
        self.assert_nonaccepting(result)

        for observed in (baseline[1:], baseline[:-1], list(reversed(baseline))):
            changed = runs("weyl_context_core_prewarmed_dual")
            rust = next(item for item in changed if item["engine"] == "rust")
            rust["stderr"] = stderr("rust", observed)
            result = contract.classify_capture(
                case("weyl_context_core_prewarmed_dual"), changed
            )
            self.assertFalse(result["full_stderr_equal"])
            self.assert_nonaccepting(result)

        for mutation in (
            lambda value: value + b"trailing text\n",
            lambda value: value.replace(
                b"Runtime error at <stdin>:1:1:", b"Runtime error at <stdin>:1", 1),
        ):
            changed = runs("weyl_context_core_prewarmed_dual")
            rust = next(item for item in changed if item["engine"] == "rust")
            rust["stderr"] = mutation(stderr("rust", baseline))
            result = contract.classify_capture(
                case("weyl_context_core_prewarmed_dual"), changed
            )
            self.assertFalse(result["full_stderr_equal"])
            self.assert_nonaccepting(result)

        changed = runs("weyl_context_core_prewarmed_dual")
        oracle = next(item for item in changed if item["engine"] == "oracle")
        oracle["stderr"] = oracle["stderr"].replace(
            b"Evaluation aborted.\n", b"", 1)
        rust = next(item for item in changed if item["engine"] == "rust")
        rust["stderr"] = stderr("rust", baseline)
        result = contract.classify_capture(
            case("weyl_context_core_prewarmed_dual"), changed
        )
        self.assertFalse(result["full_stderr_equal"])
        self.assert_nonaccepting(result)

    def test_accept_case_keeps_stderr_byte_equality(self):
        # The cold_dual (accept intent) keeps byte equality: an empty oracle
        # stream and a nonempty rust stream differ even if the rust stream's
        # messages would extract cleanly.
        changed = runs("weyl_context_core_cold_dual")
        oracle = next(item for item in changed if item["engine"] == "oracle")
        rust = next(item for item in changed if item["engine"] == "rust")
        rust["stderr"] = oracle["stderr"]
        result = contract.classify_capture(
            case("weyl_context_core_cold_dual"), changed
        )
        self.assertTrue(result["full_stderr_equal"])
        self.assert_nonaccepting(result)

    def test_raw_hashes_lengths_lines_and_equality_are_preserved(self):
        captured = runs()
        result = contract.classify_capture(case(), captured)
        for engine in ("oracle", "rust"):
            source = next(item for item in captured if item["engine"] == engine)
            arm = result["arms"][engine]
            self.assertEqual(arm["observation"]["timed_out"], False)
            self.assertEqual(arm["observation"]["termination_uncertain"], False)
            self.assertIsNone(arm["observation"]["signal"])
            self.assertEqual(arm["stdout"]["sha256"], hashlib.sha256(source["stdout"]).hexdigest())
            self.assertEqual(arm["stderr"]["sha256"], hashlib.sha256(source["stderr"]).hexdigest())
            self.assertEqual(arm["stdout"]["bytes"], len(source["stdout"]))
            self.assertEqual(arm["stderr"]["bytes"], len(source["stderr"]))
            begin = ("MATH_BEGIN " + case()["id"] + "\n").encode()
            end = ("MATH_END " + case()["id"] + "\n").encode()
            payload = source["stdout"][len(begin):source["stdout"].index(end)]
            self.assertEqual(
                arm["frame"]["payload_sha256"], hashlib.sha256(payload).hexdigest()
            )
            self.assertEqual(arm["frame"]["payload_bytes"], len(payload))
            for marker in arm["frame"]["markers"]:
                self.assertEqual(bytes.fromhex(marker["line_hex"]).decode(), marker["line"])
                raw_line = bytes.fromhex(marker["line_hex"])
                self.assertEqual(marker["line_sha256"], hashlib.sha256(raw_line).hexdigest())
                self.assertEqual(marker["line_bytes"], len(raw_line))
            for diagnostic in arm["diagnostic_causes"]:
                raw_line = bytes.fromhex(diagnostic["line_hex"])
                self.assertEqual(diagnostic["line_sha256"], hashlib.sha256(raw_line).hexdigest())
                self.assertEqual(diagnostic["line_bytes"], len(raw_line))
                self.assertGreaterEqual(diagnostic["line_number"], 1)
        self.assertFalse(result["full_stdout_equal"])
        self.assertFalse(result["full_stderr_equal"])
        self.assertFalse(result["exit_status_equal"])
        json.dumps(result, allow_nan=False)
        self.assert_nonaccepting(result)

    def test_run_schema_and_case_identity_are_strict(self):
        changed_case = case()
        changed_case["extra"] = True
        with self.assertRaisesRegex(ValueError, "unknown or changed"):
            contract.classify_capture(changed_case, runs())
        duplicate = runs()
        duplicate[1]["engine"] = duplicate[0]["engine"]
        with self.assertRaisesRegex(ValueError, "exactly once"):
            contract.classify_capture(case(), duplicate)
        extra = runs()
        extra[0]["extra"] = True
        with self.assertRaisesRegex(ValueError, "schema changed"):
            contract.classify_capture(case(), extra)
        for mutate in (
            lambda value: value[0]["observation"].update(extra=True),
            lambda value: value[0]["observation"].pop("engine"),
            lambda value: value[0]["observation"].update(signal=False),
            lambda value: value[0]["observation"].update(seconds=None),
            lambda value: value[0]["observation"].update(maxrss_kb=True),
        ):
            changed = runs()
            mutate(changed)
            with self.assertRaisesRegex(ValueError, "observation|signal|metric|RSS"):
                contract.classify_capture(case(), changed)

    def test_g2_exact_catalog_and_fixture_hashes(self):
        root = Path(__file__).resolve().parents[1] / "tests" / "math" / "generics"
        raw = (root / "weyl_context_g2_catalog.json").read_bytes()
        self.assertEqual(hashlib.sha256(raw).hexdigest(), G2_CATALOG_SHA256)
        catalog = contract.decode_g2_catalog(raw)
        self.assertEqual(catalog, g2_expected_catalog())
        for row in catalog["cases"]:
            self.assertEqual(
                hashlib.sha256((root / row["file"]).read_bytes()).hexdigest(),
                row["fixture_sha256"],
            )

    def test_g2_catalog_mutations_are_rejected(self):
        mutations = []
        for mutate in (
            lambda value: value.update(schema="other"),
            lambda value: value.update(evidence_maturity="captured"),
            lambda value: value.update(extra=True),
            lambda value: value["cases"].reverse(),
            lambda value: value["cases"][0].update(fixture_sha256="0" * 64),
            lambda value: value["cases"][0].update(timeout_seconds=True),
            lambda value: value["cases"].pop(),
        ):
            changed = g2_expected_catalog()
            mutate(changed)
            mutations.append(changed)
        for changed in mutations:
            with self.subTest(changed=changed):
                with self.assertRaisesRegex(ValueError, "G2 catalog changed"):
                    contract.validate_g2_catalog(changed)

    def test_g2_both_source_predictions_observed_unreviewed(self):
        self.assertEqual(contract.HIGH_WORD_G2, HIGH_WORD_G2)
        for case_id in G2_EXPECTED:
            with self.subTest(case_id=case_id):
                for engine in ("oracle", "rust"):
                    expected = G2_EXPECTED[case_id][engine]
                    prediction = contract.G2_PREDICTIONS[case_id][engine]
                    self.assertEqual(
                        prediction["exit_status"], expected["exit_status"]
                    )
                    self.assertEqual(
                        prediction["payload_lines"], expected["payload_lines"]
                    )
                    self.assertEqual(
                        prediction["marker_lines"], expected["marker_lines"]
                    )
                    self.assertEqual(prediction["causes"], expected["causes"])
                result = contract.classify_capture(case(case_id), runs(case_id))
                self.assertEqual(
                    result["status"], "SOURCE_PREDICTIONS_OBSERVED_UNREVIEWED"
                )
                self.assertEqual(
                    result["source_predictions_observed"],
                    {"oracle": True, "rust": True},
                )
                self.assertTrue(result["full_stdout_equal"])
                self.assertTrue(result["full_stderr_equal"])
                self.assertTrue(result["exit_status_equal"])
                self.assert_nonaccepting(result)

    def test_g2_prediction_differences_are_distinct(self):
        changed = runs("weyl_context_g2_cold_dual")
        rust = next(item for item in changed if item["engine"] == "rust")
        rust["stdout"] = rust["stdout"].replace(
            b"WG_BRAID|true|6", b"WG_BRAID|false|6"
        )
        result = contract.classify_capture(
            case("weyl_context_g2_cold_dual"), changed
        )
        self.assertEqual(result["status"], "RUST_SOURCE_PREDICTION_DIFFERED")
        self.assertFalse(result["full_stdout_equal"])
        self.assert_nonaccepting(result)

        changed = runs("weyl_context_g2_prewarmed_dual")
        oracle = next(item for item in changed if item["engine"] == "oracle")
        oracle["stderr"] = oracle["stderr"].replace(
            b"Illegal Weyl word entry 2 (should be <2)",
            b"Illegal Weyl word entry 3 (should be <2)",
        )
        result = contract.classify_capture(
            case("weyl_context_g2_prewarmed_dual"), changed
        )
        self.assertEqual(result["status"], "ORIGINAL_SOURCE_PREDICTION_DIFFERED")
        self.assert_nonaccepting(result)


if __name__ == "__main__":
    unittest.main()

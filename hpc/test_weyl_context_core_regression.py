import ast
import builtins
from copy import deepcopy
import hashlib
from pathlib import Path
import unittest
from unittest.mock import patch

import weyl_context_core_regression as regression


ROOT = Path(__file__).resolve().parents[1]
GENERICS = ROOT / "tests" / "math" / "generics"
INSPECTION = ROOT / regression.INSPECTION_FILE


def metric():
    return {
        "format": "gnu-time-v",
        "seconds": 1.25,
        "user_cpu_seconds": 1.0,
        "system_cpu_seconds": 0.25,
        "maxrss_kb": 4096,
        "maxrss_approximate": False,
        "timed_out": False,
        "termination_uncertain": False,
        "signal": None,
    }


def frozen_inputs():
    catalog_raw = (
        GENERICS / "weyl_context_core_regression_catalog.json"
    ).read_bytes()
    inspection_raw = INSPECTION.read_bytes()
    catalog = regression.decode_catalog(catalog_raw)
    artifacts = {}
    for case in catalog["cases"]:
        for name in (
            case["fixture"],
            case["oracle_stdout"]["file"],
            case["oracle_stderr"]["file"],
        ):
            artifacts[name] = (GENERICS / name).read_bytes()
    return catalog_raw, inspection_raw, catalog, artifacts


def inventory_log():
    names = list(regression.SELECTOR_TESTS) + [regression.CONTROL_TEST]
    names.extend(
        "session::tests::frozen_baseline_%03d" % index
        for index in range(regression.EXPECTED_INVENTORY_COUNT - len(names))
    )
    return (
        "\n".join(name + ": test" for name in names)
        + "\n\n632 tests, 0 benchmarks\n"
    ).encode()


def expected_failure_log():
    return (
        "running 2 tests\n"
        "WEYL_CONTEXT_CORE_READY weyl_context_core_cold_dual diagnostics=2\n"
        "thread 'session::tests::weyl_context_core_cold_dual_original' "
        "panicked at crates/atlas-core/src/session.rs:1:1:\n"
        "complete original Weyl-context stdout and ordered diagnostics\n"
        "WEYL_CONTEXT_CORE_READY weyl_context_core_prewarmed_dual diagnostics=4\n"
        "thread 'session::tests::weyl_context_core_prewarmed_dual_original' "
        "panicked at crates/atlas-core/src/session.rs:1:1:\n"
        "complete original Weyl-context stdout and ordered diagnostics\n"
        "test result: FAILED. 0 passed; 2 failed; 0 ignored; 0 measured; "
        "630 filtered out; finished in 0.01s\n"
    ).encode()


def unexpected_pass_log():
    return (
        "running 2 tests\n"
        "WEYL_CONTEXT_CORE_READY weyl_context_core_cold_dual diagnostics=0\n"
        "WEYL_CONTEXT_CORE_READY weyl_context_core_prewarmed_dual diagnostics=8\n"
        "test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; "
        "630 filtered out; finished in 0.01s\n"
    ).encode()


def control_log():
    return (
        "running 1 test\n"
        "test session::tests::root_ladder_coordinate_boundary_original ... ok\n"
        "test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; "
        "631 filtered out; finished in 0.01s\n"
    ).encode()


def command(name, code, log):
    return {
        "name": name,
        "exit_status": code,
        "log": log,
        "metrics": metric(),
    }


def observation(
    catalog, artifacts, *, selector_passes=False, schema=regression.OBSERVATION_SCHEMA
):
    original = []
    for case in catalog["cases"]:
        original.append(
            {
                "case_id": case["id"],
                "engine": "oracle",
                "exit_status": case["oracle_exit_status"],
                "stdout": artifacts[case["oracle_stdout"]["file"]],
                "stderr": artifacts[case["oracle_stderr"]["file"]],
                "fresh_process": True,
                "timed_out": False,
                "termination_uncertain": False,
                "metrics": metric(),
            }
        )
    claims = {
        "acceptance_eligible": False,
        "math_gate_released": False,
        "cache_gate_released": False,
        "performance_gate_released": False,
        "rank_gate_released": False,
    }
    return {
        "schema": schema,
        "provenance": regression.expected_provenance(),
        "original_reruns": original,
        "selector_command": command(
            "weyl-context-regressions",
            0 if selector_passes else 101,
            unexpected_pass_log() if selector_passes else expected_failure_log(),
        ),
        "retained_control_command": command(
            "root-ladder-control", 0, control_log()
        ),
        "inventory_command": command(
            "atlas-core-test-inventory", 0, inventory_log()
        ),
        "claims": claims,
    }


class WeylContextCoreRegressionTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        (
            cls.catalog_raw,
            cls.inspection_raw,
            cls.catalog,
            cls.artifacts,
        ) = frozen_inputs()

    def classify(self, observed):
        return regression.classify_before(
            self.catalog_raw,
            self.inspection_raw,
            self.artifacts,
            observed,
        )

    def classify_after(self, observed):
        return regression.classify_after(
            self.catalog_raw,
            self.inspection_raw,
            self.artifacts,
            observed,
        )

    def assert_no_release(self, result):
        for key in (
            "acceptance_eligible",
            "math_gate_released",
            "cache_gate_released",
            "performance_gate_released",
            "rank_gate_released",
        ):
            self.assertIs(result[key], False)

    def test_frozen_catalog_inspection_and_artifact_hashes(self):
        self.assertEqual(len(self.catalog_raw), regression.CATALOG_BYTES)
        self.assertEqual(
            hashlib.sha256(self.catalog_raw).hexdigest(),
            regression.CATALOG_SHA256,
        )
        self.assertEqual(len(self.inspection_raw), regression.INSPECTION_BYTES)
        self.assertEqual(
            hashlib.sha256(self.inspection_raw).hexdigest(),
            regression.INSPECTION_SHA256,
        )
        inspection = regression.decode_inspection(
            self.inspection_raw, self.catalog
        )
        self.assertEqual(inspection["job"]["id"], regression.CAPTURE_JOB)
        malformed = deepcopy(inspection)
        malformed["captures"] = [None, malformed["captures"][1]]
        with self.assertRaises(regression.ProvenanceError):
            regression.validate_inspection(malformed, self.catalog)
        self.assertIs(
            regression.validate_artifacts(self.catalog, self.artifacts),
            self.artifacts,
        )

    def test_catalog_and_inspection_require_byte_exact_inputs(self):
        for value in (
            self.catalog_raw + b"\n",
            self.catalog_raw.replace(b'"schema"', b'"schema2"', 1),
            self.catalog_raw.decode(),
        ):
            with self.subTest(value=type(value)):
                with self.assertRaises(regression.ProvenanceError):
                    regression.decode_catalog(value)
        for value in (
            self.inspection_raw + b" ",
            self.inspection_raw.replace(b'"classification"', b'"classificatioN"', 1),
            None,
        ):
            with self.subTest(value=type(value)):
                with self.assertRaises(regression.ProvenanceError):
                    regression.decode_inspection(value, self.catalog)

    def test_catalog_schema_keys_types_and_case_order_are_exact(self):
        for mutate in (
            lambda value: value.update(extra=False),
            lambda value: value.update(evidence_maturity="accepted"),
            lambda value: value["source_capture"].update(report_bytes=True),
            lambda value: value["cases"].reverse(),
            lambda value: value["cases"][0].update(fixture_bytes=True),
            lambda value: value["cases"][1]["ordered_oracle_diagnostics"].reverse(),
            lambda value: value.update(cases=[None, value["cases"][1]]),
        ):
            changed = deepcopy(self.catalog)
            mutate(changed)
            with self.subTest(mutate=mutate):
                with self.assertRaises(regression.ProvenanceError):
                    regression.validate_catalog(changed)

    def test_fixture_envelopes_empty_stderr_and_ordered_diagnostics(self):
        cold = regression._CASE_METADATA[regression.COLD_CASE]
        warm = regression._CASE_METADATA[regression.PREWARM_CASE]
        self.assertEqual(
            self.artifacts[cold["oracle_stderr"]["file"]], b""
        )
        self.assertEqual(
            regression._diagnostic_blocks(
                self.artifacts[warm["oracle_stderr"]["file"]]
            ),
            warm["ordered_diagnostics"],
        )
        for case in self.catalog["cases"]:
            stdout = self.artifacts[case["oracle_stdout"]["file"]]
            self.assertTrue(stdout.startswith(("MATH_BEGIN " + case["id"] + "\n").encode()))
            self.assertTrue(stdout.endswith(("MATH_END " + case["id"] + "\nBye.\n").encode()))

    def test_missing_extra_nonbytes_and_changed_artifacts_are_rejected(self):
        changes = []
        missing = dict(self.artifacts)
        missing.pop(next(iter(missing)))
        changes.append(missing)
        extra = dict(self.artifacts, unexpected=b"")
        changes.append(extra)
        nonbytes = dict(self.artifacts)
        nonbytes[next(iter(nonbytes))] = "not bytes"
        changes.append(nonbytes)
        changed = dict(self.artifacts)
        name = self.catalog["cases"][0]["oracle_stdout"]["file"]
        changed[name] += b"x"
        changes.append(changed)
        cold_stderr = dict(self.artifacts)
        name = self.catalog["cases"][0]["oracle_stderr"]["file"]
        cold_stderr[name] = b"x"
        changes.append(cold_stderr)
        for value in changes:
            with self.subTest(names=sorted(value)):
                with self.assertRaises(regression.ProvenanceError):
                    regression.validate_artifacts(self.catalog, value)

    def test_module_is_import_pure_and_stdlib_only(self):
        path = Path(regression.__file__)
        source = path.read_text()
        tree = ast.parse(source)
        imports = {
            alias.name.split(".", 1)[0]
            for node in ast.walk(tree)
            if isinstance(node, ast.Import)
            for alias in node.names
        }
        imports.update(
            node.module.split(".", 1)[0]
            for node in ast.walk(tree)
            if isinstance(node, ast.ImportFrom) and node.module
        )
        self.assertEqual(imports, {"hashlib", "json", "math", "re"})
        namespace = {"__name__": "weyl_context_regression_purity"}
        with patch.object(builtins, "open", side_effect=AssertionError("import I/O")):
            exec(compile(source, str(path), "exec"), namespace)
        self.assertIn("classify_before", namespace)

    def test_expected_before_requires_all_original_backed_failures(self):
        result = self.classify(observation(self.catalog, self.artifacts))
        self.assertEqual(result["status"], regression.BEFORE_REPRODUCED)
        self.assertTrue(result["original_goldens_matched"])
        self.assertTrue(result["expected_regressions_observed"])
        self.assertFalse(result["unexpected_regressions_passed"])
        self.assertTrue(result["retained_control_passed"])
        self.assertTrue(result["inventory_complete"])
        self.assertTrue(result["metrics_complete"])
        self.assert_no_release(result)

    def test_unexpected_pass_is_distinct_and_nonaccepting(self):
        result = self.classify(
            observation(self.catalog, self.artifacts, selector_passes=True)
        )
        self.assertEqual(result["status"], regression.UNEXPECTED_PASS)
        self.assertFalse(result["expected_regressions_observed"])
        self.assertTrue(result["unexpected_regressions_passed"])
        self.assert_no_release(result)

    def test_provenance_and_harness_failures_are_distinct(self):
        changed = observation(self.catalog, self.artifacts)
        changed["provenance"]["tests_only_session_sha256"] = "0" * 64
        provenance = self.classify(changed)
        self.assertEqual(provenance["status"], regression.PROVENANCE_FAILURE)

        changed = observation(self.catalog, self.artifacts)
        changed["selector_command"]["exit_status"] = 1
        harness = self.classify(changed)
        self.assertEqual(harness["status"], regression.HARNESS_FAILURE)
        self.assertNotEqual(provenance["status"], harness["status"])
        self.assert_no_release(provenance)
        self.assert_no_release(harness)

    def test_original_reruns_are_byte_exact_fresh_and_ordered(self):
        changes = []
        for mutate in (
            lambda value: value["original_reruns"].reverse(),
            lambda value: value["original_reruns"][0].update(stdout=b"changed"),
            lambda value: value["original_reruns"][0].update(exit_status=1),
            lambda value: value["original_reruns"][0].update(fresh_process=False),
            lambda value: value["original_reruns"][0].update(timed_out=True),
        ):
            changed = observation(self.catalog, self.artifacts)
            mutate(changed)
            changes.append(changed)
        for changed in changes:
            with self.subTest(run=changed["original_reruns"][0]["case_id"]):
                result = self.classify(changed)
                self.assertEqual(result["status"], regression.PROVENANCE_FAILURE)
                self.assert_no_release(result)
        changed = observation(self.catalog, self.artifacts)
        changed["original_reruns"][0] = None
        self.assertEqual(
            self.classify(changed)["status"], regression.PROVENANCE_FAILURE
        )

    def test_selector_requires_exact_names_summary_ready_counts_and_exit101(self):
        replacements = (
            (b"diagnostics=2", b"diagnostics=1"),
            (b"diagnostics=4", b"diagnostics=5"),
            (b"0 passed", b"1 passed"),
            (b"2 failed", b"1 failed"),
            (b"0 ignored", b"1 ignored"),
            (b"630 filtered", b"629 filtered"),
            (
                b"session::tests::weyl_context_core_cold_dual_original",
                b"session::tests::unrelated",
            ),
        )
        for old, new in replacements:
            changed = observation(self.catalog, self.artifacts)
            changed["selector_command"]["log"] = changed["selector_command"][
                "log"
            ].replace(old, new, 1)
            with self.subTest(old=old):
                self.assertEqual(
                    self.classify(changed)["status"], regression.HARNESS_FAILURE
                )

    def test_each_ready_and_failure_record_is_required_once(self):
        for mode in ("missing", "duplicate"):
            changed = observation(self.catalog, self.artifacts)
            line = (
                b"WEYL_CONTEXT_CORE_READY weyl_context_core_cold_dual "
                b"diagnostics=2\n"
            )
            if mode == "missing":
                changed["selector_command"]["log"] = changed[
                    "selector_command"
                ]["log"].replace(line, b"")
            else:
                changed["selector_command"]["log"] += line
            with self.subTest(mode=mode):
                self.assertEqual(
                    self.classify(changed)["status"], regression.HARNESS_FAILURE
                )

    def test_retained_root_ladder_control_is_mandatory(self):
        changes = []
        for mutate in (
            lambda value: value["retained_control_command"].update(exit_status=101),
            lambda value: value["retained_control_command"].update(
                log=value["retained_control_command"]["log"].replace(
                    b"1 passed", b"0 passed"
                )
            ),
            lambda value: value["retained_control_command"].update(
                log=value["retained_control_command"]["log"].replace(
                    regression.CONTROL_TEST.encode(), b"session::tests::unrelated"
                )
            ),
        ):
            changed = observation(self.catalog, self.artifacts)
            mutate(changed)
            changes.append(changed)
        for changed in changes:
            self.assertEqual(
                self.classify(changed)["status"], regression.HARNESS_FAILURE
            )

    def test_inventory_requires_632_unique_tests_and_all_three_controls(self):
        changes = []
        missing = observation(self.catalog, self.artifacts)
        missing["inventory_command"]["log"] = missing["inventory_command"][
            "log"
        ].replace((regression.CONTROL_TEST + ": test\n").encode(), b"")
        changes.append(missing)
        duplicate = observation(self.catalog, self.artifacts)
        line = (regression.SELECTOR_TESTS[0] + ": test\n").encode()
        duplicate["inventory_command"]["log"] += line
        changes.append(duplicate)
        wrong_footer = observation(self.catalog, self.artifacts)
        wrong_footer["inventory_command"]["log"] = wrong_footer[
            "inventory_command"
        ]["log"].replace(b"632 tests", b"631 tests")
        changes.append(wrong_footer)
        for changed in changes:
            self.assertEqual(
                self.classify(changed)["status"], regression.HARNESS_FAILURE
            )

    def test_every_command_and_oracle_rerun_requires_complete_gnu_time(self):
        locations = (
            ("selector_command", None),
            ("retained_control_command", None),
            ("inventory_command", None),
            ("original_reruns", 0),
            ("original_reruns", 1),
        )
        for key, index in locations:
            changed = observation(self.catalog, self.artifacts)
            record = changed[key] if index is None else changed[key][index]
            record["metrics"].pop("user_cpu_seconds")
            with self.subTest(key=key, index=index):
                expected = (
                    regression.PROVENANCE_FAILURE
                    if index is not None
                    else regression.HARNESS_FAILURE
                )
                self.assertEqual(self.classify(changed)["status"], expected)

    def test_approximate_timeout_uncertain_and_nonfinite_metrics_fail(self):
        changes = (
            ("maxrss_approximate", True),
            ("timed_out", True),
            ("termination_uncertain", True),
            ("signal", 9),
            ("seconds", float("nan")),
            ("maxrss_kb", True),
        )
        for key, value in changes:
            changed = observation(self.catalog, self.artifacts)
            changed["selector_command"]["metrics"][key] = value
            with self.subTest(key=key):
                self.assertEqual(
                    self.classify(changed)["status"], regression.HARNESS_FAILURE
                )

    def test_observation_cannot_add_keys_or_release_any_gate(self):
        changed = observation(self.catalog, self.artifacts)
        changed["extra"] = False
        self.assertEqual(
            self.classify(changed)["status"], regression.PROVENANCE_FAILURE
        )
        for key in changed["claims"]:
            changed = observation(self.catalog, self.artifacts)
            changed["claims"][key] = True
            with self.subTest(key=key):
                result = self.classify(changed)
                self.assertEqual(result["status"], regression.HARNESS_FAILURE)
                self.assert_no_release(result)

    def test_after_passing_selector_reports_regressions_passed(self):
        result = self.classify_after(
            observation(
                self.catalog,
                self.artifacts,
                selector_passes=True,
                schema=regression.AFTER_OBSERVATION_SCHEMA,
            )
        )
        self.assertEqual(result["schema"], regression.AFTER_CLASSIFICATION_SCHEMA)
        self.assertEqual(result["status"], regression.AFTER_REPRODUCED)
        self.assertEqual(result["evidence_maturity"], "tests_first_after")
        self.assertIs(result["expected_regressions_passed"], True)
        self.assertIs(result["regressions_still_failing"], False)
        self.assertTrue(result["original_goldens_matched"])
        self.assertTrue(result["retained_control_passed"])
        self.assertTrue(result["inventory_complete"])
        self.assertTrue(result["metrics_complete"])
        self.assert_no_release(result)

    def test_after_still_failing_selector_reports_still_failing(self):
        result = self.classify_after(
            observation(
                self.catalog,
                self.artifacts,
                schema=regression.AFTER_OBSERVATION_SCHEMA,
            )
        )
        self.assertEqual(result["schema"], regression.AFTER_CLASSIFICATION_SCHEMA)
        self.assertEqual(result["status"], regression.AFTER_STILL_FAILING)
        self.assertEqual(result["evidence_maturity"], "tests_first_after")
        self.assertIs(result["regressions_still_failing"], True)
        self.assertIs(result["expected_regressions_passed"], False)
        self.assertTrue(result["original_goldens_matched"])
        self.assertTrue(result["retained_control_passed"])
        self.assertTrue(result["inventory_complete"])
        self.assertTrue(result["metrics_complete"])
        self.assert_no_release(result)

    def test_after_rejects_before_or_bogus_observation_schema(self):
        for schema in (regression.OBSERVATION_SCHEMA, "bogus-schema"):
            observed = observation(
                self.catalog,
                self.artifacts,
                selector_passes=True,
                schema=schema,
            )
            with self.subTest(schema=schema):
                result = self.classify_after(observed)
                self.assertEqual(
                    result["status"], regression.AFTER_PROVENANCE_FAILURE
                )
                self.assertIs(result["expected_regressions_passed"], False)
                self.assertIs(result["regressions_still_failing"], False)
                self.assert_no_release(result)

    def test_after_selector_rejects_unexpected_exit_status(self):
        for code in (1, 2):
            with self.subTest(exit_status=code):
                record = command(
                    "weyl-context-regressions", code, expected_failure_log()
                )
                with self.assertRaises(regression.HarnessError):
                    regression._selector_state_after(record)


if __name__ == "__main__":
    unittest.main()

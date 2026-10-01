import builtins
import copy
from contextlib import contextmanager
import dis
import hashlib
import inspect
from pathlib import Path
import tempfile
import types
import unittest
from unittest.mock import call, patch

import math_ladder_boundary_after as driver
import stage_ladder_boundary_after as stager


def unresolved_globals(code, namespace):
    missing = {
        instruction.argval for instruction in dis.get_instructions(code)
        if instruction.opname == "LOAD_GLOBAL"
        and instruction.argval not in namespace
        and not hasattr(builtins, instruction.argval)
    }
    for constant in code.co_consts:
        if isinstance(constant, types.CodeType):
            missing.update(unresolved_globals(constant, namespace))
    return missing


class LadderAfter(unittest.TestCase):
    def inputs(self):
        result = {name: "a" * 64 for name in driver.STAGE_INPUT_NAMES}
        result.update(driver.PATCH_HASHES)
        result[driver.BEFORE_EVIDENCE_PATH] = driver.BEFORE_EVIDENCE_HASH
        result[driver.V1_FAILURE_EVIDENCE_PATH] = driver.V1_FAILURE_EVIDENCE_HASH
        result[driver.V2_FAILURE_EVIDENCE_PATH] = driver.V2_FAILURE_EVIDENCE_HASH
        return result

    def log(self, kind, *, full=False):
        names = driver.DOMAIN_NAMES if kind == "domain" else driver.CORE_NAMES
        total = 521 if kind == "domain" else 630
        passed = total if full else len(names)
        filtered = 519 if kind == "domain" else 629
        return (
            "test result: ok. %d passed; 0 failed; 0 ignored; "
            "0 measured; %d filtered out\n"
            % (passed, 0 if full else filtered)
        )

    def boundary(self):
        case = driver.CAPTURE_CASES[2]
        name = case["id"]
        fixture = b"sealed fixture\n"
        source = ('prints("MATH_BEGIN ' + name + '")\n').encode() + fixture \
            + ('\nprints("MATH_END ' + name + '")\nquit\n').encode()
        stdout = ("MATH_BEGIN " + name + "\n").encode() \
            + b"LADDER_DATUM\n" * 11 + b"LADDER_ROW\n" * 22 \
            + ("MATH_END " + name + "\nBye.\n").encode()
        raw = dict(
            input=source,
            oracle_stdout=stdout,
            oracle_stderr=b"",
            rust_stdout=b"LADDER_ROW\n" * 10,
            rust_stderr=b"root-system arithmetic overflow\n" * 6,
        )
        hashes = {
            source: case["source_sha256"],
            fixture: case["fixture_sha256"],
            stdout: driver.FIXTURE_HASHES[driver.ORACLE_STDOUT_PATH],
            b"": driver.FIXTURE_HASHES[driver.ORACLE_STDERR_PATH],
        }
        return raw, fixture, lambda value: hashes.get(value, hashlib.sha256(value).hexdigest())

    def test_globals_bound_and_legacy_gate_imports_absent(self):
        banned = {
            "verified_gates", "artifact", "streams", "classify",
            "math_overload_command_verified", "math_type_equivalence",
            "weyl_context_contract", "stage_merged_math",
        }
        self.assertTrue(banned.isdisjoint(vars(driver)))
        for module in (driver, stager):
            for value in vars(module).values():
                if isinstance(value, types.FunctionType) and value.__module__ == module.__name__:
                    target = inspect.unwrap(value)
                    self.assertEqual(
                        unresolved_globals(target.__code__, target.__globals__),
                        set(),
                        value.__name__,
                    )

    def test_both_exact_focused_after_results_pass(self):
        self.assertEqual(driver.passing_log(self.log("domain"), "domain")["passed"], 2)
        self.assertEqual(driver.passing_log(self.log("core"), "core")["passed"], 1)

    def test_setup_or_failed_result_is_not_after_evidence(self):
        for raw in (
                "compile error",
                self.log("domain").replace("test result: ok", "test result: FAILED"),
                self.log("domain") + "root-system arithmetic overflow"):
            with self.assertRaises(ValueError):
                driver.passing_log(raw, "domain")
        caught_panic = self.log("domain", full=True) + (
            "thread 'rep_table::tests::poisoned_kl_cache_returns_a_stable_error' "
            "panicked at crates/atlas-real-group/src/rep_table.rs:2202:52:\n"
            "poison KL cache\n"
        )
        with self.assertRaises(ValueError):
            driver.passing_log(caught_panic, "domain", full=True)

    def test_pass_counts_filtered_counts_and_ignored_are_required(self):
        for old, new in (("519 filtered", "518 filtered"),
                         ("0 ignored", "1 ignored"),
                         ("2 passed", "1 passed")):
            with self.assertRaises(ValueError):
                driver.passing_log(self.log("domain").replace(old, new), "domain")

    def test_full_lib_results_are_exact_and_unfiltered(self):
        self.assertEqual(
            driver.passing_log(self.log("domain", full=True), "domain", full=True)["passed"],
            521,
        )
        self.assertEqual(
            driver.passing_log(self.log("core", full=True), "core", full=True)["passed"],
            630,
        )
        with self.assertRaises(ValueError):
            driver.passing_log(self.log("core"), "core", full=True)
        boundary = b"\n#[cfg(test)]\nmod tests {"
        parent = b"old production" + boundary + b"old tests"
        tests_only = b"old production" + boundary + b"new tests"
        final = b"new production" + boundary + b"new tests"
        self.assertTrue(driver.production_unchanged(parent, tests_only))
        self.assertTrue(driver.production_patch_only(parent, tests_only, final))
        self.assertFalse(driver.production_patch_only(
            parent, tests_only, b"new production" + boundary + b"changed tests"))

    def test_full_suite_capture_isolated_and_environment_scrubbed(self):
        inventory, focused, full = driver.crate_test_commands(
            "atlas-real-group", "ladder_coordinate_boundary_")
        base = ["cargo", "test", "--offline", "--locked", "-p",
                "atlas-real-group", "--lib"]
        self.assertEqual(inventory, base + ["--", "--list"])
        self.assertEqual(
            focused,
            base + ["ladder_coordinate_boundary_", "--", "--nocapture",
                    "--test-threads=1"],
        )
        self.assertEqual(full, base + ["--", "--test-threads=1"])
        self.assertIn("--nocapture", focused)
        self.assertNotIn("--nocapture", full)

        inherited = {
            "KEEP_ME": "yes",
            "PATH": "/toolchain/bin",
            "LD_LIBRARY_PATH": "/toolchain/lib",
            "RUST_TEST_NOCAPTURE": "1",
            "ATLAS_LEGACY": "forbidden",
            "RUSTFLAGS": "forbidden",
        }
        original = dict(inherited)
        env = driver.command_environment(inherited, Path("/ephemeral"))
        self.assertEqual(inherited, original)
        self.assertEqual(env["KEEP_ME"], "yes")
        self.assertEqual(env["PATH"], "/toolchain/bin")
        self.assertEqual(env["LD_LIBRARY_PATH"], "/toolchain/lib")
        self.assertNotIn("RUST_TEST_NOCAPTURE", env)
        self.assertNotIn("ATLAS_LEGACY", env)
        self.assertNotIn("RUSTFLAGS", env)
        self.assertEqual(env["CARGO_TARGET_DIR"], "/ephemeral/target")

    def test_boundary_fixture_is_derived_only_from_exact_sealed_streams(self):
        raw, fixture, fake_hash = self.boundary()
        with patch.object(driver, "sha256_bytes", side_effect=fake_hash):
            result = driver.boundary_fixture(raw)
            self.assertEqual(result[driver.FIXTURE_PATH], fixture)
            for key, replacement in (
                    ("input", b"unwrapped"),
                    ("oracle_stdout", raw["oracle_stdout"].replace(b"LADDER_ROW", b"MISSING", 1)),
                    ("rust_stdout", b"LADDER_ROW\n" * 9),
                    ("rust_stderr", b"root-system arithmetic overflow\n" * 5)):
                changed = dict(raw)
                changed[key] = replacement
                with self.assertRaises(ValueError):
                    driver.boundary_fixture(changed)

    def test_pin_is_exact_path_free_after_v3_shape(self):
        inputs = self.inputs()
        reference = dict(driver.PARENT_SEAL_REFERENCE)
        retired = dict(driver.RETIRED_STAGER_BUNDLE_REFERENCE)
        pin = dict(schema=driver.PIN_SCHEMA, inputs=inputs,
                   parent_seal_object=reference,
                   retired_stager_object=retired,
                   lifecycle=copy.deepcopy(driver.LIFECYCLE),
                   ladder_boundary_before_evidence=driver.BEFORE_REFERENCE,
                   ladder_boundary_after_v1_failure=driver.V1_FAILURE_REFERENCE,
                   ladder_boundary_after_v2_failure=driver.V2_FAILURE_REFERENCE,
                   ladder_boundary_test_hashes=driver.TEST_HASHES,
                   ladder_boundary_final_hashes=driver.FINAL_HASHES,
                   ladder_boundary_patch_hashes=driver.PATCH_HASHES)
        self.assertEqual(stager.LIFECYCLE, driver.LIFECYCLE)
        self.assertEqual(stager.prepared_pin(reference, inputs), pin)
        self.assertIs(driver.validate_pin(pin, inputs), pin)
        isolated = stager.prepared_pin(reference, inputs)
        self.assertIsNot(isolated["inputs"], inputs)
        self.assertIsNot(isolated["lifecycle"], stager.LIFECYCLE)
        for key, source in (
                ("parent_seal_object", stager.PARENT_SEAL_REFERENCE),
                ("retired_stager_object", stager.RETIRED_STAGER_BUNDLE_REFERENCE),
                ("ladder_boundary_before_evidence", stager.BEFORE_REFERENCE),
                ("ladder_boundary_after_v1_failure", stager.V1_FAILURE_REFERENCE),
                ("ladder_boundary_after_v2_failure", stager.V2_FAILURE_REFERENCE)):
            self.assertIsNot(isolated[key], source)
        isolated["inputs"][driver.PRODUCTION_PATCH] = "f" * 64
        isolated["lifecycle"]["changed_input_reasons"].append("tampered")
        self.assertEqual(inputs[driver.PRODUCTION_PATCH], driver.PATCH_HASHES[
            driver.PRODUCTION_PATCH])
        self.assertEqual(stager.LIFECYCLE, driver.LIFECYCLE)
        changes = (
            lambda value: value.update(schema="atlas-ladder-boundary-after-pin-v0"),
            lambda value: value.update(unexpected_legacy_reference={"path": "/legacy"}),
            lambda value: value["parent_seal_object"].update(path="/legacy"),
            lambda value: value["parent_seal_object"].update(role="legacy-evidence"),
        )
        for change in changes:
            changed = dict(pin, parent_seal_object=dict(reference))
            change(changed)
            with self.assertRaises(ValueError):
                driver.validate_pin(changed, inputs)
        changed_lifecycle = copy.deepcopy(pin)
        changed_lifecycle["lifecycle"]["retention_class"] = "PERMANENT"
        with self.assertRaises(ValueError):
            driver.validate_pin(changed_lifecycle, inputs)
        with self.assertRaises(ValueError):
            stager.validate_pin(changed_lifecycle)
        for key, value in (
                ("schema", "atlas-campaign-blob-v2"),
                ("role", "other-parent-seal"),
                ("sha256", "c" * 64),
                ("bytes", reference["bytes"] + 1)):
            changed_reference = dict(reference)
            changed_reference[key] = value
            changed = dict(pin, parent_seal_object=changed_reference)
            with self.assertRaises(ValueError):
                driver.validate_pin(changed, inputs)
            with self.assertRaises(ValueError):
                stager.validate_pin(changed)
            with self.assertRaises(ValueError):
                stager.prepared_pin(changed_reference, inputs)
        changed_inputs = dict(inputs)
        changed_inputs[driver.PRODUCTION_PATCH] = "c" * 64
        changed = dict(pin, inputs=changed_inputs)
        with self.assertRaises(ValueError):
            driver.validate_pin(changed, changed_inputs)
        with self.assertRaises(ValueError):
            stager.validate_pin(changed)
        evidence = {
            "status": driver.BEFORE_REFERENCE["status"],
            "parent_seal_object": reference,
            "accounting": {"job": "3873400", "state": "COMPLETED", "exit_code": "0:0"},
            "report": {
                "sha256": driver.BEFORE_REFERENCE["report_sha256"],
                "status": "LADDER_BOUNDARY_THREE_FAILURES_CONFIRMED",
            },
            "checkers": {"passed": 65},
            "inspection": {"production_unchanged": True},
        }
        with patch.object(stager, "read", return_value=evidence):
            self.assertIs(
                stager.validate_before_evidence(Path("/sealed"), inputs), evidence)
        changed_evidence = dict(evidence, report={"sha256": "0" * 64})
        with patch.object(stager, "read", return_value=changed_evidence):
            with self.assertRaises(ValueError):
                stager.validate_before_evidence(Path("/sealed"), inputs)
        changed_evidence = dict(
            evidence, parent_seal_object=dict(reference, sha256="c" * 64))
        with patch.object(stager, "read", return_value=changed_evidence):
            with self.assertRaises(ValueError):
                stager.validate_before_evidence(Path("/sealed"), inputs)

    def test_after_v3_binds_and_preflights_retired_stager_cas_reference(self):
        inputs = self.inputs()
        parent = dict(driver.PARENT_SEAL_REFERENCE)
        retired = dict(driver.RETIRED_STAGER_BUNDLE_REFERENCE)
        self.assertEqual(stager.RETIRED_STAGER_BUNDLE_REFERENCE, retired)
        pin = stager.prepared_pin(parent, inputs)
        self.assertEqual(pin["retired_stager_object"], retired)
        self.assertIs(driver.validate_pin(pin, inputs), pin)
        with patch.object(stager, "verify_blob", return_value=Path("/cas/object")) as verify:
            self.assertEqual(
                stager.validate_retired_stager_object(Path("/campaign"), retired),
                retired,
            )
        verify.assert_called_once_with(Path("/campaign"), retired)
        for key, value in (
                ("schema", "atlas-campaign-blob-v2"),
                ("role", "legacy-evidence"),
                ("sha256", "c" * 64),
                ("bytes", retired["bytes"] + 1)):
            changed_reference = dict(retired)
            changed_reference[key] = value
            changed_pin = dict(pin, retired_stager_object=changed_reference)
            with self.assertRaises(ValueError):
                driver.validate_pin(changed_pin, inputs)
            with self.assertRaises(ValueError):
                stager.validate_pin(changed_pin)
            with patch.object(stager, "verify_blob") as verify:
                with self.assertRaises(ValueError):
                    stager.validate_retired_stager_object(
                        Path("/campaign"), changed_reference)
                verify.assert_not_called()

        failure_evidence = {
            "schema": "atlas-ladder-boundary-after-inspection-v1",
            "status": driver.V1_FAILURE_REFERENCE["status"],
            "accounting": {
                "job": driver.V1_FAILURE_REFERENCE["job"],
                "state": "FAILED",
                "exit_code": "1:0",
            },
            "artifacts": {"report": {
                "file_sha256": driver.V1_FAILURE_REFERENCE["report_sha256"],
            }},
            "failure": {
                "kind": "FULL_STAGER_FILENAME_INVENTORY_MISMATCH",
                "parent_source_stagers": 0,
                "actual_reconstructed_stagers": 3,
                "missing_historical_stagers": 67,
                "expected_frozen_stagers": 70,
            },
            "math": {
                "after_regressions": "NOT_REACHED",
                "full_suites": "NOT_REACHED",
                "inventories": "NOT_REACHED",
            },
        }
        with patch.object(stager, "read", return_value=failure_evidence):
            self.assertIs(
                stager.validate_v1_failure_evidence(Path("/sealed"), inputs),
                failure_evidence,
            )
        changed_failure = dict(
            failure_evidence,
            failure=dict(failure_evidence["failure"], missing_historical_stagers=66),
        )
        with patch.object(stager, "read", return_value=changed_failure):
            with self.assertRaises(ValueError):
                stager.validate_v1_failure_evidence(Path("/sealed"), inputs)

    def test_v2_harness_failure_is_the_exact_direct_predecessor(self):
        inputs = self.inputs()
        executed = [
            {"name": name, "exit_status": 0}
            for name in (
                "test_campaign_workspace", "test_campaign_source",
                "test_campaign_blob", "test_local_worktree_guard",
                "test_stager_allowlist", "test_math_acceptance_index",
                "test_math_ladder_boundary_after", "rustc", "cargo",
                "full-stager-inventory", "domain-inventory",
                "domain-focused", "domain-all",
            )
        ]
        evidence = {
            "schema": "atlas-ladder-boundary-after-inspection-v2",
            "status": driver.V2_FAILURE_REFERENCE["status"],
            "accounting": {
                "job": driver.V2_FAILURE_REFERENCE["job"],
                "state": "FAILED",
                "exit_code": "1:0",
            },
            "artifacts": {
                "overrides_manifest_sha256": (
                    "b3e4fa16cdb9e86aba36d4bdd5723401"
                    "a32b0c07ef21cdce29165dd938daa699"
                ),
                "report": {
                    "file_sha256": driver.V2_FAILURE_REFERENCE["report_sha256"],
                    "schema": "atlas-ladder-boundary-after-v2",
                    "status": "HARNESS_FAILURE",
                    "command_count": 13,
                },
            },
            "commands": {
                "checker_tests_passed": 93,
                "successful_prefix_count": 13,
                "executed": executed,
                "not_reached": [
                    "core-inventory", "core-focused", "core-all",
                    "final-preflight", "final-integrity",
                ],
            },
            "failure": {
                "kind": "EXPECTED_CAUGHT_PANIC_REJECTED_BY_LOG_PARSER",
                "exception": "ValueError: exact passing ladder AFTER result required",
                "trigger": {
                    "file": "crates/atlas-real-group/src/rep_table.rs",
                    "test": (
                        "rep_table::tests::"
                        "poisoned_kl_cache_returns_a_stable_error"
                    ),
                },
            },
            "inspection": {
                "command_artifacts": 26,
                "command_artifacts_rehashed": True,
                "ephemeral_workspace_absent": True,
                "top_level_atlas_rust_directories": 318,
            },
            "math": {
                "core": "NOT_REACHED",
                "domain_focused": {
                    "failed": 0, "filtered_out": 519,
                    "ignored": 0, "passed": 2,
                },
                "domain_full": {
                    "failed": 0, "filtered_out": 0,
                    "ignored": 0, "passed": 521,
                },
                "domain_inventory": 521,
                "final_status": "NOT_ACCEPTED",
            },
            "predecessor": {
                "evidence_path": driver.V1_FAILURE_EVIDENCE_PATH,
                "evidence_sha256": driver.V1_FAILURE_EVIDENCE_HASH,
                "job": driver.V1_FAILURE_REFERENCE["job"],
                "report_sha256": driver.V1_FAILURE_REFERENCE["report_sha256"],
                "status": driver.V1_FAILURE_REFERENCE["status"],
            },
        }
        with patch.object(stager, "read", return_value=evidence):
            self.assertIs(
                stager.validate_v2_failure_evidence(Path("/sealed"), inputs),
                evidence,
            )
        changed_inputs = dict(inputs)
        changed_inputs[driver.V2_FAILURE_EVIDENCE_PATH] = "0" * 64
        with patch.object(stager, "read", return_value=evidence):
            with self.assertRaises(ValueError):
                stager.validate_v2_failure_evidence(
                    Path("/sealed"), changed_inputs)
        mutations = (
            lambda value: value.update(
                schema="atlas-ladder-boundary-after-inspection-v0"),
            lambda value: value["accounting"].update(job="0"),
            lambda value: value["accounting"].update(state="COMPLETED"),
            lambda value: value["accounting"].update(exit_code="0:0"),
            lambda value: value.update(status="LADDER_BOUNDARY_AFTER_GATES_PASS"),
            lambda value: value["artifacts"]["report"].update(
                file_sha256="0" * 64),
            lambda value: value["artifacts"]["report"].update(
                status="LADDER_BOUNDARY_AFTER_GATES_PASS"),
            lambda value: value["artifacts"]["report"].update(schema="wrong"),
            lambda value: value["artifacts"]["report"].update(command_count=12),
            lambda value: value["artifacts"].update(
                overrides_manifest_sha256="0" * 64),
            lambda value: value["commands"].update(checker_tests_passed=92),
            lambda value: value["commands"].update(successful_prefix_count=12),
            lambda value: value["commands"]["executed"][0].update(name="wrong"),
            lambda value: value["commands"]["executed"].pop(),
            lambda value: value["commands"]["executed"].__setitem__(
                0, "not-an-object"),
            lambda value: value["commands"]["executed"][-1].update(exit_status=1),
            lambda value: value["commands"]["not_reached"].pop(),
            lambda value: value["failure"].update(kind="MATHEMATICAL_FAILURE"),
            lambda value: value["failure"].update(exception="ValueError: changed"),
            lambda value: value["failure"]["trigger"].update(test="wrong"),
            lambda value: value["inspection"].update(command_artifacts=25),
            lambda value: value["inspection"].update(
                command_artifacts_rehashed=False),
            lambda value: value["inspection"].update(
                ephemeral_workspace_absent=False),
            lambda value: value["inspection"].update(
                top_level_atlas_rust_directories=317),
            lambda value: value["math"]["domain_full"].update(passed=520),
            lambda value: value["math"].update(core={"passed": 630}),
            lambda value: value["math"].update(final_status="ACCEPTED"),
            lambda value: value["predecessor"].update(job="0"),
            lambda value: value["predecessor"].update(evidence_path="wrong"),
            lambda value: value["predecessor"].update(
                evidence_sha256="0" * 64),
            lambda value: value["predecessor"].update(report_sha256="0" * 64),
            lambda value: value["predecessor"].update(status="wrong"),
        )
        for mutation in mutations:
            changed = copy.deepcopy(evidence)
            mutation(changed)
            with patch.object(stager, "read", return_value=changed):
                with self.assertRaises(ValueError):
                    stager.validate_v2_failure_evidence(Path("/sealed"), inputs)

    def test_inventories_must_equal_seal_baseline_plus_regressions(self):
        self.assertEqual(
            driver.verify_inventory(["old", "new"], ["old"], {"new"}, 2),
            ["old", "new"],
        )
        for inventory in (["old"], ["old", "other"], ["old", "old"]):
            with self.assertRaises(ValueError):
                driver.verify_inventory(inventory, ["old"], {"new"}, 2)

    def test_stage_is_exact_after_v3_and_retry_is_fail_closed(self):
        inputs = self.inputs()
        reference = dict(driver.PARENT_SEAL_REFERENCE)
        self.assertTrue(stager.SUBMISSION_ENABLED)
        self.assertTrue(driver.SUBMISSION_ENABLED)
        self.assertEqual(stager.STAGE_NAME, driver.STAGE_NAME)
        self.assertEqual(stager.STAGE_INPUT_NAMES, driver.STAGE_INPUT_NAMES)
        self.assertIs(driver.frozen_stage_inputs, stager.frozen_stage_inputs)
        self.assertEqual(stager.BEFORE_REFERENCE, driver.BEFORE_REFERENCE)
        self.assertEqual(stager.V1_FAILURE_REFERENCE, driver.V1_FAILURE_REFERENCE)
        self.assertEqual(stager.V1_FAILURE_EVIDENCE_PATH, driver.V1_FAILURE_EVIDENCE_PATH)
        self.assertEqual(stager.V1_FAILURE_EVIDENCE_HASH, driver.V1_FAILURE_EVIDENCE_HASH)
        self.assertEqual(stager.V2_FAILURE_REFERENCE, driver.V2_FAILURE_REFERENCE)
        self.assertEqual(stager.V2_FAILURE_EVIDENCE_PATH, driver.V2_FAILURE_EVIDENCE_PATH)
        self.assertEqual(stager.V2_FAILURE_EVIDENCE_HASH, driver.V2_FAILURE_EVIDENCE_HASH)
        self.assertEqual(stager.PARENT_SEAL_REFERENCE, driver.PARENT_SEAL_REFERENCE)
        self.assertEqual(
            stager.RETIRED_STAGER_BUNDLE_REFERENCE,
            driver.RETIRED_STAGER_BUNDLE_REFERENCE,
        )
        self.assertEqual(stager.CHILD_ONLY_INPUTS, driver.CHILD_ONLY_INPUTS)
        self.assertTrue(driver.CHILD_ONLY_INPUTS.isdisjoint(driver.SEALED_SHARED_INPUTS))
        self.assertTrue(driver.CHILD_ONLY_INPUTS <= driver.STAGE_INPUT_NAMES)
        self.assertEqual(
            {name for name in driver.STAGE_INPUT_NAMES
             if name.startswith("hpc/stage_")},
            {
                "hpc/stage_weyl_parent_seal.py",
                "hpc/stage_ladder_boundary_before.py",
                "hpc/stage_ladder_boundary_after.py",
            },
        )
        self.assertTrue({
            "hpc/local_worktree_guard.py",
            "hpc/test_local_worktree_guard.py",
            "docs/worktree_registry.json",
            driver.V2_FAILURE_EVIDENCE_PATH,
        } <= driver.CHILD_ONLY_INPUTS)
        main_source = inspect.getsource(driver.main)
        self.assertIn("full-stager-inventory", main_source)
        self.assertEqual(
            driver.FULL_STAGER_INVENTORY_PROGRAM,
            "import json, sys; sys.path.insert(0, sys.argv[1]); "
            "from campaign_blob import read_blob; "
            "from test_stager_allowlist import reconstructed_stage_sources, "
            "validate_historical_inventory; "
            "raw = read_blob(sys.argv[2], json.loads(sys.argv[3])); "
            "staged = reconstructed_stage_sources(raw); "
            "retired = validate_historical_inventory(staged); "
            "print('FULL_STAGE_INVENTORY_OK', len(staged), len(retired))",
        )
        self.assertEqual(main_source.count("FULL_STAGER_INVENTORY_PROGRAM"), 1)
        self.assertIn('str(root / "hpc")', main_source)
        self.assertIn("str(campaign_stage(root))", main_source)
        self.assertNotIn('str(source / "hpc")', main_source)
        self.assertLess(
            main_source.index('"full-stager-inventory"'),
            main_source.index("materialize_source_archive("),
        )
        stage_source = inspect.getsource(stager._stage_and_submit)
        for preflight_name in (
                "validate_before_evidence(", "validate_v1_failure_evidence(",
                "validate_v2_failure_evidence(",
                "validate_retired_stager_object(", "load_parent_seal("):
            self.assertLess(
                stage_source.index(preflight_name),
                stage_source.index("install_overrides("),
                preflight_name,
            )
        self.assertEqual(stager.CHECKER_TESTS, sum(count for _, count in driver.CHECKERS))
        self.assertEqual(stager.CHECKER_TESTS, 95)
        self.assertEqual(
            stager.seal_reference(reference["sha256"], str(reference["bytes"])),
            reference,
        )
        for sha, size in (("a" * 63, "1"), ("c" * 64, str(reference["bytes"])),
                          (reference["sha256"], str(reference["bytes"] + 1))):
            with self.assertRaises(ValueError):
                stager.seal_reference(sha, size)
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            pin = stager.prepared_pin(reference, inputs)
            self.assertEqual(stager.stage_mode(root, pin), "prepare")
            (root / stager.PIN_NAME).write_text("{}\n")
            self.assertEqual(stager.stage_mode(root, pin), "retry")
            (root / "submission-intent.json").write_text("{}\n")
            with patch.object(stager, "confirmed_record", side_effect=ValueError("unresolved")):
                with self.assertRaises(ValueError):
                    stager.stage_mode(root, pin)

    def test_direct_driver_requires_compute_node_before_preflight_or_result_creation(self):
        with (patch.object(driver, "gates") as gates,
              patch.object(driver, "create_result_folder") as create_result,
              patch.dict(driver.os.environ, {}, clear=True)):
            with self.assertRaisesRegex(SystemExit, "compute nodes only"):
                driver.main()
            gates.assert_not_called()
            create_result.assert_not_called()

    def test_frozen_manifest_includes_docs_registry_and_rejects_drift(self):
        repository = Path(stager.__file__).resolve().parents[1]
        registry_name = "docs/worktree_registry.json"
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder) / "stage"
            expected = {}
            for name in sorted(driver.STAGE_INPUT_NAMES):
                source = repository / name
                self.assertTrue(source.is_file(), name)
                destination = root / name
                destination.parent.mkdir(parents=True, exist_ok=True)
                destination.write_bytes(source.read_bytes())
                destination.chmod(0o444)
                expected[name] = hashlib.sha256(destination.read_bytes()).hexdigest()

            manifest = stager.frozen_stage_inputs(root)
            self.assertEqual(manifest, expected)
            registry = root / registry_name
            self.assertEqual(registry.stat().st_nlink, 1)
            self.assertEqual(registry.stat().st_mode & 0o777, 0o444)
            pin = stager.prepared_pin(driver.PARENT_SEAL_REFERENCE, manifest)
            self.assertIs(driver.validate_pin(pin, manifest), pin)

            registry.chmod(0o600)
            registry.write_bytes(b'{"changed":true}\n')
            registry.chmod(0o444)
            changed = stager.frozen_stage_inputs(root)
            with self.assertRaises(ValueError):
                driver.validate_pin(pin, changed)

            source_registry = repository / registry_name
            registry.chmod(0o600)
            registry.write_bytes(source_registry.read_bytes())
            registry.chmod(0o644)
            with self.assertRaises(ValueError):
                stager.frozen_stage_inputs(root)
            registry.chmod(0o444)
            hardlink = root / "worktree-registry-hardlink"
            stager.os.link(registry, hardlink)
            with self.assertRaises(ValueError):
                stager.frozen_stage_inputs(root)
            hardlink.unlink()
            self.assertEqual(registry.stat().st_nlink, 1)
            registry.unlink()
            registry.symlink_to(source_registry)
            with self.assertRaises(ValueError):
                stager.frozen_stage_inputs(root)

            registry.unlink()
            missing = stager.frozen_stage_inputs(root)
            self.assertEqual(set(missing), driver.STAGE_INPUT_NAMES - {registry_name})
            missing_pin = dict(pin, inputs=missing)
            with self.assertRaises(ValueError):
                driver.validate_pin(missing_pin, missing)

    def test_json_inputs_use_stable_sha_pinned_reads(self):
        with tempfile.TemporaryDirectory() as folder:
            path = Path(folder) / "state.json"
            path.write_text('{"value": 1}\n')
            wanted = hashlib.sha256(path.read_bytes()).hexdigest()
            self.assertEqual(stager.read(path, wanted), {"value": 1})
            path.write_text('{"value": 2}\n')
            with self.assertRaises(ValueError):
                stager.read(path, wanted)
            target = Path(folder) / "target.json"
            target.write_text('{"value": 1}\n')
            link = Path(folder) / "link.json"
            link.symlink_to(target)
            with self.assertRaises(ValueError):
                stager.read(link, hashlib.sha256(target.read_bytes()).hexdigest())

            stage = Path(folder) / "stage"
            staged_hpc = stage / "hpc"
            staged_hpc.mkdir(parents=True)
            frozen = staged_hpc / "input.py"
            frozen.write_bytes(b"frozen input\n")
            frozen.chmod(0o444)
            expected = {"hpc/input.py": hashlib.sha256(frozen.read_bytes()).hexdigest()}
            self.assertEqual(stager.frozen_stage_inputs(stage), expected)
            frozen.chmod(0o644)
            with self.assertRaises(ValueError):
                stager.frozen_stage_inputs(stage)
            frozen.chmod(0o444)
            outside_link = Path(folder) / "outside-hardlink"
            stager.os.link(frozen, outside_link)
            with self.assertRaises(ValueError):
                stager.frozen_stage_inputs(stage)
            outside_link.unlink()
            frozen.unlink()
            frozen.symlink_to(target)
            with self.assertRaises(ValueError):
                stager.frozen_stage_inputs(stage)

        inputs = self.inputs()
        reference = dict(driver.PARENT_SEAL_REFERENCE)
        pin = stager.prepared_pin(reference, inputs)
        root = Path("/not-present") / driver.STAGE_NAME
        pin_sha = stager.saved_json_sha(pin)
        seal = {"migration_inputs": inputs}
        job = "12345"
        record = dict(
            stage=str(root.resolve()), script=driver.SBATCH,
            queue_before=[], status="SUBMITTED", max_outstanding=10, job=job,
            pin_sha256=pin_sha,
        )
        receipt = stager.submission_receipt(record, pin)
        self.assertEqual(receipt["lifecycle"], driver.LIFECYCLE)
        self.assertIsNot(receipt["queue_before"], record["queue_before"])
        self.assertIsNot(receipt["lifecycle"], pin["lifecycle"])
        for key in (
                "before_evidence", "predecessor_failure", "earlier_failure",
                "parent_seal_object", "retired_stager_object"):
            self.assertIsNot(receipt[key], pin[{
                "before_evidence": "ladder_boundary_before_evidence",
                "predecessor_failure": "ladder_boundary_after_v2_failure",
                "earlier_failure": "ladder_boundary_after_v1_failure",
                "parent_seal_object": "parent_seal_object",
                "retired_stager_object": "retired_stager_object",
            }[key]])
        changed_lifecycle = copy.deepcopy(pin)
        changed_lifecycle["lifecycle"]["stage"] = "other-stage"
        with self.assertRaises(ValueError):
            stager.submission_receipt(record, changed_lifecycle)
        for key, value in (
                ("status", "SUBMISSION_INTENT_NOT_CONFIRMED"),
                ("script", "hpc/other.sbatch"),
                ("stage", "relative/" + driver.STAGE_NAME),
                ("stage", "/not-present/other-stage"),
                ("max_outstanding", True),
                ("job", "not-a-job"),
                ("queue_before", [job]),
                ("pin_sha256", "f" * 64)):
            changed_record = copy.deepcopy(record)
            changed_record[key] = value
            with self.subTest(record_field=key, value=value), self.assertRaises(ValueError):
                stager.submission_receipt(changed_record, pin)
        changed_receipt = copy.deepcopy(receipt)
        changed_receipt["queue_before"].append("99999")
        changed_receipt["lifecycle"]["retention_class"] = "PERMANENT"
        self.assertEqual(record["queue_before"], [])
        self.assertEqual(pin["lifecycle"], driver.LIFECYCLE)
        changed_stage = dict(
            record, stage="/other/campaign/stages/" + driver.STAGE_NAME)
        with self.assertRaises(ValueError):
            stager.submission_receipt(changed_stage, pin, root=root)

        @contextmanager
        def ledger_lock(_directory, _name):
            yield

        def json_state(path, wanted=None):
            path = Path(path)
            if path.name == driver.PIN_NAME:
                self.assertEqual(wanted, pin_sha)
                return pin
            if path.name == ".atlas-progressive-submit.json":
                self.assertIsNone(wanted)
                return [record]
            if path.name == "submission-intent.json":
                self.assertIsNone(wanted)
                return record
            if path.name == "submission.json":
                self.assertIsNone(wanted)
                return receipt
            self.fail("unexpected JSON state read: " + str(path))

        with (patch.dict(driver.os.environ, {
                  "LADDER_BOUNDARY_AFTER_PIN_SHA256": pin_sha,
                  "LADDER_BOUNDARY_AFTER_SPOOL": "/spool/script",
                  "SLURM_JOB_ID": job,
              }),
              patch.object(driver, "campaign_stage", return_value=Path("/campaign")),
              patch.object(driver, "stage_inputs", return_value=inputs),
              patch.object(driver, "existing_exclusive_lock", side_effect=ledger_lock),
              patch.object(driver, "read_json_file", side_effect=json_state) as stable_read,
              patch.object(driver, "digest", return_value=inputs[driver.SBATCH]),
              patch.object(driver, "load_parent_seal", return_value=seal),
              patch.object(driver, "read_blob", return_value=b"retired-stagers") as read_blob,
              patch.object(driver, "boundary_bytes", return_value={}),
              patch.object(driver, "boundary_fixture", return_value={})):
            driver.gates(root, allow_recovery=False)
        read_blob.assert_called_once_with(
            Path("/campaign"), driver.RETIRED_STAGER_BUNDLE_REFERENCE)
        stable_read.assert_has_calls([
            call(root.resolve() / driver.PIN_NAME, pin_sha),
            call(Path("/campaign/.atlas-progressive-submit.json")),
            call(root.resolve() / "submission-intent.json"),
            call(root.resolve() / "submission.json"),
        ])
        other = dict(record, stage="/campaign/stages/other", job="67890")
        for changed_history, changed_intent, changed_job in (
                ([], {}, job),
                ([dict(record, status="SUBMISSION_INTENT_NOT_CONFIRMED")], record, job),
                ([dict(record, pin_sha256="d" * 64)], record, job),
                ([record, dict(other, stage=str(root.resolve()))], record, job),
                ([record, dict(other, job=job)], record, job),
                ([record, dict(other, max_outstanding=9)], record, job),
                ([record], dict(record, job="999"), job),
                ([record], record, "999")):
            with self.assertRaises(ValueError):
                driver.validate_submission_state(
                    root.resolve(), inputs, changed_history, changed_intent,
                    changed_job, pin_sha)

    def test_submission_delegates_to_shared_pinned_boundary(self):
        root = Path("/campaign/stages") / stager.STAGE_NAME
        pin_sha = "0" * 64
        record = {"status": "SUBMITTED"}
        with (patch.object(stager, "SUBMISSION_ENABLED", False),
              patch.object(stager, "submit_one") as shared):
            with self.assertRaises(ValueError):
                stager.submit_pinned(root, pin_sha)
            shared.assert_not_called()
        with (patch.object(stager, "SUBMISSION_ENABLED", True),
              patch.object(stager, "submit_one", return_value=record) as shared):
            self.assertIs(stager.submit_pinned(root, pin_sha), record)
        args, kwargs = shared.call_args
        self.assertEqual(args[:2], (root, stager.SBATCH))
        self.assertEqual(kwargs, {"pin_sha256": pin_sha})
        self.assertEqual(args[2]["LADDER_BOUNDARY_AFTER_PIN_SHA256"], pin_sha)

    def test_stage_lock_revalidates_and_symlink_destination_fails_closed(self):
        events = []

        @contextmanager
        def locked(_root, name):
            self.assertEqual(name, stager.STAGE_LOCK)
            events.append("lock-enter")
            yield
            events.append("lock-exit")

        def staged(_root, _reference, _sha, validate_only=False,
                   existing_stage_lock=False):
            events.append("validate" if validate_only else "mutate")
            self.assertFalse(existing_stage_lock)
            return False if validate_only else None

        with (patch.object(stager, "SUBMISSION_ENABLED", True),
              patch.object(stager, "exclusive_lock", side_effect=locked),
              patch.object(stager, "_stage_and_submit", side_effect=staged)):
            stager.run_enabled(Path("/stage"), {}, "a" * 64)
        self.assertEqual(events, ["validate", "lock-enter", "mutate", "lock-exit"])

        events.clear()

        def recovering(_root, _reference, _sha, validate_only=False,
                       existing_stage_lock=False):
            events.append("validate" if validate_only else "mutate")
            self.assertEqual(existing_stage_lock, not validate_only)
            return True if validate_only else None

        with (patch.object(stager, "SUBMISSION_ENABLED", True),
              patch.object(stager, "existing_lock", side_effect=locked),
              patch.object(stager, "exclusive_lock") as create_lock,
              patch.object(stager, "_stage_and_submit", side_effect=recovering)):
            stager.run_enabled(Path("/stage"), {}, "a" * 64)
        self.assertEqual(events, ["validate", "lock-enter", "mutate", "lock-exit"])
        create_lock.assert_not_called()

        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            missing = root / stager.STAGE_LOCK
            with self.assertRaises(ValueError):
                with stager.existing_lock(root, stager.STAGE_LOCK):
                    pass
            self.assertFalse(missing.exists())
            with self.assertRaises(ValueError):
                with driver.existing_exclusive_lock(root, stager.STAGE_LOCK):
                    pass
            self.assertFalse(missing.exists())

            campaign = root / "campaign"
            stage = root / "stage"
            campaign.mkdir()
            stage.mkdir()
            with self.assertRaises(ValueError):
                stager.requires_existing_stage_lock(
                    stage, campaign, prepared=False,
                    has_submission_state=False, partial_inputs={})
            campaign_lock = campaign / ".atlas-progressive-submit.lock"
            self.assertFalse(campaign_lock.exists())
            campaign_lock.write_bytes(b"")
            campaign_lock.chmod(0o600)
            self.assertFalse(stager.requires_existing_stage_lock(
                stage, campaign, prepared=False,
                has_submission_state=False, partial_inputs={}))
            with self.assertRaises(ValueError):
                stager.requires_existing_stage_lock(
                    stage, campaign, prepared=True,
                    has_submission_state=False, partial_inputs={})
            self.assertFalse((stage / stager.STAGE_LOCK).exists())

            incoming = root / ".incoming"
            incoming.mkdir()
            real = root / "real"
            real.mkdir()
            (root / "linked").symlink_to(real, target_is_directory=True)
            with self.assertRaises(ValueError):
                with stager._safe_destination_parent(root / "linked/file", incoming):
                    pass

    def test_dirfd_publication_cannot_follow_replaced_parent_and_cleans_nlink_two(self):
        raw = b"sealed ladder input\n"
        wanted = hashlib.sha256(raw).hexdigest()
        with tempfile.TemporaryDirectory() as folder:
            base = Path(folder)
            stage = base / "stage"
            outside = base / "outside"
            stage.mkdir()
            outside.mkdir()
            incoming = stage / ".incoming"
            stager.clean_incoming(incoming)
            destination = stage / "hpc/input.py"
            with stager._safe_destination_parent(destination, incoming) as descriptors:
                _, incoming_descriptor, destination_descriptor, name = descriptors
                (stage / "hpc").rename(stage / "held-parent")
                (stage / "hpc").symlink_to(outside, target_is_directory=True)
                stager._atomic_install(
                    raw, wanted, incoming_descriptor, destination_descriptor, name)
            self.assertFalse((outside / "input.py").exists())
            installed = stage / "held-parent/input.py"
            self.assertEqual(installed.read_bytes(), raw)
            self.assertEqual(installed.stat().st_mode & 0o777, 0o444)
            (stage / "hpc").unlink()
            (stage / "held-parent").rename(stage / "hpc")
            installed = stage / "hpc/input.py"
            installed.chmod(0o644)
            with stager._safe_destination_parent(installed, incoming) as descriptors:
                _, incoming_descriptor, destination_descriptor, name = descriptors
                with self.assertRaises(ValueError):
                    stager._atomic_install(
                        raw, wanted, incoming_descriptor, destination_descriptor, name)
            installed.chmod(0o444)

            interrupted = incoming / (".copy-" + "a" * 32)
            interrupted.write_bytes(raw)
            retained = stage / "interrupted-published"
            stager.os.link(interrupted, retained)
            self.assertEqual(interrupted.stat().st_nlink, 2)
            with self.assertRaises(ValueError):
                stager.clean_incoming(incoming)
            self.assertTrue(interrupted.exists())
            self.assertEqual(retained.read_bytes(), raw)

            marker = outside / "marker"
            marker.write_bytes(b"outside")
            hostile = incoming / (".copy-" + "b" * 32)
            hostile.symlink_to(marker)
            with self.assertRaises(ValueError):
                stager.clean_incoming(incoming)
            self.assertEqual(marker.read_bytes(), b"outside")

            # An exact interrupted link publication is recognized without
            # mutation, selects the pre-existing lock, and is cleaned only in
            # the locked second pass before strict frozen validation.
            recovery = base / "recovery" / stager.STAGE_NAME
            recovery_incoming = recovery / ".incoming"
            recovery_hpc = recovery / "hpc"
            recovery_incoming.mkdir(parents=True)
            recovery_hpc.mkdir()
            recovery_lock = recovery / stager.STAGE_LOCK
            recovery_lock.write_bytes(b"")
            recovery_lock.chmod(0o600)
            recovery_destination = recovery_hpc / "input.py"
            recovery_destination.write_bytes(raw)
            recovery_destination.chmod(0o444)
            recovery_temporary = recovery_incoming / (".copy-" + "c" * 32)
            stager.os.link(recovery_destination, recovery_temporary)
            partial_temporary = recovery_incoming / (".copy-" + "f" * 32)
            partial_temporary.write_bytes(b"partial write")
            partial_temporary.chmod(0o600)
            prelink_temporary = recovery_incoming / (".copy-" + "a" * 32)
            prelink_temporary.write_bytes(b"complete before link")
            prelink_temporary.chmod(0o444)
            recovery_overrides = {"hpc/input.py": wanted}
            overrides_root = recovery / "overrides"
            (overrides_root / "hpc").mkdir(parents=True)
            override_source = overrides_root / "hpc/input.py"
            override_source.write_bytes(raw)
            overrides_path = overrides_root / "overrides.json"
            stager.save(overrides_path, recovery_overrides)
            overrides_sha = hashlib.sha256(overrides_path.read_bytes()).hexdigest()
            campaign = base / "campaign"
            campaign.mkdir()
            campaign_lock = campaign / ".atlas-progressive-submit.lock"
            campaign_lock.write_bytes(b"")
            campaign_lock.chmod(0o600)
            recovery_events = []
            real_existing_lock = stager.existing_lock
            real_interrupted_publications = stager.interrupted_publications

            @contextmanager
            def observed_existing_lock(directory, name):
                recovery_events.append("lock-enter")
                with real_existing_lock(directory, name):
                    yield
                recovery_events.append("lock-exit")

            def observed_publications(*args, **kwargs):
                locked_cleanup = kwargs.get("locked_cleanup", False)
                recovery_events.append("clean" if locked_cleanup else "recognize")
                result = real_interrupted_publications(*args, **kwargs)
                if result is not None:
                    self.assertEqual(result["inputs"], recovery_overrides)
                    self.assertEqual(
                        set(result["temporary_names"]),
                        {recovery_temporary.name, partial_temporary.name,
                         prelink_temporary.name},
                    )
                return result

            def submitted(_root, pin_sha):
                return dict(
                    stage=str(recovery.resolve()), script=stager.SBATCH,
                    queue_before=[], status="SUBMITTED", max_outstanding=10,
                    job="24680", pin_sha256=pin_sha,
                )

            with (patch.object(stager, "SUBMISSION_ENABLED", True),
                  patch.object(stager, "STAGE_INPUT_NAMES",
                               set(recovery_overrides)),
                  patch.object(stager, "SBATCH", "hpc/input.py"),
                  patch.object(stager, "PATCH_HASHES", {}),
                  patch.object(stager, "validate_pin"),
                  patch.object(stager, "validate_before_evidence"),
                  patch.object(stager, "validate_v1_failure_evidence"),
                  patch.object(stager, "validate_v2_failure_evidence"),
                  patch.object(stager, "validate_retired_stager_object"),
                  patch.object(stager, "SEALED_SHARED_INPUTS", set()),
                  patch.object(stager, "campaign_stage", return_value=campaign),
                  patch.object(stager, "load_parent_seal",
                               return_value={"migration_inputs": {}}),
                  patch.object(stager, "existing_lock",
                               side_effect=observed_existing_lock),
                  patch.object(stager, "exclusive_lock") as create_lock,
                  patch.object(stager, "interrupted_publications",
                               side_effect=observed_publications),
                  patch.object(stager, "submit_pinned", side_effect=submitted)):
                receipt = stager.run_enabled(
                    recovery, stager.PARENT_SEAL_REFERENCE, overrides_sha)
                self.assertEqual(receipt["job"], "24680")
            create_lock.assert_not_called()
            self.assertEqual(
                recovery_events,
                ["recognize", "lock-enter", "recognize", "clean", "lock-exit"],
            )
            self.assertFalse(recovery_temporary.exists())
            self.assertFalse(partial_temporary.exists())
            self.assertFalse(prelink_temporary.exists())
            self.assertEqual(recovery_destination.stat().st_nlink, 1)
            self.assertEqual(
                stager.frozen_stage_inputs(recovery), recovery_overrides)

            # Unsafe-mode or symlink-hostile scratch is rejected read-only.
            unpaired = base / "unsafe-mode"
            (unpaired / ".incoming").mkdir(parents=True)
            unpaired_temporary = unpaired / ".incoming" / (".copy-" + "d" * 32)
            unpaired_temporary.write_bytes(raw)
            unpaired_temporary.chmod(0o640)
            with self.assertRaises(ValueError):
                stager.interrupted_publications(unpaired, recovery_overrides)
            self.assertEqual(unpaired_temporary.read_bytes(), raw)
            self.assertEqual(unpaired_temporary.stat().st_nlink, 1)

            hostile = base / "hostile"
            (hostile / ".incoming").mkdir(parents=True)
            (hostile / "hpc").mkdir()
            hostile_temporary = hostile / ".incoming" / (".copy-" + "e" * 32)
            hostile_temporary.write_bytes(raw)
            hostile_temporary.chmod(0o444)
            hostile_outside = base / "hostile-outside-link"
            stager.os.link(hostile_temporary, hostile_outside)
            hostile_destination = hostile / "hpc/input.py"
            hostile_destination.symlink_to(marker)
            with self.assertRaises(ValueError):
                stager.interrupted_publications(hostile, recovery_overrides)
            self.assertTrue(hostile_temporary.exists())
            self.assertEqual(hostile_temporary.stat().st_nlink, 2)
            self.assertTrue(hostile_destination.is_symlink())

    def test_final_publication_faults_commit_only_failure_reports(self):
        passing_status = "LADDER_BOUNDARY_AFTER_GATES_PASS"
        with tempfile.TemporaryDirectory() as folder:
            base = Path(folder)
            real_replace = driver.os.replace
            real_stage = driver._stage_output
            for operation, target in (
                    ("write", "report.sha256"), ("write", "report.json"),
                    ("publish", "report.sha256"), ("publish", "report.json")):
                out = base / (operation + "-" + target)
                out.mkdir()
                stager.save(out / "report.json", {"status": "RUNNING"})
                report = {"schema": "test", "status": passing_status}
                faulted = []

                def faulty_replace(source, destination):
                    if (operation == "publish" and Path(destination).name == target
                            and not faulted):
                        faulted.append(target)
                        raise OSError("injected publication fault")
                    return real_replace(source, destination)

                def faulty_stage(path, raw):
                    if (operation == "write" and Path(path).name == target
                            and not faulted):
                        faulted.append(target)
                        raise OSError("injected publication write fault")
                    return real_stage(path, raw)

                with (patch.object(driver.os, "replace", side_effect=faulty_replace),
                      patch.object(driver, "_stage_output", side_effect=faulty_stage)):
                    self.assertFalse(driver.publish_outcome(out, report))
                self.assertEqual(faulted, [target])
                published = stager.read_json_file(out / "report.json")
                self.assertEqual(published["status"], "HARNESS_FAILURE")
                self.assertNotEqual(published["status"], passing_status)
                self.assertEqual(
                    (out / "report.sha256").read_text().strip(),
                    hashlib.sha256((out / "report.json").read_bytes()).hexdigest(),
                )

    def test_signal_reaps_active_group_and_publishes_harness_failure(self):
        class ActiveProcess:
            pid = 424242

            def __init__(self):
                self.waits = []
                self.reaped = False

            def poll(self):
                return -9 if self.reaped else None

            def wait(self, timeout=None):
                self.waits.append(timeout)
                if len(self.waits) == 1:
                    raise driver.subprocess.TimeoutExpired("active", timeout)
                self.reaped = True
                return -9

        captured = {}
        process = ActiveProcess()

        def install_handler(signum, handler):
            captured[signum] = handler

        def interrupted_command(_out, _name, _argv, _cwd, _env, active):
            active["process"] = process
            captured[driver.signal.SIGTERM](driver.signal.SIGTERM, None)

        @contextmanager
        def disposable_workspace(out, _name):
            with tempfile.TemporaryDirectory(dir=out.parent) as folder:
                yield Path(folder)

        self.assertIn(
            "start_new_session=True",
            inspect.getsource(driver.command).replace(" ", ""),
        )
        with tempfile.TemporaryDirectory() as folder:
            base = Path(folder)
            out = base / "result"
            out.mkdir()
            pin = stager.prepared_pin(
                driver.PARENT_SEAL_REFERENCE, self.inputs())
            preflight = (
                pin, {"closure": {}, "source_file_count": 0}, {}, {}, {}, "a" * 64,
            )
            with (patch.dict(driver.os.environ, {"SLURM_JOB_ID": "12345"}, clear=True),
                  patch.object(driver, "gates", return_value=preflight),
                  patch.object(driver, "create_result_folder", return_value=out),
                  patch.object(driver, "ephemeral_job_workspace",
                               side_effect=disposable_workspace),
                  patch.object(driver.signal, "signal", side_effect=install_handler),
                  patch.object(driver, "command", side_effect=interrupted_command),
                  patch.object(driver.os, "killpg") as kill_group):
                self.assertEqual(driver.main(), 1)
            self.assertEqual(
                set(captured), {driver.signal.SIGTERM, driver.signal.SIGINT})
            self.assertTrue(process.reaped)
            self.assertEqual(process.waits, [driver.COMMAND_KILL_AFTER_SECONDS,
                                             driver.COMMAND_KILL_AFTER_SECONDS])
            self.assertEqual(
                kill_group.call_args_list,
                [call(process.pid, driver.signal.SIGTERM),
                 call(process.pid, driver.signal.SIGKILL)],
            )
            published = stager.read_json_file(out / "report.json")
            self.assertEqual(published["status"], "HARNESS_FAILURE")
            self.assertIn("received signal", published["error"])
            self.assertEqual(
                (out / "report.sha256").read_text().strip(),
                hashlib.sha256((out / "report.json").read_bytes()).hexdigest(),
            )

    def test_partial_confirmation_recovery_is_exact_and_final_validation_does_not_heal(self):
        inputs = self.inputs()
        reference = dict(driver.PARENT_SEAL_REFERENCE)
        pin = stager.prepared_pin(reference, inputs)
        pin_sha = stager.saved_json_sha(pin)
        job = "24680"
        with tempfile.TemporaryDirectory() as folder:
            base = Path(folder)
            root = base / driver.STAGE_NAME
            root.mkdir()
            ledger = base / ".atlas-progressive-submit.json"
            intent = root / "submission-intent.json"
            receipt = root / "submission.json"
            uncertain = dict(
                stage=str(root.resolve()), script=driver.SBATCH, queue_before=["7"],
                status="SUBMISSION_INTENT_NOT_CONFIRMED", max_outstanding=10,
                pin_sha256=pin_sha,
            )
            stager.save(ledger, [uncertain])
            stager.save(intent, uncertain)
            driver.recover_submission_state(
                root.resolve(), ledger, intent, receipt, job, pin, pin_sha)
            confirmed = dict(uncertain, status="SUBMITTED", job=job)
            self.assertEqual(stager.read_json_file(ledger), [confirmed])
            self.assertEqual(stager.read_json_file(intent), confirmed)
            self.assertEqual(
                stager.read_json_file(receipt), stager.submission_receipt(confirmed, pin))

            # Final validation is deliberately read-only: an uncertain state
            # is rejected rather than being repaired after mathematical work.
            stager.save(ledger, [uncertain])
            stager.save(intent, uncertain)
            before = (ledger.read_bytes(), intent.read_bytes(), receipt.read_bytes())
            with self.assertRaises(ValueError):
                driver.validate_submission_state(
                    root.resolve(), inputs, [uncertain], uncertain, job, pin_sha)
            self.assertEqual(
                (ledger.read_bytes(), intent.read_bytes(), receipt.read_bytes()), before)

            # A published receipt forbids rollback, and a duplicate job in
            # another stage makes the whole ledger unrecoverable.
            with self.assertRaises(ValueError):
                driver.recover_submission_state(
                    root.resolve(), ledger, intent, receipt, job, pin, pin_sha)
            receipt.unlink()
            other = dict(confirmed, stage=str(base / "other"))
            stager.save(ledger, [other, uncertain])
            stager.save(intent, uncertain)
            before = (ledger.read_bytes(), intent.read_bytes())
            with self.assertRaises(ValueError):
                driver.recover_submission_state(
                    root.resolve(), ledger, intent, receipt, job, pin, pin_sha)
            self.assertEqual((ledger.read_bytes(), intent.read_bytes()), before)


if __name__ == "__main__":
    unittest.main()

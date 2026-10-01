"""Focused durability tests for the active ladder index publication gate.

The fixture hashes repository inputs dynamically because production hashes are
bound by the external override-manifest trust root rather than self-referential
constants. These tests are intended for the HPC compute-node gate, not local use.
"""
import hashlib
import json
import os
from contextlib import nullcontext
from pathlib import Path
import tempfile
import unittest
from unittest import mock

from progressive_submit import one_job_script
import campaign_blob
import math_ladder_boundary_index as driver
import stage_ladder_boundary_index as stage


REPO_ROOT = Path(__file__).resolve().parents[1]


def sha256(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def dynamic_input_hashes():
    """Hash current candidate bytes only inside the compute-test fixture."""
    result = {}
    for name in sorted(stage.STAGE_INPUT_NAMES):
        path = REPO_ROOT / name
        if not path.is_file() or path.is_symlink():
            raise AssertionError("missing regular candidate input: " + name)
        result[name] = sha256(path)
    return result


def dynamic_manifest_sha(inputs):
    raw = (json.dumps(inputs, indent=2, sort_keys=True) + "\n").encode("utf-8")
    return hashlib.sha256(raw).hexdigest()


def ready_test_counts():
    counts = dict(stage.EXPECTED_TEST_COUNTS)
    counts["test-stager-allowlist"] = 7
    return counts


class LadderBoundaryIndexStage(unittest.TestCase):
    def test_production_contract_is_enabled_with_exact_suite_counts(self):
        self.assertIs(stage.SUBMISSION_ENABLED, True)
        self.assertIs(driver.SUBMISSION_ENABLED, True)
        self.assertEqual(
            stage.EXPECTED_TEST_COUNTS["test-stage-ladder-boundary-index"], 20)
        self.assertEqual(
            stage.EXPECTED_TEST_COUNTS["test-math-acceptance-index"], 34)
        self.assertEqual(
            stage.EXPECTED_TEST_COUNTS["test-stager-allowlist"], 7)
        self.assertEqual(
            stage.require_enabled_launcher(), stage.EXPECTED_TEST_COUNTS)

    def test_main_guard_precedes_argument_and_filesystem_use(self):
        with mock.patch.object(stage, "SUBMISSION_ENABLED", False):
            with mock.patch.object(stage.sys, "argv", ["stage", "bad"]):
                with mock.patch.object(stage, "run_enabled") as run:
                    with self.assertRaises(SystemExit):
                        stage.main()
        run.assert_not_called()
        with mock.patch.object(driver, "SUBMISSION_ENABLED", False):
            with mock.patch.object(driver, "job_identity") as identity:
                with self.assertRaises(SystemExit):
                    driver.main()
        identity.assert_not_called()

    def test_dynamic_candidate_fixture_has_the_exact_input_inventory(self):
        inputs = dynamic_input_hashes()
        counts = ready_test_counts()
        self.assertEqual(set(inputs), stage.STAGE_INPUT_NAMES)
        manifest_sha = dynamic_manifest_sha(inputs)
        validated_inputs, validated_counts = stage.validate_configuration(
            inputs, manifest_sha, counts)
        self.assertEqual(validated_inputs, inputs)
        self.assertEqual(validated_counts, counts)
        self.assertIsNot(validated_inputs, inputs)
        self.assertIsNot(validated_counts, counts)

    def test_configuration_rejects_every_unfrozen_dimension(self):
        inputs = dynamic_input_hashes()
        manifest_sha = dynamic_manifest_sha(inputs)
        counts = ready_test_counts()
        malformed_inputs = dict(inputs)
        malformed_inputs[stage.INDEX_PATH] = "0" * 63
        wrong_acceptance_count = dict(counts)
        wrong_acceptance_count["test-math-acceptance-index"] = 33
        unfrozen_allowlist_count = dict(counts)
        unfrozen_allowlist_count["test-stager-allowlist"] = 0
        cases = [
            ({}, manifest_sha, counts),
            (inputs, "UNFROZEN_SHA256_REQUIRES_REVIEW", counts),
            (inputs, manifest_sha, unfrozen_allowlist_count),
            (inputs, manifest_sha, wrong_acceptance_count),
            (malformed_inputs, manifest_sha, counts),
        ]
        for arguments in cases:
            with self.subTest(manifest=arguments[1], counts=arguments[2]):
                with self.assertRaises(ValueError):
                    stage.validate_configuration(*arguments)

    def test_production_sources_have_no_self_hash_freeze_constants(self):
        stage_source = (REPO_ROOT / "hpc/stage_ladder_boundary_index.py").read_text(
            encoding="utf-8")
        driver_source = (REPO_ROOT / "hpc/math_ladder_boundary_index.py").read_text(
            encoding="utf-8")
        for forbidden in (
                "FROZEN_INPUTS", "FROZEN_OVERRIDES_SHA256",
                "FROZEN_PIN_SHA256", "UNFROZEN_SHA256"):
            with self.subTest(forbidden=forbidden):
                self.assertNotIn(forbidden, stage_source)
                self.assertNotIn(forbidden, driver_source)
        self.assertIn("hpc/stage_ladder_boundary_index.py", stage.STAGE_INPUT_NAMES)
        self.assertIn("hpc/math_ladder_boundary_index.py", stage.STAGE_INPUT_NAMES)
        self.assertIn("hpc/campaign_blob.py", stage.STAGE_INPUT_NAMES)

    def test_review_transitive_evidence_is_pinned(self):
        self.assertTrue({
            stage.BEFORE_EVIDENCE_PATH,
            stage.ORIGINAL_CAPTURE_PATH,
        }.issubset(stage.STAGE_INPUT_NAMES))

    def test_unittest_count_parser_requires_one_exact_summary(self):
        with tempfile.TemporaryDirectory() as directory:
            log = Path(directory) / "suite.log"
            log.write_text("Ran 34 tests in 0.125s\n\nOK\n", encoding="utf-8")
            self.assertEqual(driver.parse_unittest_count(log), 34)
            log.write_text(
                "Ran 34 tests in 0.125s\nRan 34 tests in 0.126s\n",
                encoding="utf-8",
            )
            self.assertIsNone(driver.parse_unittest_count(log))
            log.write_text(
                "Ran 34 tests in 0.125s\n\nOK (skipped=34)\n",
                encoding="utf-8",
            )
            self.assertIsNone(driver.parse_unittest_count(log))
            log.write_text(
                "OK\nRan 34 tests in 0.125s\n\nOK (skipped=34)\n",
                encoding="utf-8",
            )
            self.assertIsNone(driver.parse_unittest_count(log))
            log.write_text("OK without summary\n", encoding="utf-8")
            self.assertIsNone(driver.parse_unittest_count(log))

    def test_wrong_allocation_cannot_repair_intent_or_write_receipt(self):
        durable_record = {"job": "4000001"}
        with mock.patch.object(
                stage, "submission_scope", return_value=Path("/campaign")):
            with mock.patch.object(
                    stage, "existing_lock", return_value=nullcontext()):
                with mock.patch.object(
                        stage, "read_json_file", return_value=[]) as read:
                    with mock.patch.object(
                            stage, "validate_campaign_history",
                            return_value=durable_record):
                        with mock.patch.object(
                                stage, "validate_submission_record",
                                return_value=durable_record):
                            with mock.patch.object(stage, "save") as save:
                                with self.assertRaisesRegex(
                                        ValueError, "allocation differs"):
                                    stage.confirmed_record(
                                        Path("/stage"), {}, repair_intent=True,
                                        expected_job="4000002")
        self.assertEqual(read.call_count, 1)
        save.assert_not_called()

    def test_campaign_history_requires_one_direct_exact_successor(self):
        root = Path(
            "/public/home/majj/atlas-rust-campaign-20260930/"
            "stages/ladder-boundary-index-v1"
        )

        def record(number, path):
            return {
                "stage": str(path),
                "script": "hpc/job.sbatch",
                "queue_before": [],
                "status": "SUBMITTED",
                "max_outstanding": 10,
                "job": str(5000000 + number),
                "pin_sha256": format(number + 1, "064x"),
            }

        predecessor = [
            record(index, Path("/campaign") / ("stage-" + str(index)))
            for index in range(6)
        ]
        child = record(6, root)
        unrelated = record(7, Path("/campaign/unrelated"))
        expected_predecessor_sha = stage.saved_json_sha(predecessor)
        with mock.patch.dict(stage.PREDECESSOR, {
                "campaign_ledger_sha256": expected_predecessor_sha}):
            self.assertIsNone(stage.validate_campaign_history(predecessor, root))
            self.assertEqual(
                stage.validate_campaign_history(
                    predecessor + [child], root, require_own=True),
                child,
            )
            invalid_histories = [
                predecessor + [child, child],
                predecessor + [child, unrelated],
                [{**predecessor[0], "job": "5999999"}, *predecessor[1:]],
                predecessor + [unrelated],
            ]
            for history in invalid_histories:
                with self.subTest(history=history):
                    with self.assertRaises(ValueError):
                        stage.validate_campaign_history(history, root)
            with self.assertRaises(ValueError):
                stage.validate_campaign_history(
                    predecessor, root, require_own=True)

    def test_final_input_read_failure_still_publishes_harness_report(self):
        with tempfile.TemporaryDirectory() as directory:
            out = Path(directory)
            report = {"schema": "test-ladder-index-report"}
            with mock.patch.object(
                    driver, "frozen_stage_inputs",
                    side_effect=ValueError("simulated final drift")):
                success, published, report_sha256 = driver.publish_final_report(
                    out, out, {"input": "hash"}, {"input": "hash"}, [],
                    None, True, None, report)
            self.assertFalse(success)
            self.assertEqual(
                published["status"], "LADDER_BOUNDARY_INDEX_HARNESS_FAILURE")
            self.assertEqual(published["failure"], "final-input-recheck")
            self.assertIs(published["final_input_recheck_error"], True)
            self.assertIs(published["source_integrity_rechecked"], False)
            self.assertTrue((out / "report.json").is_file())
            self.assertEqual(
                (out / "report.sha256").read_text(encoding="utf-8").strip(),
                report_sha256,
            )

    def test_current_candidate_predecessor_chain_is_exact(self):
        inputs = dynamic_input_hashes()
        index = stage.validate_predecessor(REPO_ROOT, inputs)
        self.assertEqual(len(index["entries"]), 3)
        self.assertEqual(
            index["entries"][-1]["entry_sha256"],
            stage.PREDECESSOR["index_head_sha256"],
        )
        changed = dict(inputs)
        changed[stage.REPORT_PATH] = "0" * 64
        with self.assertRaises(ValueError):
            stage.validate_predecessor(REPO_ROOT, changed)
        transitive = [
            (stage.BEFORE_EVIDENCE_PATH, "before_evidence",
             stage.PREDECESSOR["before_evidence_sha256"]),
            (stage.ORIGINAL_CAPTURE_PATH, "original_capture",
             stage.PREDECESSOR["original_capture_sha256"]),
        ]
        review = json.loads((REPO_ROOT / stage.REVIEW_PATH).read_text(
            encoding="utf-8"))
        loaded = {
            name: json.loads((REPO_ROOT / name).read_text(encoding="utf-8"))
            for name in (
                stage.REPORT_PATH, stage.INSPECTION_PATH, stage.REVIEW_PATH,
                stage.INDEX_PATH, stage.BEFORE_EVIDENCE_PATH,
                stage.ORIGINAL_CAPTURE_PATH,
            )
        }
        for path, review_key, expected_sha256 in transitive:
            with self.subTest(path=path, mutation="manifest"):
                self.assertEqual(inputs[path], expected_sha256)
                changed = dict(inputs)
                changed[path] = "0" * 64
                with self.assertRaises(ValueError):
                    stage.validate_predecessor(REPO_ROOT, changed)
            with self.subTest(path=path, mutation="review-reference"):
                mutated_review = json.loads(json.dumps(review))
                mutated_review[review_key]["sha256"] = "0" * 64
                mutated_loaded = dict(loaded)
                mutated_loaded[stage.REVIEW_PATH] = mutated_review

                def load_mutated(_root, name, _wanted, **_options):
                    return json.loads(json.dumps(mutated_loaded[name]))

                with mock.patch.object(
                        stage, "load_relative_json", side_effect=load_mutated):
                    with self.assertRaisesRegex(
                            ValueError, "index review changed"):
                        stage.validate_predecessor(REPO_ROOT, inputs)

    def test_stable_reader_rejects_symlinks_and_extra_hardlinks(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "real").write_bytes(b"payload")
            os.chmod(root / "real", 0o444)
            (root / "link").symlink_to("real")
            with self.assertRaises(ValueError):
                stage.stable_relative_bytes(root, "link")
            os.link(root / "real", root / "second")
            with self.assertRaises(ValueError):
                stage.stable_relative_bytes(root, "real", mode=0o444, nlink=1)

    def test_stable_reader_rejects_parent_escape(self):
        with tempfile.TemporaryDirectory() as directory:
            for name in ("../escape", "/absolute", "a/./b", "a//b", ""):
                with self.subTest(name=name):
                    with self.assertRaises(ValueError):
                        stage.stable_relative_bytes(directory, name)

    def test_frozen_stage_inputs_rejects_extra_namespace_files(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for name in stage.STAGE_INPUT_NAMES:
                path = root / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes((name + "\n").encode("utf-8"))
                path.chmod(0o444)
            self.assertEqual(
                set(stage.frozen_stage_inputs(root)), stage.STAGE_INPUT_NAMES)
            for name in ("hpc/typing.py", "tests/unpinned-evidence.json"):
                with self.subTest(name=name):
                    path = root / name
                    path.write_text("unpinned\n", encoding="utf-8")
                    path.chmod(0o444)
                    with self.assertRaisesRegex(
                            ValueError, "namespace differs from the pin"):
                        stage.frozen_stage_inputs(root)
                    path.unlink()
            for name in ("hpc/unpinned-empty", "tests/unpinned/empty"):
                with self.subTest(name=name):
                    path = root / name
                    path.mkdir(parents=True)
                    with self.assertRaisesRegex(
                            ValueError, "namespace differs from the pin"):
                        stage.frozen_stage_inputs(root)
                    path.rmdir()
                    if name.startswith("tests/"):
                        path.parent.rmdir()

    def test_atomic_install_publishes_single_link_read_only_bytes(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            incoming = root / ".incoming"
            incoming.mkdir()
            raw = b"immutable-index-input\n"
            wanted = hashlib.sha256(raw).hexdigest()
            stage._atomic_install(root, incoming, "nested/input.json", raw, wanted)
            target = root / "nested/input.json"
            value = target.stat()
            self.assertEqual(target.read_bytes(), raw)
            self.assertEqual(value.st_mode & 0o777, 0o444)
            self.assertEqual(value.st_nlink, 1)
            self.assertEqual(list(incoming.iterdir()), [])
            stage._atomic_install(root, incoming, "nested/input.json", raw, wanted)
            with self.assertRaises(ValueError):
                stage._atomic_install(
                    root, incoming, "nested/input.json", b"changed", "0" * 64)

    def test_nonempty_publication_scratch_fails_closed(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            incoming = root / ".incoming"
            incoming.mkdir()
            (incoming / ".copy-interrupted").write_bytes(b"partial")
            with self.assertRaisesRegex(ValueError, "manual reconciliation"):
                stage._prepare_incoming(root)
            self.assertTrue((incoming / ".copy-interrupted").is_file())

    def test_pin_and_receipt_detach_nested_state(self):
        inputs = dynamic_input_hashes()
        manifest_sha = dynamic_manifest_sha(inputs)
        pin = stage.build_pin(inputs, manifest_sha, ready_test_counts())
        root = Path(
            "/public/home/majj/atlas-rust-campaign-20260930/"
            "stages/ladder-boundary-index-v1"
        )
        record = {
            "stage": str(root),
            "script": stage.SBATCH,
            "queue_before": [],
            "status": "SUBMITTED",
            "max_outstanding": 10,
            "job": "4000001",
            "pin_sha256": stage.saved_json_sha(pin),
        }
        receipt = stage.submission_receipt(record, pin, root=root)
        self.assertEqual(receipt["cargo_commands"], 0)
        self.assertEqual(receipt["atlas_commands"], 0)
        self.assertEqual(receipt["index"]["entries"], 3)
        self.assertEqual(receipt["test_counts"], ready_test_counts())
        self.assertEqual(
            receipt["retired_stager_object"],
            stage.RETIRED_STAGER_BUNDLE_REFERENCE,
        )
        record["queue_before"].append("3999999")
        pin["predecessor"]["job"] = "changed"
        self.assertEqual(receipt["queue_before"], [])
        self.assertEqual(receipt["predecessor"]["job"], "3875239")
        changed_pin = stage.build_pin(
            inputs, manifest_sha, ready_test_counts())
        changed_pin["retired_stager_object"]["sha256"] = "0" * 64
        with self.assertRaises(ValueError):
            stage.validate_pin(changed_pin)

        with tempfile.TemporaryDirectory() as directory:
            campaign = Path(directory)
            reference = stage.RETIRED_STAGER_BUNDLE_REFERENCE
            shard = campaign / "objects/sha256" / reference["sha256"][:2]
            shard.mkdir(parents=True)
            with mock.patch.object(
                    campaign_blob, "campaign_storage_root",
                    return_value=campaign):
                with self.assertRaisesRegex(
                        ValueError, "missing or unsafe campaign blob"):
                    campaign_blob.verify_blob(campaign, reference)
                object_path = shard / reference["sha256"]
                object_path.write_bytes(b"changed")
                object_path.chmod(0o444)
                with self.assertRaisesRegex(
                        ValueError, "missing or changed campaign blob"):
                    campaign_blob.verify_blob(campaign, reference)

    def test_receipt_rejects_wrong_stage_script_pin_or_unconfirmed_state(self):
        inputs = dynamic_input_hashes()
        manifest_sha = dynamic_manifest_sha(inputs)
        pin = stage.build_pin(inputs, manifest_sha, ready_test_counts())
        root = Path(
            "/public/home/majj/atlas-rust-campaign-20260930/"
            "stages/ladder-boundary-index-v1"
        )
        base = {
            "stage": str(root),
            "script": stage.SBATCH,
            "queue_before": [],
            "status": "SUBMITTED",
            "max_outstanding": 10,
            "job": "4000001",
            "pin_sha256": stage.saved_json_sha(pin),
        }
        mutations = [
            {**base, "stage": str(root.parent / "other")},
            {**base, "script": "hpc/other.sbatch"},
            {**base, "pin_sha256": "0" * 64},
            {**base, "status": "SUBMISSION_INTENT_NOT_CONFIRMED"},
        ]
        for record in mutations:
            with self.subTest(record=record):
                with self.assertRaises(ValueError):
                    stage.submission_receipt(record, pin, root=root)

    def test_sbatch_is_one_small_non_array_non_dependency_job(self):
        raw = (REPO_ROOT / stage.SBATCH).read_text(encoding="utf-8")
        one_job_script(raw)
        self.assertIn("#SBATCH --cpus-per-task=1", raw)
        self.assertIn("#SBATCH --mem=2G", raw)
        self.assertIn("#SBATCH --time=00:15:00", raw)
        self.assertNotIn("--array", raw)
        self.assertNotIn("--dependency", raw)
        self.assertTrue(raw.startswith("#!/bin/bash -p\n"))
        self.assertIn(
            "exec /public/software/anaconda/anaconda3-2022.5/bin/python3.9 \\\n"
            "  -E -s -S -B hpc/math_ladder_boundary_index.py\n",
            raw,
        )

    def test_stage_inputs_do_not_name_a_legacy_top_level_tree(self):
        self.assertTrue(stage.STAGE_INPUT_NAMES)
        for name in stage.STAGE_INPUT_NAMES:
            self.assertFalse(name.startswith("/"), name)
            self.assertNotIn("..", Path(name).parts)
            self.assertNotIn("atlas-rust-", name)


if __name__ == "__main__":
    unittest.main()

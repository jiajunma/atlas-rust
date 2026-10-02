"""Infrastructure-only checks for the campaign submission boundary."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

from progressive_submit import (capacity, one_job_script, queue_ids, save,
                                submit_one as _submit_one)


SYNTHETIC_CREATION_SHA256 = "c" * 64


def submit_one(root, script, env, **options):
    """Authorize old unit-test stages without weakening the production gate.

    Receipt validation itself is exercised against real published stages in
    test_campaign_stage_creation.py.  This file isolates the later queue,
    ledger, script-snapshot and uncertain-submission behavior.
    """
    options.setdefault("pin_sha256", "a" * 64)
    options.setdefault("stage_creation_sha256", SYNTHETIC_CREATION_SHA256)
    try:
        script_sha256 = hashlib.sha256(
            (Path(root) / script).read_bytes()).hexdigest()
    except (OSError, TypeError, ValueError):
        # Invalid paths are deliberately handed to the production path guard;
        # this placeholder is unreachable when that guard rejects correctly.
        script_sha256 = "0" * 64
    stage = Path(root).resolve()
    stage_stat = stage.stat()
    receipt = {
        "schema": "atlas-stage-creation-receipt-v1",
        "status": "PUBLISHED",
        "stage": str(stage),
        "stage_device": stage_stat.st_dev,
        "stage_inode": stage_stat.st_ino,
        "contract": {
            "script": {"path": script, "sha256": script_sha256},
            "pin": {
                "path": "pin.json",
                "stage_creation_key": "stage_creation",
            },
        },
    }
    with patch("progressive_submit.validate_stage_creation",
               return_value=receipt):
        return _submit_one(root, script, env, **options)


class ProgressiveSubmission(unittest.TestCase):
    def make_stage(self, home, name="stage"):
        campaign = Path(home) / "atlas-rust-campaign-20260930"
        root = campaign / "stages" / name
        root.mkdir(parents=True)
        (root / "one.sbatch").write_text("#SBATCH --nodes=1\n")
        (root / "one.sbatch").chmod(0o444)
        ledger = campaign / ".atlas-progressive-submit.json"
        if not ledger.exists():
            ledger.write_text("[]\n")
        return root

    def test_empty_queue(self):
        self.assertEqual(capacity(""), [])

    def test_nine_slots(self):
        self.assertEqual(len(capacity("\n".join(str(index) for index in range(1, 10)))), 9)

    def test_ten_blocks(self):
        with self.assertRaises(ValueError):
            capacity("\n".join(str(index) for index in range(1, 11)))

    def test_every_array_child_counts(self):
        with self.assertRaises(ValueError):
            capacity("\n".join("42_" + str(index) for index in range(11)))

    def test_ambiguous_inventory_is_rejected(self):
        for raw in ("42_[0-99%2]\n", "42\n42\n", "42\n\n", "warning\n", " 42\n"):
            with self.subTest(raw=raw), self.assertRaises(ValueError):
                queue_ids(raw)

    def test_array_directives_are_rejected(self):
        for flag in ("--array=0-99%2", "--array 0-1", "-a 0-1", "-a0-1"):
            with self.subTest(flag=flag), self.assertRaises(ValueError):
                one_job_script("#SBATCH " + flag)

    def test_dependencies_and_nested_submission_are_rejected(self):
        for flag in ("--dependency=afterok:42", "-d afterok:42"):
            with self.subTest(flag=flag), self.assertRaises(ValueError):
                one_job_script("#SBATCH " + flag)
        for command in ("sbatch other.sbatch", "command sbatch other.sbatch",
                        "job=$(sbatch --parsable other.sbatch)"):
            with self.subTest(command=command), self.assertRaises(ValueError):
                one_job_script(command)

    def test_option_shaped_script_name_is_rejected_before_scheduler(self):
        with (tempfile.TemporaryDirectory() as folder,
              patch.dict(os.environ, USER="guard-test"),
              patch("campaign_workspace.HPC_HOME", Path(folder))):
            root = self.make_stage(folder)
            (root / "--wrap=echo").write_text("#SBATCH --nodes=1\n")
            with patch("progressive_submit.subprocess.check_output") as scheduler:
                with self.assertRaises(ValueError):
                    submit_one(root, "--wrap=echo", {})
                scheduler.assert_not_called()

    def test_absolute_and_parent_script_paths_are_rejected(self):
        with (tempfile.TemporaryDirectory() as folder,
              patch.dict(os.environ, USER="guard-test"),
              patch("campaign_workspace.HPC_HOME", Path(folder))):
            root = self.make_stage(folder)
            outside = Path(folder) / "outside.sbatch"
            outside.write_text("#SBATCH --nodes=1\n")
            with patch("progressive_submit.subprocess.check_output") as scheduler:
                for script in (str(outside), "../one.sbatch"):
                    with self.subTest(script=script), self.assertRaises(ValueError):
                        submit_one(root, script, {})
                scheduler.assert_not_called()

    def test_final_script_symlink_is_rejected(self):
        with (tempfile.TemporaryDirectory() as folder,
              patch.dict(os.environ, USER="guard-test"),
              patch("campaign_workspace.HPC_HOME", Path(folder))):
            root = self.make_stage(folder)
            (root / "linked.sbatch").symlink_to(root / "one.sbatch")
            with patch("progressive_submit.subprocess.check_output") as scheduler:
                with self.assertRaises(ValueError):
                    submit_one(root, "linked.sbatch", {})
                scheduler.assert_not_called()

    def test_intermediate_directory_symlink_is_rejected(self):
        with (tempfile.TemporaryDirectory() as folder,
              patch.dict(os.environ, USER="guard-test"),
              patch("campaign_workspace.HPC_HOME", Path(folder))):
            first = self.make_stage(folder, "first")
            second = self.make_stage(folder, "second")
            (first / "linked").symlink_to(second, target_is_directory=True)
            with patch("progressive_submit.subprocess.check_output") as scheduler:
                with self.assertRaises(ValueError):
                    submit_one(first, "linked/one.sbatch", {})
                scheduler.assert_not_called()

    def test_uncertain_submission_cannot_retry(self):
        with (tempfile.TemporaryDirectory() as folder,
              patch.dict(os.environ, USER="guard-test"),
              patch("campaign_workspace.HPC_HOME", Path(folder))):
            root = self.make_stage(folder)
            with patch("progressive_submit.subprocess.check_output",
                       side_effect=["", subprocess.TimeoutExpired("sbatch", 25)]) as scheduler:
                with self.assertRaises(subprocess.TimeoutExpired):
                    submit_one(root, "one.sbatch", {})
                self.assertEqual(scheduler.call_count, 2)
            with patch("progressive_submit.subprocess.check_output") as scheduler:
                with self.assertRaisesRegex(ValueError, "unresolved submission"):
                    submit_one(root, "one.sbatch", {})
                scheduler.assert_not_called()

    def test_failed_status_query_never_writes_intent_or_submits(self):
        with (tempfile.TemporaryDirectory() as folder,
              patch.dict(os.environ, USER="guard-test"),
              patch("campaign_workspace.HPC_HOME", Path(folder))):
            root = self.make_stage(folder)
            failure = subprocess.CalledProcessError(1, "squeue")
            with patch("progressive_submit.subprocess.check_output", side_effect=failure) as scheduler:
                with self.assertRaises(subprocess.CalledProcessError):
                    submit_one(root, "one.sbatch", {})
                self.assertEqual(scheduler.call_count, 1)
            self.assertFalse((root / "submission-intent.json").exists())

    def test_campaign_stages_share_one_ledger(self):
        with (tempfile.TemporaryDirectory() as folder,
              patch.dict(os.environ, USER="guard-test"),
              patch("campaign_workspace.HPC_HOME", Path(folder))):
            first = self.make_stage(folder, "first")
            second = self.make_stage(folder, "second")
            with patch("progressive_submit.subprocess.check_output",
                       side_effect=["", "101", "", "102"]):
                submit_one(first, "one.sbatch", {})
                submit_one(second, "one.sbatch", {})
            ledger = first.parents[1] / ".atlas-progressive-submit.json"
            self.assertEqual([row["job"] for row in json.loads(ledger.read_text())],
                             ["101", "102"])

    def test_submission_uses_the_prechecked_script_snapshot(self):
        with (tempfile.TemporaryDirectory() as folder,
              patch.dict(os.environ, USER="guard-test"),
              patch("campaign_workspace.HPC_HOME", Path(folder))):
            root = self.make_stage(folder)
            original = "#SBATCH --nodes=1\n"
            pin_sha = "a" * 64

            def scheduler(command, **options):
                if command[0] == "squeue":
                    (root / "one.sbatch").chmod(0o644)
                    (root / "one.sbatch").write_text("sbatch nested.sbatch\n")
                    return ""
                self.assertEqual(command, ["sbatch", "--parsable", "--export=ALL"])
                self.assertEqual(options["input"], original)
                passed = options["pass_fds"]
                self.assertIsInstance(passed, tuple)
                self.assertEqual(len(passed), 1)
                descriptor = passed[0]
                self.assertEqual(
                    options["cwd"], "/proc/self/fd/" + str(descriptor))
                descriptor_stat = os.fstat(descriptor)
                cwd_stat = os.stat(options["cwd"])
                root_stat = root.stat()
                self.assertEqual(
                    (descriptor_stat.st_dev, descriptor_stat.st_ino),
                    (cwd_stat.st_dev, cwd_stat.st_ino),
                )
                self.assertEqual(
                    (descriptor_stat.st_dev, descriptor_stat.st_ino),
                    (root_stat.st_dev, root_stat.st_ino),
                )
                self.assertEqual(options["env"], {})
                self.assertIs(options["text"], True)
                self.assertEqual(options["timeout"], 25)
                ledger = json.loads((root.parents[1]
                                     / ".atlas-progressive-submit.json").read_text())
                intent = json.loads((root / "submission-intent.json").read_text())
                self.assertEqual(ledger[-1]["pin_sha256"], pin_sha)
                self.assertEqual(
                    ledger[-1]["stage_creation_sha256"],
                    SYNTHETIC_CREATION_SHA256,
                )
                self.assertEqual(intent["pin_sha256"], pin_sha)
                self.assertEqual(ledger[-1], intent)
                return "101"

            with patch("progressive_submit.subprocess.check_output", side_effect=scheduler):
                record = submit_one(
                    root, "one.sbatch", {}, pin_sha256=pin_sha)
            self.assertEqual(record["pin_sha256"], pin_sha)
            self.assertEqual(
                json.loads((root / "submission-intent.json").read_text()), record)

            invalid = self.make_stage(folder, "invalid-pin")
            with patch("progressive_submit.subprocess.check_output") as scheduler:
                with self.assertRaises(ValueError):
                    submit_one(
                        invalid, "one.sbatch", {}, pin_sha256="A" * 64)
                scheduler.assert_not_called()

        for queued in ("201", "201_3"):
            with (self.subTest(queued=queued),
                  tempfile.TemporaryDirectory() as folder,
                  patch.dict(os.environ, USER="guard-test"),
                  patch("campaign_workspace.HPC_HOME", Path(folder))):
                root = self.make_stage(folder)
                expected = dict(
                    stage=str(root.resolve()), script="one.sbatch", queue_before=[queued],
                    status="SUBMISSION_INTENT_NOT_CONFIRMED",
                    max_outstanding=10, pin_sha256=pin_sha,
                    stage_creation_sha256=SYNTHETIC_CREATION_SHA256)
                with patch("progressive_submit.subprocess.check_output",
                           side_effect=[queued + "\n", "201"]) as scheduler:
                    with self.assertRaisesRegex(ValueError, "unresolved submission"):
                        submit_one(
                            root, "one.sbatch", {}, pin_sha256=pin_sha)
                    self.assertEqual(scheduler.call_count, 2)
                ledger = root.parents[1] / ".atlas-progressive-submit.json"
                self.assertEqual(json.loads(ledger.read_text()), [expected])
                self.assertEqual(
                    json.loads((root / "submission-intent.json").read_text()),
                    expected)

    def test_submission_records_are_atomically_replaced(self):
        with tempfile.TemporaryDirectory() as folder:
            target = Path(folder) / "submission.json"
            save(target, {"status": "FIRST"})
            save(target, {"status": "SECOND"})
            self.assertEqual(json.loads(target.read_text()), {"status": "SECOND"})
            self.assertEqual(list(Path(folder).glob(".submission.json.*")), [])

    def test_same_stage_cannot_submit_twice(self):
        with (tempfile.TemporaryDirectory() as folder,
              patch.dict(os.environ, USER="guard-test"),
              patch("campaign_workspace.HPC_HOME", Path(folder))):
            root = self.make_stage(folder)
            campaign = root.parents[1]
            outside_lock = Path(folder) / "outside-lock"
            (campaign / ".atlas-progressive-submit.lock").symlink_to(outside_lock)
            with patch("progressive_submit.subprocess.check_output") as scheduler:
                with self.assertRaises(ValueError):
                    submit_one(root, "one.sbatch", {})
                scheduler.assert_not_called()
            self.assertFalse(outside_lock.exists())
            (campaign / ".atlas-progressive-submit.lock").unlink()
            outside_ledger = Path(folder) / "outside-ledger.json"
            outside_ledger.write_text("[]\n")
            ledger = campaign / ".atlas-progressive-submit.json"
            ledger.unlink()
            ledger.symlink_to(outside_ledger)
            with patch("progressive_submit.subprocess.check_output") as scheduler:
                with self.assertRaises(ValueError):
                    submit_one(root, "one.sbatch", {})
                scheduler.assert_not_called()
            self.assertEqual(outside_ledger.read_text(), "[]\n")
            ledger.unlink()
            older = dict(stage=str(root.with_name("older")), script="one.sbatch",
                         queue_before=[], status="SUBMITTED",
                         max_outstanding=10, job="90")
            malformed_histories = (
                [dict(older, max_outstanding=9)],
                [dict(older, extra=True)],
                [dict(older, pin_sha256="A" * 64)],
                [dict(older, queue_before=["90"])],
                [dict(older, queue_before=["90_3"])],
                [dict(older, status="SUBMISSION_INTENT_NOT_CONFIRMED")],
                [older, dict(older, stage=str(root.with_name("duplicate")))],
                [older, dict(older, job="91")],
            )
            for history in malformed_histories:
                ledger.write_text(json.dumps(history))
                with patch("progressive_submit.subprocess.check_output") as scheduler:
                    with self.assertRaisesRegex(ValueError, "unresolved submission"):
                        submit_one(root, "one.sbatch", {})
                    scheduler.assert_not_called()
            ledger.unlink()
            ledger.write_text("[]\n")
            outside_intent = Path(folder) / "outside-intent.json"
            outside_intent.write_text("{}\n")
            (root / "submission-intent.json").symlink_to(outside_intent)
            with patch("progressive_submit.subprocess.check_output") as scheduler:
                with self.assertRaises(ValueError):
                    submit_one(root, "one.sbatch", {})
                self.assertEqual(scheduler.call_count, 0)
            self.assertEqual(outside_intent.read_text(), "{}\n")
            (root / "submission-intent.json").unlink()
            with patch("progressive_submit.subprocess.check_output",
                       side_effect=["", "101"]):
                submit_one(root, "one.sbatch", {})
            with patch("progressive_submit.subprocess.check_output") as scheduler:
                with self.assertRaises(ValueError):
                    submit_one(root, "one.sbatch", {})
                scheduler.assert_not_called()


if __name__ == "__main__":
    unittest.main()

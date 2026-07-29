#!/usr/bin/env python3
"""Focused checks for raw upstream reference artifact naming."""

import json
import os
import pathlib
import sys
import tempfile
import unittest
from unittest.mock import patch


HPC_DIR = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HPC_DIR))

from reference_capture import artifact_base, capture_fixture, main, sha256  # noqa: E402


class ReferenceCaptureTest(unittest.TestCase):
    def test_artifact_path_retains_fixture_subdirectory(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            fixture = root / "tests/fixtures/commands/subscription_context.atlas"
            output = root / "output"
            self.assertEqual(
                artifact_base(fixture, root, output),
                output / "tests/fixtures/commands/subscription_context.atlas",
            )

    def test_artifact_paths_do_not_collide_when_flat_names_would(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            output = root / "output"
            first = root / "a/b__c.atlas"
            second = root / "a__b/c.atlas"
            dotted = root / "a__b/c.extra.atlas"

            bases = {
                artifact_base(first, root, output),
                artifact_base(second, root, output),
                artifact_base(dotted, root, output),
            }
            self.assertEqual(len(bases), 3)

    def test_batch_script_binds_slurm_spool_to_frozen_commit(self) -> None:
        script = (HPC_DIR / "reference_capture.sbatch").read_text(encoding="utf-8")
        self.assertIn('git hash-object "$job_script"', script)
        self.assertIn('git hash-object "$snapshot_job_script"', script)
        self.assertIn(
            'git rev-parse --verify "$detected_commit:hpc/reference_capture.sbatch"',
            script,
        )
        self.assertIn("post_snapshot_dirty_tree=", script)
        self.assertNotIn('cp "$job_script"', script)

    def test_batch_script_binds_helper_and_clean_snapshot_to_commit(self) -> None:
        script = (HPC_DIR / "reference_capture.sbatch").read_text(encoding="utf-8")
        self.assertIn(
            'git rev-parse --verify "$detected_commit:hpc/source_state.sh"',
            script,
        )
        self.assertIn(
            'source <(git show "$detected_commit:hpc/source_state.sh")',
            script,
        )
        self.assertNotIn('source "$submit_dir/hpc/source_state.sh"', script)
        self.assertIn('git archive --format=tar "$detected_commit"', script)
        self.assertIn('source_snapshot_provenance="git_archive:$detected_commit"', script)
        self.assertIn('source_snapshot_provenance="${detected_dirty_tree}_submit_tree"', script)
        self.assertIn('snapshot_scope="full tracked Git archive"', script)
        self.assertIn('source_snapshot_scope"] = sys.argv[5]', script)
        self.assertIn('snapshot_source_state_oid="$(git hash-object', script)
        self.assertIn('"$snapshot_source_state_oid" != "$source_state_helper_blob_oid"', script)

    def test_nonzero_oracle_exit_is_still_a_completed_capture(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            atlas = root / "upstream/atlas"
            scripts = atlas.parent / "atlas-scripts"
            scripts.mkdir(parents=True)
            atlas.write_text(
                "#!/bin/sh\ncat >/dev/null\nprintf 'raw stdout\\n'\n"
                "printf 'raw stderr\\n' >&2\nexit 7\n",
                encoding="utf-8",
            )
            os.chmod(atlas, 0o755)
            fixture = root / "tests/fixtures/commands/case.atlas"
            fixture.parent.mkdir(parents=True)
            fixture.write_text("1\n", encoding="utf-8")
            output = root / "output"
            output.mkdir()

            entry, completed = capture_fixture(
                atlas, fixture, root, output, timeout=5
            )

            self.assertTrue(completed)
            self.assertEqual(entry["capture_status"], "CAPTURED")
            self.assertEqual(entry["oracle_exit_status"], 7)
            self.assertEqual(entry["stdout"]["text"], "raw stdout\n")
            self.assertEqual(entry["stderr"]["text"], "raw stderr\n")

    def test_runtime_mutation_after_capture_invalidates_report(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            atlas = root / "upstream/atlas"
            scripts = atlas.parent / "atlas-scripts"
            scripts.mkdir(parents=True)
            (scripts / "helper.at").write_text("before\n", encoding="utf-8")
            atlas.write_text(
                "#!/bin/sh\ncat >/dev/null\nprintf 'oracle\\n'\n",
                encoding="utf-8",
            )
            os.chmod(atlas, 0o755)
            fixture = root / "tests/fixtures/commands/case.atlas"
            fixture.parent.mkdir(parents=True)
            fixture.write_text("1\n", encoding="utf-8")
            binary_sha = sha256(atlas.read_bytes())
            pinned_revision = "1" * 40
            submit_commit = "2" * 40
            output = root / "output"
            argv = [
                "reference_capture.py",
                str(atlas),
                str(output),
                str(fixture),
                "--workspace-root",
                str(root),
                "--commit",
                submit_commit,
                "--dirty-tree",
                "false",
                "--detected-commit",
                submit_commit,
                "--detected-dirty-tree",
                "false",
                "--job-id",
                "test-job",
                "--reference-revision",
                pinned_revision,
                "--source-snapshot-sha256",
                "snapshot",
                "--expected-binary-sha256",
                binary_sha,
            ]

            def capture_then_mutate(*args: object, **kwargs: object):
                entry, completed = capture_fixture(*args, **kwargs)
                atlas.write_bytes(atlas.read_bytes() + b"# replaced\n")
                (scripts / "helper.at").write_text("after\n", encoding="utf-8")
                return entry, completed

            with patch.object(sys, "argv", argv), patch(
                "reference_capture.command_output",
                side_effect=[
                    pinned_revision,
                    "",
                    pinned_revision,
                    "",
                    "test-host",
                ],
            ), patch(
                "reference_capture.capture_fixture",
                side_effect=capture_then_mutate,
            ):
                self.assertEqual(main(), 1)

            report = json.loads(
                (output / "reference_capture_report.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(report["status"], "FAIL")
            self.assertFalse(report["reference_runtime_unchanged"])
            self.assertFalse(report["provenance_checks"]["runtime_unchanged"])
            self.assertNotEqual(
                report["reference_atlas_binary_sha256"],
                report["post_reference_atlas_binary_sha256"],
            )
            self.assertNotEqual(
                report["reference_atlas_scripts_sha256"],
                report["post_reference_atlas_scripts_sha256"],
            )

    def test_main_requires_clean_pinned_provenance(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            atlas = root / "upstream/atlas"
            scripts = atlas.parent / "atlas-scripts"
            scripts.mkdir(parents=True)
            atlas.write_text(
                "#!/bin/sh\ncat >/dev/null\nprintf 'oracle\\n'\n", encoding="utf-8"
            )
            os.chmod(atlas, 0o755)
            fixture = root / "tests/fixtures/commands/case.atlas"
            fixture.parent.mkdir(parents=True)
            fixture.write_text("1\n", encoding="utf-8")
            binary_sha = sha256(atlas.read_bytes())
            pinned_revision = "1" * 40
            submit_commit = "2" * 40
            cases = (
                ("valid", pinned_revision, "", binary_sha, submit_commit, 0, "PASS"),
                (
                    "invalid-revision",
                    "pinned-revision",
                    "",
                    binary_sha,
                    submit_commit,
                    1,
                    "FAIL",
                ),
                (
                    "missing-binary-pin",
                    pinned_revision,
                    "",
                    None,
                    submit_commit,
                    1,
                    "FAIL",
                ),
                (
                    "dirty-source",
                    pinned_revision,
                    " M sources/interpreter/axis.w",
                    binary_sha,
                    submit_commit,
                    1,
                    "FAIL",
                ),
                (
                    "submit-state-mismatch",
                    pinned_revision,
                    "",
                    binary_sha,
                    "3" * 40,
                    1,
                    "FAIL",
                ),
            )
            for (
                name,
                requested_revision,
                tree_status,
                expected_binary,
                detected_submit_commit,
                exit_status,
                status,
            ) in cases:
                output = root / name
                argv = [
                    "reference_capture.py",
                    str(atlas),
                    str(output),
                    str(fixture),
                    "--workspace-root",
                    str(root),
                    "--commit",
                    submit_commit,
                    "--dirty-tree",
                    "false",
                    "--detected-commit",
                    detected_submit_commit,
                    "--detected-dirty-tree",
                    "false",
                    "--job-id",
                    "test-job",
                    "--reference-revision",
                    requested_revision,
                    "--source-snapshot-sha256",
                    "snapshot",
                ]
                if expected_binary is not None:
                    argv.extend(["--expected-binary-sha256", expected_binary])
                with self.subTest(case=name), patch.object(sys, "argv", argv), patch(
                    "reference_capture.command_output",
                    side_effect=[
                        requested_revision,
                        tree_status,
                        requested_revision,
                        tree_status,
                        "test-host",
                    ],
                ):
                    self.assertEqual(main(), exit_status)

                report = json.loads(
                    (output / "reference_capture_report.json").read_text(
                        encoding="utf-8"
                    )
                )
                self.assertEqual(report["status"], status)
                self.assertFalse(report["compatibility_claim"])
                self.assertFalse(report["expectations_consumed"])
                self.assertEqual(report["fixtures"][0]["oracle_exit_status"], 0)


if __name__ == "__main__":
    unittest.main()

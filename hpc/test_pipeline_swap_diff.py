#!/usr/bin/env python3
"""Focused checks for the typed pipeline frozen-event adapter."""

import json
import os
import pathlib
import subprocess
import sys
import tempfile
import unittest


HPC_DIR = pathlib.Path(__file__).resolve().parent
REPOSITORY = HPC_DIR.parent
sys.path.insert(0, str(HPC_DIR))

from pipeline_swap_diff import (  # noqa: E402
    FIXTURE_PLANS,
    FixturePlan,
    PINNED_ATLAS_REVISION,
    PENDING_OVERLOADS,
    expected_cli_observation,
    parse_cli_diagnostics,
    selected_fixture_source,
    run_fixture,
    validate_plan,
)


class PipelineSwapDiffTest(unittest.TestCase):
    def test_expected_cli_observation_preserves_multiline_values(self) -> None:
        observation = expected_cli_observation(
            [
                {"kind": "ReportLine", "text": "Declaring x\n"},
                {
                    "kind": "Value",
                    "value": {"type": "matrix", "display": "\n| 2 |\n"},
                },
            ]
        )

        self.assertEqual(
            observation["stdout"],
            "Declaring x\nValue: \n| 2 |\n\nBye.\n",
        )
        self.assertEqual(observation["exit_status"], 0)

    def test_cli_diagnostics_ignore_only_rendered_source_context(self) -> None:
        diagnostics, unparsed = parse_cli_diagnostics(
            "Type error at /tmp/case.atlas:1:2: wrong type\n"
            "  | f(1)\n"
            "  |  ^^^^\n"
            "unexpected trailer\n"
        )

        self.assertEqual(
            diagnostics, [{"category": "type", "message": "wrong type"}]
        )
        self.assertEqual(unparsed, ["unexpected trailer"])

    def test_fixture_plans_cover_every_line_and_reference_event(self) -> None:
        for plan in FIXTURE_PLANS:
            fixture = (
                REPOSITORY / "tests/fixtures" / f"{plan.name}.atlas"
            ).read_text(encoding="utf-8")
            events = json.loads(
                (
                    REPOSITORY
                    / "tests/reference"
                    / f"{plan.name}.events.json"
                ).read_text(encoding="utf-8")
            )["events"]
            with self.subTest(plan=plan.name):
                _, _, errors = validate_plan(plan, fixture, events)
                self.assertEqual(errors, [])

    def test_selected_fixture_source_retains_declared_order(self) -> None:
        self.assertEqual(
            selected_fixture_source("one\ntwo\nthree\n", (3, 1)),
            "three\none\n",
        )

    def test_plan_rejects_reordered_or_duplicate_selections(self) -> None:
        plan = FixturePlan(
            name="invalid",
            runnable_lines=(2, 1, 1),
            runnable_events=(1, 0, 0),
        )
        _, _, errors = validate_plan(
            plan,
            "one\ntwo\n",
            [{"kind": "Value"}, {"kind": "Value"}],
        )

        self.assertIn("runnable source lines are not unique and increasing", errors)
        self.assertIn("runnable event indices are not unique and increasing", errors)

    def test_unimplemented_overloads_are_explicit_pending_cases(self) -> None:
        self.assertEqual(
            {(item["feature"], item["signature"]) for item in PENDING_OVERLOADS},
            {
                ("involution", "(LieType,[int],string) -> mat"),
                ("involution", "(LieType,mat,string) -> mat"),
                ("real_form", "(InnerClass,mat,ratvec) -> RealForm"),
            },
        )

    def test_batch_script_binds_slurm_spool_to_frozen_commit(self) -> None:
        script = (REPOSITORY / "hpc/pipeline_swap_diff.sbatch").read_text(
            encoding="utf-8"
        )
        self.assertIn('git hash-object "$job_script"', script)
        self.assertIn('git hash-object "$snapshot_job_script"', script)
        self.assertIn(
            'git rev-parse --verify "$detected_commit:hpc/pipeline_swap_diff.sbatch"',
            script,
        )
        self.assertIn("post_snapshot_dirty_tree=", script)
        self.assertNotIn('cp "$job_script"', script)

    def test_batch_script_binds_helper_and_clean_snapshot_to_commit(self) -> None:
        script = (REPOSITORY / "hpc/pipeline_swap_diff.sbatch").read_text(
            encoding="utf-8"
        )
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

    def test_fixture_runs_the_full_constructor_surface(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            temporary = pathlib.Path(directory)
            cli = temporary / "atlas-cli"
            expected_source = (
                REPOSITORY
                / "tests/fixtures/eval/pipeline_swap_constructors.atlas"
            ).read_text(encoding="utf-8")
            cli.write_text(
                "#!/usr/bin/env python3\n"
                "import pathlib\n"
                "import sys\n"
                f"EXPECTED_SOURCE = {expected_source!r}\n"
                "if len(sys.argv) != 2 or "
                "pathlib.Path(sys.argv[1]).read_text() != EXPECTED_SOURCE:\n"
                "    print('unexpected fixture input', file=sys.stderr)\n"
                "    raise SystemExit(9)\n"
                "print(\"Value: Lie type 'A1'\")\n"
                "print(\"Value: simply connected root datum of Lie type 'A1'\")\n"
                "print('Value: true')\n"
                "print(\"Value: adjoint root datum of Lie type 'A1'\")\n"
                "print('Value: false')\n"
                "print(\"Value: simply connected root datum of Lie type 'A1'\")\n"
                "print('Bye.')\n",
                encoding="utf-8",
            )
            os.chmod(cli, 0o755)

            entry, passed = run_fixture(
                FIXTURE_PLANS[0],
                cli,
                temporary / "artifacts",
                REPOSITORY / "tests/fixtures",
                REPOSITORY / "tests/reference",
                REPOSITORY,
                PINNED_ATLAS_REVISION,
                5,
            )

            self.assertTrue(passed)
            self.assertEqual(entry["status"], "PASS")
            self.assertEqual(entry["pending"], [])

            for name, text in (("wrong.atlas", "Lie_type(\"B2\")\n"), ("empty.atlas", "")):
                wrong_input = temporary / name
                wrong_input.write_text(text, encoding="utf-8")
                with self.subTest(input=name):
                    completed = subprocess.run(
                        [str(cli), str(wrong_input)],
                        capture_output=True,
                        text=True,
                        timeout=5,
                    )
                    self.assertEqual(completed.returncode, 9)
                    self.assertIn("unexpected fixture input", completed.stderr)

    def test_domain_display_fixture_keeps_unported_surface_pending(self) -> None:
        plan = next(
            plan
            for plan in FIXTURE_PLANS
            if plan.name == "eval/pipeline_swap_domain_equality"
        )
        self.assertEqual(plan.runnable_lines, (1, 2))
        self.assertEqual(plan.runnable_events, (0, 1))
        self.assertEqual(len(plan.pending), 12)
        self.assertTrue(
            all(
                case.feature == "inner_class_real_form_display_and_relations"
                for case in plan.pending
            )
        )


if __name__ == "__main__":
    unittest.main()

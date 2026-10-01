import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

from campaign_workspace import (campaign_stage, campaign_storage_root,
                                create_result_folder, ephemeral_job_workspace,
                                submission_scope)


class CampaignWorkspace(unittest.TestCase):
    def test_one_campaign_contains_many_stages(self):
        with tempfile.TemporaryDirectory() as folder:
            home = Path(folder)
            campaign = home / "atlas-rust-campaign-20260930"
            one, two = campaign / "stages/ladder-before", campaign / "stages/ladder-after"
            one.mkdir(parents=True)
            two.mkdir()
            self.assertEqual(campaign_stage(one, home), campaign.resolve())
            self.assertEqual(campaign_stage(two, home), campaign.resolve())
            self.assertEqual(submission_scope(one, home), submission_scope(two, home))

    def test_top_level_stage_and_ambiguous_names_are_rejected(self):
        with tempfile.TemporaryDirectory() as folder, patch("campaign_workspace.HPC_HOME", Path(folder)):
            home = Path(folder)
            active = home / "atlas-rust-campaign-20260930"
            active.mkdir()
            self.assertEqual(campaign_storage_root(active), active.resolve())
            for path in (home / "atlas-rust-ladder-before",
                         home / "atlas-rust-campaign-current/stages/one",
                         home / "atlas-rust-campaign-20261001/stages/one",
                         home / "atlas-rust-campaign-20260930/stages/../bad"):
                path.mkdir(parents=True, exist_ok=True)
                with self.assertRaises(ValueError):
                    campaign_stage(path, home)
                with self.assertRaises(ValueError):
                    submission_scope(path, home)
            for path in (home / "atlas-rust-ladder-before",
                         home / "atlas-rust-campaign-current",
                         home / "atlas-rust-campaign-20261001"):
                with self.assertRaises(ValueError):
                    campaign_storage_root(path)

    def test_workspace_is_removed_but_result_evidence_remains(self):
        with (tempfile.TemporaryDirectory() as folder,
              patch("campaign_workspace.HPC_HOME", Path(folder)),
              patch.dict(os.environ, {}, clear=False)):
            os.environ.pop("SLURM_TMPDIR", None)
            stage = Path(folder) / "atlas-rust-campaign-20260930/stages/stage"
            stage.mkdir(parents=True)
            out = create_result_folder(stage, "123")
            evidence = out / "report.json"
            evidence.write_text("{}\n")
            with ephemeral_job_workspace(out, "ladder-before") as work:
                (work / "source").mkdir()
                (work / "target").mkdir()
                saved = work
                self.assertTrue(saved.is_dir())
            self.assertFalse(saved.exists())
            self.assertEqual(evidence.read_text(), "{}\n")

    def test_node_local_workspace_is_exact_and_removed(self):
        with (tempfile.TemporaryDirectory() as folder,
              tempfile.TemporaryDirectory() as scratch,
              patch("campaign_workspace.HPC_HOME", Path(folder))):
            stage = Path(folder) / "atlas-rust-campaign-20260930/stages/stage"
            stage.mkdir(parents=True)
            out = create_result_folder(stage, "456")
            with patch.dict(os.environ, {"SLURM_TMPDIR": scratch}):
                with ephemeral_job_workspace(out, "profile") as work:
                    self.assertEqual(work.parent, Path(scratch).resolve())
                    saved = work
            self.assertFalse(saved.exists())

    def test_bad_result_label_or_scratch_is_rejected(self):
        with (tempfile.TemporaryDirectory() as folder,
              patch("campaign_workspace.HPC_HOME", Path(folder))):
            bad = Path(folder) / "not-results/job"
            bad.mkdir(parents=True)
            with self.assertRaises(ValueError):
                with ephemeral_job_workspace(bad, "ok"): pass
            out = Path(folder) / "atlas-rust-campaign-20260930/stages/stage/results/7"
            out.mkdir(parents=True)
            with self.assertRaises(ValueError):
                with ephemeral_job_workspace(out, "BAD_LABEL"): pass
            with patch.dict(os.environ, {"SLURM_TMPDIR": str(Path(folder) / "missing")}), self.assertRaises(ValueError):
                with ephemeral_job_workspace(out, "ok"): pass
            persistent = Path(folder) / "persistent-scratch"
            persistent.mkdir()
            with patch.dict(os.environ, {"SLURM_TMPDIR": str(persistent)}), self.assertRaises(ValueError):
                with ephemeral_job_workspace(out, "ok"): pass
            relative = Path(folder) / "relative"
            relative.mkdir()
            old_cwd = Path.cwd()
            try:
                os.chdir(folder)
                with patch.dict(os.environ, {"SLURM_TMPDIR": "relative"}), self.assertRaises(ValueError):
                    with ephemeral_job_workspace(out, "ok"): pass
            finally:
                os.chdir(old_cwd)
            stage = Path(folder) / "atlas-rust-campaign-20260930/stages/safe-results"
            stage.mkdir(parents=True)
            outside = Path(folder) / "outside-results"
            outside.mkdir()
            (stage / "results").symlink_to(outside, target_is_directory=True)
            with self.assertRaises(ValueError):
                create_result_folder(stage, "99")
            self.assertEqual(list(outside.iterdir()), [])
            (stage / "results").unlink()
            escaped = Path(folder) / "escaped-job"
            with self.assertRaises(ValueError):
                create_result_folder(stage, str(escaped))
            self.assertFalse(escaped.exists())


if __name__ == "__main__":
    unittest.main()

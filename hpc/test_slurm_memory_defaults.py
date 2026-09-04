#!/usr/bin/env python3
"""Keep checked-in Slurm defaults within the XMU per-CPU memory policy."""

import pathlib
import re
import sys
import unittest
from unittest import mock

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import script_corpus_diff
from script_corpus_diff import DEFAULT_MEM_CAP_GB


HPC_DIR = pathlib.Path(__file__).resolve().parent
# The controller prints 4012 MB for MaxMemPerCPU, while the effective submit
# boundary accepts totals through 4096 MiB times the requested CPUs. Real jobs
# can receive more CPUs than requested; the exact allocation is recorded
# because the observed adjustment is not consistent at every total boundary.
CPU_SUBMIT_CAP_MIB = 4096
# CPU compute nodes override the controller default with AllowedRAMSpace=90.
CPU_ALLOWED_RAM_PERCENT = 90
CORPUS_DRIVER_HEADROOM_GIB = 2

EXPECTED_CPU_RESOURCES = {
    "differential.sbatch": (2, 8 * 1024),
    "filekl_diff.sbatch": (2, 8 * 1024),
    "kgb_differential.sbatch": (4, 16 * 1024),
    "massif_profile.sbatch": (4, 8 * 1024),
    "perf_sample_workers.sbatch": (4, 8 * 1024),
    "pipeline_swap_diff.sbatch": (2, 8 * 1024),
    "probe_diff.sbatch": (1, 4012),
    "quick_check.sbatch": (4, 8 * 1024),
    "real_group_preflight.sbatch": (2, 8 * 1024),
    "script_corpus.sbatch": (4, 16 * 1024),
    "weyl_focused.sbatch": (2, 8 * 1024),
}


def resource_request(script_path: pathlib.Path) -> tuple[int, int]:
    script = script_path.read_text(encoding="utf-8")
    cpus = re.search(r"^#SBATCH --cpus-per-task=(\d+)$", script, re.MULTILINE)
    memory = re.search(r"^#SBATCH --mem=(\d+)([MG])$", script, re.MULTILINE)
    if cpus is None or memory is None:
        raise AssertionError(f"{script_path.name} must declare CPU and memory")
    memory_mib = int(memory.group(1)) * (1024 if memory.group(2) == "G" else 1)
    return int(cpus.group(1)), memory_mib


class SlurmMemoryDefaultsTest(unittest.TestCase):
    def test_every_checked_in_job_declares_partition_and_memory(self) -> None:
        for script_path in sorted(HPC_DIR.glob("*.sbatch")):
            script = script_path.read_text(encoding="utf-8")
            partition = re.search(r"^#SBATCH --partition=(\S+)$", script, re.MULTILINE)
            memory = re.search(r"^#SBATCH --mem=(\d+)[MG]$", script, re.MULTILINE)
            with self.subTest(script=script_path.name):
                self.assertIsNotNone(partition, "partition must be explicit")
                self.assertIsNotNone(memory, "memory must be explicit")

    def test_cpu_partition_defaults_fit_per_cpu_memory_cap(self) -> None:
        for script_path in sorted(HPC_DIR.glob("*.sbatch")):
            script = script_path.read_text(encoding="utf-8")
            partition = re.search(r"^#SBATCH --partition=(\S+)$", script, re.MULTILINE)
            memory = re.search(r"^#SBATCH --mem=(\d+)([MG])$", script, re.MULTILINE)
            cpus = re.search(r"^#SBATCH --cpus-per-task=(\d+)$", script, re.MULTILINE)
            if partition is None or memory is None or partition.group(1) != "cpu":
                continue
            self.assertIsNotNone(cpus, f"{script_path.name} must declare CPUs")
            amount = int(memory.group(1)) * (1024 if memory.group(2) == "G" else 1)
            allowed = int(cpus.group(1)) * CPU_SUBMIT_CAP_MIB
            with self.subTest(script=script_path.name):
                self.assertLessEqual(
                    amount,
                    allowed,
                    f"{script_path.name} requests more than {CPU_SUBMIT_CAP_MIB} MiB per requested CPU",
                )

    def test_cpu_jobs_use_reviewed_memory_defaults(self) -> None:
        for name, expected in EXPECTED_CPU_RESOURCES.items():
            with self.subTest(script=name):
                self.assertEqual(resource_request(HPC_DIR / name), expected)

    def test_corpus_child_limit_fits_cpu_partition_default(self) -> None:
        _, requested_mib = resource_request(HPC_DIR / "script_corpus.sbatch")
        enforced_gib = requested_mib / 1024 * CPU_ALLOWED_RAM_PERCENT / 100
        self.assertLessEqual(
            DEFAULT_MEM_CAP_GB + CORPUS_DRIVER_HEADROOM_GIB,
            enforced_gib,
        )
        self.assertEqual(DEFAULT_MEM_CAP_GB, 6)

    def test_corpus_cap_rejects_smaller_cgroup_budget(self) -> None:
        with self.assertRaisesRegex(ValueError, "RLIMIT_AS cap"):
            script_corpus_diff.validate_memory_cap(
                6, cgroup_limit_bytes=4 * 1024**3, headroom_gib=2
            )

    def test_corpus_cap_accepts_default_cpu_cgroup_budget(self) -> None:
        script_corpus_diff.validate_memory_cap(
            6, cgroup_limit_bytes=14 * 1024**3, headroom_gib=2
        )

    def test_darwin_rlimit_failure_is_not_a_child_start_failure(self) -> None:
        with mock.patch.object(script_corpus_diff.sys, "platform", "darwin"), \
                mock.patch.object(
                    script_corpus_diff.resource,
                    "setrlimit",
                    side_effect=ValueError("current limit exceeds maximum limit"),
                ):
            script_corpus_diff.apply_memory_limit(6)

    def test_non_darwin_rlimit_failure_is_reported(self) -> None:
        with mock.patch.object(script_corpus_diff.sys, "platform", "linux"), \
                mock.patch.object(
                    script_corpus_diff.resource,
                    "setrlimit",
                    side_effect=ValueError("invalid limit"),
                ):
            with self.assertRaisesRegex(ValueError, "invalid limit"):
                script_corpus_diff.apply_memory_limit(6)

    def test_memory_snapshot_reads_both_cgroup_generations(self) -> None:
        script = (HPC_DIR / "memory_snapshot.sbatch").read_text(encoding="utf-8")
        for marker in (
            "cpus_per_task_requested",
            "job_cpus_per_node_allocated",
            "JobSubmitPlugins",
            "numactl --hardware",
            "numactl --show",
            "Cpus_allowed_list",
            "Mems_allowed_list",
            "overcommit_memory",
            "swappiness",
            "memory.max",
            "memory.current",
            "memory.stat",
            "memory.events",
            "memory.swap.max",
            "memory.limit_in_bytes",
            "memory.soft_limit_in_bytes",
            "memory.usage_in_bytes",
            "memory.oom_control",
            "memory.swappiness",
            "/etc/slurm/cgroup.conf",
            "SNAPSHOT_SLEEP_SECONDS",
        ):
            with self.subTest(marker=marker):
                self.assertIn(marker, script)
        self.assertNotIn("cpus_allocated=", script)
        self.assertIn('*/job_"${SLURM_JOB_ID}"', script)


if __name__ == "__main__":
    unittest.main()

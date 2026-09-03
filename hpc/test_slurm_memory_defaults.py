#!/usr/bin/env python3
"""Keep checked-in Slurm defaults within the XMU per-CPU memory policy."""

import pathlib
import re
import sys
import unittest

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from script_corpus_diff import DEFAULT_MEM_CAP_GB


HPC_DIR = pathlib.Path(__file__).resolve().parent
# The controller prints 4012 MB for MaxMemPerCPU, but accepts an exact
# `--mem=4G`/`--mem=4096M` request at the scheduler boundary.  Keep the test
# in the same units as checked-in scripts and the submit-time behavior.
CPU_MEMORY_CAP_MB = 4096


class SlurmMemoryDefaultsTest(unittest.TestCase):
    def test_every_checked_in_job_declares_partition_and_memory(self) -> None:
        for script_path in sorted(HPC_DIR.glob("*.sbatch")):
            script = script_path.read_text(encoding="utf-8")
            partition = re.search(r"^#SBATCH --partition=(\S+)$", script, re.MULTILINE)
            memory = re.search(r"^#SBATCH --mem=(\d+)G$", script, re.MULTILINE)
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
            allowed = int(cpus.group(1)) * CPU_MEMORY_CAP_MB
            with self.subTest(script=script_path.name):
                self.assertLessEqual(
                    amount,
                    allowed,
                    f"{script_path.name} requests more than {CPU_MEMORY_CAP_MB} MiB per CPU",
                )

    def test_corpus_child_limit_fits_cpu_partition_default(self) -> None:
        # script_corpus.sbatch requests 4G for one task; the child cap leaves
        # one GiB for the Python driver and process/runtime overhead.
        self.assertLess(DEFAULT_MEM_CAP_GB, 4)

    def test_memory_snapshot_reads_both_cgroup_generations(self) -> None:
        script = (HPC_DIR / "memory_snapshot.sbatch").read_text(encoding="utf-8")
        for marker in (
            "memory.max",
            "memory.current",
            "memory.limit_in_bytes",
            "memory.soft_limit_in_bytes",
            "memory.usage_in_bytes",
            "memory.oom_control",
            "/etc/slurm/cgroup.conf",
            "SNAPSHOT_SLEEP_SECONDS",
        ):
            with self.subTest(marker=marker):
                self.assertIn(marker, script)


if __name__ == "__main__":
    unittest.main()

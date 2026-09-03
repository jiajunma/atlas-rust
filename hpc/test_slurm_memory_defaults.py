#!/usr/bin/env python3
"""Keep CPU-partition job defaults within the cluster's per-job cap."""

import pathlib
import re
import sys
import unittest

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from script_corpus_diff import DEFAULT_MEM_CAP_GB


HPC_DIR = pathlib.Path(__file__).resolve().parent
CPU_MEMORY_CAP_GB = 4


class SlurmMemoryDefaultsTest(unittest.TestCase):
    def test_every_checked_in_job_declares_partition_and_memory(self) -> None:
        for script_path in sorted(HPC_DIR.glob("*.sbatch")):
            script = script_path.read_text(encoding="utf-8")
            partition = re.search(r"^#SBATCH --partition=(\S+)$", script, re.MULTILINE)
            memory = re.search(r"^#SBATCH --mem=(\d+)G$", script, re.MULTILINE)
            with self.subTest(script=script_path.name):
                self.assertIsNotNone(partition, "partition must be explicit")
                self.assertIsNotNone(memory, "memory must be explicit")

    def test_cpu_partition_defaults_fit_memory_cap(self) -> None:
        for script_path in sorted(HPC_DIR.glob("*.sbatch")):
            script = script_path.read_text(encoding="utf-8")
            partition = re.search(r"^#SBATCH --partition=(\S+)$", script, re.MULTILINE)
            memory = re.search(r"^#SBATCH --mem=(\d+)G$", script, re.MULTILINE)
            if partition is None or memory is None or partition.group(1) != "cpu":
                continue
            with self.subTest(script=script_path.name):
                self.assertLessEqual(
                    int(memory.group(1)),
                    CPU_MEMORY_CAP_GB,
                    f"{script_path.name} requests more than the cpu partition cap",
                )

    def test_corpus_child_limit_fits_cpu_partition_default(self) -> None:
        self.assertLessEqual(DEFAULT_MEM_CAP_GB, CPU_MEMORY_CAP_GB)


if __name__ == "__main__":
    unittest.main()

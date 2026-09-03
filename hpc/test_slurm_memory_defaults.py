#!/usr/bin/env python3
"""Keep CPU-partition job defaults within the cluster's per-job cap."""

import pathlib
import re
import unittest


HPC_DIR = pathlib.Path(__file__).resolve().parent
CPU_MEMORY_CAP_GB = 4


class SlurmMemoryDefaultsTest(unittest.TestCase):
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


if __name__ == "__main__":
    unittest.main()

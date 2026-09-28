from pathlib import Path
import unittest

import math_kl_boundary_probe
from math_kl_repair_build import build_environments


class KlRepairBuildTests(unittest.TestCase):
    def test_probe_is_importable_without_running_a_build(self):
        self.assertTrue(callable(math_kl_boundary_probe.main))

    def test_regression_phases_have_independent_sanitized_targets(self):
        inherited = {"PATH": "/toolchain", "CARGO_TARGET_DIR": "/shared",
                     "ATLAS_TRACE": "1", "RUSTFLAGS": "-Copt-level=3",
                     "CARGO_ENCODED_RUSTFLAGS": "-Copt-level=3", "CARGO_BUILD_JOBS": "128"}
        envs = build_environments(Path("/job/result"), inherited)
        self.assertEqual(inherited["CARGO_TARGET_DIR"], "/shared")
        for phase in ("before", "after"):
            self.assertEqual(envs[phase], {
                "PATH": "/toolchain", "CARGO_TARGET_DIR": "/job/result/target-" + phase,
                "CARGO_BUILD_JOBS": "2", "CARGO_PROFILE_TEST_DEBUG": "0"})
        self.assertIsNot(envs["before"], envs["after"])


if __name__ == "__main__":
    unittest.main()

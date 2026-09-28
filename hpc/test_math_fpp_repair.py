from pathlib import Path
import unittest

from math_fpp_repair_build import build_environments


class RepairBuildTests(unittest.TestCase):
    def test_before_and_after_never_share_cargo_targets(self):
        inherited = {"PATH": "/toolchain", "CARGO_TARGET_DIR": "/shared/target"}
        envs = build_environments(Path("/job/result"), inherited)
        self.assertEqual(envs["before"]["CARGO_TARGET_DIR"], "/job/result/target-before")
        self.assertEqual(envs["after"]["CARGO_TARGET_DIR"], "/job/result/target-after")
        self.assertEqual(inherited["CARGO_TARGET_DIR"], "/shared/target")
        self.assertIsNot(envs["before"], envs["after"])

    def test_both_phases_have_the_same_sanitized_toolchain_environment(self):
        envs = build_environments(Path("/job/result"), {
            "PATH": "/toolchain", "ATLAS_PROBE": "1", "RUSTFLAGS": "-C opt-level=3",
            "CARGO_ENCODED_RUSTFLAGS": "-Copt-level=3", "CARGO_BUILD_JOBS": "128"})
        for env in envs.values():
            self.assertEqual(env["PATH"], "/toolchain")
            self.assertEqual(env["CARGO_BUILD_JOBS"], "2")
            self.assertEqual(env["CARGO_PROFILE_TEST_DEBUG"], "0")
            self.assertFalse(any(k.startswith(("ATLAS_", "RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS"))
                                 for k in env))
        self.assertEqual({k: v for k, v in envs["before"].items() if k != "CARGO_TARGET_DIR"},
                         {k: v for k, v in envs["after"].items() if k != "CARGO_TARGET_DIR"})


if __name__ == "__main__":
    unittest.main()

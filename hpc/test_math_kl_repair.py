from pathlib import Path
import unittest

import math_kl_boundary_probe
import math_form_order_repair_build
import math_endgame_repair_build
import math_cayley_repair_build
from math_kl_repair_build import build_environments, without_added_regression


class KlRepairBuildTests(unittest.TestCase):
    def test_only_marked_regression_is_removed_for_parent_comparison(self):
        self.assertEqual(without_added_regression("prefixTEST new codeNEXT suffix", "TEST", "NEXT"),
                         "prefixNEXT suffix")

    def test_missing_or_duplicate_regression_markers_are_rejected(self):
        for source in ("prefixNEXT suffix", "TESTTESTNEXT"):
            with self.assertRaises(ValueError):
                without_added_regression(source, "TEST", "NEXT")

    def test_following_marker_must_follow_the_regression(self):
        with self.assertRaises(ValueError):
            without_added_regression("NEXT prefixTEST suffix", "TEST", "NEXT")

    def test_probe_is_importable_without_running_a_build(self):
        self.assertTrue(callable(math_kl_boundary_probe.main))

    def test_form_order_builder_is_importable_without_running_a_build(self):
        self.assertTrue(callable(math_form_order_repair_build.main))

    def test_endgame_builder_is_importable_without_running_a_build(self):
        self.assertTrue(callable(math_endgame_repair_build.main))

    def test_cayley_builder_is_importable_without_running_a_build(self):
        self.assertTrue(callable(math_cayley_repair_build.main))

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

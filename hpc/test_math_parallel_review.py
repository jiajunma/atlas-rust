import copy
from pathlib import Path
import tempfile
import unittest

from math_baseline_build import digest
from math_parallel_review import check_metrics, experiment_scope, recompute_summary, review_rounds


class ParallelReviewTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.folder = Path(self.temporary.name)
        self.case = {"id": "E7_kgb", "expected": "accept"}
        self.build = {"binaries": {engine: {"path": "/frozen/" + engine}
                                   for engine in ("oracle", "rust")}}
        comparison = {"status": "MATH_MATCH", "valid": {"oracle": True, "rust": True},
                      "independent_checks": {"oracle": None, "rust": None},
                      "full_stdout_equal": True, "full_stderr_equal": True, "exit_equal": True}
        self.report = {"status": "AB_MATCH_PENDING_INDEPENDENT_REVIEW", "resources": {
            "allocated_cpus": 4, "affinity_cpu_ids": [4, 5, 6, 7], "omp_threads": 1,
            "openblas_threads": 1, "child_address_space_gib": 6,
            "same_runtime_binary_all_arms": True}, "rounds": []}
        for i in range(4):
            row = {"index": i, "status": "MATH_MATCH", "observations": {},
                   "order": ["oracle", "rust-1", "rust-4"] if i % 2 == 0 else ["rust-4", "rust-1", "oracle"],
                   "comparisons": {a: copy.deepcopy(comparison) for a in ("rust-1", "rust-4")}}
            for arm, seconds, wall, cpu, rss in (("oracle", 2, "0:02.00", 1, 100),
                                                ("rust-1", 100, "1:40.00", 80, 1000),
                                                ("rust-4", 40, "0:40.00", 80, 1100)):
                engine = "oracle" if arm == "oracle" else "rust"
                directory = self.folder / ("round-" + str(i)) / arm
                directory.mkdir(parents=True)
                paths = {s: directory / (engine + "." + s) for s in ("stdout", "stderr", "metrics")}
                paths["stdout"].write_bytes(b"preamble\nMATH_BEGIN E7_kgb\n[1,2,3]\nMATH_END E7_kgb\nbye\n")
                paths["stderr"].write_bytes(b"")
                paths["metrics"].write_text(
                    '\tCommand being timed: "timeout --kill-after=5s 300 /frozen/' + engine + '"\n'
                    + '\tUser time (seconds): ' + str(cpu) + '\n\tSystem time (seconds): 0.1\n'
                    + '\tElapsed (wall clock) time (h:mm:ss or m:ss): ' + wall + '\n'
                    + '\tMaximum resident set size (kbytes): ' + str(rss) + '\n\tExit status: 0\n')
                row["observations"][arm] = {
                    "engine": engine, "seconds": seconds, "user_cpu_seconds": cpu, "system_cpu_seconds": 0.1,
                    "maxrss_kb": rss, "maxrss_approximate": False, "exit_status": 0,
                    "timed_out": False, "termination_uncertain": False,
                    "affinity_cpu_ids": [4, 5, 6, 7], "rayon_num_threads_requested": 4 if arm == "rust-4" else 1,
                    "resource_limit_gib": 6, "timeout_seconds": 300,
                    "command": ["/usr/bin/time", "-v", "-o", str(paths["metrics"]),
                                "timeout", "--kill-after=5s", "300", self.build["binaries"][engine]["path"]],
                    "artifacts": {s: {"path": str(p), "sha256": digest(p), "bytes": p.stat().st_size}
                                  for s, p in paths.items()}}
            self.report["rounds"].append(row)
        self.report["summary"] = recompute_summary(self.report["rounds"])

    def review(self):
        return review_rounds(self.report, self.folder, self.case, self.build)

    def rewrite(self, index, arm, suffix, old, new, repin=True):
        entry = self.report["rounds"][index]["observations"][arm]["artifacts"][suffix]
        path = Path(entry["path"])
        path.write_bytes(path.read_bytes().replace(old, new))
        if repin:
            entry.update(sha256=digest(path), bytes=path.stat().st_size)

    def test_full_review_and_independently_expected_ratios(self):
        summary, rounds = self.review()
        self.assertEqual(len(rounds), 4)
        self.assertEqual(summary["serial_parallel_speedup"]["paired"], [2.5] * 4)
        self.assertEqual(summary["parallel_efficiency"], 0.625)
        self.assertEqual(summary["parallel_serial_peak_rss_ratio"], 1.1)
        self.assertEqual(summary["original_over_rust"]["rust-4"], [0.05] * 4)
        self.assertEqual(summary["selection"], "PROMISING_PENDING_REVIEW")

    def test_scope_names_the_actual_case_not_a_historical_workload(self):
        for case_id in ("E7_kgb", "D8_kgb"):
            scope = experiment_scope({"id": case_id})
            self.assertIn(case_id, scope)
            self.assertIn("not other mathematics", scope)
        self.assertNotIn("E7", experiment_scope({"id": "D8_kgb"}))

    def test_raw_hash_changes_are_rejected(self):
        self.rewrite(0, "rust-4", "stdout", b"1,2,3", b"1,9,3", repin=False)
        with self.assertRaisesRegex(ValueError, "artifact"):
            self.review()

    def test_rehashed_coefficient_change_is_still_not_a_pass(self):
        self.rewrite(0, "rust-4", "stdout", b"1,2,3", b"1,9,3")
        with self.assertRaisesRegex(ValueError, "raw output"):
            self.review()

    def test_preamble_outside_math_section_is_compared(self):
        self.rewrite(0, "rust-1", "stdout", b"preamble", b"other")
        with self.assertRaisesRegex(ValueError, "raw output"):
            self.review()

    def test_equal_within_round_but_drift_between_rounds_fails(self):
        for arm in ("oracle", "rust-1", "rust-4"):
            self.rewrite(1, arm, "stdout", b"1,2,3", b"3,2,1")
        with self.assertRaisesRegex(ValueError, "between rounds"):
            self.review()

    def test_missing_round_and_reordered_schedule_fail(self):
        original = copy.deepcopy(self.report)
        self.report["rounds"].pop()
        with self.assertRaisesRegex(ValueError, "four completed"):
            self.review()
        self.report = original
        self.report["rounds"][1]["order"] = ["oracle", "rust-1", "rust-4"]
        with self.assertRaisesRegex(ValueError, "schedule"):
            self.review()

    def test_threads_affinity_limits_and_binary_command_are_bound(self):
        original = copy.deepcopy(self.report)
        for key, value in (("rayon_num_threads_requested", 1), ("affinity_cpu_ids", [0, 1, 2, 3]),
                           ("timeout_seconds", 600), ("resource_limit_gib", 8), ("command", ["other"])):
            with self.subTest(key=key):
                self.report = copy.deepcopy(original)
                self.report["rounds"][0]["observations"]["rust-4"][key] = value
                with self.assertRaisesRegex(ValueError, "command, threads"):
                    self.review()

    def test_missing_or_extra_arms_fail(self):
        original = copy.deepcopy(self.report)
        self.report["rounds"][0]["observations"].pop("oracle")
        with self.assertRaisesRegex(ValueError, "arms"):
            self.review()
        self.report = original
        self.report["rounds"][0]["observations"]["rust-8"] = {}
        with self.assertRaisesRegex(ValueError, "arms"):
            self.review()

    def test_failure_and_uncertainty_never_produce_speed_ratio(self):
        original = copy.deepcopy(self.report)
        for key, value in (("exit_status", 137), ("timed_out", True),
                           ("termination_uncertain", True), ("maxrss_approximate", True)):
            self.report = copy.deepcopy(original)
            self.report["rounds"][0]["observations"]["rust-4"][key] = value
            with self.assertRaisesRegex(ValueError, "unsuccessful"):
                self.review()

    def test_recorded_wall_cpu_and_rss_must_agree_with_gnu_time(self):
        original = copy.deepcopy(self.report)
        for key, value in (("seconds", 10), ("user_cpu_seconds", 4),
                           ("system_cpu_seconds", 1), ("maxrss_kb", 1)):
            self.report = copy.deepcopy(original)
            self.report["rounds"][0]["observations"]["rust-4"][key] = value
            with self.assertRaisesRegex(ValueError, "disagrees"):
                self.review()

    def test_nonfinite_negative_and_nonpositive_metrics_fail(self):
        original = copy.deepcopy(self.report)
        for key, value in (("seconds", float("nan")), ("seconds", float("inf")),
                           ("seconds", 0), ("user_cpu_seconds", -1), ("maxrss_kb", 0)):
            self.report = copy.deepcopy(original)
            self.report["rounds"][0]["observations"]["rust-4"][key] = value
            with self.assertRaises(ValueError):
                self.review()

    def test_gnu_exit_status_command_and_duplicate_metrics_fail(self):
        obs = self.report["rounds"][0]["observations"]["rust-4"]
        raw = Path(obs["artifacts"]["metrics"]["path"]).read_text()
        for changed in (raw.replace("Exit status: 0", "Exit status: 1"),
                        raw.replace("/frozen/rust", "/wrong/rust"),
                        raw + "\tUser time (seconds): 80\n"):
            with self.assertRaises(ValueError):
                check_metrics(obs, changed, obs["command"])

    def test_fabricated_summary_is_rejected(self):
        self.report["summary"]["serial_parallel_speedup"]["median"] = 4
        with self.assertRaisesRegex(ValueError, "statistics"):
            self.review()

    def test_selection_retains_speed_and_memory_guards(self):
        rows = copy.deepcopy(self.report["rounds"])
        rows[0]["observations"]["rust-4"]["maxrss_kb"] = 2100
        self.assertEqual(recompute_summary(rows)["selection"], "NO_CLEAR_BENEFIT_OR_MEMORY_GUARD")
        rows[0]["observations"]["rust-4"]["maxrss_kb"] = 1100
        rows[0]["observations"]["rust-4"]["seconds"] = 101
        self.assertEqual(recompute_summary(rows)["selection"], "NO_CLEAR_BENEFIT_OR_MEMORY_GUARD")


if __name__ == "__main__":
    unittest.main()

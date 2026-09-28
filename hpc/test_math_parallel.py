import copy
import unittest

from math_parallel import order, summarize


def rounds():
    return [{"status": "MATH_MATCH", "observations": {
        arm: {"seconds": seconds + i, "exit_status": 0, "maxrss_kb": 1000,
              "maxrss_approximate": False, "rayon_num_threads_requested": threads,
              "user_cpu_seconds": 20, "system_cpu_seconds": 1}
        for arm, seconds, threads in (("oracle", 20, 1), ("rust-1", 100, 1), ("rust-4", 40, 4))
    }} for i in range(4)]


class ParallelTests(unittest.TestCase):
    def test_order_balances_arms_and_oracle_placement(self):
        self.assertEqual(order(0), ["oracle", "rust-1", "rust-4"])
        self.assertEqual(order(1), ["rust-4", "rust-1", "oracle"])
        self.assertEqual(order(2), order(0))

    def test_failure_or_incomplete_rounds_have_no_accepted_speedup(self):
        for count in (0, 1, 3):
            self.assertFalse(summarize(rounds()[:count])["accepted"])
        for failure in ("TIMEOUT", "MATH_MISMATCH", "OUTPUT_CHANGED_BETWEEN_ROUNDS"):
            data = rounds()
            data[0]["status"] = failure
            self.assertIsNone(summarize(data)["serial_parallel_speedup"])

    def test_scaling_is_not_original_speedup(self):
        summary = summarize(rounds())
        self.assertGreater(summary["serial_parallel_speedup"]["median"], 2)
        self.assertTrue(all(r < 1 for r in summary["original_over_rust"]["rust-4"]))
        self.assertEqual(summary["selection"], "PROMISING_PENDING_REVIEW")
        self.assertFalse(summary["arms"]["rust-4"]["all_in_60_to_600_seconds"])

    def test_invalid_metrics_or_thread_setting_are_rejected(self):
        for key, value in (("seconds", float("nan")), ("seconds", 0), ("maxrss_kb", None),
                           ("maxrss_approximate", True), ("rayon_num_threads_requested", 1),
                           ("user_cpu_seconds", None), ("system_cpu_seconds", -1), ("exit_status", 1)):
            data = rounds()
            data[0]["observations"]["rust-4"][key] = value
            with self.assertRaises(ValueError):
                summarize(data)

    def test_overlapping_ranges_are_not_a_clear_win(self):
        data = rounds()
        data[0]["observations"]["rust-4"]["seconds"] = 105
        self.assertEqual(summarize(data)["selection"], "NO_CLEAR_BENEFIT_OR_MEMORY_GUARD")

    def test_large_memory_regression_is_not_selected(self):
        data = copy.deepcopy(rounds())
        data[0]["observations"]["rust-4"]["maxrss_kb"] = 2500
        self.assertEqual(summarize(data)["selection"], "NO_CLEAR_BENEFIT_OR_MEMORY_GUARD")


if __name__ == "__main__":
    unittest.main()

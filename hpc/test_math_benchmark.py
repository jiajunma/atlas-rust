import copy
import json
from pathlib import Path
import tempfile
import unittest

from math_benchmark import execution_order, specification, summarize, summarize_files


def rounds():
    return [{"status": "MATH_MATCH", "metrics": {
        "oracle": {"seconds": 120+i, "exit_status": 0, "maxrss_kb": 100, "maxrss_approximate": False},
        "rust": {"seconds": 60+i, "exit_status": 0, "maxrss_kb": 200, "maxrss_approximate": False}
    }} for i in range(4)]


class BenchmarkTests(unittest.TestCase):
    def test_frozen_catalog_binding_and_balanced_order(self):
        root = Path(__file__).resolve().parents[1]
        self.assertEqual([specification(root, i)[1]["id"] for i in range(3)],
                         ["E7_kgb", "D6_kgb", "D8_kgb"])
        self.assertEqual([execution_order(i)[0] for i in range(4)],
                         ["oracle", "rust", "oracle", "rust"])

    def test_only_complete_matched_rounds_produce_ratios(self):
        for status in ("RUST_TIMEOUT", "MATH_MISMATCH", "ORACLE_FAILURE", "HARNESS_FAILURE"):
            data = rounds()
            data[1]["status"] = status
            self.assertIsNone(summarize(data)["speed_ratio"])
        self.assertIsNone(summarize(rounds()[:3])["speed_ratio"])
        self.assertTrue(summarize(rounds())["accepted"])

    def test_metric_validation_and_window_are_not_inferred(self):
        for key, value in (("seconds", 0), ("seconds", float("nan")),
                           ("maxrss_kb", None), ("maxrss_approximate", True), ("exit_status", 1)):
            data = rounds()
            data[0]["metrics"]["rust"][key] = value
            with self.assertRaises(ValueError):
                summarize(data)
        data = rounds()
        data[0]["metrics"]["oracle"]["seconds"] = 1
        self.assertFalse(summarize(data)["engines"]["oracle"]["all_runs_in_60_to_600_seconds"])

    def test_full_output_must_be_stable_between_rounds(self):
        with tempfile.TemporaryDirectory() as tmp:
            paths = []
            for i, row in enumerate(rounds()):
                directory = Path(tmp) / str(i)
                directory.mkdir()
                # Equal engines in each pair still fail if both change between rounds.
                raw = f"MATH_BEGIN test\n[1,{i}]\nMATH_END test\n"
                for engine in ("oracle", "rust"):
                    (directory / (engine + ".stdout")).write_text(raw)
                p = directory / "report.json"
                p.write_text(json.dumps(dict(row, order=execution_order(i),
                                            observations=copy.deepcopy(row["metrics"]))))
                paths.append(p)
            with self.assertRaisesRegex(ValueError, "between rounds"):
                summarize_files(paths, {"id": "test"})


if __name__ == "__main__":
    unittest.main()

#!/usr/bin/env python3
"""HPC-only serial/parallel A/B screening with a current-original oracle gate."""
import json
import math
import os
from pathlib import Path
import platform
import statistics
import subprocess
import sys
import traceback

from math_baseline_build import digest
from math_suite import cases, compare, observe
from math_suite_review import review_build


def order(index):
    # Balance both the two Rust arms and placement of the original.
    return ["oracle", "rust-1", "rust-4"] if index % 2 == 0 else ["rust-4", "rust-1", "oracle"]


def summarize(rounds):
    if len(rounds) != 4 or any(r["status"] != "MATH_MATCH" for r in rounds):
        return {"accepted": False, "serial_parallel_speedup": None,
                "selection": "NOT_ELIGIBLE"}
    for row in rounds:
        for arm, m in row["observations"].items():
            expected_threads = 4 if arm == "rust-4" else 1
            if (m["exit_status"] != 0 or m["maxrss_approximate"]
                    or m["rayon_num_threads_requested"] != expected_threads
                    or not isinstance(m["maxrss_kb"], int) or m["maxrss_kb"] <= 0
                    or not math.isfinite(m["seconds"]) or m["seconds"] <= 0
                    or any(m[k] is None or not math.isfinite(m[k]) or m[k] < 0
                           for k in ("user_cpu_seconds", "system_cpu_seconds"))):
                raise ValueError("exact successful metrics and pinned thread settings required")
    samples = {arm: [r["observations"][arm]["seconds"] for r in rounds]
               for arm in ("oracle", "rust-1", "rust-4")}
    speedups = [a / b for a, b in zip(samples["rust-1"], samples["rust-4"])]
    rss_ratio = max(r["observations"]["rust-4"]["maxrss_kb"] for r in rounds) / max(
        r["observations"]["rust-1"]["maxrss_kb"] for r in rounds)
    # Conservative screening only: no claim of statistical significance or
    # generalization to other inputs, machines, builds or warm in-process caches.
    promising = (min(speedups) >= 1.05 and max(samples["rust-4"]) < min(samples["rust-1"])
                 and rss_ratio <= 2)
    return {"accepted": True,
            "serial_parallel_speedup": {"definition": "Rust1_seconds / Rust4_seconds",
                                        "paired": speedups, "median": statistics.median(speedups)},
            "parallel_efficiency": statistics.median(speedups) / 4,
            "parallel_serial_peak_rss_ratio": rss_ratio,
            "original_over_rust": {arm: [o / r for o, r in zip(samples["oracle"], samples[arm])]
                                   for arm in ("rust-1", "rust-4")},
            "arms": {arm: {"seconds": values, "median_seconds": statistics.median(values),
                           "min_seconds": min(values), "max_seconds": max(values),
                           "all_in_60_to_600_seconds": all(60 <= t <= 600 for t in values),
                           "maxrss_kb": [r["observations"][arm]["maxrss_kb"] for r in rounds]}
                     for arm, values in samples.items()},
            "selection": "PROMISING_PENDING_REVIEW" if promising else "NO_CLEAR_BENEFIT_OR_MEMORY_GUARD",
            "scope": "Four paired fresh-process trials, same node/build/input; filesystem caches not flushed. Screening only, independent review required."}


def main():
    if not os.environ.get("SLURM_JOB_ID"):
        raise SystemExit("Parallel A/B tests require an HPC compute allocation")
    root = Path.cwd()
    out = root / "results" / os.environ["SLURM_JOB_ID"]
    out.mkdir(parents=True, exist_ok=False)
    report = {"schema": "atlas-parallel-ab-v1", "status": "FAIL",
              "job": os.environ["SLURM_JOB_ID"], "node": platform.node(), "rounds": []}

    def save():
        path = out / "report.json"
        path.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n")
        path.with_suffix(".sha256").write_text(digest(path) + "\n")

    try:
        if digest(root / "parallel-pin.json") != os.environ["PARALLEL_PIN_SHA256"]:
            raise ValueError("parallel pin changed")
        pin = json.loads((root / "parallel-pin.json").read_text())
        report["pin"] = pin
        actual = {str(p.relative_to(root)): digest(p) for p in sorted(root.rglob("*"))
                  if p.is_file() and p.relative_to(root).parts[0] in ("hpc", "tests")}
        if actual != pin["inputs"]:
            raise ValueError("source/fixture inputs changed")
        if digest(os.environ["PARALLEL_SPOOL"]) != pin["inputs"]["hpc/math_parallel.sbatch"]:
            raise ValueError("submitted script changed")
        cpus = int(os.environ["SLURM_CPUS_PER_TASK"])
        affinity = sorted(os.sched_getaffinity(0))
        if cpus < 4 or len(affinity) < 4:
            raise ValueError("four allocated and accessible CPUs required")
        report["resources"] = {"allocated_cpus": cpus, "affinity_cpu_ids": affinity,
                               "cpu_model": next((line.split(":", 1)[1].strip()
                                   for line in Path("/proc/cpuinfo").read_text().splitlines()
                                   if line.startswith("model name")), "unavailable"),
                               "logical_processor_count": os.cpu_count(),
                               "omp_threads": 1, "openblas_threads": 1,
                               "child_address_space_gib": 6, "same_runtime_binary_all_arms": True}
        with (out / "checker.log").open("wb") as log:
            check = subprocess.run([sys.executable, "-m", "unittest", "discover", "-s", "hpc",
                                    "-p", "test_math_parallel.py", "-v"], stdout=log, stderr=subprocess.STDOUT)
        report["checker"] = {"exit_status": check.returncode, "log_sha256": digest(out / "checker.log")}
        if check.returncode:
            raise ValueError("parallel checker failed")
        lock = json.loads((root / "tests/math/baseline.json").read_text())
        build, counts = review_build(Path(pin["build"]), pin["build_sha256"], lock)
        report.update(source_files_rehashed=counts, binary_pins=build["binaries"])
        selected = [case for case in cases(root)[1] if case["id"] == pin["case_id"]]
        if len(selected) != 1 or selected[0]["expected"] != "accept":
            raise ValueError("one positive mathematical catalog case required")
        case = selected[0]
        report["case"] = {k: v for k, v in case.items() if k != "source"}
        (out / "input.atlas").write_text(case["source"])
        scripts = Path(build["binaries"]["oracle"]["path"]).parent / "atlas-scripts"
        reference = None
        report["status"] = "RUNNING"
        save()
        for index in range(4):
            row = {"index": index, "order": order(index), "observations": {}, "comparisons": {}}
            streams = {}
            for arm in row["order"]:
                folder = out / ("round-" + str(index)) / arm
                folder.mkdir(parents=True, exist_ok=False)
                engine = "oracle" if arm == "oracle" else "rust"
                threads = 4 if arm == "rust-4" else 1
                observation, raw = observe(engine, build["binaries"][engine]["path"], scripts,
                                           case, folder, 300, rayon_threads=threads)
                row["observations"][arm], streams[arm] = observation, raw
            for arm in ("rust-1", "rust-4"):
                row["comparisons"][arm] = compare(case,
                    {"oracle": row["observations"]["oracle"], "rust": row["observations"][arm]},
                    {"oracle": streams["oracle"], "rust": streams[arm]})
            row["status"] = ("MATH_MATCH" if all(c["status"] == "MATH_MATCH"
                             and c["full_stdout_equal"] and c["full_stderr_equal"]
                             for c in row["comparisons"].values()) else "MATH_OR_EXECUTION_FAILURE")
            if row["status"] == "MATH_MATCH":
                complete = streams["oracle"]
                if reference is not None and complete != reference:
                    row["status"] = "OUTPUT_CHANGED_BETWEEN_ROUNDS"
                reference = complete
            report["rounds"].append(row)
            save()
            if row["status"] != "MATH_MATCH":
                break
        report["summary"] = summarize(report["rounds"])
        review_build(Path(pin["build"]), pin["build_sha256"], lock)
        if any(digest(root / n) != h for n, h in pin["inputs"].items()):
            raise ValueError("inputs changed during A/B")
        report["status"] = ("AB_MATCH_PENDING_INDEPENDENT_REVIEW" if report["summary"]["accepted"]
                            else "NOT_BENCHMARK_ACCEPTED")
    except Exception:
        report["status"] = "FAIL"
        report["error"] = traceback.format_exc()
    save()
    print(report["status"], out, flush=True)
    return int(report["status"] != "AB_MATCH_PENDING_INDEPENDENT_REVIEW")


if __name__ == "__main__":
    raise SystemExit(main())

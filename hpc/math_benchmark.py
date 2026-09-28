#!/usr/bin/env python3
"""HPC-only fresh-process repetitions, gated by complete mathematical equality."""
import argparse
import json
import math
import os
from pathlib import Path
import platform
import statistics

from math_baseline_build import digest
from math_suite import cases, payload, run


def execution_order(round_index):
    return ["oracle", "rust"] if round_index % 2 == 0 else ["rust", "oracle"]


def specification(root, benchmark_index):
    config = json.loads((root / "tests/math/benchmarks.json").read_text())
    if (config["schema"] != "atlas-math-benchmarks-v1" or config["rounds"] != 4
            or config["timeout_per_engine_seconds"] != 300):
        raise ValueError("reviewed four-round/300-second policy required")
    if not 0 <= benchmark_index < len(config["cases"]):
        raise ValueError("invalid benchmark index")
    entry = config["cases"][benchmark_index]
    case = cases(root)[1][entry["case_index"]]
    if case["id"] != entry["id"] or case["expected"] != "accept":
        raise ValueError("benchmark must bind an accepted-input catalog case")
    return config, entry, case


def summarize(rounds, expected_rounds=4):
    if len(rounds) != expected_rounds or any(r["status"] != "MATH_MATCH" for r in rounds):
        return {"accepted": False, "speed_ratio": None}
    samples = {e: [r["metrics"][e]["seconds"] for r in rounds] for e in ("oracle", "rust")}
    for r in rounds:
        for m in r["metrics"].values():
            if (m["exit_status"] != 0 or m["maxrss_approximate"]
                    or not math.isfinite(m["seconds"]) or m["seconds"] <= 0
                    or not isinstance(m["maxrss_kb"], int) or m["maxrss_kb"] <= 0):
                raise ValueError("positive exact successful metrics required")
    ratios = [o / r for o, r in zip(samples["oracle"], samples["rust"])]
    return {
        "accepted": True,
        "speed_ratio": {
            "definition": "original_seconds / rust_seconds; >1 means Rust faster",
            "paired_values": ratios, "median": statistics.median(ratios),
            "min": min(ratios), "max": max(ratios)},
        "engines": {e: {
            "seconds": v, "median_seconds": statistics.median(v),
            "min_seconds": min(v), "max_seconds": max(v),
            "maxrss_kb": [r["metrics"][e]["maxrss_kb"] for r in rounds],
            "all_runs_in_60_to_600_seconds": all(60 <= t <= 600 for t in v)}
            for e, v in samples.items()},
        "scope": "four fresh-process repetitions on one node, not a universal speed claim"}


def summarize_files(paths, case):
    rounds, original = [], None
    for i, path in enumerate(paths):
        report = json.loads(path.read_text())
        if report["order"] != execution_order(i):
            raise ValueError("execution order differs from alternating schedule")
        streams = {e: (path.parent / (e + ".stdout")).read_bytes() for e in ("oracle", "rust")}
        if report["status"] == "MATH_MATCH":
            sections = {e: payload(out, case["id"]) for e, out in streams.items()}
            if sections["oracle"] is None or sections["oracle"] != sections["rust"]:
                raise ValueError("complete paired output differs")
            if original is None:
                original = sections["oracle"]
            elif original != sections["oracle"]:
                raise ValueError("complete mathematical output changed between rounds")
        rounds.append({"status": report["status"], "metrics": {
            e: {k: obs[k] for k in ("seconds", "exit_status", "maxrss_kb", "maxrss_approximate")}
            for e, obs in report["observations"].items()}})
    return summarize(rounds)


def main():
    if not os.environ.get("SLURM_JOB_ID"):
        raise SystemExit("Benchmark execution requires an HPC compute job")
    parser = argparse.ArgumentParser()
    parser.add_argument("--benchmark", type=int, required=True)
    parser.add_argument("--build", type=Path, required=True)
    args = parser.parse_args()
    root = Path.cwd()
    config, entry, case = specification(root, args.benchmark)
    out = root / "results" / os.environ["SLURM_JOB_ID"]
    out.mkdir(parents=True, exist_ok=False)
    paths = []
    report = {"schema": "atlas-math-benchmark-v1", "job": os.environ["SLURM_JOB_ID"],
              "node": platform.node(), "status": "RUNNING",
              "benchmark_index": args.benchmark, "case": entry,
              "configuration": config, "input_sha256": case["input_sha256"],
              "build_report": str(args.build.resolve()), "build_report_sha256": digest(args.build),
              "input_manifest_sha256": digest(root / "suite-inputs.json"), "rounds": []}

    def save():
        p = out / "benchmark.json"
        p.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n")
        p.with_suffix(".sha256").write_text(digest(p) + "\n")

    save()
    for i in range(config["rounds"]):
        directory = out / ("round-" + str(i))
        run(root, entry["case_index"], args.build.resolve(), config["timeout_per_engine_seconds"],
            output=directory, spool_name="hpc/math_benchmark.sbatch", order=execution_order(i))
        path = directory / "report.json"
        trial = json.loads(path.read_text())
        paths.append(path)
        report["rounds"].append({"index": i, "path": str(path),
                                "sha256": digest(path), "status": trial["status"]})
        save()
        if trial["status"] != "MATH_MATCH":
            break
    report["summary"] = summarize_files(paths, case)
    report["status"] = ("REPEATED_MATH_MATCH_PENDING_REVIEW" if report["summary"]["accepted"]
                        else "NOT_BENCHMARK_ACCEPTED")
    save()
    print(report["status"], out, flush=True)
    return 0 if report["summary"]["accepted"] else 1


if __name__ == "__main__":
    raise SystemExit(main())

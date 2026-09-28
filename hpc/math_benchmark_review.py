#!/usr/bin/env python3
"""Rehash and independently accept/reject complete benchmark rounds on HPC."""
import argparse
import json
import os
from pathlib import Path
import statistics

from math_baseline_build import digest
from math_benchmark import specification
from math_suite_review import checked_json, complete_section, review_build, review_case


def review_benchmark(path, root, manifest, build_path, build_sha, build):
    report = checked_json(path)
    config, entry, case = specification(root, report["benchmark_index"])
    if (report["schema"] != "atlas-math-benchmark-v1" or report["case"] != entry
            or report["configuration"] != config or report["input_sha256"] != case["input_sha256"]
            or report["build_report"] != str(build_path) or report["build_report_sha256"] != build_sha
            or report["input_manifest_sha256"] != digest(root / "suite-inputs.json")
            or report["status"] not in ("NOT_BENCHMARK_ACCEPTED", "REPEATED_MATH_MATCH_PENDING_REVIEW")
            or not 1 <= len(report["rounds"]) <= config["rounds"]):
        raise ValueError("benchmark provenance or completion mismatch")
    reviewed, reference, nodes = [], None, set()
    for i, pin in enumerate(report["rounds"]):
        trial_path = path.parent / ("round-" + str(i)) / "report.json"
        if pin["index"] != i or pin["path"] != str(trial_path) or digest(trial_path) != pin["sha256"]:
            raise ValueError("round path or digest changed")
        trial = checked_json(trial_path)
        wanted = ["oracle", "rust"] if i % 2 == 0 else ["rust", "oracle"]
        if (trial["order"] != wanted or trial["case_index"] != entry["case_index"]
                or trial["job_id"] != report["job"] or pin["status"] != trial["status"]
                or any(o["timeout_seconds"] != 300 or o["resource_limit_gib"] != 6
                       for o in trial["observations"].values())):
            raise ValueError("changed schedule, identity or resource envelope")
        nodes.add(trial["node"])
        result = review_case(trial_path, case, build_path, build_sha, build, manifest)
        reviewed.append(result)
        if result["status"] == "MATH_MATCH":
            raw = (trial_path.parent / "oracle.stdout").read_bytes()
            section = complete_section(raw, case["id"])
            if reference is None:
                reference = section
            elif section != reference:
                raise ValueError("mathematics changed between repetitions")
        elif i != len(report["rounds"]) - 1:
            raise ValueError("benchmark continued after failed mathematics")
    if nodes != {report["node"]}:
        raise ValueError("rounds did not share the recorded node")
    accepted = len(reviewed) == 4 and all(r["status"] == "MATH_MATCH" for r in reviewed)
    if accepted != report["summary"]["accepted"]:
        raise ValueError("incorrect benchmark acceptance")
    summary = {"accepted": accepted, "speed_ratio": None}
    if accepted:
        from math_benchmark import summarize
        # Aggregate fields are recomputed only from independently rehashed rounds.
        summary = summarize(reviewed)
        ratios = [r["metrics"]["oracle"]["seconds"] / r["metrics"]["rust"]["seconds"]
                  for r in reviewed]
        if summary["speed_ratio"]["median"] != statistics.median(ratios):
            raise ValueError("incorrect paired median")
    if summary != report["summary"]:
        raise ValueError("recorded statistics differ from rehashed round metrics")
    return {"job": report["job"], "node": report["node"], "case": entry,
            "benchmark_sha256": digest(path), "rounds": reviewed, "summary": summary}


def main():
    if not os.environ.get("SLURM_JOB_ID"):
        raise SystemExit("Benchmark review requires an HPC compute job")
    p = argparse.ArgumentParser()
    p.add_argument("--jobs", required=True)
    p.add_argument("--build", type=Path, required=True)
    args = p.parse_args()
    root = Path.cwd()
    manifest = checked_json(root / "suite-inputs.json", os.environ["SUITE_INPUTS_SHA256"])
    actual = {str(f.relative_to(root)): digest(f) for f in root.rglob("*")
              if f.is_file() and f.relative_to(root).parts[0] in ("hpc", "tests")}
    if actual != manifest:
        raise ValueError("changed benchmark inputs")
    if digest(os.environ["MATH_REVIEW_SPOOL"]) != manifest["hpc/math_benchmark_review.sbatch"]:
        raise ValueError("submitted review differs from pinned script")
    build_sha = os.environ["MATH_BUILD_SHA256"]
    lock = json.loads((root / "tests/math/baseline.json").read_text())
    build, counts = review_build(args.build, build_sha, lock)
    jobs = args.jobs.split(",")
    if len(set(jobs)) != len(jobs) or any(not j.isdecimal() for j in jobs):
        raise ValueError("explicit unique numeric job IDs required")
    reviewed = [review_benchmark(root / "results" / j / "benchmark.json",
                                root, manifest, args.build, build_sha, build) for j in jobs]
    out = root / "results" / os.environ["SLURM_JOB_ID"]
    out.mkdir(parents=True, exist_ok=False)
    summary = {"schema": "atlas-math-benchmark-review-v1", "job": os.environ["SLURM_JOB_ID"],
               "status": "BENCHMARK_ARTIFACTS_VERIFIED", "build_report_sha256": build_sha,
               "input_manifest_sha256": digest(root / "suite-inputs.json"),
               "source_files_rehashed": counts, "binaries": build["binaries"],
               "benchmarks": reviewed}
    path = out / "benchmark-review.json"
    path.write_text(json.dumps(summary, indent=2, sort_keys=True) + "\n")
    path.with_suffix(".sha256").write_text(digest(path) + "\n")
    print(summary["status"], path, digest(path), flush=True)


if __name__ == "__main__":
    main()

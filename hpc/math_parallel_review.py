#!/usr/bin/env python3
"""Recheck frozen parallel trials on HPC without rerunning either interpreter.

Do not import the experiment's ordering or summary implementation: the review
reconstructs the schedule, full-stream comparisons and statistics independently.
"""
import json
import math
import os
from pathlib import Path
import re
import statistics
import subprocess
import sys
import traceback

from math_baseline_build import digest
from math_cycle_check import mathematical_evidence
from math_suite import cases
from math_suite_review import checked_json, independently_classify, review_build


ARMS = ("oracle", "rust-1", "rust-4")


def experiment_scope(case):
    return ("Four fresh-process " + case["id"]
            + " trials only; not other mathematics, real forms, hardware or statistical significance.")


def inputs(root):
    return {str(p.relative_to(root)): digest(p)
            for name in ("hpc", "tests") for p in sorted((root / name).rglob("*"))
            if p.is_file()}


def one_metric(raw, label):
    values = re.findall(r"^\s*" + re.escape(label) + r":\s*(.*?)\s*$", raw, re.M)
    if len(values) != 1:
        raise ValueError("missing or duplicate GNU time metric: " + label)
    return values[0]


def check_metrics(observation, raw, command):
    if (observation["exit_status"] != 0 or observation["timed_out"]
            or observation["termination_uncertain"] or observation["maxrss_approximate"]):
        raise ValueError("unsuccessful or approximate observation")
    for key in ("seconds", "user_cpu_seconds", "system_cpu_seconds"):
        value = observation[key]
        if type(value) not in (int, float) or not math.isfinite(value) or value < 0:
            raise ValueError("invalid finite time: " + key)
    if observation["seconds"] <= 0 or type(observation["maxrss_kb"]) is not int:
        raise ValueError("positive wall time and integer RSS required")
    rss = int(one_metric(raw, "Maximum resident set size (kbytes)"))
    if rss <= 0 or rss != observation["maxrss_kb"]:
        raise ValueError("RSS disagrees with original GNU time artifact")
    for key, label in (("user_cpu_seconds", "User time (seconds)"),
                       ("system_cpu_seconds", "System time (seconds)")):
        if float(one_metric(raw, label)) != observation[key]:
            raise ValueError("CPU time disagrees with original GNU time artifact")
    elapsed = one_metric(raw, "Elapsed (wall clock) time (h:mm:ss or m:ss)")
    if not re.fullmatch(r"(?:\d+:)?\d+:[0-5]\d(?:\.\d+)?", elapsed):
        raise ValueError("malformed GNU elapsed time")
    wall = 0.0
    for component in elapsed.split(":"):
        wall = 60 * wall + float(component)
    # The monotonic measurement wraps process setup. GNU time is rounded and
    # starts inside it; retain both, with an explicit bounded overhead check.
    if wall <= 0 or not -0.02 <= observation["seconds"] - wall <= 1.0:
        raise ValueError("wall time disagrees with GNU time (1s setup tolerance)")
    if (one_metric(raw, "Exit status") != "0"
            or one_metric(raw, "Command being timed") != '"' + " ".join(command[4:]) + '"'):
        raise ValueError("GNU time command or exit status changed")
    return wall


def recompute_summary(rows):
    samples = {arm: [row["observations"][arm]["seconds"] for row in rows] for arm in ARMS}
    rss = {arm: [row["observations"][arm]["maxrss_kb"] for row in rows] for arm in ARMS}
    ratios = [samples["rust-1"][i] / samples["rust-4"][i] for i in range(4)]
    memory_ratio = max(rss["rust-4"]) / max(rss["rust-1"])
    selected = (all(value >= 1.05 for value in ratios)
                and max(samples["rust-4"]) < min(samples["rust-1"]) and memory_ratio <= 2)
    return {
        "accepted": True,
        "serial_parallel_speedup": {"definition": "Rust1_seconds / Rust4_seconds",
                                    "paired": ratios, "median": statistics.median(ratios)},
        "parallel_efficiency": statistics.median(ratios) / 4,
        "parallel_serial_peak_rss_ratio": memory_ratio,
        "original_over_rust": {arm: [samples["oracle"][i] / samples[arm][i] for i in range(4)]
                               for arm in ("rust-1", "rust-4")},
        "arms": {arm: {"seconds": values, "median_seconds": statistics.median(values),
                       "min_seconds": min(values), "max_seconds": max(values),
                       "all_in_60_to_600_seconds": all(60 <= value <= 600 for value in values),
                       "maxrss_kb": rss[arm]} for arm, values in samples.items()},
        "selection": "PROMISING_PENDING_REVIEW" if selected else "NO_CLEAR_BENEFIT_OR_MEMORY_GUARD",
        "scope": "Four paired fresh-process trials, same node/build/input; filesystem caches not flushed. Screening only, independent review required."}


def review_rounds(report, folder, case, build):
    resources = report["resources"]
    affinity = resources["affinity_cpu_ids"]
    if (resources["allocated_cpus"] != 4 or len(affinity) != 4
            or any(type(cpu) is not int or cpu < 0 for cpu in affinity)
            or affinity != sorted(set(affinity))
            or resources["omp_threads"] != 1 or resources["openblas_threads"] != 1
            or resources["child_address_space_gib"] != 6
            or resources["same_runtime_binary_all_arms"] is not True):
        raise ValueError("resource envelope or CPU affinity changed")
    if report["status"] != "AB_MATCH_PENDING_INDEPENDENT_REVIEW" or len(report["rounds"]) != 4:
        raise ValueError("four completed successful rounds required")
    reference = None
    checked = []
    for i, row in enumerate(report["rounds"]):
        wanted = ["oracle", "rust-1", "rust-4"] if i in (0, 2) else ["rust-4", "rust-1", "oracle"]
        if (row["index"] != i or row["order"] != wanted or row["status"] != "MATH_MATCH"
                or set(row["observations"]) != set(ARMS)
                or set(row["comparisons"]) != {"rust-1", "rust-4"}):
            raise ValueError("changed round schedule, arms or outcome")
        streams, gnu_wall = {}, {}
        for arm in ARMS:
            obs = row["observations"][arm]
            engine = "oracle" if arm == "oracle" else "rust"
            paths = {s: folder / ("round-" + str(i)) / arm / (engine + "." + s)
                     for s in ("stdout", "stderr", "metrics")}
            command = ["/usr/bin/time", "-v", "-o", str(paths["metrics"]),
                       "timeout", "--kill-after=5s", "300", build["binaries"][engine]["path"]]
            if (obs["engine"] != engine or obs["command"] != command
                    or obs["rayon_num_threads_requested"] != (4 if arm == "rust-4" else 1)
                    or obs["affinity_cpu_ids"] != affinity or obs["resource_limit_gib"] != 6
                    or obs["timeout_seconds"] != 300 or set(obs["artifacts"]) != set(paths)):
                raise ValueError("changed command, threads, affinity or limits")
            for suffix, path in paths.items():
                if obs["artifacts"][suffix] != {"path": str(path), "sha256": digest(path),
                                                "bytes": path.stat().st_size}:
                    raise ValueError("changed full execution artifact")
            gnu_wall[arm] = check_metrics(obs, paths["metrics"].read_text(), command)
            streams[arm] = (paths["stdout"].read_bytes(), paths["stderr"].read_bytes())
        for arm in ("rust-1", "rust-4"):
            entries = {"oracle": row["observations"]["oracle"], "rust": row["observations"][arm]}
            raw = {"oracle": streams["oracle"], "rust": streams[arm]}
            status = independently_classify(case, entries, raw)
            if status != "MATH_MATCH" or streams[arm] != streams["oracle"]:
                raise ValueError("complete raw output does not match original")
            comparison = {"status": status, "valid": {"oracle": True, "rust": True},
                          "independent_checks": {e: mathematical_evidence(case, entries[e], *raw[e])
                                                 for e in ("oracle", "rust")},
                          "full_stdout_equal": True, "full_stderr_equal": True, "exit_equal": True}
            if comparison != row["comparisons"][arm]:
                raise ValueError("recorded comparison disagrees with rechecked evidence")
        if reference is not None and streams["oracle"] != reference:
            raise ValueError("original full output changed between rounds")
        reference = streams["oracle"]
        checked.append({"index": i, "status": "FULL_ORACLE_MATCH", "gnu_wall_seconds": gnu_wall,
                        "cpu_seconds": {arm: row["observations"][arm]["user_cpu_seconds"]
                                        + row["observations"][arm]["system_cpu_seconds"] for arm in ARMS}})
    summary = recompute_summary(report["rounds"])
    if summary != report["summary"]:
        raise ValueError("independently recomputed statistics differ")
    return summary, checked


def review_experiment(pin):
    path = Path(pin["report"])
    stage = path.parent.parent.parent
    report = checked_json(path, pin["report_sha256"])
    original_pin = checked_json(stage / "parallel-pin.json", pin["parallel_pin_sha256"])
    if report["schema"] != "atlas-parallel-ab-v1" or report["pin"] != original_pin:
        raise ValueError("experiment provenance changed")
    if inputs(stage) != original_pin["inputs"]:
        raise ValueError("frozen experiment inputs changed")
    # The external scheduler snapshot binds the claimed job, host and CPU count.
    scheduler = Path(pin["scheduler"]["path"])
    if digest(scheduler) != pin["scheduler"]["sha256"]:
        raise ValueError("scheduler snapshot changed")
    expected_scheduler = "|".join((report["job"], "COMPLETED", "0:0", "4", report["node"]))
    if (scheduler.read_text().strip() != expected_scheduler or path.parent.name != report["job"]
            or report["job"] != pin["experiment_job"]):
        raise ValueError("job identity or scheduler completion mismatch")
    check_log = path.parent / "checker.log"
    if (report["checker"]["exit_status"] != 0
            or digest(check_log) != report["checker"]["log_sha256"]
            or not re.search(r"Ran 6 tests in .*\n\nOK\s*$", check_log.read_text())):
        raise ValueError("experiment checker evidence changed")
    lock = json.loads((stage / "tests/math/baseline.json").read_text())
    build, counts = review_build(Path(original_pin["build"]), original_pin["build_sha256"], lock)
    if report["binary_pins"] != build["binaries"] or report["source_files_rehashed"] != counts:
        raise ValueError("runtime binary/source provenance mismatch")
    selected = [case for case in cases(stage)[1] if case["id"] == original_pin["case_id"]]
    if len(selected) != 1 or selected[0]["expected"] != "accept":
        raise ValueError("one positive catalog case required")
    case = selected[0]
    if (report["case"] != {k: v for k, v in case.items() if k != "source"}
            or (path.parent / "input.atlas").read_bytes() != case["source"].encode()):
        raise ValueError("case or expanded input changed")
    summary, rounds = review_rounds(report, path.parent, case, build)
    review_build(Path(original_pin["build"]), original_pin["build_sha256"], lock)
    if inputs(stage) != original_pin["inputs"] or digest(path) != pin["report_sha256"]:
        raise ValueError("experiment mutated during review")
    return {"experiment_job": report["job"], "node": report["node"], "resources": report["resources"],
            "case": report["case"], "source_files_rehashed": counts, "scripts_rehashed": len(build["scripts"]),
            "binaries": build["binaries"], "artifacts_rehashed": 36, "rounds": rounds, "summary": summary,
            "decision": "SELECTED_FOR_THIS_WORKLOAD" if summary["selection"] == "PROMISING_PENDING_REVIEW"
                        else "NOT_SELECTED", "default_thread_setting_changed": False,
            "scope": experiment_scope(case)}


def main():
    if not os.environ.get("SLURM_JOB_ID"):
        raise SystemExit("Parallel artifact review requires an HPC compute allocation")
    root = Path.cwd()
    out = root / "results" / os.environ["SLURM_JOB_ID"]
    out.mkdir(parents=True, exist_ok=False)
    result = {"schema": "atlas-parallel-independent-review-v1", "job": os.environ["SLURM_JOB_ID"],
              "status": "FAIL"}
    try:
        pin = checked_json(root / "review-pin.json", os.environ["REVIEW_PIN_SHA256"])
        result["pin"] = pin
        if (inputs(root) != pin["inputs"]
                or digest(os.environ["PARALLEL_REVIEW_SPOOL"]) != pin["inputs"]["hpc/math_parallel_review.sbatch"]):
            raise ValueError("review source or submitted script changed")
        with (out / "checker.log").open("wb") as log:
            check = subprocess.run([sys.executable, "-m", "unittest", "discover", "-s", "hpc",
                                    "-p", "test_math*.py", "-v"], stdout=log, stderr=subprocess.STDOUT)
        result["checker"] = {"exit_status": check.returncode, "log_sha256": digest(out / "checker.log")}
        if check.returncode:
            raise ValueError("independent checker tests failed")
        result["review"] = review_experiment(pin)
        if inputs(root) != pin["inputs"]:
            raise ValueError("review source changed during execution")
        result["status"] = "PARALLEL_ARTIFACTS_VERIFIED"
    except Exception:
        result["error"] = traceback.format_exc()
    path = out / "review.json"
    path.write_text(json.dumps(result, indent=2, sort_keys=True) + "\n")
    path.with_suffix(".sha256").write_text(digest(path) + "\n")
    print(result["status"], path, digest(path), flush=True)
    return int(result["status"] != "PARALLEL_ARTIFACTS_VERIFIED")


if __name__ == "__main__":
    raise SystemExit(main())

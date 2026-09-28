#!/usr/bin/env python3
"""Versioned mathematical differential catalog; execution is HPC-only.

Keep raw streams and compare complete delimited mathematical output. Loading
and diagnostic differences remain separately visible, never normalized away.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import resource
import subprocess
import time
import traceback

from math_baseline_build import digest, file_manifest


def cases(root):
    catalog = json.loads((root / "tests/math/catalog.json").read_text())
    result = []
    grids = [(catalog["groups"], catalog["operations"] + catalog["rejections"]),
             ([{"type": "Language", "family": "language", "rank": 0, "tier": "small"}],
              catalog.get("language_operations", []))]
    grids.extend((grid.get("groups", catalog["groups"]), grid["operations"])
                 for grid in catalog.get("additional_grids", []))
    for groups, operations in grids:
        for group, operation in ((g, op) for g in groups for op in operations):
            case = dict(group, **operation)
            case["operation"] = operation["id"]
            case["id"] = group["type"] + "_" + operation["id"]
            case["expected"] = "reject" if "diagnostic" in operation else "accept"
            if case["expected"] == "reject":
                case["diagnostic"] += " {}:{}".format(group["rank"], group["rank"] + 1)
            template = root / "tests/math/templates" / operation["template"]
            if template.parent.resolve() != (root / "tests/math/templates").resolve():
                raise ValueError("template must be in the catalog template directory")
            source = template.read_text()
            for token, value in case.get("parameters", {}).items():
                if not re.fullmatch(r"[A-Z_]+", token) or token in ("TYPE", "RANK", "ID"):
                    raise ValueError("invalid additional template parameter")
                source = source.replace("@" + token + "@", str(value))
            for token, value in [("TYPE", group["type"]), ("RANK", group["rank"]),
                                 ("ID", case["id"])]:
                source = source.replace("@" + token + "@", str(value))
            if re.search(r"@[A-Z_]+@", source):
                raise ValueError("unexpanded placeholder")
            case["source"] = source + "\nquit\n"
            case["input_sha256"] = hashlib.sha256(case["source"].encode()).hexdigest()
            case["template_sha256"] = digest(template)
            result.append(case)
    if len({case["id"] for case in result}) != len(result):
        raise ValueError("duplicate case id")
    return catalog, result


def payload(stdout, case_id):
    begin = ("MATH_BEGIN " + case_id + "\n").encode()
    end = ("MATH_END " + case_id + "\n").encode()
    if stdout.count(begin) != 1 or stdout.count(end) != 1:
        return None
    start = stdout.index(begin) + len(begin)
    finish = stdout.index(end)
    if finish <= start:
        return None
    return stdout[start:finish]


def observation_ok(case, record, stdout, stderr):
    if record["timed_out"] or record["exit_status"] in (None, 124, 137, 143):
        return False
    if case["expected"] == "reject":
        combined = stdout + stderr
        unexpected = re.search(rb"(?:Syntax|Lexical|Type|Name|Internal|Program) error", combined, re.I)
        return (record["exit_status"] == 1 and not unexpected
                and combined.count(case["diagnostic"].encode()) == 1
                and ("MATH_BEGIN " + case["id"] + "\n").encode() in stdout)
    return (record["exit_status"] == 0 and not stderr
            and payload(stdout, case["id"]) is not None)


def compare(case, records, streams):
    valid = {engine: observation_ok(case, records[engine], *streams[engine])
             for engine in ("oracle", "rust")}
    if not valid["oracle"]:
        status = failure_status("ORACLE", records["oracle"])
    elif not valid["rust"]:
        status = failure_status("RUST", records["rust"])
    elif case["expected"] == "reject":
        status = "REJECTION_CATEGORY_MATCH"
    elif payload(streams["oracle"][0], case["id"]) == payload(streams["rust"][0], case["id"]):
        status = "MATH_MATCH"
    else:
        status = "MATH_MISMATCH"
    return {"status": status, "valid": valid,
            "full_stdout_equal": streams["oracle"][0] == streams["rust"][0],
            "full_stderr_equal": streams["oracle"][1] == streams["rust"][1],
            "exit_equal": records["oracle"]["exit_status"] == records["rust"]["exit_status"]}


def failure_status(engine, record):
    code = record["exit_status"]
    if code == 124:
        return engine + "_TIMEOUT"
    # Exit 137 can be either SIGKILL/OOM or timeout's escalation. Do not
    # silently call every kill a timeout without scheduler evidence.
    if code is not None and (code < 0 or code >= 128):
        return engine + "_SIGNAL_OR_RESOURCE_FAILURE"
    return engine + "_FAILURE"


def verify_build(build, lock):
    if build["status"] != "BUILT_NOT_DIFFERENTIALLY_VERIFIED" or not build["harness_unchanged"]:
        raise ValueError("baseline build did not complete")
    if build["baseline"] != lock:
        raise ValueError("build differs from catalog baseline lock")
    for entry in build["binaries"].values():
        if digest(entry["path"]) != entry["sha256"]:
            raise ValueError("baseline binary changed")
    scripts = Path(build["binaries"]["oracle"]["path"]).parent / "atlas-scripts"
    if file_manifest(scripts) != build["scripts"]:
        raise ValueError("upstream scripts changed")
    return scripts


def observe(engine, binary, scripts, case, output, timeout):
    paths = {suffix: output / (engine + "." + suffix)
             for suffix in ("stdout", "stderr", "metrics")}
    env = {k: v for k, v in os.environ.items() if not k.startswith("ATLAS_")}
    env.update(LC_ALL="C", RAYON_NUM_THREADS="1", OMP_NUM_THREADS="1",
               OPENBLAS_NUM_THREADS="1")
    def limit():
        resource.setrlimit(resource.RLIMIT_AS, (6 * 1024**3, 6 * 1024**3))
        resource.setrlimit(resource.RLIMIT_CORE, (0, 0))
    command = ["/usr/bin/time", "-v", "-o", str(paths["metrics"]),
               "timeout", "--kill-after=5s", str(timeout), str(binary)]
    start = time.monotonic()
    with paths["stdout"].open("wb") as out, paths["stderr"].open("wb") as err:
        proc = subprocess.run(command, input=case["source"].encode(), cwd=scripts,
                              env=env, stdout=out, stderr=err, preexec_fn=limit)
    elapsed = time.monotonic() - start
    metrics = paths["metrics"].read_text()
    rss = re.search(r"Maximum resident set size \(kbytes\):\s*(\d+)", metrics)
    record = {"engine": engine, "command": command, "exit_status": proc.returncode,
              "timed_out": proc.returncode == 124, "seconds": elapsed,
              "termination_uncertain": proc.returncode < 0 or proc.returncode >= 128,
              "maxrss_kb": int(rss[1]) if rss else None, "maxrss_approximate": False,
              "resource_limit_gib": 6, "timeout_seconds": timeout,
              "artifacts": {key: {"path": str(path), "sha256": digest(path),
                                   "bytes": path.stat().st_size} for key, path in paths.items()}}
    return record, (paths["stdout"].read_bytes(), paths["stderr"].read_bytes())


def run(root, index, build_path, timeout):
    job = os.environ.get("SLURM_JOB_ID")
    if not job:
        raise SystemExit("Differential execution requires a SLURM compute job")
    output = root / "results" / job
    output.mkdir(parents=True, exist_ok=False)
    report = {"schema": "atlas-math-survey-v1", "status": "HARNESS_FAILURE",
              "job_id": job, "node": platform.node(), "case_index": index,
              "performance_scope": "one-shot correctness survey, NOT a stable speed claim",
              "observations": {}}
    try:
        harness = {str(p.relative_to(root)): digest(p)
                   for p in sorted(root.rglob("*")) if p.is_file()
                   and p.relative_to(root).parts[0] in ("hpc", "tests")}
        expected = json.loads((root / "suite-inputs.json").read_text())
        if digest(root / "suite-inputs.json") != os.environ["SUITE_INPUTS_SHA256"]:
            raise ValueError("submission input manifest changed")
        if harness != expected:
            raise ValueError("suite inputs changed")
        if digest(Path(os.environ["MATH_SUITE_SPOOL"])) != harness["hpc/math_suite.sbatch"]:
            raise ValueError("submitted batch script differs")
        report["harness"] = harness
        catalog, all_cases = cases(root)
        if index < 0 or index >= len(all_cases):
            raise ValueError("invalid case index")
        case = all_cases[index]
        report["case"] = {k: v for k, v in case.items() if k != "source"}
        report["open_requirements"] = catalog["open_requirements"]
        (output / "input.atlas").write_text(case["source"])
        lock = json.loads((root / "tests/math/baseline.json").read_text())
        build = json.loads(build_path.read_text())
        report.update(build_report=str(build_path), build_report_sha256=digest(build_path),
                      baseline=lock, binary_pins=build["binaries"])
        scripts = verify_build(build, lock)
        streams = {}
        order = ("oracle", "rust") if index % 2 == 0 else ("rust", "oracle")
        report["order"] = order
        for engine in order:
            report["observations"][engine], streams[engine] = observe(
                engine, build["binaries"][engine]["path"], scripts, case, output, timeout)
        report.update(compare(case, report["observations"], streams))
        verify_build(build, lock)
        for name, sha in harness.items():
            if digest(root / name) != sha:
                raise ValueError("suite input changed during execution")
        if any(x["maxrss_kb"] is None for x in report["observations"].values()):
            raise ValueError("missing peak RSS measurement")
    except Exception:
        report["status"] = "HARNESS_FAILURE"
        report["error"] = traceback.format_exc()
    path = output / "report.json"
    path.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n")
    (output / "report.sha256").write_text(digest(path) + "\n")
    print(report["status"], output, flush=True)
    return 0 if report["status"] in ("MATH_MATCH", "REJECTION_CATEGORY_MATCH") else 1


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--list", action="store_true")
    parser.add_argument("--case", type=int)
    parser.add_argument("--build", type=Path)
    parser.add_argument("--timeout", type=int, default=300)
    args = parser.parse_args()
    root = Path.cwd()
    if args.list:
        for i, case in enumerate(cases(root)[1]):
            print(i, case["id"], case["expected"], case["tier"])
        return 0
    if args.case is None or args.build is None or not 1 <= args.timeout <= 600:
        parser.error("--case and --build required, timeout must be 1..600 seconds")
    return run(root, args.case, args.build.resolve(), args.timeout)


if __name__ == "__main__":
    raise SystemExit(main())

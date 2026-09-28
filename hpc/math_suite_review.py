#!/usr/bin/env python3
"""Independently recheck complete stored artifacts; never rerun an interpreter."""
import argparse
from collections import Counter
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import tarfile

from math_baseline_build import digest, file_manifest
from math_suite import cases
from math_cycle_check import mathematical_evidence


def checked_json(path, expected_sha=None):
    sha = digest(path)
    if expected_sha is not None and sha != expected_sha:
        raise ValueError("wrong externally pinned JSON: " + str(path))
    sidecar = path.with_suffix(".sha256")
    if sidecar.exists() and sidecar.read_text().strip() != sha:
        raise ValueError("wrong JSON checksum: " + str(path))
    return json.loads(path.read_text())


def complete_section(raw, case_id):
    lines = raw.splitlines(keepends=True)
    starts = [i for i, line in enumerate(lines) if line == ("MATH_BEGIN " + case_id + "\n").encode()]
    ends = [i for i, line in enumerate(lines) if line == ("MATH_END " + case_id + "\n").encode()]
    if len(starts) != 1 or len(ends) != 1 or ends[0] <= starts[0] + 1:
        return None
    return b"".join(lines[starts[0] + 1:ends[0]])


def independently_classify(case, entries, streams):
    valid = {}
    for engine in ("oracle", "rust"):
        entry, (out, err) = entries[engine], streams[engine]
        if case["expected"] == "accept":
            valid[engine] = (entry["exit_status"] == 0 and not entry["timed_out"]
                             and not err and complete_section(out, case["id"]) is not None)
        else:
            raw = out + err
            valid[engine] = (entry["exit_status"] == 1 and not entry["timed_out"]
                             and raw.count(case["diagnostic"].encode()) == 1
                             and ("MATH_BEGIN " + case["id"] + "\n").encode() in out
                             and not re.search(rb"(?:Syntax|Lexical|Type|Name|Internal|Program) error", raw, re.I))
    for engine in ("oracle", "rust"):
        if not valid[engine]:
            code = entries[engine]["exit_status"]
            if code == 124:
                return engine.upper() + "_TIMEOUT"
            if code is not None and (code < 0 or code >= 128):
                return engine.upper() + "_SIGNAL_OR_RESOURCE_FAILURE"
            return engine.upper() + "_FAILURE"
        evidence = mathematical_evidence(case, entries[engine], *streams[engine])
        if evidence and evidence["status"] == "FAIL":
            return engine.upper() + "_INVARIANT_FAILURE"
    if case["expected"] == "reject":
        return "REJECTION_CATEGORY_MATCH"
    left = complete_section(streams["oracle"][0], case["id"])
    right = complete_section(streams["rust"][0], case["id"])
    return "MATH_MATCH" if left == right else "MATH_MISMATCH"


def archive_source_manifest(archive):
    """Use the same relative names as an extracted tree, rejecting aliases."""
    archived = {}
    with tarfile.open(archive) as tar:
        for member in tar:
            path = PurePosixPath(member.name)
            if (path.is_absolute() or ".." in path.parts
                    or not (member.isfile() or member.isdir())):
                raise ValueError("unsupported archive member: " + member.name)
            if member.isdir():
                continue
            name = str(path)
            if name == "." or name in archived:
                raise ValueError("empty or duplicate archive file: " + member.name)
            with tar.extractfile(member) as stream:
                h = hashlib.sha256()
                for chunk in iter(lambda: stream.read(1024 * 1024), b""):
                    h.update(chunk)
                archived[name] = h.hexdigest()
    return archived


def review_build(path, expected_sha, lock):
    build = checked_json(path, expected_sha)
    if (build["baseline"] != lock or build["status"] != "BUILT_NOT_DIFFERENTIALLY_VERIFIED"
            or not build["harness_unchanged"]):
        raise ValueError("build provenance mismatch")
    stage = path.parent.parent.parent
    counts = {}
    for engine in ("oracle", "rust"):
        pin = lock[engine]
        archive = stage / pin["archive"]
        if digest(archive) != pin["archive_sha256"]:
            raise ValueError("changed source archive")
        archived = archive_source_manifest(archive)
        if archived != build["source_files"][engine]:
            raise ValueError("build source manifest does not describe pinned archive")
        for name, sha in archived.items():
            if digest(path.parent / (engine + "-source") / name) != sha:
                raise ValueError("changed original source file: " + name)
        counts[engine] = len(archived)
        binary = build["binaries"][engine]
        if digest(binary["path"]) != binary["sha256"]:
            raise ValueError("changed baseline executable")
    for command in build["commands"]:
        if command["exit_status"] != 0 or digest(path.parent / (command["name"] + ".log")) != command["log_sha256"]:
            raise ValueError("invalid build command log")
    for name, sha in build["harness"].items():
        if digest(stage / name) != sha:
            raise ValueError("changed build harness")
    scripts = Path(build["binaries"]["oracle"]["path"]).parent / "atlas-scripts"
    if file_manifest(scripts) != build["scripts"]:
        raise ValueError("changed upstream scripts")
    return build, counts


def review_case(path, expected, build_path, build_sha, build, manifest):
    report = checked_json(path)
    if report["case"] != {k: v for k, v in expected.items() if k != "source"}:
        raise ValueError("case identity/source mismatch")
    if (path.parent / "input.atlas").read_bytes() != expected["source"].encode():
        raise ValueError("expanded input changed")
    if (report["build_report"] != str(build_path) or report["build_report_sha256"] != build_sha
            or report["binary_pins"] != build["binaries"] or report["harness"] != manifest):
        raise ValueError("provenance changed between build and execution")
    streams = {}
    for engine in ("oracle", "rust"):
        observation = report["observations"][engine]
        for suffix in ("stdout", "stderr", "metrics"):
            artifact = path.parent / (engine + "." + suffix)
            pin = observation["artifacts"][suffix]
            if (pin["path"] != str(artifact) or pin["sha256"] != digest(artifact)
                    or pin["bytes"] != artifact.stat().st_size):
                raise ValueError("changed full execution artifact")
        metric = (path.parent / (engine + ".metrics")).read_text()
        rss = re.search(r"Maximum resident set size \(kbytes\):\s*(\d+)", metric)
        if (not rss or int(rss[1]) != observation["maxrss_kb"]
                or observation["maxrss_approximate"] or observation["seconds"] < 0):
            raise ValueError("invalid execution metrics")
        streams[engine] = tuple((path.parent / (engine + "." + suffix)).read_bytes()
                                for suffix in ("stdout", "stderr"))
    status = independently_classify(expected, report["observations"], streams)
    if status != report["status"]:
        raise ValueError("recomputed outcome differs from reported outcome")
    evidence = {e: mathematical_evidence(expected, report["observations"][e], *streams[e])
                for e in ("oracle", "rust")}
    if evidence != report.get("independent_checks"):
        raise ValueError("independently recomputed mathematical evidence changed")
    return {"id": expected["id"], "index": report["case_index"], "job": report["job_id"],
            "group": expected["type"], "operation": expected["operation"], "status": status,
            "independent_checks": evidence,
            "report_sha256": digest(path), "input_sha256": expected["input_sha256"],
            "metrics": {e: {k: o[k] for k in ("exit_status", "seconds", "maxrss_kb", "maxrss_approximate")}
                        for e, o in report["observations"].items()},
            "full_stdout_equal": streams["oracle"][0] == streams["rust"][0],
            "full_stderr_equal": streams["oracle"][1] == streams["rust"][1],
            "first_diagnostics": {e: data[1].decode(errors="replace").splitlines()[:5]
                                  for e, data in streams.items()}}


def main():
    if not os.environ.get("SLURM_JOB_ID"):
        raise SystemExit("Artifact verification runs on an HPC compute node")
    parser = argparse.ArgumentParser()
    parser.add_argument("--stage", type=Path, required=True)
    parser.add_argument("--inputs-sha", required=True)
    parser.add_argument("--build", type=Path, required=True)
    parser.add_argument("--build-sha", required=True)
    parser.add_argument("--indices", required=True, help="Explicit comma-separated case indices")
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if args.output.exists():
        raise SystemExit("Refusing to replace an existing review")
    manifest = checked_json(args.stage / "suite-inputs.json", args.inputs_sha)
    actual = {str(p.relative_to(args.stage)): digest(p) for p in sorted(args.stage.rglob("*"))
              if p.is_file() and p.relative_to(args.stage).parts[0] in ("hpc", "tests")}
    if manifest != actual:
        raise ValueError("frozen suite inputs changed")
    lock = json.loads((args.stage / "tests/math/baseline.json").read_text())
    build, source_counts = review_build(args.build, args.build_sha, lock)
    catalog, case_list = cases(args.stage)
    wanted = {int(x) for x in args.indices.split(",")}
    found = {}
    for path in sorted((args.stage / "results").glob("*/report.json")):
        report = json.loads(path.read_text())
        if report.get("schema") != "atlas-math-survey-v1" or report["case_index"] not in wanted:
            continue
        index = report["case_index"]
        if index in found:
            raise ValueError("duplicate case observation, specify a distinct frozen stage")
        found[index] = review_case(path, case_list[index], args.build, args.build_sha, build, manifest)
    if set(found) != wanted:
        raise ValueError("incomplete observation set: " + str(sorted(wanted - set(found))))
    reviewed = [found[index] for index in sorted(found)]
    summary = {"schema": "atlas-math-independent-review-v1", "status": "ARTIFACTS_VERIFIED_NOT_ALL_MATH_PASS",
               "job": os.environ["SLURM_JOB_ID"], "stage": str(args.stage),
               "build_report_sha256": args.build_sha, "source_files_rehashed": source_counts,
               "scripts_rehashed": len(build["scripts"]), "binaries": build["binaries"],
               "input_manifest_sha256": args.inputs_sha, "cases": reviewed,
               "counts": dict(Counter(case["status"] for case in reviewed)),
               "open_requirements": catalog["open_requirements"],
               "performance_scope": "single-shot observations; no stable speedup claimed"}
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(summary, indent=2, sort_keys=True) + "\n")
    args.output.with_suffix(".sha256").write_text(digest(args.output) + "\n")
    print(json.dumps({"status": summary["status"], "counts": summary["counts"],
                      "review_sha256": digest(args.output), "output": str(args.output)}), flush=True)


if __name__ == "__main__":
    main()

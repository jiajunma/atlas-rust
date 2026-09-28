#!/usr/bin/env python3
"""Capture current-oracle generic contracts without blessing Rust rejections."""
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import subprocess
import sys
import traceback

from math_baseline_build import digest
from math_suite import observe, payload
from math_suite_review import review_build


def load_cases(root):
    directory = root / "tests/math/generics"
    catalog = json.loads((directory / "catalog.json").read_text())
    result, seen = [], set()
    for entry in catalog["cases"]:
        if (not re.fullmatch(r"[a-z][a-z0-9_]*", entry["id"])
                or entry["id"] in seen or entry["intent"] not in ("accept", "reject")):
            raise ValueError("invalid or duplicate generic case")
        seen.add(entry["id"])
        path = directory / entry["file"]
        if path.parent.resolve() != directory.resolve() or path.suffix != ".atlas":
            raise ValueError("generic source must stay in fixture directory")
        case_id = "generic_" + entry["id"]
        source = ('prints("MATH_BEGIN ' + case_id + '")\n' + path.read_text()
                  + '\nprints("MATH_END ' + case_id + '")\nquit\n')
        result.append(dict(entry, id=case_id, source=source,
                           source_sha256=hashlib.sha256(source.encode()).hexdigest(),
                           fixture_sha256=digest(path)))
    return result


def observed_category(case, record, streams):
    out, err = streams
    code = record["exit_status"]
    if record["timed_out"] or code == 124:
        return "TIMEOUT"
    if code is None or code < 0 or code >= 128:
        return "RESOURCE_OR_SIGNAL_FAILURE"
    if code == 0 and not err and payload(out, case["id"]) is not None:
        return "ACCEPTED"
    kinds = re.findall(rb"(Syntax|Lexical|Type|Name|Runtime|Internal|Program) error",
                       out + err, re.I)
    if code == 1 and kinds:
        return "REJECTED_" + "+".join(sorted({x.decode().upper() for x in kinds}))
    return "OTHER_FAILURE"


def main():
    if not os.environ.get("SLURM_JOB_ID"):
        raise SystemExit("Generic contract capture requires an HPC compute node")
    root = Path.cwd()
    out = root / "results" / os.environ["SLURM_JOB_ID"]
    out.mkdir(parents=True, exist_ok=False)
    report = {"schema": "atlas-generic-contract-capture-v1", "status": "FAIL",
              "job": os.environ["SLURM_JOB_ID"], "node": platform.node(), "cases": []}
    try:
        if digest(root / "probe.json") != os.environ["PROBE_SHA256"]:
            raise ValueError("contract-capture pin changed")
        pin = json.loads((root / "probe.json").read_text())
        report["pin"] = pin
        actual = {str(p.relative_to(root)): digest(p) for p in sorted(root.rglob("*"))
                  if p.is_file() and p.relative_to(root).parts[0] in ("hpc", "tests")}
        if actual != pin["inputs"]:
            raise ValueError("contract-capture input changed")
        if digest(os.environ["MATH_GENERIC_SPOOL"]) != pin["inputs"]["hpc/math_generic_probe.sbatch"]:
            raise ValueError("submitted batch script changed")
        with (out / "checker.log").open("wb") as log:
            checks = subprocess.run([sys.executable, "-m", "unittest", "discover", "-s", "hpc",
                                     "-p", "test_math_generic_probe.py", "-v"],
                                    stdout=log, stderr=subprocess.STDOUT)
        report["checker"] = {"exit_status": checks.returncode,
                             "log_sha256": digest(out / "checker.log")}
        if checks.returncode:
            raise ValueError("generic capture checker tests failed")
        baseline = json.loads((root / "tests/math/baseline.json").read_text())
        build, counts = review_build(Path(pin["build"]), pin["build_sha256"], baseline)
        report["source_files_rehashed"] = counts
        report["binary_pins"] = build["binaries"]
        scripts = Path(build["binaries"]["oracle"]["path"]).parent / "atlas-scripts"
        for index, case in enumerate(load_cases(root)):
            folder = out / case["id"]
            folder.mkdir()
            (folder / "input.atlas").write_text(case["source"])
            record = {"case": {k: v for k, v in case.items() if k != "source"},
                      "observations": {}, "categories": {}, "first_diagnostics": {}}
            streams = {}
            for engine in (("oracle", "rust") if index % 2 == 0 else ("rust", "oracle")):
                observation, stream = observe(engine, build["binaries"][engine]["path"],
                                               scripts, case, folder, 45)
                record["observations"][engine] = observation
                record["categories"][engine] = observed_category(case, observation, stream)
                record["first_diagnostics"][engine] = stream[1].decode(errors="replace").splitlines()[:12]
                streams[engine] = stream
            record["full_stdout_equal"] = streams["oracle"][0] == streams["rust"][0]
            record["full_stderr_equal"] = streams["oracle"][1] == streams["rust"][1]
            record["oracle_matches_provisional_intent"] = (
                record["categories"]["oracle"] == "ACCEPTED" if case["intent"] == "accept"
                else record["categories"]["oracle"].startswith("REJECTED_"))
            report["cases"].append(record)
        review_build(Path(pin["build"]), pin["build_sha256"], baseline)
        if any(digest(root / name) != sha for name, sha in pin["inputs"].items()):
            raise ValueError("input changed during capture")
        report["status"] = "CAPTURED_NOT_YET_ACCEPTED_AS_LANGUAGE_CONTRACTS"
        report["scope"] = ("All raw streams, time and RSS retained. Reject intent is provisional; "
                           "a Rust syntax error must not count as matching an oracle type error. "
                           "This capture is not mathematical or language-feature acceptance.")
    except Exception:
        report["error"] = traceback.format_exc()
    path = out / "report.json"
    path.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n")
    path.with_suffix(".sha256").write_text(digest(path) + "\n")
    print(report["status"], path, flush=True)
    return int(report["status"] == "FAIL")


if __name__ == "__main__":
    raise SystemExit(main())

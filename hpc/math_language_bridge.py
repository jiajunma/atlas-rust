"""Three-arm capture for active parser/analyzer migration; never bless mismatches."""
import json
import os
from pathlib import Path
import platform
import re
import subprocess
import sys

from math_baseline_build import digest, file_manifest
from math_generic_probe import load_cases, observed_category
from math_suite import observe
from math_suite_review import review_build


def oracle_execution_valid(cases):
    return bool(cases) and all(
        c["categories"]["oracle"] == "ACCEPTED"
        or c["categories"]["oracle"].startswith("REJECTED_") for c in cases)


def foundation_completion_status(capture_report):
    """Keep build evidence usable without declaring an unavailable oracle valid."""
    cases = capture_report.get("cases", [])
    valid = oracle_execution_valid(cases)
    status = capture_report.get("status")
    if status == "CAPTURED_NOT_LANGUAGE_ACCEPTANCE" and valid:
        return "FOUNDATION_UNITS_PASS_LANGUAGE_NOT_PORTED"
    if status == "CAPTURE_FAILED_ORACLE_EXECUTION" and cases and not valid:
        return "FOUNDATION_UNITS_PASS_ORACLE_UNAVAILABLE"
    raise ValueError("capture status contradicts its oracle execution evidence")


def candidate_unit_gate_passed(parent):
    # These are only candidate eligibility states. Callers must still rehash
    # source, inputs, binaries and every successful/expected-failure command.
    if parent.get("status") == "FOUNDATION_UNITS_PASS_LANGUAGE_NOT_PORTED":
        return True
    return (parent.get("status") == "FOUNDATION_UNITS_PASS_ORACLE_UNAVAILABLE"
            and parent.get("source_integrity_rechecked") is True)


def capture(root, out, pin, candidate):
    # The caller pins all harness/fixture files, exact candidate sources,
    # its fresh target and successful build log before entering this helper.
    baseline = json.loads((root / "tests/math/baseline.json").read_text())
    build, counts = review_build(Path(pin["build"]), pin["build_sha256"], baseline)
    scripts = Path(build["binaries"]["oracle"]["path"]).parent / "atlas-scripts"
    binaries = {"oracle": build["binaries"]["oracle"],
                "rust_before": build["binaries"]["rust"],
                "rust": {"path": str(candidate), "sha256": digest(candidate)}}
    checker_log = out / "capture-checker.log"
    with checker_log.open("wb") as log:
        checks = subprocess.run([sys.executable, "-m", "unittest", "discover", "-s", "hpc",
                                 "-p", "test_math_generic_probe.py", "-v"],
                                stdout=log, stderr=subprocess.STDOUT, cwd=root)
    if checks.returncode or not re.search(r"Ran 6 tests.*\n\nOK", checker_log.read_text(), re.S):
        raise ValueError("capture checker tests did not pass")
    report = {"schema": "atlas-language-bridge-capture-v1", "cases": [],
              "source_files_rehashed": counts, "binary_pins": binaries,
              "checker_log_sha256": digest(checker_log),
              "runtime_library_path": os.environ.get("LD_LIBRARY_PATH", ""),
              "performance_scope": "debug candidate versus release before/oracle; correctness discovery only, no speed ratio"}
    for index, case in enumerate(load_cases(root)):
        folder = out / case["id"]
        folder.mkdir()
        (folder / "input.atlas").write_text(case["source"])
        entry = {"case": {k: v for k, v in case.items() if k != "source"},
                 "observations": {}, "categories": {}, "first_diagnostics": {}}
        streams = {}
        order = ("oracle", "rust_before", "rust") if index % 2 == 0 else ("rust", "rust_before", "oracle")
        entry["order"] = order
        for engine in order:
            observation, stream = observe(engine, binaries[engine]["path"], scripts, case, folder, 45)
            entry["observations"][engine] = observation
            entry["categories"][engine] = observed_category(case, observation, stream)
            entry["first_diagnostics"][engine] = stream[1].decode(errors="replace").splitlines()[:12]
            streams[engine] = stream
        entry["oracle_matches_provisional_intent"] = (
            entry["categories"]["oracle"] == "ACCEPTED" if case["intent"] == "accept"
            else entry["categories"]["oracle"].startswith("REJECTED_"))
        entry["full_stdout_equal"] = streams["oracle"][0] == streams["rust"][0]
        entry["full_stderr_equal"] = streams["oracle"][1] == streams["rust"][1]
        entry["before_full_stdout_equal"] = streams["oracle"][0] == streams["rust_before"][0]
        entry["before_full_stderr_equal"] = streams["oracle"][1] == streams["rust_before"][1]
        report["cases"].append(entry)
    review_build(Path(pin["build"]), pin["build_sha256"], baseline)
    if any(digest(b["path"]) != b["sha256"] for b in binaries.values()):
        raise ValueError("capture executable changed")
    report["status"] = ("CAPTURED_NOT_LANGUAGE_ACCEPTANCE" if oracle_execution_valid(report["cases"])
                        else "CAPTURE_FAILED_ORACLE_EXECUTION")
    return report


def main():
    """Replay only the capture, retaining the unchanged verified build/source."""
    import traceback
    if not os.environ.get("SLURM_JOB_ID"):
        raise SystemExit("Language capture replay requires an HPC compute node")
    root = Path.cwd()
    out = root / "results" / os.environ["SLURM_JOB_ID"]
    out.mkdir(parents=True, exist_ok=False)
    report = {"schema": "atlas-language-bridge-replay-v1", "status": "FAIL",
              "job": os.environ["SLURM_JOB_ID"], "node": platform.node()}
    try:
        if digest(root / "pin.json") != os.environ["BRIDGE_PIN_SHA256"]:
            raise ValueError("replay pin changed")
        pin = json.loads((root / "pin.json").read_text())
        report["pin"] = pin
        for name, sha in pin["inputs"].items():
            if digest(root / name) != sha:
                raise ValueError("replay input changed: " + name)
        if digest(os.environ["BRIDGE_SPOOL"]) != pin["inputs"]["hpc/math_language_bridge.sbatch"]:
            raise ValueError("replay batch script changed")
        parent_path = Path(pin["parent_report"])
        if digest(parent_path) != pin["parent_report_sha256"]:
            raise ValueError("build evidence changed")
        parent = json.loads(parent_path.read_text())
        if not candidate_unit_gate_passed(parent):
            raise ValueError("candidate build checks did not pass")
        source = parent_path.parent / "source"
        if file_manifest(source) != parent["source_files"]:
            raise ValueError("verified candidate sources changed")
        for command in parent["commands"]:
            if (command["exit_status"] != command["expected_exit"]
                    or digest(parent_path.parent / (command["name"] + ".log")) != command["log_sha256"]):
                raise ValueError("candidate command evidence changed")
        for name, sha in parent["pin"]["inputs"].items():
            if digest(parent_path.parent.parent.parent / name) != sha:
                raise ValueError("parent build input changed")
        binary = parent["language_capture"]["binary_pins"]["rust"]
        if digest(binary["path"]) != binary["sha256"]:
            raise ValueError("candidate binary changed")
        with (out / "bridge-checker.log").open("wb") as log:
            checks = subprocess.run([sys.executable, "-m", "unittest", "discover", "-s", "hpc",
                                     "-p", "test_math_language_bridge.py", "-v"],
                                    stdout=log, stderr=subprocess.STDOUT)
        if checks.returncode:
            raise ValueError("bridge checker tests failed")
        report["bridge_checker_log_sha256"] = digest(out / "bridge-checker.log")
        report["candidate_source_files_rehashed"] = len(parent["source_files"])
        report["language_capture"] = capture(root, out, parent["pin"]["language_capture"],
                                              Path(binary["path"]))
        if (file_manifest(source) != parent["source_files"]
                or digest(parent_path) != pin["parent_report_sha256"]
                or any(digest(root / name) != sha for name, sha in pin["inputs"].items())):
            raise ValueError("replay sources or inputs changed")
        report["status"] = report["language_capture"]["status"]
    except Exception:
        report["error"] = traceback.format_exc()
    path = out / "report.json"
    path.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n")
    path.with_suffix(".sha256").write_text(digest(path) + "\n")
    print(report["status"], path, flush=True)
    return int(report["status"] != "CAPTURED_NOT_LANGUAGE_ACCEPTANCE")


if __name__ == "__main__":
    raise SystemExit(main())

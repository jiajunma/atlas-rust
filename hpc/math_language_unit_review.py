"""Recheck a frozen language candidate's full core suite without rebuilding it."""
import json
import os
from pathlib import Path
import platform
import re
import subprocess
import sys
import time
import traceback

from math_baseline_build import digest, file_manifest


KNOWN_FAILURES = {
    "typed::tests::polymorphic_empty_global_is_not_assignable": "POLYMORPHIC_GLOBAL_REGRESSION constant=false rejected=false",
    "typed::tests::polymorphic_empty_local_is_not_assignable": "POLYMORPHIC_LOCAL_REGRESSION rejected=false",
}


def inventory(text):
    names = re.findall(r"^([^\s]+): test$", text, re.M)
    if len(names) != len(set(names)) or not set(KNOWN_FAILURES).issubset(names):
        raise ValueError("missing regression or duplicate test in inventory")
    if len(names) <= len(KNOWN_FAILURES):
        raise ValueError("no passing-suite tests inventoried")
    return names


def check_summary(text, passed, failed, filtered):
    status = "FAILED" if failed else "ok"
    pattern = (rf"^test result: {status}\. {passed} passed; {failed} failed; "
               rf"0 ignored; 0 measured; {filtered} filtered out;")
    if len(re.findall(pattern, text, re.M)) != 1:
        raise ValueError("test summary differs from exact inventory")


def main():
    if not os.environ.get("SLURM_JOB_ID"):
        raise SystemExit("Core unit review requires an HPC compute node")
    root = Path.cwd()
    out = root / "results" / os.environ["SLURM_JOB_ID"]
    out.mkdir(parents=True, exist_ok=False)
    report = {"schema": "atlas-language-core-review-v1", "status": "FAIL",
              "job": os.environ["SLURM_JOB_ID"], "node": platform.node(), "commands": []}
    try:
        if digest(root / "pin.json") != os.environ["CORE_REVIEW_PIN_SHA256"]:
            raise ValueError("review pin changed")
        pin = json.loads((root / "pin.json").read_text())
        report["pin"] = pin
        for name, sha in pin["inputs"].items():
            if digest(root / name) != sha:
                raise ValueError("review input changed: " + name)
        if digest(os.environ["CORE_REVIEW_SPOOL"]) != pin["inputs"]["hpc/math_language_unit_review.sbatch"]:
            raise ValueError("submitted review changed")
        parent_path = Path(pin["parent_report"])
        if digest(parent_path) != pin["parent_report_sha256"]:
            raise ValueError("parent report changed")
        parent = json.loads(parent_path.read_text())
        if parent["status"] != "FOUNDATION_UNITS_PASS_LANGUAGE_NOT_PORTED":
            raise ValueError("parent build is not verified")
        source = parent_path.parent / "source"
        binary = Path(pin["test_binary"])
        if file_manifest(source) != parent["source_files"]:
            raise ValueError("candidate source changed")
        if digest(binary) != pin["test_binary_sha256"]:
            raise ValueError("test binary changed")
        for command in parent["commands"]:
            if command["exit_status"] != command["expected_exit"]:
                raise ValueError("parent command failed")
            for suffix in ("log", "time"):
                if digest(parent_path.parent / (command["name"] + "." + suffix)) != command[suffix + "_sha256"]:
                    raise ValueError("parent command artifact changed")
        unit_log = (parent_path.parent / "type-units.log").read_text()
        if not re.search(r"Running unittests src/lib\.rs \(" + re.escape(str(binary)) + r"\)", unit_log):
            raise ValueError("binary is not the executed parent unit target")
        for name, sha in parent["pin"]["inputs"].items():
            if digest(parent_path.parent.parent.parent / name) != sha:
                raise ValueError("parent source staging input changed")
        env = {k: v for k, v in os.environ.items()
               if not k.startswith(("ATLAS_", "RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS"))}
        env["RAYON_NUM_THREADS"] = "1"

        def run(name, argv, cwd=source, expected=0):
            started = time.monotonic()
            timing = out / (name + ".time")
            log_path = out / (name + ".log")
            with log_path.open("wb") as log:
                result = subprocess.run(["/usr/bin/time", "-v", "-o", str(timing),
                                         "timeout", "--kill-after=15s", "900"] + argv,
                                        cwd=cwd, env=env, stdout=log, stderr=subprocess.STDOUT)
            rss = re.search(r"Maximum resident set size \(kbytes\):\s*(\d+)", timing.read_text())
            report["commands"].append({"name": name, "argv": argv, "cwd": str(cwd),
                "exit_status": result.returncode, "expected_exit": expected,
                "seconds": time.monotonic() - started,
                "maxrss_kb": int(rss.group(1)) if rss else None, "maxrss_approximate": False,
                "log_sha256": digest(log_path), "time_sha256": digest(timing)})
            if result.returncode != expected:
                raise ValueError(name + " unexpected exit")
            return log_path.read_text()

        checker = run("checker", [sys.executable, "-m", "unittest", "discover", "-s", "hpc",
                                  "-p", "test_math_language_unit_review.py", "-v"], cwd=root)
        if not re.search(r"Ran 3 tests.*\n\nOK", checker, re.S):
            raise ValueError("review checker did not execute its tests")
        names = inventory(run("inventory", [str(binary), "--list"]))
        report["test_count"] = len(names)
        report["source_files_rehashed"] = len(parent["source_files"])
        args = [str(binary), "--test-threads=2", "--nocapture"]
        for name in KNOWN_FAILURES:
            args.extend(["--skip", name])
        log = run("core-suite", args)
        check_summary(log, len(names) - len(KNOWN_FAILURES), 0, len(KNOWN_FAILURES))
        for index, (name, marker) in enumerate(KNOWN_FAILURES.items()):
            log = run("retained-failure-" + str(index),
                      [str(binary), name, "--exact", "--nocapture"], expected=101)
            check_summary(log, 0, 1, len(names) - 1)
            if marker not in log or "polymorphic " not in log:
                raise ValueError("retained regression did not reach its intended assertion")
        if (file_manifest(source) != parent["source_files"]
                or digest(binary) != pin["test_binary_sha256"]
                or digest(parent_path) != pin["parent_report_sha256"]
                or any(digest(root / n) != h for n, h in pin["inputs"].items())):
            raise ValueError("review inputs changed during execution")
        report["status"] = "CORE_SUITE_PASS_EXCEPT_TWO_REPRODUCED_KNOWN_FAILURES"
    except Exception:
        report["error"] = traceback.format_exc()
    path = out / "report.json"
    path.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n")
    path.with_suffix(".sha256").write_text(digest(path) + "\n")
    print(report["status"], path, flush=True)
    return int(report["status"] == "FAIL")


if __name__ == "__main__":
    raise SystemExit(main())

#!/usr/bin/env python3
"""Pinned HPC-only checks for the generic type foundation, not language acceptance."""
import json
import os
from pathlib import Path
import platform
import re
import shutil
import subprocess
import sys
import time
import traceback

from math_baseline_build import digest, file_manifest, unpack


def main():
    if not os.environ.get("SLURM_JOB_ID"):
        raise SystemExit("Type checks must run on an HPC compute node")
    root = Path.cwd()
    out = root / "results" / os.environ["SLURM_JOB_ID"]
    out.mkdir(parents=True, exist_ok=False)
    report = {"schema": "atlas-type-foundation-v1", "status": "FAIL",
              "job": os.environ["SLURM_JOB_ID"], "node": platform.node(), "commands": []}
    try:
        if digest(root / "pin.json") != os.environ["TYPE_PIN_SHA256"]:
            raise ValueError("pin changed")
        pin = json.loads((root / "pin.json").read_text())
        report["pin"] = pin
        for name, sha in pin["inputs"].items():
            if digest(root / name) != sha:
                raise ValueError("input changed: " + name)
        if digest(os.environ["TYPE_SPOOL"]) != pin["inputs"]["hpc/math_type_foundation.sbatch"]:
            raise ValueError("submitted script changed")
        if digest(pin["parent_report"]) != pin["parent_report_sha256"]:
            raise ValueError("parent evidence changed")
        parent = json.loads(Path(pin["parent_report"]).read_text())
        if parent["status"] != "EXPECTED_PANIC_REPRODUCED_NOT_REPAIRED":
            raise ValueError("expected the retained unchanged-runtime global regression")
        if digest(pin["parent_archive"]) != pin["parent_archive_sha256"]:
            raise ValueError("parent source archive changed")
        source = out / "source"
        unpack(pin["parent_archive"], source)
        if file_manifest(source) != parent["source_files"]:
            raise ValueError("parent archive and executed source disagree")
        with (root / "candidate.patch").open("rb") as patch, (out / "patch.log").open("wb") as log:
            subprocess.run(["patch", "--no-backup-if-mismatch", "--batch", "-p1"],
                           cwd=source, stdin=patch, stdout=log, stderr=subprocess.STDOUT, check=True)
        expected = dict(parent["source_files"])
        expected.update(pin["changed_files"])
        if file_manifest(source) != expected:
            raise ValueError("candidate source differs beyond the explicitly pinned files")
        report["source_files"] = expected
        env = {k: v for k, v in os.environ.items()
               if not k.startswith(("ATLAS_", "RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS"))}
        env.update(CARGO_TARGET_DIR=str(out / "target"), CARGO_BUILD_JOBS="2",
                   CARGO_PROFILE_TEST_DEBUG="0")
        report["target_dir"] = env["CARGO_TARGET_DIR"]

        def run(name, argv, expected_exit=0, before=False):
            start = time.monotonic()
            timing = out / (name + ".time")
            command_env = dict(env)
            command_source = source
            if before:
                command_source = out / "source-before"
                command_env["CARGO_TARGET_DIR"] = str(out / "target-before")
            with (out / (name + ".log")).open("wb") as log:
                proc = subprocess.run(["/usr/bin/time", "-v", "-o", str(timing),
                                       "timeout", "--kill-after=15s", "1500"] + argv,
                                      cwd=command_source, env=command_env, stdout=log, stderr=subprocess.STDOUT)
            rss = re.search(r"Maximum resident set size \(kbytes\):\s*(\d+)", timing.read_text())
            report["commands"].append({"name": name, "argv": argv, "exit_status": proc.returncode,
                                       "expected_exit": expected_exit, "seconds": time.monotonic() - start,
                                       "cwd": str(command_source),
                                       "target_dir": command_env["CARGO_TARGET_DIR"],
                                       "maxrss_kb": int(rss.group(1)) if rss else None,
                                       "maxrss_approximate": False,
                                       "log_sha256": digest(out / (name + ".log")),
                                       "time_sha256": digest(timing)})
            if proc.returncode != expected_exit:
                raise ValueError(name + " unexpected exit: " + str(proc.returncode))
            return (out / (name + ".log")).read_text()

        run("rustc-version", ["rustc", "-vV"])
        run("cargo-version", ["cargo", "-V"])
        if "before_types" in pin:
            # A focused before/after proof uses the same new regression body;
            # only the types.rs implementation is restored for the before run.
            previous = pin["before_types"]
            if digest(previous["path"]) != previous["sha256"]:
                raise ValueError("before implementation changed")
            before_source = out / "source-before"
            shutil.copytree(source, before_source)
            module = "crates/atlas-core/src/types.rs"
            shutil.copyfile(previous["path"], before_source / module)
            before_expected = dict(expected)
            before_expected[module] = previous["sha256"]
            if file_manifest(before_source) != before_expected:
                raise ValueError("only the type implementation may change before/after")
            unit = "types::polymorphic::tests::specialising_applied_constructor_exposes_and_refines_its_structure"
            log = run("applied-structure-before", ["cargo", "test", "--offline", "--locked", "-p",
                      "atlas-core", "--lib", unit, "--", "--exact", "--nocapture"],
                      expected_exit=101, before=True)
            if ("test result: FAILED. 0 passed; 1 failed;" not in log
                    or "assertion `left == right` failed" not in log
                    or "Applied(TypeNumber(0), [Undetermined])" not in log):
                raise ValueError("before must execute the unreplaced applied-type assertion")
            report["before_source_files"] = before_expected
        test_filters = [("type-units", "types::"), ("coercion-units", "coercions::tests::")]
        test_filters.extend(pin.get("additional_test_filters", []))
        if (len({name for name, _ in test_filters}) != len(test_filters)
                or any(not re.fullmatch(r"[a-z][a-z-]*", name) for name, _ in test_filters)):
            raise ValueError("invalid or duplicate test command name")
        for name, test_filter in test_filters:
            log = run(name, ["cargo", "test", "--offline", "--locked", "-p", "atlas-core", "--lib",
                             test_filter, "--", "--nocapture"])
            match = re.search(r"test result: ok\. (\d+) passed; 0 failed;", log)
            if not match or int(match.group(1)) == 0:
                raise ValueError(name + " did not execute passing tests")
        # Keep the before regression untouched. This foundation does not yet
        # connect schemes to global bindings or claim to repair the language.
        log = run("known-global-failure", ["cargo", "test", "--offline", "--locked", "-p", "atlas-core",
                  "--lib", "typed::tests::polymorphic_empty_global_is_not_assignable",
                  "--", "--exact", "--nocapture"], expected_exit=101)
        if ("test result: FAILED. 0 passed; 1 failed;" not in log
                or "POLYMORPHIC_GLOBAL_REGRESSION constant=false rejected=false" not in log
                or "polymorphic global must reject assignment" not in log):
            raise ValueError("known language regression changed unexpectedly")
        run("cli-check", ["cargo", "check", "--offline", "--locked", "-p", "atlas-cli"])
        if pin.get("full_core_review", False):
            from math_language_unit_review import KNOWN_FAILURES, check_summary, inventory
            checker = run("core-review-checker", [sys.executable,
                str(root / "hpc/test_math_language_unit_review.py"), "-v"])
            if not re.search(r"Ran 3 tests.*\n\nOK", checker, re.S):
                raise ValueError("full-core checker did not execute its tests")
            test = ["cargo", "test", "--offline", "--locked", "-p", "atlas-core", "--lib"]
            names = inventory(run("core-inventory", test + ["--", "--list"]))
            args = test + ["--", "--test-threads=2", "--nocapture"]
            for name in KNOWN_FAILURES:
                args.extend(["--skip", name])
            log = run("core-suite", args)
            check_summary(log, len(names) - len(KNOWN_FAILURES), 0, len(KNOWN_FAILURES))
            for index, (name, marker) in enumerate(KNOWN_FAILURES.items()):
                log = run("retained-failure-" + str(index),
                          test + [name, "--", "--exact", "--nocapture"], expected_exit=101)
                check_summary(log, 0, 1, len(names) - 1)
                if marker not in log or "polymorphic " not in log:
                    raise ValueError("retained regression missed its intended assertion")
            report["full_core_review"] = {
                "test_count": len(names), "passed": len(names) - len(KNOWN_FAILURES),
                "known_failures_executed": list(KNOWN_FAILURES), "ignored": 0,
            }
        if "language_capture" in pin:
            run("cli-build", ["cargo", "build", "--offline", "--locked", "-p", "atlas-cli"])
            from math_language_bridge import capture
            report["language_capture"] = capture(root, out, pin["language_capture"],
                                                  out / "target/debug/atlas-cli")
            if report["language_capture"]["status"] != "CAPTURED_NOT_LANGUAGE_ACCEPTANCE":
                raise ValueError("oracle execution failed; capture is not differential evidence")
        # Emit formatting as an artifact; never modify the pinned tested source.
        for index, name in enumerate(n for n in pin["changed_files"] if n.endswith(".rs")):
            run("formatted-" + str(index), ["rustfmt", "--edition", "2021", "--config",
                "skip_children=true", "--emit", "stdout", name])
        if file_manifest(source) != expected:
            raise ValueError("tested source changed")
        if "before_source_files" in report and file_manifest(out / "source-before") != report["before_source_files"]:
            raise ValueError("before source changed during checks")
        if any(digest(root / n) != h for n, h in pin["inputs"].items()):
            raise ValueError("inputs changed during checks")
        report["status"] = "FOUNDATION_UNITS_PASS_LANGUAGE_NOT_PORTED"
    except Exception:
        report["error"] = traceback.format_exc()
    path = out / "report.json"
    path.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n")
    path.with_suffix(".sha256").write_text(digest(path) + "\n")
    print(report["status"], path, flush=True)
    return int(report["status"] == "FAIL")


if __name__ == "__main__":
    raise SystemExit(main())

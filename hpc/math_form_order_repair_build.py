#!/usr/bin/env python3
"""HPC-only before/after proof for fundamental-fiber external numbering."""
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import time
import traceback

from math_baseline_build import digest, file_manifest, unpack
from math_kl_repair_build import build_environments, without_added_regression
from math_suite_review import review_build


def main():
    if not os.environ.get("SLURM_JOB_ID"):
        raise SystemExit("Repair builds and tests require an HPC compute node")
    root = Path.cwd()
    out = root / "results" / os.environ["SLURM_JOB_ID"]
    out.mkdir(parents=True, exist_ok=False)
    manifest = json.loads((root / "suite-inputs.json").read_text())
    if digest(root / "suite-inputs.json") != os.environ["SUITE_INPUTS_SHA256"]:
        raise ValueError("input manifest changed")
    for name, sha in manifest.items():
        if digest(root / name) != sha:
            raise ValueError("repair harness changed")
    if digest(os.environ["MATH_BUILD_SPOOL"]) != manifest["hpc/math_form_order_repair_build.sbatch"]:
        raise ValueError("submitted script changed")
    lock = json.loads((root / "tests/math/baseline.json").read_text())
    report = {"schema": "atlas-math-build-v1", "job_id": os.environ["SLURM_JOB_ID"],
              "node": platform.node(), "baseline": lock, "status": "FAIL",
              "commands": [], "source_files": {}, "binaries": {}, "harness": manifest}
    try:
        parent, _ = review_build(Path(lock["repair"]["parent_build"]),
                                 lock["repair"]["parent_build_sha256"],
                                 lock["repair"]["parent_baseline"])
        if lock["oracle"] != parent["baseline"]["oracle"]:
            raise ValueError("reference must stay unchanged")
        for engine in ("oracle", "rust"):
            archive = root / lock[engine]["archive"]
            if digest(archive) != lock[engine]["archive_sha256"]:
                raise ValueError("source archive changed")
            unpack(archive, out / (engine + "-source"))
            report["source_files"][engine] = file_manifest(out / (engine + "-source"))
        before_archive = root / "rust-before.tar"
        if digest(before_archive) != lock["repair"]["before_archive_sha256"]:
            raise ValueError("before archive changed")
        unpack(before_archive, out / "rust-before")
        before = file_manifest(out / "rust-before")
        after = report["source_files"]["rust"]
        previous = parent["source_files"]["rust"]
        module = "crates/atlas-real-group/src/real_form_order.rs"
        if (before.keys() != after.keys() or before.keys() != previous.keys()
                or {n for n in before if before[n] != after[n]} != {module}
                or {n for n in before if before[n] != previous[n]} != {module}):
            raise ValueError("only external-form order may change")
        before_text = (out / "rust-before" / module).read_text()
        after_text = (out / "rust-source" / module).read_text()
        stripped = without_added_regression(
            before_text,
            "    // Regression: external numbering uses fixed fundamental-coweight bits,",
            "    #[test]\n    fn sl2_orders_compact_zero_and_split_last()")
        parent_module = Path(lock["repair"]["parent_build"]).parent / "rust-source" / module
        if stripped != parent_module.read_text():
            raise ValueError("before phase may add only the regression")
        if before_text.split("#[cfg(test)]", 1)[1] != after_text.split("#[cfg(test)]", 1)[1]:
            raise ValueError("regression tests changed between phases")
        report["repair_source_check"] = {
            "before_archive_sha256": digest(before_archive), "module": module,
            "before_module_sha256": before[module], "after_module_sha256": after[module],
            "unchanged_regressions": True}
        environments = build_environments(out, os.environ)
        report["build_environment"] = {
            phase: {k: env[k] for k in
                    ("CARGO_TARGET_DIR", "CARGO_BUILD_JOBS", "CARGO_PROFILE_TEST_DEBUG")}
            for phase, env in environments.items()}

        def command(name, args, phase="after", expected_exit=0):
            cwd = out / ("rust-before" if phase == "before" else "rust-source")
            start = time.monotonic()
            with (out / (name + ".log")).open("wb") as stream:
                run = subprocess.run(["timeout", "--kill-after=15s", "1400"] + args,
                                     cwd=cwd, env=environments[phase], stdout=stream,
                                     stderr=subprocess.STDOUT)
            entry = {"name": name, "argv": args, "cwd": str(cwd), "exit_status": run.returncode,
                     "seconds": time.monotonic() - start, "log_sha256": digest(out / (name + ".log")),
                     "target_dir": environments[phase]["CARGO_TARGET_DIR"]}
            report.setdefault("expected_failures" if expected_exit else "commands", []).append(entry)
            if run.returncode != expected_exit:
                raise ValueError(name + " unexpected exit: " + str(run.returncode))
            return (out / (name + ".log")).read_text()

        for name, argv in (("rustc-version", ["rustc", "-vV"]),
                           ("cargo-version", ["cargo", "-V"]),
                           ("gcc-version", ["g++", "--version"])):
            command(name, argv)
        unit = ["cargo", "test", "--offline", "--locked", "-p", "atlas-real-group", "--lib",
                "real_form_order::tests::e6_twisted_generator_coordinates_follow_ambient_basis",
                "--", "--exact", "--nocapture"]
        failed = command("regression-before", unit, "before", 101)
        if ("test result: FAILED. 0 passed; 1 failed;" not in failed
                or "twist-fixed generator coordinate" not in failed
                or "E6_FORM_COORDINATE_REGRESSION bit=1" not in failed):
            raise ValueError("before must execute the intended invariant failure")
        passed = command("regression-after", unit)
        if "test result: ok. 1 passed; 0 failed;" not in passed:
            raise ValueError("after must execute the unchanged regression")
        for name, package, test_filter, exact in (
                ("form-order-regressions", "atlas-real-group", "real_form_order::tests::", False),
                ("kl-table-regressions", "atlas-real-group", "kl_table::tests::", False),
                ("locator-regressions", "atlas-real-group", "locator::tests::", False),
                ("f4-boundary-regression", "atlas-core",
                 "domain_builtins::tests::f4_partial_kl_recursion_handles_interval_boundary_links", True),
                ("fpp-reflection-regression", "atlas-core",
                 "domain_builtins::tests::alcove_reflection_words_act_as_exact_root_reflections", True)):
            args = ["cargo", "test", "--offline", "--locked", "-p", package, "--lib", test_filter, "--"]
            if exact:
                args.append("--exact")
            log = command(name, args)
            if "test result: ok." not in log or "test result: ok. 0 passed;" in log:
                raise ValueError(name + " did not execute")
        command("rust-build", ["cargo", "build", "--offline", "--locked", "--release", "-p", "atlas-cli"])
        oracle = out / "oracle-source/atlas"
        shutil.copy2(parent["binaries"]["oracle"]["path"], oracle)
        if digest(oracle) != parent["binaries"]["oracle"]["sha256"]:
            raise ValueError("reference executable changed")
        for engine, binary in (("oracle", oracle), ("rust", out / "target-after/release/atlas-cli")):
            report["binaries"][engine] = {"path": str(binary), "sha256": digest(binary)}
            for name, sha in report["source_files"][engine].items():
                if digest(out / (engine + "-source") / name) != sha:
                    raise ValueError("build changed source files")
        report["scripts"] = file_manifest(out / "oracle-source/atlas-scripts")
        if report["scripts"] != parent["scripts"]:
            raise ValueError("reference scripts changed")
        report["oracle_reuse"] = {"parent_build": lock["repair"]["parent_build"],
                                 "parent_build_sha256": lock["repair"]["parent_build_sha256"]}
        report["status"] = "BUILT_NOT_DIFFERENTIALLY_VERIFIED"
    except Exception:
        report["error"] = traceback.format_exc()
    report["harness_unchanged"] = all(digest(root / n) == h for n, h in manifest.items())
    if not report["harness_unchanged"]:
        report["status"] = "FAIL"
    path = out / "build.json"
    path.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n")
    path.with_suffix(".sha256").write_text(digest(path) + "\n")
    print(report["status"], path, flush=True)
    return int(report["status"] != "BUILT_NOT_DIFFERENTIALLY_VERIFIED")


if __name__ == "__main__":
    raise SystemExit(main())

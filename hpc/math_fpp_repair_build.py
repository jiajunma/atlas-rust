#!/usr/bin/env python3
"""HPC-only before/after regression and source-bound FPP candidate build."""
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import time
import traceback

from math_baseline_build import digest, file_manifest, unpack
from math_suite_review import review_build


def build_environments(out, inherited):
    clean = {k: v for k, v in inherited.items()
             if not k.startswith(("ATLAS_", "RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS"))}
    clean.update(CARGO_BUILD_JOBS="2", CARGO_PROFILE_TEST_DEBUG="0")
    # Archived sources preserve timestamps. Sharing a target directory across
    # equal-path packages can make Cargo execute the old binary after a repair.
    # Independent targets are part of the before/after evidence, not optional.
    return {phase: dict(clean, CARGO_TARGET_DIR=str(out / ("target-" + phase)))
            for phase in ("before", "after")}


def main():
    if not os.environ.get("SLURM_JOB_ID"):
        raise SystemExit("Repair builds and regression tests require HPC")
    root = Path.cwd()
    out = root / "results" / os.environ["SLURM_JOB_ID"]
    out.mkdir(parents=True, exist_ok=False)
    manifest = json.loads((root / "suite-inputs.json").read_text())
    if digest(root / "suite-inputs.json") != os.environ["SUITE_INPUTS_SHA256"]:
        raise ValueError("input manifest changed")
    for name, sha in manifest.items():
        if digest(root / name) != sha:
            raise ValueError("changed repair harness")
    if digest(os.environ["MATH_BUILD_SPOOL"]) != manifest["hpc/math_fpp_repair_build.sbatch"]:
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
            raise ValueError("reference must remain unchanged")
        for engine in ("oracle", "rust"):
            archive = root / lock[engine]["archive"]
            if digest(archive) != lock[engine]["archive_sha256"]:
                raise ValueError("changed pinned source archive")
            unpack(archive, out / (engine + "-source"))
            report["source_files"][engine] = file_manifest(out / (engine + "-source"))
        before_archive = root / "rust-before.tar"
        if digest(before_archive) != lock["repair"]["before_archive_sha256"]:
            raise ValueError("changed pre-repair source")
        unpack(before_archive, out / "rust-before")
        before_files = file_manifest(out / "rust-before")
        core = "crates/atlas-core/src/domain_builtins.rs"
        if {n for n in before_files if before_files[n] != report["source_files"]["rust"].get(n)} != {core}:
            raise ValueError("repair changes more than the declared core module")
        for n, sha in before_files.items():
            if n != core and sha != parent["source_files"]["rust"].get(n):
                raise ValueError("unexpected change from original Rust main")
        before = (out / "rust-before" / core).read_text()
        after = (out / "rust-source" / core).read_text()
        old = "let mirror: Vec<usize> = word[..word.len() - 1].to_vec();"
        new = "let mirror: Vec<usize> = word[..word.len() - 1].iter().rev().copied().collect();"
        expected = before.replace(old, new).replace(
            "/// first descent until simple, then mirror the path.",
            "/// first descent until simple, then retrace it in reverse for conjugation.")
        if before.count(old) != 1 or after != expected or "fn alcove_reflection_words_act_as_exact_root_reflections()" not in before:
            raise ValueError("before/after source is not exactly the declared regression and repair")
        report["repair_source_check"] = {
            "before_archive_sha256": digest(before_archive),
            "before_core_sha256": before_files[core],
            "after_core_sha256": report["source_files"]["rust"][core],
            "only_runtime_change": "reverse conjugating suffix in reflection_word"}
        environments = build_environments(out, os.environ)
        report["build_environment"] = {
            phase: {k: env[k] for k in
                    ("CARGO_TARGET_DIR", "CARGO_BUILD_JOBS", "CARGO_PROFILE_TEST_DEBUG")}
            for phase, env in environments.items()}

        def command(name, args, cwd, expected_exit=0, phase="after"):
            start = time.monotonic()
            with (out / (name + ".log")).open("wb") as stream:
                p = subprocess.run(["timeout", "--kill-after=15s", "1400"] + args,
                                   cwd=cwd, env=environments[phase], stdout=stream,
                                   stderr=subprocess.STDOUT)
            entry = {"name": name, "argv": args, "cwd": str(cwd), "exit_status": p.returncode,
                     "seconds": time.monotonic() - start, "log_sha256": digest(out / (name + ".log")),
                     "target_dir": environments[phase]["CARGO_TARGET_DIR"]}
            if expected_exit == 0:
                report["commands"].append(entry)
            else:
                report["regression_before"] = entry
            if p.returncode != expected_exit:
                raise ValueError(name + " unexpected exit: " + str(p.returncode))
            return (out / (name + ".log")).read_text()

        for name, args in (("rustc-version", ["rustc", "-vV"]),
                           ("cargo-version", ["cargo", "-V"]),
                           ("gcc-version", ["g++", "--version"])):
            command(name, args, out)
        unit = ["cargo", "test", "--offline", "--locked", "-p", "atlas-core", "--lib",
                "domain_builtins::tests::alcove_reflection_words_act_as_exact_root_reflections",
                "--", "--exact", "--nocapture"]
        before_log = command("regression-before", unit, out / "rust-before", 101, phase="before")
        if "test result: FAILED. 0 passed; 1 failed;" not in before_log:
            raise ValueError("before test must fail an executed assertion, not compilation")
        after_log = command("regression-after", unit, out / "rust-source")
        if "test result: ok. 1 passed; 0 failed;" not in after_log:
            raise ValueError("after test did not execute the exact regression")
        command("rust-build", ["cargo", "build", "--offline", "--locked", "--release",
                               "-p", "atlas-cli"], out / "rust-source")
        oracle = out / "oracle-source" / "atlas"
        shutil.copy2(parent["binaries"]["oracle"]["path"], oracle)
        if digest(oracle) != parent["binaries"]["oracle"]["sha256"]:
            raise ValueError("reused reference binary changed")
        for engine, path in (("oracle", oracle), ("rust", out / "target-after/release/atlas-cli")):
            report["binaries"][engine] = {"path": str(path), "sha256": digest(path)}
            for n, sha in report["source_files"][engine].items():
                if digest(out / (engine + "-source") / n) != sha:
                    raise ValueError("build changed source")
        report["scripts"] = file_manifest(out / "oracle-source/atlas-scripts")
        if report["scripts"] != parent["scripts"]:
            raise ValueError("upstream scripts changed")
        report["oracle_reuse"] = {"parent_build": lock["repair"]["parent_build"],
                                 "parent_build_sha256": lock["repair"]["parent_build_sha256"]}
        report["status"] = "BUILT_NOT_DIFFERENTIALLY_VERIFIED"
    except Exception:
        report["error"] = traceback.format_exc()
    report["harness_unchanged"] = all(digest(root / n) == h for n, h in manifest.items())
    if not report["harness_unchanged"]:
        report["status"] = "FAIL"
    p = out / "build.json"
    p.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n")
    p.with_suffix(".sha256").write_text(digest(p) + "\n")
    print(report["status"], p, flush=True)
    return 0 if report["status"] == "BUILT_NOT_DIFFERENTIALLY_VERIFIED" else 1


if __name__ == "__main__":
    raise SystemExit(main())

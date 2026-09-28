#!/usr/bin/env python3
"""Build immutable, explicitly pinned math baselines on a compute node only."""
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess
import tarfile
import time
import traceback


def digest(path):
    value = hashlib.sha256()
    with Path(path).open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            value.update(chunk)
    return value.hexdigest()


def file_manifest(root):
    return {str(p.relative_to(root)): digest(p)
            for p in sorted(root.rglob("*")) if p.is_file()}


def unpack(archive, destination):
    destination.mkdir()
    with tarfile.open(archive) as source:
        for member in source.getmembers():
            if (Path(member.name).is_absolute() or ".." in Path(member.name).parts
                    or not (member.isfile() or member.isdir())):
                raise ValueError("unsupported archive member: " + member.name)
        source.extractall(destination)


def main():
    if not os.environ.get("SLURM_JOB_ID"):
        raise SystemExit("Builds must be submitted with sbatch, not run locally/login.")
    root = Path.cwd()
    result = root / "results" / os.environ["SLURM_JOB_ID"]
    result.mkdir(parents=True, exist_ok=False)
    report = {"schema": "atlas-math-build-v1", "status": "FAIL",
              "job_id": os.environ["SLURM_JOB_ID"], "node": platform.node(),
              "commands": [], "source_files": {}, "binaries": {}}
    inputs = [root / "tests/math/baseline.json", root / "hpc/math_baseline_build.py",
              root / "hpc/math_baseline_build.sbatch"]
    report["harness"] = {str(p.relative_to(root)): digest(p) for p in inputs}
    try:
        assert digest(inputs[0]) == os.environ["BASELINE_SHA256"]
        assert digest(inputs[1]) == os.environ["DRIVER_SHA256"]
        assert digest(inputs[2]) == os.environ["SCRIPT_SHA256"]
        assert digest(Path(os.environ["MATH_BUILD_SPOOL"])) == digest(inputs[2])
        lock = json.loads(inputs[0].read_text())
        report["baseline"] = lock
        env = {k: v for k, v in os.environ.items()
               if not k.startswith(("ATLAS_", "RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS"))}
        env.update(CARGO_TARGET_DIR=str(result / "target"), CARGO_BUILD_JOBS="2")
        def run(name, argv, cwd, timeout):
            entry = {"name": name, "argv": argv, "cwd": str(cwd)}
            report["commands"].append(entry)
            start = time.monotonic()
            with (result / (name + ".log")).open("wb") as log:
                proc = subprocess.run(["timeout", "--kill-after=15s", str(timeout)] + argv,
                                      cwd=cwd, env=env, stdout=log,
                                      stderr=subprocess.STDOUT)
            entry.update(exit_status=proc.returncode, seconds=time.monotonic() - start,
                         log_sha256=digest(result / (name + ".log")))
            if proc.returncode:
                raise RuntimeError(name + " failed: " + str(proc.returncode))
        for engine in ("oracle", "rust"):
            pin = lock[engine]
            archive = root / pin["archive"]
            assert digest(archive) == pin["archive_sha256"]
            source = result / (engine + "-source")
            unpack(archive, source)
            report["source_files"][engine] = file_manifest(source)
        oracle = result / "oracle-source"
        rust = result / "rust-source"
        for name, cmd in [("gcc-version", ["g++", "--version"]),
                          ("bison-version", ["bison", "--version"]),
                          ("rustc-version", ["rustc", "-vV"]),
                          ("cargo-version", ["cargo", "-V"])]:
            run(name, cmd, result, 30)
        run("oracle-build", ["make", "-j2", "atlas", "optimize=true", "readline=false"], oracle, 1400)
        run("rust-build", ["cargo", "build", "--offline", "--locked", "--release", "-p", "atlas-cli"], rust, 1400)
        for engine, binary in [("oracle", oracle / "atlas"),
                               ("rust", result / "target/release/atlas-cli")]:
            report["binaries"][engine] = {"path": str(binary), "sha256": digest(binary)}
            source = result / (engine + "-source")
            for name, expected in report["source_files"][engine].items():
                assert digest(source / name) == expected, (engine, name, "source changed")
        report["scripts"] = file_manifest(oracle / "atlas-scripts")
        report["status"] = "BUILT_NOT_DIFFERENTIALLY_VERIFIED"
    except Exception:
        report["error"] = traceback.format_exc()
    finally:
        report["harness_unchanged"] = all(digest(root / name) == sha
                                          for name, sha in report["harness"].items())
        if not report["harness_unchanged"]:
            report["status"] = "FAIL"
        path = result / "build.json"
        path.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n")
        (result / "build.sha256").write_text(digest(path) + "\n")
    print(report["status"], result, flush=True)
    return int(report["status"] == "FAIL")


if __name__ == "__main__":
    raise SystemExit(main())

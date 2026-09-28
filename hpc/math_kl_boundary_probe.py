#!/usr/bin/env python3
"""HPC-only unchanged-runtime probes for focused mathematical regressions."""
import json
import os
from pathlib import Path
import platform
import subprocess
import time
import traceback

from math_baseline_build import digest, file_manifest, unpack
from math_suite_review import review_build


def main():
    if not os.environ.get("SLURM_JOB_ID"):
        raise SystemExit("Run the topology regression on an HPC compute node")
    root = Path.cwd()
    out = root / "results" / os.environ["SLURM_JOB_ID"]
    out.mkdir(parents=True, exist_ok=False)
    report = {"schema": "atlas-kl-boundary-probe-v1", "status": "FAIL",
              "job": os.environ["SLURM_JOB_ID"], "node": platform.node()}
    try:
        if digest(root / "probe.json") != os.environ["PROBE_SHA256"]:
            raise ValueError("probe pin changed")
        pin = json.loads((root / "probe.json").read_text())
        report["pin"] = pin
        for name, sha in pin["inputs"].items():
            if digest(root / name) != sha:
                raise ValueError("probe input changed: " + name)
        if digest(os.environ["MATH_PROBE_SPOOL"]) != pin["inputs"]["hpc/math_kl_boundary_probe.sbatch"]:
            raise ValueError("submitted script changed")
        baseline = json.loads((root / "tests/math/baseline.json").read_text())
        parent, _ = review_build(Path(pin["parent_build"]), pin["parent_build_sha256"], baseline)
        unpack(root / "rust-probe.tar", out / "source")
        files = file_manifest(out / "source")
        report["source_files"] = files
        kind = pin.get("probe_kind", "kl_boundary")
        if kind == "kl_boundary":
            core = "crates/atlas-core/src/domain_builtins.rs"
            first = "    #[test]\n    fn f4_partial_kl_recursion_handles_interval_boundary_links()"
            following = "    #[test]\n    fn alcove_reflection_words_act_as_exact_root_reflections()"
            test_filter = "domain_builtins::tests::f4_partial_kl_recursion_handles_interval_boundary_links"
            package, count, trace_prefix = "atlas-core", 1, "F4 partial KL boundary:"
        elif kind == "locator_coroots":
            core = "crates/atlas-real-group/src/locator.rs"
            first = "    // Regression: integral-datum closure uses coroot addition."
            following = "    // Conventions for the hand computations below."
            test_filter = "locator::tests::integral_coroot_"
            package, count, trace_prefix = "atlas-real-group", 2, "LOCATOR_COROOT_REGRESSION "
        elif kind == "e6_form_coordinates":
            core = "crates/atlas-real-group/src/real_form_order.rs"
            first = "    // Regression: external numbering uses fixed fundamental-coweight bits,"
            following = "    #[test]\n    fn sl2_orders_compact_zero_and_split_last()"
            test_filter = "real_form_order::tests::e6_twisted_generator_coordinates_follow_ambient_basis"
            package, count, trace_prefix = "atlas-real-group", 1, "E6_FORM_COORDINATE_REGRESSION "
        else:
            raise ValueError("unknown regression probe")
        previous = parent["source_files"]["rust"]
        if (files.keys() != previous.keys()
                or {n for n in files if files[n] != previous[n]} != {core}):
            raise ValueError("only the regression module may differ from the parent")
        parent_core = Path(pin["parent_build"]).parent / "rust-source" / core
        current = (out / "source" / core).read_text()
        start = current.index(first)
        end = current.index(following, start)
        if current[:start] + current[end:] != parent_core.read_text():
            raise ValueError("the diagnostic must add only the regression, not change runtime")
        env = {k: v for k, v in os.environ.items()
               if not k.startswith(("ATLAS_", "RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS"))}
        env.update(CARGO_TARGET_DIR=str(out / "target"), CARGO_BUILD_JOBS="2",
                   CARGO_PROFILE_TEST_DEBUG="0")
        command = ["cargo", "test", "--offline", "--locked", "-p", package, "--lib",
                   test_filter, "--", "--nocapture"]
        if count == 1:
            command.append("--exact")
        start = time.monotonic()
        with (out / "unit.log").open("wb") as stream:
            run = subprocess.run(["timeout", "--kill-after=15s", "1500"] + command,
                                 cwd=out / "source", env=env, stdout=stream, stderr=subprocess.STDOUT)
        report["command"] = {"argv": command, "exit_status": run.returncode,
                             "seconds": time.monotonic() - start,
                             "log_sha256": digest(out / "unit.log"),
                             "target_dir": env["CARGO_TARGET_DIR"]}
        log = (out / "unit.log").read_text()
        report["boundary_links"] = [s for s in log.splitlines() if s.startswith(trace_prefix)]
        if (run.returncode != 101 or f"test result: FAILED. 0 passed; {count} failed;" not in log
                or len(report["boundary_links"]) < count):
            raise ValueError("must reproduce executed regression failures, not compilation errors")
        expected_error = {"kl_boundary": "cross of extremal",
                          "locator_coroots": "integral image positivity",
                          "e6_form_coordinates": "twist-fixed generator coordinate"}[kind]
        if expected_error not in log:
            raise ValueError("expected mathematical invariant failure is missing")
        if any(digest(out / "source" / n) != h for n, h in files.items()):
            raise ValueError("source changed during probe")
        if any(digest(root / n) != h for n, h in pin["inputs"].items()):
            raise ValueError("probe inputs changed during execution")
        report["status"] = "EXPECTED_PANIC_REPRODUCED_NOT_REPAIRED"
    except Exception:
        report["error"] = traceback.format_exc()
    path = out / "report.json"
    path.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n")
    path.with_suffix(".sha256").write_text(digest(path) + "\n")
    print(report["status"], path, flush=True)
    return int(report["status"] == "FAIL")


if __name__ == "__main__":
    raise SystemExit(main())

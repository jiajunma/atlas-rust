#!/usr/bin/env python3
"""Capture authoritative Atlas output for the typed scalar fixture set.

This intentionally invokes only the upstream Atlas executable.  It writes the
unmodified stdout and stderr for every input and produces a compact JSON
manifest with exit statuses, checksums, reference revision, and checks against
the checked-in semantic expectation.  Rust comparison belongs to the later
differential stage.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import pathlib
import subprocess
import sys
import time
from typing import Any


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def command_output(command: list[str], cwd: pathlib.Path | None = None) -> str:
    try:
        return subprocess.check_output(command, cwd=cwd, text=True).strip()
    except (OSError, subprocess.CalledProcessError):
        return "unavailable"


def render_value(value: dict[str, Any]) -> str:
    value_type = value["type"]
    if value_type == "integer":
        return value["value"]
    if value_type == "rational":
        return f"{value['numerator']}/{value['denominator']}"
    if value_type == "boolean":
        return "true" if value["value"] else "false"
    if value_type == "string":
        return f'"{value["value"]}"'
    if value_type == "tuple":
        return "(" + ",".join(render_value(item) for item in value["values"]) + ")"
    raise ValueError(f"unsupported scalar expectation type: {value_type}")


def load_expectation(fixture: pathlib.Path, reference_root: pathlib.Path) -> dict[str, Any]:
    path = reference_root / f"{fixture.stem}.events.json"
    if not path.is_file():
        return {
            "path": str(path),
            "load_error": "missing expectation file",
            "value_lines": [],
            "diagnostics": [],
        }
    document = json.loads(path.read_text(encoding="utf-8"))
    value_lines = [
        f"Value: {render_value(event['value'])}"
        for event in document.get("events", [])
        if event.get("kind") == "Value"
    ]
    diagnostics = [
        event["message"]
        for event in document.get("events", [])
        if event.get("kind") == "Diagnostic"
    ]
    return {
        "path": str(path),
        "sha256": sha256(path.read_bytes()),
        "value_lines": value_lines,
        "diagnostics": diagnostics,
    }


def validate_expectation(
    expectation: dict[str, Any], stdout: bytes, stderr: bytes, timed_out: bool
) -> tuple[dict[str, bool], bool]:
    """Validate declared event content without inferring an exit-code policy."""
    stdout_lines = stdout.decode("utf-8", errors="replace").splitlines()
    combined_text = (stdout + b"\n" + stderr).decode("utf-8", errors="replace")
    actual_values = [line for line in stdout_lines if line.startswith("Value: ")]
    actual_diagnostics = [
        line.strip()
        for line in stderr.decode("utf-8", errors="replace").splitlines()
        if line.startswith("  ") and line.strip()
    ]
    checks = {
        "expectation_loaded": "load_error" not in expectation,
        "expected_value_lines_exact": actual_values == expectation["value_lines"],
        "expected_diagnostics_exact": actual_diagnostics == expectation["diagnostics"],
        "no_unexpected_runtime_error": "Runtime error:" not in combined_text
        if not expectation["diagnostics"]
        else True,
    }
    return checks, not timed_out and all(checks.values())


def fixture_entry(
    atlas_bin: pathlib.Path,
    fixture: pathlib.Path,
    output_dir: pathlib.Path,
    reference_root: pathlib.Path,
    timeout: int,
) -> tuple[dict[str, Any], bool]:
    source = fixture.read_bytes()
    expectation = load_expectation(fixture, reference_root)
    input_bytes = source + (b"" if source.endswith(b"\n") else b"\n") + b"quit\n"
    started = time.monotonic()
    timed_out = False
    try:
        completed = subprocess.run(
            [str(atlas_bin)],
            input=input_bytes,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            cwd=atlas_bin.parent / "atlas-scripts",
            timeout=timeout,
        )
        stdout, stderr, exit_status = completed.stdout, completed.stderr, completed.returncode
    except subprocess.TimeoutExpired as error:
        timed_out = True
        stdout = error.stdout or b""
        stderr = error.stderr or b""
        exit_status = None
    elapsed = round(time.monotonic() - started, 3)

    artifact_base = output_dir / fixture.stem
    stdout_path = artifact_base.with_suffix(".stdout")
    stderr_path = artifact_base.with_suffix(".stderr")
    stdout_path.write_bytes(stdout)
    stderr_path.write_bytes(stderr)

    checks, passed = validate_expectation(expectation, stdout, stderr, timed_out)

    entry = {
        "fixture": str(fixture),
        "fixture_sha256": sha256(source),
        "expectation": expectation,
        "input_appended": "quit\\n",
        "stdout": {"path": stdout_path.name, "sha256": sha256(stdout)},
        "stderr": {"path": stderr_path.name, "sha256": sha256(stderr)},
        "exit_status": exit_status,
        "timed_out": timed_out,
        "seconds": elapsed,
        "checks": checks,
        "status": "PASS" if passed else "FAIL",
    }
    return entry, passed


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("atlas_bin", type=pathlib.Path)
    parser.add_argument("output_dir", type=pathlib.Path)
    parser.add_argument("fixtures", type=pathlib.Path, nargs="+")
    parser.add_argument("--reference-root", type=pathlib.Path, required=True)
    parser.add_argument("--commit", required=True)
    parser.add_argument("--dirty-tree", required=True)
    parser.add_argument("--job-id", required=True)
    parser.add_argument("--reference-revision", default="not-provided")
    parser.add_argument("--cweb-version", default="not-provided")
    parser.add_argument("--source-snapshot-sha256", required=True)
    parser.add_argument("--timeout", type=int, default=30)
    args = parser.parse_args()

    atlas_bin = args.atlas_bin.resolve()
    if not os.access(atlas_bin, os.X_OK):
        parser.error(f"Atlas executable is not executable: {atlas_bin}")
    atlas_scripts = atlas_bin.parent / "atlas-scripts"
    if not atlas_scripts.is_dir():
        parser.error(f"missing Atlas script directory: {atlas_scripts}")

    args.output_dir.mkdir(parents=True, exist_ok=True)
    entries = []
    all_passed = True
    for fixture in args.fixtures:
        entry, passed = fixture_entry(
            atlas_bin,
            fixture.resolve(),
            args.output_dir,
            args.reference_root.resolve(),
            args.timeout,
        )
        entries.append(entry)
        all_passed = all_passed and passed

    detected_revision = command_output(
        ["git", "rev-parse", "HEAD"], cwd=atlas_bin.parent
    )
    report = {
        "schema": "atlas-scalar-reference-v1",
        "stage": "typed-scalar-reference",
        "commit": args.commit,
        "dirty_tree": args.dirty_tree,
        "source_snapshot_sha256": args.source_snapshot_sha256,
        "harness_sha256": sha256(pathlib.Path(__file__).read_bytes()),
        "reference_atlas_binary": str(atlas_bin),
        "reference_atlas_binary_sha256": sha256(atlas_bin.read_bytes()),
        "reference_atlas_revision": (
            args.reference_revision
            if args.reference_revision != "not-provided"
            else detected_revision
        ),
        "cweb_version": args.cweb_version,
        "rustc": command_output(["rustc", "--version"]),
        "cargo": command_output(["cargo", "--version"]),
        "slurm": {
            "job_id": args.job_id,
            "node_list": os.environ.get("SLURM_JOB_NODELIST", "unavailable"),
            "hostname": command_output(["hostname"]),
        },
        "fixtures": entries,
        "status": "PASS" if all_passed else "FAIL",
    }
    report_path = args.output_dir / "scalar_reference_report.json"
    report_path.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"scalar reference: {len(entries)} fixtures, {report['status']}")
    print(f"report: {report_path}")
    return 0 if all_passed else 1


if __name__ == "__main__":
    raise SystemExit(main())

#!/usr/bin/env python3
"""Capture raw output from a pinned upstream Atlas executable.

This is an evidence collector, not a differential test.  It does not consume
or update checked-in expectations and it treats a nonzero interpreter exit as
an observation rather than a capture failure.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import pathlib
import platform
import re
import resource
import shutil
import subprocess
import sys
import tempfile
import time
from typing import Any


GIT_REVISION = re.compile(r"^[0-9a-fA-F]{40}$")
SHA256_DIGEST = re.compile(r"^[0-9a-fA-F]{64}$")
COMMIT_TOKEN = re.compile(r"^(?:[0-9a-fA-F]{40}|unversioned)$")
DIRTY_TREE_TOKENS = {"true", "false", "unknown"}


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def tree_sha256(root: pathlib.Path) -> str:
    hasher = hashlib.sha256()
    for path in sorted(
        (item for item in root.rglob("*") if item.is_file()),
        key=lambda item: item.relative_to(root).as_posix(),
    ):
        relative = path.relative_to(root).as_posix().encode()
        hasher.update(relative + b"\0" + hashlib.sha256(path.read_bytes()).digest())
    return hasher.hexdigest()


def command_output(command: list[str], cwd: pathlib.Path | None = None) -> str:
    try:
        return subprocess.check_output(
            command, cwd=cwd, text=True, stderr=subprocess.DEVNULL
        ).strip()
    except (OSError, subprocess.CalledProcessError):
        return "unavailable"


def artifact_base(
    fixture: pathlib.Path,
    workspace_root: pathlib.Path,
    output_dir: pathlib.Path,
) -> pathlib.Path:
    try:
        relative = fixture.relative_to(workspace_root)
    except ValueError:
        digest = sha256(str(fixture).encode())[:16]
        relative = pathlib.Path("external") / f"{fixture.stem}-{digest}"
    return output_dir / relative


def capture_fixture(
    atlas_bin: pathlib.Path,
    fixture: pathlib.Path,
    workspace_root: pathlib.Path,
    output_dir: pathlib.Path,
    timeout: int,
) -> tuple[dict[str, Any], bool]:
    source = fixture.read_bytes()
    input_bytes = source + (b"" if source.endswith(b"\n") else b"\n") + b"quit\n"
    started = time.monotonic()
    timed_out = False
    maxrss_kb = None
    maxrss_approximate = False
    if _USE_GNU_TIME:
        with tempfile.TemporaryDirectory() as directory:
            metric_path = pathlib.Path(directory) / "time.metrics"
            try:
                completed = subprocess.run(
                    [_TIME_BIN, "-v", "-o", str(metric_path), str(atlas_bin)],
                    input=input_bytes,
                    stdout=subprocess.PIPE,
                    stderr=subprocess.PIPE,
                    cwd=atlas_bin.parent / "atlas-scripts",
                    timeout=timeout,
                )
                stdout = completed.stdout
                stderr = completed.stderr
                exit_status = completed.returncode
            except subprocess.TimeoutExpired as error:
                timed_out = True
                stdout = error.stdout or b""
                stderr = error.stderr or b""
                exit_status = None
            maxrss_kb = _parse_time_metrics(metric_path)
    else:
        try:
            completed = subprocess.run(
                [str(atlas_bin)],
                input=input_bytes,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                cwd=atlas_bin.parent / "atlas-scripts",
                timeout=timeout,
            )
            stdout = completed.stdout
            stderr = completed.stderr
            exit_status = completed.returncode
        except subprocess.TimeoutExpired as error:
            timed_out = True
            stdout = error.stdout or b""
            stderr = error.stderr or b""
            exit_status = None
        maxrss_kb, maxrss_approximate = _measured_maxrss()

    base = artifact_base(fixture, workspace_root, output_dir)
    base.parent.mkdir(parents=True, exist_ok=True)
    stdout_path = pathlib.Path(f"{base}.stdout")
    stderr_path = pathlib.Path(f"{base}.stderr")
    stdout_path.write_bytes(stdout)
    stderr_path.write_bytes(stderr)
    try:
        fixture_name = fixture.relative_to(workspace_root).as_posix()
    except ValueError:
        fixture_name = str(fixture)
    entry = {
        "fixture": fixture_name,
        "fixture_sha256": sha256(source),
        "input_appended": "quit\\n",
        "input_sha256": sha256(input_bytes),
        "stdout": {
            "path": stdout_path.relative_to(output_dir).as_posix(),
            "sha256": sha256(stdout),
            "bytes": len(stdout),
            "text": stdout.decode("utf-8", errors="replace"),
        },
        "stderr": {
            "path": stderr_path.relative_to(output_dir).as_posix(),
            "sha256": sha256(stderr),
            "bytes": len(stderr),
            "text": stderr.decode("utf-8", errors="replace"),
        },
        "oracle_exit_status": exit_status,
        "timed_out": timed_out,
        "seconds": round(time.monotonic() - started, 3),
        "maxrss_kb": maxrss_kb,
        "maxrss_approximate": maxrss_approximate,
        "capture_status": "FAIL" if timed_out else "CAPTURED",
    }
    return entry, not timed_out


def _parse_time_metrics(path: pathlib.Path) -> int | None:
    if not path.exists():
        return None
    for line in path.read_text(encoding="utf-8", errors="replace").splitlines():
        if "maximum resident set size" in line.lower():
            digits = re.sub(r"\D", "", line.split(":", 1)[-1])
            if digits:
                return int(digits)
    return None


# /usr/bin/time -v (GNU coreutils) exists on the Linux HPC nodes; the mac
# boxes only have the BSD variant, so fall back to the cumulative
# child-process peak from getrusage (labelled approximate).
_TIME_BIN = shutil.which("/usr/bin/time") or "/usr/bin/time"
_USE_GNU_TIME = os.path.exists(_TIME_BIN) and platform.system() != "Darwin"


def _measured_maxrss() -> tuple[int | None, bool]:
    if _USE_GNU_TIME:
        # The oracle itself was run before this call with GNU time; the
        # metric file lives in a temp dir owned by the caller.
        return None, False
    try:
        rss = resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss
        if sys.platform == "darwin":
            rss //= 1024
        return int(rss), True
    except (AttributeError, ValueError):
        return None, True


def parse_dirty_tree(value: str) -> bool | str:
    return {"true": True, "false": False}.get(value.lower(), value)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("atlas_bin", type=pathlib.Path)
    parser.add_argument("output_dir", type=pathlib.Path)
    parser.add_argument("fixtures", type=pathlib.Path, nargs="+")
    parser.add_argument("--workspace-root", type=pathlib.Path, required=True)
    parser.add_argument("--commit", required=True)
    parser.add_argument("--dirty-tree", required=True)
    parser.add_argument("--detected-commit", required=True)
    parser.add_argument("--detected-dirty-tree", required=True)
    parser.add_argument("--job-id", required=True)
    parser.add_argument("--reference-revision", required=True)
    parser.add_argument("--expected-binary-sha256")
    parser.add_argument("--cweb-version", default="not-provided")
    parser.add_argument("--source-snapshot-sha256", required=True)
    parser.add_argument("--timeout", type=int, default=30)
    args = parser.parse_args()

    atlas_bin = args.atlas_bin.resolve()
    if not os.access(atlas_bin, os.X_OK):
        parser.error(f"Atlas executable is not executable: {atlas_bin}")
    if not (atlas_bin.parent / "atlas-scripts").is_dir():
        parser.error(f"missing Atlas script directory beside {atlas_bin}")
    workspace_root = args.workspace_root.resolve()
    fixtures = [path.resolve() for path in args.fixtures]
    for fixture in fixtures:
        if not fixture.is_file():
            parser.error(f"fixture does not exist: {fixture}")
        try:
            fixture.relative_to(workspace_root)
        except ValueError:
            parser.error(f"fixture is outside the frozen workspace: {fixture}")
    output_dir = args.output_dir.resolve()
    output_dir.mkdir(parents=True, exist_ok=True)

    binary = atlas_bin.read_bytes()
    binary_sha = sha256(binary)
    scripts_dir = atlas_bin.parent / "atlas-scripts"
    scripts_sha = tree_sha256(scripts_dir)
    detected_revision = command_output(["git", "rev-parse", "HEAD"], atlas_bin.parent)
    upstream_tree_state = command_output(
        ["git", "status", "--porcelain", "--untracked-files=all"], atlas_bin.parent
    )
    requested_revision_valid = bool(GIT_REVISION.fullmatch(args.reference_revision))
    detected_revision_valid = bool(GIT_REVISION.fullmatch(detected_revision))
    revision_exact = (
        requested_revision_valid
        and detected_revision_valid
        and detected_revision == args.reference_revision
    )
    binary_pin_valid = bool(
        args.expected_binary_sha256
        and SHA256_DIGEST.fullmatch(args.expected_binary_sha256)
    )
    binary_exact = (
        binary_pin_valid
        and binary_sha == args.expected_binary_sha256
    )
    source_tree_clean = upstream_tree_state == ""
    source_state_checks = {
        "declared_commit_valid": bool(COMMIT_TOKEN.fullmatch(args.commit)),
        "detected_commit_valid": bool(COMMIT_TOKEN.fullmatch(args.detected_commit)),
        "commit_exact": args.commit == args.detected_commit,
        "declared_dirty_tree_valid": args.dirty_tree in DIRTY_TREE_TOKENS,
        "detected_dirty_tree_valid": (
            args.detected_dirty_tree in DIRTY_TREE_TOKENS
        ),
        "dirty_tree_exact": args.dirty_tree == args.detected_dirty_tree,
    }
    source_state_verified = all(source_state_checks.values())

    entries = []
    captures_completed = True
    for fixture in fixtures:
        entry, completed = capture_fixture(
            atlas_bin,
            fixture,
            workspace_root,
            output_dir,
            args.timeout,
        )
        entries.append(entry)
        captures_completed = captures_completed and completed

    post_binary_sha = sha256(atlas_bin.read_bytes())
    post_detected_revision = command_output(
        ["git", "rev-parse", "HEAD"], atlas_bin.parent
    )
    post_tree_state = command_output(
        ["git", "status", "--porcelain", "--untracked-files=all"], atlas_bin.parent
    )
    post_scripts_sha = tree_sha256(scripts_dir)
    runtime_unchanged = (
        post_binary_sha == binary_sha
        and post_detected_revision == detected_revision
        and post_tree_state == upstream_tree_state
        and post_scripts_sha == scripts_sha
    )

    verified = (
        captures_completed
        and revision_exact
        and binary_exact
        and source_tree_clean
        and source_state_verified
        and runtime_unchanged
    )
    report = {
        "schema": "atlas-reference-raw-capture-v1",
        "stage": "upstream-atlas-raw-reference-capture",
        "status": "PASS" if verified else "FAIL",
        "compatibility_claim": False,
        "expectations_consumed": False,
        "expectations_modified": False,
        "commit": args.commit,
        "dirty_tree": parse_dirty_tree(args.dirty_tree),
        "source_state": {
            "declared_commit": args.commit,
            "detected_commit": args.detected_commit,
            "declared_dirty_tree": parse_dirty_tree(args.dirty_tree),
            "detected_dirty_tree": parse_dirty_tree(args.detected_dirty_tree),
            "verified": source_state_verified,
            "checks": source_state_checks,
        },
        "source_snapshot_sha256": args.source_snapshot_sha256,
        "source_snapshot_scope": (
            "provided snapshot (exact scope annotated by the batch job)"
        ),
        "harness_sha256": sha256(pathlib.Path(__file__).read_bytes()),
        "reference_atlas_binary": str(atlas_bin),
        "reference_working_directory": str(atlas_bin.parent / "atlas-scripts"),
        "reference_atlas_binary_sha256": binary_sha,
        "post_reference_atlas_binary_sha256": post_binary_sha,
        "reference_atlas_scripts_sha256": scripts_sha,
        "post_reference_atlas_scripts_sha256": post_scripts_sha,
        "expected_atlas_binary_sha256": args.expected_binary_sha256,
        "binary_checksum_exact": binary_exact,
        "reference_atlas_revision": args.reference_revision,
        "detected_atlas_revision": detected_revision,
        "post_detected_atlas_revision": post_detected_revision,
        "reference_revision_exact": revision_exact,
        "reference_source_tree_clean": source_tree_clean,
        "reference_source_tree_status": upstream_tree_state,
        "post_reference_source_tree_status": post_tree_state,
        "reference_runtime_unchanged": runtime_unchanged,
        "provenance_checks": {
            "captures_completed": captures_completed,
            "requested_revision_valid": requested_revision_valid,
            "detected_revision_valid": detected_revision_valid,
            "revision_exact": revision_exact,
            "binary_pin_valid": binary_pin_valid,
            "binary_checksum_exact": binary_exact,
            "source_tree_clean": source_tree_clean,
            "runtime_unchanged": runtime_unchanged,
        },
        "cweb_version": args.cweb_version,
        "slurm": {
            "job_id": args.job_id,
            "node_list": os.environ.get("SLURM_JOB_NODELIST", "unavailable"),
            "hostname": command_output(["hostname"]),
        },
        "fixtures": entries,
    }
    report_path = output_dir / "reference_capture_report.json"
    report_path.write_text(
        json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    print(f"reference capture: {len(entries)} fixtures, {report['status']}")
    print(f"report: {report_path}")
    return 0 if verified else 1


if __name__ == "__main__":
    raise SystemExit(main())

"""Compute-node durability gate for the bounded root-ladder index entry."""
import hashlib
import json
import os
from pathlib import Path
import re
import signal
import subprocess
import sys
import time
import traceback


# Install the legacy-top-level guard before importing project helpers.  The
# job may access only its child of the one active campaign.
_ACTIVE_CAMPAIGN = "atlas-rust-campaign-20260930"
_HPC_HOME = Path("/public/home/majj")
LEGACY_PATH_OPEN_ATTEMPTS = []


def forbidden_legacy_path(value):
    if not isinstance(value, (str, bytes, os.PathLike)):
        return False
    try:
        path = Path(os.path.abspath(os.fsdecode(value)))
        relative = path.relative_to(_HPC_HOME)
    except (TypeError, ValueError, UnicodeError):
        return False
    return bool(relative.parts) and relative.parts[0].startswith("atlas-") \
        and relative.parts[0] != _ACTIVE_CAMPAIGN


def _path_audit(event, arguments):
    if event == "open" and arguments and forbidden_legacy_path(arguments[0]):
        LEGACY_PATH_OPEN_ATTEMPTS.append(True)
        raise RuntimeError("legacy top-level Atlas path access rejected")


sys.addaudithook(_path_audit)

from campaign_blob import verify_blob
from campaign_workspace import (ACTIVE_CAMPAIGN, campaign_stage,
                                create_result_folder, ephemeral_job_workspace)
from progressive_submit import read_json_file, save
from stage_ladder_boundary_index import (EXPECTED_TEST_COUNTS, PIN_NAME,
                                         RETIRED_STAGER_BUNDLE_REFERENCE,
                                         STAGE_LOCK, STAGE_NAME,
                                         confirmed_record,
                                         existing_lock,
                                         frozen_stage_inputs,
                                         submission_receipt, validate_pin,
                                         validate_test_counts)


if ACTIVE_CAMPAIGN != _ACTIVE_CAMPAIGN:
    raise RuntimeError("ladder boundary index campaign policy changed")

SUBMISSION_ENABLED = False
COMMAND_TIMEOUT_SECONDS = 300
COMMAND_KILL_AFTER_SECONDS = 15
SBATCH = "hpc/math_ladder_boundary_index.sbatch"
CHECK_COMMANDS = [
    (
        "test-stage-ladder-boundary-index",
        [
            sys.executable, "-I", "-S", "-B", "-m", "unittest", "discover",
            "-s", "hpc", "-p", "test_stage_ladder_boundary_index.py", "-v",
        ],
    ),
    (
        "test-math-acceptance-index",
        [
            sys.executable, "-I", "-S", "-B", "-m", "unittest", "discover",
            "-s", "hpc", "-p", "test_math_acceptance_index.py", "-v",
        ],
    ),
    (
        "test-stager-allowlist",
        [
            sys.executable, "-I", "-S", "-B", "-m", "unittest", "discover",
            "-s", "hpc", "-p", "test_stager_allowlist.py", "-v",
        ],
    ),
]
SHA256_PATTERN = r"[0-9a-f]{64}\Z"
UNITTEST_SUMMARY_PATTERN = (
    r"^Ran ([0-9]+) tests? in [^\r\n]+\r?\n\r?\nOK[ \t]*\r?\n?\Z"
)


def digest(path):
    value = hashlib.sha256()
    with Path(path).open("rb") as handle:
        for block in iter(lambda: handle.read(1024 * 1024), b""):
            value.update(block)
    return value.hexdigest()


def require_enabled_driver(pin_sha256):
    if SUBMISSION_ENABLED is not True:
        raise ValueError("ladder boundary index compute driver remains disabled")
    if (not isinstance(pin_sha256, str)
            or re.fullmatch(SHA256_PATTERN, pin_sha256) is None):
        raise ValueError("ladder boundary index compute pin environment is invalid")
    return pin_sha256, validate_test_counts(EXPECTED_TEST_COUNTS)


def parse_maxrss(path):
    prefix = "Maximum resident set size (kbytes):"
    for line in Path(path).read_text(encoding="utf-8", errors="replace").splitlines():
        if line.strip().startswith(prefix):
            value = line.split(":", 1)[1].strip()
            if value.isdecimal():
                return int(value)
    raise ValueError("GNU time output lacks maximum RSS")


def parse_unittest_count(path):
    raw = Path(path).read_text(encoding="utf-8", errors="replace")
    matches = re.findall(UNITTEST_SUMMARY_PATTERN, raw, re.MULTILINE)
    return int(matches[0]) if len(matches) == 1 else None


def run_command(name, argv, expected_tests, cwd, out, environment):
    log = Path(out) / (name + ".log")
    timing = Path(out) / (name + ".time")
    command = ["/usr/bin/time", "-v", "-o", str(timing), "--", *argv]
    started = time.monotonic()
    status = None
    timed_out = False
    with log.open("wb") as output:
        process = subprocess.Popen(
            command,
            cwd=cwd,
            env=environment,
            stdin=subprocess.DEVNULL,
            stdout=output,
            stderr=subprocess.STDOUT,
            start_new_session=True,
        )
        try:
            status = process.wait(timeout=COMMAND_TIMEOUT_SECONDS)
        except subprocess.TimeoutExpired:
            timed_out = True
            os.killpg(process.pid, signal.SIGTERM)
            try:
                status = process.wait(timeout=COMMAND_KILL_AFTER_SECONDS)
            except subprocess.TimeoutExpired:
                os.killpg(process.pid, signal.SIGKILL)
                status = process.wait()
    seconds = time.monotonic() - started
    observed_tests = parse_unittest_count(log)
    result = {
        "name": name,
        "argv": argv,
        "exit_status": status,
        "timed_out": timed_out,
        "seconds": seconds,
        "log_sha256": digest(log),
        "time_sha256": digest(timing),
        "maxrss_kb": parse_maxrss(timing),
        "maxrss_approximate": False,
        "expected_tests": expected_tests,
        "observed_tests": observed_tests,
        "test_count_matches": observed_tests == expected_tests,
    }
    return result


def atomic_text(path, value):
    path = Path(path)
    temporary = path.with_name("." + path.name + ".tmp")
    flags = (os.O_WRONLY | os.O_CREAT | os.O_EXCL
             | getattr(os, "O_CLOEXEC", 0) | getattr(os, "O_NOFOLLOW", 0))
    descriptor = os.open(temporary, flags, 0o600)
    try:
        raw = value.encode("utf-8")
        with os.fdopen(descriptor, "wb") as handle:
            descriptor = None
            handle.write(raw)
            handle.flush()
            os.fsync(handle.fileno())
        os.replace(temporary, path)
        directory = os.open(
            path.parent,
            os.O_RDONLY | getattr(os, "O_DIRECTORY", 0)
            | getattr(os, "O_CLOEXEC", 0),
        )
        try:
            os.fsync(directory)
        finally:
            os.close(directory)
    finally:
        if descriptor is not None:
            os.close(descriptor)
        temporary.unlink(missing_ok=True)


def capture_current_exception(out, label):
    """Append one bounded traceback and return the resulting durable hash."""
    path = Path(out) / "driver-exception.log"
    flags = (os.O_WRONLY | os.O_CREAT | os.O_APPEND
             | getattr(os, "O_CLOEXEC", 0) | getattr(os, "O_NOFOLLOW", 0))
    descriptor = os.open(path, flags, 0o600)
    with os.fdopen(descriptor, "ab") as handle:
        handle.write(("[" + label + "]\n").encode("utf-8"))
        handle.write(traceback.format_exc().encode("utf-8", errors="replace"))
        handle.flush()
        os.fsync(handle.fileno())
    return digest(path)


def publish_final_report(out, root, initial_inputs, pinned_inputs, commands,
                         failure, workspace_absent, exception_log_sha256,
                         report):
    """Recheck inputs and publish a report even when that recheck fails."""
    final_inputs = None
    final_input_recheck_error = False
    try:
        final_inputs = frozen_stage_inputs(root)
    except Exception:
        final_input_recheck_error = True
        failure = failure or "final-input-recheck"
        exception_log_sha256 = capture_current_exception(
            out, "final-input-recheck")
    source_integrity = final_inputs == initial_inputs == pinned_inputs
    if not final_input_recheck_error and not source_integrity:
        failure = failure or "final-input-drift"
    success = (
        failure is None
        and len(commands) == len(CHECK_COMMANDS)
        and all(row["exit_status"] == 0 and not row["timed_out"]
                and row["test_count_matches"]
                for row in commands)
        and source_integrity
        and workspace_absent
        and report.get("retired_stager_bundle_rechecked") is True
        and not LEGACY_PATH_OPEN_ATTEMPTS
    )
    report.update({
        "status": (
            "LADDER_BOUNDARY_INDEX_GATES_PASS" if success
            else "LADDER_BOUNDARY_INDEX_HARNESS_FAILURE"
        ),
        "commands": commands,
        "observed_test_counts": {
            row["name"]: row["observed_tests"] for row in commands
        },
        "not_reached": [name for name, _ in CHECK_COMMANDS[len(commands):]],
        "failure": failure,
        "exception_log_sha256": exception_log_sha256,
        "final_input_recheck_error": final_input_recheck_error,
        "source_integrity_rechecked": source_integrity,
        "ephemeral_workspace_absent": workspace_absent,
        "legacy_path_open_attempts": len(LEGACY_PATH_OPEN_ATTEMPTS),
    })
    report_path = Path(out) / "report.json"
    save(report_path, report)
    report_sha256 = digest(report_path)
    atomic_text(Path(out) / "report.sha256", report_sha256 + "\n")
    return success, report, report_sha256


def job_identity():
    job = os.environ.get("SLURM_JOB_ID")
    node = os.environ.get("SLURMD_NODENAME") or os.environ.get("HOSTNAME")
    submit = os.environ.get("SLURM_SUBMIT_DIR")
    if (not isinstance(job, str) or not job.isdecimal()
            or not isinstance(node, str) or not node
            or not isinstance(submit, str) or not submit):
        raise ValueError("compute driver requires an exact SLURM allocation")
    return job, node, Path(submit).resolve()


def main():
    if not SUBMISSION_ENABLED:
        raise SystemExit("ladder boundary index compute driver is not frozen or enabled")
    environment_pin = os.environ.get("LADDER_BOUNDARY_INDEX_PIN_SHA256")
    expected_pin_sha256, expected_test_counts = require_enabled_driver(
        environment_pin)
    job, node, root = job_identity()
    campaign = campaign_stage(root)
    if root.name != STAGE_NAME:
        raise ValueError("unexpected ladder boundary index compute stage")
    verify_blob(campaign, RETIRED_STAGER_BUNDLE_REFERENCE)
    spool = os.environ.get("LADDER_BOUNDARY_INDEX_SPOOL")
    if not isinstance(spool, str) or not spool:
        raise ValueError("compute driver lacks the frozen spool or pin environment")

    with existing_lock(root, STAGE_LOCK):
        pin = read_json_file(root / PIN_NAME, expected_pin_sha256)
        validate_pin(pin)
        if pin["test_counts"] != expected_test_counts:
            raise ValueError("compute test counts differ from the reviewed driver")
        if digest(spool) != pin["inputs"][SBATCH]:
            raise ValueError("submitted ladder boundary index spool changed")
        record = confirmed_record(
            root, pin, repair_intent=True, expected_job=job)
        expected_receipt = submission_receipt(record, pin, root=root)
        receipt_path = root / "submission.json"
        if os.path.lexists(receipt_path):
            receipt = read_json_file(receipt_path)
            if receipt != expected_receipt:
                raise ValueError("durable index-stage receipt changed")
        else:
            receipt = expected_receipt
            save(receipt_path, receipt)
    initial_inputs = frozen_stage_inputs(root)
    if initial_inputs != pin["inputs"]:
        raise ValueError("compute inputs differ from the immutable pin")

    out = create_result_folder(root, job)
    commands = []
    failure = None
    workspace_path = None
    exception_log_sha256 = None
    retired_stager_bundle_rechecked = False
    try:
        with ephemeral_job_workspace(out, "index-checks") as workspace:
            workspace_path = Path(workspace)
            environment = dict(
                os.environ,
                PYTHONDONTWRITEBYTECODE="1",
                TMPDIR=str(workspace_path),
                TEMP=str(workspace_path),
                TMP=str(workspace_path),
            )
            for name, argv in CHECK_COMMANDS:
                result = run_command(
                    name, argv, pin["test_counts"][name], root, out, environment)
                commands.append(result)
                if (result["timed_out"] or result["exit_status"] != 0
                        or not result["test_count_matches"]):
                    failure = name
                    break
        verify_blob(campaign, RETIRED_STAGER_BUNDLE_REFERENCE)
        retired_stager_bundle_rechecked = True
    except Exception:
        failure = failure or "driver-exception"
        exception_log_sha256 = capture_current_exception(out, failure)

    workspace_absent = workspace_path is not None and not workspace_path.exists()
    report = {
        "schema": "atlas-ladder-boundary-index-v1",
        "job": job,
        "node": node,
        "stage": str(root),
        "campaign": str(campaign),
        "pin_sha256": expected_pin_sha256,
        "pin": pin,
        "submission_receipt": receipt,
        "expected_test_counts": pin["test_counts"],
        "cargo_commands": 0,
        "atlas_commands": 0,
        "index": pin["index"],
        "predecessor": pin["predecessor"],
        "scope": pin["scope"],
        "retired_stager_object": pin["retired_stager_object"],
        "retired_stager_bundle_rechecked": retired_stager_bundle_rechecked,
        "limitations": [
            "publication_and_durability_gate_only",
            "legacy_path_audit_covers_driver_process_only",
            "no_new_mathematics",
            "no_rank_release",
            "no_performance_or_parallel_claim",
        ],
    }
    success, report, report_sha256 = publish_final_report(
        out, root, initial_inputs, pin["inputs"], commands, failure,
        workspace_absent, exception_log_sha256, report)
    if not success:
        raise SystemExit(1)
    print(json.dumps({
        "job": job,
        "report_sha256": report_sha256,
        "status": report["status"],
    }, sort_keys=True), flush=True)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

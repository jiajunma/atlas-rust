"""Retained compute driver for the completed root-ladder AFTER v3 gate."""
from contextlib import ExitStack, contextmanager
import fcntl
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import signal
import stat
import subprocess
import sys
import time
import traceback


# Install the old-top-level guard before importing any project helper. Relative
# paths are normalized without touching the filesystem. The active campaign is
# the only atlas-* child of the HPC home this process may open.
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

from campaign_blob import SCHEMA as BLOB_SCHEMA, read_blob
from campaign_source import digest, file_manifest, materialize_source_archive
from campaign_workspace import (ACTIVE_CAMPAIGN, campaign_stage,
                                create_result_folder, ephemeral_job_workspace)
from progressive_submit import (queue_ids, read_json_file, save,
                                validate_confirmed_history)
from stage_ladder_boundary_after import frozen_stage_inputs, submission_receipt
from weyl_parent_seal import CAPTURE_CASES, boundary_bytes, load_parent_seal


if ACTIVE_CAMPAIGN != _ACTIVE_CAMPAIGN:
    raise RuntimeError("ladder AFTER v3 campaign policy changed")

STAGE_NAME = "ladder-boundary-after-v3"
PIN_NAME = "ladder-boundary-after-v3-pin.json"
PIN_SCHEMA = "atlas-ladder-boundary-after-pin-v3"
STAGE_LOCK = ".ladder-boundary-after-v3-stage.lock"
SUBMISSION_ENABLED = False
COMMAND_TIMEOUT_SECONDS = 1200
COMMAND_KILL_AFTER_SECONDS = 30
ROOT = "crates/atlas-real-group/src/root_system.rs"
SESSION = "crates/atlas-core/src/session.rs"
TEST_PATCH = "hpc/patches/ladder_boundary_tests.patch"
PRODUCTION_PATCH = "hpc/patches/ladder_boundary_fix.patch"
SBATCH = "hpc/math_ladder_boundary_after.sbatch"
BEFORE_EVIDENCE_PATH = (
    "tests/reference/hpc/math_ladder_boundary_before_v3_2026_10_01.json"
)
BEFORE_EVIDENCE_HASH = (
    "608e996acac10ea1bacca177468fcd83f3ec39c3e579899440e11aab674fc37f"
)
BEFORE_REFERENCE = {
    "job": "3873400",
    "report_sha256":
        "51cc7a14a0dbb8188a4ea47705338e5689212bf1f707819bab8c4e5681e4052b",
    "status": "LADDER_BOUNDARY_BEFORE_ACCEPTED",
}
V1_FAILURE_EVIDENCE_PATH = (
    "tests/reference/hpc/math_ladder_boundary_after_v1_failure_2026_10_01.json"
)
V1_FAILURE_EVIDENCE_HASH = (
    "b276865e1be26da0c2032dc8fc5953ec5181044efbdb949e6dd37f81fbcab4f0"
)
V1_FAILURE_REFERENCE = {
    "evidence_sha256": V1_FAILURE_EVIDENCE_HASH,
    "job": "3873497",
    "report_sha256":
        "fec90eb2f4cf0a4dff4820c4527c5263c151f8dff4aabf272007618646149e15",
    "status": "HARNESS_FAILURE",
}
V2_FAILURE_EVIDENCE_PATH = (
    "tests/reference/hpc/math_ladder_boundary_after_v2_failure_2026_10_01.json"
)
V2_FAILURE_EVIDENCE_HASH = (
    "e757f56ce6da55f1b96978daaaa2156b82b1bc3f83b36c2971f7a7a66be1f8f1"
)
V2_FAILURE_REFERENCE = {
    "evidence_sha256": V2_FAILURE_EVIDENCE_HASH,
    "job": "3874203",
    "report_sha256":
        "256247b2be5fd7358ccb56e87ff262c6c0e87ca782e822e2f3c96419c4b4a694",
    "status": "HARNESS_FAILURE",
}
LIFECYCLE = {
    "stage": STAGE_NAME,
    "changed_input_reasons": [
        (
            "Capture full libtest output and scrub inherited "
            "RUST_TEST_NOCAPTURE while focused tests retain nocapture."
        ),
        (
            "Hard-disable local ACTIVE/precreate state, freeze the exact 24 "
            "LEGACY identities, and double-sample Git plus no-follow Atlas "
            "sibling and linked-worktree admin ownership."
        ),
        (
            "Validate the complete pin and confirmed submission record before "
            "receipt publication, detach nested JSON state, and pin the exact "
            "full-stager inventory program."
        ),
    ],
    "retention_class": "ACTIVE_GATE_COMPACT",
    "retirement_condition": (
        "FINAL independent inspection, acceptance-index disposition, zero "
        "live or uncertain job ownership, verified campaign-CAS closure, and "
        "separate explicit deletion authorization."
    ),
}
PARENT_SEAL_REFERENCE = {
    "schema": "atlas-campaign-blob-v1",
    "role": "weyl-parent-seal",
    "sha256": "db67234c0d67dbd6a6f0327386dd113b6093d25a46569710c33dfc8bc482cdcb",
    "bytes": 12488095,
}
RETIRED_STAGER_BUNDLE_REFERENCE = {
    "schema": "atlas-campaign-blob-v1",
    "role": "retired-stager-bundle",
    "sha256": "550b1330ec8806d1343d50d6ceb8488f89eb03546f6bb37c538cb5cd460e9b09",
    "bytes": 311925,
}
DOMAIN_NAMES = {
    "root_system::tests::ladder_coordinate_boundary_roots",
    "root_system::tests::ladder_coordinate_boundary_coroots",
}
CORE_NAMES = {"session::tests::root_ladder_coordinate_boundary_original"}
TEST_HASHES = {
    ROOT: "109076ec05cf25e532fc7a951af0cbf5a725efefb318d35c2172a4528fd999f7",
    SESSION: "cef588d9afc7fcd513ee1660758b229a8767c252adacdec7feea9737675de6aa",
}
FINAL_HASHES = {
    ROOT: "cc6a1764e1c2425f7de8b4c27ca34a8bdc855c764d6db2a6ab9d6092e7b8cfe9",
    SESSION: "cef588d9afc7fcd513ee1660758b229a8767c252adacdec7feea9737675de6aa",
}
PATCH_HASHES = {
    TEST_PATCH: "69676d60b16591851570f58bc62ddc3bfe3a80b0ec6dad64a8eff36a9f777ca9",
    PRODUCTION_PATCH: "cada2e341bfba80ee587acb966413ce8cb6297593d6394b93890e02cdd730e75",
}
FIXTURE_PATH = "tests/math/generics/root_ladder_coordinate_boundary.atlas"
ORACLE_STDOUT_PATH = "tests/math/generics/root_ladder_coordinate_boundary.oracle.stdout"
ORACLE_STDERR_PATH = "tests/math/generics/root_ladder_coordinate_boundary.oracle.stderr"
FIXTURE_HASHES = {
    FIXTURE_PATH: "dc88d6606ae855b618dcf12589ecde82edcbe482873a94bcff1001163691ec65",
    ORACLE_STDOUT_PATH: "3a7fdade43c46cf4f3048b52cf81f282db060012951ee559296f017f7d3eab80",
    ORACLE_STDERR_PATH: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
}
BASE_STAGE_INPUT_NAMES = {
    "hpc/" + name for name in (
        "campaign_workspace.py", "test_campaign_workspace.py",
        "campaign_source.py", "test_campaign_source.py",
        "campaign_blob.py", "test_campaign_blob.py",
        "progressive_submit.py", "weyl_parent_seal.py",
        "stage_ladder_boundary_before.py", "math_ladder_boundary_before.py",
        "stage_ladder_boundary_after.py", "math_ladder_boundary_after.py",
        "test_math_ladder_boundary_after.py",
        "math_ladder_boundary_after.sbatch",
        "patches/ladder_boundary_tests.patch",
        "patches/ladder_boundary_fix.patch",
    )
}
CHILD_ONLY_INPUTS = {
    "hpc/stage_weyl_parent_seal.py",
    "hpc/test_stager_allowlist.py",
    "hpc/local_worktree_guard.py",
    "hpc/test_local_worktree_guard.py",
    "hpc/test_math_acceptance_index.py",
    "docs/worktree_registry.json",
    "tests/reference/hpc/math_acceptance_index_2026_10_01.json",
    "tests/reference/hpc/math_full_deform_after_2026_09_30.json",
    "tests/reference/hpc/math_cycle_rank1_2026_09_30.json",
    "tests/reference/hpc/math_cycle_rank1_review_2026_09_30.json",
    BEFORE_EVIDENCE_PATH,
    V1_FAILURE_EVIDENCE_PATH,
    V2_FAILURE_EVIDENCE_PATH,
}
STAGE_INPUT_NAMES = BASE_STAGE_INPUT_NAMES | CHILD_ONLY_INPUTS
SEALED_SHARED_INPUTS = STAGE_INPUT_NAMES - CHILD_ONLY_INPUTS - {
    "hpc/stage_ladder_boundary_before.py",
    "hpc/math_ladder_boundary_before.py",
    "hpc/stage_ladder_boundary_after.py",
    "hpc/math_ladder_boundary_after.py",
    "hpc/test_math_ladder_boundary_after.py",
    "hpc/math_ladder_boundary_after.sbatch",
    TEST_PATCH,
    PRODUCTION_PATCH,
}
CHECKERS = (
    ("test_campaign_workspace", 5),
    ("test_campaign_source", 10),
    ("test_campaign_blob", 6),
    ("test_local_worktree_guard", 22),
    ("test_stager_allowlist", 7),
    ("test_math_acceptance_index", 24),
    ("test_math_ladder_boundary_after", 21),
)
FULL_STAGER_INVENTORY_PROGRAM = (
    "import json, sys; "
    "sys.path.insert(0, sys.argv[1]); "
    "from campaign_blob import read_blob; "
    "from test_stager_allowlist import "
    "reconstructed_stage_sources, validate_historical_inventory; "
    "raw = read_blob(sys.argv[2], json.loads(sys.argv[3])); "
    "staged = reconstructed_stage_sources(raw); "
    "retired = validate_historical_inventory(staged); "
    "print('FULL_STAGE_INVENTORY_OK', len(staged), len(retired))"
)

SUBMISSION_RECORD_KEYS = {
    "stage", "script", "queue_before", "status", "max_outstanding", "job",
    "pin_sha256",
}


def sha256_bytes(value):
    return hashlib.sha256(value).hexdigest()


def stage_inputs(root):
    """Use the stager's exact stable no-follow frozen-input boundary."""
    return frozen_stage_inputs(root)


def valid_sha(value):
    return isinstance(value, str) and re.fullmatch(r"[a-f0-9]{64}", value) is not None


def validate_pin(pin, current_inputs):
    if (not isinstance(pin, dict)
            or set(pin) != {"schema", "inputs", "parent_seal_object",
                            "retired_stager_object",
                            "lifecycle",
                            "ladder_boundary_before_evidence",
                            "ladder_boundary_after_v1_failure",
                            "ladder_boundary_after_v2_failure",
                            "ladder_boundary_test_hashes",
                            "ladder_boundary_final_hashes",
                            "ladder_boundary_patch_hashes"}
            or pin.get("schema") != PIN_SCHEMA
            or pin.get("inputs") != current_inputs
            or set(current_inputs) != STAGE_INPUT_NAMES
            or any(not isinstance(name, str) or not valid_sha(sha)
                   for name, sha in current_inputs.items())
            or pin.get("ladder_boundary_test_hashes") != TEST_HASHES
            or pin.get("ladder_boundary_before_evidence") != BEFORE_REFERENCE
            or pin.get("ladder_boundary_after_v1_failure") != V1_FAILURE_REFERENCE
            or pin.get("ladder_boundary_after_v2_failure") != V2_FAILURE_REFERENCE
            or current_inputs.get(BEFORE_EVIDENCE_PATH) != BEFORE_EVIDENCE_HASH
            or current_inputs.get(V1_FAILURE_EVIDENCE_PATH)
               != V1_FAILURE_EVIDENCE_HASH
            or current_inputs.get(V2_FAILURE_EVIDENCE_PATH)
               != V2_FAILURE_EVIDENCE_HASH
            or pin.get("ladder_boundary_final_hashes") != FINAL_HASHES
            or pin.get("ladder_boundary_patch_hashes") != PATCH_HASHES
            or pin.get("lifecycle") != LIFECYCLE
            or any(current_inputs.get(name) != wanted
                   for name, wanted in PATCH_HASHES.items())):
        raise ValueError("changed ladder AFTER v3 pin")
    if pin.get("parent_seal_object") != PARENT_SEAL_REFERENCE:
        raise ValueError("parent seal reference is not the accepted predecessor")
    if pin.get("retired_stager_object") != RETIRED_STAGER_BUNDLE_REFERENCE:
        raise ValueError("retired stager bundle is not the frozen campaign object")
    return pin


def boundary_fixture(raw):
    if not isinstance(raw, dict) or set(raw) != {
            "input", "oracle_stdout", "oracle_stderr", "rust_stdout", "rust_stderr"}:
        raise ValueError("incomplete sealed ladder boundary streams")
    case = CAPTURE_CASES[2]
    name = case["id"]
    prefix = ('prints("MATH_BEGIN ' + name + '")\n').encode()
    suffix = ('\nprints("MATH_END ' + name + '")\nquit\n').encode()
    source = raw["input"]
    if (not isinstance(source, bytes) or not source.startswith(prefix)
            or not source.endswith(suffix)
            or sha256_bytes(source) != case["source_sha256"]):
        raise ValueError("sealed ladder input envelope changed")
    fixture = source[len(prefix):-len(suffix)]
    stdout = raw["oracle_stdout"]
    stderr = raw["oracle_stderr"]
    out_prefix = ("MATH_BEGIN " + name + "\n").encode()
    out_suffix = ("MATH_END " + name + "\nBye.\n").encode()
    if (sha256_bytes(fixture) != case["fixture_sha256"]
            or sha256_bytes(stdout) != FIXTURE_HASHES[ORACLE_STDOUT_PATH]
            or sha256_bytes(stderr) != FIXTURE_HASHES[ORACLE_STDERR_PATH]
            or not stdout.startswith(out_prefix) or not stdout.endswith(out_suffix)
            or stdout.count(b"LADDER_DATUM") != 11
            or stdout.count(b"LADDER_ROW") != 22
            or stderr != b""
            or raw["rust_stdout"].count(b"LADDER_ROW") != 10
            or raw["rust_stderr"].count(b"root-system arithmetic overflow") != 6):
        raise ValueError("sealed original acceptance or Rust boundary failure changed")
    return {
        FIXTURE_PATH: fixture,
        ORACLE_STDOUT_PATH: stdout,
        ORACLE_STDERR_PATH: stderr,
    }


def production_unchanged(before, after):
    boundary = b"\n#[cfg(test)]\nmod tests {"
    return before.count(boundary) == after.count(boundary) == 1 \
        and before.split(boundary)[0] == after.split(boundary)[0]


def production_patch_only(parent, tests_only, final):
    """Require the repair to preserve the tests-first suffix exactly."""
    boundary = b"\n#[cfg(test)]\nmod tests {"
    if any(value.count(boundary) != 1 for value in (parent, tests_only, final)):
        return False
    parent_production, _ = parent.split(boundary, 1)
    tests_production, tests_suffix = tests_only.split(boundary, 1)
    final_production, final_suffix = final.split(boundary, 1)
    return (parent_production == tests_production
            and tests_suffix == final_suffix
            and final_production != parent_production)


def passing_log(raw, kind, *, full=False):
    if type(full) is not bool:
        raise ValueError("invalid ladder AFTER result scope")
    names, total, filtered = (
        (DOMAIN_NAMES, 521, 519) if kind == "domain"
        else (CORE_NAMES, 630, 629) if kind == "core"
        else (None, None, None)
    )
    if names is None:
        raise ValueError("unknown ladder AFTER inventory kind")
    passed = total if full else len(names)
    expected_filtered = 0 if full else filtered
    result = re.findall(
        r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored; "
        r"\d+ measured; (\d+) filtered out",
        raw,
    )
    if (result != [(str(passed), "0", "0", str(expected_filtered))]
            or "panicked at" in raw or "test result: FAILED" in raw
            or "root-system arithmetic overflow" in raw):
        raise ValueError("exact passing ladder AFTER result required")
    return dict(
        passed=passed, failed=0, filtered_out=expected_filtered, ignored=0,
    )


def verify_inventory(inventory, baseline, additions, expected_count):
    expected = set(baseline) | set(additions)
    if (len(inventory) != expected_count or len(set(inventory)) != expected_count
            or len(expected) != expected_count or set(inventory) != expected):
        raise ValueError("ladder AFTER inventory changed")
    return inventory


def crate_test_commands(crate, selector):
    """Return the exact inventory, focused and captured-full test commands."""
    if (not isinstance(crate, str) or not crate
            or not isinstance(selector, str) or not selector):
        raise ValueError("invalid ladder AFTER v3 crate test command")
    cargo = ["cargo", "test", "--offline", "--locked", "-p", crate, "--lib"]
    return (
        cargo + ["--", "--list"],
        cargo + [selector, "--", "--nocapture", "--test-threads=1"],
        cargo + ["--", "--test-threads=1"],
    )


def command_environment(inherited, work):
    """Build an isolated command environment with libtest capture enforced."""
    env = {
        key: value for key, value in inherited.items()
        if key != "RUST_TEST_NOCAPTURE"
        and not key.startswith(("ATLAS_", "RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS",
                                "RUST_MIN_STACK", "CARGO_PROFILE_"))
    }
    env.update(
        CARGO_TARGET_DIR=str(Path(work) / "target"),
        CARGO_BUILD_JOBS="2",
        RAYON_NUM_THREADS="1",
        CARGO_PROFILE_TEST_DEBUG="0",
        CARGO_PROFILE_TEST_OPT_LEVEL="0",
        PYTHONDONTWRITEBYTECODE="1",
    )
    return env


def terminate_process_group(process):
    """Terminate and reap one command session without leaving its children."""
    if process is None:
        return
    if process.poll() is None:
        try:
            os.killpg(process.pid, signal.SIGTERM)
        except ProcessLookupError:
            pass
        try:
            process.wait(timeout=COMMAND_KILL_AFTER_SECONDS)
        except subprocess.TimeoutExpired:
            try:
                os.killpg(process.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
            process.wait(timeout=COMMAND_KILL_AFTER_SECONDS)
    else:
        process.wait()


def interrupt_active_process(active, signum):
    """Reap the active command group before entering the report-failure path."""
    process = active.get("process")
    try:
        terminate_process_group(process)
    finally:
        if active.get("process") is process:
            active["process"] = None
    raise RuntimeError("received signal " + str(signum))


def _stage_output(path, raw):
    """Create one fsynced, immutable publication candidate without following links."""
    path = Path(path)
    pending = path.with_name("." + path.name + ".pending")
    flags = (os.O_WRONLY | os.O_CREAT | os.O_EXCL
             | getattr(os, "O_CLOEXEC", 0) | getattr(os, "O_NOFOLLOW", 0))
    descriptor = None
    try:
        descriptor = os.open(pending, flags, 0o600)
        view = memoryview(raw)
        while view:
            written = os.write(descriptor, view)
            if written <= 0:
                raise OSError("short ladder AFTER publication write")
            view = view[written:]
        os.fchmod(descriptor, 0o444)
        os.fsync(descriptor)
        os.close(descriptor)
        descriptor = None
        return pending
    except Exception:
        if descriptor is not None:
            os.close(descriptor)
        try:
            pending.unlink()
        except FileNotFoundError:
            pass
        raise


def _discard_pending(*paths):
    for path in paths:
        if path is not None:
            try:
                path.unlink()
            except FileNotFoundError:
                pass


def publish_final_report(out, report):
    """Publish checksum first and the status-bearing report as the commit point."""
    out = Path(out)
    raw = (json.dumps(report, indent=2, sort_keys=True) + "\n").encode("utf-8")
    checksum = (sha256_bytes(raw) + "\n").encode("ascii")
    report_pending = checksum_pending = None
    directory_descriptor = None
    try:
        report_pending = _stage_output(out / "report.json", raw)
        checksum_pending = _stage_output(out / "report.sha256", checksum)
        directory_descriptor = os.open(
            out,
            os.O_RDONLY | getattr(os, "O_CLOEXEC", 0)
            | getattr(os, "O_DIRECTORY", 0) | getattr(os, "O_NOFOLLOW", 0),
        )
        os.replace(checksum_pending, out / "report.sha256")
        checksum_pending = None
        os.fsync(directory_descriptor)
        os.replace(report_pending, out / "report.json")
        report_pending = None
        os.fsync(directory_descriptor)
        os.close(directory_descriptor)
        directory_descriptor = None
        return
    except Exception:
        if directory_descriptor is not None:
            os.close(directory_descriptor)
        _discard_pending(report_pending, checksum_pending)
        raise


def publish_outcome(out, report):
    """Return false after any publication fault, with a best-effort failure report."""
    try:
        publish_final_report(out, report)
        return True
    except Exception:
        publication_error = traceback.format_exc()
    report.update(status="HARNESS_FAILURE", publication_error=publication_error)
    try:
        publish_final_report(out, report)
    except Exception:
        report["publication_fallback_error"] = traceback.format_exc()
        try:
            (Path(out) / "report.sha256").unlink()
        except FileNotFoundError:
            pass
    return False


def command(out, name, argv, cwd, env, active):
    start = time.monotonic()
    with (out / (name + ".log")).open("wb") as log:
        process = None
        blocked = {signal.SIGTERM, signal.SIGINT}
        old_mask = signal.pthread_sigmask(signal.SIG_BLOCK, blocked)
        try:
            process = subprocess.Popen(
                ["/usr/bin/time", "-v", "-o", str(out / (name + ".time")),
                 "timeout", "--signal=TERM",
                 "--kill-after=" + str(COMMAND_KILL_AFTER_SECONDS) + "s",
                 str(COMMAND_TIMEOUT_SECONDS), *argv],
                cwd=cwd, env=env, stdout=log, stderr=subprocess.STDOUT,
                start_new_session=True,
            )
            active["process"] = process
            signal.pthread_sigmask(signal.SIG_SETMASK, old_mask)
            old_mask = None
            returncode = process.wait()
        except BaseException:
            terminate_process_group(process)
            raise
        finally:
            if old_mask is not None:
                signal.pthread_sigmask(signal.SIG_SETMASK, old_mask)
            if active.get("process") is process:
                active["process"] = None
    metrics = (out / (name + ".time")).read_text()
    rss = re.search(r"Maximum resident set size \(kbytes\):\s*(\d+)", metrics)
    return dict(
        name=name,
        argv=argv,
        exit_status=returncode,
        seconds=time.monotonic() - start,
        maxrss_kb=int(rss[1]) if rss else None,
        maxrss_approximate=False,
        log_sha256=digest(out / (name + ".log")),
        time_sha256=digest(out / (name + ".time")),
    )


def verify_command_artifacts(out, commands):
    for record in commands:
        for suffix in ("log", "time"):
            if digest(out / (record["name"] + "." + suffix)) != record[suffix + "_sha256"]:
                raise ValueError("ladder AFTER command artifact changed")


def validate_submission_state(root, current_inputs, history, intent, job_id,
                              pin_sha):
    """Bind this compute process to one fully confirmed progressive submission."""
    validate_confirmed_history(history)
    matches = [row for row in history if row.get("stage") == str(root)]
    if len(matches) != 1:
        raise ValueError("campaign ledger lacks one unique ladder AFTER v3 submission")
    record = matches[0]
    queue = record.get("queue_before")
    if (set(record) != SUBMISSION_RECORD_KEYS
            or record.get("script") != SBATCH
            or record.get("status") != "SUBMITTED"
            or type(record.get("max_outstanding")) is not int
            or record["max_outstanding"] != 10
            or not isinstance(queue, list)
            or not all(isinstance(item, str) for item in queue)
            or queue_ids("\n".join(queue)) != queue
            or len(queue) >= 10
            or not isinstance(job_id, str) or not job_id.isdecimal()
            or record.get("job") != job_id
            or not isinstance(pin_sha, str)
            or re.fullmatch(r"[a-f0-9]{64}", pin_sha) is None
            or record.get("pin_sha256") != pin_sha
            or intent != record):
        raise ValueError("ladder AFTER v3 submission state is not exact and confirmed")
    return record


def validate_submission_receipt(record, pin, receipt):
    """Require one exact receipt derived from the confirmed shared record."""
    if receipt != submission_receipt(record, pin):
        raise ValueError("ladder AFTER v3 submission receipt changed")
    return receipt


@contextmanager
def existing_exclusive_lock(directory, name):
    """Lock an existing single-link inode without an O_CREAT recovery path."""
    if (not isinstance(name, str) or not name or "/" in name
            or name in (".", "..") or name.startswith("-")):
        raise ValueError("unsafe existing ladder AFTER v3 lock name")
    nofollow = getattr(os, "O_NOFOLLOW", None)
    directory_flag = getattr(os, "O_DIRECTORY", None)
    if nofollow is None or directory_flag is None:
        raise ValueError("safe existing ladder AFTER v3 lock traversal is unavailable")
    directory_descriptor = descriptor = None
    try:
        directory_descriptor = os.open(
            directory, os.O_RDONLY | directory_flag | nofollow)
        descriptor = os.open(
            name, os.O_RDWR | getattr(os, "O_CLOEXEC", 0) | nofollow,
            dir_fd=directory_descriptor)
    except OSError as error:
        if descriptor is not None:
            os.close(descriptor)
        if directory_descriptor is not None:
            os.close(directory_descriptor)
        raise ValueError("existing ladder AFTER v3 lock could not be opened") from error
    try:
        opened = os.fstat(descriptor)
        if (not stat.S_ISREG(opened.st_mode) or opened.st_nlink != 1
                or stat.S_IMODE(opened.st_mode) & 0o022):
            raise ValueError("existing ladder AFTER v3 lock is not private and single-link")
        try:
            fcntl.flock(descriptor, fcntl.LOCK_EX)
            current_descriptor = os.open(
                name, os.O_RDWR | getattr(os, "O_CLOEXEC", 0) | nofollow,
                dir_fd=directory_descriptor)
            try:
                current = os.fstat(current_descriptor)
            finally:
                os.close(current_descriptor)
        except OSError as error:
            raise ValueError("existing ladder AFTER v3 lock could not be acquired") from error
        identity = lambda value: (value.st_dev, value.st_ino, value.st_mode,
                                  value.st_nlink)
        if identity(current) != identity(opened):
            raise ValueError("existing ladder AFTER v3 lock changed while acquired")
        yield descriptor
    finally:
        os.close(descriptor)
        os.close(directory_descriptor)


def recover_submission_state(root, ledger_path, intent_path, receipt_path,
                             job_id, pin, pin_sha):
    """Finish only the exact pinned submission attempt running this job."""
    if (not isinstance(job_id, str) or not job_id.isdecimal()
            or not isinstance(pin_sha, str)
            or re.fullmatch(r"[a-f0-9]{64}", pin_sha) is None):
        raise ValueError("invalid running ladder AFTER v3 submission identity")
    if not os.path.lexists(ledger_path) or not os.path.lexists(intent_path):
        raise ValueError("ladder AFTER v3 submission attempt is incomplete")
    history = read_json_file(ledger_path)
    intent = read_json_file(intent_path)
    if not isinstance(history, list):
        raise ValueError("ladder AFTER v3 campaign ledger is not a list")
    matches = [(index, row) for index, row in enumerate(history)
               if isinstance(row, dict) and row.get("stage") == str(root)]
    if len(matches) != 1:
        raise ValueError("campaign ledger lacks one unique ladder AFTER v3 attempt")
    index, current = matches[0]
    uncertain_keys = SUBMISSION_RECORD_KEYS - {"job"}
    if (set(current) == uncertain_keys
            and current.get("status") == "SUBMISSION_INTENT_NOT_CONFIRMED"):
        if intent != current:
            raise ValueError("unconfirmed ladder AFTER v3 attempt state disagrees")
        confirmed = dict(current, status="SUBMITTED", job=job_id)
        repaired_history = list(history)
        repaired_history[index] = confirmed
    elif (set(current) == SUBMISSION_RECORD_KEYS
          and current.get("status") == "SUBMITTED"
          and current.get("job") == job_id):
        confirmed = current
        uncertain = dict(confirmed, status="SUBMISSION_INTENT_NOT_CONFIRMED")
        uncertain.pop("job")
        if intent != confirmed and intent != uncertain:
            raise ValueError("confirmed ladder AFTER v3 attempt state disagrees")
        repaired_history = history
    else:
        raise ValueError("campaign ledger does not identify this ladder AFTER v3 job")
    if (confirmed.get("script") != SBATCH
            or confirmed.get("pin_sha256") != pin_sha):
        raise ValueError("running ladder AFTER v3 attempt has different inputs")
    validate_confirmed_history(repaired_history)
    final_receipt = submission_receipt(confirmed, pin)
    if os.path.lexists(receipt_path):
        receipt = read_json_file(receipt_path)
        if history != repaired_history or intent != confirmed:
            raise ValueError("ladder AFTER v3 receipt exists for incomplete state")
        validate_submission_receipt(confirmed, pin, receipt)
        return
    if history != repaired_history:
        save(ledger_path, repaired_history)
    if intent != confirmed:
        save(intent_path, confirmed)
    save(receipt_path, final_receipt)


def gates(root, *, allow_recovery=True):
    if type(allow_recovery) is not bool:
        raise ValueError("ladder AFTER v3 recovery policy must be boolean")
    root = Path(root).resolve()
    campaign = campaign_stage(root)
    if root.name != STAGE_NAME:
        raise ValueError("unexpected ladder AFTER v3 stage")
    pin_sha = os.environ.get("LADDER_BOUNDARY_AFTER_PIN_SHA256")
    spool = os.environ.get("LADDER_BOUNDARY_AFTER_SPOOL")
    job_id = os.environ.get("SLURM_JOB_ID")
    if (not isinstance(pin_sha, str)
            or re.fullmatch(r"[a-f0-9]{64}", pin_sha) is None
            or not isinstance(spool, str) or not spool
            or not isinstance(job_id, str) or not job_id.isdecimal()):
        raise ValueError("missing or invalid ladder AFTER v3 environment")
    with existing_exclusive_lock(root, STAGE_LOCK):
        with existing_exclusive_lock(campaign, ".atlas-progressive-submit.lock"):
            pin_path = root / PIN_NAME
            current_inputs = stage_inputs(root)
            pin = validate_pin(read_json_file(pin_path, pin_sha), current_inputs)
            if digest(spool) != current_inputs[SBATCH]:
                raise ValueError("submitted ladder AFTER script changed")
            ledger_path = campaign / ".atlas-progressive-submit.json"
            intent_path = root / "submission-intent.json"
            receipt_path = root / "submission.json"
            if allow_recovery:
                recover_submission_state(
                    root, ledger_path, intent_path, receipt_path,
                    job_id, pin, pin_sha)
            history = read_json_file(ledger_path)
            intent = read_json_file(intent_path)
            record = validate_submission_state(
                root, current_inputs, history, intent, job_id, pin_sha)
            receipt = read_json_file(receipt_path)
            validate_submission_receipt(record, pin, receipt)
    seal = load_parent_seal(campaign, pin["parent_seal_object"], verify_archival=True)
    read_blob(campaign, pin["retired_stager_object"])
    if any(seal["migration_inputs"].get(name) != current_inputs[name]
           for name in SEALED_SHARED_INPUTS):
        raise ValueError("shared validator differs from the accepted parent seal")
    fixtures = boundary_fixture(boundary_bytes(campaign, seal))
    return pin, seal, fixtures, record, receipt, pin_sha


def main():
    # A direct ``sbatch`` of the staged script reaches this guard before a
    # scheduler environment check, path traversal, result creation, or report
    # write. The compute driver retains this independent emergency boundary
    # even while the reviewed ladder stage is active.
    if not SUBMISSION_ENABLED:
        raise SystemExit("completed ladder AFTER v3 compute driver is disabled")
    if not os.environ.get("SLURM_JOB_ID"):
        raise SystemExit("compute nodes only")
    root = Path.cwd().resolve()
    # The complete parent, pin, script-spool and sealed-stream validation is
    # read-only and must finish before the durable result folder is created.
    preflight = gates(root, allow_recovery=True)
    pin, seal, fixture_bytes, submission_record, submission_receipt_value, pin_sha = preflight
    out = create_result_folder(root, os.environ["SLURM_JOB_ID"])
    report = dict(
        schema="atlas-ladder-boundary-after-v3",
        job=os.environ["SLURM_JOB_ID"],
        node=platform.node(),
        status="HARNESS_FAILURE",
        commands=[],
        inventories={},
        after_regressions={},
        full_suites={},
        legacy_path_open_attempts=0,
        storage_policy=(
            "Parent evidence and accepted source are read only from active-campaign "
            "CAS; expanded source and Cargo target are disposable."
        ),
        scope=(
            "Original3868832-backed A1+torus boundary: two kernel assertions and "
            "one full-stream regression repaired by one exact production hunk, plus "
            "both complete crate lib suites. Not a performance claim."
        ),
        command_timeout_seconds=COMMAND_TIMEOUT_SECONDS,
        command_kill_after_seconds=COMMAND_KILL_AFTER_SECONDS,
    )

    active_command = {"process": None}

    def interrupted(signum, _frame):
        interrupt_active_process(active_command, signum)

    def checkpoint(phase):
        save(out / "report.json", dict(report, status="RUNNING", phase=phase))

    workspace = ExitStack()
    try:
        signal.signal(signal.SIGTERM, interrupted)
        signal.signal(signal.SIGINT, interrupted)
        report.update(
            pin=pin,
            pin_sha256=pin_sha,
            submission_record=submission_record,
            submission_receipt=submission_receipt_value,
            parent_seal_object=pin["parent_seal_object"],
            retired_stager_object=pin["retired_stager_object"],
            predecessor_failure=pin["ladder_boundary_after_v2_failure"],
            earlier_failure=pin["ladder_boundary_after_v1_failure"],
            lifecycle=pin["lifecycle"],
            parent_closure=seal["closure"],
            source_file_count=seal["source_file_count"],
        )
        work = workspace.enter_context(ephemeral_job_workspace(out, "ladder-after-v3"))
        env = command_environment(os.environ, work)

        def run(name, argv, cwd=root, expected=0):
            checkpoint(name)
            record = command(out, name, argv, cwd, env, active_command)
            report["commands"].append(record)
            if record["exit_status"] != expected or record["maxrss_kb"] is None:
                raise ValueError("unexpected command result: " + name)
            return (out / (name + ".log")).read_text(errors="replace")

        for name, count in CHECKERS:
            raw = run(
                name,
                [sys.executable, "-m", "unittest", "discover", "-s", "hpc",
                 "-p", name + ".py", "-v"],
            )
            if not re.search(r"Ran " + str(count) + r" tests in [0-9.]+s\s+OK\s*$", raw):
                raise ValueError("checker inventory changed")
        run("rustc", ["rustc", "-vV"])
        run("cargo", ["cargo", "-vV"])

        retired_reference = json.dumps(
            pin["retired_stager_object"], sort_keys=True, separators=(",", ":"))
        full_stager_log = run(
            "full-stager-inventory",
            [
                sys.executable,
                "-I",
                "-S",
                "-B",
                "-c",
                FULL_STAGER_INVENTORY_PROGRAM,
                str(root / "hpc"),
                str(campaign_stage(root)),
                retired_reference,
            ],
            root / "hpc",
        )
        if full_stager_log != "FULL_STAGE_INVENTORY_OK 70 67\n":
            raise ValueError("full historical stager inventory changed")
        source = work / "source"
        materialize_source_archive(
            campaign_stage(root), seal["source_object"], seal["source_manifest"], source,
        )
        old_bytes = {name: (source / name).read_bytes() for name in (ROOT, SESSION)}
        subprocess.run(
            ["patch", "--batch", "--fuzz=0", "--no-backup-if-mismatch", "-p1",
             "-i", str(root / TEST_PATCH)],
            cwd=source, check=True,
        )
        tests_only_bytes = {
            name: (source / name).read_bytes() for name in (ROOT, SESSION)
        }
        for name, wanted in TEST_HASHES.items():
            if (digest(source / name) != wanted
                    or not production_unchanged(old_bytes[name], tests_only_bytes[name])):
                raise ValueError("tests-only ladder patch changed production")
        for name, contents in fixture_bytes.items():
            destination = source / name
            destination.parent.mkdir(parents=True, exist_ok=True)
            destination.write_bytes(contents)

        tests_only_manifest = dict(seal["source_manifest"])
        tests_only_manifest.update(TEST_HASHES)
        tests_only_manifest.update(FIXTURE_HASHES)
        if file_manifest(source) != tests_only_manifest:
            raise ValueError("tests-only ladder source changed")

        subprocess.run(
            ["patch", "--batch", "--fuzz=0", "--no-backup-if-mismatch", "-p1",
             "-i", str(root / PRODUCTION_PATCH)],
            cwd=source, check=True,
        )
        final_bytes = {
            name: (source / name).read_bytes() for name in (ROOT, SESSION)
        }
        if (not production_patch_only(
                old_bytes[ROOT], tests_only_bytes[ROOT], final_bytes[ROOT])
                or final_bytes[SESSION] != tests_only_bytes[SESSION]
                or any(digest(source / name) != wanted
                       for name, wanted in FINAL_HASHES.items())):
            raise ValueError("ladder production repair differs from its exact hunk")
        expected_manifest = dict(seal["source_manifest"])
        expected_manifest.update(FINAL_HASHES)
        expected_manifest.update(FIXTURE_HASHES)
        if file_manifest(source) != expected_manifest:
            raise ValueError("final ladder AFTER source changed outside the repair")
        report.update(
            tests_only_source_hashes=TEST_HASHES,
            source_files=expected_manifest,
            patch_hashes=PATCH_HASHES,
            production_patch_only=True,
        )

        specifications = (
            ("domain", "atlas-real-group", seal["domain_inventory"], DOMAIN_NAMES,
             521, "ladder_coordinate_boundary_"),
            ("core", "atlas-core", seal["core_inventory"], CORE_NAMES,
             630, "root_ladder_coordinate_boundary_original"),
        )
        for kind, crate, baseline, additions, count, selector in specifications:
            inventory_command, focused_command, full_command = crate_test_commands(
                crate, selector)
            inventory_log = run(kind + "-inventory", inventory_command, source)
            inventory = re.findall(r"(?m)^(.+): test$", inventory_log)
            verify_inventory(inventory, baseline, additions, count)
            report["inventories"][kind] = inventory
            focused_log = run(
                kind + "-focused",
                focused_command,
                source,
            )
            report["after_regressions"][kind] = passing_log(focused_log, kind)
            full_log = run(
                kind + "-all",
                full_command,
                source,
            )
            report["full_suites"][kind] = passing_log(full_log, kind, full=True)

        final_preflight = gates(root, allow_recovery=False)
        if final_preflight != preflight:
            raise ValueError("ladder AFTER v3 preflight changed during execution")
        if file_manifest(source) != expected_manifest:
            raise ValueError("ladder AFTER source changed during regressions")
        verify_command_artifacts(out, report["commands"])
        if LEGACY_PATH_OPEN_ATTEMPTS:
            raise ValueError("legacy path access audit was not empty")
        report.update(
            status="LADDER_BOUNDARY_AFTER_GATES_PASS",
            integrity_rechecked=True,
            source_integrity_rechecked=True,
            legacy_path_open_attempts=0,
        )
    except Exception:
        report.update(status="HARNESS_FAILURE", error=traceback.format_exc())
    finally:
        try:
            terminate_process_group(active_command.get("process"))
            active_command["process"] = None
            workspace.close()
            if "work" in locals():
                if work.exists():
                    raise ValueError("ephemeral ladder AFTER v3 workspace was retained")
                if report["status"] != "HARNESS_FAILURE":
                    report["ephemeral_workspace_verified"] = True
        except Exception:
            report.update(status="HARNESS_FAILURE", cleanup_error=traceback.format_exc())
        report["legacy_path_open_attempts"] = len(LEGACY_PATH_OPEN_ATTEMPTS)
    published = publish_outcome(out, report)
    print(report["status"], out, flush=True)
    return int(not published or report["status"] != "LADDER_BOUNDARY_AFTER_GATES_PASS")


if __name__ == "__main__":
    raise SystemExit(main())

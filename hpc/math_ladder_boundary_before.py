"""Seal-only original-backed ladder overflow BEFORE; no production changes."""
from contextlib import ExitStack, contextmanager
import fcntl
import hashlib
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

from campaign_blob import SCHEMA as BLOB_SCHEMA
from campaign_source import digest, file_manifest, materialize_source_archive
from campaign_workspace import (ACTIVE_CAMPAIGN, campaign_stage,
                                create_result_folder, ephemeral_job_workspace)
from progressive_submit import (queue_ids, read_json_file, save,
                                validate_confirmed_history)
from stage_ladder_boundary_before import frozen_stage_inputs, submission_receipt
from weyl_parent_seal import CAPTURE_CASES, boundary_bytes, load_parent_seal


if ACTIVE_CAMPAIGN != _ACTIVE_CAMPAIGN:
    raise RuntimeError("ladder BEFORE v2 campaign policy changed")

STAGE_NAME = "ladder-boundary-before-v3"
PIN_NAME = "ladder-boundary-before-v3-pin.json"
PIN_SCHEMA = "atlas-ladder-boundary-before-pin-v2"
STAGE_LOCK = ".ladder-boundary-before-v3-stage.lock"
SUBMISSION_ENABLED = False
COMMAND_TIMEOUT_SECONDS = 1200
COMMAND_KILL_AFTER_SECONDS = 30
ROOT = "crates/atlas-real-group/src/root_system.rs"
SESSION = "crates/atlas-core/src/session.rs"
PATCH = "hpc/patches/ladder_boundary_tests.patch"
SBATCH = "hpc/math_ladder_boundary_before.sbatch"
DOMAIN_NAMES = {
    "root_system::tests::ladder_coordinate_boundary_roots",
    "root_system::tests::ladder_coordinate_boundary_coroots",
}
CORE_NAMES = {"session::tests::root_ladder_coordinate_boundary_original"}
TEST_HASHES = {
    ROOT: "109076ec05cf25e532fc7a951af0cbf5a725efefb318d35c2172a4528fd999f7",
    SESSION: "cef588d9afc7fcd513ee1660758b229a8767c252adacdec7feea9737675de6aa",
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
        "progressive_submit.py",
        "weyl_parent_seal.py", "math_ladder_boundary_before.py",
        "test_math_ladder_boundary_before.py",
        "math_ladder_boundary_before.sbatch",
        "stage_ladder_boundary_before.py",
        "patches/ladder_boundary_tests.patch",
    )
}
CHILD_ONLY_INPUTS = {
    "hpc/stage_weyl_parent_seal.py",
    "hpc/test_stager_allowlist.py",
    "hpc/test_math_acceptance_index.py",
    "tests/reference/hpc/math_acceptance_index_2026_10_01.json",
    "tests/reference/hpc/math_full_deform_after_2026_09_30.json",
    "tests/reference/hpc/math_cycle_rank1_2026_09_30.json",
    "tests/reference/hpc/math_cycle_rank1_review_2026_09_30.json",
}
STAGE_INPUT_NAMES = BASE_STAGE_INPUT_NAMES | CHILD_ONLY_INPUTS
SEALED_SHARED_INPUTS = STAGE_INPUT_NAMES - CHILD_ONLY_INPUTS - {
    "hpc/math_ladder_boundary_before.py",
    "hpc/test_math_ladder_boundary_before.py",
    "hpc/math_ladder_boundary_before.sbatch",
    "hpc/stage_ladder_boundary_before.py",
    PATCH,
}
CHECKERS = (
    ("test_campaign_workspace", 5),
    ("test_campaign_source", 10),
    ("test_campaign_blob", 6),
    ("test_stager_allowlist", 5),
    ("test_math_acceptance_index", 24),
    ("test_math_ladder_boundary_before", 15),
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
                            "ladder_boundary_test_hashes"}
            or pin.get("schema") != PIN_SCHEMA
            or pin.get("inputs") != current_inputs
            or set(current_inputs) != STAGE_INPUT_NAMES
            or any(not isinstance(name, str) or not valid_sha(sha)
                   for name, sha in current_inputs.items())
            or pin.get("ladder_boundary_test_hashes") != TEST_HASHES):
        raise ValueError("changed ladder BEFORE v2 pin")
    reference = pin.get("parent_seal_object")
    if (not isinstance(reference, dict)
            or set(reference) != {"schema", "role", "sha256", "bytes"}
            or reference.get("schema") != BLOB_SCHEMA
            or reference.get("role") != "weyl-parent-seal"
            or not valid_sha(reference.get("sha256"))
            or type(reference.get("bytes")) is not int
            or reference["bytes"] < 0):
        raise ValueError("invalid path-free parent seal reference")
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


def before_log(raw, kind):
    names, filtered = (DOMAIN_NAMES, 519) if kind == "domain" else (CORE_NAMES, 629)
    panics = re.findall(r"thread '([^']+)' (?:\(\d+\) )?panicked at", raw)
    result = re.findall(
        r"test result: FAILED\. (\d+) passed; (\d+) failed; (\d+) ignored; "
        r"\d+ measured; (\d+) filtered out",
        raw,
    )
    if (len(panics) != len(names) or set(panics) != names
            or result != [("0", str(len(names)), "0", str(filtered))]):
        raise ValueError("actual named boundary assertion failures required")
    if kind == "domain":
        for dual in ("false", "true"):
            message = "ladder membership must not reject m=1073741824, dual=" \
                + dual + ": Err(ArithmeticOverflow)"
            if raw.count(message) != 1:
                raise ValueError("wrong domain failure or failed setup")
    elif kind == "core":
        if raw.count("root-system arithmetic overflow") != 6:
            raise ValueError("six original-accepted overflow errors required")
    else:
        raise ValueError("unknown ladder BEFORE inventory kind")
    return dict(failed=len(names), passed=0, filtered_out=filtered, ignored=0)


def verify_inventory(inventory, baseline, additions, expected_count):
    expected = set(baseline) | set(additions)
    if (len(inventory) != expected_count or len(set(inventory)) != expected_count
            or len(expected) != expected_count or set(inventory) != expected):
        raise ValueError("ladder BEFORE inventory changed")
    return inventory


def command(out, name, argv, cwd, env):
    start = time.monotonic()
    with (out / (name + ".log")).open("wb") as log:
        done = subprocess.run(
            ["/usr/bin/time", "-v", "-o", str(out / (name + ".time")),
             "timeout", "--signal=TERM",
             "--kill-after=" + str(COMMAND_KILL_AFTER_SECONDS) + "s",
             str(COMMAND_TIMEOUT_SECONDS), *argv],
            cwd=cwd, env=env, stdout=log, stderr=subprocess.STDOUT,
        )
    metrics = (out / (name + ".time")).read_text()
    rss = re.search(r"Maximum resident set size \(kbytes\):\s*(\d+)", metrics)
    return dict(
        name=name,
        argv=argv,
        exit_status=done.returncode,
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
                raise ValueError("ladder BEFORE command artifact changed")


def validate_submission_state(root, current_inputs, history, intent, job_id,
                              pin_sha):
    """Bind this compute process to one fully confirmed progressive submission."""
    validate_confirmed_history(history)
    matches = [row for row in history if row.get("stage") == str(root)]
    if len(matches) != 1:
        raise ValueError("campaign ledger lacks one unique ladder v2 submission")
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
        raise ValueError("ladder v2 submission state is not exact and confirmed")
    return record


def validate_submission_receipt(record, pin, receipt):
    """Require one exact receipt derived from the confirmed shared record."""
    if receipt != submission_receipt(record, pin):
        raise ValueError("ladder v2 submission receipt changed")
    return receipt


@contextmanager
def existing_exclusive_lock(directory, name):
    """Lock an existing single-link inode without an O_CREAT recovery path."""
    if (not isinstance(name, str) or not name or "/" in name
            or name in (".", "..") or name.startswith("-")):
        raise ValueError("unsafe existing ladder v2 lock name")
    nofollow = getattr(os, "O_NOFOLLOW", None)
    directory_flag = getattr(os, "O_DIRECTORY", None)
    if nofollow is None or directory_flag is None:
        raise ValueError("safe existing ladder v2 lock traversal is unavailable")
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
        raise ValueError("existing ladder v2 lock could not be opened") from error
    try:
        opened = os.fstat(descriptor)
        if (not stat.S_ISREG(opened.st_mode) or opened.st_nlink != 1
                or stat.S_IMODE(opened.st_mode) & 0o022):
            raise ValueError("existing ladder v2 lock is not private and single-link")
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
            raise ValueError("existing ladder v2 lock could not be acquired") from error
        identity = lambda value: (value.st_dev, value.st_ino, value.st_mode,
                                  value.st_nlink)
        if identity(current) != identity(opened):
            raise ValueError("existing ladder v2 lock changed while acquired")
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
        raise ValueError("invalid running ladder v2 submission identity")
    if not os.path.lexists(ledger_path) or not os.path.lexists(intent_path):
        raise ValueError("ladder v2 submission attempt is incomplete")
    history = read_json_file(ledger_path)
    intent = read_json_file(intent_path)
    if not isinstance(history, list):
        raise ValueError("ladder v2 campaign ledger is not a list")
    matches = [(index, row) for index, row in enumerate(history)
               if isinstance(row, dict) and row.get("stage") == str(root)]
    if len(matches) != 1:
        raise ValueError("campaign ledger lacks one unique ladder v2 attempt")
    index, current = matches[0]
    uncertain_keys = SUBMISSION_RECORD_KEYS - {"job"}
    if (set(current) == uncertain_keys
            and current.get("status") == "SUBMISSION_INTENT_NOT_CONFIRMED"):
        if intent != current:
            raise ValueError("unconfirmed ladder v2 attempt state disagrees")
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
            raise ValueError("confirmed ladder v2 attempt state disagrees")
        repaired_history = history
    else:
        raise ValueError("campaign ledger does not identify this ladder v2 job")
    if (confirmed.get("script") != SBATCH
            or confirmed.get("pin_sha256") != pin_sha):
        raise ValueError("running ladder v2 attempt has different inputs")
    validate_confirmed_history(repaired_history)
    final_receipt = submission_receipt(confirmed, pin)
    if os.path.lexists(receipt_path):
        receipt = read_json_file(receipt_path)
        if history != repaired_history or intent != confirmed:
            raise ValueError("ladder v2 receipt exists for incomplete state")
        validate_submission_receipt(confirmed, pin, receipt)
        return
    if history != repaired_history:
        save(ledger_path, repaired_history)
    if intent != confirmed:
        save(intent_path, confirmed)
    save(receipt_path, final_receipt)


def gates(root, *, allow_recovery=True):
    if type(allow_recovery) is not bool:
        raise ValueError("ladder v2 recovery policy must be boolean")
    root = Path(root).resolve()
    campaign = campaign_stage(root)
    if root.name != STAGE_NAME:
        raise ValueError("unexpected ladder BEFORE v2 stage")
    pin_sha = os.environ.get("LADDER_BOUNDARY_BEFORE_PIN_SHA256")
    spool = os.environ.get("LADDER_BOUNDARY_BEFORE_SPOOL")
    job_id = os.environ.get("SLURM_JOB_ID")
    if (not isinstance(pin_sha, str)
            or re.fullmatch(r"[a-f0-9]{64}", pin_sha) is None
            or not isinstance(spool, str) or not spool
            or not isinstance(job_id, str) or not job_id.isdecimal()):
        raise ValueError("missing or invalid ladder v2 environment")
    with existing_exclusive_lock(root, STAGE_LOCK):
        with existing_exclusive_lock(campaign, ".atlas-progressive-submit.lock"):
            pin_path = root / PIN_NAME
            current_inputs = stage_inputs(root)
            pin = validate_pin(read_json_file(pin_path, pin_sha), current_inputs)
            if digest(spool) != current_inputs[SBATCH]:
                raise ValueError("submitted ladder BEFORE script changed")
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
        raise SystemExit("seal-only ladder BEFORE v2 driver is disabled")
    if not os.environ.get("SLURM_JOB_ID"):
        raise SystemExit("compute nodes only")
    root = Path.cwd().resolve()
    # The complete parent, pin, script-spool and sealed-stream validation is
    # read-only and must finish before the durable result folder is created.
    preflight = gates(root, allow_recovery=True)
    pin, seal, fixture_bytes, submission_record, submission_receipt_value, pin_sha = preflight
    out = create_result_folder(root, os.environ["SLURM_JOB_ID"])
    report = dict(
        schema="atlas-ladder-boundary-before-v2",
        job=os.environ["SLURM_JOB_ID"],
        node=platform.node(),
        status="HARNESS_FAILURE",
        commands=[],
        inventories={},
        before_regressions={},
        legacy_path_open_attempts=0,
        storage_policy=(
            "Parent evidence and accepted source are read only from active-campaign "
            "CAS; expanded source and Cargo target are disposable."
        ),
        scope=(
            "Original3868832-backed A1+torus boundary: two kernel assertions and "
            "one full-stream regression, unchanged production. Not a performance claim."
        ),
        command_timeout_seconds=COMMAND_TIMEOUT_SECONDS,
        command_kill_after_seconds=COMMAND_KILL_AFTER_SECONDS,
    )

    def interrupted(signum, _frame):
        raise RuntimeError("received signal " + str(signum))

    signal.signal(signal.SIGTERM, interrupted)
    signal.signal(signal.SIGINT, interrupted)

    def checkpoint(phase):
        save(out / "report.json", dict(report, status="RUNNING", phase=phase))

    workspace = ExitStack()
    try:
        report.update(
            pin=pin,
            pin_sha256=pin_sha,
            submission_record=submission_record,
            submission_receipt=submission_receipt_value,
            parent_seal_object=pin["parent_seal_object"],
            parent_closure=seal["closure"],
            source_file_count=seal["source_file_count"],
        )
        work = workspace.enter_context(ephemeral_job_workspace(out, "ladder-before-v2"))
        env = {
            key: value for key, value in os.environ.items()
            if not key.startswith(("ATLAS_", "RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS",
                                   "RUST_MIN_STACK", "CARGO_PROFILE_"))
        }
        env.update(
            CARGO_TARGET_DIR=str(work / "target"),
            CARGO_BUILD_JOBS="2",
            RAYON_NUM_THREADS="1",
            CARGO_PROFILE_TEST_DEBUG="0",
            CARGO_PROFILE_TEST_OPT_LEVEL="0",
            PYTHONDONTWRITEBYTECODE="1",
        )

        def run(name, argv, cwd=root, expected=0):
            checkpoint(name)
            record = command(out, name, argv, cwd, env)
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

        source = work / "source"
        materialize_source_archive(
            campaign_stage(root), seal["source_object"], seal["source_manifest"], source,
        )
        old_bytes = {name: (source / name).read_bytes() for name in (ROOT, SESSION)}
        subprocess.run(
            ["patch", "--batch", "--fuzz=0", "--no-backup-if-mismatch", "-p1",
             "-i", str(root / PATCH)],
            cwd=source, check=True,
        )
        for name, wanted in TEST_HASHES.items():
            if (digest(source / name) != wanted
                    or not production_unchanged(old_bytes[name], (source / name).read_bytes())):
                raise ValueError("tests-only ladder patch changed production")
        for name, contents in fixture_bytes.items():
            destination = source / name
            destination.parent.mkdir(parents=True, exist_ok=True)
            destination.write_bytes(contents)

        expected_manifest = dict(seal["source_manifest"])
        expected_manifest.update(TEST_HASHES)
        expected_manifest.update(FIXTURE_HASHES)
        if file_manifest(source) != expected_manifest:
            raise ValueError("tests-only ladder source changed")
        report.update(source_files=expected_manifest, production_unchanged=True)

        specifications = (
            ("domain", "atlas-real-group", seal["domain_inventory"], DOMAIN_NAMES,
             521, "ladder_coordinate_boundary_"),
            ("core", "atlas-core", seal["core_inventory"], CORE_NAMES,
             630, "root_ladder_coordinate_boundary_original"),
        )
        for kind, crate, baseline, additions, count, selector in specifications:
            cargo = ["cargo", "test", "--offline", "--locked", "-p", crate, "--lib"]
            inventory_log = run(kind + "-inventory", cargo + ["--", "--list"], source)
            inventory = re.findall(r"(?m)^(.+): test$", inventory_log)
            verify_inventory(inventory, baseline, additions, count)
            report["inventories"][kind] = inventory
            failure_log = run(
                kind + "-before",
                cargo + [selector, "--", "--nocapture", "--test-threads=1"],
                source,
                expected=101,
            )
            report["before_regressions"][kind] = before_log(failure_log, kind)

        final_preflight = gates(root, allow_recovery=False)
        if final_preflight != preflight:
            raise ValueError("ladder BEFORE v2 preflight changed during execution")
        if file_manifest(source) != expected_manifest:
            raise ValueError("ladder BEFORE source changed during regressions")
        verify_command_artifacts(out, report["commands"])
        if LEGACY_PATH_OPEN_ATTEMPTS:
            raise ValueError("legacy path access audit was not empty")
        report.update(
            status="LADDER_BOUNDARY_THREE_FAILURES_CONFIRMED",
            integrity_rechecked=True,
            source_integrity_rechecked=True,
            legacy_path_open_attempts=0,
        )
    except Exception:
        report.update(status="HARNESS_FAILURE", error=traceback.format_exc())
    finally:
        try:
            workspace.close()
            if "work" in locals():
                if work.exists():
                    raise ValueError("ephemeral ladder BEFORE v2 workspace was retained")
                if report["status"] != "HARNESS_FAILURE":
                    report["ephemeral_workspace_verified"] = True
        except Exception:
            report.update(status="HARNESS_FAILURE", cleanup_error=traceback.format_exc())
        report["legacy_path_open_attempts"] = len(LEGACY_PATH_OPEN_ATTEMPTS)
    save(out / "report.json", report)
    (out / "report.sha256").write_text(digest(out / "report.json") + "\n")
    print(report["status"], out, flush=True)
    return int(report["status"] == "HARNESS_FAILURE")


if __name__ == "__main__":
    raise SystemExit(main())

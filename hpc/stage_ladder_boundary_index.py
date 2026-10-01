"""Prepare the bounded root-ladder acceptance-index publication stage.

This launcher submits one pure-Python compute-node job which validates the
already accepted ladder report, its independent review and the append-only
index disposition.  The exact override-manifest SHA supplied at invocation is
the external trust root for the reviewed candidate bytes.
"""
from contextlib import contextmanager
import copy
import fcntl
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import secrets
import stat
import sys

sys.dont_write_bytecode = True

from campaign_blob import verify_blob
from campaign_source import file_manifest
from campaign_workspace import campaign_stage, submission_scope
from progressive_submit import (exclusive_lock, read_json_file, save,
                                submit_one, validate_confirmed_history)


STAGE_NAME = "ladder-boundary-index-v1"
PIN_NAME = "ladder-boundary-index-v1-pin.json"
PIN_SCHEMA = "atlas-ladder-boundary-index-pin-v1"
STAGE_LOCK = ".ladder-boundary-index-v1-stage.lock"
SBATCH = "hpc/math_ladder_boundary_index.sbatch"

# Input and pin hashes are deliberately not embedded here: this file and the
# compute driver are themselves pinned inputs, so embedding either hash would
# make a fixed point impossible.  The reviewed overrides.json SHA-256 supplied
# on the command line is the external trust root.  This is the sole active
# launcher; the parent, BEFORE and AFTER launchers are all fail-closed.
SUBMISSION_ENABLED = True

# These are exact reviewed discovery counts, not merely expected exit codes.
EXPECTED_TEST_COUNTS = {
    "test-stage-ladder-boundary-index": 20,
    "test-math-acceptance-index": 34,
    "test-stager-allowlist": 7,
}

INDEX_PATH = "tests/reference/hpc/math_acceptance_index_2026_10_01.json"
REPORT_PATH = (
    "tests/reference/hpc/"
    "math_ladder_boundary_after_v3_report_2026_10_01.json"
)
INSPECTION_PATH = (
    "tests/reference/hpc/math_ladder_boundary_after_v3_2026_10_01.json"
)
REVIEW_PATH = (
    "tests/reference/hpc/"
    "math_ladder_boundary_after_v3_index_review_2026_10_01.json"
)
BEFORE_EVIDENCE_PATH = (
    "tests/reference/hpc/math_ladder_boundary_before_v3_2026_10_01.json"
)
ORIGINAL_CAPTURE_PATH = (
    "tests/reference/hpc/math_weyl_context_capture_2026_09_30.json"
)

STAGE_INPUT_NAMES = {
    "hpc/campaign_blob.py",
    "hpc/campaign_source.py",
    "hpc/campaign_workspace.py",
    "hpc/progressive_submit.py",
    "hpc/stage_weyl_parent_seal.py",
    "hpc/stage_ladder_boundary_before.py",
    "hpc/stage_ladder_boundary_after.py",
    "hpc/stage_ladder_boundary_index.py",
    "hpc/math_ladder_boundary_before.py",
    "hpc/math_ladder_boundary_after.py",
    "hpc/math_ladder_boundary_index.py",
    "hpc/math_ladder_boundary_index.sbatch",
    "hpc/test_stager_allowlist.py",
    "hpc/test_math_acceptance_index.py",
    "hpc/test_stage_ladder_boundary_index.py",
    INDEX_PATH,
    "tests/reference/hpc/math_full_deform_after_2026_09_30.json",
    "tests/reference/hpc/math_cycle_rank1_2026_09_30.json",
    "tests/reference/hpc/math_cycle_rank1_review_2026_09_30.json",
    BEFORE_EVIDENCE_PATH,
    ORIGINAL_CAPTURE_PATH,
    REPORT_PATH,
    INSPECTION_PATH,
    REVIEW_PATH,
}

RETIRED_STAGER_BUNDLE_REFERENCE = {
    "schema": "atlas-campaign-blob-v1",
    "role": "retired-stager-bundle",
    "sha256": "550b1330ec8806d1343d50d6ceb8488f89eb03546f6bb37c538cb5cd460e9b09",
    "bytes": 311925,
}

PREDECESSOR = {
    "stage": (
        "/public/home/majj/atlas-rust-campaign-20260930/"
        "stages/ladder-boundary-after-v3"
    ),
    "job": "3875239",
    "pin_sha256": (
        "7af79aa20b37d0fcee9a60d8e3c6b4dbec05417cf6817747520b6ebe3ec41f88"
    ),
    "receipt_sha256": (
        "ac1f6f52f99de9b86b7b7bbfe56e7478809f193f1938a0c07219bc6061ee2254"
    ),
    "campaign_ledger_sha256": (
        "a1776412e7dd6633c329743b52c5cf4b1615dde8a7a2f0b4fddd002b5615a618"
    ),
    "report_sha256": (
        "771fc790dd4340f408235e50c3f6eee754ebe4c850cbad36d4af4f902a24627c"
    ),
    "inspection_sha256": (
        "a459fa08117ff8a721181d349467ebdd9267e798cd25b5bcb1380996c2d15e15"
    ),
    "review_sha256": (
        "3c0eed61cc5bf6af809096da73ac4bf1d8217b5cc81954a2e8000e5da44f9b58"
    ),
    "source_manifest_sha256": (
        "d2a6367379432c1ed09b90c903a072cfce0fb463349032100dd644db7c3fc213"
    ),
    "before_evidence_sha256": (
        "608e996acac10ea1bacca177468fcd83f3ec39c3e579899440e11aab674fc37f"
    ),
    "original_capture_sha256": (
        "bdee12811fe6ce4160ffbc2a25df18d9a9338a7b29f22d5fcd19352cf970f832"
    ),
    "previous_index_head_sha256": (
        "e44a8b6d8ed6d2a7781ca9f9e2ba4b2e57c06f118a5a288d7a61c0d77d022c6c"
    ),
    "index_head_sha256": (
        "1628ee21c71a91376a02982c38404cee958cb6183c57f791a42c4a668a1efc12"
    ),
}

LIFECYCLE = {
    "stage": STAGE_NAME,
    "changed_input_reasons": [
        (
            "Publish the independently reviewed A1-plus-torus root/coroot "
            "ladder boundary claim as the third append-only acceptance-index "
            "entry."
        ),
        (
            "Transition the frozen AFTER-v3 launcher policy to a separately "
            "bounded pure-Python index-verification launcher."
        ),
    ],
    "retention_class": "ACTIVE_GATE_COMPACT",
    "retirement_condition": (
        "FINAL independent inspection, focused commit and push, zero live or "
        "uncertain job ownership, verified campaign-CAS closure, and separate "
        "explicit exact-path deletion authorization."
    ),
}

SUBMISSION_RECORD_KEYS = {
    "stage", "script", "queue_before", "status", "max_outstanding", "job",
    "pin_sha256",
}
PIN_KEYS = {
    "schema", "inputs", "overrides_sha256", "test_counts", "predecessor",
    "index", "lifecycle", "retired_stager_object", "scope",
}
SHA256_PATTERN = r"[0-9a-f]{64}\Z"


def strict_json_loads(raw):
    """Decode JSON while rejecting duplicate keys and non-finite values."""
    def unique(pairs):
        value = {}
        for key, item in pairs:
            if key in value:
                raise ValueError("duplicate JSON key: " + key)
            value[key] = item
        return value

    def reject_constant(value):
        raise ValueError("non-finite JSON value: " + value)

    return json.loads(raw, object_pairs_hook=unique, parse_constant=reject_constant)


def saved_json_sha(value):
    """Hash the exact canonical byte form written by progressive_submit.save."""
    raw = (json.dumps(value, indent=2, sort_keys=True) + "\n").encode("utf-8")
    return hashlib.sha256(raw).hexdigest()


def validate_test_counts(test_counts):
    """Require exact discovery counts, including a frozen allowlist count."""
    if (not isinstance(test_counts, dict)
            or set(test_counts) != set(EXPECTED_TEST_COUNTS)
            or any(type(value) is not int or value < 1
                   for value in test_counts.values())
            or test_counts != EXPECTED_TEST_COUNTS):
        raise ValueError("ladder boundary index test counts are not frozen")
    return copy.deepcopy(test_counts)


def validate_configuration(inputs, overrides_sha256, test_counts):
    """Validate the external manifest trust root and return detached state."""
    if (not isinstance(inputs, dict) or set(inputs) != STAGE_INPUT_NAMES
            or any(not isinstance(name, str)
                   or not isinstance(value, str)
                   or re.fullmatch(SHA256_PATTERN, value) is None
                   for name, value in inputs.items())):
        raise ValueError("ladder boundary index input hashes are not frozen")
    if (not isinstance(overrides_sha256, str)
            or re.fullmatch(SHA256_PATTERN, overrides_sha256) is None):
        raise ValueError("ladder boundary index override manifest is not frozen")
    return copy.deepcopy(inputs), validate_test_counts(test_counts)


def require_enabled_launcher():
    if SUBMISSION_ENABLED is not True:
        raise ValueError("ladder boundary index submission remains disabled")
    return validate_test_counts(EXPECTED_TEST_COUNTS)


def _safe_relative(name):
    if not isinstance(name, str) or not name:
        raise ValueError("unsafe stage-relative input path")
    pure = PurePosixPath(name)
    if (pure.is_absolute()
            or pure.as_posix() != name
            or any(part in ("", ".", "..") for part in pure.parts)):
        raise ValueError("unsafe stage-relative input path")
    return pure.parts


def _directory_flags():
    nofollow = getattr(os, "O_NOFOLLOW", None)
    directory = getattr(os, "O_DIRECTORY", None)
    if nofollow is None or directory is None:
        raise ValueError("safe no-follow directory traversal is unavailable")
    return os.O_RDONLY | getattr(os, "O_CLOEXEC", 0) | nofollow | directory


def _open_directory(path):
    try:
        descriptor = os.open(path, _directory_flags())
    except OSError as error:
        raise ValueError("required stage directory is missing or unsafe") from error
    if not stat.S_ISDIR(os.fstat(descriptor).st_mode):
        os.close(descriptor)
        raise ValueError("required stage path is not a directory")
    return descriptor


def _open_child_directory(parent, name, create=False):
    if not isinstance(name, str) or name in ("", ".", "..") or "/" in name:
        raise ValueError("unsafe stage directory component")
    if create:
        try:
            os.mkdir(name, 0o755, dir_fd=parent)
            os.fsync(parent)
        except FileExistsError:
            pass
    try:
        descriptor = os.open(name, _directory_flags(), dir_fd=parent)
    except OSError as error:
        raise ValueError("stage directory component is missing or unsafe") from error
    if not stat.S_ISDIR(os.fstat(descriptor).st_mode):
        os.close(descriptor)
        raise ValueError("stage directory component is not a directory")
    return descriptor


@contextmanager
def _open_parent(root, name, create=False):
    descriptors = []
    try:
        current = _open_directory(root)
        descriptors.append(current)
        parts = _safe_relative(name)
        for part in parts[:-1]:
            current = _open_child_directory(current, part, create=create)
            descriptors.append(current)
        yield current, parts[-1]
    finally:
        for descriptor in reversed(descriptors):
            os.close(descriptor)


def stable_relative_bytes(root, name, *, mode=None, nlink=None):
    """Read one stable regular file without following any path component."""
    descriptor = current_descriptor = None
    try:
        with _open_parent(root, name) as (parent, leaf):
            flags = (os.O_RDONLY | getattr(os, "O_CLOEXEC", 0)
                     | getattr(os, "O_NOFOLLOW", 0))
            descriptor = os.open(leaf, flags, dir_fd=parent)
            before = os.fstat(descriptor)
            if (not stat.S_ISREG(before.st_mode)
                    or (mode is not None and stat.S_IMODE(before.st_mode) != mode)
                    or (nlink is not None and before.st_nlink != nlink)):
                raise ValueError("stage input is not the required regular file")
            chunks = []
            while True:
                chunk = os.read(descriptor, 64 * 1024)
                if not chunk:
                    break
                chunks.append(chunk)
            after = os.fstat(descriptor)
            current_descriptor = os.open(leaf, flags, dir_fd=parent)
            current = os.fstat(current_descriptor)
        identity = lambda value: (
            value.st_dev, value.st_ino, value.st_size, value.st_mtime_ns,
            value.st_ctime_ns, stat.S_IFMT(value.st_mode), value.st_nlink,
        )
        raw = b"".join(chunks)
        if (identity(before) != identity(after)
                or identity(before) != identity(current)
                or len(raw) != before.st_size):
            raise ValueError("stage input changed while it was read")
        return raw
    except OSError as error:
        raise ValueError("stage input could not be read safely") from error
    finally:
        if current_descriptor is not None:
            os.close(current_descriptor)
        if descriptor is not None:
            os.close(descriptor)


def load_relative_json(root, name, wanted, *, mode=None, nlink=None):
    raw = stable_relative_bytes(root, name, mode=mode, nlink=nlink)
    if hashlib.sha256(raw).hexdigest() != wanted:
        raise ValueError("changed JSON stage input: " + name)
    try:
        value = strict_json_loads(raw.decode("utf-8"))
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise ValueError("invalid JSON stage input: " + name) from error
    return value


def frozen_stage_inputs(root):
    """Require exact executable/evidence trees and rehash every frozen input."""
    root = Path(root)
    for namespace in ("hpc", "tests"):
        prefix = namespace + "/"
        namespace_root = root / namespace
        expected = {
            name[len(prefix):]
            for name in STAGE_INPUT_NAMES
            if name.startswith(prefix)
        }
        expected_directories = set()
        for name in expected:
            for parent in PurePosixPath(name).parents:
                if parent.as_posix() != ".":
                    expected_directories.add(parent.as_posix())
        namespace_manifest = file_manifest(namespace_root)
        actual_directories = set()
        for path in namespace_root.rglob("*"):
            value = path.lstat()
            if stat.S_ISDIR(value.st_mode):
                actual_directories.add(str(path.relative_to(namespace_root)))
        if (set(namespace_manifest) != expected
                or actual_directories != expected_directories):
            raise ValueError(
                "installed " + namespace + " namespace differs from the pin")
    result = {}
    for name in sorted(STAGE_INPUT_NAMES):
        raw = stable_relative_bytes(root, name, mode=0o444, nlink=1)
        result[name] = hashlib.sha256(raw).hexdigest()
    return result


def _write_all(descriptor, raw):
    remaining = memoryview(raw)
    while remaining:
        written = os.write(descriptor, remaining)
        if written <= 0:
            raise ValueError("short stage-input write")
        remaining = remaining[written:]


def _atomic_install(root, incoming, name, raw, wanted):
    """Publish one immutable input with held no-follow directory handles."""
    with _open_parent(root, name, create=True) as (destination, leaf):
        try:
            present = stable_relative_bytes(root, name, mode=0o444, nlink=1)
        except ValueError:
            present = None
        if present is not None:
            if hashlib.sha256(present).hexdigest() != wanted:
                raise ValueError("installed stage input changed: " + name)
            return

        incoming_descriptor = _open_directory(incoming)
        temporary_name = ".copy-" + secrets.token_hex(16)
        descriptor = None
        linked = False
        try:
            flags = (os.O_RDWR | os.O_CREAT | os.O_EXCL
                     | getattr(os, "O_CLOEXEC", 0)
                     | getattr(os, "O_NOFOLLOW", 0))
            descriptor = os.open(
                temporary_name, flags, 0o600, dir_fd=incoming_descriptor)
            _write_all(descriptor, raw)
            os.fchmod(descriptor, 0o444)
            os.fsync(descriptor)
            if hashlib.sha256(raw).hexdigest() != wanted:
                raise ValueError("override bytes changed before publication")
            try:
                os.link(
                    temporary_name,
                    leaf,
                    src_dir_fd=incoming_descriptor,
                    dst_dir_fd=destination,
                    follow_symlinks=False,
                )
                linked = True
                os.fsync(destination)
            except FileExistsError:
                present = stable_relative_bytes(root, name, mode=0o444, nlink=1)
                if hashlib.sha256(present).hexdigest() != wanted:
                    raise ValueError("stage input appeared with changed bytes")
        finally:
            if descriptor is not None:
                os.close(descriptor)
            try:
                os.unlink(temporary_name, dir_fd=incoming_descriptor)
            except FileNotFoundError:
                pass
            os.fsync(incoming_descriptor)
            os.close(incoming_descriptor)
        if linked:
            final = stable_relative_bytes(root, name, mode=0o444, nlink=1)
            if hashlib.sha256(final).hexdigest() != wanted:
                raise ValueError("published stage input changed: " + name)


def _prepare_incoming(root):
    root_descriptor = _open_directory(root)
    try:
        incoming_descriptor = _open_child_directory(
            root_descriptor, ".incoming", create=True)
        try:
            if os.listdir(incoming_descriptor):
                raise ValueError(
                    "nonempty publication scratch requires manual reconciliation"
                )
        finally:
            os.close(incoming_descriptor)
    finally:
        os.close(root_descriptor)
    return Path(root) / ".incoming"


def install_inputs(root, overrides):
    incoming = _prepare_incoming(root)
    override_root = Path(root) / "overrides"
    for name in sorted(STAGE_INPUT_NAMES):
        raw = stable_relative_bytes(
            override_root, name, mode=0o444, nlink=1)
        if hashlib.sha256(raw).hexdigest() != overrides[name]:
            raise ValueError("changed frozen override: " + name)
        _atomic_install(root, incoming, name, raw, overrides[name])
    if any(incoming.iterdir()):
        raise ValueError("publication scratch is not empty after installation")


def validate_predecessor(root, inputs):
    """Bind the report, independent inspection, review and third index head."""
    report = load_relative_json(root, REPORT_PATH, inputs[REPORT_PATH])
    inspection = load_relative_json(root, INSPECTION_PATH, inputs[INSPECTION_PATH])
    review = load_relative_json(root, REVIEW_PATH, inputs[REVIEW_PATH])
    index = load_relative_json(root, INDEX_PATH, inputs[INDEX_PATH])
    before_evidence = load_relative_json(
        root, BEFORE_EVIDENCE_PATH, inputs[BEFORE_EVIDENCE_PATH])
    original_capture = load_relative_json(
        root, ORIGINAL_CAPTURE_PATH, inputs[ORIGINAL_CAPTURE_PATH])
    if not all(isinstance(value, dict)
               for value in (report, inspection, review, index,
                             before_evidence, original_capture)):
        raise ValueError("root-ladder predecessor JSON must contain objects")
    if (inputs[REPORT_PATH] != PREDECESSOR["report_sha256"]
            or inputs[INSPECTION_PATH] != PREDECESSOR["inspection_sha256"]
            or inputs[REVIEW_PATH] != PREDECESSOR["review_sha256"]
            or inputs[BEFORE_EVIDENCE_PATH]
            != PREDECESSOR["before_evidence_sha256"]
            or inputs[ORIGINAL_CAPTURE_PATH]
            != PREDECESSOR["original_capture_sha256"]):
        raise ValueError("root-ladder predecessor file hashes changed")
    if (report.get("schema") != "atlas-ladder-boundary-after-v3"
            or report.get("job") != PREDECESSOR["job"]
            or report.get("status") != "LADDER_BOUNDARY_AFTER_GATES_PASS"
            or report.get("source_integrity_rechecked") is not True
            or report.get("integrity_rechecked") is not True):
        raise ValueError("root-ladder execution report identity changed")
    if (inspection.get("schema")
            != "atlas-ladder-boundary-after-inspection-v3"
            or inspection.get("status") != "LADDER_BOUNDARY_AFTER_ACCEPTED"
            or inspection.get("accounting", {}).get("job")
            != PREDECESSOR["job"]
            or inspection.get("accounting", {}).get("state") != "COMPLETED"
            or inspection.get("accounting", {}).get("exit_code") != "0:0"
            or inspection.get("artifacts", {}).get("report", {}).get(
                "file_sha256") != PREDECESSOR["report_sha256"]
            or inspection.get("artifacts", {}).get("pin_sha256")
            != PREDECESSOR["pin_sha256"]
            or inspection.get("artifacts", {}).get("receipt_sha256")
            != PREDECESSOR["receipt_sha256"]
            or inspection.get("artifacts", {}).get("campaign_ledger_sha256")
            != PREDECESSOR["campaign_ledger_sha256"]):
        raise ValueError("root-ladder independent inspection changed")
    expected_report = {"file": REPORT_PATH,
                       "sha256": PREDECESSOR["report_sha256"]}
    expected_inspection = {"file": INSPECTION_PATH,
                           "sha256": PREDECESSOR["inspection_sha256"]}
    expected_before_evidence = {
        "file": BEFORE_EVIDENCE_PATH,
        "sha256": PREDECESSOR["before_evidence_sha256"],
    }
    expected_original_capture = {
        "file": ORIGINAL_CAPTURE_PATH,
        "sha256": PREDECESSOR["original_capture_sha256"],
    }
    if (review.get("schema") != "atlas-root-ladder-acceptance-review-v1"
            or review.get("status")
            != "ROOT_LADDER_COORDINATE_BOUNDARY_REVIEWED"
            or review.get("job") != PREDECESSOR["job"]
            or review.get("operation") != "root_ladder"
            or review.get("claim_id")
            != "a1_torus_root_coroot_ladder_coordinate_boundary"
            or review.get("execution_report") != expected_report
            or review.get("independent_inspection") != expected_inspection
            or review.get("before_evidence") != expected_before_evidence
            or review.get("original_capture") != expected_original_capture
            or review.get("source_manifest_sha256")
            != PREDECESSOR["source_manifest_sha256"]):
        raise ValueError("root-ladder index review changed")
    entries = index.get("entries") if isinstance(index, dict) else None
    if (index.get("schema") != "atlas-math-acceptance-index-v1"
            or not isinstance(entries, list) or len(entries) != 3
            or entries[1].get("entry_sha256")
            != PREDECESSOR["previous_index_head_sha256"]
            or entries[2].get("entry_sha256")
            != PREDECESSOR["index_head_sha256"]
            or entries[2].get("claim_id")
            != "a1_torus_root_coroot_ladder_coordinate_boundary"
            or entries[2].get("acceptance") != "accepted"
            or entries[2].get("status") != "math_pass"
            or entries[2].get("report") != expected_report
            or entries[2].get("review_evidence")
            != {"file": REVIEW_PATH,
                "sha256": PREDECESSOR["review_sha256"]}):
        raise ValueError("root-ladder acceptance-index tail changed")
    return copy.deepcopy(index)


def build_pin(inputs, overrides_sha256, test_counts):
    frozen, counts = validate_configuration(
        inputs, overrides_sha256, test_counts)
    return {
        "schema": PIN_SCHEMA,
        "inputs": frozen,
        "overrides_sha256": overrides_sha256,
        "test_counts": counts,
        "predecessor": copy.deepcopy(PREDECESSOR),
        "index": {
            "file": INDEX_PATH,
            "entries": 3,
            "head_sha256": PREDECESSOR["index_head_sha256"],
            "claim_id": "a1_torus_root_coroot_ladder_coordinate_boundary",
        },
        "lifecycle": copy.deepcopy(LIFECYCLE),
        "retired_stager_object": copy.deepcopy(
            RETIRED_STAGER_BUNDLE_REFERENCE),
        "scope": (
            "Pure-Python durability verification of the independently reviewed "
            "bounded root-ladder acceptance-index entry; no new mathematics, "
            "rank release, benchmark, Cargo build or Atlas execution."
        ),
    }


def validate_pin(pin):
    if not isinstance(pin, dict) or set(pin) != PIN_KEYS:
        raise ValueError("changed ladder boundary index pin")
    try:
        wanted = build_pin(
            pin["inputs"], pin["overrides_sha256"], pin["test_counts"])
    except (KeyError, TypeError, ValueError) as error:
        raise ValueError("changed ladder boundary index pin") from error
    if pin != wanted:
        raise ValueError("changed ladder boundary index pin")
    return copy.deepcopy(pin)


def validate_submission_record(record, pin, root=None):
    validate_pin(pin)
    if not isinstance(record, dict) or set(record) != SUBMISSION_RECORD_KEYS:
        raise ValueError("ladder boundary index submission record is not exact")
    try:
        validate_confirmed_history([record])
    except ValueError as error:
        raise ValueError("ladder boundary index submission is not confirmed") from error
    stage = Path(record["stage"])
    if (not stage.is_absolute() or str(stage) != record["stage"]
            or ".." in stage.parts or stage.name != STAGE_NAME
            or record.get("script") != SBATCH
            or record.get("pin_sha256") != saved_json_sha(pin)):
        raise ValueError("ladder boundary index submission record changed")
    if root is not None and stage != Path(root).resolve():
        raise ValueError("submission record names another stage")
    return copy.deepcopy(record)


def validate_campaign_history(history, root, *, require_own=False):
    """Require the six-record predecessor plus at most this exact successor."""
    validate_confirmed_history(history)
    root = str(Path(root).resolve())
    matches = [row for row in history if row.get("stage") == root]
    if len(matches) > 1 or (require_own and len(matches) != 1):
        raise ValueError("campaign ledger has an invalid index-stage multiplicity")
    if matches:
        if history[-1] != matches[0]:
            raise ValueError("index stage is not the direct campaign successor")
        predecessor = history[:-1]
    else:
        predecessor = history
    if saved_json_sha(predecessor) != PREDECESSOR["campaign_ledger_sha256"]:
        raise ValueError("campaign ledger predecessor changed")
    return copy.deepcopy(matches[0]) if matches else None


def submission_receipt(record, pin, root=None):
    record = validate_submission_record(record, pin, root=root)
    return copy.deepcopy(dict(
        record,
        schema="atlas-ladder-boundary-index-submission-v1",
        status="SUBMITTED_NOT_VERIFIED",
        test_counts=pin["test_counts"],
        predecessor=pin["predecessor"],
        index=pin["index"],
        lifecycle=pin["lifecycle"],
        retired_stager_object=pin["retired_stager_object"],
        scope=pin["scope"],
        cargo_commands=0,
        atlas_commands=0,
        storage_policy=(
            "One existing-campaign child; only pin, immutable inputs, command "
            "logs/timing, report and checksum remain durable."
        ),
    ))


@contextmanager
def existing_lock(directory, name):
    """Lock one existing inode; receipt recovery must never create the lock."""
    directory_descriptor = descriptor = current_descriptor = None
    try:
        directory_descriptor = _open_directory(directory)
        flags = os.O_RDWR | getattr(os, "O_CLOEXEC", 0) | getattr(os, "O_NOFOLLOW", 0)
        descriptor = os.open(name, flags, dir_fd=directory_descriptor)
        opened = os.fstat(descriptor)
        if (not stat.S_ISREG(opened.st_mode) or opened.st_nlink != 1
                or stat.S_IMODE(opened.st_mode) & 0o022):
            raise ValueError("existing submission lock is unsafe")
        fcntl.flock(descriptor, fcntl.LOCK_EX)
        current_descriptor = os.open(name, flags, dir_fd=directory_descriptor)
        current = os.fstat(current_descriptor)
        identity = lambda value: (
            value.st_dev, value.st_ino, stat.S_IFMT(value.st_mode), value.st_nlink,
        )
        if identity(current) != identity(opened):
            raise ValueError("existing submission lock changed while acquired")
        yield descriptor
    except OSError as error:
        raise ValueError("existing submission lock could not be acquired") from error
    finally:
        if current_descriptor is not None:
            os.close(current_descriptor)
        if descriptor is not None:
            os.close(descriptor)
        if directory_descriptor is not None:
            os.close(directory_descriptor)


def confirmed_record(root, pin, *, repair_intent=False, expected_job=None):
    root = Path(root).resolve()
    campaign = submission_scope(root)
    ledger_path = campaign / ".atlas-progressive-submit.json"
    intent_path = root / "submission-intent.json"
    with existing_lock(campaign, ".atlas-progressive-submit.lock"):
        history = read_json_file(ledger_path)
        candidate = validate_campaign_history(history, root, require_own=True)
        record = validate_submission_record(candidate, pin, root=root)
        if expected_job is not None and record.get("job") != expected_job:
            raise ValueError("compute allocation differs from the durable record")
        intent = read_json_file(intent_path)
        if intent != record:
            uncertain_keys = SUBMISSION_RECORD_KEYS - {"job"}
            same_attempt = (
                isinstance(intent, dict)
                and set(intent) == uncertain_keys
                and intent.get("status") == "SUBMISSION_INTENT_NOT_CONFIRMED"
                and all(intent.get(key) == record.get(key)
                        for key in uncertain_keys - {"status"})
            )
            if not repair_intent or not same_attempt:
                raise ValueError("index intent and campaign ledger disagree")
            save(intent_path, record)
        return record


def submit_pinned(root, pin_sha256):
    if not SUBMISSION_ENABLED:
        raise ValueError("ladder boundary index submission remains disabled")
    return submit_one(
        root,
        SBATCH,
        dict(os.environ, LADDER_BOUNDARY_INDEX_PIN_SHA256=pin_sha256),
        pin_sha256=pin_sha256,
    )


def _read_override_manifest(root, wanted):
    overrides_root = Path(root) / "overrides"
    manifest = load_relative_json(
        overrides_root, "overrides.json", wanted, mode=0o444, nlink=1)
    manifest, _ = validate_configuration(
        manifest, wanted, EXPECTED_TEST_COUNTS)
    expected_tree = dict(manifest)
    expected_tree["overrides.json"] = wanted
    if file_manifest(overrides_root) != expected_tree:
        raise ValueError("override tree contains changed or extra files")
    return copy.deepcopy(manifest)


def run_enabled(root, overrides_sha256):
    """Validate fully, then publish once and use the shared one-job boundary."""
    test_counts = require_enabled_launcher()
    if (not isinstance(overrides_sha256, str)
            or re.fullmatch(SHA256_PATTERN, overrides_sha256) is None):
        raise ValueError("invalid override-manifest trust root")
    root = Path(root).resolve()
    campaign = campaign_stage(root)
    if root.name != STAGE_NAME:
        raise ValueError("unexpected ladder boundary index stage")
    root_descriptor = _open_directory(root)
    os.close(root_descriptor)
    manifest = _read_override_manifest(root, overrides_sha256)
    validate_predecessor(Path(root) / "overrides", manifest)
    verify_blob(campaign, RETIRED_STAGER_BUNDLE_REFERENCE)

    campaign = submission_scope(root)
    with existing_lock(campaign, ".atlas-progressive-submit.lock"):
        history = read_json_file(campaign / ".atlas-progressive-submit.json")
        has_own_record = validate_campaign_history(history, root) is not None
    if has_own_record and not (
            os.path.lexists(root / "submission-intent.json")
            or os.path.lexists(root / "submission.json")):
        raise ValueError("campaign ledger record lacks durable stage state")

    with exclusive_lock(root, STAGE_LOCK):
        install_inputs(root, manifest)
        if frozen_stage_inputs(root) != manifest:
            raise ValueError("installed index-stage inputs changed")
        validate_predecessor(root, manifest)
        pin = build_pin(manifest, overrides_sha256, test_counts)
        pin_path = root / PIN_NAME
        if os.path.lexists(pin_path):
            if read_json_file(pin_path, saved_json_sha(pin)) != pin:
                raise ValueError("prepared index-stage pin changed")
        else:
            save(pin_path, pin)
        pin_sha256 = saved_json_sha(pin)
        receipt_path = root / "submission.json"
        intent_path = root / "submission-intent.json"
        if os.path.lexists(receipt_path):
            record = confirmed_record(root, pin)
            receipt = read_json_file(receipt_path)
            if receipt != submission_receipt(record, pin, root=root):
                raise ValueError("ladder boundary index receipt changed")
        elif os.path.lexists(intent_path):
            record = confirmed_record(root, pin, repair_intent=True)
            receipt = submission_receipt(record, pin, root=root)
            save(receipt_path, receipt)
        else:
            record = submit_pinned(root, pin_sha256)
            receipt = submission_receipt(record, pin, root=root)
            save(receipt_path, receipt)
        print(json.dumps(receipt, sort_keys=True), flush=True)
        return copy.deepcopy(receipt)


def main():
    if not SUBMISSION_ENABLED:
        raise SystemExit("ladder boundary index launcher is not frozen or enabled")
    if len(sys.argv) != 3:
        raise SystemExit("usage: stage ROOT OVERRIDES_SHA256")
    run_enabled(Path(sys.argv[1]), sys.argv[2])


if __name__ == "__main__":
    main()

"""Prepare the v3 attempt of the seal-only ladder BEFORE stage.

The parent seal has been accepted on HPC. ``SUBMISSION_ENABLED`` remains an
independent, first-statement fail-closed feature guard for this launcher. The
pin and receipt retain their v2 schemas because their structure is unchanged.
"""
from contextlib import contextmanager
import fcntl
import hashlib
import json
import os
from pathlib import Path
import re
import secrets
import stat
import sys

sys.dont_write_bytecode = True
from campaign_blob import SCHEMA as BLOB_SCHEMA
from campaign_source import file_manifest
from campaign_workspace import campaign_stage, submission_scope
from progressive_submit import (exclusive_lock, queue_ids, read_json_file, save,
                                submit_one, validate_confirmed_history)
from weyl_parent_seal import load_parent_seal


STAGE_NAME = "ladder-boundary-before-v3"
PIN_NAME = "ladder-boundary-before-v3-pin.json"
PIN_SCHEMA = "atlas-ladder-boundary-before-pin-v2"
STAGE_LOCK = ".ladder-boundary-before-v3-stage.lock"
SUBMISSION_ENABLED = False
CHECKER_TESTS = 65
SBATCH = "hpc/math_ladder_boundary_before.sbatch"
TEST_HASHES = {
    "crates/atlas-real-group/src/root_system.rs":
        "109076ec05cf25e532fc7a951af0cbf5a725efefb318d35c2172a4528fd999f7",
    "crates/atlas-core/src/session.rs":
        "cef588d9afc7fcd513ee1660758b229a8767c252adacdec7feea9737675de6aa",
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
    "hpc/patches/ladder_boundary_tests.patch",
}

SUBMISSION_RECORD_KEYS = {
    "stage", "script", "queue_before", "status", "max_outstanding", "job",
    "pin_sha256",
}


def read(path, wanted=None):
    """Read one SHA-pinned JSON object through one stable file handle."""
    value = read_json_file(path, wanted)
    if not isinstance(value, dict):
        raise ValueError("JSON input must be an object")
    return value


def saved_json_sha(value):
    """Hash the exact canonical byte form written by progressive_submit.save."""
    raw = (json.dumps(value, indent=2, sort_keys=True) + "\n").encode("utf-8")
    return hashlib.sha256(raw).hexdigest()


def stage_inputs(root):
    """Compatibility name for the strict frozen-input verifier."""
    return frozen_stage_inputs(root)


def validate_partial_inputs(root, expected):
    current = stage_inputs(root)
    if any(name not in expected or expected[name] != sha
           for name, sha in current.items()):
        raise ValueError("unprepared ladder BEFORE v2 inputs are not an exact subset")
    return current


def seal_reference(sha, size):
    if (not isinstance(sha, str) or not re.fullmatch(r"[a-f0-9]{64}", sha)
            or not isinstance(size, str) or not re.fullmatch(r"0|[1-9][0-9]*", size)):
        raise ValueError("invalid path-free parent-seal reference")
    return dict(schema=BLOB_SCHEMA, role="weyl-parent-seal",
                sha256=sha, bytes=int(size))


def submission_receipt(record, pin):
    """Derive the durable child receipt from one shared pinned submission."""
    if (not isinstance(pin, dict)
            or not isinstance(record, dict)
            or set(record) != SUBMISSION_RECORD_KEYS
            or record.get("pin_sha256") != saved_json_sha(pin)):
        raise ValueError("ladder v2 receipt is not bound to its immutable pin")
    return dict(
        record,
        schema="atlas-ladder-boundary-before-submission-v2",
        status="SUBMITTED_NOT_VERIFIED",
        checker_tests=CHECKER_TESTS,
        core_inventory=630,
        domain_inventory=521,
        expected_failures=3,
        toolchain_commands=2,
        production_unchanged=True,
        parent_seal_object=pin["parent_seal_object"],
        storage_policy=(
            "One active-campaign child stage; parent evidence and source come only "
            "from campaign CAS. Expanded source and Cargo target are disposable."
        ),
        scope=(
            "Two root/coroot kernel regressions and one original-backed full-stream "
            "regression, tests-only BEFORE; no runtime or performance change."
        ),
    )


@contextmanager
def existing_lock(directory, name):
    """Open and lock one existing inode; recovery must never recreate it."""
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


def confirmed_record(root, pin, *, repair_intent=False):
    """Read one exact shared-ledger record without resubmitting its stage."""
    root = Path(root).resolve()
    pin_sha = saved_json_sha(pin)
    campaign = submission_scope(root)
    intent_path = root / "submission-intent.json"
    ledger_path = campaign / ".atlas-progressive-submit.json"
    with existing_lock(campaign, ".atlas-progressive-submit.lock"):
        history = read_json_file(ledger_path)
        validate_confirmed_history(history)
        matches = [row for row in history if row.get("stage") == str(root)]
        if len(matches) != 1:
            raise ValueError("ladder v2 campaign ledger lacks one unique stage")
        record = matches[0]
        queue = record.get("queue_before")
        if (set(record) != SUBMISSION_RECORD_KEYS
                or record.get("script") != SBATCH
                or record.get("status") != "SUBMITTED"
                or record.get("pin_sha256") != pin_sha
                or type(record.get("max_outstanding")) is not int
                or record["max_outstanding"] != 10
                or not isinstance(record.get("job"), str)
                or not record["job"].isdecimal()
                or not isinstance(queue, list)
                or queue_ids("\n".join(queue)) != queue
                or len(queue) >= 10):
            raise ValueError("ladder v2 confirmed submission record changed")
        intent = read(intent_path)
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
                raise ValueError("ladder v2 intent and campaign ledger disagree")
            save(intent_path, record)
        return record


def stage_mode(root, pin):
    """Classify a frozen attempt without ever issuing a second submission."""
    root = Path(root)
    receipt_path = root / "submission.json"
    if os.path.lexists(receipt_path):
        record = confirmed_record(root, pin)
        if read(receipt_path) != submission_receipt(record, pin):
            raise ValueError("ladder v2 submission receipt changed")
        return "complete"
    intent_path = root / "submission-intent.json"
    if os.path.lexists(intent_path):
        try:
            confirmed_record(root, pin, repair_intent=True)
        except ValueError as error:
            raise ValueError(
                "unresolved ladder v2 submission intent; reconcile this same stage"
            ) from error
        return "recover-receipt"
    if os.path.lexists(root / PIN_NAME):
        return "retry"
    return "prepare"


def requires_existing_stage_lock(root, campaign, *, prepared,
                                 has_submission_state, partial_inputs):
    """Fresh preparation may create one lock; every recovery reopens it."""
    root, campaign = Path(root), Path(campaign)
    stage_started = bool(
        prepared or has_submission_state or partial_inputs
        or any(os.path.lexists(root / name)
               for name in (STAGE_LOCK, ".incoming", "hpc", "tests", "results"))
    )
    if stage_started and not os.path.lexists(root / STAGE_LOCK):
        raise ValueError("recovering ladder v2 stage lacks its original lock")
    if not os.path.lexists(campaign / ".atlas-progressive-submit.lock"):
        raise ValueError("ladder v2 campaign lacks its existing submission lock")
    return stage_started


def _real_directory(path):
    try:
        value = Path(path).lstat()
    except FileNotFoundError as error:
        raise ValueError("required ladder v2 directory is missing") from error
    if not stat.S_ISDIR(value.st_mode):
        raise ValueError("ladder v2 path is not a real directory")


def _directory_flags():
    nofollow = getattr(os, "O_NOFOLLOW", None)
    directory = getattr(os, "O_DIRECTORY", None)
    if nofollow is None or directory is None:
        raise ValueError("safe ladder v2 directory traversal is unavailable")
    return os.O_RDONLY | getattr(os, "O_CLOEXEC", 0) | nofollow | directory


def _open_directory(path):
    try:
        descriptor = os.open(path, _directory_flags())
    except OSError as error:
        raise ValueError("ladder v2 directory could not be opened safely") from error
    if not stat.S_ISDIR(os.fstat(descriptor).st_mode):
        os.close(descriptor)
        raise ValueError("ladder v2 path is not a directory")
    return descriptor


def _open_child_directory(parent, name, create=False):
    if not isinstance(name, str) or name in ("", ".", "..") or "/" in name:
        raise ValueError("unsafe ladder v2 directory component")
    if create:
        try:
            os.mkdir(name, 0o755, dir_fd=parent)
            os.fsync(parent)
        except FileExistsError:
            pass
    try:
        descriptor = os.open(name, _directory_flags(), dir_fd=parent)
    except OSError as error:
        raise ValueError("ladder v2 child directory could not be opened safely") from error
    if not stat.S_ISDIR(os.fstat(descriptor).st_mode):
        os.close(descriptor)
        raise ValueError("ladder v2 child is not a directory")
    return descriptor


@contextmanager
def _open_incoming(incoming, create=False):
    incoming = Path(incoming).absolute()
    if incoming.name != ".incoming":
        raise ValueError("unexpected ladder v2 preparation scratch name")
    stage_descriptor = incoming_descriptor = None
    try:
        stage_descriptor = _open_directory(incoming.parent)
        incoming_descriptor = _open_child_directory(
            stage_descriptor, incoming.name, create=create)
        yield stage_descriptor, incoming_descriptor
    finally:
        if incoming_descriptor is not None:
            os.close(incoming_descriptor)
        if stage_descriptor is not None:
            os.close(stage_descriptor)


@contextmanager
def _safe_destination_parent(destination, incoming):
    """Hold every no-follow dirfd through the final destination publication."""
    destination, incoming = Path(destination).absolute(), Path(incoming).absolute()
    stage = incoming.parent
    try:
        relative = destination.relative_to(stage)
    except ValueError as error:
        raise ValueError("frozen ladder v2 destination escaped its stage") from error
    if (not relative.parts
            or any(part in ("", ".", "..") or "/" in part for part in relative.parts)):
        raise ValueError("frozen ladder v2 destination has an unsafe component")
    parent_descriptor = None
    with _open_incoming(incoming) as (stage_descriptor, incoming_descriptor):
        try:
            parent_descriptor = os.dup(stage_descriptor)
            for part in relative.parts[:-1]:
                child = _open_child_directory(parent_descriptor, part, create=True)
                os.close(parent_descriptor)
                parent_descriptor = child
            yield (stage_descriptor, incoming_descriptor, parent_descriptor,
                   relative.parts[-1])
        finally:
            if parent_descriptor is not None:
                os.close(parent_descriptor)


@contextmanager
def _safe_source_parent(source, stage, stage_descriptor):
    """Hold a no-follow source-parent dirfd rooted in the already-open stage."""
    source, stage = Path(source).absolute(), Path(stage).absolute()
    try:
        relative = source.relative_to(stage)
    except ValueError as error:
        raise ValueError("frozen ladder v2 source escaped its stage") from error
    if (not relative.parts
            or any(part in ("", ".", "..") or "/" in part for part in relative.parts)):
        raise ValueError("frozen ladder v2 source has an unsafe component")
    parent_descriptor = os.dup(stage_descriptor)
    try:
        for part in relative.parts[:-1]:
            child = _open_child_directory(parent_descriptor, part)
            os.close(parent_descriptor)
            parent_descriptor = child
        yield parent_descriptor, relative.parts[-1]
    finally:
        os.close(parent_descriptor)


def _regular_bytes_at(directory, name, required_mode=0o444, required_nlink=1):
    """Read one stable single-link regular child through a held directory fd."""
    descriptor = current_descriptor = None
    nofollow = getattr(os, "O_NOFOLLOW", None)
    if (nofollow is None or not isinstance(name, str) or name in ("", ".", "..")
            or "/" in name):
        raise ValueError("unsafe ladder v2 file name")
    try:
        descriptor = os.open(
            name, os.O_RDONLY | getattr(os, "O_CLOEXEC", 0) | nofollow,
            dir_fd=directory)
    except FileNotFoundError:
        raise
    except OSError as error:
        raise ValueError("ladder v2 file could not be opened safely") from error
    try:
        opened = os.fstat(descriptor)
        if (not stat.S_ISREG(opened.st_mode) or opened.st_nlink != required_nlink
                or (required_mode is not None
                    and stat.S_IMODE(opened.st_mode) != required_mode)):
            raise ValueError("ladder v2 child is not a single-link regular file")
        chunks = []
        while True:
            chunk = os.read(descriptor, 64 * 1024)
            if not chunk:
                break
            chunks.append(chunk)
        raw = b"".join(chunks)
        after = os.fstat(descriptor)
        current_descriptor = os.open(
            name, os.O_RDONLY | getattr(os, "O_CLOEXEC", 0) | nofollow,
            dir_fd=directory)
        current = os.fstat(current_descriptor)
        identity = lambda value: (value.st_dev, value.st_ino, value.st_size,
                                  value.st_mtime_ns, value.st_ctime_ns,
                                  stat.S_IFMT(value.st_mode), value.st_nlink)
        if (identity(after) != identity(opened) or identity(current) != identity(opened)
                or len(raw) != opened.st_size):
            raise ValueError("ladder v2 child changed while it was read")
        return raw
    finally:
        if current_descriptor is not None:
            os.close(current_descriptor)
        if descriptor is not None:
            os.close(descriptor)


def _directory_identity(value):
    return (value.st_dev, value.st_ino, stat.S_IFMT(value.st_mode),
            value.st_nlink, value.st_mtime_ns, value.st_ctime_ns)


def _frozen_directory_files(descriptor, prefix, result, interrupted=None):
    """Hash one stable no-follow tree of exact immutable regular files."""
    opened = os.fstat(descriptor)
    if not stat.S_ISDIR(opened.st_mode):
        raise ValueError("frozen ladder v2 input parent is not a directory")
    try:
        names = os.listdir(descriptor)
    except OSError as error:
        raise ValueError("frozen ladder v2 input directory could not be listed") from error
    if (len(names) != len(set(names))
            or any(not isinstance(name, str) or name in ("", ".", "..")
                   or "/" in name for name in names)):
        raise ValueError("frozen ladder v2 input directory has unsafe entries")
    for name in sorted(names):
        try:
            metadata = os.stat(name, dir_fd=descriptor, follow_symlinks=False)
        except OSError as error:
            raise ValueError("frozen ladder v2 input entry could not be inspected") from error
        relative = prefix + "/" + name
        if stat.S_ISREG(metadata.st_mode):
            if metadata.st_nlink == 1:
                raw = _regular_bytes_at(descriptor, name)
            elif interrupted is not None and metadata.st_nlink == 2:
                raw = _regular_bytes_at(
                    descriptor, name, required_mode=0o444, required_nlink=2)
                key = (metadata.st_dev, metadata.st_ino)
                row = interrupted.get(key)
                if row is None or row["destination"] is not None:
                    raise ValueError("frozen ladder v2 linked input is not uniquely paired")
                row["destination"] = relative
            else:
                raise ValueError("frozen ladder v2 input is not single-link")
            sha = hashlib.sha256(raw).hexdigest()
            if (metadata.st_nlink == 2
                    and sha != interrupted[(metadata.st_dev, metadata.st_ino)]["sha256"]):
                raise ValueError("frozen ladder v2 linked input bytes disagree")
            result[relative] = sha
        elif stat.S_ISDIR(metadata.st_mode):
            child = _open_child_directory(descriptor, name)
            try:
                child_identity = _directory_identity(os.fstat(child))
                if child_identity != _directory_identity(metadata):
                    raise ValueError("frozen ladder v2 input directory changed before traversal")
                _frozen_directory_files(child, relative, result, interrupted)
                reopened = _open_child_directory(descriptor, name)
                try:
                    if _directory_identity(os.fstat(reopened)) != child_identity:
                        raise ValueError("frozen ladder v2 input directory was replaced")
                finally:
                    os.close(reopened)
            finally:
                os.close(child)
        else:
            raise ValueError("frozen ladder v2 input is not a regular file or directory")
    try:
        names_after = os.listdir(descriptor)
    except OSError as error:
        raise ValueError("frozen ladder v2 input directory could not be rechecked") from error
    if (sorted(names_after) != sorted(names)
            or _directory_identity(os.fstat(descriptor)) != _directory_identity(opened)):
        raise ValueError("frozen ladder v2 input directory changed during traversal")


def _stage_input_manifest(root, interrupted=None):
    root = Path(root)
    result = {}
    for folder in ("hpc", "tests"):
        directory = root / folder
        if not os.path.lexists(directory):
            continue
        descriptor = _open_directory(directory)
        try:
            opened = _directory_identity(os.fstat(descriptor))
            _frozen_directory_files(descriptor, folder, result, interrupted)
            reopened = _open_directory(directory)
            try:
                if _directory_identity(os.fstat(reopened)) != opened:
                    raise ValueError("frozen ladder v2 input root was replaced")
            finally:
                os.close(reopened)
        finally:
            os.close(descriptor)
    return result


def frozen_stage_inputs(root):
    """Return a manifest only for stable 0444, single-link staged files."""
    return _stage_input_manifest(root)


def _regular_identity(value):
    return (value.st_dev, value.st_ino, stat.S_IFMT(value.st_mode),
            stat.S_IMODE(value.st_mode), value.st_nlink, value.st_size,
            value.st_mtime_ns, value.st_ctime_ns)


def interrupted_publications(root, overrides, *, locked_cleanup=False):
    """Recognize scratch; ``locked_cleanup`` is for the existing-lock pass only."""
    if type(locked_cleanup) is not bool or not isinstance(overrides, dict):
        raise ValueError("invalid interrupted ladder v2 recovery request")
    root = Path(root).absolute()
    incoming = root / ".incoming"
    if not os.path.lexists(incoming):
        return None
    with _open_incoming(incoming) as (_, incoming_descriptor):
        opened = os.fstat(incoming_descriptor)
        try:
            names = os.listdir(incoming_descriptor)
        except OSError as error:
            raise ValueError("interrupted ladder v2 scratch could not be listed") from error
        if len(names) != len(set(names)):
            raise ValueError("duplicate interrupted ladder v2 scratch entries")
        linked = {}
        for name in sorted(names):
            if not re.fullmatch(r"\.copy-[a-f0-9]{32}", name):
                raise ValueError("unexpected interrupted ladder v2 scratch entry")
            try:
                metadata = os.stat(
                    name, dir_fd=incoming_descriptor, follow_symlinks=False)
            except OSError as error:
                raise ValueError("interrupted ladder v2 scratch could not be inspected") from error
            mode = stat.S_IMODE(metadata.st_mode)
            if not stat.S_ISREG(metadata.st_mode):
                raise ValueError("interrupted ladder v2 scratch is not regular")
            if metadata.st_nlink == 1 and mode in (0o600, 0o444):
                published = False
                raw = _regular_bytes_at(
                    incoming_descriptor, name,
                    required_mode=mode, required_nlink=1)
            elif metadata.st_nlink == 2 and mode == 0o444:
                published = True
                raw = _regular_bytes_at(
                    incoming_descriptor, name,
                    required_mode=0o444, required_nlink=2)
            else:
                raise ValueError("interrupted ladder v2 scratch has unsafe mode or links")
            key = (metadata.st_dev, metadata.st_ino)
            if key in linked:
                raise ValueError("interrupted ladder v2 scratch inode is not unique")
            linked[key] = {
                "name": name,
                "sha256": hashlib.sha256(raw).hexdigest(),
                "identity": _regular_identity(metadata),
                "destination": None,
                "published": published,
            }
        if (sorted(os.listdir(incoming_descriptor)) != sorted(names)
                or _directory_identity(os.fstat(incoming_descriptor))
                != _directory_identity(opened)):
            raise ValueError("interrupted ladder v2 scratch changed while inspected")
        if not linked:
            return None

        manifest = _stage_input_manifest(root, linked)
        if (any(name not in overrides or overrides[name] != sha
                for name, sha in manifest.items())
                or any((row["published"]
                        and (row["destination"] is None
                             or overrides.get(row["destination"])
                             != row["sha256"]))
                       or (not row["published"]
                           and row["destination"] is not None)
                       for row in linked.values())):
            raise ValueError("interrupted ladder v2 publication is not exactly paired")
        recovery = {
            "inputs": manifest,
            "temporary_names": tuple(sorted(row["name"] for row in linked.values())),
        }
        if not locked_cleanup:
            return recovery

        # The caller may reach this branch only while holding the pre-existing
        # stage lock. Recheck every held scratch name before the first unlink.
        for row in linked.values():
            current = os.stat(
                row["name"], dir_fd=incoming_descriptor, follow_symlinks=False)
            if _regular_identity(current) != row["identity"]:
                raise ValueError("interrupted ladder v2 scratch changed before cleanup")
        for row in linked.values():
            os.unlink(row["name"], dir_fd=incoming_descriptor)
        os.fsync(incoming_descriptor)
        strict = frozen_stage_inputs(root)
        if strict != manifest:
            raise ValueError("interrupted ladder v2 cleanup did not restore frozen inputs")
        return recovery


def _new_temporary(incoming_descriptor):
    flags = (os.O_RDWR | os.O_CREAT | os.O_EXCL | getattr(os, "O_CLOEXEC", 0)
             | getattr(os, "O_NOFOLLOW", 0))
    for _ in range(128):
        name = ".copy-" + secrets.token_hex(16)
        try:
            return name, os.open(name, flags, 0o600, dir_fd=incoming_descriptor)
        except FileExistsError:
            continue
    raise ValueError("could not allocate ladder v2 preparation scratch")


def _write_all(descriptor, raw):
    remaining = memoryview(raw)
    while remaining:
        written = os.write(descriptor, remaining)
        if written <= 0:
            raise ValueError("short ladder v2 temporary write")
        remaining = remaining[written:]


def _atomic_install(raw, wanted, incoming_descriptor,
                    destination_descriptor, destination_name):
    """Publish exact bytes using only held source and destination dirfds."""
    try:
        existing = _regular_bytes_at(destination_descriptor, destination_name)
    except FileNotFoundError:
        existing = None
    if existing is not None:
        if hashlib.sha256(existing).hexdigest() != wanted:
            raise ValueError("prepared ladder v2 input changed")
        return

    temporary_name, descriptor = _new_temporary(incoming_descriptor)
    linked = False
    try:
        _write_all(descriptor, raw)
        os.fchmod(descriptor, 0o444)
        os.fsync(descriptor)
        temporary = os.fstat(descriptor)
        named = os.stat(
            temporary_name, dir_fd=incoming_descriptor, follow_symlinks=False)
        if (not stat.S_ISREG(temporary.st_mode) or temporary.st_nlink != 1
                or (temporary.st_dev, temporary.st_ino, temporary.st_size)
                != (named.st_dev, named.st_ino, named.st_size)
                or temporary.st_size != len(raw)):
            raise ValueError("temporary ladder v2 input changed")
        os.lseek(descriptor, 0, os.SEEK_SET)
        copied = b"".join(iter(lambda: os.read(descriptor, 64 * 1024), b""))
        if copied != raw or hashlib.sha256(copied).hexdigest() != wanted:
            raise ValueError("temporary ladder v2 bytes changed")
        try:
            os.link(
                temporary_name,
                destination_name,
                src_dir_fd=incoming_descriptor,
                dst_dir_fd=destination_descriptor,
                follow_symlinks=False,
            )
            linked = True
        except FileExistsError:
            appeared = _regular_bytes_at(destination_descriptor, destination_name)
            if hashlib.sha256(appeared).hexdigest() != wanted:
                raise ValueError("ladder v2 input appeared with different bytes")
        if linked:
            published = os.stat(
                destination_name,
                dir_fd=destination_descriptor,
                follow_symlinks=False,
            )
            temporary = os.fstat(descriptor)
            if ((published.st_dev, published.st_ino, published.st_size)
                    != (temporary.st_dev, temporary.st_ino, temporary.st_size)
                    or published.st_nlink != 2):
                os.unlink(destination_name, dir_fd=destination_descriptor)
                os.fsync(destination_descriptor)
                linked = False
                raise ValueError("published ladder v2 input changed")
            os.fsync(destination_descriptor)
    finally:
        os.close(descriptor)
        try:
            os.unlink(temporary_name, dir_fd=incoming_descriptor)
        except FileNotFoundError:
            pass
        os.fsync(incoming_descriptor)
    final = _regular_bytes_at(destination_descriptor, destination_name)
    if hashlib.sha256(final).hexdigest() != wanted:
        raise ValueError("installed ladder v2 input changed")


def frozen_copy(source, destination, wanted, incoming):
    """Atomically install exact bytes without resolving a checked parent again."""
    stage = Path(incoming).absolute().parent
    with _safe_destination_parent(destination, incoming) as descriptors:
        stage_descriptor, incoming_descriptor, destination_descriptor, name = descriptors
        with _safe_source_parent(source, stage, stage_descriptor) as source_descriptor:
            raw = _regular_bytes_at(*source_descriptor, required_mode=None)
        if hashlib.sha256(raw).hexdigest() != wanted:
            raise ValueError("changed ladder v2 override input")
        _atomic_install(
            raw, wanted, incoming_descriptor, destination_descriptor, name)


def clean_incoming(incoming):
    """Require empty scratch; interrupted links use the locked recognizer."""
    with _open_incoming(incoming, create=True) as (_, incoming_descriptor):
        if os.listdir(incoming_descriptor):
            raise ValueError(
                "ladder v2 preparation scratch requires locked recovery")
        os.fsync(incoming_descriptor)


def install_overrides(root, overrides):
    root = Path(root)
    incoming = root / ".incoming"
    clean_incoming(incoming)
    for name, wanted in overrides.items():
        frozen_copy(root / "overrides" / name, root / name, wanted, incoming)


def prepared_pin(parent_seal_object, overrides):
    return dict(schema=PIN_SCHEMA, inputs=overrides,
                parent_seal_object=parent_seal_object,
                ladder_boundary_test_hashes=TEST_HASHES)


def submit_pinned(root, pin_sha):
    """Delegate submission to the sole shared, pin-aware campaign boundary."""
    if not SUBMISSION_ENABLED:
        raise ValueError("seal-only ladder BEFORE v2 remains disabled")
    return submit_one(
        root,
        SBATCH,
        dict(os.environ, LADDER_BOUNDARY_BEFORE_PIN_SHA256=pin_sha),
        pin_sha256=pin_sha,
    )


def _stage_and_submit(root, parent_seal_object, overrides_sha, validate_only=False,
                      existing_stage_lock=False):
    """Validate fully; mutate only on the second, stage-locked invocation."""
    if not SUBMISSION_ENABLED:
        raise ValueError("seal-only ladder BEFORE v2 remains disabled")
    if type(validate_only) is not bool or type(existing_stage_lock) is not bool:
        raise ValueError("invalid ladder v2 staging lock policy")
    root = Path(root).resolve()
    campaign = campaign_stage(root)
    if root.name != STAGE_NAME:
        raise ValueError("unexpected ladder BEFORE v2 stage")
    _real_directory(root)
    overrides_root = root / "overrides"
    _real_directory(overrides_root)
    overrides_path = overrides_root / "overrides.json"
    overrides = read(overrides_path, overrides_sha)
    if set(overrides) != STAGE_INPUT_NAMES:
        raise ValueError("unexpected ladder BEFORE v2 overrides")
    expected_override_files = dict(overrides)
    expected_override_files["overrides.json"] = overrides_sha
    if file_manifest(overrides_root) != expected_override_files:
        raise ValueError("ladder BEFORE v2 override tree changed")
    if not os.path.lexists(campaign / ".atlas-progressive-submit.lock"):
        raise ValueError("ladder v2 campaign submission lock is missing")

    # This is the only parent dependency. Full archival verification reads
    # campaign CAS objects, never any historical atlas-* directory.
    seal = load_parent_seal(campaign, parent_seal_object, verify_archival=True)
    if any(seal["migration_inputs"].get(name) != overrides[name]
           for name in SEALED_SHARED_INPUTS):
        raise ValueError("shared seal validator inputs differ from the accepted migration")
    pin = prepared_pin(parent_seal_object, overrides)
    pin_sha = saved_json_sha(pin)
    interrupted = interrupted_publications(root, overrides)
    interrupted_inputs = interrupted["inputs"] if interrupted is not None else None
    prepared = os.path.lexists(root / PIN_NAME)
    has_submission_state = (
        os.path.lexists(root / "submission-intent.json")
        or os.path.lexists(root / "submission.json")
    )
    if has_submission_state and not prepared:
        raise ValueError("ladder v2 submission state lacks its immutable pin")
    partial_inputs = {}
    if prepared:
        current_inputs = (interrupted_inputs if interrupted is not None
                          else stage_inputs(root))
        if (read(root / PIN_NAME, pin_sha) != pin
                or current_inputs != overrides):
            raise ValueError("prepared ladder BEFORE v2 stage changed")
    else:
        partial_inputs = (interrupted_inputs if interrupted is not None
                          else validate_partial_inputs(root, overrides))
    require_existing_lock = requires_existing_stage_lock(
        root,
        campaign,
        prepared=prepared,
        has_submission_state=has_submission_state,
        partial_inputs=partial_inputs,
    )
    if validate_only:
        return require_existing_lock

    if interrupted is not None:
        if not existing_stage_lock or not require_existing_lock:
            raise ValueError("interrupted ladder v2 publication lacks its existing lock")
        if interrupted_publications(
                root, overrides, locked_cleanup=True) != interrupted:
            raise ValueError("interrupted ladder v2 publication changed before cleanup")

    mode = stage_mode(root, pin)

    if mode in ("prepare", "retry"):
        install_overrides(root, overrides)
        if stage_inputs(root) != overrides:
            raise ValueError("installed ladder BEFORE v2 inputs changed")
        if mode == "prepare":
            save(root / PIN_NAME, pin)
        elif read(root / PIN_NAME, pin_sha) != pin:
            raise ValueError("prepared ladder BEFORE v2 pin changed before retry")

    if read(root / PIN_NAME, pin_sha) != pin:
        raise ValueError("ladder BEFORE v2 pin changed before receipt or submission")
    current_inputs = stage_inputs(root)
    if current_inputs != pin["inputs"] or current_inputs[SBATCH] != overrides[SBATCH]:
        raise ValueError("ladder BEFORE v2 inputs changed before receipt or submission")
    if mode == "complete":
        receipt = read(root / "submission.json")
        print(json.dumps(receipt, sort_keys=True), flush=True)
        return receipt
    if mode == "recover-receipt":
        record = confirmed_record(root, pin)
    else:
        record = submit_pinned(root, pin_sha)
    receipt = submission_receipt(record, pin)
    save(root / "submission.json", receipt)
    print(json.dumps(receipt, sort_keys=True), flush=True)
    return receipt


def run_enabled(root, parent_seal_object, overrides_sha):
    """Repeat the entire read-only preflight under one stage-exclusive lock."""
    if not SUBMISSION_ENABLED:
        raise ValueError("seal-only ladder BEFORE v2 remains disabled")
    require_existing_lock = _stage_and_submit(
        root, parent_seal_object, overrides_sha, validate_only=True)
    locker = existing_lock if require_existing_lock else exclusive_lock
    with locker(root, STAGE_LOCK):
        return _stage_and_submit(
            root, parent_seal_object, overrides_sha,
            existing_stage_lock=require_existing_lock)


def main():
    # The independent feature guard remains first so an emergency disable fails
    # before parsing paths, reading CAS, staging bytes, querying SLURM, or
    # creating a lock/intent.
    if not SUBMISSION_ENABLED:
        raise SystemExit("seal-only ladder BEFORE v2 is disabled")
    if len(sys.argv) != 5:
        raise SystemExit("usage: stage ROOT SEAL_SHA256 SEAL_BYTES OVERRIDES_SHA256")
    root = Path(sys.argv[1]).resolve()
    run_enabled(root, seal_reference(sys.argv[2], sys.argv[3]), sys.argv[4])


if __name__ == "__main__":
    main()

"""Stage one resumable archival parent-seal migration below one campaign."""
from contextlib import contextmanager
import fcntl
import json
import os
from pathlib import Path
import re
import shutil
import stat
import sys
import tempfile

sys.dont_write_bytecode = True
from campaign_blob import store_blob, verify_blob
from campaign_source import (digest as safe_digest, file_manifest, store_source_archive,
                             verify_source_archive)
from campaign_workspace import (_real_directory, campaign_stage,
                                campaign_storage_root, submission_scope)
from progressive_submit import (exclusive_lock, queue_ids, read_json_file, save,
                                submit_one, validate_confirmed_history)
from weyl_parent_seal import (MIGRATION_OVERRIDE_NAMES, PIN_NAME, SBATCH,
                              STAGE_LOCK, STAGE_NAME, submission_receipt)


def digest(path):
    return safe_digest(path)


def read(path, sha=None):
    return read_json_file(path, sha)


def inputs(root):
    root = Path(root)
    result = {}
    for folder in ("hpc", "tests"):
        directory = root / folder
        if not os.path.lexists(directory):
            continue
        result.update({folder + "/" + name: sha
                       for name, sha in file_manifest(directory).items()})
    return result


def validate_partial_inputs(root, expected):
    current = inputs(root)
    if any(name not in expected or expected[name] != sha
           for name, sha in current.items()):
        raise ValueError("unprepared parent-seal inputs are not an exact safe subset")
    return current


@contextmanager
def existing_lock(directory, name):
    """Lock an existing inode without ever creating a second lock domain."""
    if (not isinstance(name, str) or not name or "/" in name
            or name in (".", "..") or name.startswith("-")):
        raise ValueError("unsafe existing lock name")
    nofollow = getattr(os, "O_NOFOLLOW", None)
    directory_flag = getattr(os, "O_DIRECTORY", None)
    if nofollow is None or directory_flag is None:
        raise ValueError("safe existing lock-file traversal is unavailable")
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
        raise ValueError("existing lock path could not be opened safely") from error
    try:
        opened = os.fstat(descriptor)
        if (not stat.S_ISREG(opened.st_mode) or opened.st_nlink != 1
                or stat.S_IMODE(opened.st_mode) & 0o022):
            raise ValueError("existing lock is not a private single-link regular file")
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
            raise ValueError("existing lock path could not be acquired safely") from error
        identity = lambda value: (value.st_dev, value.st_ino, value.st_mode,
                                  value.st_nlink)
        if identity(current) != identity(opened):
            raise ValueError("existing lock path changed while it was acquired")
        yield descriptor
    finally:
        os.close(descriptor)
        os.close(directory_descriptor)


def confirmed_record(root, pin_sha, repair_intent=False):
    root = Path(root).resolve()
    if (not isinstance(pin_sha, str)
            or re.fullmatch(r"[a-f0-9]{64}", pin_sha) is None):
        raise ValueError("invalid current parent-seal pin digest")
    intent_path = root / "submission-intent.json"
    scope = submission_scope(root)
    ledger_path = scope / ".atlas-progressive-submit.json"
    with existing_lock(scope, ".atlas-progressive-submit.lock"):
        ledger = read(ledger_path) if os.path.lexists(ledger_path) else []
        validate_confirmed_history(ledger)
        expected_keys = {
            "stage", "script", "queue_before", "status", "max_outstanding", "job",
            "pin_sha256",
        }
        matches = [row for row in ledger if row.get("stage") == str(root)]
        if len(matches) != 1:
            raise ValueError("parent-seal campaign ledger has no unique stage record")
        record = matches[0]
        queue = record.get("queue_before")
        if (set(record) != expected_keys or record.get("script") != SBATCH
                or record.get("status") != "SUBMITTED"
                or type(record.get("max_outstanding")) is not int
                or record["max_outstanding"] != 10
                or not isinstance(record.get("job"), str) or not record["job"].isdecimal()
                or record.get("pin_sha256") != pin_sha
                or not isinstance(queue, list)
                or not all(isinstance(item, str) for item in queue)
                or queue_ids("\n".join(queue)) != queue or len(queue) >= 10):
            raise ValueError("invalid confirmed parent-seal campaign record")
        intent = read(intent_path)
        if intent != record:
            uncertain_keys = expected_keys - {"job"}
            same_attempt = (isinstance(intent, dict) and set(intent) == uncertain_keys
                            and intent.get("status") == "SUBMISSION_INTENT_NOT_CONFIRMED"
                            and all(intent.get(key) == record.get(key)
                                    for key in uncertain_keys - {"status"}))
            if not repair_intent or not same_attempt:
                raise ValueError("parent-seal intent and campaign ledger disagree")
            save(intent_path, record)
        return record


def stage_mode(root):
    root = Path(root)
    if os.path.lexists(root / "submission.json"):
        read(root / "submission.json")
        return "complete"
    intent = root / "submission-intent.json"
    if os.path.lexists(intent):
        try:
            confirmed_record(root, digest(root / PIN_NAME), repair_intent=True)
        except ValueError as error:
            raise ValueError("unresolved parent-seal submission intent; reconcile this same stage") from error
        return "recover-receipt"
    pin = root / PIN_NAME
    if os.path.lexists(pin):
        read(pin)
        return "retry"
    return "prepare"


def requires_existing_stage_lock(root, campaign, *, prepared,
                                 has_submission_state, partial_inputs):
    """Classify fresh preparation versus recovery without creating a lock."""
    root, campaign = Path(root), Path(campaign)
    stage_started = bool(prepared or has_submission_state or partial_inputs
                         or any(os.path.lexists(root / name)
                                for name in (STAGE_LOCK, ".incoming", "hpc",
                                             "tests", "results")))
    if stage_started and not os.path.lexists(root / STAGE_LOCK):
        raise ValueError("recovering parent-seal stage lacks its original lock")
    if (has_submission_state
            and not os.path.lexists(campaign / ".atlas-progressive-submit.lock")):
        raise ValueError("recovering parent-seal submission lacks its original campaign lock")
    return stage_started


def _fsync_directory(path):
    descriptor = os.open(path, os.O_RDONLY | getattr(os, "O_DIRECTORY", 0))
    try:
        os.fsync(descriptor)
    finally:
        os.close(descriptor)


def _regular(path):
    try:
        value = Path(path).lstat()
    except FileNotFoundError:
        return False
    if not stat.S_ISREG(value.st_mode):
        raise ValueError("frozen stage input is not a regular file")
    return True


def _frozen_destination(path, wanted):
    """Require one stable read-only inode for an already-installed input."""
    path = Path(path)
    try:
        before = path.lstat()
        if (not stat.S_ISREG(before.st_mode) or before.st_nlink != 1
                or stat.S_IMODE(before.st_mode) != 0o444):
            raise ValueError("frozen stage input is not an immutable single-link file")
        if digest(path) != wanted:
            raise ValueError("prepared parent-seal input changed")
        after = path.lstat()
    except OSError as error:
        raise ValueError("frozen stage input could not be inspected safely") from error
    identity = lambda value: (value.st_dev, value.st_ino, value.st_mode,
                              value.st_nlink, value.st_size,
                              value.st_mtime_ns, value.st_ctime_ns)
    if identity(after) != identity(before):
        raise ValueError("frozen stage input changed while it was inspected")


def _safe_destination_parent(destination, incoming):
    """Create only real directories below the exact stage owning incoming."""
    destination, incoming = Path(destination), Path(incoming)
    if (any(part in ("", ".", "..") for part in destination.parts)
            or any(part in ("", ".", "..") for part in incoming.parts)):
        raise ValueError("frozen destination contains an unsafe path component")
    destination, incoming = destination.absolute(), incoming.absolute()
    stage = incoming.parent
    _real_directory(stage)
    try:
        relative = destination.relative_to(stage)
    except ValueError as error:
        raise ValueError("frozen destination escaped its stage") from error
    current = stage
    for part in relative.parent.parts:
        current = current / part
        try:
            current.mkdir()
        except FileExistsError:
            pass
        _real_directory(current)


def frozen_copy(source, destination, wanted, incoming):
    """Install one immutable byte sequence; completed files are reusable."""
    source, destination = Path(source), Path(destination)
    incoming = Path(incoming)
    _safe_destination_parent(destination, incoming)
    _real_directory(incoming)
    if not _regular(source) or digest(source) != wanted:
        raise ValueError("changed parent-seal source input")
    if os.path.lexists(destination):
        _frozen_destination(destination, wanted)
        return
    descriptor, temporary_name = tempfile.mkstemp(prefix=".copy-", dir=incoming)
    temporary = Path(temporary_name)
    try:
        with source.open("rb") as input_handle, os.fdopen(descriptor, "wb") as output:
            descriptor = None
            shutil.copyfileobj(input_handle, output)
            output.flush()
            os.fchmod(output.fileno(), 0o444)
            os.fsync(output.fileno())
        if digest(source) != wanted or digest(temporary) != wanted:
            raise ValueError("stage input changed while it was copied")
        try:
            os.link(temporary, destination, follow_symlinks=False)
            _fsync_directory(destination.parent)
        except FileExistsError:
            _frozen_destination(destination, wanted)
    finally:
        if descriptor is not None:
            os.close(descriptor)
        temporary.unlink(missing_ok=True)
    _frozen_destination(destination, wanted)


def install_manifest(manifest, resolver, destination, incoming):
    for name, wanted in manifest.items():
        frozen_copy(resolver(name), Path(destination) / name, wanted, incoming)


def clean_incoming(incoming):
    incoming = Path(incoming)
    try:
        metadata = incoming.lstat()
    except FileNotFoundError:
        incoming.mkdir(mode=0o700)
        metadata = incoming.lstat()
    if not stat.S_ISDIR(metadata.st_mode):
        raise ValueError("parent-seal preparation scratch is not a real directory")
    for path in incoming.iterdir():
        if not path.name.startswith(".copy-") or not _regular(path):
            raise ValueError("unexpected parent-seal preparation scratch")
        path.unlink()
    _fsync_directory(incoming)


def archive_legacy_harness(campaign, parent, legacy_inputs, old_pin_path):
    """Store the frozen harness as opaque CAS objects, never runnable files."""
    campaign = campaign_storage_root(campaign)
    with tempfile.TemporaryDirectory(prefix=".parent-seal-legacy-") as directory:
        snapshot = Path(directory)
        incoming = snapshot / ".incoming"
        clean_incoming(incoming)
        install_manifest(legacy_inputs, lambda name: parent / name,
                         snapshot, incoming)
        if file_manifest(snapshot) != legacy_inputs:
            raise ValueError("temporary legacy harness snapshot changed")
        source_object = store_source_archive(campaign, snapshot, legacy_inputs)
    pin_object = store_blob(campaign, old_pin_path, "legacy-harness-pin")
    verify_source_archive(campaign, source_object, legacy_inputs)
    verify_blob(campaign, pin_object)
    if pin_object["sha256"] != digest(old_pin_path):
        raise ValueError("legacy harness pin changed while it was archived")
    return source_object, pin_object


def _main(validate_only=False):
    root, capture_ref = (Path(value).resolve() for value in sys.argv[1:3])
    capture_sha = sys.argv[3]
    overrides_sha = sys.argv[4]
    campaign = campaign_stage(root)
    overrides_root = root / "overrides"
    for directory in (overrides_root, overrides_root / "hpc"):
        try:
            metadata = directory.lstat()
        except FileNotFoundError as error:
            raise ValueError("parent-seal override directory is missing") from error
        if not stat.S_ISDIR(metadata.st_mode):
            raise ValueError("parent-seal override directory is not a real directory")
    overrides_manifest = overrides_root / "overrides.json"
    if (root.name != STAGE_NAME
            or not re.fullmatch(r"[a-f0-9]{64}", capture_sha)
            or not re.fullmatch(r"[a-f0-9]{64}", overrides_sha)
            or not _regular(overrides_manifest)
            or digest(overrides_manifest) != overrides_sha):
        raise ValueError("unexpected parent-seal stage")
    parent = capture_ref.parents[2]
    if (parent.parent != Path("/public/home/majj")
            or not parent.name.startswith("atlas-weyl-context-capture-20260930.")
            or capture_ref.name != "report.json" or capture_ref.parent.parent.name != "results"
            or not capture_ref.parent.name.isdecimal()):
        raise ValueError("unexpected Weyl capture parent")
    old_pin_path = parent / "weyl-context-capture-pin.json"
    if not _regular(old_pin_path) or not _regular(capture_ref):
        raise ValueError("legacy capture inputs are not regular files")
    old = read(old_pin_path)
    capture = read(capture_ref, capture_sha)
    if (inputs(parent) != old["inputs"] or capture.get("pin") != old
            or capture.get("status") != "WEYL_CONTEXT_DISCOVERY_CAPTURED"
            or capture.get("integrity_rechecked") is not True
            or capture.get("production_unchanged") is not True):
        raise ValueError("unverified Weyl capture parent")
    overrides = read(overrides_manifest)
    names = MIGRATION_OVERRIDE_NAMES
    if (not isinstance(overrides, dict) or set(overrides) != names
            or file_manifest(overrides_root) != {**overrides, "overrides.json": overrides_sha}):
        raise ValueError("unexpected parent-seal overrides")
    if any(not _regular(overrides_root / name)
           or digest(overrides_root / name) != wanted for name, wanted in overrides.items()):
        raise ValueError("parent-seal override input changed")
    excluded_stagers = {name: sha for name, sha in old["inputs"].items()
                        if name.startswith("hpc/stage_")}
    expected = overrides
    pin_base = dict(schema="atlas-weyl-parent-seal-migration-pin-v1", inputs=expected,
                    legacy_inputs=old["inputs"],
                    excluded_legacy_stagers=excluded_stagers,
                    overrides_manifest_sha256=overrides_sha,
                    capture_reference=dict(path=str(capture_ref), sha256=capture_sha))
    prepared = os.path.lexists(root / PIN_NAME)
    has_submission_state = (os.path.lexists(root / "submission-intent.json")
                            or os.path.lexists(root / "submission.json"))
    if os.path.lexists(root / "legacy"):
        raise ValueError("expanded legacy harness is forbidden in the durable stage")
    if has_submission_state and not prepared:
        raise ValueError("parent-seal submission state lacks its immutable pin")
    pin = None
    partial_inputs = {}
    if prepared:
        pin = read(root / PIN_NAME)
        fixed = {key: value for key, value in pin.items()
                 if key not in {"legacy_source_object", "legacy_pin_object"}}
        if (set(pin) != set(pin_base) | {"legacy_source_object", "legacy_pin_object"}
                or fixed != pin_base or inputs(root) != expected):
            raise ValueError("prepared parent-seal stage changed")
        for name, wanted in expected.items():
            _frozen_destination(root / name, wanted)
        verify_source_archive(campaign, pin["legacy_source_object"], old["inputs"])
        verify_blob(campaign, pin["legacy_pin_object"])
        if (pin["legacy_pin_object"].get("role") != "legacy-harness-pin"
                or pin["legacy_pin_object"]["sha256"] != digest(old_pin_path)):
            raise ValueError("prepared legacy harness pin changed")
    else:
        partial_inputs = validate_partial_inputs(root, expected)
    if validate_only:
        return requires_existing_stage_lock(
            root, campaign, prepared=prepared,
            has_submission_state=has_submission_state,
            partial_inputs=partial_inputs)
    if not prepared:
        legacy_source_object, legacy_pin_object = archive_legacy_harness(
            campaign, parent, old["inputs"], old_pin_path)
        pin = dict(pin_base, legacy_source_object=legacy_source_object,
                   legacy_pin_object=legacy_pin_object)
    # Parent evidence and the complete immutable input map must be valid before
    # stage_mode is allowed to repair an intent or perform any other stage write.
    mode = stage_mode(root)

    if mode in ("prepare", "retry"):
        incoming = root / ".incoming"
        clean_incoming(incoming)
        install_manifest(expected, lambda name: root / "overrides" / name,
                         root, incoming)
        if inputs(root) != expected or os.path.lexists(root / "legacy"):
            raise ValueError("parent-seal stage manifest changed")
        if mode == "prepare":
            save(root / PIN_NAME, pin)
        elif read(root / PIN_NAME) != pin:
            raise ValueError("prepared parent-seal pin changed before retry")
    pin_sha = digest(root / PIN_NAME)
    if mode == "complete":
        existing = read(root / "submission.json")
        if existing != submission_receipt(confirmed_record(root, pin_sha), pin_sha):
            raise ValueError("parent-seal submission receipt changed")
        print(json.dumps(existing), flush=True)
        return
    if mode == "recover-receipt":
        record = confirmed_record(root, pin_sha)
    else:
        record = submit_one(root, SBATCH,
                            dict(os.environ, WEYL_PARENT_SEAL_PIN_SHA256=pin_sha),
                            pin_sha256=pin_sha)
    final = submission_receipt(record, pin_sha)
    save(root / "submission.json", final)
    print(json.dumps(final), flush=True)


def main():
    raise SystemExit("parent seal is closed")
    root = Path(sys.argv[1]).resolve()
    if root.name != STAGE_NAME:
        raise ValueError("unexpected parent-seal stage")
    campaign_stage(root)
    # Complete read-only validation precedes lock selection.  Fresh preparation
    # may create one lock; every recovery reopens the already-pinned inode.
    # The same validation repeats under that lock before any repair or write.
    require_existing_lock = _main(validate_only=True)
    locker = existing_lock if require_existing_lock else exclusive_lock
    with locker(root, STAGE_LOCK):
        return _main()


if __name__ == "__main__":
    main()

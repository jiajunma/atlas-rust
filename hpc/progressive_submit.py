"""Campaign-only serialized one-job submission; arrays/dependencies forbidden.

This guards new progressive launchers. It does not make old bulk launchers safe.
Counting ALL the user's queued tasks is deliberately stricter than Atlas-only.
An uncertain sbatch response leaves a durable intent requiring reconciliation.
Legacy top-level stages are rejected before the scheduler is queried.
"""
from contextlib import contextmanager
import fcntl
import hashlib
import json
import os
from pathlib import Path
import re
import stat
import subprocess
import tempfile

from campaign_workspace import submission_scope


def regular_bytes(path):
    """Read one stable, single-link regular file without following its name."""
    nofollow = getattr(os, "O_NOFOLLOW", None)
    if nofollow is None:
        raise ValueError("safe state-file reads are unavailable")
    descriptor = current_descriptor = None
    try:
        descriptor = os.open(path, os.O_RDONLY | getattr(os, "O_CLOEXEC", 0) | nofollow)
        opened = os.fstat(descriptor)
        if not stat.S_ISREG(opened.st_mode) or opened.st_nlink != 1:
            raise ValueError("state path is not a single-link regular file")
        chunks = []
        while True:
            chunk = os.read(descriptor, 64 * 1024)
            if not chunk:
                break
            chunks.append(chunk)
        raw = b"".join(chunks)
        after = os.fstat(descriptor)
        current_descriptor = os.open(
            path, os.O_RDONLY | getattr(os, "O_CLOEXEC", 0) | nofollow)
        current = os.fstat(current_descriptor)
        identity = lambda value: (value.st_dev, value.st_ino, value.st_size,
                                  value.st_mtime_ns, value.st_ctime_ns,
                                  stat.S_IFMT(value.st_mode), value.st_nlink)
        if (identity(after) != identity(opened) or identity(current) != identity(opened)
                or len(raw) != opened.st_size):
            raise ValueError("state path changed while it was read")
        return raw
    except OSError as error:
        raise ValueError("state path could not be read safely") from error
    finally:
        if current_descriptor is not None:
            os.close(current_descriptor)
        if descriptor is not None:
            os.close(descriptor)


def read_json_file(path, sha256=None):
    """Decode a safely-read JSON state file and optionally pin its exact bytes."""
    raw = regular_bytes(path)
    if sha256 is not None and hashlib.sha256(raw).hexdigest() != sha256:
        raise ValueError("changed JSON state input: " + str(path))
    try:
        return json.loads(raw)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise ValueError("invalid JSON state input: " + str(path)) from error


@contextmanager
def exclusive_lock(directory, name):
    """Lock one real single-link file below one already-validated directory."""
    if (not isinstance(name, str) or not name or "/" in name
            or name in (".", "..") or name.startswith("-")):
        raise ValueError("unsafe lock name")
    nofollow = getattr(os, "O_NOFOLLOW", None)
    directory_flag = getattr(os, "O_DIRECTORY", None)
    if nofollow is None or directory_flag is None:
        raise ValueError("safe lock-file traversal is unavailable")
    directory_descriptor = descriptor = None
    try:
        directory_descriptor = os.open(
            directory, os.O_RDONLY | directory_flag | nofollow)
        descriptor = os.open(
            name, os.O_RDWR | os.O_CREAT | getattr(os, "O_CLOEXEC", 0) | nofollow,
            0o600, dir_fd=directory_descriptor)
    except OSError as error:
        if descriptor is not None:
            os.close(descriptor)
        if directory_descriptor is not None:
            os.close(directory_descriptor)
        raise ValueError("lock path could not be opened safely") from error
    try:
        opened = os.fstat(descriptor)
        if (not stat.S_ISREG(opened.st_mode) or opened.st_nlink != 1
                or stat.S_IMODE(opened.st_mode) & 0o022):
            raise ValueError("lock is not a private single-link regular file")
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
            raise ValueError("lock path could not be acquired safely") from error
        identity = lambda value: (value.st_dev, value.st_ino, stat.S_IFMT(value.st_mode),
                                  value.st_nlink)
        if identity(current) != identity(opened):
            raise ValueError("lock path changed while it was acquired")
        yield descriptor
    finally:
        os.close(descriptor)
        os.close(directory_descriptor)


def queue_ids(raw):
    ids = raw.splitlines()
    if any(not re.fullmatch(r"[0-9]+(?:_[0-9]+)?", item) for item in ids):
        raise ValueError("queue is not an expanded, unambiguous task inventory")
    if len(ids) != len(set(ids)):
        raise ValueError("duplicate queue tasks")
    return ids


def capacity(raw):
    ids = queue_ids(raw)
    if len(ids) >= 10:
        raise ValueError("10-job cap reached; no submission")
    return ids


def validate_confirmed_history(history):
    """Require one exact, fully confirmed, collision-free campaign ledger."""
    legacy_keys = {
        "stage", "script", "queue_before", "status", "max_outstanding", "job",
    }
    pinned_keys = legacy_keys | {"pin_sha256"}
    if not isinstance(history, list):
        raise ValueError("unresolved submission intent; inspect scheduler before retry")
    for row in history:
        queue = row.get("queue_before") if isinstance(row, dict) else None
        keys = set(row) if isinstance(row, dict) else set()
        if (not isinstance(row, dict) or keys not in (legacy_keys, pinned_keys)
                or not isinstance(row.get("stage"), str) or not row["stage"]
                or not isinstance(row.get("script"), str) or not row["script"]
                or row.get("status") != "SUBMITTED"
                or type(row.get("max_outstanding")) is not int
                or row["max_outstanding"] != 10
                or not isinstance(row.get("job"), str) or not row["job"].isdecimal()
                or not isinstance(queue, list)
                or any(not isinstance(item, str) for item in queue)
                or queue_ids("\n".join(queue)) != queue
                or len(queue) >= 10
                or any(item.partition("_")[0] == row["job"] for item in queue)
                or ("pin_sha256" in row
                    and (not isinstance(row["pin_sha256"], str)
                         or re.fullmatch(r"[a-f0-9]{64}", row["pin_sha256"]) is None))):
            raise ValueError("unresolved submission intent; inspect scheduler before retry")
    if (len({row["stage"] for row in history}) != len(history)
            or len({row["job"] for row in history}) != len(history)):
        raise ValueError("unresolved submission intent; inspect scheduler before retry")
    return history


def one_job_script(script):
    for line in script.splitlines():
        if line.lstrip().startswith("#SBATCH") and re.search(r"--(?:array|dependency)|(?:^|\s)-[ad](?:\s|=|[0-9])", line):
            raise ValueError("arrays and speculative dependency chains are forbidden")
        if not line.lstrip().startswith("#") and re.search(r"\bsbatch\b", line):
            raise ValueError("nested scheduler submission is forbidden")


def stage_script(root, script):
    requested = Path(script)
    if (requested.is_absolute() or not requested.parts or str(requested).startswith("-")
            or any(part in ("", ".", "..") for part in requested.parts)):
        raise ValueError("job script must be a safe stage-relative path")
    nofollow = getattr(os, "O_NOFOLLOW", None)
    directory_flag = getattr(os, "O_DIRECTORY", None)
    if nofollow is None or directory_flag is None:
        raise ValueError("safe stage-relative script traversal is unavailable")
    directory_descriptors = []
    descriptor = None
    try:
        current = os.open(root, os.O_RDONLY | directory_flag | nofollow)
        directory_descriptors.append(current)
        for part in requested.parts[:-1]:
            current = os.open(part, os.O_RDONLY | directory_flag | nofollow, dir_fd=current)
            directory_descriptors.append(current)
        descriptor = os.open(requested.parts[-1], os.O_RDONLY | nofollow, dir_fd=current)
        opened = os.fstat(descriptor)
        if not stat.S_ISREG(opened.st_mode):
            raise ValueError("job script must be a regular file")
        identity = lambda value: (value.st_dev, value.st_ino, value.st_mode,
                                  value.st_size, value.st_mtime_ns)
        chunks = []
        while True:
            chunk = os.read(descriptor, 64 * 1024)
            if not chunk:
                break
            chunks.append(chunk)
        raw = b"".join(chunks)
        after = os.fstat(descriptor)
        final_descriptor = os.open(requested.parts[-1], os.O_RDONLY | nofollow, dir_fd=current)
        try:
            final = os.fstat(final_descriptor)
        finally:
            os.close(final_descriptor)
        if (identity(after) != identity(opened) or identity(final) != identity(opened)
                or len(raw) != opened.st_size):
            raise ValueError("job script changed while it was read")
    except OSError as error:
        raise ValueError("job script path could not be opened safely") from error
    finally:
        if descriptor is not None:
            os.close(descriptor)
        for directory_descriptor in reversed(directory_descriptors):
            os.close(directory_descriptor)
    try:
        contents = raw.decode("utf-8")
    except UnicodeDecodeError as error:
        raise ValueError("job script is not UTF-8") from error
    return requested, contents


def save(path, record):
    # Persist intent before contacting sbatch; never silently retry an unknown job.
    path = Path(path)
    descriptor, temporary_name = tempfile.mkstemp(prefix="." + path.name + ".", dir=path.parent)
    temporary = Path(temporary_name)
    try:
        with os.fdopen(descriptor, "w") as handle:
            descriptor = None
            json.dump(record, handle, indent=2, sort_keys=True)
            handle.write("\n")
            handle.flush()
            os.fsync(handle.fileno())
        os.replace(temporary, path)
        directory = os.open(path.parent, os.O_RDONLY | getattr(os, "O_DIRECTORY", 0))
        try:
            os.fsync(directory)
        finally:
            os.close(directory)
    finally:
        if descriptor is not None:
            os.close(descriptor)
        temporary.unlink(missing_ok=True)


def submit_one(root, script, env, *, pin_sha256=None):
    root = Path(root).resolve()
    if (pin_sha256 is not None
            and (not isinstance(pin_sha256, str)
                 or re.fullmatch(r"[a-f0-9]{64}", pin_sha256) is None)):
        raise ValueError("submission pin SHA-256 is invalid")
    # This is the common fail-closed boundary: historical top-level stagers
    # remain readable evidence, but they can no longer submit new jobs.
    scope = submission_scope(root)
    requested_script, script_contents = stage_script(root, script)
    script_name = str(requested_script)
    one_job_script(script_contents)
    ledger = scope / ".atlas-progressive-submit.json"
    with exclusive_lock(scope, ".atlas-progressive-submit.lock"):
        history = read_json_file(ledger) if os.path.lexists(ledger) else []
        validate_confirmed_history(history)
        if any(r.get("stage") == str(root) for r in history):
            raise ValueError("this stage was already submitted")
        if os.path.lexists(root / "submission-intent.json"):
            read_json_file(root / "submission-intent.json")
            raise ValueError("stage has an untracked submission intent")
        raw = subprocess.check_output(["squeue", "-h", "-r", "-u", os.environ["USER"], "-o", "%i"],
                                      text=True, timeout=20)
        active = capacity(raw)
        record = dict(stage=str(root), script=script_name, queue_before=active,
                      status="SUBMISSION_INTENT_NOT_CONFIRMED", max_outstanding=10)
        if pin_sha256 is not None:
            record["pin_sha256"] = pin_sha256
        history.append(record)
        save(ledger, history)
        save(root / "submission-intent.json", record)
        clean_env = {k: v for k, v in env.items() if not k.startswith("SBATCH_")}
        # Submit the exact bytes checked above.  Passing the stage path here
        # would reopen it after squeue/ledger writes and reintroduce a race.
        result = subprocess.check_output(["sbatch", "--parsable", "--export=ALL"],
                                         cwd=root, env=clean_env, input=script_contents,
                                         text=True, timeout=25).strip()
        if not re.fullmatch(r"[0-9]+(?:;[A-Za-z0-9_.-]+)?", result):
            raise ValueError("uncertain sbatch response; reconcile durable intent")
        record.update(job=result.split(";")[0], status="SUBMITTED")
        validate_confirmed_history(history)
        save(ledger, history)
        save(root / "submission-intent.json", record)
        return record

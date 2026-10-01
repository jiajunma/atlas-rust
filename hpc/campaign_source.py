"""Content-addressed immutable source archives for one HPC campaign."""
from contextlib import contextmanager
import hashlib
import os
from pathlib import Path, PurePosixPath
import shutil
import stat
import sys
import tarfile
import tempfile

from campaign_workspace import campaign_storage_root


SCHEMA = "atlas-campaign-source-tar-v1"
ARCHIVE_FILE_MODE = 0o644
OBJECT_MODE = 0o444


def _stable_identity(value):
    return (value.st_dev, value.st_ino, value.st_size, value.st_mtime_ns, value.st_ctime_ns)


@contextmanager
def _regular_reader(path):
    flags = os.O_RDONLY | getattr(os, "O_CLOEXEC", 0) | getattr(os, "O_NOFOLLOW", 0)
    descriptor = os.open(path, flags)
    try:
        before = os.fstat(descriptor)
        if not stat.S_ISREG(before.st_mode):
            raise ValueError("artifact is not a regular file")
        with os.fdopen(descriptor, "rb", closefd=False) as handle:
            yield handle, before
        after = os.fstat(descriptor)
        if _stable_identity(before) != _stable_identity(after):
            raise ValueError("artifact changed while it was read")
    finally:
        os.close(descriptor)


def _digest_handle(handle):
    value = hashlib.sha256()
    handle.seek(0)
    for block in iter(lambda: handle.read(1024 * 1024), b""):
        value.update(block)
    handle.seek(0)
    return value.hexdigest()


def digest(path):
    with _regular_reader(Path(path)) as (handle, _):
        return _digest_handle(handle)


def _validate_expected(expected):
    if not isinstance(expected, dict):
        raise ValueError("invalid accepted source manifest")
    for name, sha in expected.items():
        pure = PurePosixPath(name) if isinstance(name, str) else None
        if (pure is None or not name or not pure.parts or name == "."
                or pure.is_absolute() or pure.as_posix() != name
                or any(part in ("", ".", "..") for part in pure.parts)
                or not isinstance(sha, str) or len(sha) != 64
                or not all(c in "0123456789abcdef" for c in sha)):
            raise ValueError("invalid accepted source manifest")


def _real_directory(path):
    try:
        value = os.lstat(path)
    except FileNotFoundError as error:
        raise ValueError("required directory is missing") from error
    if not stat.S_ISDIR(value.st_mode):
        raise ValueError("required directory is not a real directory")


def file_manifest(root):
    root = Path(root)
    _real_directory(root)
    result = {}
    for path in sorted(root.rglob("*")):
        value = path.lstat()
        if stat.S_ISLNK(value.st_mode):
            raise ValueError("source archives do not accept symlinks")
        if stat.S_ISDIR(value.st_mode):
            continue
        if not stat.S_ISREG(value.st_mode):
            raise ValueError("source archives accept only directories and regular files")
        result[str(path.relative_to(root))] = digest(path)
    return result


def _validate_reference(reference):
    if (not isinstance(reference, dict)
            or set(reference) != {"schema", "role", "sha256", "bytes", "files",
                                  "source_bytes"}
            or reference.get("schema") != SCHEMA or reference.get("role") != "rust-source"
            or not isinstance(reference.get("sha256"), str) or len(reference["sha256"]) != 64
            or not all(c in "0123456789abcdef" for c in reference["sha256"])
            or type(reference.get("bytes")) is not int or reference["bytes"] < 0
            or type(reference.get("files")) is not int or reference["files"] < 0
            or type(reference.get("source_bytes")) is not int or reference["source_bytes"] < 0):
        raise ValueError("invalid source-object reference")


def object_path(campaign, reference):
    _validate_reference(reference)
    campaign = campaign_storage_root(campaign)
    sha = reference["sha256"]
    return Path(campaign) / "objects/sha256" / sha[:2] / sha


def _safe_member_name(name):
    pure = PurePosixPath(name)
    return bool(name) and name != "." and bool(pure.parts) and not pure.is_absolute() and pure.as_posix() == name and all(
        part not in ("", ".", "..") for part in pure.parts)


def _inspect_archive(handle, reference, expected=None):
    if _digest_handle(handle) != reference["sha256"]:
        raise ValueError("changed campaign source object")
    if expected is not None:
        _validate_expected(expected)
    with tarfile.open(fileobj=handle, mode="r:") as archive:
        members = archive.getmembers()
        names = [member.name for member in members]
        if (len(members) != reference["files"] or names != sorted(names) or len(names) != len(set(names))
                or any(not member.isfile() or not _safe_member_name(member.name)
                       or member.mode != ARCHIVE_FILE_MODE
                       or member.uid != 0 or member.gid != 0 or member.mtime != 0
                       or member.uname != "" or member.gname != "" for member in members)
                or sum(member.size for member in members) != reference["source_bytes"]
                or (expected is not None and names != sorted(expected))):
            raise ValueError("unsafe or incomplete source archive")
        if expected is not None:
            for member in members:
                payload = archive.extractfile(member)
                if payload is None:
                    raise ValueError("missing regular-file payload")
                value = hashlib.sha256()
                with payload:
                    for block in iter(lambda: payload.read(1024 * 1024), b""):
                        value.update(block)
                if value.hexdigest() != expected[member.name]:
                    raise ValueError("source archive differs from accepted manifest")
    if _digest_handle(handle) != reference["sha256"]:
        raise ValueError("campaign source object changed while it was read")


@contextmanager
def _verified_object(campaign, reference):
    path = object_path(campaign, reference)
    try:
        reader = _regular_reader(path)
        handle, value = reader.__enter__()
    except (FileNotFoundError, OSError) as error:
        raise ValueError("missing or unsafe campaign source object") from error
    try:
        if value.st_size != reference["bytes"] or stat.S_IMODE(value.st_mode) != OBJECT_MODE:
            raise ValueError("missing or changed campaign source object")
        _inspect_archive(handle, reference)
        yield path, handle
    finally:
        reader.__exit__(*sys.exc_info())


def verify_source_archive(campaign, reference, expected=None):
    _validate_reference(reference)
    _validate_object_directories(campaign, reference["sha256"])
    with _verified_object(campaign, reference) as (path, handle):
        if expected is not None:
            _inspect_archive(handle, reference, expected)
        return path


def _fsync_directory(path):
    flags = os.O_RDONLY | getattr(os, "O_DIRECTORY", 0) | getattr(os, "O_CLOEXEC", 0) | getattr(os, "O_NOFOLLOW", 0)
    descriptor = os.open(path, flags)
    try:
        os.fsync(descriptor)
    finally:
        os.close(descriptor)


def _ensure_real_child(parent, name):
    parent = Path(parent)
    _real_directory(parent)
    child = parent / name
    try:
        child.mkdir()
    except FileExistsError:
        pass
    else:
        _fsync_directory(parent)
    _real_directory(child)
    return child


def _ensure_object_directories(campaign, shard):
    campaign = Path(campaign)
    campaign.mkdir(parents=True, exist_ok=True)
    _real_directory(campaign)
    objects = _ensure_real_child(campaign, "objects")
    sha256 = _ensure_real_child(objects, "sha256")
    return objects, _ensure_real_child(sha256, shard)


def _validate_object_directories(campaign, sha):
    campaign = Path(campaign)
    objects = campaign / "objects"
    sha256 = objects / "sha256"
    for path in (campaign, objects, sha256, sha256 / sha[:2]):
        _real_directory(path)


def store_source_archive(campaign, source, expected):
    """Store a deterministic regular-file tree and return a path-free ref."""
    campaign, source = campaign_storage_root(campaign), Path(source)
    _validate_expected(expected)
    if file_manifest(source) != expected:
        raise ValueError("source tree differs from its accepted manifest")
    campaign.mkdir(parents=True, exist_ok=True)
    _real_directory(campaign)
    objects = _ensure_real_child(campaign, "objects")
    incoming = _ensure_real_child(objects, "incoming")
    descriptor, temporary_name = tempfile.mkstemp(prefix=".source-", dir=incoming)
    temporary = Path(temporary_name)
    total = 0
    try:
        with os.fdopen(descriptor, "w+b") as temporary_handle:
            descriptor = None
            with tarfile.open(fileobj=temporary_handle, mode="w", format=tarfile.PAX_FORMAT) as archive:
                for name in sorted(expected):
                    path = source / name
                    with _regular_reader(path) as (handle, value):
                        if _digest_handle(handle) != expected[name]:
                            raise ValueError("source changed while it was archived")
                        info = tarfile.TarInfo(name)
                        info.size = value.st_size
                        info.mode = ARCHIVE_FILE_MODE
                        info.mtime = info.uid = info.gid = 0
                        info.uname = info.gname = ""
                        total += info.size
                        archive.addfile(info, handle)
            temporary_handle.flush()
            os.fchmod(temporary_handle.fileno(), OBJECT_MODE)
            os.fsync(temporary_handle.fileno())
        sha = digest(temporary)
        reference = dict(schema=SCHEMA, role="rust-source", sha256=sha,
                         bytes=temporary.stat().st_size, files=len(expected), source_bytes=total)
        with _regular_reader(temporary) as (handle, _):
            _inspect_archive(handle, reference, expected)
        _, shard = _ensure_object_directories(campaign, sha[:2])
        destination = shard / sha
        try:
            os.link(temporary, destination, follow_symlinks=False)
            _fsync_directory(destination.parent)
        except FileExistsError:
            verify_source_archive(campaign, reference)
        verify_source_archive(campaign, reference)
        return reference
    finally:
        if descriptor is not None:
            os.close(descriptor)
        temporary.unlink(missing_ok=True)


def materialize_source_archive(campaign, reference, expected, destination):
    """Copy a verified object into a new mutable job workspace."""
    campaign = campaign_storage_root(campaign)
    _validate_reference(reference)
    _validate_expected(expected)
    destination = Path(destination)
    if os.path.lexists(destination):
        raise ValueError("source materialization destination already exists")
    _real_directory(destination.parent)
    temporary = Path(tempfile.mkdtemp(prefix=".source-materialize-", dir=destination.parent))
    try:
        _validate_object_directories(campaign, reference["sha256"])
        with _verified_object(campaign, reference) as (_, handle):
            _inspect_archive(handle, reference, expected)
            handle.seek(0)
            with tarfile.open(fileobj=handle, mode="r:") as archive:
                for member in archive.getmembers():
                    target = temporary / member.name
                    target.parent.mkdir(parents=True, exist_ok=True)
                    payload = archive.extractfile(member)
                    if payload is None:
                        raise ValueError("missing regular-file payload")
                    flags = os.O_WRONLY | os.O_CREAT | os.O_EXCL | getattr(os, "O_NOFOLLOW", 0)
                    descriptor = os.open(target, flags, ARCHIVE_FILE_MODE)
                    try:
                        os.fchmod(descriptor, ARCHIVE_FILE_MODE)
                        with payload, os.fdopen(descriptor, "wb") as output:
                            descriptor = None
                            shutil.copyfileobj(payload, output)
                    finally:
                        if descriptor is not None:
                            os.close(descriptor)
            if _digest_handle(handle) != reference["sha256"]:
                raise ValueError("campaign source object changed during materialization")
        if file_manifest(temporary) != expected:
            raise ValueError("materialized source differs from accepted manifest")
        if os.path.lexists(destination):
            raise ValueError("source materialization destination appeared during extraction")
        temporary.rename(destination)
        return destination
    except Exception:
        shutil.rmtree(temporary, ignore_errors=True)
        raise

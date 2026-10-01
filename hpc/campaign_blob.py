"""Immutable path-free blobs sharing the campaign SHA-256 object store."""
from contextlib import contextmanager
import os
from pathlib import Path
import re
import shutil
import stat
import tempfile

from campaign_source import (OBJECT_MODE, _digest_handle, _ensure_object_directories,
                             _ensure_real_child, _fsync_directory, _real_directory,
                             _regular_reader, _validate_object_directories)
from campaign_workspace import campaign_storage_root


SCHEMA = "atlas-campaign-blob-v1"
ROLE_RE = re.compile(r"[a-z][a-z0-9-]{0,62}")


def _validate_reference(reference):
    if (not isinstance(reference, dict)
            or set(reference) != {"schema", "role", "sha256", "bytes"}
            or reference.get("schema") != SCHEMA
            or not isinstance(reference.get("role"), str) or not ROLE_RE.fullmatch(reference["role"])
            or not isinstance(reference.get("sha256"), str) or len(reference["sha256"]) != 64
            or not all(c in "0123456789abcdef" for c in reference["sha256"])
            or type(reference.get("bytes")) is not int or reference["bytes"] < 0):
        raise ValueError("invalid campaign blob reference")


def blob_path(campaign, reference):
    _validate_reference(reference)
    campaign = campaign_storage_root(campaign)
    sha = reference["sha256"]
    return Path(campaign) / "objects/sha256" / sha[:2] / sha


@contextmanager
def _verified_blob(campaign, reference):
    _validate_reference(reference)
    _validate_object_directories(campaign, reference["sha256"])
    path = blob_path(campaign, reference)
    try:
        reader = _regular_reader(path)
        handle, value = reader.__enter__()
    except (FileNotFoundError, OSError) as error:
        raise ValueError("missing or unsafe campaign blob") from error
    try:
        if (value.st_size != reference["bytes"] or stat.S_IMODE(value.st_mode) != OBJECT_MODE
                or _digest_handle(handle) != reference["sha256"]):
            raise ValueError("missing or changed campaign blob")
        yield path, handle
        if _digest_handle(handle) != reference["sha256"]:
            raise ValueError("campaign blob changed while it was read")
    finally:
        reader.__exit__(*__import__("sys").exc_info())


def verify_blob(campaign, reference):
    with _verified_blob(campaign, reference) as (path, _):
        return path


def read_blob(campaign, reference):
    with _verified_blob(campaign, reference) as (_, handle):
        handle.seek(0)
        return handle.read()


def store_blob(campaign, source, role):
    if not isinstance(role, str) or not ROLE_RE.fullmatch(role):
        raise ValueError("invalid campaign blob role")
    campaign, source = campaign_storage_root(campaign), Path(source)
    campaign.mkdir(parents=True, exist_ok=True)
    _real_directory(campaign)
    objects = _ensure_real_child(campaign, "objects")
    incoming = _ensure_real_child(objects, "incoming")
    descriptor, temporary_name = tempfile.mkstemp(prefix=".blob-", dir=incoming)
    temporary = Path(temporary_name)
    try:
        with _regular_reader(source) as (input_handle, before), os.fdopen(descriptor, "wb") as output:
            descriptor = None
            shutil.copyfileobj(input_handle, output)
            output.flush()
            os.fchmod(output.fileno(), OBJECT_MODE)
            os.fsync(output.fileno())
        with _regular_reader(temporary) as (handle, _):
            sha = _digest_handle(handle)
        reference = dict(schema=SCHEMA, role=role, sha256=sha, bytes=before.st_size)
        if temporary.stat().st_size != before.st_size:
            raise ValueError("blob source changed while it was copied")
        _, shard = _ensure_object_directories(campaign, sha[:2])
        destination = shard / sha
        try:
            os.link(temporary, destination, follow_symlinks=False)
            _fsync_directory(destination.parent)
        except FileExistsError:
            verify_blob(campaign, reference)
        verify_blob(campaign, reference)
        return reference
    finally:
        if descriptor is not None:
            os.close(descriptor)
        temporary.unlink(missing_ok=True)

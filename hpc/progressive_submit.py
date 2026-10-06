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
from pathlib import Path, PurePosixPath
import pwd
import re
import stat
import subprocess
import tempfile

import campaign_workspace
from campaign_workspace import submission_scope


ACTIVE_STAGE_NAME = "weyl-context-core-after-v5"
STAGE_CREATION_CONTRACT_SCHEMA = "atlas-stage-creation-contract-v14"
STAGE_CREATION_PREDECESSOR_SCHEMA = \
    "atlas-stage-creation-predecessor-v14"
STAGE_TREE_INVENTORY_SCHEMA = "atlas-stage-tree-inventory-v1"
STAGE_CREATION_EVENT_SCHEMA = "atlas-stage-creation-event-v1"
STAGE_CREATION_RECEIPT_SCHEMA = "atlas-stage-creation-receipt-v1"
STAGE_CREATION_TRANSACTION_SCHEMA = "atlas-stage-creation-transaction-v1"
STAGE_CREATION_PRIOR_FAILURE_SCHEMA = \
    "atlas-stage-creation-prior-failure-v1"
STAGE_CREATION_RECEIPT = ".atlas-stage-creation.json"
STAGE_CREATION_TRANSACTION = ".atlas-stage-creation-transaction.json"
STAGE_CREATION_MARKER = STAGE_CREATION_TRANSACTION
STAGE_CREATION_PREPARED = (
    ".atlas-stage-creation-" + ACTIVE_STAGE_NAME + "-prepared.json"
)
STAGE_CREATION_SEALED = (
    ".atlas-stage-creation-" + ACTIVE_STAGE_NAME + "-sealed.json"
)
STAGE_CREATION_PUBLISHED = (
    ".atlas-stage-creation-" + ACTIVE_STAGE_NAME + "-published.json"
)
STAGE_CREATION_LOCK = ".atlas-stage-creation.lock"
_SHA256 = re.compile(r"[0-9a-f]{64}\Z")
EXPECTED_PREDECESSOR_LINEAGE = (
    ("weyl-parent-seal-v1", "3872554"),
    ("ladder-boundary-before-v2", "3872594"),
    ("ladder-boundary-before-v3", "3873400"),
    ("ladder-boundary-after-v1", "3873497"),
    ("ladder-boundary-after-v2", "3874203"),
    ("ladder-boundary-after-v3", "3875239"),
    ("ladder-boundary-index-v1", "3879103"),
    ("weyl-context-core-capture-v1", "3884124"),
    ("weyl-context-core-capture-v2", "3884371"),
    ("weyl-context-core-capture-v3", "3884456"),
    ("weyl-context-core-capture-v4", "3884494"),
    ("weyl-context-core-capture-v5", "3884727"),
    ("weyl-context-core-capture-v6", "3884751"),
    ("weyl-context-core-capture-v7", "3884780"),
    ("weyl-context-core-capture-v8", "3884807"),
    ("weyl-context-core-before-v1", "3884862"),
    ("weyl-context-core-before-v2", "3884880"),
    ("weyl-context-core-before-v3", "3884903"),
    ("weyl-context-core-before-v4", "3886748"),
    ("weyl-context-core-after-v1", "3890328"),
    ("weyl-context-core-after-v2", "3890580"),
    ("weyl-context-core-after-v3", "3899303"),
    ("weyl-context-core-after-v4", "3899885"),
)


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


def _strict_json(raw, label):
    def unique(pairs):
        result = {}
        for key, value in pairs:
            if key in result:
                raise ValueError("duplicate JSON key in " + label)
            result[key] = value
        return result

    def reject_constant(value):
        raise ValueError("non-finite JSON value in " + label + ": " + value)

    try:
        return json.loads(raw.decode("utf-8"), object_pairs_hook=unique,
                          parse_constant=reject_constant)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise ValueError("invalid JSON state input: " + label) from error


def _json_bytes(value):
    return (json.dumps(value, indent=2, sort_keys=True) + "\n").encode("utf-8")


def _json_sha(value):
    return hashlib.sha256(_json_bytes(value)).hexdigest()


def _safe_relative(name):
    if not isinstance(name, str) or not name:
        raise ValueError("unsafe stage-relative path")
    value = PurePosixPath(name)
    if (value.is_absolute() or value.as_posix() != name
            or any(part in ("", ".", "..") for part in value.parts)):
        raise ValueError("unsafe stage-relative path")
    return value.parts


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
        raise ValueError("required directory is missing or unsafe") from error
    value = os.fstat(descriptor)
    if not stat.S_ISDIR(value.st_mode):
        os.close(descriptor)
        raise ValueError("required path is not a directory")
    return descriptor


def _open_child_directory(parent, name, *, create=False, mode=0o755):
    if (not isinstance(name, str) or not name or "/" in name
            or name in (".", "..")):
        raise ValueError("unsafe directory component")
    if create:
        try:
            os.mkdir(name, mode, dir_fd=parent)
        except FileExistsError:
            pass
        # Replaying the parent sync is required after a process or machine
        # failure: visibility of an existing child does not prove that the
        # directory entry reached stable storage in the earlier attempt.
        os.fsync(parent)
    try:
        descriptor = os.open(name, _directory_flags(), dir_fd=parent)
    except OSError as error:
        raise ValueError("directory component is missing or unsafe") from error
    if not stat.S_ISDIR(os.fstat(descriptor).st_mode):
        os.close(descriptor)
        raise ValueError("directory component is not a directory")
    return descriptor


def _require_private_writes(value, label):
    """Reject state that another group member or user may rewrite in place."""
    if stat.S_IMODE(value.st_mode) & 0o022:
        raise ValueError(label + " is group/world writable")
    return value


def _stable_file_at(parent, name, *, mode=None, nlink=1,
                    forbid_shared_write=False):
    flags = (os.O_RDONLY | getattr(os, "O_CLOEXEC", 0)
             | getattr(os, "O_NOFOLLOW", 0))
    descriptor = current_descriptor = None
    try:
        descriptor = os.open(name, flags, dir_fd=parent)
        opened = os.fstat(descriptor)
        if (not stat.S_ISREG(opened.st_mode)
                or (mode is not None and stat.S_IMODE(opened.st_mode) != mode)
                or (forbid_shared_write
                    and stat.S_IMODE(opened.st_mode) & 0o022)
                or (nlink is not None and opened.st_nlink != nlink)):
            raise ValueError("state path is not the required regular file")
        chunks = []
        while True:
            chunk = os.read(descriptor, 64 * 1024)
            if not chunk:
                break
            chunks.append(chunk)
        after = os.fstat(descriptor)
        current_descriptor = os.open(name, flags, dir_fd=parent)
        current = os.fstat(current_descriptor)
        identity = lambda value: (
            value.st_dev, value.st_ino, value.st_size, value.st_mtime_ns,
            value.st_ctime_ns, stat.S_IFMT(value.st_mode), value.st_nlink,
        )
        raw = b"".join(chunks)
        if (identity(opened) != identity(after)
                or identity(opened) != identity(current)
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


def _stable_relative(root, name, *, mode=None, nlink=1,
                     forbid_shared_write=False):
    descriptors = []
    try:
        current = _open_directory(root)
        descriptors.append(current)
        parts = _safe_relative(name)
        for part in parts[:-1]:
            current = _open_child_directory(current, part)
            descriptors.append(current)
        return _stable_file_at(
            current, parts[-1], mode=mode, nlink=nlink,
            forbid_shared_write=forbid_shared_write)
    finally:
        for descriptor in reversed(descriptors):
            os.close(descriptor)


def _stable_relative_at(root, name, *, mode=None, nlink=1,
                        forbid_shared_write=False):
    """Read a stable relative file beneath an already verified directory."""
    descriptors = []
    try:
        current = os.dup(root)
        descriptors.append(current)
        parts = _safe_relative(name)
        for part in parts[:-1]:
            current = _open_child_directory(current, part)
            descriptors.append(current)
        return _stable_file_at(
            current, parts[-1], mode=mode, nlink=nlink,
            forbid_shared_write=forbid_shared_write)
    finally:
        for descriptor in reversed(descriptors):
            os.close(descriptor)


def _transaction_temp_name(name, raw):
    """Return the one deterministic temporary leaf for these exact bytes."""
    if (not isinstance(name, str) or not name or "/" in name
            or name in (".", "..")):
        raise ValueError("unsafe publication name")
    digest = hashlib.sha256(name.encode("utf-8") + b"\0" + raw).hexdigest()
    return ".atlas-publish-" + digest + ".tmp"


def _read_descriptor(descriptor):
    os.lseek(descriptor, 0, os.SEEK_SET)
    chunks = []
    while True:
        chunk = os.read(descriptor, 64 * 1024)
        if not chunk:
            break
        chunks.append(chunk)
    return b"".join(chunks)


def _lstat_at(parent, name):
    try:
        return os.stat(name, dir_fd=parent, follow_symlinks=False)
    except FileNotFoundError:
        return None
    except OSError as error:
        raise ValueError("publication path could not be inspected") from error


def _publication_identity(value):
    return (
        value.st_dev, value.st_ino, value.st_size, value.st_mtime_ns,
        value.st_ctime_ns, stat.S_IFMT(value.st_mode),
        stat.S_IMODE(value.st_mode), value.st_nlink,
    )


def _exact_publication_file_at(parent, name, raw, *, nlink):
    """Return one stable exact immutable publication inode."""
    before = _lstat_at(parent, name)
    if (before is None or not stat.S_ISREG(before.st_mode)
            or stat.S_IMODE(before.st_mode) != 0o444
            or before.st_nlink != nlink):
        raise ValueError("publication path is not the required immutable file")
    if _stable_file_at(parent, name, mode=0o444, nlink=nlink) != raw:
        raise ValueError("publication path has changed bytes")
    after = _lstat_at(parent, name)
    if after is None or _publication_identity(after) != _publication_identity(before):
        raise ValueError("publication path changed while it was inspected")
    return after


def _publication_residual_names(parent):
    try:
        names = os.listdir(parent)
    except OSError as error:
        raise ValueError("publication directory could not be listed") from error
    return {
        name for name in names
        if name.startswith(".atlas-publish-") and name.endswith(".tmp")
    }


def _exact_link_pair_at(parent, name, temporary, raw):
    target = _exact_publication_file_at(parent, name, raw, nlink=2)
    source = _exact_publication_file_at(parent, temporary, raw, nlink=2)
    if (target.st_dev, target.st_ino) != (source.st_dev, source.st_ino):
        raise ValueError("publication names do not identify one inode")
    return target


def _settle_link_pair_at(parent, name, raw, *, _fault=None):
    """Consume only the exact two-name state of this known publication."""
    temporary = _transaction_temp_name(name, raw)
    target = _lstat_at(parent, name)
    source = _lstat_at(parent, temporary)
    if target is None:
        return False
    if target.st_nlink == 1:
        _exact_publication_file_at(parent, name, raw, nlink=1)
        if source is not None:
            raise ValueError("completed publication retains a transaction residual")
        return False
    if target.st_nlink != 2 or source is None:
        raise ValueError("publication target has an unknown link state")
    _exact_link_pair_at(parent, name, temporary, raw)
    # The first parent sync makes both names durable.  Recovery deliberately
    # repeats it before consuming the only durable source-side name.
    os.fsync(parent)
    _call_fault(_fault, "after_file_parent_fsync", {
        "name": name, "temporary": temporary,
    })
    _exact_link_pair_at(parent, name, temporary, raw)
    try:
        os.unlink(temporary, dir_fd=parent)
    except OSError as error:
        raise ValueError("publication transaction residual could not be removed") \
            from error
    _call_fault(_fault, "after_file_unlink", {
        "name": name, "temporary": temporary,
    })
    os.fsync(parent)
    _exact_publication_file_at(parent, name, raw, nlink=1)
    return True


def _open_publication_residual(parent, temporary, mode):
    """Open only a pre-inspected, single-link regular transaction leaf."""
    residual = _lstat_at(parent, temporary)
    if (residual is None or not stat.S_ISREG(residual.st_mode)
            or residual.st_nlink != 1
            or stat.S_IMODE(residual.st_mode) not in (0o600, mode)):
        raise ValueError("publication transaction residual is unsafe")
    nonblock = getattr(os, "O_NONBLOCK", 0)
    try:
        return os.open(
            temporary,
            os.O_RDWR | getattr(os, "O_CLOEXEC", 0)
            | getattr(os, "O_NOFOLLOW", 0) | nonblock,
            dir_fd=parent,
        )
    except PermissionError:
        try:
            return os.open(
                temporary,
                os.O_RDONLY | getattr(os, "O_CLOEXEC", 0)
                | getattr(os, "O_NOFOLLOW", 0) | nonblock,
                dir_fd=parent,
            )
        except OSError as error:
            raise ValueError(
                "publication transaction residual could not be opened") \
                from error
    except OSError as error:
        raise ValueError(
            "publication transaction residual could not be opened") from error


def _recover_complete_publication_at(parent, name, raw, *, _fault=None):
    """Settle an exact link pair, but never create or consume other state."""
    target = _lstat_at(parent, name)
    if target is None:
        return False
    _settle_link_pair_at(parent, name, raw, _fault=_fault)
    return True


def _publish_bytes_at(parent, name, raw, *, mode=0o444, _fault=None):
    """Publish exact bytes through one portable hardlink no-replace commit.

    The deterministic temporary inode is frozen and synced before ``link``
    atomically claims the final leaf.  Only an exact two-name, two-link inode
    may be recovered.  Unknown state is retained for external reconciliation.
    """
    if (not isinstance(name, str) or not name or "/" in name
            or name in (".", "..") or not isinstance(raw, bytes)
            or mode != 0o444):
        raise ValueError("immutable publication bytes, name, or mode changed")
    temporary = _transaction_temp_name(name, raw)
    residuals = _publication_residual_names(parent)
    if residuals - {temporary}:
        raise ValueError("unknown publication transaction residual")

    target = _lstat_at(parent, name)
    if target is not None:
        _settle_link_pair_at(parent, name, raw, _fault=_fault)
        if _publication_residual_names(parent):
            raise ValueError("completed publication retains a transaction residual")
        # A visible final may follow an interrupted final parent sync.  Replay
        # durability even for an otherwise byte-and-inode-idempotent call.
        os.fsync(parent)
        _exact_publication_file_at(parent, name, raw, nlink=1)
        return

    flags = (os.O_RDWR | os.O_CREAT | os.O_EXCL
             | getattr(os, "O_CLOEXEC", 0)
             | getattr(os, "O_NOFOLLOW", 0))
    descriptor = None
    try:
        if _lstat_at(parent, temporary) is None:
            try:
                descriptor = os.open(temporary, flags, 0o600, dir_fd=parent)
            except FileExistsError:
                descriptor = _open_publication_residual(
                    parent, temporary, mode)
            except OSError as error:
                raise ValueError(
                    "publication transaction residual could not be created") \
                    from error
        else:
            descriptor = _open_publication_residual(parent, temporary, mode)
        opened = os.fstat(descriptor)
        permissions = stat.S_IMODE(opened.st_mode)
        if (not stat.S_ISREG(opened.st_mode) or opened.st_nlink != 1
                or permissions not in (0o600, mode)):
            raise ValueError("publication transaction residual is unsafe")
        present = _read_descriptor(descriptor)
        if not raw.startswith(present):
            raise ValueError("publication transaction residual has changed bytes")
        if permissions == mode:
            if present != raw:
                raise ValueError("frozen publication transaction residual is partial")
        else:
            if present != raw:
                os.lseek(descriptor, len(present), os.SEEK_SET)
                remaining = memoryview(raw)[len(present):]
                while remaining:
                    written = os.write(descriptor, remaining)
                    if written <= 0:
                        raise ValueError("short immutable publication write")
                    remaining = remaining[written:]
            os.fchmod(descriptor, mode)
        os.fsync(descriptor)
        final = os.fstat(descriptor)
        if ((final.st_dev, final.st_ino) != (opened.st_dev, opened.st_ino)
                or final.st_nlink != 1
                or stat.S_IMODE(final.st_mode) != mode):
            raise ValueError("publication transaction inode changed")
        os.close(descriptor)
        descriptor = None
        _exact_publication_file_at(parent, temporary, raw, nlink=1)
        try:
            os.link(
                temporary, name,
                src_dir_fd=parent, dst_dir_fd=parent,
                follow_symlinks=False,
            )
        except FileExistsError as error:
            # A cooperative retry may observe a link completed by its exact
            # predecessor.  Any independently-created target remains intact.
            try:
                if _settle_link_pair_at(
                        parent, name, raw, _fault=_fault):
                    return
            except ValueError:
                pass
            raise ValueError("immutable publication target already exists") from error
        except OSError as error:
            raise ValueError("immutable publication link could not be created") \
                from error
        _exact_link_pair_at(parent, name, temporary, raw)
        _call_fault(_fault, "after_file_link", {
            "name": name, "temporary": temporary,
        })
        os.fsync(parent)
        _call_fault(_fault, "after_file_parent_fsync", {
            "name": name, "temporary": temporary,
        })
        _exact_link_pair_at(parent, name, temporary, raw)
        try:
            os.unlink(temporary, dir_fd=parent)
        except OSError as error:
            raise ValueError("publication transaction residual could not be removed") \
                from error
        _call_fault(_fault, "after_file_unlink", {
            "name": name, "temporary": temporary,
        })
        os.fsync(parent)
        _exact_publication_file_at(parent, name, raw, nlink=1)
    except FileNotFoundError as error:
        raise ValueError("publication transaction residual disappeared") from error
    finally:
        if descriptor is not None:
            os.close(descriptor)


def _publish_json_at(parent, name, value, *, mode=0o444, _fault=None):
    """Publish one immutable JSON file without replacing an existing name."""
    _publish_bytes_at(
        parent, name, _json_bytes(value), mode=mode, _fault=_fault)


class _HeldLock:
    """A held lock whose pathname identity can be rechecked before mutation."""

    def __init__(self, directory_descriptor, descriptor, name, opened):
        self._directory_descriptor = directory_descriptor
        self._descriptor = descriptor
        self._name = name
        self._identity = (
            opened.st_dev, opened.st_ino, stat.S_IFMT(opened.st_mode),
            opened.st_nlink,
        )
        directory = os.fstat(directory_descriptor)
        self._directory_identity = (directory.st_dev, directory.st_ino)

    def fileno(self):
        return self._descriptor

    def __index__(self):
        return self._descriptor

    def validate(self):
        held_directory = os.fstat(self._directory_descriptor)
        if (held_directory.st_dev, held_directory.st_ino) \
                != self._directory_identity:
            raise ValueError("lock directory changed while held")
        held = os.fstat(self._descriptor)
        if (held.st_dev, held.st_ino, stat.S_IFMT(held.st_mode), held.st_nlink) \
                != self._identity:
            raise ValueError("held lock inode changed")
        current_descriptor = None
        try:
            current_descriptor = os.open(
                self._name,
                os.O_RDWR | getattr(os, "O_CLOEXEC", 0)
                | getattr(os, "O_NOFOLLOW", 0),
                dir_fd=self._directory_descriptor,
            )
            current = os.fstat(current_descriptor)
        except OSError as error:
            raise ValueError("lock path changed while it was held") from error
        finally:
            if current_descriptor is not None:
                os.close(current_descriptor)
        if (current.st_dev, current.st_ino, stat.S_IFMT(current.st_mode),
                current.st_nlink) != self._identity:
            raise ValueError("lock path changed while it was held")
        return self


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
        held = _HeldLock(directory_descriptor, descriptor, name, opened)
        yield held
        held.validate()
    finally:
        os.close(descriptor)
        os.close(directory_descriptor)


def _fixed_campaign(campaign):
    expected = (Path(campaign_workspace.HPC_HOME)
                / campaign_workspace.ACTIVE_CAMPAIGN).absolute()
    candidate = Path(campaign)
    if (not candidate.is_absolute() or candidate != expected
            or candidate.name != campaign_workspace.ACTIVE_CAMPAIGN):
        raise ValueError("stage creation requires the exact active campaign")
    descriptor = _open_directory(candidate)
    try:
        _require_private_writes(os.fstat(descriptor), "active campaign")
    finally:
        os.close(descriptor)
    return candidate


def _require_directory_identity(path, expected, label):
    descriptor = _open_directory(path)
    try:
        observed = os.fstat(descriptor)
        if (observed.st_dev, observed.st_ino) != (expected.st_dev, expected.st_ino):
            raise ValueError(label + " changed during the transaction")
    finally:
        os.close(descriptor)


def _validate_bound_files(value, *, campaign_root=False):
    if not isinstance(value, dict) or not value:
        raise ValueError("stage-creation predecessor files are not exact")
    result = {}
    for name, binding in value.items():
        parts = _safe_relative(name)
        if campaign_root and len(parts) != 1:
            raise ValueError("campaign predecessor file is not a root leaf")
        if (not isinstance(binding, dict)
                or set(binding) != {"sha256", "bytes", "mode", "nlink"}
                or not isinstance(binding.get("sha256"), str)
                or _SHA256.fullmatch(binding["sha256"]) is None
                or type(binding.get("bytes")) is not int
                or binding["bytes"] < 0
                or not isinstance(binding.get("mode"), str)
                or re.fullmatch(r"0[0-7]{3}", binding["mode"]) is None
                or type(binding.get("nlink")) is not int
                or binding["nlink"] != 1):
            raise ValueError("stage-creation predecessor file binding changed")
        result[name] = dict(binding)
    return result


def _predecessor_tree_scan(parent, prefix="", *, root_device=None,
                           seen_directories=None):
    """Return one stable no-follow inventory beneath an opened stage."""
    opened = os.fstat(parent)
    _require_private_writes(opened, "predecessor stage directory")
    if root_device is None:
        root_device = opened.st_dev
    if opened.st_dev != root_device:
        raise ValueError("predecessor stage tree crosses a filesystem")
    if seen_directories is None:
        seen_directories = {(opened.st_dev, opened.st_ino)}
    before_identity = _publication_identity(opened)
    try:
        names = sorted(os.listdir(parent))
    except OSError as error:
        raise ValueError("predecessor stage tree could not be listed") from error
    records = []
    for leaf in names:
        try:
            if (leaf in ("", ".", "..") or "/" in leaf
                    or leaf.encode("utf-8").decode("utf-8") != leaf):
                raise ValueError("predecessor stage tree has an unsafe name")
        except UnicodeError as error:
            raise ValueError(
                "predecessor stage tree has an unsafe name") from error
        name = prefix + leaf
        _safe_relative(name)
        try:
            value = os.stat(leaf, dir_fd=parent, follow_symlinks=False)
        except OSError as error:
            raise ValueError(
                "predecessor stage entry could not be inspected") from error
        if value.st_dev != root_device:
            raise ValueError("predecessor stage tree crosses a filesystem")
        if stat.S_ISDIR(value.st_mode):
            _require_private_writes(value, "predecessor stage directory")
            identity = (value.st_dev, value.st_ino)
            if identity in seen_directories:
                raise ValueError("predecessor stage repeats a directory")
            seen_directories.add(identity)
            child = _open_child_directory(parent, leaf)
            try:
                child_opened = os.fstat(child)
                if _publication_identity(value) != _publication_identity(
                        child_opened):
                    raise ValueError(
                        "predecessor stage directory changed while opened")
                records.append({
                    "path": name,
                    "kind": "directory",
                    "mode": format(stat.S_IMODE(child_opened.st_mode), "04o"),
                })
                records.extend(_predecessor_tree_scan(
                    child, name + "/", root_device=root_device,
                    seen_directories=seen_directories))
                child_after = os.fstat(child)
                current = os.stat(
                    leaf, dir_fd=parent, follow_symlinks=False)
                if (_publication_identity(child_opened)
                        != _publication_identity(child_after)
                        or _publication_identity(child_opened)
                        != _publication_identity(current)):
                    raise ValueError(
                        "predecessor stage directory changed during inventory")
            finally:
                os.close(child)
        elif stat.S_ISREG(value.st_mode) and value.st_nlink == 1:
            raw = _stable_file_at(
                parent, leaf, nlink=1, forbid_shared_write=True)
            current = os.stat(leaf, dir_fd=parent, follow_symlinks=False)
            if (_publication_identity(value) != _publication_identity(current)
                    or len(raw) != value.st_size):
                raise ValueError(
                    "predecessor stage file changed during inventory")
            records.append({
                "path": name,
                "kind": "file",
                "mode": format(stat.S_IMODE(value.st_mode), "04o"),
                "nlink": value.st_nlink,
                "bytes": len(raw),
                "sha256": hashlib.sha256(raw).hexdigest(),
            })
        else:
            raise ValueError(
                "predecessor stage entry is not a private regular object")
    try:
        names_after = sorted(os.listdir(parent))
    except OSError as error:
        raise ValueError("predecessor stage tree could not be relisted") from error
    if (names_after != names
            or _publication_identity(os.fstat(parent)) != before_identity):
        raise ValueError("predecessor stage directory changed during inventory")
    return records


def _predecessor_tree_binding(stage_descriptor):
    """Bind all predecessor children, including files not called out by name."""
    root = os.fstat(stage_descriptor)
    first = _predecessor_tree_scan(stage_descriptor)
    second = _predecessor_tree_scan(stage_descriptor)
    if first != second or _publication_identity(os.fstat(stage_descriptor)) \
            != _publication_identity(root):
        raise ValueError("predecessor stage tree changed between inventories")
    entries = sorted(first, key=lambda item: item["path"])
    inventory = {
        "schema": STAGE_TREE_INVENTORY_SCHEMA,
        "entries": entries,
    }
    return {
        "stage_tree_sha256": hashlib.sha256(
            _json_bytes(inventory)).hexdigest(),
        "stage_tree_files": sum(
            item["kind"] == "file" for item in entries),
        "stage_tree_directories": sum(
            item["kind"] == "directory" for item in entries),
        "stage_tree_bytes": sum(
            item.get("bytes", 0) for item in entries),
    }


def _validate_predecessor_state_descriptor(value, campaign):
    if value is None:
        raise ValueError("BEFORE v4 stage creation requires an exact predecessor")
    keys = {
        "schema", "stage", "stage_device", "stage_inode", "record",
        "campaign_files", "stage_files", "stage_tree_sha256",
        "stage_tree_files", "stage_tree_directories", "stage_tree_bytes",
    }
    if not isinstance(value, dict) or set(value) != keys:
        raise ValueError("stage-creation predecessor descriptor is not exact")
    stage = Path(value.get("stage", ""))
    expected_parent = Path(campaign) / "stages"
    record = value.get("record")
    record_keys = {
        "stage", "script", "queue_before", "status", "max_outstanding",
        "job", "pin_sha256", "stage_creation_sha256",
    }
    if (value.get("schema") != STAGE_CREATION_PREDECESSOR_SCHEMA
            or not stage.is_absolute() or stage.parent != expected_parent
            or stage.name != EXPECTED_PREDECESSOR_LINEAGE[-1][0]
            or not isinstance(record, dict)
            or set(record) != record_keys
            or record.get("stage") != str(stage)
            or record.get("job") != EXPECTED_PREDECESSOR_LINEAGE[-1][1]
            or record.get("status") != "SUBMITTED"
            or type(value.get("stage_device")) is not int
            or value["stage_device"] <= 0
            or type(value.get("stage_inode")) is not int
            or value["stage_inode"] <= 0
            or not isinstance(value.get("stage_tree_sha256"), str)
            or _SHA256.fullmatch(value["stage_tree_sha256"]) is None
            or type(value.get("stage_tree_files")) is not int
            or value["stage_tree_files"] < 1
            or type(value.get("stage_tree_directories")) is not int
            or value["stage_tree_directories"] < 0
            or type(value.get("stage_tree_bytes")) is not int
            or value["stage_tree_bytes"] < 0):
        raise ValueError("stage-creation predecessor descriptor changed")
    validate_confirmed_history([record])
    campaign_files = _validate_bound_files(
        value.get("campaign_files"), campaign_root=True)
    stage_files = _validate_bound_files(value.get("stage_files"))
    legacy_campaign_files = {
        ".atlas-stage-creation-prepared.json",
        ".atlas-stage-creation-sealed.json",
        ".atlas-stage-creation-published.json",
    }
    v2_scoped_campaign_files = {
        ".atlas-stage-creation-weyl-context-core-capture-v2-"
        + status + ".json"
        for status in ("prepared", "sealed", "published")
    }
    v3_scoped_campaign_files = {
        ".atlas-stage-creation-weyl-context-core-capture-v3-"
        + status + ".json"
        for status in ("prepared", "sealed", "published")
    }
    v4_scoped_campaign_files = {
        ".atlas-stage-creation-weyl-context-core-capture-v4-"
        + status + ".json"
        for status in ("prepared", "sealed", "published")
    }
    v5_scoped_campaign_files = {
        ".atlas-stage-creation-weyl-context-core-capture-v5-"
        + status + ".json"
        for status in ("prepared", "sealed", "published")
    }
    v6_scoped_campaign_files = {
        ".atlas-stage-creation-weyl-context-core-capture-v6-"
        + status + ".json"
        for status in ("prepared", "sealed", "published")
    }
    v7_scoped_campaign_files = {
        ".atlas-stage-creation-weyl-context-core-capture-v7-"
        + status + ".json"
        for status in ("prepared", "sealed", "published")
    }
    v8_scoped_campaign_files = {
        ".atlas-stage-creation-weyl-context-core-capture-v8-"
        + status + ".json"
        for status in ("prepared", "sealed", "published")
    }
    predecessor_scoped_campaign_files = {
        ".atlas-stage-creation-" + stage.name + "-" + status + ".json"
        for status in ("prepared", "sealed", "published")
    }
    before_v1_scoped_campaign_files = {
        ".atlas-stage-creation-weyl-context-core-before-v1-"
        + status + ".json"
        for status in ("prepared", "sealed", "published")
    }
    before_v2_scoped_campaign_files = {
        ".atlas-stage-creation-weyl-context-core-before-v2-"
        + status + ".json"
        for status in ("prepared", "sealed", "published")
    }
    before_v3_scoped_campaign_files = {
        ".atlas-stage-creation-weyl-context-core-before-v3-"
        + status + ".json"
        for status in ("prepared", "sealed", "published")
    }
    before_v4_scoped_campaign_files = {
        ".atlas-stage-creation-weyl-context-core-before-v4-"
        + status + ".json"
        for status in ("prepared", "sealed", "published")
    }
    after_v1_scoped_campaign_files = {
        ".atlas-stage-creation-weyl-context-core-after-v1-"
        + status + ".json"
        for status in ("prepared", "sealed", "published")
    }
    after_v2_scoped_campaign_files = {
        ".atlas-stage-creation-weyl-context-core-after-v2-"
        + status + ".json"
        for status in ("prepared", "sealed", "published")
    }
    after_v3_scoped_campaign_files = {
        ".atlas-stage-creation-weyl-context-core-after-v3-"
        + status + ".json"
        for status in ("prepared", "sealed", "published")
    }
    required_campaign_files = (
        legacy_campaign_files | v2_scoped_campaign_files
        | v3_scoped_campaign_files | v4_scoped_campaign_files
        | v5_scoped_campaign_files | v6_scoped_campaign_files
        | v7_scoped_campaign_files | v8_scoped_campaign_files
        | before_v1_scoped_campaign_files | before_v2_scoped_campaign_files
        | before_v3_scoped_campaign_files | before_v4_scoped_campaign_files
        | after_v1_scoped_campaign_files | after_v2_scoped_campaign_files
        | after_v3_scoped_campaign_files
        | predecessor_scoped_campaign_files
    )
    failure_pattern = re.compile(
        r"\.atlas-stage-creation-failure-([0-9a-f]{64})\.json\Z")
    failure_files = {
        name: failure_pattern.fullmatch(name)
        for name in campaign_files
        if name not in required_campaign_files
    }
    if (not required_campaign_files <= set(campaign_files)
            or any(match is None for match in failure_files.values())
            or not failure_files
            or len(campaign_files)
               != len(required_campaign_files) + len(failure_files)
            or any(match.group(1) != campaign_files[name]["sha256"]
                   for name, match in failure_files.items())):
        raise ValueError("stage-creation predecessor campaign state changed")
    receipt_name = STAGE_CREATION_RECEIPT
    pin_name = stage.name + "-pin.json"
    if (receipt_name not in stage_files or pin_name not in stage_files
            or stage_files[receipt_name]["sha256"]
               != record["stage_creation_sha256"]
            or stage_files[pin_name]["sha256"] != record["pin_sha256"]
            or value["stage_tree_files"] < len(stage_files)
            or value["stage_tree_bytes"] < sum(
                binding["bytes"] for binding in stage_files.values())):
        raise ValueError("stage-creation predecessor receipt binding changed")
    result = json.loads(json.dumps(value))
    result["campaign_files"] = campaign_files
    result["stage_files"] = stage_files
    return result


def _validate_creation_contract(contract, campaign):
    keys = {
        "schema", "campaign", "stage_name", "predecessor_ledger_sha256",
        "predecessor_state", "overrides_sha256", "inputs", "script", "pin",
        "lifecycle",
    }
    if not isinstance(contract, dict) or set(contract) != keys:
        raise ValueError("stage-creation contract is not exact")
    campaign = _fixed_campaign(campaign)
    inputs = contract.get("inputs")
    script = contract.get("script")
    pin = contract.get("pin")
    lifecycle = contract.get("lifecycle")
    if (contract.get("schema") != STAGE_CREATION_CONTRACT_SCHEMA
            or contract.get("campaign") != str(campaign)
            or contract.get("stage_name") != ACTIVE_STAGE_NAME
            or not isinstance(inputs, dict) or not inputs
            or any(not isinstance(name, str) or not isinstance(wanted, str)
                   or not _SHA256.fullmatch(wanted)
                   for name, wanted in inputs.items())
            or not _SHA256.fullmatch(
                contract.get("predecessor_ledger_sha256", ""))
            or not _SHA256.fullmatch(contract.get("overrides_sha256", ""))):
        raise ValueError("stage-creation contract changed")
    for name in inputs:
        _safe_relative(name)
        if name == "overrides.json" or name.startswith(".atlas-"):
            raise ValueError("stage-creation input collides with lifecycle state")
    if (not isinstance(script, dict)
            or set(script) != {"path", "sha256"}
            or script.get("path") not in inputs
            or script.get("sha256") != inputs.get(script.get("path"))):
        raise ValueError("stage-creation script descriptor changed")
    if (not isinstance(pin, dict)
            or set(pin) != {"path", "schema", "stage_creation_key"}
            or pin.get("path") != ACTIVE_STAGE_NAME + "-pin.json"
            or pin.get("schema")
               != "atlas-weyl-context-core-after-pin-v5"
            or pin.get("stage_creation_key") != "stage_creation"):
        raise ValueError("stage-creation pin descriptor changed")
    _safe_relative(pin["path"])
    lifecycle_keys = {
        "stage", "predecessor_stage", "changed_input_reasons",
        "retention_class", "retirement_condition",
    }
    if (not isinstance(lifecycle, dict) or set(lifecycle) != lifecycle_keys
            or lifecycle.get("stage") != ACTIVE_STAGE_NAME
            or not isinstance(lifecycle.get("predecessor_stage"), str)
            or not lifecycle["predecessor_stage"]
            or not isinstance(lifecycle.get("changed_input_reasons"), list)
            or not lifecycle["changed_input_reasons"]
            or any(not isinstance(value, str) or not value
                   for value in lifecycle["changed_input_reasons"])
            or not isinstance(lifecycle.get("retention_class"), str)
            or not lifecycle["retention_class"]
            or not isinstance(lifecycle.get("retirement_condition"), str)
            or not lifecycle["retirement_condition"]):
        raise ValueError("stage-creation lifecycle changed")
    predecessor = _validate_predecessor_state_descriptor(
        contract.get("predecessor_state"), campaign)
    if lifecycle["predecessor_stage"] != Path(predecessor["stage"]).name:
        raise ValueError("stage-creation lifecycle predecessor changed")
    return json.loads(json.dumps(contract))


def _tree_inventory(parent, prefix="", allowed_temporaries=frozenset()):
    files, directories = {}, set()
    try:
        names = sorted(os.listdir(parent))
    except OSError as error:
        raise ValueError("payload tree could not be listed safely") from error
    for leaf in names:
        if leaf in ("", ".", "..") or "/" in leaf:
            raise ValueError("payload tree contains an unsafe name")
        name = prefix + leaf
        try:
            value = os.stat(leaf, dir_fd=parent, follow_symlinks=False)
        except OSError as error:
            raise ValueError("payload entry could not be inspected") from error
        if name in allowed_temporaries:
            if (not stat.S_ISREG(value.st_mode) or value.st_nlink != 1
                    or stat.S_IMODE(value.st_mode) not in (0o600, 0o444)):
                raise ValueError("payload transaction residual is unsafe")
            continue
        if stat.S_ISDIR(value.st_mode):
            _require_private_writes(value, "payload directory")
            descriptor = _open_child_directory(parent, leaf)
            directories.add(name)
            try:
                child_files, child_directories = _tree_inventory(
                    descriptor, name + "/", allowed_temporaries)
                files.update(child_files)
                directories.update(child_directories)
            finally:
                os.close(descriptor)
        elif stat.S_ISREG(value.st_mode) and value.st_nlink == 1:
            files[name] = _stable_file_at(parent, leaf, mode=0o444, nlink=1)
        else:
            raise ValueError("payload entry is not an immutable regular file")
    return files, directories


def _expected_directories(names):
    result = set()
    for name in names:
        parts = PurePosixPath(name).parts
        for end in range(1, len(parts)):
            result.add("/".join(parts[:end]))
    return result


def _payload_snapshot(payload_root, contract):
    root_descriptor = override_descriptor = None
    try:
        root_descriptor = _open_directory(payload_root)
        _require_private_writes(os.fstat(root_descriptor), "payload root")
        override_descriptor = _open_child_directory(root_descriptor, "overrides")
        _require_private_writes(
            os.fstat(override_descriptor), "override payload root")
        files, directories = _tree_inventory(override_descriptor)
    finally:
        if override_descriptor is not None:
            os.close(override_descriptor)
        if root_descriptor is not None:
            os.close(root_descriptor)
    expected_names = set(contract["inputs"]) | {"overrides.json"}
    if set(files) != expected_names or directories != _expected_directories(expected_names):
        raise ValueError("override payload topology differs from the contract")
    raw_manifest = files["overrides.json"]
    if (hashlib.sha256(raw_manifest).hexdigest() != contract["overrides_sha256"]
            or _strict_json(raw_manifest, "overrides.json") != contract["inputs"]):
        raise ValueError("override payload manifest differs from the contract")
    for name, wanted in contract["inputs"].items():
        if hashlib.sha256(files[name]).hexdigest() != wanted:
            raise ValueError("override payload bytes differ from the contract")
    return files


def _write_new_file(parent, name, raw, *, mode=0o444, _fault=None):
    _publish_bytes_at(parent, name, raw, mode=mode, _fault=_fault)


def _payload_transaction_paths(payload):
    result = set()
    for name, raw in payload.items():
        parent_name, _, leaf = name.rpartition("/")
        temporary = _transaction_temp_name(leaf, raw)
        result.add(parent_name + "/" + temporary if parent_name else temporary)
    return result


def _recover_payload_link_pairs(root_descriptor, payload, fault):
    """Finish exact file-link commits before inventory requires nlink one."""
    directories = _expected_directories(payload)
    opened = {"": root_descriptor}
    try:
        for directory in sorted(
                directories, key=lambda value: (value.count("/"), value)):
            parent_name, _, leaf = directory.rpartition("/")
            parent = opened.get(parent_name)
            if parent is None or _lstat_at(parent, leaf) is None:
                opened[directory] = None
                continue
            opened[directory] = _open_child_directory(parent, leaf)
        for name, raw in sorted(payload.items()):
            parent_name, _, leaf = name.rpartition("/")
            parent = opened.get(parent_name)
            if parent is not None and _lstat_at(parent, leaf) is not None:
                _recover_complete_publication_at(
                    parent, leaf, raw, _fault=fault)
    finally:
        for name in sorted(
                (name for name, descriptor in opened.items()
                 if name and descriptor is not None),
                key=lambda value: value.count("/"), reverse=True):
            os.close(opened[name])


def _install_payload(hidden_descriptor, payload, fault):
    override_descriptor = _open_child_directory(
        hidden_descriptor, "overrides", create=True)
    try:
        _recover_payload_link_pairs(override_descriptor, payload, fault)
        allowed_temporaries = _payload_transaction_paths(payload)
        present, directories = _tree_inventory(
            override_descriptor, allowed_temporaries=allowed_temporaries)
        expected_names = set(payload)
        expected_directories = _expected_directories(expected_names)
        if (not set(present) <= expected_names
                or not directories <= expected_directories
                or any(present[name] != payload[name] for name in present)):
            raise ValueError("partial override payload is not recoverable")
        opened = {"": override_descriptor}
        try:
            for directory in sorted(expected_directories,
                                    key=lambda value: (value.count("/"), value)):
                parent_name, _, leaf = directory.rpartition("/")
                opened[directory] = _open_child_directory(
                    opened[parent_name], leaf, create=True)
                _require_private_writes(
                    os.fstat(opened[directory]), "installed payload directory")
            for index, name in enumerate(sorted(payload)):
                parent_name, _, leaf = name.rpartition("/")
                if name not in present:
                    _write_new_file(
                        opened[parent_name], leaf, payload[name], _fault=fault)
                else:
                    # This exact final may be visible from a prior link whose
                    # parent sync was interrupted.  Replay leaf durability even
                    # when no later file is written in the same directory.
                    os.fsync(opened[parent_name])
                if fault is not None:
                    fault("after_payload_file", {"index": index, "path": name})
        finally:
            for name in sorted((name for name in opened if name),
                               key=lambda value: value.count("/"), reverse=True):
                os.close(opened[name])
        complete, complete_directories = _tree_inventory(override_descriptor)
        if complete != payload or complete_directories != expected_directories:
            raise ValueError("published override payload is incomplete")
    finally:
        os.close(override_descriptor)


def _read_optional_json_at(parent, name, *, mode=0o444, recover=False,
                           _fault=None):
    if recover:
        target = _lstat_at(parent, name)
        if target is not None and target.st_nlink == 2:
            raw = _stable_file_at(parent, name, mode=mode, nlink=2)
            _recover_complete_publication_at(
                parent, name, raw, _fault=_fault)
    try:
        raw = _stable_file_at(parent, name, mode=mode, nlink=1)
    except ValueError:
        try:
            os.stat(name, dir_fd=parent, follow_symlinks=False)
        except FileNotFoundError:
            return None, None
        raise
    temporary = _transaction_temp_name(name, raw)
    try:
        os.stat(temporary, dir_fd=parent, follow_symlinks=False)
    except FileNotFoundError:
        pass
    else:
        raise ValueError("completed publication retains a transaction residual")
    # The final name may be visible from a prior link whose parent sync was
    # interrupted.  Re-reading valid immutable bytes is not durability proof;
    # replay the directory sync before any state transition or successful
    # return relies on this terminal event.
    os.fsync(parent)
    value = _strict_json(raw, name)
    if _json_bytes(value) != raw:
        raise ValueError("publication JSON is not canonical: " + name)
    return value, hashlib.sha256(raw).hexdigest()


def _require_publication_residuals(parent, allowed):
    observed = set()
    for name in os.listdir(parent):
        if name.startswith(".atlas-publish-") and name.endswith(".tmp"):
            value = os.stat(name, dir_fd=parent, follow_symlinks=False)
            if (not stat.S_ISREG(value.st_mode) or value.st_nlink != 1
                    or stat.S_IMODE(value.st_mode) not in (0o600, 0o444)):
                raise ValueError("campaign publication residual is unsafe")
            observed.add(name)
    if observed != set(allowed):
        raise ValueError("unknown campaign publication residual")


def _validate_submission_intent_history(history, expected):
    """Validate one exact unconfirmed tail against a confirmed predecessor."""
    expected_keys = {
        "stage", "script", "queue_before", "status", "max_outstanding",
        "pin_sha256", "stage_creation_sha256",
    }
    if (not isinstance(history, list) or not history
            or not isinstance(expected, dict)
            or set(expected) != expected_keys
            or history[-1] != expected
            or expected.get("status") != "SUBMISSION_INTENT_NOT_CONFIRMED"
            or not isinstance(expected.get("stage"), str)
            or not expected["stage"]
            or not isinstance(expected.get("script"), str)
            or not expected["script"]
            or type(expected.get("max_outstanding")) is not int
            or expected["max_outstanding"] != 10
            or not isinstance(expected.get("pin_sha256"), str)
            or _SHA256.fullmatch(expected["pin_sha256"]) is None
            or not isinstance(expected.get("stage_creation_sha256"), str)
            or _SHA256.fullmatch(
                expected["stage_creation_sha256"]) is None):
        raise ValueError(
            "submission intent differs from the unique unconfirmed ledger tail")
    queue = expected.get("queue_before")
    if (not isinstance(queue, list)
            or any(not isinstance(item, str) for item in queue)
            or queue_ids("\n".join(queue)) != queue
            or len(queue) >= 10):
        raise ValueError(
            "submission intent differs from the unique unconfirmed ledger tail")
    predecessor = validate_confirmed_history(history[:-1])
    if any(row["stage"] == expected["stage"] for row in predecessor):
        raise ValueError(
            "submission intent differs from the unique unconfirmed ledger tail")
    return history


def _history_and_raw(campaign_descriptor, submission_intent=None):
    raw = _stable_file_at(
        campaign_descriptor, ".atlas-progressive-submit.json", nlink=1,
        forbid_shared_write=True)
    history = _strict_json(raw, ".atlas-progressive-submit.json")
    if submission_intent is None:
        validate_confirmed_history(history)
    else:
        _validate_submission_intent_history(history, submission_intent)
        if raw != _json_bytes(history):
            raise ValueError("unconfirmed campaign ledger is not canonical")
    return history, raw


def _validate_predecessor_history(history, raw, contract):
    stage = str(Path(contract["campaign"]) / "stages" / ACTIVE_STAGE_NAME)
    matches = [index for index, row in enumerate(history)
               if row.get("stage") == stage]
    if not matches:
        predecessor_raw = raw
    elif matches == [len(history) - 1]:
        predecessor_raw = _json_bytes(history[:-1])
    else:
        raise ValueError("stage-creation ledger lineage changed")
    if hashlib.sha256(predecessor_raw).hexdigest() \
            != contract["predecessor_ledger_sha256"]:
        raise ValueError("stage-creation predecessor ledger changed")
    return bool(matches)


def _validate_stage_inventory(campaign_descriptor, history, *, hidden=None,
                              allow_final=False):
    stages = _open_child_directory(campaign_descriptor, "stages")
    try:
        _require_private_writes(os.fstat(stages), "campaign stages directory")
        expected = set()
        campaign_path = Path((Path(campaign_workspace.HPC_HOME)
                              / campaign_workspace.ACTIVE_CAMPAIGN).absolute())
        for row in history:
            value = Path(row["stage"])
            if (not value.is_absolute() or value.parent != campaign_path / "stages"
                    or not campaign_workspace.STAGE_RE.fullmatch(value.name)):
                raise ValueError("campaign ledger names a non-child stage")
            if value.name in expected:
                raise ValueError("campaign ledger repeats a stage child")
            expected.add(value.name)
        if allow_final:
            expected.add(ACTIVE_STAGE_NAME)
        if hidden is not None:
            expected.add(hidden)
        names = set(os.listdir(stages))
        if names != expected:
            raise ValueError("campaign stage sibling inventory changed")
        for name in names:
            value = os.stat(name, dir_fd=stages, follow_symlinks=False)
            if not stat.S_ISDIR(value.st_mode):
                raise ValueError("campaign stage child is not a real directory")
            _require_private_writes(value, "campaign stage child")
            descriptor = _open_child_directory(stages, name)
            os.close(descriptor)
        return stages
    except Exception:
        os.close(stages)
        raise


def _bound_file_bytes(parent, name, binding):
    raw = _stable_relative_at(
        parent, name, mode=int(binding["mode"], 8),
        nlink=binding["nlink"])
    if (len(raw) != binding["bytes"]
            or hashlib.sha256(raw).hexdigest() != binding["sha256"]):
        raise ValueError("stage-creation predecessor file changed: " + name)
    return raw


def _validate_predecessor_state(campaign_descriptor, history, contract):
    descriptor = _validate_predecessor_state_descriptor(
        contract.get("predecessor_state"), contract["campaign"])
    if descriptor is None:
        return None
    current_stage = str(
        Path(contract["campaign"]) / "stages" / ACTIVE_STAGE_NAME)
    current_rows = [index for index, row in enumerate(history)
                    if row.get("stage") == current_stage]
    predecessor_index = len(history) - 2 if current_rows else len(history) - 1
    predecessor_history = history[:-1] if current_rows else history
    observed_lineage = tuple(
        (Path(row["stage"]).name, row["job"])
        for row in predecessor_history
    )
    if (observed_lineage != EXPECTED_PREDECESSOR_LINEAGE
            or predecessor_index < 0
            or history[predecessor_index] != descriptor["record"]):
        raise ValueError("stage-creation predecessor is not the ledger tail")

    current_names = {
        STAGE_CREATION_PREPARED,
        STAGE_CREATION_SEALED,
        STAGE_CREATION_PUBLISHED,
        STAGE_CREATION_LOCK,
    }
    observed = {
        name for name in os.listdir(campaign_descriptor)
        if name.startswith(".atlas-stage-creation")
        and name not in current_names
    }
    if observed != set(descriptor["campaign_files"]):
        raise ValueError("historical stage-creation namespace changed")
    for name, binding in descriptor["campaign_files"].items():
        _bound_file_bytes(campaign_descriptor, name, binding)

    stages = _open_child_directory(campaign_descriptor, "stages")
    stage_descriptor = None
    current_stage_descriptor = None
    try:
        stage_descriptor = _open_child_directory(
            stages, Path(descriptor["stage"]).name)
        stage_stat = os.fstat(stage_descriptor)
        if ((stage_stat.st_dev, stage_stat.st_ino) != (
                descriptor["stage_device"], descriptor["stage_inode"])
                or stat.S_IMODE(stage_stat.st_mode) != 0o755):
            raise ValueError("stage-creation predecessor directory changed")
        for name, binding in descriptor["stage_files"].items():
            _bound_file_bytes(stage_descriptor, name, binding)
        observed_tree = _predecessor_tree_binding(stage_descriptor)
        expected_tree = {
            name: descriptor[name] for name in (
                "stage_tree_sha256", "stage_tree_files",
                "stage_tree_directories", "stage_tree_bytes",
            )
        }
        if observed_tree != expected_tree:
            raise ValueError("stage-creation predecessor tree changed")
        after = os.fstat(stage_descriptor)
        current_stage_descriptor = _open_child_directory(
            stages, Path(descriptor["stage"]).name)
        current = os.fstat(current_stage_descriptor)
        expected_identity = (
            descriptor["stage_device"], descriptor["stage_inode"])
        if ((after.st_dev, after.st_ino) != expected_identity
                or (current.st_dev, current.st_ino) != expected_identity
                or stat.S_IMODE(current.st_mode) != 0o755):
            raise ValueError("stage-creation predecessor changed during validation")
    finally:
        if current_stage_descriptor is not None:
            os.close(current_stage_descriptor)
        if stage_descriptor is not None:
            os.close(stage_descriptor)
        os.close(stages)
    return descriptor


def _preserved_failure_archives(contract):
    predecessor = contract.get("predecessor_state")
    files = predecessor.get("campaign_files", {}) \
        if isinstance(predecessor, dict) else {}
    result = set()
    for name, binding in files.items():
        match = re.fullmatch(
            r"\.atlas-stage-creation-failure-([0-9a-f]{64})\.json", name)
        if match is not None:
            if (not isinstance(binding, dict)
                    or match.group(1) != binding.get("sha256")):
                raise ValueError(
                    "preserved stage-creation failure archive changed")
            result.add(name)
    return result


def _creation_events(campaign_descriptor, *, recover=False, _fault=None):
    prepared, prepared_sha = _read_optional_json_at(
        campaign_descriptor, STAGE_CREATION_PREPARED,
        recover=recover, _fault=_fault)
    sealed, sealed_sha = _read_optional_json_at(
        campaign_descriptor, STAGE_CREATION_SEALED,
        recover=recover, _fault=_fault)
    published, published_sha = _read_optional_json_at(
        campaign_descriptor, STAGE_CREATION_PUBLISHED,
        recover=recover, _fault=_fault)
    if sealed is not None and prepared is None:
        raise ValueError("sealed stage creation lacks PREPARED state")
    if published is not None and (prepared is None or sealed is None):
        raise ValueError("published stage creation lacks PREPARED/SEALED state")
    return (prepared, prepared_sha, sealed, sealed_sha,
            published, published_sha)


def _prior_failure_archive_names(parent):
    try:
        names = os.listdir(parent)
    except OSError as error:
        raise ValueError("prior-failure namespace could not be listed") from error
    return {
        name for name in names
        if (name.startswith(".atlas-stage-creation-failure-")
            and name.endswith(".json"))
    }


def _validate_prior_failure_descriptor(descriptor):
    keys = {
        "schema", "temporary", "archive", "sha256", "bytes", "mode",
        "destination", "contract_sha256", "transaction",
    }
    if not isinstance(descriptor, dict) or set(descriptor) != keys:
        raise ValueError("prior stage-creation failure descriptor is not exact")
    digest = descriptor.get("sha256")
    temporary = descriptor.get("temporary")
    archive = descriptor.get("archive")
    transaction = descriptor.get("transaction")
    if (descriptor.get("schema") != STAGE_CREATION_PRIOR_FAILURE_SCHEMA
            or not isinstance(digest, str) or _SHA256.fullmatch(digest) is None
            or type(descriptor.get("bytes")) is not int
            or descriptor["bytes"] <= 0
            or descriptor.get("mode") != "0444"
            or descriptor.get("destination") != STAGE_CREATION_PREPARED
            or not isinstance(descriptor.get("contract_sha256"), str)
            or _SHA256.fullmatch(descriptor["contract_sha256"]) is None
            or not isinstance(temporary, str) or "/" in temporary
            or not temporary.startswith(".atlas-publish-")
            or not temporary.endswith(".tmp")
            or archive != ".atlas-stage-creation-failure-" + digest + ".json"
            or not isinstance(transaction, str)
            or not transaction.startswith(".atlas-stage-creation-")
            or not transaction.endswith(".txn")
            or "/" in transaction):
        raise ValueError("prior stage-creation failure descriptor changed")
    return dict(descriptor)


def _prior_failure_raw(parent, descriptor):
    temporary = descriptor["temporary"]
    archive = descriptor["archive"]
    source = _lstat_at(parent, temporary)
    saved = _lstat_at(parent, archive)
    expected_archives = {archive} if saved is not None else set()
    if _prior_failure_archive_names(parent) != expected_archives:
        raise ValueError("unknown prior stage-creation failure archive")
    # While the legacy source still exists it must be the only publication
    # residual.  Once the archive is the sole name, current-transaction
    # residuals are left for the ordinary event recovery below.
    if (source is not None
            and _publication_residual_names(parent) != {temporary}):
        raise ValueError("unknown prior stage-creation publication residual")
    if source is None and saved is None:
        raise ValueError("prior stage-creation failure evidence is missing")
    if source is not None and saved is not None:
        if source.st_nlink != 2 or saved.st_nlink != 2:
            raise ValueError("prior stage-creation failure link state changed")
        raw = _stable_file_at(parent, temporary, mode=0o444, nlink=2)
        saved_raw = _stable_file_at(parent, archive, mode=0o444, nlink=2)
        source = _lstat_at(parent, temporary)
        saved = _lstat_at(parent, archive)
        if (source is None or saved is None or raw != saved_raw
                or (source.st_dev, source.st_ino) != (saved.st_dev, saved.st_ino)
                or source.st_nlink != 2 or saved.st_nlink != 2):
            raise ValueError("prior stage-creation failure names changed")
    elif source is not None:
        raw = _stable_file_at(parent, temporary, mode=0o444, nlink=1)
    else:
        raw = _stable_file_at(parent, archive, mode=0o444, nlink=1)
    if (len(raw) != descriptor["bytes"]
            or hashlib.sha256(raw).hexdigest() != descriptor["sha256"]):
        raise ValueError("prior stage-creation failure bytes changed")
    return raw, source is not None, saved is not None


def _validate_prior_failure_event(raw, descriptor, contract, campaign):
    event = _strict_json(raw, "prior stage-creation PREPARED failure")
    if _json_bytes(event) != raw or not isinstance(event, dict):
        raise ValueError("prior stage-creation failure is not canonical JSON")
    old_contract = event.get("contract")
    old_contract = _validate_creation_contract(old_contract, campaign)
    if (old_contract == contract
            or old_contract["campaign"] != contract["campaign"]
            or old_contract["stage_name"] != contract["stage_name"]
            or old_contract["predecessor_ledger_sha256"]
               != contract["predecessor_ledger_sha256"]):
        raise ValueError("prior stage-creation contract lineage changed")
    old_sha = _json_sha(old_contract)
    transaction = ".atlas-stage-creation-" + old_sha[:24] + ".txn"
    expected = {
        "schema": STAGE_CREATION_EVENT_SCHEMA,
        "status": "PREPARED",
        "sequence": 1,
        "contract": old_contract,
        "contract_sha256": old_sha,
        "transaction": transaction,
    }
    if (event != expected or raw != _json_bytes(expected)
            or descriptor["contract_sha256"] != old_sha
            or descriptor["transaction"] != transaction
            or descriptor["temporary"] != _transaction_temp_name(
                descriptor["destination"], raw)):
        raise ValueError("prior stage-creation failure event changed")
    return event


def _exact_prior_failure_pair(parent, descriptor, raw):
    source = _exact_publication_file_at(
        parent, descriptor["temporary"], raw, nlink=2)
    archive = _exact_publication_file_at(
        parent, descriptor["archive"], raw, nlink=2)
    if (source.st_dev, source.st_ino) != (archive.st_dev, archive.st_ino):
        raise ValueError("prior stage-creation evidence names differ")


def _archive_prior_failure(parent, campaign, campaign_stat, contract,
                           descriptor, creation_lock, submission_lock, fault,
                           preserved_archives=frozenset()):
    if descriptor is None:
        if _prior_failure_archive_names(parent) != set(preserved_archives):
            raise ValueError("unregistered prior stage-creation failure archive")
        return
    descriptor = _validate_prior_failure_descriptor(descriptor)
    raw, source_exists, archive_exists = _prior_failure_raw(parent, descriptor)
    _validate_prior_failure_event(raw, descriptor, contract, campaign)
    if archive_exists and not source_exists:
        os.fsync(parent)
        _exact_publication_file_at(
            parent, descriptor["archive"], raw, nlink=1)
        return
    if not archive_exists:
        try:
            os.link(
                descriptor["temporary"], descriptor["archive"],
                src_dir_fd=parent, dst_dir_fd=parent,
                follow_symlinks=False,
            )
        except OSError as error:
            raise ValueError(
                "prior stage-creation failure archive could not be linked") \
                from error
    _exact_prior_failure_pair(parent, descriptor, raw)
    os.fsync(parent)
    _call_fault(fault, "after_prior_failure_parent_fsync", {
        "temporary": descriptor["temporary"],
        "archive": descriptor["archive"],
    })
    creation_lock.validate()
    submission_lock.validate()
    _require_directory_identity(
        campaign, campaign_stat, "active campaign")
    _exact_prior_failure_pair(parent, descriptor, raw)
    try:
        os.unlink(descriptor["temporary"], dir_fd=parent)
    except OSError as error:
        raise ValueError(
            "prior stage-creation failure residual could not be consumed") \
            from error
    _call_fault(fault, "after_prior_failure_unlink", {
        "temporary": descriptor["temporary"],
        "archive": descriptor["archive"],
    })
    os.fsync(parent)
    _exact_publication_file_at(
        parent, descriptor["archive"], raw, nlink=1)


def _validate_prepared(value, contract, transaction):
    expected = {
        "schema": STAGE_CREATION_EVENT_SCHEMA,
        "status": "PREPARED",
        "sequence": 1,
        "contract": contract,
        "contract_sha256": _json_sha(contract),
        "transaction": transaction,
    }
    if value != expected or _json_bytes(value) != _json_bytes(expected):
        raise ValueError("PREPARED stage-creation state changed")
    return expected


def _receipt_value(contract, prepared_sha, transaction, stage_stat):
    return {
        "schema": STAGE_CREATION_RECEIPT_SCHEMA,
        "status": "PUBLISHED",
        "contract": contract,
        "contract_sha256": _json_sha(contract),
        "prepared_sha256": prepared_sha,
        "stage": str(Path(contract["campaign"]) / "stages" / ACTIVE_STAGE_NAME),
        "transaction": transaction,
        "stage_device": stage_stat.st_dev,
        "stage_inode": stage_stat.st_ino,
    }


def _marker_value(contract, prepared_sha, transaction):
    return {
        "schema": STAGE_CREATION_TRANSACTION_SCHEMA,
        "status": "PREPARED",
        "contract_sha256": _json_sha(contract),
        "prepared_sha256": prepared_sha,
        "transaction": transaction,
    }


def _sealed_value(receipt, receipt_sha):
    return {
        "schema": STAGE_CREATION_EVENT_SCHEMA,
        "status": "SEALED",
        "sequence": 2,
        "contract_sha256": receipt["contract_sha256"],
        "prepared_sha256": receipt["prepared_sha256"],
        "receipt_sha256": receipt_sha,
        "stage": receipt["stage"],
        "transaction": receipt["transaction"],
        "hidden_device": receipt["stage_device"],
        "hidden_inode": receipt["stage_inode"],
    }


def _published_value(receipt, receipt_sha, sealed_sha):
    return {
        "schema": STAGE_CREATION_EVENT_SCHEMA,
        "status": "PUBLISHED",
        "sequence": 3,
        "contract_sha256": receipt["contract_sha256"],
        "prepared_sha256": receipt["prepared_sha256"],
        "sealed_sha256": sealed_sha,
        "receipt_sha256": receipt_sha,
        "stage": receipt["stage"],
        "transaction": receipt["transaction"],
        "stage_device": receipt["stage_device"],
        "stage_inode": receipt["stage_inode"],
    }


def _call_fault(fault, checkpoint, context):
    if fault is not None:
        if not callable(fault):
            raise ValueError("stage-creation fault hook must be callable")
        fault(checkpoint, context)


def _validate_published_stage(campaign_descriptor, contract, prepared,
                              prepared_sha, sealed, sealed_sha, published,
                              published_sha=None, expected_sha256=None,
                              pin_sha256=None):
    transaction = ".atlas-stage-creation-" + _json_sha(contract)[:24] + ".txn"
    _validate_prepared(prepared, contract, transaction)
    if not isinstance(sealed, dict):
        raise ValueError("SEALED stage-creation state is missing")
    stages = _open_child_directory(campaign_descriptor, "stages")
    stage_descriptor = None
    try:
        stage_descriptor = _open_child_directory(stages, ACTIVE_STAGE_NAME)
        stage_stat = os.fstat(stage_descriptor)
        _require_private_writes(stage_stat, "published stage directory")
        if stat.S_IMODE(stage_stat.st_mode) != 0o755:
            raise ValueError("published stage directory mode changed")
        receipt_raw = _stable_file_at(
            stage_descriptor, STAGE_CREATION_RECEIPT, mode=0o444, nlink=1)
        receipt_sha = hashlib.sha256(receipt_raw).hexdigest()
        receipt = _strict_json(receipt_raw, STAGE_CREATION_RECEIPT)
        expected_receipt = _receipt_value(
            contract, prepared_sha, transaction, stage_stat)
        if (receipt != expected_receipt
                or receipt_raw != _json_bytes(expected_receipt)):
            raise ValueError("stage-creation receipt changed")
        if expected_sha256 is not None and receipt_sha != expected_sha256:
            raise ValueError("stage-creation receipt SHA-256 changed")
        marker_raw = _stable_file_at(
            stage_descriptor, STAGE_CREATION_TRANSACTION, mode=0o444, nlink=1)
        marker = _strict_json(marker_raw, STAGE_CREATION_TRANSACTION)
        expected_marker = _marker_value(contract, prepared_sha, transaction)
        if marker != expected_marker or marker_raw != _json_bytes(expected_marker):
            raise ValueError("stage-creation transaction marker changed")
        expected_sealed = _sealed_value(receipt, receipt_sha)
        if (sealed != expected_sealed
                or sealed_sha != _json_sha(expected_sealed)
                or (stage_stat.st_dev, stage_stat.st_ino) != (
                    sealed["hidden_device"], sealed["hidden_inode"])):
            raise ValueError("SEALED stage-creation state changed")
        if published is not None:
            expected_published = _published_value(
                receipt, receipt_sha, sealed_sha)
            if (published != expected_published
                    or published_sha != _json_sha(expected_published)):
                raise ValueError("PUBLISHED stage-creation state changed")
        elif published_sha is not None:
            raise ValueError("PUBLISHED stage-creation SHA-256 lacks state")
    finally:
        if stage_descriptor is not None:
            os.close(stage_descriptor)
        os.close(stages)

    stage = Path(contract["campaign"]) / "stages" / ACTIVE_STAGE_NAME
    payload = _payload_snapshot(stage, contract)
    script_path = contract["script"]["path"]
    if hashlib.sha256(payload[script_path]).hexdigest() \
            != contract["script"]["sha256"]:
        raise ValueError("creation receipt names changed script bytes")
    if pin_sha256 is not None:
        if not _SHA256.fullmatch(pin_sha256):
            raise ValueError("submission pin SHA-256 is invalid")
        script_raw = _stable_relative(
            stage, script_path, mode=0o444, nlink=1)
        if hashlib.sha256(script_raw).hexdigest() != contract["script"]["sha256"]:
            raise ValueError("installed submission script differs from creation receipt")
        pin_raw = _stable_relative(
            stage, contract["pin"]["path"], mode=0o444, nlink=1)
        if hashlib.sha256(pin_raw).hexdigest() != pin_sha256:
            raise ValueError("installed submission pin differs from requested pin")
        pin = _strict_json(pin_raw, contract["pin"]["path"])
        backlink = pin.get(contract["pin"]["stage_creation_key"]) \
            if isinstance(pin, dict) else None
        if (pin.get("schema") != contract["pin"]["schema"]
                or backlink != {
                    "receipt_sha256": receipt_sha,
                    "contract_sha256": _json_sha(contract),
                }):
            raise ValueError("submission pin is not bound to stage creation")
    current_stage = _open_directory(stage)
    try:
        current_stat = os.fstat(current_stage)
        if (current_stat.st_dev, current_stat.st_ino) != (
                receipt["stage_device"], receipt["stage_inode"]):
            raise ValueError("published stage changed during validation")
    finally:
        os.close(current_stage)
    return receipt, receipt_sha


def _require_creation_event_snapshot(
        campaign_descriptor, prepared, prepared_sha, sealed, sealed_sha,
        published):
    """Bind parsed event values to their current canonical on-disk bytes."""
    (actual_prepared, actual_prepared_sha, actual_sealed, actual_sealed_sha,
     actual_published, actual_published_sha) = _creation_events(
         campaign_descriptor)
    expected_published_sha = None if published is None else _json_sha(published)
    if ((actual_prepared, actual_prepared_sha) != (prepared, prepared_sha)
            or (actual_sealed, actual_sealed_sha) != (sealed, sealed_sha)
            or (actual_published, actual_published_sha) != (
                published, expected_published_sha)):
        raise ValueError("stage-creation event snapshot changed")
    return actual_published_sha


def _revalidate_publish_boundary(
        campaign_descriptor, campaign, campaign_stat, raw_history, contract,
        prepared, prepared_sha, sealed, sealed_sha, published_value,
        receipt_sha, creation_lock, submission_lock):
    """Recheck every identity immediately before committing PUBLISHED."""
    creation_lock.validate()
    submission_lock.validate()
    _require_directory_identity(campaign, campaign_stat, "active campaign")
    _require_creation_event_snapshot(
        campaign_descriptor, prepared, prepared_sha, sealed, sealed_sha, None)
    published_raw = _json_bytes(published_value)
    published_temporary = _transaction_temp_name(
        STAGE_CREATION_PUBLISHED, published_raw)
    residuals = _publication_residual_names(campaign_descriptor)
    _require_publication_residuals(
        campaign_descriptor,
        {published_temporary} if published_temporary in residuals else set())
    current_history, current_raw_history = _history_and_raw(
        campaign_descriptor)
    if current_raw_history != raw_history:
        raise ValueError("campaign ledger changed before PUBLISHED")
    _validate_predecessor_history(
        current_history, current_raw_history, contract)
    inventory = _validate_stage_inventory(
        campaign_descriptor, current_history, allow_final=True)
    os.close(inventory)
    _validate_predecessor_state(
        campaign_descriptor, current_history, contract)
    receipt, observed_sha = _validate_published_stage(
        campaign_descriptor, contract, prepared, prepared_sha,
        sealed, sealed_sha, None, expected_sha256=receipt_sha)
    if observed_sha != receipt_sha:
        raise ValueError("stage-creation receipt changed before PUBLISHED")
    creation_lock.validate()
    submission_lock.validate()
    _require_directory_identity(campaign, campaign_stat, "active campaign")
    stage = Path(contract["campaign"]) / "stages" / ACTIVE_STAGE_NAME
    current = _open_directory(stage)
    try:
        value = os.fstat(current)
        if (value.st_dev, value.st_ino) != (
                receipt["stage_device"], receipt["stage_inode"]):
            raise ValueError("fixed stage target changed before PUBLISHED")
    finally:
        os.close(current)
    return receipt


def _validate_published_commit(
        campaign_descriptor, campaign, campaign_stat, raw_history, contract,
        prepared, prepared_sha, sealed, sealed_sha, published_value,
        receipt_sha, creation_lock, submission_lock):
    """Re-read and validate the exact committed terminal event and stage."""
    creation_lock.validate()
    submission_lock.validate()
    _require_directory_identity(campaign, campaign_stat, "active campaign")
    published_sha = _require_creation_event_snapshot(
        campaign_descriptor, prepared, prepared_sha, sealed, sealed_sha,
        published_value)
    _require_publication_residuals(campaign_descriptor, set())
    current_history, current_raw_history = _history_and_raw(
        campaign_descriptor)
    if current_raw_history != raw_history:
        raise ValueError("campaign ledger changed after PUBLISHED")
    _validate_predecessor_history(
        current_history, current_raw_history, contract)
    inventory = _validate_stage_inventory(
        campaign_descriptor, current_history, allow_final=True)
    os.close(inventory)
    _validate_predecessor_state(
        campaign_descriptor, current_history, contract)
    receipt, observed_sha = _validate_published_stage(
        campaign_descriptor, contract, prepared, prepared_sha,
        sealed, sealed_sha, published_value,
        published_sha=published_sha, expected_sha256=receipt_sha)
    if observed_sha != receipt_sha:
        raise ValueError("stage-creation receipt changed after PUBLISHED")
    creation_lock.validate()
    submission_lock.validate()
    _require_directory_identity(campaign, campaign_stat, "active campaign")
    stage = Path(contract["campaign"]) / "stages" / ACTIVE_STAGE_NAME
    current = _open_directory(stage)
    try:
        value = os.fstat(current)
        if (value.st_dev, value.st_ino) != (
                receipt["stage_device"], receipt["stage_inode"]):
            raise ValueError("fixed stage target changed after PUBLISHED")
    finally:
        os.close(current)
    return receipt


def validate_stage_creation(root, expected_sha256=None, expected_contract=None,
                            *, pin_sha256=None, submission_intent=None):
    """Validate the fixed stage publication before any scheduler contact."""
    if expected_sha256 is not None and not _SHA256.fullmatch(expected_sha256):
        raise ValueError("stage-creation receipt SHA-256 is invalid")
    campaign = _fixed_campaign(Path(root).parent.parent)
    expected_root = campaign / "stages" / ACTIVE_STAGE_NAME
    if Path(root) != expected_root:
        raise ValueError("submission root is not the fixed created stage")
    campaign_descriptor = _open_directory(campaign)
    campaign_stat = os.fstat(campaign_descriptor)
    try:
        history, raw_history = _history_and_raw(
            campaign_descriptor, submission_intent=submission_intent)
        (prepared, prepared_sha, sealed, sealed_sha,
         published, published_sha) = _creation_events(campaign_descriptor)
        if prepared is None or sealed is None or published is None:
            raise ValueError("stage creation is not PUBLISHED")
        _require_publication_residuals(campaign_descriptor, set())
        contract = prepared.get("contract") if isinstance(prepared, dict) else None
        contract = _validate_creation_contract(contract, campaign)
        if expected_contract is not None and contract != expected_contract:
            raise ValueError("stage-creation contract differs from the caller")
        _validate_predecessor_history(history, raw_history, contract)
        _validate_predecessor_state(campaign_descriptor, history, contract)
        stages = _validate_stage_inventory(
            campaign_descriptor, history, allow_final=True)
        os.close(stages)
        receipt, receipt_sha = _validate_published_stage(
            campaign_descriptor, contract, prepared, prepared_sha,
            sealed, sealed_sha, published,
            published_sha=published_sha,
            expected_sha256=expected_sha256, pin_sha256=pin_sha256)
        if submission_intent is not None:
            stage_descriptor = _open_directory(expected_root)
            try:
                intent_raw = _stable_file_at(
                    stage_descriptor, "submission-intent.json",
                    mode=0o444, nlink=1)
            finally:
                os.close(stage_descriptor)
            if (intent_raw != _json_bytes(submission_intent)
                    or _strict_json(
                        intent_raw, "submission-intent.json")
                       != submission_intent):
                raise ValueError(
                    "submission intent differs from the unique unconfirmed ledger tail")
        current = _open_directory(campaign)
        try:
            value = os.fstat(current)
            if (value.st_dev, value.st_ino) != (
                    campaign_stat.st_dev, campaign_stat.st_ino):
                raise ValueError("active campaign changed during validation")
        finally:
            os.close(current)
        return receipt
    finally:
        os.close(campaign_descriptor)


def create_fixed_stage(campaign, payload_root, contract, *, prior_failure=None,
                       _fault=None):
    """Create or recover the fixed stage with PUBLISHED as its commit point."""
    campaign = _fixed_campaign(campaign)
    contract = _validate_creation_contract(contract, campaign)
    if prior_failure is not None:
        raise ValueError("prior stage-creation migration is closed for BEFORE v4")
    payload = _payload_snapshot(payload_root, contract)
    contract_sha = _json_sha(contract)
    transaction = ".atlas-stage-creation-" + contract_sha[:24] + ".txn"
    stage = campaign / "stages" / ACTIVE_STAGE_NAME
    with exclusive_lock(campaign, STAGE_CREATION_LOCK) as creation_lock:
        with exclusive_lock(
                campaign, ".atlas-progressive-submit.lock") as submission_lock:
            campaign_descriptor = _open_directory(campaign)
            campaign_stat = os.fstat(campaign_descriptor)
            _require_private_writes(campaign_stat, "active campaign")
            try:
                history, raw_history = _history_and_raw(campaign_descriptor)
                submitted = _validate_predecessor_history(
                    history, raw_history, contract)
                _validate_predecessor_state(
                    campaign_descriptor, history, contract)
                _archive_prior_failure(
                    campaign_descriptor, campaign, campaign_stat, contract,
                    prior_failure, creation_lock, submission_lock, _fault,
                    preserved_archives=_preserved_failure_archives(contract))
                (prepared, prepared_sha, sealed, sealed_sha,
                 published, published_sha) = _creation_events(
                    campaign_descriptor, recover=True, _fault=_fault)
                if prepared is None:
                    if sealed is not None or published is not None or submitted:
                        raise ValueError("stage creation has impossible initial state")
                    stages = _validate_stage_inventory(campaign_descriptor, history)
                    os.close(stages)
                    prepared = {
                        "schema": STAGE_CREATION_EVENT_SCHEMA,
                        "status": "PREPARED",
                        "sequence": 1,
                        "contract": contract,
                        "contract_sha256": contract_sha,
                        "transaction": transaction,
                    }
                    prepared_temporary = _transaction_temp_name(
                        STAGE_CREATION_PREPARED, _json_bytes(prepared))
                    residuals = {prepared_temporary} \
                        if prepared_temporary in os.listdir(
                            campaign_descriptor) else set()
                    _require_publication_residuals(
                        campaign_descriptor, residuals)
                    _publish_json_at(
                        campaign_descriptor, STAGE_CREATION_PREPARED, prepared,
                        _fault=_fault)
                    prepared_sha = _json_sha(prepared)
                    _call_fault(_fault, "after_prepared", {"stage": str(stage)})
                else:
                    _validate_prepared(prepared, contract, transaction)

                if published is not None:
                    _require_publication_residuals(campaign_descriptor, set())
                    receipt, receipt_sha = _validate_published_stage(
                        campaign_descriptor, contract, prepared, prepared_sha,
                        sealed, sealed_sha, published,
                        published_sha=published_sha)
                    _validate_published_commit(
                        campaign_descriptor, campaign, campaign_stat,
                        raw_history, contract, prepared, prepared_sha,
                        sealed, sealed_sha, published, receipt_sha,
                        creation_lock, submission_lock)
                    return stage, receipt_sha
                if submitted:
                    raise ValueError(
                        "submitted stage lacks complete creation publication")

                stages = _open_child_directory(campaign_descriptor, "stages")
                try:
                    _require_private_writes(
                        os.fstat(stages), "campaign stages directory")
                    names = set(os.listdir(stages))
                    if transaction in names:
                        raise ValueError(
                            "legacy hidden stage transaction requires reconciliation")
                    final_exists = ACTIVE_STAGE_NAME in names
                    inventory = _validate_stage_inventory(
                        campaign_descriptor, history, allow_final=final_exists)
                    os.close(inventory)

                    if final_exists and sealed is not None:
                        receipt, receipt_sha = _validate_published_stage(
                            campaign_descriptor, contract, prepared, prepared_sha,
                            sealed, sealed_sha, None)
                        os.fsync(stages)
                        published_value = _published_value(
                            receipt, receipt_sha, sealed_sha)
                        published_temporary = _transaction_temp_name(
                            STAGE_CREATION_PUBLISHED,
                            _json_bytes(published_value))
                        _require_publication_residuals(
                            campaign_descriptor, {published_temporary}
                            if published_temporary in os.listdir(
                                campaign_descriptor) else set())
                        _call_fault(_fault, "before_published", {
                            "stage": str(stage), "receipt_sha256": receipt_sha,
                        })
                        _revalidate_publish_boundary(
                            campaign_descriptor, campaign, campaign_stat,
                            raw_history, contract, prepared, prepared_sha,
                            sealed, sealed_sha, published_value, receipt_sha,
                            creation_lock, submission_lock)
                        _publish_json_at(
                            campaign_descriptor, STAGE_CREATION_PUBLISHED,
                            published_value, _fault=_fault)
                        _call_fault(_fault, "after_published", {
                            "stage": str(stage), "receipt_sha256": receipt_sha,
                        })
                        _validate_published_commit(
                            campaign_descriptor, campaign, campaign_stat,
                            raw_history, contract, prepared, prepared_sha,
                            sealed, sealed_sha, published_value, receipt_sha,
                            creation_lock, submission_lock)
                        return stage, receipt_sha

                    if not final_exists:
                        if sealed is not None:
                            raise ValueError(
                                "SEALED state has no matching stage directory")
                        _require_publication_residuals(
                            campaign_descriptor, set())
                        _call_fault(_fault, "before_target_claim", {
                            "stage": str(stage), "transaction": transaction,
                        })
                        try:
                            os.mkdir(ACTIVE_STAGE_NAME, 0o700, dir_fd=stages)
                        except FileExistsError as error:
                            raise ValueError(
                                "fixed stage target claim was concurrently occupied") \
                                from error
                        os.fsync(stages)
                        _call_fault(_fault, "after_target_claim", {
                            "stage": str(stage), "transaction": transaction,
                        })

                    target = _open_child_directory(stages, ACTIVE_STAGE_NAME)
                    try:
                        target_stat = os.fstat(target)
                        _require_private_writes(
                            target_stat, "fixed stage target")
                        if stat.S_IMODE(target_stat.st_mode) not in (0o700, 0o755):
                            raise ValueError("fixed stage target mode changed")
                        marker_value = _marker_value(
                            contract, prepared_sha, transaction)
                        marker_raw = _json_bytes(marker_value)
                        marker_temporary = _transaction_temp_name(
                            STAGE_CREATION_TRANSACTION, marker_raw)
                        marker, marker_sha = _read_optional_json_at(
                            target, STAGE_CREATION_TRANSACTION,
                            recover=True, _fault=_fault)
                        if marker is None:
                            contents = set(os.listdir(target))
                            if final_exists:
                                if contents != {marker_temporary}:
                                    raise ValueError(
                                        "fixed stage target without marker is ambiguous")
                                _exact_publication_file_at(
                                    target, marker_temporary, marker_raw, nlink=1)
                            elif contents:
                                raise ValueError(
                                    "new fixed stage target contains unexpected state")
                            _publish_json_at(
                                target, STAGE_CREATION_TRANSACTION,
                                marker_value, _fault=_fault)
                            _call_fault(_fault, "after_marker", {
                                "stage": str(stage),
                                "transaction": transaction,
                            })
                        elif (marker != marker_value
                              or marker_sha != _json_sha(marker_value)):
                            raise ValueError("fixed stage target marker changed")

                        receipt = _receipt_value(
                            contract, prepared_sha, transaction, target_stat)
                        receipt_temporary = _transaction_temp_name(
                            STAGE_CREATION_RECEIPT, _json_bytes(receipt))
                        allowed = {
                            STAGE_CREATION_TRANSACTION, STAGE_CREATION_RECEIPT,
                            receipt_temporary, "overrides",
                        }
                        if not set(os.listdir(target)) <= allowed:
                            raise ValueError("fixed stage target contains an extra entry")
                        _install_payload(target, payload, _fault)
                        _call_fault(_fault, "after_payload", {
                            "stage": str(stage), "files": len(payload),
                        })
                        existing_receipt, existing_sha = _read_optional_json_at(
                            target, STAGE_CREATION_RECEIPT,
                            recover=True, _fault=_fault)
                        if existing_receipt is None:
                            _publish_json_at(
                                target, STAGE_CREATION_RECEIPT, receipt,
                                _fault=_fault)
                            receipt_sha = _json_sha(receipt)
                            _call_fault(_fault, "after_receipt", {
                                "stage": str(stage),
                                "receipt_sha256": receipt_sha,
                            })
                        elif (existing_receipt == receipt
                              and existing_sha == _json_sha(receipt)):
                            receipt_sha = existing_sha
                        else:
                            raise ValueError("fixed stage target receipt changed")
                        os.fchmod(target, 0o755)
                        os.fsync(target)
                        target_after = os.fstat(target)
                        if ((target_after.st_dev, target_after.st_ino) != (
                                receipt["stage_device"], receipt["stage_inode"])
                                or stat.S_IMODE(target_after.st_mode) != 0o755):
                            raise ValueError(
                                "fixed stage target inode changed before seal")
                    finally:
                        os.close(target)

                    _require_directory_identity(
                        stage, target_stat, "fixed stage target")
                    inventory = _validate_stage_inventory(
                        campaign_descriptor, history, allow_final=True)
                    os.close(inventory)
                    sealed_value = _sealed_value(receipt, receipt_sha)
                    sealed_temporary = _transaction_temp_name(
                        STAGE_CREATION_SEALED, _json_bytes(sealed_value))
                    if sealed is None:
                        residuals = {sealed_temporary} \
                            if sealed_temporary in os.listdir(
                                campaign_descriptor) else set()
                        _require_publication_residuals(
                            campaign_descriptor, residuals)
                        _publish_json_at(
                            campaign_descriptor, STAGE_CREATION_SEALED,
                            sealed_value, _fault=_fault)
                        sealed = sealed_value
                        sealed_sha = _json_sha(sealed_value)
                        _call_fault(_fault, "after_sealed", {
                            "stage": str(stage),
                            "receipt_sha256": receipt_sha,
                            "hidden_device": receipt["stage_device"],
                            "hidden_inode": receipt["stage_inode"],
                        })
                    elif sealed != sealed_value:
                        raise ValueError("SEALED stage-creation state changed")
                    elif sealed_sha != _json_sha(sealed_value):
                        raise ValueError("SEALED stage-creation SHA-256 changed")
                    else:
                        _require_publication_residuals(
                            campaign_descriptor, set())
                    creation_lock.validate()
                    submission_lock.validate()
                    _require_directory_identity(
                        campaign, campaign_stat, "active campaign")
                    _require_directory_identity(
                        stage, target_stat, "fixed stage target")
                    # The target name was claimed before construction.  This
                    # parent sync is replayed before the campaign PUBLISHED
                    # event becomes the sole consumer-visible commit point.
                    os.fsync(stages)
                finally:
                    os.close(stages)

                inventory = _validate_stage_inventory(
                    campaign_descriptor, history, allow_final=True)
                os.close(inventory)
                receipt, receipt_sha = _validate_published_stage(
                    campaign_descriptor, contract, prepared, prepared_sha,
                    sealed, sealed_sha, None)
                published_value = _published_value(
                    receipt, receipt_sha, sealed_sha)
                published_temporary = _transaction_temp_name(
                    STAGE_CREATION_PUBLISHED, _json_bytes(published_value))
                residuals = {published_temporary} \
                    if published_temporary in os.listdir(
                        campaign_descriptor) else set()
                _require_publication_residuals(
                    campaign_descriptor, residuals)
                _call_fault(_fault, "before_published", {
                    "stage": str(stage), "receipt_sha256": receipt_sha,
                })
                _revalidate_publish_boundary(
                    campaign_descriptor, campaign, campaign_stat, raw_history,
                    contract, prepared, prepared_sha, sealed, sealed_sha,
                    published_value, receipt_sha,
                    creation_lock, submission_lock)
                _publish_json_at(
                    campaign_descriptor, STAGE_CREATION_PUBLISHED,
                    published_value, _fault=_fault)
                _call_fault(_fault, "after_published", {
                    "stage": str(stage), "receipt_sha256": receipt_sha,
                })
                _validate_published_commit(
                    campaign_descriptor, campaign, campaign_stat, raw_history,
                    contract, prepared, prepared_sha, sealed, sealed_sha,
                    published_value, receipt_sha,
                    creation_lock, submission_lock)
                return stage, receipt_sha
            finally:
                os.close(campaign_descriptor)


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


def scheduler_user():
    """Return the scheduler account bound to this process identity."""
    try:
        user = pwd.getpwuid(os.geteuid()).pw_name
    except (KeyError, OSError) as error:
        raise ValueError("scheduler user could not be resolved") from error
    if (not isinstance(user, str)
            or re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9_.-]*", user) is None):
        raise ValueError("scheduler user is invalid")
    return user


def validate_confirmed_history(history):
    """Require one exact, fully confirmed, collision-free campaign ledger."""
    legacy_keys = {
        "stage", "script", "queue_before", "status", "max_outstanding", "job",
    }
    pinned_keys = legacy_keys | {"pin_sha256"}
    created_keys = pinned_keys | {"stage_creation_sha256"}
    if not isinstance(history, list):
        raise ValueError("unresolved submission intent; inspect scheduler before retry")
    for index, row in enumerate(history):
        queue = row.get("queue_before") if isinstance(row, dict) else None
        keys = set(row) if isinstance(row, dict) else set()
        if (not isinstance(row, dict)
                or keys not in (legacy_keys, pinned_keys, created_keys)
                or (index >= 7 and keys != created_keys)
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
                         or _SHA256.fullmatch(row["pin_sha256"]) is None))
                or ("stage_creation_sha256" in row
                    and (not isinstance(row["stage_creation_sha256"], str)
                         or _SHA256.fullmatch(
                             row["stage_creation_sha256"]) is None))):
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
        _require_private_writes(os.fstat(current), "submission stage directory")
        directory_descriptors.append(current)
        for part in requested.parts[:-1]:
            current = os.open(part, os.O_RDONLY | directory_flag | nofollow, dir_fd=current)
            _require_private_writes(
                os.fstat(current), "submission script parent directory")
            directory_descriptors.append(current)
        descriptor = os.open(requested.parts[-1], os.O_RDONLY | nofollow, dir_fd=current)
        opened = os.fstat(descriptor)
        if (not stat.S_ISREG(opened.st_mode) or opened.st_nlink != 1
                or stat.S_IMODE(opened.st_mode) != 0o444):
            raise ValueError("job script must be a frozen single-link regular file")
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
            os.fchmod(handle.fileno(), 0o444)
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


def _claim_submission_intent(root, record):
    """Win the one durable submission right without replacing prior state."""
    directory = _open_directory(root)
    descriptor = None
    raw = _json_bytes(record)
    try:
        _require_private_writes(
            os.fstat(directory), "submission stage directory")
        try:
            descriptor = os.open(
                "submission-intent.json",
                os.O_WRONLY | os.O_CREAT | os.O_EXCL
                | getattr(os, "O_CLOEXEC", 0)
                | getattr(os, "O_NOFOLLOW", 0),
                0o600,
                dir_fd=directory,
            )
        except FileExistsError as error:
            raise ValueError(
                "stage has an existing submission intent") from error
        remaining = memoryview(raw)
        while remaining:
            written = os.write(descriptor, remaining)
            if written <= 0:
                raise ValueError("short submission-intent write")
            remaining = remaining[written:]
        os.fchmod(descriptor, 0o444)
        os.fsync(descriptor)
        opened = os.fstat(descriptor)
        if (not stat.S_ISREG(opened.st_mode) or opened.st_nlink != 1
                or stat.S_IMODE(opened.st_mode) != 0o444):
            raise ValueError("submission intent inode changed")
        os.close(descriptor)
        descriptor = None
        if _stable_file_at(
                directory, "submission-intent.json", mode=0o444,
                nlink=1) != raw:
            raise ValueError("submission intent changed during publication")
        os.fsync(directory)
    finally:
        if descriptor is not None:
            os.close(descriptor)
        os.close(directory)


def submit_one(root, script, env, *, pin_sha256=None,
               stage_creation_sha256=None):
    root = Path(root).resolve()
    if not isinstance(pin_sha256, str) or _SHA256.fullmatch(pin_sha256) is None:
        raise ValueError("submission pin SHA-256 is invalid")
    if (not isinstance(stage_creation_sha256, str)
            or _SHA256.fullmatch(stage_creation_sha256) is None):
        raise ValueError("stage-creation receipt SHA-256 is invalid")
    # This is the common fail-closed boundary: historical top-level stagers
    # remain readable evidence, but they can no longer submit new jobs.
    scope = submission_scope(root)
    creation_receipt = validate_stage_creation(
        root, stage_creation_sha256, pin_sha256=pin_sha256)
    requested_script, script_contents = stage_script(root, script)
    script_name = str(requested_script)
    creation_script = creation_receipt["contract"]["script"]
    if (script_name != creation_script["path"]
            or hashlib.sha256(script_contents.encode("utf-8")).hexdigest()
               != creation_script["sha256"]):
        raise ValueError("submission script snapshot differs from stage creation")
    one_job_script(script_contents)
    ledger = scope / ".atlas-progressive-submit.json"
    with exclusive_lock(
            scope, ".atlas-progressive-submit.lock") as submission_lock:
        scope_descriptor = _open_directory(scope)
        scope_stat = os.fstat(scope_descriptor)
        _require_private_writes(scope_stat, "active campaign")
        try:
            validate_stage_creation(
                root, stage_creation_sha256, pin_sha256=pin_sha256)
            history, _ = _history_and_raw(scope_descriptor)
        finally:
            os.close(scope_descriptor)
        if any(r.get("stage") == str(root) for r in history):
            raise ValueError("this stage was already submitted")
        if os.path.lexists(root / "submission-intent.json"):
            read_json_file(root / "submission-intent.json")
            raise ValueError("stage has an existing submission intent")
        raw = subprocess.check_output(
            ["squeue", "-h", "-r", "-u", scheduler_user(), "-o", "%i"],
            text=True, timeout=20)
        active = capacity(raw)
        submission_lock.validate()
        _require_directory_identity(scope, scope_stat, "active campaign")
        validate_stage_creation(
            root, stage_creation_sha256, pin_sha256=pin_sha256)
        record = dict(stage=str(root), script=script_name, queue_before=active,
                      status="SUBMISSION_INTENT_NOT_CONFIRMED", max_outstanding=10,
                      pin_sha256=pin_sha256,
                      stage_creation_sha256=stage_creation_sha256)
        # This no-replace publication is the unique submission gate.  Only its
        # winner may append the ledger or contact sbatch.
        _claim_submission_intent(root, record)
        history.append(record)
        save(ledger, history)
        clean_env = {k: v for k, v in env.items() if not k.startswith("SBATCH_")}
        # Submit the exact bytes checked above.  Passing the stage path here
        # would reopen it after squeue/ledger writes and reintroduce a race.
        submission_lock.validate()
        _require_directory_identity(scope, scope_stat, "active campaign")
        creation_receipt = validate_stage_creation(
            root, stage_creation_sha256, pin_sha256=pin_sha256,
            submission_intent=record)
        stage_descriptor = _open_directory(root)
        try:
            stage_stat = _require_private_writes(
                os.fstat(stage_descriptor), "submission stage directory")
            if (stage_stat.st_dev, stage_stat.st_ino) != (
                    creation_receipt["stage_device"],
                    creation_receipt["stage_inode"]):
                raise ValueError(
                    "submission stage changed before descriptor binding")
            descriptor_cwd = "/proc/self/fd/" + str(stage_descriptor)
            try:
                descriptor_stat = os.stat(descriptor_cwd)
            except OSError as error:
                raise ValueError(
                    "descriptor-bound submission cwd is unavailable") from error
            if (descriptor_stat.st_dev, descriptor_stat.st_ino) != (
                    stage_stat.st_dev, stage_stat.st_ino):
                raise ValueError("descriptor-bound submission cwd changed")
            result = subprocess.check_output(
                ["sbatch", "--parsable", "--export=ALL"],
                cwd=descriptor_cwd,
                pass_fds=(stage_descriptor,),
                env=clean_env,
                input=script_contents,
                text=True,
                timeout=25,
            ).strip()
            try:
                submission_lock.validate()
                _require_directory_identity(
                    scope, scope_stat, "active campaign")
                _require_directory_identity(
                    root, stage_stat, "submission stage")
            except ValueError as error:
                raise ValueError(
                    "uncertain sbatch response; reconcile durable intent"
                ) from error
        finally:
            os.close(stage_descriptor)
        if not re.fullmatch(r"[0-9]+(?:;[A-Za-z0-9_.-]+)?", result):
            raise ValueError("uncertain sbatch response; reconcile durable intent")
        record.update(job=result.split(";")[0], status="SUBMITTED")
        validate_confirmed_history(history)
        save(ledger, history)
        save(root / "submission-intent.json", record)
        return record

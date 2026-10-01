"""Read-only, fail-closed validation of the local Git worktree registry.

PRIMARY intentionally pins path and branch but not HEAD: committing this registry
advances PRIMARY HEAD. LEGACY worktrees remain exact-HEAD pinned. Until a
receipt-bound creator and retirement transaction are implemented, the complete
audited LEGACY set is frozen, ACTIVE is forbidden, and ``precreate`` always
fails before reading state.

Normal checks sample the registry, Git inventory and managed filesystem
namespace twice. This narrows but cannot eliminate races after the process
returns. Git output limits are checked only after ``subprocess.run`` has
buffered each stream, so they do not cap allocation.
"""

import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import stat
import subprocess
import sys


REPOSITORY_ROOT = Path(__file__).resolve().parents[1]
REGISTRY_PATH = REPOSITORY_ROOT / "docs" / "worktree_registry.json"
REGISTRY_SCHEMA = "atlas-local-worktree-registry-v1"
MAX_REGISTRY_BYTES = 1024 * 1024
MAX_GIT_OUTPUT_BYTES = 4 * 1024 * 1024
MAX_GIT_LINK_BYTES = 4096
GIT_TIMEOUT_SECONDS = 15
GIT_EXECUTABLE = "/usr/bin/git"
MANAGED_WORKTREE_PARENT = Path("/home/hoxide/mycodes")
MANAGED_WORKTREE_PREFIX = "atlas"
SANITIZED_GIT_ENV = {
    "HOME": "/nonexistent",
    "XDG_CONFIG_HOME": "/nonexistent",
    "LANG": "C",
    "LC_ALL": "C",
    "GIT_CONFIG_NOSYSTEM": "1",
    "GIT_CONFIG_GLOBAL": "/dev/null",
    "GIT_OPTIONAL_LOCKS": "0",
    "GIT_NO_LAZY_FETCH": "1",
    "GIT_TERMINAL_PROMPT": "0",
}
SHA1 = re.compile(r"[0-9a-f]{40}\Z")
ROOT_KEYS = frozenset({"schema", "max_active_task_worktrees", "worktrees"})
PRIMARY_ENTRY_KEYS = frozenset({"path", "branch", "status"})
PINNED_ENTRY_KEYS = PRIMARY_ENTRY_KEYS | frozenset({"head"})
ACTIVE_ENTRY_KEYS = PINNED_ENTRY_KEYS | frozenset(
    {"starting_commit", "owner", "purpose", "retirement_condition"}
)
STATUSES = frozenset({"PRIMARY", "LEGACY", "ACTIVE"})
MUTATING_WORKTREE_SUBCOMMANDS = frozenset(
    {"add", "lock", "move", "prune", "remove", "repair", "unlock"}
)
FROZEN_LEGACY_IDENTITIES = frozenset(
    {
        (
            "/home/hoxide/mycodes/atlas-coroot-coordinate-repair",
            "a2af0e79061ffe5a627ddf62b7e284ffd5b8cb7e",
            "refs/heads/codex/coroot-coordinate-repair",
        ),
        (
            "/home/hoxide/mycodes/atlas-cycle-foundation-repair",
            "5e567112bec8658b83d52794af4aa5f643737f7b",
            "refs/heads/codex/cycle-foundation-repair",
        ),
        (
            "/home/hoxide/mycodes/atlas-deform-bucket-audit",
            "2216d5ca515b410ef235c39040cc36c8ea1056cd",
            "refs/heads/codex/deform-bucket-audit",
        ),
        (
            "/home/hoxide/mycodes/atlas-deform-groups-parallel",
            "e49c0a2aea41bf58a7d2a5fa2c08b0ad2628d3fb",
            "refs/heads/codex/deform-groups-parallel",
        ),
        (
            "/home/hoxide/mycodes/atlas-deform-primitive-groups",
            "35f927abae1fcabc4aa78e871f1fcce6bc82cde5",
            "refs/heads/codex/deform-primitive-groups",
        ),
        (
            "/home/hoxide/mycodes/atlas-deform-target-audit",
            "7443757405cdd2218ff9c1afec46775dff4f3f64",
            "refs/heads/codex/deform-target-audit",
        ),
        (
            "/home/hoxide/mycodes/atlas-dispatch-s1-wt",
            "d2bc0e1426e0d65544233d9a0f3f1eb47651f43b",
            "refs/heads/codex/dispatch-s1",
        ),
        (
            "/home/hoxide/mycodes/atlas-dispatch-s2-wt",
            "bd9d61a698cd7eb1e555e02e2d8fe5a878d09f3e",
            "refs/heads/codex/dispatch-s2",
        ),
        (
            "/home/hoxide/mycodes/atlas-kl-exact-reserve",
            "e38b365eed9d2aea051a62cea6f4cbbe3750e0c4",
            "refs/heads/codex/kl-exact-reserve",
        ),
        (
            "/home/hoxide/mycodes/atlas-kl-prefix-storage",
            "ea9f7092f1c9176cec04b9a2d5b4f5bacb4bfe26",
            "refs/heads/codex/kl-prefix-storage",
        ),
        (
            "/home/hoxide/mycodes/atlas-kl-retained-memory",
            "8ce8e9f1e5806be0d7df88d92da06cf953aab33a",
            "refs/heads/codex/kl-retained-memory",
        ),
        (
            "/home/hoxide/mycodes/atlas-mu-fiber-audit",
            "208a9f2e20a1034824cacedb3e1d445d29727211",
            "refs/heads/codex/mu-fiber-audit",
        ),
        (
            "/home/hoxide/mycodes/atlas-mu-fiber-prototype",
            "b4714c3b3d3fee9b800d1f16f9700af867f83d19",
            "refs/heads/codex/mu-fiber-prototype",
        ),
        (
            "/home/hoxide/mycodes/atlas-pfor-wt",
            "a976445b7e9ab96fc422195189809611232c4834",
            "refs/heads/agent-dispatch-s0",
        ),
        (
            "/home/hoxide/mycodes/atlas-recursion-operands-audit",
            "f048d47f73238c032a1d833e546f07311d03a9e6",
            "refs/heads/codex/recursion-operands-audit",
        ),
        (
            "/home/hoxide/mycodes/atlas-recursion-operands-reuse",
            "a27b6f8d56fb84b84c8be67a62a81b9e86b50136",
            "refs/heads/codex/recursion-operands-reuse",
        ),
        (
            "/home/hoxide/mycodes/atlas-rust-avopt",
            "5a2c17bdc08529496eed3ce6bc50b80520e9ae9f",
            "refs/heads/agent-evalmemo",
        ),
        (
            "/home/hoxide/mycodes/atlas-rust-avopt-shardunit",
            "7625055c5ac3dbfa6c2f438cf75b9999509fa467",
            "refs/heads/agent-shardunit",
        ),
        (
            "/home/hoxide/mycodes/atlas-torus-shape-repair",
            "b7cd5646ff6bf46c22c9b084f001efb9bd5f52f7",
            "refs/heads/codex/torus-shape-repair",
        ),
        (
            "/home/hoxide/mycodes/atlas-trace-lines-wt",
            "5e8808009b51ee0c67309993988b398eaa4c8050",
            "refs/heads/codex/trace-source-lines",
        ),
        (
            "/home/hoxide/mycodes/atlas-unitary-d5-wt",
            "b4bb996253438444a25dc4e422841490f29d9b43",
            "refs/heads/codex/pgo-avann-refresh",
        ),
        (
            "/home/hoxide/mycodes/atlas-unitary-singular-cayley",
            "803f5a3f632dc0e4ec2657141d6e6fb7ed41d78e",
            "refs/heads/codex/unitary-singular-cayley",
        ),
        (
            "/home/hoxide/mycodes/atlas-wcell-cost-audit",
            "9a8a8d958e78e375a1e082d14929d1785205dfd8",
            "refs/heads/codex/wcell-cost-audit",
        ),
        (
            "/home/hoxide/mycodes/atlas-weyl-value-compact",
            "766b862913da5133ee0d5800e0e766f529d7f6ce",
            "refs/heads/codex/weyl-value-compact",
        ),
    }
)


class GuardError(ValueError):
    """The registry or live Git worktree state is not safe to use."""


def _unique_object(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise GuardError(f"duplicate JSON key: {key!r}")
        result[key] = value
    return result


def _reject_json_constant(value):
    raise GuardError(f"non-finite JSON value is forbidden: {value}")


def _read_regular_file(path):
    path = Path(path)
    try:
        before = os.lstat(path)
    except OSError as error:
        raise GuardError(f"cannot inspect registry {path}: {error}") from error
    if stat.S_ISLNK(before.st_mode):
        raise GuardError(f"registry must not be a symlink: {path}")
    if not stat.S_ISREG(before.st_mode):
        raise GuardError(f"registry must be a regular file: {path}")
    if before.st_size > MAX_REGISTRY_BYTES:
        raise GuardError("registry exceeds the size limit")

    flags = os.O_RDONLY | getattr(os, "O_CLOEXEC", 0) | getattr(os, "O_NOFOLLOW", 0)
    try:
        descriptor = os.open(path, flags)
    except OSError as error:
        raise GuardError(f"cannot open registry {path}: {error}") from error
    try:
        opened = os.fstat(descriptor)
        if not stat.S_ISREG(opened.st_mode):
            raise GuardError(f"registry must remain a regular file: {path}")
        if (opened.st_dev, opened.st_ino) != (before.st_dev, before.st_ino):
            raise GuardError("registry changed while it was opened")
        chunks = []
        total = 0
        while True:
            chunk = os.read(descriptor, min(65536, MAX_REGISTRY_BYTES + 1 - total))
            if not chunk:
                break
            chunks.append(chunk)
            total += len(chunk)
            if total > MAX_REGISTRY_BYTES:
                raise GuardError("registry exceeds the size limit")
        after = os.fstat(descriptor)
    finally:
        os.close(descriptor)

    stable_before = (opened.st_dev, opened.st_ino, opened.st_size, opened.st_mtime_ns)
    stable_after = (after.st_dev, after.st_ino, after.st_size, after.st_mtime_ns)
    data = b"".join(chunks)
    if stable_before != stable_after or len(data) != after.st_size:
        raise GuardError("registry changed while it was read")
    return data


def _strict_absolute_path(value, label):
    if type(value) is not str or not value:
        raise GuardError(f"{label} must be a nonempty string")
    if any(ord(character) < 32 or ord(character) == 127 for character in value):
        raise GuardError(f"{label} contains a control character")
    path = PurePosixPath(value)
    if (not path.is_absolute() or path.as_posix() != value or value == "/"
            or value.startswith("//")):
        raise GuardError(f"{label} must be a normalized absolute POSIX path")
    if any(part in ("", ".", "..") for part in path.parts[1:]):
        raise GuardError(f"{label} contains an ambiguous component")
    return value


def _strict_sha(value, label):
    if type(value) is not str or SHA1.fullmatch(value) is None:
        raise GuardError(f"{label} must be a lowercase 40-hex commit id")
    return value


def _strict_branch(value, label):
    if type(value) is not str or not value.startswith("refs/heads/"):
        raise GuardError(f"{label} must be a full refs/heads/ name")
    short = value[len("refs/heads/"):]
    forbidden = " ~^:?*[\\"
    if (not short or short.startswith("/") or short.endswith(("/", "."))
            or "//" in short or ".." in short or "@{" in short or short == "@"
            or any(character in forbidden or ord(character) < 32
                   or ord(character) == 127 for character in short)
            or any(part.startswith(".") or part.endswith(".lock")
                   for part in short.split("/"))):
        raise GuardError(f"{label} is not a conservative Git branch ref")
    return value


def _strict_lifecycle_text(value, label):
    if (type(value) is not str or not value or value.strip() != value
            or len(value) > 512
            or any(ord(character) < 32 or ord(character) == 127
                   for character in value)):
        raise GuardError(f"{label} must be a short, nonempty single-line string")
    return value


def _parse_registry(raw):
    try:
        text = raw.decode("utf-8", errors="strict")
        document = json.loads(
            text,
            object_pairs_hook=_unique_object,
            parse_constant=_reject_json_constant,
        )
    except GuardError:
        raise
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise GuardError(f"registry is not strict UTF-8 JSON: {error}") from error

    if type(document) is not dict or frozenset(document) != ROOT_KEYS:
        raise GuardError("registry has unexpected or missing top-level keys")
    if document["schema"] != REGISTRY_SCHEMA:
        raise GuardError("registry schema is not supported")
    maximum = document["max_active_task_worktrees"]
    if type(maximum) is not int or maximum != 1:
        raise GuardError("max_active_task_worktrees must be exactly 1")
    rows = document["worktrees"]
    if type(rows) is not list or not rows:
        raise GuardError("worktrees must be a nonempty list")

    paths = set()
    branches = set()
    primary_count = 0
    legacy_count = 0
    active_count = 0
    for index, row in enumerate(rows):
        label = f"worktrees[{index}]"
        if type(row) is not dict:
            raise GuardError(f"{label} must be an object")
        status_value = row.get("status")
        if type(status_value) is not str or status_value not in STATUSES:
            raise GuardError(f"{label}.status is invalid")
        if status_value == "PRIMARY":
            expected_keys = PRIMARY_ENTRY_KEYS
        elif status_value == "ACTIVE":
            expected_keys = ACTIVE_ENTRY_KEYS
        else:
            expected_keys = PINNED_ENTRY_KEYS
        if frozenset(row) != expected_keys:
            raise GuardError(f"{label} has unexpected or missing keys")
        path_value = _strict_absolute_path(row["path"], f"{label}.path")
        if status_value != "PRIMARY":
            _strict_sha(row["head"], f"{label}.head")
        branch_value = _strict_branch(row["branch"], f"{label}.branch")
        if path_value in paths:
            raise GuardError(f"duplicate registered path: {path_value}")
        if branch_value in branches:
            raise GuardError(f"duplicate registered branch: {branch_value}")
        paths.add(path_value)
        branches.add(branch_value)
        if status_value == "PRIMARY":
            primary_count += 1
        elif status_value == "LEGACY":
            legacy_count += 1
        elif status_value == "ACTIVE":
            active_count += 1
            _strict_sha(row["starting_commit"], f"{label}.starting_commit")
            for field in ("owner", "purpose", "retirement_condition"):
                _strict_lifecycle_text(row[field], f"{label}.{field}")
    if rows[0]["status"] != "PRIMARY" or primary_count != 1:
        raise GuardError("registry must contain exactly one first-row PRIMARY worktree")
    if active_count > maximum:
        raise GuardError("registry contains more than one ACTIVE task worktree")
    if active_count:
        raise GuardError(
            "ACTIVE task worktrees are disabled until a receipt-bound creator exists"
        )
    return document


def _validate_frozen_legacy_identities(document):
    """Require the exact audited baseline until retirement receipts exist."""
    identities = frozenset(
        (row["path"], row["head"], row["branch"])
        for row in document["worktrees"]
        if row["status"] == "LEGACY"
    )
    unexpected = identities - FROZEN_LEGACY_IDENTITIES
    missing = FROZEN_LEGACY_IDENTITIES - identities
    if unexpected or missing:
        raise GuardError(
            "registry LEGACY snapshot changed without a retirement receipt; "
            "missing=" + repr(sorted(missing))
            + ", unexpected=" + repr(sorted(unexpected))
        )
    frozen_paths = {path for path, _head, _branch in FROZEN_LEGACY_IDENTITIES}
    frozen_branches = {
        branch for _path, _head, branch in FROZEN_LEGACY_IDENTITIES
    }
    reused = sorted(
        (row["path"], row["branch"])
        for row in document["worktrees"]
        if row["status"] == "ACTIVE"
        and (row["path"] in frozen_paths or row["branch"] in frozen_branches)
    )
    if reused:
        raise GuardError(
            "ACTIVE worktree reuses a frozen LEGACY identity: " + repr(reused)
        )


def _load_registry_snapshot(path):
    raw = _read_regular_file(path)
    document = _parse_registry(raw)
    _validate_frozen_legacy_identities(document)
    return document, hashlib.sha256(raw).hexdigest()


def load_registry(path=REGISTRY_PATH):
    return _load_registry_snapshot(path)[0]


def parse_worktree_porcelain(raw):
    if type(raw) is not bytes:
        raise GuardError("Git worktree inventory must be bytes")
    if not raw or len(raw) > MAX_GIT_OUTPUT_BYTES:
        raise GuardError("Git worktree inventory is empty or too large")
    if not raw.endswith(b"\0\0"):
        raise GuardError("Git worktree inventory lacks its record terminator")
    body = raw[:-2]
    if not body:
        raise GuardError("Git worktree inventory has no records")

    rows = []
    paths = set()
    branches = set()
    for index, record in enumerate(body.split(b"\0\0")):
        fields = record.split(b"\0")
        if len(fields) != 3:
            raise GuardError(
                f"worktree record {index} is malformed, detached, locked, prunable, or unknown"
            )
        expected = (b"worktree ", b"HEAD ", b"branch ")
        values = []
        for field, prefix in zip(fields, expected):
            if not field.startswith(prefix) or len(field) == len(prefix):
                raise GuardError(f"worktree record {index} has an unexpected field")
            try:
                values.append(field[len(prefix):].decode("utf-8", errors="strict"))
            except UnicodeDecodeError as error:
                raise GuardError(f"worktree record {index} is not UTF-8") from error
        path_value = _strict_absolute_path(values[0], f"worktree record {index} path")
        head_value = _strict_sha(values[1], f"worktree record {index} HEAD")
        branch_value = _strict_branch(values[2], f"worktree record {index} branch")
        if path_value in paths:
            raise GuardError(f"duplicate live worktree path: {path_value}")
        if branch_value in branches:
            raise GuardError(f"duplicate live worktree branch: {branch_value}")
        paths.add(path_value)
        branches.add(branch_value)
        rows.append({"path": path_value, "head": head_value, "branch": branch_value})
    return rows


def _git_arguments_are_read_only(arguments):
    arguments = tuple(arguments)
    if arguments == ("rev-parse", "--show-toplevel"):
        return True
    if arguments == ("worktree", "list", "--porcelain", "-z"):
        return True
    return False


def _run_git(arguments, repo_root, allowed_returncodes=(0,)):
    arguments = tuple(arguments)
    if not _git_arguments_are_read_only(arguments):
        raise GuardError("internal error: attempted a non-read-only Git command")
    command = [GIT_EXECUTABLE, "-C", str(repo_root), *arguments]
    try:
        completed = subprocess.run(
            command,
            stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
            shell=False,
            timeout=GIT_TIMEOUT_SECONDS,
            env=dict(SANITIZED_GIT_ENV),
        )
    except (OSError, subprocess.TimeoutExpired) as error:
        raise GuardError(f"read-only Git inspection failed: {error}") from error
    # subprocess.run has already allocated these byte strings; this is a
    # validation cap, not an incremental allocation bound.
    if (len(completed.stdout) > MAX_GIT_OUTPUT_BYTES
            or len(completed.stderr) > MAX_GIT_OUTPUT_BYTES):
        raise GuardError("Git inspection output exceeds the size limit")
    if completed.returncode not in allowed_returncodes:
        detail = completed.stderr.decode("utf-8", errors="replace").strip()
        if len(detail) > 300:
            detail = detail[:300] + "..."
        raise GuardError(
            f"read-only Git inspection exited {completed.returncode}"
            + (f": {detail}" if detail else "")
        )
    if completed.returncode != 0 and (completed.stdout or completed.stderr):
        raise GuardError("allowed missing Git object/ref result produced output")
    return completed.returncode, completed.stdout


def _one_git_line(raw, label):
    if not raw.endswith(b"\n") or raw.count(b"\n") != 1 or b"\r" in raw:
        raise GuardError(f"{label} was not one newline-terminated line")
    try:
        value = raw[:-1].decode("utf-8", errors="strict")
    except UnicodeDecodeError as error:
        raise GuardError(f"{label} was not UTF-8") from error
    if not value:
        raise GuardError(f"{label} was empty")
    return value


def inspect_worktrees(repo_root=REPOSITORY_ROOT):
    repo_root = Path(repo_root)
    _, top_raw = _run_git(("rev-parse", "--show-toplevel"), repo_root)
    top = _strict_absolute_path(_one_git_line(top_raw, "Git top level"), "Git top level")
    if top != str(repo_root):
        raise GuardError(f"guard must run from the primary checkout {repo_root}, got {top}")
    _, inventory_raw = _run_git(
        ("worktree", "list", "--porcelain", "-z"), repo_root
    )
    return parse_worktree_porcelain(inventory_raw)


def _open_absolute_directory_nofollow(path, label):
    """Open every component of one absolute directory without following links."""
    path = Path(path)
    if not path.is_absolute():
        raise GuardError(f"{label} must be absolute")
    directory_flag = getattr(os, "O_DIRECTORY", None)
    nofollow_flag = getattr(os, "O_NOFOLLOW", None)
    if directory_flag is None or nofollow_flag is None:
        raise GuardError("safe no-follow directory traversal is unavailable")
    flags = (os.O_RDONLY | getattr(os, "O_CLOEXEC", 0)
             | directory_flag | nofollow_flag)
    descriptor = None
    try:
        descriptor = os.open("/", flags)
        for component in path.parts[1:]:
            next_descriptor = os.open(component, flags, dir_fd=descriptor)
            os.close(descriptor)
            descriptor = next_descriptor
        return descriptor
    except OSError as error:
        if descriptor is not None:
            os.close(descriptor)
        raise GuardError(f"cannot open {label} without following links: {error}") from error


def _read_small_regular_at(directory_descriptor, name, label):
    """Read one bounded, stable, single-link regular file without following it."""
    nofollow_flag = getattr(os, "O_NOFOLLOW", None)
    if nofollow_flag is None:
        raise GuardError("safe no-follow regular-file inspection is unavailable")
    flags = os.O_RDONLY | getattr(os, "O_CLOEXEC", 0) | nofollow_flag
    descriptor = None
    try:
        descriptor = os.open(name, flags, dir_fd=directory_descriptor)
        before = os.fstat(descriptor)
        if not stat.S_ISREG(before.st_mode) or before.st_nlink != 1:
            raise GuardError(f"{label} is not a single-link regular file")
        if before.st_size > MAX_GIT_LINK_BYTES:
            raise GuardError(f"{label} exceeds the size limit")
        chunks = []
        total = 0
        while True:
            chunk = os.read(
                descriptor, min(1024, MAX_GIT_LINK_BYTES + 1 - total))
            if not chunk:
                break
            chunks.append(chunk)
            total += len(chunk)
            if total > MAX_GIT_LINK_BYTES:
                raise GuardError(f"{label} exceeds the size limit")
        after = os.fstat(descriptor)
    except GuardError:
        raise
    except OSError as error:
        raise GuardError(f"cannot read {label} without following links: {error}") from error
    finally:
        if descriptor is not None:
            os.close(descriptor)
    identity_before = (
        before.st_dev, before.st_ino, before.st_size, before.st_mtime_ns,
    )
    identity_after = (
        after.st_dev, after.st_ino, after.st_size, after.st_mtime_ns,
    )
    raw = b"".join(chunks)
    if identity_before != identity_after or len(raw) != after.st_size:
        raise GuardError(f"{label} changed while it was read")
    return raw, after


def _linked_git_admin_path(raw, worktree_path, primary_path):
    """Parse an exact linked-worktree gitfile into its primary admin path."""
    prefix = b"gitdir: "
    if (not raw.startswith(prefix) or not raw.endswith(b"\n")
            or raw.count(b"\n") != 1 or b"\r" in raw):
        raise GuardError(f"linked worktree {worktree_path} has an invalid .git file")
    try:
        value = raw[len(prefix):-1].decode("utf-8", errors="strict")
    except UnicodeDecodeError as error:
        raise GuardError(
            f"linked worktree {worktree_path} has a non-UTF-8 .git file"
        ) from error
    normalized = _strict_absolute_path(value, f"linked worktree {worktree_path} gitdir")
    admin = Path(normalized)
    expected_parent = Path(primary_path) / ".git" / "worktrees"
    if admin.parent != expected_parent or not admin.name:
        raise GuardError(
            f"linked worktree {worktree_path} escaped the primary Git admin directory"
        )
    return admin


def inspect_managed_namespace(registry):
    """Return a no-follow identity snapshot of the exact managed Atlas siblings."""
    registered = {row["path"]: row for row in registry["worktrees"]}
    primary_path = Path(registry["worktrees"][0]["path"])
    expected_names = set()
    for path_value in registered:
        path = Path(path_value)
        if (path.parent != MANAGED_WORKTREE_PARENT
                or not path.name.startswith(MANAGED_WORKTREE_PREFIX)):
            raise GuardError(
                "registered worktree escaped the managed Atlas sibling namespace: "
                + path_value
            )
        expected_names.add(path.name)
    if len(expected_names) != len(registered):
        raise GuardError("registered worktree sibling names are not unique")

    parent_descriptor = _open_absolute_directory_nofollow(
        MANAGED_WORKTREE_PARENT, "managed worktree parent")
    try:
        parent_status = os.fstat(parent_descriptor)
        if not stat.S_ISDIR(parent_status.st_mode):
            raise GuardError("managed worktree parent is not a directory")
        try:
            names_before = sorted(
                name for name in os.listdir(parent_descriptor)
                if name.startswith(MANAGED_WORKTREE_PREFIX)
            )
        except OSError as error:
            raise GuardError(f"cannot list managed worktree namespace: {error}") from error
        actual_names = set(names_before)
        if actual_names != expected_names:
            raise GuardError(
                "managed Atlas sibling mismatch; unregistered="
                + repr(sorted(actual_names - expected_names))
                + ", missing=" + repr(sorted(expected_names - actual_names))
            )

        entries = []
        directory_flag = getattr(os, "O_DIRECTORY", None)
        nofollow_flag = getattr(os, "O_NOFOLLOW", None)
        if directory_flag is None or nofollow_flag is None:
            raise GuardError("safe no-follow worktree inspection is unavailable")
        directory_flags = (os.O_RDONLY | getattr(os, "O_CLOEXEC", 0)
                           | directory_flag | nofollow_flag)
        for name in names_before:
            child_descriptor = None
            admin_descriptor = None
            try:
                child_descriptor = os.open(
                    name, directory_flags, dir_fd=parent_descriptor)
                child_status = os.fstat(child_descriptor)
                row = registered[str(MANAGED_WORKTREE_PARENT / name)]
                if row["status"] == "PRIMARY":
                    git_status = os.stat(
                        ".git", dir_fd=child_descriptor, follow_symlinks=False)
                    if not stat.S_ISDIR(git_status.st_mode):
                        raise GuardError(
                            f"managed worktree {name!r} has an unexpected .git file type"
                        )
                    admin_identity = (None, None, None, None, None, None)
                else:
                    git_raw, git_status = _read_small_regular_at(
                        child_descriptor, ".git", f"linked worktree {name!r} .git")
                    admin_path = _linked_git_admin_path(
                        git_raw, str(MANAGED_WORKTREE_PARENT / name), primary_path)
                    admin_descriptor = _open_absolute_directory_nofollow(
                        admin_path, f"linked worktree {name!r} Git admin directory")
                    admin_status = os.fstat(admin_descriptor)
                    backlink_raw, backlink_status = _read_small_regular_at(
                        admin_descriptor, "gitdir",
                        f"linked worktree {name!r} Git admin backlink")
                    commondir_raw, commondir_status = _read_small_regular_at(
                        admin_descriptor, "commondir",
                        f"linked worktree {name!r} Git common-dir marker")
                    expected_backlink = (
                        str(MANAGED_WORKTREE_PARENT / name) + "/.git\n"
                    ).encode("utf-8")
                    if backlink_raw != expected_backlink or commondir_raw != b"../..\n":
                        raise GuardError(
                            f"linked worktree {name!r} Git admin ownership changed"
                        )
                    admin_identity = (
                        admin_status.st_dev,
                        admin_status.st_ino,
                        backlink_status.st_dev,
                        backlink_status.st_ino,
                        commondir_status.st_dev,
                        commondir_status.st_ino,
                    )
            except OSError as error:
                raise GuardError(
                    f"managed worktree {name!r} is not a real accessible directory: {error}"
                ) from error
            finally:
                if admin_descriptor is not None:
                    os.close(admin_descriptor)
                if child_descriptor is not None:
                    os.close(child_descriptor)
            if not stat.S_ISDIR(child_status.st_mode):
                raise GuardError(f"managed worktree {name!r} is not a directory")
            entries.append((
                str(MANAGED_WORKTREE_PARENT / name),
                child_status.st_dev,
                child_status.st_ino,
                git_status.st_dev,
                git_status.st_ino,
                stat.S_IFMT(git_status.st_mode),
                *admin_identity,
            ))
        try:
            names_after = sorted(
                name for name in os.listdir(parent_descriptor)
                if name.startswith(MANAGED_WORKTREE_PREFIX)
            )
        except OSError as error:
            raise GuardError(
                f"cannot recheck managed worktree namespace: {error}"
            ) from error
        if names_after != names_before:
            raise GuardError("managed worktree namespace changed during inspection")
        return (
            parent_status.st_dev,
            parent_status.st_ino,
            tuple(entries),
        )
    finally:
        os.close(parent_descriptor)


def _compare_inventory(registry, actual):
    registered = {row["path"]: row for row in registry["worktrees"]}
    expected_paths = set(registered)
    actual_by_path = {row["path"]: row for row in actual}
    actual_paths = set(actual_by_path)
    if actual_paths != expected_paths:
        extra = sorted(actual_paths - expected_paths)
        missing = sorted(expected_paths - actual_paths)
        raise GuardError(f"worktree path mismatch; unregistered={extra}, missing={missing}")
    for path_value in sorted(expected_paths):
        registered_row = registered[path_value]
        actual_row = actual_by_path[path_value]
        fields = ("branch",) if registered_row["status"] == "PRIMARY" else ("head", "branch")
        for field in fields:
            if actual_row[field] != registered_row[field]:
                raise GuardError(
                    f"worktree {path_value} {field} mismatch: "
                    f"registered={registered_row[field]}, live={actual_row[field]}"
                )


def _validate_primary(registry, actual, repo_root):
    primary = registry["worktrees"][0]
    if primary["path"] != str(repo_root):
        raise GuardError("registered PRIMARY path is not this guard's repository root")
    if not actual or actual[0]["path"] != primary["path"]:
        raise GuardError("Git did not report the registered PRIMARY worktree first")


def validate_worktrees(
    mode="check", target=None, registry_path=REGISTRY_PATH, repo_root=REPOSITORY_ROOT,
):
    if mode == "precreate":
        raise GuardError(
            "precreate is disabled until a durable receipt-bound creator exists"
        )
    if mode != "check":
        raise GuardError(f"unsupported guard mode: {mode}")
    if target is not None:
        raise GuardError("check mode does not accept a target")
    repo_root = Path(repo_root)
    registry, registry_sha256 = _load_registry_snapshot(registry_path)
    actual = inspect_worktrees(repo_root)
    _validate_primary(registry, actual, repo_root)
    _compare_inventory(registry, actual)
    namespace = inspect_managed_namespace(registry)
    final_registry, final_registry_sha256 = _load_registry_snapshot(registry_path)
    if final_registry_sha256 != registry_sha256 or final_registry != registry:
        raise GuardError("registry changed during worktree validation")
    final_actual = inspect_worktrees(repo_root)
    _validate_primary(final_registry, final_actual, repo_root)
    _compare_inventory(final_registry, final_actual)
    final_namespace = inspect_managed_namespace(final_registry)
    closing_registry, closing_registry_sha256 = _load_registry_snapshot(registry_path)
    if closing_registry_sha256 != registry_sha256 or closing_registry != registry:
        raise GuardError("registry changed during final worktree validation")
    if final_actual != actual:
        raise GuardError("Git worktree inventory changed between validation samples")
    if final_namespace != namespace:
        raise GuardError("managed worktree namespace changed between validation samples")
    return {"mode": mode, "registered": len(final_registry["worktrees"])}


def main(argv=None):
    parser = argparse.ArgumentParser(
        description="Fail closed unless local Git worktrees exactly match their registry."
    )
    subparsers = parser.add_subparsers(dest="mode", required=True)
    subparsers.add_parser("check", help="compare the complete live and registered inventories")
    precreate = subparsers.add_parser(
        "precreate", help="disabled until a receipt-bound creator is implemented"
    )
    precreate.add_argument("path", help="exact absolute path of the registered ACTIVE worktree")
    arguments = parser.parse_args(argv)
    try:
        result = validate_worktrees(
            mode=arguments.mode,
            target=getattr(arguments, "path", None),
        )
    except GuardError as error:
        print(f"worktree guard rejected state: {error}", file=sys.stderr)
        return 1
    if result["mode"] == "check":
        print(
            f"worktree registry matches {result['registered']} live entries "
            "(PRIMARY HEAD intentionally unpinned)"
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

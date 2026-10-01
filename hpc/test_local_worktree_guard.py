"""Synthetic tests for the read-only local worktree lifecycle guard."""

import copy
import hashlib
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import Mock, patch

import local_worktree_guard as guard


PRIMARY_HEAD = "1" * 40
LEGACY_HEAD = "a2af0e79061ffe5a627ddf62b7e284ffd5b8cb7e"
LEGACY_BRANCH = "refs/heads/codex/coroot-coordinate-repair"
ACTIVE_HEAD = "3" * 40


def entry(path, head, branch, status, **extra):
    row = {"path": str(path), "branch": branch, "status": status}
    if head is not None:
        row["head"] = head
    row.update(extra)
    return row


def document(rows):
    return {
        "schema": guard.REGISTRY_SCHEMA,
        "max_active_task_worktrees": 1,
        "worktrees": rows,
    }


def active_entry(path, head=ACTIVE_HEAD, branch="refs/heads/codex/active"):
    return entry(
        path,
        head,
        branch,
        "ACTIVE",
        starting_commit=head,
        owner="task-owner",
        purpose="checkout isolation for one bounded task",
        retirement_condition="handoff is complete and removal is explicitly authorized",
    )


def frozen_rows(root="/repo"):
    return [entry(root, None, "refs/heads/codex/primary", "PRIMARY")] + [
        entry(path, head, branch, "LEGACY")
        for path, head, branch in sorted(guard.FROZEN_LEGACY_IDENTITIES)
    ]


def live_rows(rows):
    return [
        {
            "path": row["path"],
            "head": row.get("head", PRIMARY_HEAD),
            "branch": row["branch"],
        }
        for row in rows
    ]


def porcelain(rows):
    records = []
    for row in rows:
        records.append(
            b"worktree " + row["path"].encode("utf-8")
            + b"\0HEAD " + row.get("head", PRIMARY_HEAD).encode("ascii")
            + b"\0branch " + row["branch"].encode("utf-8")
            + b"\0\0"
        )
    return b"".join(records)


def completed(returncode=0, stdout=b"", stderr=b""):
    return subprocess.CompletedProcess([], returncode, stdout=stdout, stderr=stderr)


class LocalWorktreeGuard(unittest.TestCase):
    def write_registry(self, folder, value):
        path = Path(folder) / "registry.json"
        path.write_text(json.dumps(value, sort_keys=True) + "\n", encoding="utf-8")
        return path

    def base_rows(self, root="/repo"):
        return frozen_rows(root)

    def git_inventory_results(self, root, rows, samples=1):
        result = [
            completed(stdout=(str(root) + "\n").encode("utf-8")),
            completed(stdout=porcelain(rows)),
        ]
        return result * samples

    def namespace_snapshot(self, rows, identity=1):
        return ("namespace", identity, tuple(sorted(row["path"] for row in rows)))

    def namespace_fixture(self, folder):
        parent = Path(folder) / "mycodes"
        parent.mkdir()
        primary = parent / "atlas-rust"
        primary.mkdir()
        primary_git = primary / ".git"
        primary_git.mkdir()
        legacy = parent / "atlas-legacy"
        legacy.mkdir()
        legacy_admin = primary_git / "worktrees" / "atlas-legacy"
        legacy_admin.mkdir(parents=True)
        (legacy / ".git").write_text(
            f"gitdir: {legacy_admin}\n", encoding="utf-8"
        )
        (legacy_admin / "gitdir").write_text(
            f"{legacy}/.git\n", encoding="utf-8"
        )
        (legacy_admin / "commondir").write_text("../..\n", encoding="utf-8")
        rows = [
            entry(primary, None, "refs/heads/codex/primary", "PRIMARY"),
            entry(legacy, LEGACY_HEAD, LEGACY_BRANCH, "LEGACY"),
        ]
        return parent, document(rows), primary, legacy

    def assert_only_read_only_git(self, runner, root):
        self.assertGreaterEqual(runner.call_count, 1)
        for invocation in runner.call_args_list:
            command = invocation.args[0]
            self.assertEqual(command[:3], [guard.GIT_EXECUTABLE, "-C", str(root)])
            self.assertTrue(
                set(command).isdisjoint(guard.MUTATING_WORKTREE_SUBCOMMANDS), command
            )
            if command[3] == "worktree":
                self.assertEqual(command[4:], ["list", "--porcelain", "-z"])
            self.assertEqual(invocation.kwargs["stdin"], subprocess.DEVNULL)
            self.assertFalse(invocation.kwargs["check"])
            self.assertFalse(invocation.kwargs["shell"])
            self.assertEqual(invocation.kwargs["env"], guard.SANITIZED_GIT_ENV)
            self.assertNotIn("PATH", invocation.kwargs["env"])
            self.assertNotIn("LD_PRELOAD", invocation.kwargs["env"])
            self.assertNotIn("LD_LIBRARY_PATH", invocation.kwargs["env"])

    def test_committed_registry_is_the_25_entry_zero_active_snapshot(self):
        self.assertEqual(
            hashlib.sha256(guard.REGISTRY_PATH.read_bytes()).hexdigest(),
            "2059e9c5a09ab03d0a10eb2af6b6cd9946a940c980ded1b5d8074d977f2790cb",
        )
        registry = guard.load_registry()
        rows = registry["worktrees"]
        self.assertEqual(len(rows), 25)
        self.assertEqual(sum(row["status"] == "PRIMARY" for row in rows), 1)
        self.assertEqual(sum(row["status"] == "LEGACY" for row in rows), 24)
        self.assertEqual(sum(row["status"] == "ACTIVE" for row in rows), 0)
        self.assertEqual(rows[0]["path"], "/home/hoxide/mycodes/atlas-rust")
        self.assertEqual(frozenset(rows[0]), frozenset({"path", "branch", "status"}))
        self.assertEqual(rows[-1]["path"], "/home/hoxide/mycodes/atlas-weyl-value-compact")
        self.assertEqual(rows[-1]["head"], "766b862913da5133ee0d5800e0e766f529d7f6ce")

    def test_frozen_legacy_baseline_requires_the_exact_24_identities(self):
        registry = guard.load_registry()
        legacy = [row for row in registry["worktrees"] if row["status"] == "LEGACY"]
        self.assertEqual(len(legacy), 24)
        self.assertEqual(
            {(row["path"], row["head"], row["branch"]) for row in legacy},
            guard.FROZEN_LEGACY_IDENTITIES,
        )
        self.assertIsNone(guard._validate_frozen_legacy_identities(registry))

        mutations = {}
        missing = copy.deepcopy(registry)
        missing["worktrees"].pop()
        mutations["missing"] = missing
        replacement = copy.deepcopy(registry)
        replacement["worktrees"][-1] = entry(
            "/home/hoxide/mycodes/atlas-new-slot",
            "4" * 40,
            "refs/heads/codex/new-slot",
            "LEGACY",
        )
        mutations["replacement"] = replacement
        changed_head = copy.deepcopy(registry)
        changed_head["worktrees"][-1]["head"] = "4" * 40
        mutations["head"] = changed_head
        changed_branch = copy.deepcopy(registry)
        changed_branch["worktrees"][-1]["branch"] = "refs/heads/codex/changed-legacy"
        mutations["branch"] = changed_branch
        for name, mutation in mutations.items():
            with self.subTest(name=name), self.assertRaisesRegex(
                guard.GuardError, "LEGACY snapshot changed"
            ):
                guard._validate_frozen_legacy_identities(mutation)

    def test_matching_live_and_registered_novel_legacy_is_rejected_before_git(self):
        rows = self.base_rows()
        rows[-1] = entry(
            "/home/hoxide/mycodes/atlas-new-slot",
            "4" * 40,
            "refs/heads/codex/new-slot",
            "LEGACY",
        )
        with tempfile.TemporaryDirectory() as folder:
            path = self.write_registry(folder, document(rows))
            namespace = Mock()
            with (
                patch.object(guard, "inspect_managed_namespace", namespace),
                patch("local_worktree_guard.subprocess.run") as runner,
                self.assertRaisesRegex(guard.GuardError, "LEGACY snapshot changed"),
            ):
                guard.validate_worktrees(
                    registry_path=path,
                    repo_root=Path("/repo"),
                )
            runner.assert_not_called()
            namespace.assert_not_called()

    def test_registry_rejects_a_symlink(self):
        with tempfile.TemporaryDirectory() as folder:
            real = self.write_registry(folder, document(self.base_rows()))
            linked = Path(folder) / "linked.json"
            linked.symlink_to(real)
            with self.assertRaisesRegex(guard.GuardError, "must not be a symlink"):
                guard.load_registry(linked)

    def test_registry_rejects_duplicate_json_keys(self):
        raw = (
            b'{"schema":"atlas-local-worktree-registry-v1",'
            b'"schema":"atlas-local-worktree-registry-v1",'
            b'"max_active_task_worktrees":1,"worktrees":[]}\n'
        )
        with self.assertRaisesRegex(guard.GuardError, "duplicate JSON key"):
            guard._parse_registry(raw)

    def test_registry_schema_is_strict(self):
        base = document(self.base_rows())
        mutations = {}
        unknown_root = copy.deepcopy(base)
        unknown_root["extra"] = True
        mutations["unknown root key"] = unknown_root
        boolean_limit = copy.deepcopy(base)
        boolean_limit["max_active_task_worktrees"] = True
        mutations["boolean limit"] = boolean_limit
        unknown_row = copy.deepcopy(base)
        unknown_row["worktrees"][1]["extra"] = True
        mutations["unknown row key"] = unknown_row
        primary_head = copy.deepcopy(base)
        primary_head["worktrees"][0]["head"] = PRIMARY_HEAD
        mutations["PRIMARY head is forbidden"] = primary_head
        bad_status = copy.deepcopy(base)
        bad_status["worktrees"][1]["status"] = "RETIRED"
        mutations["unknown status"] = bad_status
        container_status = copy.deepcopy(base)
        container_status["worktrees"][1]["status"] = []
        mutations["container status"] = container_status
        duplicate_path = copy.deepcopy(base)
        duplicate_path["worktrees"][1]["path"] = "/repo"
        mutations["duplicate path"] = duplicate_path
        duplicate_branch = copy.deepcopy(base)
        duplicate_branch["worktrees"][1]["branch"] = "refs/heads/codex/primary"
        mutations["duplicate branch"] = duplicate_branch
        noncanonical_path = copy.deepcopy(base)
        noncanonical_path["worktrees"][1]["path"] = "/legacy/../other"
        mutations["noncanonical path"] = noncanonical_path
        double_slash_path = copy.deepcopy(base)
        double_slash_path["worktrees"][1]["path"] = "//legacy"
        mutations["double slash path"] = double_slash_path
        uppercase_head = copy.deepcopy(base)
        uppercase_head["worktrees"][1]["head"] = "A" * 40
        mutations["uppercase head"] = uppercase_head
        incomplete_active = copy.deepcopy(base)
        incomplete_active["worktrees"][1]["status"] = "ACTIVE"
        mutations["incomplete active"] = incomplete_active
        blank_owner = copy.deepcopy(base)
        blank_owner["worktrees"].append(active_entry("/active"))
        blank_owner["worktrees"][-1]["owner"] = " "
        mutations["blank active owner"] = blank_owner
        second_primary = copy.deepcopy(base)
        second_primary["worktrees"][1]["status"] = "PRIMARY"
        mutations["second primary"] = second_primary
        two_active = copy.deepcopy(base)
        two_active["worktrees"].append(active_entry("/active-one"))
        two_active["worktrees"].append(
            active_entry("/active-two", "4" * 40, "refs/heads/codex/active-two")
        )
        mutations["two active"] = two_active
        for name, mutation in mutations.items():
            with self.subTest(name=name), self.assertRaises(guard.GuardError):
                guard._parse_registry(
                    (json.dumps(mutation, sort_keys=True) + "\n").encode("utf-8")
                )

    def test_active_and_precreate_are_disabled_before_state_access(self):
        primary = self.base_rows()[0]
        for rows in (
            self.base_rows() + [active_entry("/active")],
            [primary, active_entry("/active")],
        ):
            with self.subTest(rows=len(rows)), self.assertRaisesRegex(
                guard.GuardError, "ACTIVE task worktrees are disabled"
            ):
                guard._parse_registry(
                    (json.dumps(document(rows), sort_keys=True) + "\n").encode("utf-8")
                )
        with (
            patch.object(guard, "_load_registry_snapshot") as load_registry,
            patch.object(guard, "inspect_worktrees") as inspect_git,
            patch.object(guard, "inspect_managed_namespace") as inspect_namespace,
            patch("local_worktree_guard.subprocess.run") as runner,
        ):
            with self.assertRaisesRegex(guard.GuardError, "precreate is disabled"):
                guard.validate_worktrees(
                    mode="precreate",
                    target="/home/hoxide/mycodes/atlas-active",
                    registry_path=Path("/not-read"),
                    repo_root=Path("/repo"),
                )
            load_registry.assert_not_called()
            inspect_git.assert_not_called()
            inspect_namespace.assert_not_called()
            runner.assert_not_called()

    def test_porcelain_parser_accepts_normalized_paths_with_spaces(self):
        rows = [
            entry("/repo with space", None, "refs/heads/codex/primary", "PRIMARY"),
            entry("/legacy with space", LEGACY_HEAD, LEGACY_BRANCH, "LEGACY"),
        ]
        self.assertEqual(guard.parse_worktree_porcelain(porcelain(rows)), live_rows(rows))

    def test_porcelain_parser_accepts_a_frozen_real_git_shape(self):
        raw = (
            b"worktree /repo\0"
            b"HEAD 1111111111111111111111111111111111111111\0"
            b"branch refs/heads/codex/primary\0\0"
            b"worktree /legacy\0"
            b"HEAD 2222222222222222222222222222222222222222\0"
            b"branch refs/heads/codex/legacy\0\0"
        )
        parsed = guard.parse_worktree_porcelain(raw)
        self.assertEqual(parsed[0]["path"], "/repo")
        self.assertEqual(parsed[1]["branch"], "refs/heads/codex/legacy")

    def test_porcelain_parser_rejects_special_or_malformed_records(self):
        primary = entry("/repo", None, "refs/heads/codex/primary", "PRIMARY")
        ordinary = porcelain([primary])
        prefix = b"worktree /repo\0HEAD " + PRIMARY_HEAD.encode("ascii")
        cases = {
            "empty": b"",
            "oversized": b"x" * (guard.MAX_GIT_OUTPUT_BYTES + 1),
            "not terminated": ordinary[:-1],
            "detached": prefix + b"\0detached\0\0",
            "locked": ordinary[:-2] + b"\0locked reason\0\0",
            "prunable": ordinary[:-2] + b"\0prunable reason\0\0",
            "unknown": ordinary[:-2] + b"\0bare\0\0",
            "non UTF-8": ordinary.replace(b"worktree /repo", b"worktree /\xff"),
            "reordered": (
                b"HEAD " + PRIMARY_HEAD.encode("ascii")
                + b"\0worktree /repo\0branch refs/heads/codex/primary\0\0"
            ),
            "empty path": ordinary.replace(b"worktree /repo", b"worktree "),
            "empty branch": ordinary.replace(
                b"branch refs/heads/codex/primary", b"branch "
            ),
            "bad head": ordinary.replace(PRIMARY_HEAD.encode("ascii"), b"A" * 40),
            "relative path": ordinary.replace(b"worktree /repo", b"worktree relative"),
        }
        for name, raw in cases.items():
            with self.subTest(name=name), self.assertRaises(guard.GuardError):
                guard.parse_worktree_porcelain(raw)
        duplicate_path = [
            primary, entry("/repo", LEGACY_HEAD, LEGACY_BRANCH, "LEGACY")]
        with self.assertRaisesRegex(guard.GuardError, "duplicate live worktree path"):
            guard.parse_worktree_porcelain(porcelain(duplicate_path))
        duplicate_branch = [
            primary,
            entry("/legacy", LEGACY_HEAD, "refs/heads/codex/primary", "LEGACY"),
        ]
        with self.assertRaisesRegex(guard.GuardError, "duplicate live worktree branch"):
            guard.parse_worktree_porcelain(porcelain(duplicate_branch))

    def test_exact_inventory_passes_two_samples_using_only_read_only_git(self):
        rows = self.base_rows()
        namespace = Mock(return_value=self.namespace_snapshot(rows))
        with tempfile.TemporaryDirectory() as folder:
            registry = self.write_registry(folder, document(rows))
            hostile_environment = {
                "PATH": "/malicious/bin",
                "LD_PRELOAD": "/malicious/inject.so",
                "LD_LIBRARY_PATH": "/malicious/lib",
                "PYTHONPATH": "/malicious/python",
                "GIT_DIR": "/alternate/repository",
                "GIT_WORK_TREE": "/repo",
                "GIT_COMMON_DIR": "/alternate/common",
                "GIT_OBJECT_DIRECTORY": "/alternate/objects",
                "GIT_CONFIG_COUNT": "1",
                "GIT_CONFIG_KEY_0": "core.worktree",
                "GIT_CONFIG_VALUE_0": "/repo",
            }
            with patch.dict(
                "local_worktree_guard.os.environ", hostile_environment, clear=False
            ), patch(
                "local_worktree_guard.subprocess.run",
                side_effect=self.git_inventory_results("/repo", rows, samples=2),
            ) as runner, patch.object(
                guard, "inspect_managed_namespace", namespace
            ):
                result = guard.validate_worktrees(
                    registry_path=registry,
                    repo_root=Path("/repo"),
                )
            self.assertEqual(result, {"mode": "check", "registered": 25})
            self.assertEqual(namespace.call_count, 2)
            self.assert_only_read_only_git(runner, Path("/repo"))
            self.assertEqual(runner.call_count, 4)

    def test_inventory_rejects_extra_missing_and_drifted_entries(self):
        registered = self.base_rows()
        extra = registered + [
            entry("/extra", "4" * 40, "refs/heads/codex/extra", "LEGACY")]
        missing = registered[:-1]
        head_drift = copy.deepcopy(registered)
        head_drift[1]["head"] = "5" * 40
        branch_drift = copy.deepcopy(registered)
        branch_drift[1]["branch"] = "refs/heads/codex/drift"
        reordered = list(reversed(registered))
        cases = {
            "extra": extra,
            "missing": missing,
            "head drift": head_drift,
            "branch drift": branch_drift,
            "primary not first": reordered,
        }
        with tempfile.TemporaryDirectory() as folder:
            registry = self.write_registry(folder, document(registered))
            for name, live in cases.items():
                namespace = Mock()
                with (
                    self.subTest(name=name),
                    patch(
                        "local_worktree_guard.subprocess.run",
                        side_effect=self.git_inventory_results("/repo", live),
                    ) as runner,
                    patch.object(guard, "inspect_managed_namespace", namespace),
                ):
                    with self.assertRaises(guard.GuardError):
                        guard.validate_worktrees(
                            registry_path=registry,
                            repo_root=Path("/repo"),
                        )
                    self.assert_only_read_only_git(runner, Path("/repo"))
                    namespace.assert_not_called()

    def test_primary_head_may_advance_between_checks_but_not_during_one(self):
        registered = self.base_rows()
        advanced = copy.deepcopy(registered)
        advanced[0]["head"] = "9" * 40
        moved_branch = copy.deepcopy(advanced)
        moved_branch[0]["branch"] = "refs/heads/codex/other-primary"
        namespace = Mock(return_value=self.namespace_snapshot(registered))
        with tempfile.TemporaryDirectory() as folder:
            registry = self.write_registry(folder, document(registered))
            with patch(
                "local_worktree_guard.subprocess.run",
                side_effect=self.git_inventory_results("/repo", advanced, samples=2),
            ) as runner, patch.object(
                guard, "inspect_managed_namespace", namespace
            ):
                result = guard.validate_worktrees(
                    registry_path=registry,
                    repo_root=Path("/repo"),
                )
            self.assertEqual(result, {"mode": "check", "registered": 25})
            self.assert_only_read_only_git(runner, Path("/repo"))
            rejected_namespace = Mock()
            with patch(
                "local_worktree_guard.subprocess.run",
                side_effect=self.git_inventory_results("/repo", moved_branch),
            ) as runner, patch.object(
                guard, "inspect_managed_namespace", rejected_namespace
            ):
                with self.assertRaisesRegex(guard.GuardError, "branch mismatch"):
                    guard.validate_worktrees(
                        registry_path=registry,
                        repo_root=Path("/repo"),
                    )
            self.assert_only_read_only_git(runner, Path("/repo"))
            rejected_namespace.assert_not_called()

    def test_git_errors_timeouts_and_malformed_output_fail_closed(self):
        rows = self.base_rows()
        with tempfile.TemporaryDirectory() as folder:
            registry = self.write_registry(folder, document(rows))
            cases = {
                "nonzero": [completed(returncode=2, stderr=b"fatal\n")],
                "bad top": [completed(stdout=b"/repo")],
                "bad inventory": [
                    completed(stdout=b"/repo\n"), completed(stdout=b"not porcelain\0\0")],
            }
            for name, side_effect in cases.items():
                namespace = Mock()
                with (
                    self.subTest(name=name),
                    patch(
                        "local_worktree_guard.subprocess.run", side_effect=side_effect
                    ) as runner,
                    patch.object(guard, "inspect_managed_namespace", namespace),
                ):
                    with self.assertRaises(guard.GuardError):
                        guard.validate_worktrees(
                            registry_path=registry,
                            repo_root=Path("/repo"),
                        )
                    self.assert_only_read_only_git(runner, Path("/repo"))
                    namespace.assert_not_called()
            namespace = Mock()
            with patch(
                "local_worktree_guard.subprocess.run",
                side_effect=subprocess.TimeoutExpired("git", 15),
            ) as runner, patch.object(
                guard, "inspect_managed_namespace", namespace
            ):
                with self.assertRaises(guard.GuardError):
                    guard.validate_worktrees(
                        registry_path=registry,
                        repo_root=Path("/repo"),
                    )
                self.assert_only_read_only_git(runner, Path("/repo"))
                namespace.assert_not_called()

    def test_git_inventory_change_between_samples_is_rejected(self):
        rows = self.base_rows()
        changed = copy.deepcopy(rows)
        changed[0]["head"] = "9" * 40
        namespace = Mock(return_value=self.namespace_snapshot(rows))
        with tempfile.TemporaryDirectory() as folder:
            registry = self.write_registry(folder, document(rows))
            results = (
                self.git_inventory_results("/repo", rows)
                + self.git_inventory_results("/repo", changed)
            )
            with patch(
                "local_worktree_guard.subprocess.run", side_effect=results
            ) as runner, patch.object(
                guard, "inspect_managed_namespace", namespace
            ), self.assertRaisesRegex(guard.GuardError, "Git worktree inventory changed"):
                guard.validate_worktrees(
                    registry_path=registry,
                    repo_root=Path("/repo"),
                )
            self.assertEqual(namespace.call_count, 2)
            self.assert_only_read_only_git(runner, Path("/repo"))

    def test_registry_change_between_samples_is_rejected(self):
        rows = self.base_rows()
        registry = document(rows)
        changed = copy.deepcopy(registry)
        changed["worktrees"][0]["branch"] = "refs/heads/codex/changed-primary"
        namespace = Mock(return_value=self.namespace_snapshot(rows))
        with (
            patch.object(
                guard,
                "_load_registry_snapshot",
                side_effect=[(registry, "a" * 64), (changed, "b" * 64)],
            ) as load_registry,
            patch.object(guard, "inspect_worktrees", return_value=live_rows(rows)) as inspect_git,
            patch.object(guard, "inspect_managed_namespace", namespace),
        ):
            with self.assertRaisesRegex(guard.GuardError, "registry changed"):
                guard.validate_worktrees(
                    registry_path=Path("/registry"),
                    repo_root=Path("/repo"),
                )
        self.assertEqual(load_registry.call_count, 2)
        inspect_git.assert_called_once_with(Path("/repo"))
        namespace.assert_called_once_with(registry)

        closing_namespace = Mock(return_value=self.namespace_snapshot(rows))
        with (
            patch.object(
                guard,
                "_load_registry_snapshot",
                side_effect=[
                    (registry, "a" * 64),
                    (registry, "a" * 64),
                    (changed, "b" * 64),
                ],
            ) as closing_load,
            patch.object(
                guard, "inspect_worktrees", side_effect=[live_rows(rows), live_rows(rows)]
            ) as closing_git,
            patch.object(
                guard, "inspect_managed_namespace", closing_namespace
            ),
        ):
            with self.assertRaisesRegex(
                guard.GuardError, "registry changed during final worktree validation"
            ):
                guard.validate_worktrees(
                    registry_path=Path("/registry"), repo_root=Path("/repo")
                )
        self.assertEqual(closing_load.call_count, 3)
        self.assertEqual(closing_git.call_count, 2)
        self.assertEqual(closing_namespace.call_count, 2)

    def test_managed_namespace_accepts_exact_real_siblings(self):
        with tempfile.TemporaryDirectory() as folder:
            parent, registry, primary, legacy = self.namespace_fixture(folder)
            with patch.object(guard, "MANAGED_WORKTREE_PARENT", parent):
                snapshot = guard.inspect_managed_namespace(registry)
            self.assertEqual(snapshot[:2], (parent.stat().st_dev, parent.stat().st_ino))
            self.assertEqual(
                [row[0] for row in snapshot[2]], sorted((str(primary), str(legacy)))
            )
            self.assertEqual(len(snapshot[2]), 2)

    def test_managed_namespace_rejects_an_added_atlas_sibling(self):
        with tempfile.TemporaryDirectory() as folder:
            parent, registry, _primary, _legacy = self.namespace_fixture(folder)
            (parent / "atlas-unregistered-clone").mkdir()
            with patch.object(guard, "MANAGED_WORKTREE_PARENT", parent), self.assertRaisesRegex(
                guard.GuardError, "unregistered"
            ):
                guard.inspect_managed_namespace(registry)

    def test_managed_namespace_rejects_missing_or_escaped_siblings(self):
        with tempfile.TemporaryDirectory() as folder:
            parent, registry, _primary, _legacy = self.namespace_fixture(folder)
            missing = copy.deepcopy(registry)
            missing["worktrees"].append(entry(
                parent / "atlas-missing", "4" * 40, "refs/heads/codex/missing", "LEGACY"
            ))
            with patch.object(guard, "MANAGED_WORKTREE_PARENT", parent), self.assertRaisesRegex(
                guard.GuardError, "missing"
            ):
                guard.inspect_managed_namespace(missing)
            escaped = copy.deepcopy(registry)
            escaped["worktrees"][-1]["path"] = str(Path(folder) / "atlas-escaped")
            with patch.object(guard, "MANAGED_WORKTREE_PARENT", parent), self.assertRaisesRegex(
                guard.GuardError, "escaped"
            ):
                guard.inspect_managed_namespace(escaped)

    def test_managed_namespace_rejects_sibling_and_parent_symlinks_nofollow(self):
        with tempfile.TemporaryDirectory() as folder:
            parent = Path(folder) / "mycodes"
            parent.mkdir()
            primary = parent / "atlas-rust"
            primary.mkdir()
            (primary / ".git").mkdir()
            outside = Path(folder) / "outside"
            outside.mkdir()
            (outside / ".git").write_text("gitdir: /outside\n")
            linked = parent / "atlas-linked"
            linked.symlink_to(outside, target_is_directory=True)
            registry = document([
                entry(primary, None, "refs/heads/codex/primary", "PRIMARY"),
                entry(linked, LEGACY_HEAD, LEGACY_BRANCH, "LEGACY"),
            ])
            with patch.object(guard, "MANAGED_WORKTREE_PARENT", parent), self.assertRaisesRegex(
                guard.GuardError, "not a real accessible directory"
            ):
                guard.inspect_managed_namespace(registry)
        with tempfile.TemporaryDirectory() as folder:
            real_parent, registry, _primary, _legacy = self.namespace_fixture(folder)
            alias = Path(folder) / "mycodes-alias"
            alias.symlink_to(real_parent, target_is_directory=True)
            aliased = copy.deepcopy(registry)
            for row in aliased["worktrees"]:
                row["path"] = str(alias / Path(row["path"]).name)
            with patch.object(guard, "MANAGED_WORKTREE_PARENT", alias), self.assertRaisesRegex(
                guard.GuardError, "without following links"
            ):
                guard.inspect_managed_namespace(aliased)
            for missing_flag in ("O_NOFOLLOW", "O_DIRECTORY"):
                with (
                    self.subTest(missing_flag=missing_flag),
                    patch.object(guard, "MANAGED_WORKTREE_PARENT", real_parent),
                    patch.object(guard.os, missing_flag),
                ):
                    delattr(guard.os, missing_flag)
                    with self.assertRaisesRegex(
                        guard.GuardError,
                        "safe no-follow directory traversal is unavailable",
                    ):
                        guard.inspect_managed_namespace(registry)

    def test_managed_namespace_inode_replacement_is_visible_and_rejected(self):
        with tempfile.TemporaryDirectory() as folder:
            parent, registry, primary, legacy = self.namespace_fixture(folder)
            admin = primary / ".git" / "worktrees" / "atlas-legacy"
            with patch.object(guard, "MANAGED_WORKTREE_PARENT", parent):
                tamper_cases = (
                    (
                        "linked .git",
                        legacy / ".git",
                        b"gitdir: /outside\n",
                        "escaped the primary Git admin directory",
                    ),
                    (
                        "admin backlink",
                        admin / "gitdir",
                        b"/outside/.git\n",
                        "Git admin ownership changed",
                    ),
                    (
                        "admin commondir",
                        admin / "commondir",
                        b"../../../outside\n",
                        "Git admin ownership changed",
                    ),
                )
                for label, target, tampered, error in tamper_cases:
                    original = target.read_bytes()
                    try:
                        target.write_bytes(tampered)
                        with self.subTest(binding=label), self.assertRaisesRegex(
                            guard.GuardError, error
                        ):
                            guard.inspect_managed_namespace(registry)
                    finally:
                        target.write_bytes(original)
                first = guard.inspect_managed_namespace(registry)
                legacy.rename(parent / "retired")
                legacy.mkdir()
                (legacy / ".git").write_text(
                    f"gitdir: {admin}\n", encoding="utf-8"
                )
                second = guard.inspect_managed_namespace(registry)
            self.assertNotEqual(first, second)
            self.assertEqual(
                tuple(row[0] for row in first[2]), tuple(row[0] for row in second[2])
            )
            rows = registry["worktrees"]
            namespace = Mock(side_effect=[first, second])
            with (
                patch.object(
                    guard,
                    "_load_registry_snapshot",
                    side_effect=[
                        (registry, "a" * 64),
                        (registry, "a" * 64),
                        (registry, "a" * 64),
                    ],
                ),
                patch.object(
                    guard, "inspect_worktrees", side_effect=[live_rows(rows), live_rows(rows)]
                ) as inspect_git,
                patch.object(guard, "inspect_managed_namespace", namespace),
            ):
                with self.assertRaisesRegex(guard.GuardError, "namespace changed"):
                    guard.validate_worktrees(
                        registry_path=Path("/registry"),
                        repo_root=primary,
                    )
            self.assertEqual(inspect_git.call_count, 2)
            self.assertEqual(namespace.call_count, 2)

    def test_internal_git_allowlist_rejects_every_mutating_worktree_verb(self):
        for verb in sorted(guard.MUTATING_WORKTREE_SUBCOMMANDS):
            with self.subTest(verb=verb), patch(
                "local_worktree_guard.subprocess.run"
            ) as runner:
                with self.assertRaisesRegex(guard.GuardError, "non-read-only"):
                    guard._run_git(("worktree", verb), Path("/repo"))
                runner.assert_not_called()


if __name__ == "__main__":
    unittest.main()

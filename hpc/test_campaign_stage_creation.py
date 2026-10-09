"""Tests-first contract for receipt-bound campaign stage publication.

These checks are infrastructure-only.  They create small synthetic payloads;
they do not build Atlas, run mathematical fixtures, or contact SLURM.
"""
import copy
import hashlib
import json
import multiprocessing
import os
from pathlib import Path
import shutil
import stat
import subprocess
import tempfile
import unittest
from unittest.mock import patch

import campaign_workspace
import progressive_submit
from progressive_submit import (
    ACTIVE_STAGE_NAME,
    STAGE_CREATION_MARKER,
    STAGE_CREATION_PREPARED,
    STAGE_CREATION_PUBLISHED,
    STAGE_CREATION_RECEIPT,
    STAGE_CREATION_SEALED,
    create_fixed_stage,
    submit_one,
    validate_stage_creation,
)


CONTRACT_KEYS = {
    "schema", "campaign", "stage_name", "predecessor_ledger_sha256",
    "predecessor_state", "overrides_sha256", "inputs", "script", "pin",
    "lifecycle",
}
RECEIPT_KEYS = {
    "schema", "status", "contract", "contract_sha256", "prepared_sha256",
    "stage", "transaction", "stage_device", "stage_inode",
}
PREPARED_KEYS = {
    "schema", "status", "sequence", "contract", "contract_sha256",
    "transaction",
}
SEALED_KEYS = {
    "schema", "status", "sequence", "contract_sha256", "prepared_sha256",
    "receipt_sha256", "stage", "transaction", "hidden_device",
    "hidden_inode",
}
PUBLISHED_KEYS = {
    "schema", "status", "sequence", "contract_sha256", "prepared_sha256",
    "sealed_sha256", "receipt_sha256", "stage", "transaction", "stage_device",
    "stage_inode",
}
PRIOR_FAILURE_KEYS = {
    "schema", "temporary", "archive", "sha256", "bytes", "mode",
    "destination", "contract_sha256", "transaction",
}
SCRIPT_PATH = "hpc/math_weyl_context_core_capture.sbatch"
PIN_PATH = "weyl-context-g2-v1-pin.json"
PREDECESSOR_STAGE_NAME = "weyl-context-core-after-v5"
PREDECESSOR_LINEAGE = (
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
    (PREDECESSOR_STAGE_NAME, "3900050"),
)


def sha256(raw):
    return hashlib.sha256(raw).hexdigest()


def json_bytes(value):
    return (json.dumps(value, indent=2, sort_keys=True) + "\n").encode()


def write_frozen(path, raw):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(raw)
    path.chmod(0o444)


def transaction_temp_name(name, raw):
    digest = hashlib.sha256(name.encode("utf-8") + b"\0" + raw).hexdigest()
    return ".atlas-publish-" + digest + ".tmp"


def configure_child_home(campaign):
    """Make a spawned test process use its synthetic HPC home."""
    home = Path(campaign).parent
    campaign_workspace.HPC_HOME = home
    if hasattr(progressive_submit, "HPC_HOME"):
        progressive_submit.HPC_HOME = home


def create_worker(campaign, payload, contract, start, results):
    configure_child_home(campaign)
    start.wait(20)
    try:
        stage, receipt_sha256 = create_fixed_stage(
            Path(campaign), Path(payload), contract)
        results.put(("ok", str(stage), receipt_sha256))
    except BaseException as error:  # Preserve the exact competing outcome.
        results.put(("error", type(error).__name__, str(error)))


def crash_worker(campaign, payload, contract, checkpoint):
    configure_child_home(campaign)

    def crash_here(observed, _context):
        if observed == checkpoint:
            os._exit(73)

    create_fixed_stage(
        Path(campaign), Path(payload), contract, _fault=crash_here)


class CampaignStageCreation(unittest.TestCase):
    maxDiff = None

    def test_scheduler_user_uses_effective_identity_not_environment(self):
        account = type("Account", (), {"pw_name": "guard-test"})()
        with patch.dict(os.environ, {"USER": "forged-user"}, clear=True), \
                patch("progressive_submit.os.geteuid", return_value=4815), \
                patch("progressive_submit.pwd.getpwuid",
                      return_value=account) as lookup:
            self.assertEqual(progressive_submit.scheduler_user(), "guard-test")
        lookup.assert_called_once_with(4815)

        invalid = type("Account", (), {"pw_name": "-unsafe"})()
        with patch("progressive_submit.pwd.getpwuid", return_value=invalid), \
                self.assertRaisesRegex(ValueError, "scheduler user"):
            progressive_submit.scheduler_user()

        with patch("progressive_submit.pwd.getpwuid", side_effect=KeyError), \
                self.assertRaisesRegex(ValueError, "scheduler user"):
            progressive_submit.scheduler_user()

    def campaign(self, folder, *, ledger=b"[]\n"):
        campaign = Path(folder) / campaign_workspace.ACTIVE_CAMPAIGN
        (campaign / "stages").mkdir(parents=True)
        (campaign / ".atlas-progressive-submit.json").write_bytes(ledger)
        return campaign

    def bound_file(self, path):
        raw = path.read_bytes()
        value = path.stat()
        return {
            "sha256": sha256(raw),
            "bytes": len(raw),
            "mode": format(stat.S_IMODE(value.st_mode), "04o"),
            "nlink": value.st_nlink,
        }

    def predecessor_fixture(self, campaign):
        predecessor = campaign / "stages" / PREDECESSOR_STAGE_NAME
        campaign_payloads = {
            ".atlas-stage-creation-prepared.json": b'{"v1":"prepared"}\n',
            ".atlas-stage-creation-sealed.json": b'{"v1":"sealed"}\n',
            ".atlas-stage-creation-published.json": b'{"v1":"published"}\n',
            ".atlas-stage-creation-weyl-context-core-capture-v2-prepared.json":
                b'{"v2":"prepared"}\n',
            ".atlas-stage-creation-weyl-context-core-capture-v2-sealed.json":
                b'{"v2":"sealed"}\n',
            ".atlas-stage-creation-weyl-context-core-capture-v2-published.json":
                b'{"v2":"published"}\n',
            ".atlas-stage-creation-weyl-context-core-capture-v3-prepared.json":
                b'{"v3":"prepared"}\n',
            ".atlas-stage-creation-weyl-context-core-capture-v3-sealed.json":
                b'{"v3":"sealed"}\n',
            ".atlas-stage-creation-weyl-context-core-capture-v3-published.json":
                b'{"v3":"published"}\n',
            ".atlas-stage-creation-weyl-context-core-capture-v4-prepared.json":
                b'{"v4":"prepared"}\n',
            ".atlas-stage-creation-weyl-context-core-capture-v4-sealed.json":
                b'{"v4":"sealed"}\n',
            ".atlas-stage-creation-weyl-context-core-capture-v4-published.json":
                b'{"v4":"published"}\n',
            ".atlas-stage-creation-weyl-context-core-capture-v5-prepared.json":
                b'{"v5":"prepared"}\n',
            ".atlas-stage-creation-weyl-context-core-capture-v5-sealed.json":
                b'{"v5":"sealed"}\n',
            ".atlas-stage-creation-weyl-context-core-capture-v5-published.json":
                b'{"v5":"published"}\n',
            ".atlas-stage-creation-weyl-context-core-capture-v6-prepared.json":
                b'{"v6":"prepared"}\n',
            ".atlas-stage-creation-weyl-context-core-capture-v6-sealed.json":
                b'{"v6":"sealed"}\n',
            ".atlas-stage-creation-weyl-context-core-capture-v6-published.json":
                b'{"v6":"published"}\n',
            ".atlas-stage-creation-weyl-context-core-capture-v7-prepared.json":
                b'{"v7":"prepared"}\n',
            ".atlas-stage-creation-weyl-context-core-capture-v7-sealed.json":
                b'{"v7":"sealed"}\n',
            ".atlas-stage-creation-weyl-context-core-capture-v7-published.json":
                b'{"v7":"published"}\n',
            ".atlas-stage-creation-weyl-context-core-capture-v8-prepared.json":
                b'{"v8":"prepared"}\n',
            ".atlas-stage-creation-weyl-context-core-capture-v8-sealed.json":
                b'{"v8":"sealed"}\n',
            ".atlas-stage-creation-weyl-context-core-capture-v8-published.json":
                b'{"v8":"published"}\n',
            ".atlas-stage-creation-weyl-context-core-before-v3-prepared.json":
                b'{"before_v3":"prepared"}\n',
            ".atlas-stage-creation-weyl-context-core-before-v3-sealed.json":
                b'{"before_v3":"sealed"}\n',
            ".atlas-stage-creation-weyl-context-core-before-v3-published.json":
                b'{"before_v3":"published"}\n',
            ".atlas-stage-creation-weyl-context-core-before-v4-prepared.json":
                b'{"before_v4":"prepared"}\n',
            ".atlas-stage-creation-weyl-context-core-before-v4-sealed.json":
                b'{"before_v4":"sealed"}\n',
            ".atlas-stage-creation-weyl-context-core-before-v4-published.json":
                b'{"before_v4":"published"}\n',
            ".atlas-stage-creation-weyl-context-core-after-v1-prepared.json":
                b'{"after_v1":"prepared"}\n',
            ".atlas-stage-creation-weyl-context-core-after-v1-sealed.json":
                b'{"after_v1":"sealed"}\n',
            ".atlas-stage-creation-weyl-context-core-after-v1-published.json":
                b'{"after_v1":"published"}\n',
            ".atlas-stage-creation-weyl-context-core-after-v2-prepared.json":
                b'{"after_v2":"prepared"}\n',
            ".atlas-stage-creation-weyl-context-core-after-v2-sealed.json":
                b'{"after_v2":"sealed"}\n',
            ".atlas-stage-creation-weyl-context-core-after-v2-published.json":
                b'{"after_v2":"published"}\n',
            ".atlas-stage-creation-weyl-context-core-after-v3-prepared.json":
                b'{"after_v3":"prepared"}\n',
            ".atlas-stage-creation-weyl-context-core-after-v3-sealed.json":
                b'{"after_v3":"sealed"}\n',
            ".atlas-stage-creation-weyl-context-core-after-v3-published.json":
                b'{"after_v3":"published"}\n',
            ".atlas-stage-creation-weyl-context-core-after-v4-prepared.json":
                b'{"after_v4":"prepared"}\n',
            ".atlas-stage-creation-weyl-context-core-after-v4-sealed.json":
                b'{"after_v4":"sealed"}\n',
            ".atlas-stage-creation-weyl-context-core-after-v4-published.json":
                b'{"after_v4":"published"}\n',
            ".atlas-stage-creation-weyl-context-core-after-v5-prepared.json":
                b'{"after_v5":"prepared"}\n',
            ".atlas-stage-creation-weyl-context-core-after-v5-sealed.json":
                b'{"after_v5":"sealed"}\n',
            ".atlas-stage-creation-weyl-context-core-after-v5-published.json":
                b'{"after_v5":"published"}\n',
            ".atlas-stage-creation-weyl-context-core-before-v1-prepared.json":
                b'{"before_v1":"prepared"}\n',
            ".atlas-stage-creation-weyl-context-core-before-v1-sealed.json":
                b'{"before_v1":"sealed"}\n',
            ".atlas-stage-creation-weyl-context-core-before-v1-published.json":
                b'{"before_v1":"published"}\n',
            ".atlas-stage-creation-weyl-context-core-before-v2-prepared.json":
                b'{"before_v2":"prepared"}\n',
            ".atlas-stage-creation-weyl-context-core-before-v2-sealed.json":
                b'{"before_v2":"sealed"}\n',
            ".atlas-stage-creation-weyl-context-core-before-v2-published.json":
                b'{"before_v2":"published"}\n',
            ".atlas-stage-creation-failure-" + "9" * 64 + ".json":
                b'{"old":"renameat2-failure"}\n',
        }
        stage_payloads = {
            ".atlas-stage-creation-transaction.json": b'{"old":"marker"}\n',
            ".atlas-stage-creation.json": b'{"old":"receipt"}\n',
            "weyl-context-core-after-v5-pin.json": b'{"old":"pin"}\n',
            "submission-intent.json": b'{"old":"intent"}\n',
            "submission.json": b'{"old":"submission"}\n',
            "overrides/overrides.json": b'{"old":"overrides"}\n',
            "weyl-context-core-after-v5-3900050.out": b"stray v5 label\n",
        }
        tree_only_payloads = {
            "tree-only.log": b"bound only by the tree seal\n",
        }
        archive_name = next(
            name for name in campaign_payloads
            if name.startswith(".atlas-stage-creation-failure-")
        )
        archive_raw = campaign_payloads.pop(archive_name)
        archive_digest = sha256(archive_raw)
        archive_name = ".atlas-stage-creation-failure-" + archive_digest + ".json"
        campaign_payloads[archive_name] = archive_raw
        if not predecessor.exists():
            for stage_name, _job in PREDECESSOR_LINEAGE:
                (campaign / "stages" / stage_name).mkdir(mode=0o755)
            for name, raw in campaign_payloads.items():
                write_frozen(campaign / name, raw)
            for name, raw in stage_payloads.items():
                write_frozen(predecessor / name, raw)
            for name, raw in tree_only_payloads.items():
                write_frozen(predecessor / name, raw)
            history = [{
                "stage": str(campaign / "stages" / stage_name),
                "script": "hpc/math_weyl_context_core_capture.sbatch",
                "queue_before": [],
                "status": "SUBMITTED",
                "max_outstanding": 10,
                "job": job,
                "pin_sha256": sha256(
                    ("synthetic pin " + stage_name).encode("utf-8")),
                "stage_creation_sha256": sha256(
                    ("synthetic creation " + stage_name).encode("utf-8")),
            } for stage_name, job in PREDECESSOR_LINEAGE]
            record = history[-1]
            record["pin_sha256"] = self.bound_file(
                predecessor / "weyl-context-core-after-v5-pin.json"
            )["sha256"]
            record["stage_creation_sha256"] = self.bound_file(
                predecessor / ".atlas-stage-creation.json"
            )["sha256"]
            (campaign / ".atlas-progressive-submit.json").write_bytes(
                json_bytes(history))
        else:
            history = json.loads(
                (campaign / ".atlas-progressive-submit.json").read_text())
            self.assertIn(len(history), (24, 25))
            self.assertEqual(
                [(Path(row["stage"]).name, row["job"])
                 for row in history[:24]],
                list(PREDECESSOR_LINEAGE),
            )
            if len(history) == 25:
                self.assertEqual(
                    Path(history[-1]["stage"]).name, ACTIVE_STAGE_NAME)
            record = history[23]
        value = predecessor.stat()
        stage_descriptor = progressive_submit._open_directory(predecessor)
        try:
            tree_binding = progressive_submit._predecessor_tree_binding(
                stage_descriptor)
        finally:
            os.close(stage_descriptor)
        self.assertGreater(tree_binding["stage_tree_files"], len(stage_payloads))
        descriptor = {
            "schema": "atlas-stage-creation-predecessor-v14",
            "stage": str(predecessor),
            "stage_device": value.st_dev,
            "stage_inode": value.st_ino,
            "record": record,
            "campaign_files": {
                name: self.bound_file(campaign / name)
                for name in campaign_payloads
            },
            "stage_files": {
                name: self.bound_file(predecessor / name)
                for name in stage_payloads
            },
        }
        descriptor.update(tree_binding)
        return predecessor, descriptor

    def campaign_with_predecessor(self, folder):
        campaign = self.campaign(folder)
        predecessor, descriptor = self.predecessor_fixture(campaign)
        return campaign, predecessor, descriptor

    def payload(self, folder, campaign, *, suffix=b""):
        _predecessor, predecessor_state = self.predecessor_fixture(campaign)
        root = Path(folder) / ("payload" + suffix.decode("ascii"))
        overrides = root / "overrides"
        files = {
            SCRIPT_PATH: b"#!/bin/bash\n#SBATCH --nodes=1\n" + suffix,
            "tests/math/generics/stage_creation_probe.atlas":
                b'prints("stage creation probe")\nquit\n' + suffix,
        }
        for name, raw in files.items():
            write_frozen(overrides / name, raw)
        inputs = {name: sha256(raw) for name, raw in files.items()}
        manifest_raw = json_bytes(inputs)
        write_frozen(overrides / "overrides.json", manifest_raw)
        ledger_raw = (campaign / ".atlas-progressive-submit.json").read_bytes()
        contract = {
            "schema": "atlas-stage-creation-contract-v14",
            "campaign": str(campaign.resolve()),
            "stage_name": ACTIVE_STAGE_NAME,
            "predecessor_ledger_sha256": sha256(ledger_raw),
            "predecessor_state": predecessor_state,
            "overrides_sha256": sha256(manifest_raw),
            "inputs": inputs,
            "script": {"path": SCRIPT_PATH, "sha256": inputs[SCRIPT_PATH]},
            "pin": {
                "path": PIN_PATH,
                "schema": "atlas-weyl-context-g2-pin-v1",
                "stage_creation_key": "stage_creation",
            },
            "lifecycle": {
                "stage": ACTIVE_STAGE_NAME,
                "predecessor_stage": PREDECESSOR_STAGE_NAME,
                "changed_input_reasons": [
                    "Install one receipt-bound synthetic test payload."
                ],
                "retention_class": "ACTIVE_GATE_COMPACT",
                "retirement_condition": (
                    "Retain until independent review and explicit exact-path "
                    "retirement authorization."
                ),
            },
        }
        return root, contract

    def published(self, campaign):
        stage = campaign / "stages" / ACTIVE_STAGE_NAME
        receipt_path = stage / STAGE_CREATION_RECEIPT
        receipt_raw = receipt_path.read_bytes()
        return stage, receipt_raw, json.loads(receipt_raw)

    def patch_home(self, campaign):
        return patch("campaign_workspace.HPC_HOME", campaign.parent)

    def publish_bytes(self, parent, name, raw, *, fault=None):
        descriptor = os.open(
            parent,
            os.O_RDONLY | getattr(os, "O_DIRECTORY", 0)
            | getattr(os, "O_CLOEXEC", 0),
        )
        try:
            return progressive_submit._publish_bytes_at(
                descriptor, name, raw, _fault=fault)
        finally:
            os.close(descriptor)

    def prior_failure(self, campaign, contract, *, legacy=True):
        old_contract = copy.deepcopy(contract)
        if legacy:
            old_contract["lifecycle"]["changed_input_reasons"].append(
                "Legacy renameat2 publication attempt.")
        contract_sha256 = sha256(json_bytes(old_contract))
        transaction = (
            ".atlas-stage-creation-" + contract_sha256[:24] + ".txn")
        prepared = {
            "schema": "atlas-stage-creation-event-v1",
            "status": "PREPARED",
            "sequence": 1,
            "contract": old_contract,
            "contract_sha256": contract_sha256,
            "transaction": transaction,
        }
        raw = json_bytes(prepared)
        digest = sha256(raw)
        descriptor = {
            "schema": "atlas-stage-creation-prior-failure-v1",
            "temporary": transaction_temp_name(
                STAGE_CREATION_PREPARED, raw),
            "archive": ".atlas-stage-creation-failure-" + digest + ".json",
            "sha256": digest,
            "bytes": len(raw),
            "mode": "0444",
            "destination": STAGE_CREATION_PREPARED,
            "contract_sha256": contract_sha256,
            "transaction": transaction,
        }
        self.assertEqual(set(descriptor), PRIOR_FAILURE_KEYS)
        return raw, descriptor, old_contract

    def snapshot(self, root):
        result = {}
        if not root.exists() and not root.is_symlink():
            return result
        for path in [root, *sorted(root.rglob("*"))]:
            value = path.lstat()
            relative = "." if path == root else str(path.relative_to(root))
            row = {
                "mode": stat.S_IFMT(value.st_mode),
                "permissions": stat.S_IMODE(value.st_mode),
                "device": value.st_dev,
                "inode": value.st_ino,
                "nlink": value.st_nlink,
                "mtime_ns": value.st_mtime_ns,
            }
            if stat.S_ISREG(value.st_mode):
                raw = path.read_bytes()
                row.update(bytes=len(raw), sha256=sha256(raw))
            elif stat.S_ISLNK(value.st_mode):
                row["target"] = os.readlink(path)
            result[relative] = row
        return result

    def test_fixed_contract_and_published_records_are_exact(self):
        with tempfile.TemporaryDirectory() as folder:
            campaign = self.campaign(folder)
            payload, contract = self.payload(folder, campaign)
            self.assertEqual(set(contract), CONTRACT_KEYS)
            with self.patch_home(campaign):
                stage, receipt_sha256 = create_fixed_stage(
                    campaign, payload, contract)
                receipt = validate_stage_creation(stage, receipt_sha256)

            prepared = json.loads(
                (campaign / STAGE_CREATION_PREPARED).read_text())
            published = json.loads(
                (campaign / STAGE_CREATION_PUBLISHED).read_text())
            sealed_raw = (campaign / STAGE_CREATION_SEALED).read_bytes()
            sealed = json.loads(sealed_raw)
            receipt_raw = (stage / STAGE_CREATION_RECEIPT).read_bytes()
            marker = json.loads((stage / STAGE_CREATION_MARKER).read_text())
            stage_stat = stage.stat()

            self.assertEqual(stage, campaign / "stages" / ACTIVE_STAGE_NAME)
            self.assertEqual(receipt_sha256, sha256(receipt_raw))
            self.assertEqual(set(receipt), RECEIPT_KEYS)
            self.assertEqual(set(prepared), PREPARED_KEYS)
            self.assertEqual(set(sealed), SEALED_KEYS)
            self.assertEqual(set(published), PUBLISHED_KEYS)
            self.assertEqual(prepared["schema"], "atlas-stage-creation-event-v1")
            self.assertEqual(prepared["status"], "PREPARED")
            self.assertEqual(prepared["sequence"], 1)
            self.assertEqual(prepared["contract"], contract)
            self.assertEqual(sealed["schema"], "atlas-stage-creation-event-v1")
            self.assertEqual(sealed["status"], "SEALED")
            self.assertEqual(sealed["sequence"], 2)
            # ``hidden_*`` is retained as a frozen legacy field name, but the
            # target-first protocol binds it to the already-visible final
            # stage inode before PUBLISHED commits that inode.
            self.assertEqual(sealed["hidden_device"], stage_stat.st_dev)
            self.assertEqual(sealed["hidden_inode"], stage_stat.st_ino)
            self.assertEqual(published["schema"], "atlas-stage-creation-event-v1")
            self.assertEqual(published["status"], "PUBLISHED")
            self.assertEqual(published["sequence"], 3)
            self.assertEqual(receipt["schema"], "atlas-stage-creation-receipt-v1")
            self.assertEqual(receipt["status"], "PUBLISHED")
            self.assertEqual(receipt["contract"], contract)
            self.assertEqual(receipt["stage_device"], stage_stat.st_dev)
            self.assertEqual(receipt["stage_inode"], stage_stat.st_ino)
            self.assertEqual(
                (published["stage_device"], published["stage_inode"]),
                (stage_stat.st_dev, stage_stat.st_ino),
            )
            self.assertEqual(
                ((stage / STAGE_CREATION_MARKER).parent.stat().st_dev,
                 (stage / STAGE_CREATION_MARKER).parent.stat().st_ino),
                (stage_stat.st_dev, stage_stat.st_ino),
            )
            self.assertEqual(marker, {
                "schema": "atlas-stage-creation-transaction-v1",
                "status": "PREPARED",
                "contract_sha256": receipt["contract_sha256"],
                "prepared_sha256": receipt["prepared_sha256"],
                "transaction": receipt["transaction"],
            })
            self.assertEqual(published["receipt_sha256"], receipt_sha256)
            self.assertEqual(published["sealed_sha256"], sha256(sealed_raw))
            self.assertEqual(published["contract_sha256"],
                             receipt["contract_sha256"])
            self.assertEqual(published["prepared_sha256"],
                             receipt["prepared_sha256"])
            self.assertEqual(published["transaction"], receipt["transaction"])
            self.assertEqual(published["stage"], str(stage))
            for path in (
                    campaign / STAGE_CREATION_PREPARED,
                    campaign / STAGE_CREATION_SEALED,
                    campaign / STAGE_CREATION_PUBLISHED,
                    stage / STAGE_CREATION_MARKER,
                    stage / STAGE_CREATION_RECEIPT):
                self.assertEqual(stat.S_IMODE(path.stat().st_mode), 0o444)
                self.assertEqual(path.stat().st_nlink, 1)

    def test_contract_mutations_fail_before_creating_stage_state(self):
        mutations = (
            lambda value: value.pop("script"),
            lambda value: value.update(extra=True),
            lambda value: value.update(schema="atlas-stage-creation-contract-v11"),
            lambda value: value.update(campaign="/tmp/not-the-campaign"),
            lambda value: value.update(stage_name="weyl-context-core-capture-r2"),
            lambda value: value.update(predecessor_ledger_sha256="0" * 64),
            lambda value: value.update(predecessor_state=None),
            lambda value: value.update(predecessor_state={"changed": True}),
            lambda value: value["predecessor_state"].update(
                schema="atlas-stage-creation-predecessor-v11"),
            lambda value: value["predecessor_state"]["campaign_files"].pop(
                ".atlas-stage-creation-prepared.json"),
            lambda value: value["predecessor_state"]["campaign_files"].pop(
                ".atlas-stage-creation-weyl-context-core-capture-v2-"
                "prepared.json"),
            lambda value: value["predecessor_state"]["campaign_files"].pop(
                ".atlas-stage-creation-weyl-context-core-capture-v3-"
                "prepared.json"),
            lambda value: value["predecessor_state"]["campaign_files"].pop(
                ".atlas-stage-creation-weyl-context-core-capture-v4-"
                "prepared.json"),
            lambda value: value["predecessor_state"]["campaign_files"].pop(
                ".atlas-stage-creation-weyl-context-core-capture-v5-"
                "prepared.json"),
            lambda value: value["predecessor_state"]["campaign_files"].pop(
                ".atlas-stage-creation-weyl-context-core-capture-v6-"
                "prepared.json"),
            lambda value: value["predecessor_state"]["campaign_files"].pop(
                ".atlas-stage-creation-weyl-context-core-capture-v7-"
                "prepared.json"),
            lambda value: value["predecessor_state"]["campaign_files"].pop(
                ".atlas-stage-creation-weyl-context-core-capture-v8-"
                "prepared.json"),
            lambda value: value["predecessor_state"]["campaign_files"].pop(
                ".atlas-stage-creation-weyl-context-core-before-v1-"
                "prepared.json"),
            lambda value: value["predecessor_state"]["campaign_files"].pop(
                ".atlas-stage-creation-weyl-context-core-before-v3-"
                "prepared.json"),
            lambda value: value["predecessor_state"]["campaign_files"].pop(
                ".atlas-stage-creation-weyl-context-core-before-v2-"
                "prepared.json"),
            lambda value: value["predecessor_state"].update(
                stage_tree_sha256="0" * 64),
            lambda value: value["predecessor_state"].update(
                stage_tree_files=1.0),
            lambda value: value["predecessor_state"].update(
                stage_tree_directories=True),
            lambda value: value["predecessor_state"].update(
                stage_tree_bytes=-1),
            lambda value: value["predecessor_state"]["record"].update(
                pin_sha256="0" * 64),
            lambda value: value["predecessor_state"]["record"].update(
                stage_creation_sha256="0" * 64),
            lambda value: value.update(overrides_sha256="0" * 64),
            lambda value: value["inputs"].update({SCRIPT_PATH: "0" * 64}),
            lambda value: value["script"].update(sha256="0" * 64),
            lambda value: value["pin"].update(
                schema="atlas-weyl-context-core-capture-pin-v7"),
            lambda value: value["pin"].update(stage_creation_key="receipt"),
            lambda value: value["lifecycle"].update(stage="retry-stage"),
            lambda value: value["lifecycle"].update(
                predecessor_stage="another-stage"),
        )
        for index, mutate in enumerate(mutations):
            with self.subTest(index=index), tempfile.TemporaryDirectory() as folder:
                campaign = self.campaign(folder)
                payload, contract = self.payload(folder, campaign)
                changed = copy.deepcopy(contract)
                mutate(changed)
                with self.patch_home(campaign), self.assertRaises(ValueError):
                    create_fixed_stage(campaign, payload, changed)
                self.assertEqual(
                    {path.name for path in (campaign / "stages").iterdir()},
                    {stage for stage, _job in PREDECESSOR_LINEAGE},
                )
                self.assertFalse((campaign / STAGE_CREATION_PREPARED).exists())
                self.assertFalse((campaign / STAGE_CREATION_SEALED).exists())
                self.assertFalse((campaign / STAGE_CREATION_PUBLISHED).exists())

        lineage_mutations = (
            (0, "job", "3999990"),
            (len(PREDECESSOR_LINEAGE) - 1, "job", "3999991"),
        )
        for index, field, replacement in lineage_mutations:
            with self.subTest(lineage=(index, field)), \
                    tempfile.TemporaryDirectory() as folder:
                campaign = self.campaign(folder)
                payload, contract = self.payload(folder, campaign)
                history_path = campaign / ".atlas-progressive-submit.json"
                history = json.loads(history_path.read_text())
                history[index][field] = replacement
                changed_raw = json_bytes(history)
                history_path.write_bytes(changed_raw)
                contract["predecessor_ledger_sha256"] = sha256(changed_raw)
                if index == len(PREDECESSOR_LINEAGE) - 1:
                    contract["predecessor_state"]["record"] = history[index]
                with self.patch_home(campaign), self.assertRaises(ValueError):
                    create_fixed_stage(campaign, payload, contract)
                self.assertEqual(
                    {path.name for path in (campaign / "stages").iterdir()},
                    {stage for stage, _job in PREDECESSOR_LINEAGE},
                )
                self.assertFalse((campaign / STAGE_CREATION_PREPARED).exists())
                self.assertFalse((campaign / STAGE_CREATION_SEALED).exists())
                self.assertFalse((campaign / STAGE_CREATION_PUBLISHED).exists())

    def test_before_v3_creation_preserves_exact_failed_before_v1_predecessor(self):
        with tempfile.TemporaryDirectory() as folder:
            campaign, predecessor, descriptor = \
                self.campaign_with_predecessor(folder)
            payload, contract = self.payload(folder, campaign)
            contract["predecessor_state"] = descriptor
            predecessor_before = self.snapshot(predecessor)
            campaign_before = {
                name: self.snapshot(campaign / name)
                for name in descriptor["campaign_files"]
            }
            with self.patch_home(campaign):
                stage, receipt_sha = create_fixed_stage(
                    campaign, payload, contract)
                receipt = validate_stage_creation(
                    stage, receipt_sha, contract)
            self.assertEqual(stage.name, "weyl-context-g2-v1")
            self.assertEqual(receipt["contract"]["predecessor_state"],
                             descriptor)
            self.assertEqual(
                receipt["contract"]["pin"]["schema"],
                "atlas-weyl-context-g2-pin-v1",
            )
            required_history = {
                ".atlas-stage-creation-prepared.json",
                ".atlas-stage-creation-sealed.json",
                ".atlas-stage-creation-published.json",
                ".atlas-stage-creation-weyl-context-core-capture-v2-"
                "prepared.json",
                ".atlas-stage-creation-weyl-context-core-capture-v2-"
                "sealed.json",
                ".atlas-stage-creation-weyl-context-core-capture-v2-"
                "published.json",
                ".atlas-stage-creation-weyl-context-core-capture-v3-"
                "prepared.json",
                ".atlas-stage-creation-weyl-context-core-capture-v3-"
                "sealed.json",
                ".atlas-stage-creation-weyl-context-core-capture-v3-"
                "published.json",
                ".atlas-stage-creation-weyl-context-core-capture-v4-"
                "prepared.json",
                ".atlas-stage-creation-weyl-context-core-capture-v4-"
                "sealed.json",
                ".atlas-stage-creation-weyl-context-core-capture-v4-"
                "published.json",
                ".atlas-stage-creation-weyl-context-core-capture-v5-"
                "prepared.json",
                ".atlas-stage-creation-weyl-context-core-capture-v5-"
                "sealed.json",
                ".atlas-stage-creation-weyl-context-core-capture-v5-"
                "published.json",
                ".atlas-stage-creation-weyl-context-core-capture-v6-"
                "prepared.json",
                ".atlas-stage-creation-weyl-context-core-capture-v6-"
                "sealed.json",
                ".atlas-stage-creation-weyl-context-core-capture-v6-"
                "published.json",
                ".atlas-stage-creation-weyl-context-core-capture-v7-"
                "prepared.json",
                ".atlas-stage-creation-weyl-context-core-capture-v7-"
                "sealed.json",
                ".atlas-stage-creation-weyl-context-core-capture-v7-"
                "published.json",
                ".atlas-stage-creation-weyl-context-core-capture-v8-"
                "prepared.json",
                ".atlas-stage-creation-weyl-context-core-capture-v8-"
                "sealed.json",
                ".atlas-stage-creation-weyl-context-core-capture-v8-"
                "published.json",
                ".atlas-stage-creation-weyl-context-core-before-v3-"
                "prepared.json",
                ".atlas-stage-creation-weyl-context-core-before-v3-"
                "sealed.json",
                ".atlas-stage-creation-weyl-context-core-before-v3-"
                "published.json",
                ".atlas-stage-creation-weyl-context-core-before-v1-"
                "prepared.json",
                ".atlas-stage-creation-weyl-context-core-before-v1-"
                "sealed.json",
                ".atlas-stage-creation-weyl-context-core-before-v1-"
                "published.json",
                ".atlas-stage-creation-weyl-context-core-before-v2-"
                "prepared.json",
                ".atlas-stage-creation-weyl-context-core-before-v2-"
                "sealed.json",
                ".atlas-stage-creation-weyl-context-core-before-v2-"
                "published.json",
                ".atlas-stage-creation-weyl-context-core-before-v4-"
                "prepared.json",
                ".atlas-stage-creation-weyl-context-core-before-v4-"
                "sealed.json",
                ".atlas-stage-creation-weyl-context-core-before-v4-"
                "published.json",
                ".atlas-stage-creation-weyl-context-core-after-v1-"
                "prepared.json",
                ".atlas-stage-creation-weyl-context-core-after-v1-"
                "sealed.json",
                ".atlas-stage-creation-weyl-context-core-after-v1-"
                "published.json",
                ".atlas-stage-creation-weyl-context-core-after-v2-"
                "prepared.json",
                ".atlas-stage-creation-weyl-context-core-after-v2-"
                "sealed.json",
                ".atlas-stage-creation-weyl-context-core-after-v2-"
                "published.json",
                ".atlas-stage-creation-weyl-context-core-after-v3-"
                "prepared.json",
                ".atlas-stage-creation-weyl-context-core-after-v3-"
                "sealed.json",
                ".atlas-stage-creation-weyl-context-core-after-v3-"
                "published.json",
                ".atlas-stage-creation-weyl-context-core-after-v4-"
                "prepared.json",
                ".atlas-stage-creation-weyl-context-core-after-v4-"
                "sealed.json",
                ".atlas-stage-creation-weyl-context-core-after-v4-"
                "published.json",
                ".atlas-stage-creation-weyl-context-core-after-v5-"
                "prepared.json",
                ".atlas-stage-creation-weyl-context-core-after-v5-"
                "sealed.json",
                ".atlas-stage-creation-weyl-context-core-after-v5-"
                "published.json",
            }
            failure_history = {
                name for name in descriptor["campaign_files"]
                if name.startswith(".atlas-stage-creation-failure-")
            }
            self.assertTrue(failure_history)
            self.assertEqual(
                set(descriptor["campaign_files"]),
                required_history | failure_history,
            )
            for name, binding in descriptor["campaign_files"].items():
                if name.startswith(".atlas-stage-creation-failure-"):
                    self.assertEqual(
                        name.removeprefix(
                            ".atlas-stage-creation-failure-"
                        ).removesuffix(".json"),
                        binding["sha256"],
                    )
            self.assertEqual(self.snapshot(predecessor), predecessor_before)
            self.assertEqual(
                {
                    name: self.snapshot(campaign / name)
                    for name in descriptor["campaign_files"]
                },
                campaign_before,
            )

    def test_before_v1_predecessor_tampering_fails_before_new_stage_state(self):
        cases = (
            "campaign-file", "v6-scoped-campaign-file",
            "v7-scoped-campaign-file", "stage-file",
            "stage-inode", "ledger-tail", "extra-history-event",
            "archive-link", "tree-only-file", "extra-stage-file",
            "extra-stage-directory", "stage-symlink", "stage-hardlink",
        )
        for case in cases:
            with self.subTest(case=case), \
                    tempfile.TemporaryDirectory() as folder:
                campaign, predecessor, descriptor = \
                    self.campaign_with_predecessor(folder)
                payload, contract = self.payload(folder, campaign)
                contract["predecessor_state"] = descriptor
                if case == "campaign-file":
                    target = campaign / ".atlas-stage-creation-prepared.json"
                    target.chmod(0o644)
                    target.write_bytes(b'{"changed":true}\n')
                    target.chmod(0o444)
                elif case in (
                        "v6-scoped-campaign-file",
                        "v7-scoped-campaign-file"):
                    version = case.split("-", 1)[0]
                    target = campaign / (
                        ".atlas-stage-creation-weyl-context-core-capture-"
                        + version + "-prepared.json"
                    )
                    target.chmod(0o644)
                    target.write_bytes(b'{"changed":true}\n')
                    target.chmod(0o444)
                elif case == "stage-file":
                    target = predecessor / ".atlas-stage-creation.json"
                    target.chmod(0o644)
                    target.write_bytes(b'{"changed":true}\n')
                    target.chmod(0o444)
                elif case == "stage-inode":
                    predecessor.rename(predecessor.with_name("displaced-before-v3"))
                    predecessor.mkdir(mode=0o755)
                elif case == "ledger-tail":
                    changed = copy.deepcopy(descriptor["record"])
                    changed["job"] = "700002"
                    (campaign / ".atlas-progressive-submit.json").write_bytes(
                        json_bytes([changed]))
                elif case == "extra-history-event":
                    write_frozen(
                        campaign / ".atlas-stage-creation-unregistered.json",
                        b'{}\n',
                    )
                elif case == "archive-link":
                    archive = next(
                        campaign / name for name in descriptor["campaign_files"]
                        if name.startswith(".atlas-stage-creation-failure-")
                    )
                    os.link(archive, campaign / "unexpected-archive-link")
                elif case == "tree-only-file":
                    target = predecessor / "tree-only.log"
                    target.chmod(0o644)
                    target.write_bytes(b"changed but not individually bound\n")
                    target.chmod(0o444)
                elif case == "extra-stage-file":
                    write_frozen(predecessor / "unexpected-file", b"extra\n")
                elif case == "extra-stage-directory":
                    (predecessor / "unexpected-directory").mkdir()
                elif case == "stage-symlink":
                    (predecessor / "unexpected-link").symlink_to(
                        "tree-only.log")
                else:
                    os.link(
                        predecessor / "tree-only.log",
                        predecessor / "unexpected-hardlink",
                    )
                with self.patch_home(campaign), self.assertRaises(ValueError):
                    create_fixed_stage(campaign, payload, contract)
                self.assertFalse(
                    (campaign / "stages" / ACTIVE_STAGE_NAME).exists())
                self.assertFalse((campaign / STAGE_CREATION_PREPARED).exists())
                self.assertFalse((campaign / STAGE_CREATION_SEALED).exists())
                self.assertFalse((campaign / STAGE_CREATION_PUBLISHED).exists())

    def test_raw_target_extra_sibling_and_symlinks_are_never_adopted(self):
        cases = ("target", "sibling", "target-symlink", "stages-symlink")
        for case in cases:
            with self.subTest(case=case), tempfile.TemporaryDirectory() as folder:
                base = Path(folder)
                outside = base / "outside"
                outside.mkdir()
                sentinel = outside / "sentinel"
                sentinel.write_text("unchanged\n")
                if case == "stages-symlink":
                    campaign = self.campaign(folder)
                    payload, contract = self.payload(folder, campaign)
                    stages = campaign / "stages"
                    stages.rename(campaign / "stages.displaced")
                    (campaign / "stages").symlink_to(
                        outside, target_is_directory=True)
                else:
                    campaign = self.campaign(folder)
                    payload, contract = self.payload(folder, campaign)
                    stages = campaign / "stages"
                    if case == "target":
                        (stages / ACTIVE_STAGE_NAME).mkdir()
                    elif case == "sibling":
                        (stages / "unregistered-sibling").mkdir()
                    else:
                        (stages / ACTIVE_STAGE_NAME).symlink_to(
                            outside, target_is_directory=True)
                outside_before = self.snapshot(outside)
                with self.patch_home(campaign), self.assertRaises(ValueError):
                    create_fixed_stage(campaign, payload, contract)
                self.assertEqual(self.snapshot(outside), outside_before)
                self.assertFalse((campaign / STAGE_CREATION_PREPARED).exists())
                self.assertFalse((campaign / STAGE_CREATION_SEALED).exists())
                self.assertFalse((campaign / STAGE_CREATION_PUBLISHED).exists())

    def test_same_spec_is_byte_and_inode_idempotent(self):
        with tempfile.TemporaryDirectory() as folder:
            campaign = self.campaign(folder)
            payload, contract = self.payload(folder, campaign)
            with self.patch_home(campaign):
                first = create_fixed_stage(campaign, payload, contract)
                before = self.snapshot(campaign)
                second = create_fixed_stage(campaign, payload, contract)
                after = self.snapshot(campaign)
                receipt = validate_stage_creation(first[0], first[1])
            self.assertEqual(first, second)
            self.assertEqual(before, after)
            self.assertEqual(receipt["contract"], contract)
            hidden = campaign / "stages" / receipt["transaction"]
            self.assertFalse(hidden.exists())

    def test_changed_spec_cannot_replace_a_published_stage(self):
        with tempfile.TemporaryDirectory() as folder:
            campaign = self.campaign(folder)
            payload, contract = self.payload(folder, campaign)
            changed_payload, changed_contract = self.payload(
                folder, campaign, suffix=b"-changed")
            with self.patch_home(campaign):
                stage, _ = create_fixed_stage(campaign, payload, contract)
                before = self.snapshot(campaign)
                with self.assertRaises(ValueError):
                    create_fixed_stage(
                        campaign, changed_payload, changed_contract)
                lifecycle_only = copy.deepcopy(contract)
                lifecycle_only["lifecycle"]["changed_input_reasons"].append(
                    "A second incompatible reason must not replace the leaf.")
                with self.assertRaises(ValueError):
                    create_fixed_stage(campaign, payload, lifecycle_only)
                after = self.snapshot(campaign)
            self.assertEqual(before, after)
            self.assertEqual(stage.name, ACTIVE_STAGE_NAME)
            self.assertEqual(
                {path.name for path in (campaign / "stages").iterdir()},
                ({stage for stage, _job in PREDECESSOR_LINEAGE}
                 | {ACTIVE_STAGE_NAME}),
            )

    def test_process_crashes_preserve_recoverable_and_ambiguous_states(self):
        expectations = {
            "after_prepared": "recover",
            "after_target_claim": "ambiguous",
            "after_receipt": "recover",
            "after_sealed": "recover",
        }
        context = multiprocessing.get_context("spawn")
        for checkpoint, disposition in expectations.items():
            with self.subTest(checkpoint=checkpoint), \
                    tempfile.TemporaryDirectory() as folder:
                campaign = self.campaign(folder)
                payload, contract = self.payload(folder, campaign)
                process = context.Process(
                    target=crash_worker,
                    args=(str(campaign), str(payload), contract, checkpoint),
                )
                process.start()
                process.join(20)
                if process.is_alive():
                    process.terminate()
                    process.join(5)
                    self.fail("stage-creation crash worker did not terminate")
                self.assertEqual(process.exitcode, 73)
                with self.patch_home(campaign):
                    if disposition == "recover":
                        stage, receipt_sha256 = create_fixed_stage(
                            campaign, payload, contract)
                        self.assertEqual(
                            validate_stage_creation(
                                stage, receipt_sha256)["status"],
                            "PUBLISHED",
                        )
                    else:
                        target = campaign / "stages" / ACTIVE_STAGE_NAME
                        before = self.snapshot(target)
                        self.assertTrue(target.is_dir())
                        with self.assertRaisesRegex(ValueError, "reconcil|ambiguous"):
                            create_fixed_stage(campaign, payload, contract)
                        self.assertEqual(self.snapshot(target), before)

    def test_registered_fault_points_resume_without_new_transaction(self):
        for checkpoint in (
                "after_marker", "after_payload", "before_published",
                "after_published"):
            with self.subTest(checkpoint=checkpoint), \
                    tempfile.TemporaryDirectory() as folder:
                campaign = self.campaign(folder)
                payload, contract = self.payload(folder, campaign)

                def fail_here(observed, _context):
                    if observed == checkpoint:
                        raise RuntimeError("injected " + checkpoint)

                with self.patch_home(campaign):
                    with self.assertRaisesRegex(RuntimeError, checkpoint):
                        create_fixed_stage(
                            campaign, payload, contract, _fault=fail_here)
                    prepared_before = (
                        campaign / STAGE_CREATION_PREPARED).read_bytes()
                    transaction = json.loads(prepared_before)["transaction"]
                    stage, receipt_sha256 = create_fixed_stage(
                        campaign, payload, contract)
                    receipt = validate_stage_creation(stage, receipt_sha256)
                self.assertEqual(receipt["transaction"], transaction)
                self.assertEqual(
                    (campaign / STAGE_CREATION_PREPARED).read_bytes(),
                    prepared_before,
                )

    def test_forged_final_after_prepared_is_never_adopted_without_seal(self):
        with tempfile.TemporaryDirectory() as folder:
            campaign = self.campaign(folder)
            payload, contract = self.payload(folder, campaign)

            def stop_after_prepared(checkpoint, _context):
                if checkpoint == "after_prepared":
                    raise RuntimeError("prepared sentinel")

            with self.patch_home(campaign), self.assertRaisesRegex(
                    RuntimeError, "prepared sentinel"):
                create_fixed_stage(
                    campaign, payload, contract, _fault=stop_after_prepared)
            target = campaign / "stages" / ACTIVE_STAGE_NAME
            target.mkdir()
            sentinel = target / "forged"
            sentinel.write_text("must remain\n")
            before = self.snapshot(target)
            with self.patch_home(campaign), self.assertRaisesRegex(
                    ValueError, "claim|marker|ambiguous|unregistered"):
                create_fixed_stage(campaign, payload, contract)
            self.assertEqual(self.snapshot(target), before)
            self.assertFalse((campaign / STAGE_CREATION_SEALED).exists())
            self.assertFalse((campaign / STAGE_CREATION_PUBLISHED).exists())

    def test_file_publication_recovers_every_hardlink_window_without_rename(self):
        raw = b'{"immutable":"publication"}\n'
        checkpoints = {
            "after_file_link": 2,
            "after_file_parent_fsync": 2,
            "after_file_unlink": 1,
        }
        for checkpoint, interrupted_links in checkpoints.items():
            with self.subTest(checkpoint=checkpoint), \
                    tempfile.TemporaryDirectory() as folder:
                parent = Path(folder)
                name = "record.json"
                target = parent / name
                temporary = parent / transaction_temp_name(name, raw)
                observed = []
                parent_identity = (parent.stat().st_dev, parent.stat().st_ino)
                parent_sync_states = []
                real_fsync = os.fsync

                def observe_fsync(open_descriptor):
                    value = os.fstat(open_descriptor)
                    if (value.st_dev, value.st_ino) == parent_identity:
                        parent_sync_states.append(
                            (target.exists(), temporary.exists()))
                    return real_fsync(open_descriptor)

                def stop_here(value, context):
                    observed.append((value, context))
                    if value == checkpoint:
                        if value == "after_file_parent_fsync":
                            self.assertIn((True, True), parent_sync_states)
                        raise RuntimeError("injected " + checkpoint)

                no_rename = AssertionError(
                    "hardlink publication must not call any rename primitive")
                with patch.object(
                        progressive_submit, "_rename_noreplace", create=True,
                        side_effect=no_rename), \
                        patch("progressive_submit.os.rename",
                              side_effect=no_rename), \
                        patch("progressive_submit.os.replace",
                              side_effect=no_rename), \
                        patch("progressive_submit.os.fsync",
                              side_effect=observe_fsync), \
                        self.assertRaisesRegex(RuntimeError, checkpoint):
                    self.publish_bytes(
                        parent, name, raw, fault=stop_here)

                self.assertIn(checkpoint, [row[0] for row in observed])
                self.assertTrue(target.is_file())
                self.assertEqual(target.read_bytes(), raw)
                self.assertEqual(stat.S_IMODE(target.stat().st_mode), 0o444)
                self.assertEqual(target.stat().st_nlink, interrupted_links)
                if interrupted_links == 2:
                    self.assertTrue(temporary.is_file())
                    self.assertEqual(temporary.read_bytes(), raw)
                    self.assertEqual(
                        (temporary.stat().st_dev, temporary.stat().st_ino),
                        (target.stat().st_dev, target.stat().st_ino),
                    )
                else:
                    self.assertFalse(temporary.exists())

                published_inode = (target.stat().st_dev, target.stat().st_ino)
                recovery_parent_syncs = []

                def observe_recovery_fsync(open_descriptor):
                    value = os.fstat(open_descriptor)
                    if (value.st_dev, value.st_ino) == parent_identity:
                        recovery_parent_syncs.append(
                            (target.exists(), temporary.exists()))
                    return real_fsync(open_descriptor)

                with patch.object(
                        progressive_submit, "_rename_noreplace", create=True,
                        side_effect=no_rename), \
                        patch("progressive_submit.os.rename",
                              side_effect=no_rename), \
                        patch("progressive_submit.os.replace",
                              side_effect=no_rename), \
                        patch("progressive_submit.os.fsync",
                              side_effect=observe_recovery_fsync):
                    self.publish_bytes(parent, name, raw)
                    before = self.snapshot(parent)
                    self.publish_bytes(parent, name, raw)
                self.assertEqual(self.snapshot(parent), before)
                self.assertFalse(temporary.exists())
                self.assertEqual(target.stat().st_nlink, 1)
                self.assertIn((True, False), recovery_parent_syncs)
                self.assertEqual(
                    (target.stat().st_dev, target.stat().st_ino),
                    published_inode,
                )

    def test_file_publication_rejects_unsafe_residuals_without_mutation(self):
        raw = b"immutable publication\n"
        cases = (
            "different-inodes", "extra-link", "wrong-mode", "wrong-bytes",
            "unknown-residual", "temp-symlink", "temp-directory", "temp-fifo",
        )
        for case in cases:
            with self.subTest(case=case), \
                    tempfile.TemporaryDirectory() as folder:
                parent = Path(folder)
                name = "record.json"
                target = parent / name
                temporary = parent / transaction_temp_name(name, raw)
                if case == "different-inodes":
                    write_frozen(temporary, raw)
                    write_frozen(target, raw)
                    self.assertNotEqual(temporary.stat().st_ino,
                                        target.stat().st_ino)
                elif case == "extra-link":
                    write_frozen(temporary, raw)
                    os.link(temporary, target)
                    os.link(temporary, parent / "third-link")
                elif case == "wrong-mode":
                    temporary.write_bytes(raw)
                    temporary.chmod(0o644)
                elif case == "wrong-bytes":
                    write_frozen(temporary, raw + b"changed")
                elif case == "temp-symlink":
                    (parent / "symlink-target").write_bytes(raw)
                    temporary.symlink_to(parent / "symlink-target")
                elif case == "temp-directory":
                    temporary.mkdir()
                elif case == "temp-fifo":
                    os.mkfifo(temporary, 0o444)
                else:
                    write_frozen(
                        parent / (".atlas-publish-" + "f" * 64 + ".tmp"),
                        raw,
                    )
                before = self.snapshot(parent)
                no_rename = AssertionError(
                    "rejected residual must not enter a rename fallback")
                real_open = os.open

                def reject_special_open(path, flags, mode=0o777, *, dir_fd=None):
                    if (case.startswith("temp-")
                            and os.fspath(path) == temporary.name):
                        raise AssertionError(
                            "special residual must be rejected before open")
                    return real_open(path, flags, mode, dir_fd=dir_fd)

                with patch.object(
                        progressive_submit, "_rename_noreplace", create=True,
                        side_effect=no_rename), \
                        patch("progressive_submit.os.rename",
                              side_effect=no_rename), \
                        patch("progressive_submit.os.replace",
                              side_effect=no_rename), \
                        patch("progressive_submit.os.open",
                              side_effect=reject_special_open), \
                        self.assertRaises(ValueError):
                    self.publish_bytes(parent, name, raw)
                self.assertEqual(self.snapshot(parent), before)

    def test_target_claim_is_ambiguous_until_exact_marker_temp_exists(self):
        with tempfile.TemporaryDirectory() as folder:
            campaign = self.campaign(folder)
            payload, contract = self.payload(folder, campaign)

            def stop_after_claim(checkpoint, _context):
                if checkpoint == "after_target_claim":
                    raise RuntimeError("target claim sentinel")

            with self.patch_home(campaign), self.assertRaisesRegex(
                    RuntimeError, "target claim sentinel"):
                create_fixed_stage(
                    campaign, payload, contract, _fault=stop_after_claim)
            target = campaign / "stages" / ACTIVE_STAGE_NAME
            self.assertTrue(target.is_dir())
            self.assertEqual(list(target.iterdir()), [])
            ambiguous = self.snapshot(target)
            with self.patch_home(campaign), self.assertRaisesRegex(
                    ValueError, "ambiguous"):
                create_fixed_stage(campaign, payload, contract)
            self.assertEqual(self.snapshot(target), ambiguous)

            prepared = json.loads(
                (campaign / STAGE_CREATION_PREPARED).read_text())
            marker = {
                "schema": "atlas-stage-creation-transaction-v1",
                "status": "PREPARED",
                "contract_sha256": prepared["contract_sha256"],
                "prepared_sha256": sha256(json_bytes(prepared)),
                "transaction": prepared["transaction"],
            }
            marker_raw = json_bytes(marker)
            marker_temp = target / transaction_temp_name(
                STAGE_CREATION_MARKER, marker_raw)
            write_frozen(marker_temp, marker_raw)
            marker_inode = marker_temp.stat().st_ino
            with self.patch_home(campaign):
                stage, receipt_sha256 = create_fixed_stage(
                    campaign, payload, contract)
                validate_stage_creation(stage, receipt_sha256)
            self.assertFalse(marker_temp.exists())
            self.assertEqual(
                (stage / STAGE_CREATION_MARKER).stat().st_ino,
                marker_inode,
            )
            self.assertFalse(
                (campaign / "stages" / prepared["transaction"]).exists())

    def test_before_v1_rejects_retired_prior_failure_migration_without_mutation(self):
        with tempfile.TemporaryDirectory() as folder:
            campaign = self.campaign(folder)
            payload, contract = self.payload(folder, campaign)
            raw, descriptor, _old_contract = self.prior_failure(
                campaign, contract)
            source = campaign / descriptor["temporary"]
            write_frozen(source, raw)
            before = self.snapshot(campaign)
            with self.patch_home(campaign), self.assertRaisesRegex(
                    ValueError, "migration is closed"):
                create_fixed_stage(
                    campaign, payload, contract, prior_failure=descriptor)
            self.assertEqual(self.snapshot(campaign), before)
            self.assertFalse(
                (campaign / "stages" / ACTIVE_STAGE_NAME).exists())
            self.assertFalse((campaign / STAGE_CREATION_PREPARED).exists())
            self.assertFalse((campaign / STAGE_CREATION_SEALED).exists())
            self.assertFalse((campaign / STAGE_CREATION_PUBLISHED).exists())

    def test_predecessor_archive_tampering_is_rejected_without_new_state(self):
        cases = (
            "mode", "bytes", "extra-link", "missing",
            "descriptor-hash", "descriptor-name-digest",
        )
        for case in cases:
            with self.subTest(case=case), \
                    tempfile.TemporaryDirectory() as folder:
                campaign, predecessor, descriptor = \
                    self.campaign_with_predecessor(folder)
                payload, contract = self.payload(folder, campaign)
                archive_name = next(
                    name for name in descriptor["campaign_files"]
                    if name.startswith(".atlas-stage-creation-failure-")
                )
                archive = campaign / archive_name
                if case == "mode":
                    archive.chmod(0o644)
                elif case == "bytes":
                    archive.chmod(0o644)
                    archive.write_bytes(b'{"changed":true}\n')
                    archive.chmod(0o444)
                elif case == "extra-link":
                    os.link(archive, campaign / "unexpected-archive-link")
                elif case == "missing":
                    archive.unlink()
                elif case == "descriptor-hash":
                    contract["predecessor_state"]["campaign_files"][
                        archive_name]["sha256"] = "0" * 64
                else:
                    binding = contract["predecessor_state"][
                        "campaign_files"].pop(archive_name)
                    contract["predecessor_state"]["campaign_files"][
                        ".atlas-stage-creation-failure-" + "0" * 64 + ".json"
                    ] = binding
                archive_before = self.snapshot(archive)
                predecessor_before = self.snapshot(predecessor)
                extra_link = campaign / "unexpected-archive-link"
                extra_before = self.snapshot(extra_link)
                with self.patch_home(campaign), self.assertRaises(ValueError):
                    create_fixed_stage(campaign, payload, contract)
                self.assertEqual(self.snapshot(archive), archive_before)
                self.assertEqual(
                    self.snapshot(predecessor), predecessor_before)
                self.assertEqual(self.snapshot(extra_link), extra_before)
                self.assertFalse((campaign / STAGE_CREATION_PREPARED).exists())
                self.assertFalse((campaign / STAGE_CREATION_SEALED).exists())
                self.assertFalse((campaign / STAGE_CREATION_PUBLISHED).exists())
                self.assertFalse(
                    (campaign / "stages" / ACTIVE_STAGE_NAME).exists())

    def test_changed_seal_cannot_publish_or_adopt_stage(self):
        with tempfile.TemporaryDirectory() as folder:
            campaign = self.campaign(folder)
            payload, contract = self.payload(folder, campaign)

            def stop_after_sealed(checkpoint, _context):
                if checkpoint == "after_sealed":
                    raise RuntimeError("sealed sentinel")

            with self.patch_home(campaign), self.assertRaisesRegex(
                    RuntimeError, "sealed sentinel"):
                create_fixed_stage(
                    campaign, payload, contract, _fault=stop_after_sealed)
            seal_path = campaign / STAGE_CREATION_SEALED
            seal = json.loads(seal_path.read_text())
            seal["hidden_inode"] += 1
            seal_path.chmod(0o644)
            seal_path.write_bytes(json_bytes(seal))
            seal_path.chmod(0o444)
            target = campaign / "stages" / ACTIVE_STAGE_NAME
            target_before = self.snapshot(target)
            with self.patch_home(campaign), self.assertRaisesRegex(
                    ValueError, "SEALED"):
                create_fixed_stage(campaign, payload, contract)
            self.assertTrue(target.is_dir())
            self.assertEqual(self.snapshot(target), target_before)
            self.assertFalse((campaign / STAGE_CREATION_PUBLISHED).exists())

    def test_raw_racer_wins_final_leaf_without_being_overwritten(self):
        for case in ("empty-dir", "nonempty-dir", "file", "symlink"):
            with self.subTest(case=case), \
                    tempfile.TemporaryDirectory() as folder:
                campaign = self.campaign(folder)
                payload, contract = self.payload(folder, campaign)
                target = campaign / "stages" / ACTIVE_STAGE_NAME
                outside = Path(folder) / "outside"
                outside.mkdir()
                (outside / "sentinel").write_text("unchanged\n")
                outside_before = self.snapshot(outside)
                raced = {}

                def install_racer(checkpoint, _context):
                    if checkpoint != "before_target_claim":
                        return
                    if case in ("empty-dir", "nonempty-dir"):
                        target.mkdir()
                        if case == "nonempty-dir":
                            (target / "racer").write_text("must survive\n")
                    elif case == "file":
                        target.write_text("must survive\n")
                    else:
                        target.symlink_to(outside, target_is_directory=True)
                    raced["snapshot"] = self.snapshot(target)

                with self.patch_home(campaign), self.assertRaises(ValueError):
                    create_fixed_stage(
                        campaign, payload, contract, _fault=install_racer)
                self.assertIn("snapshot", raced)
                self.assertEqual(self.snapshot(target), raced["snapshot"])
                self.assertEqual(self.snapshot(outside), outside_before)
                self.assertEqual(
                    {path.name for path in (campaign / "stages").iterdir()},
                    ({stage for stage, _job in PREDECESSOR_LINEAGE}
                     | {ACTIVE_STAGE_NAME}),
                )
                self.assertTrue((campaign / STAGE_CREATION_PREPARED).is_file())
                self.assertFalse((campaign / STAGE_CREATION_SEALED).exists())
                self.assertFalse((campaign / STAGE_CREATION_PUBLISHED).exists())

    def test_target_first_creation_never_calls_a_rename_primitive(self):
        with tempfile.TemporaryDirectory() as folder:
            campaign = self.campaign(folder)
            payload, contract = self.payload(folder, campaign)
            mkdir_names = []
            real_mkdir = os.mkdir

            def observe_mkdir(name, mode=0o777, *, dir_fd=None):
                mkdir_names.append((str(name), dir_fd))
                return real_mkdir(name, mode, dir_fd=dir_fd)

            forbidden = AssertionError(
                "target-first stage and hardlink files must never be renamed")
            with self.patch_home(campaign), \
                    patch.object(
                        progressive_submit, "_rename_noreplace", create=True,
                        side_effect=forbidden), \
                    patch("progressive_submit.os.rename",
                          side_effect=forbidden), \
                    patch("progressive_submit.os.replace",
                          side_effect=forbidden), \
                    patch("progressive_submit.os.mkdir",
                          side_effect=observe_mkdir):
                stage, receipt_sha256 = create_fixed_stage(
                    campaign, payload, contract)
                validate_stage_creation(stage, receipt_sha256)

            receipt = json.loads(
                (stage / STAGE_CREATION_RECEIPT).read_text())
            self.assertTrue(any(
                name == ACTIVE_STAGE_NAME or Path(name) == stage
                for name, _dir_fd in mkdir_names
            ))
            self.assertFalse(any(
                name == receipt["transaction"]
                or Path(name) == campaign / "stages" / receipt["transaction"]
                for name, _dir_fd in mkdir_names
            ))
            self.assertFalse(
                (campaign / "stages" / receipt["transaction"]).exists())
            self.assertEqual(
                {path.name for path in (campaign / "stages").iterdir()},
                ({stage for stage, _job in PREDECESSOR_LINEAGE}
                 | {ACTIVE_STAGE_NAME}),
            )

    def test_target_first_stage_is_not_valid_or_submittable_before_published(self):
        with tempfile.TemporaryDirectory() as folder:
            campaign = self.campaign(folder)
            payload, contract = self.payload(folder, campaign)

            def stop_before_published(checkpoint, _context):
                if checkpoint == "before_published":
                    raise RuntimeError("before PUBLISHED sentinel")

            with self.patch_home(campaign), self.assertRaisesRegex(
                    RuntimeError, "before PUBLISHED sentinel"):
                create_fixed_stage(
                    campaign, payload, contract,
                    _fault=stop_before_published)
            stage = campaign / "stages" / ACTIVE_STAGE_NAME
            stage_stat = stage.stat()
            receipt_raw = (stage / STAGE_CREATION_RECEIPT).read_bytes()
            receipt = json.loads(receipt_raw)
            sealed = json.loads(
                (campaign / STAGE_CREATION_SEALED).read_text())
            identity = (stage_stat.st_dev, stage_stat.st_ino)
            self.assertEqual(
                (receipt["stage_device"], receipt["stage_inode"]), identity)
            self.assertEqual(
                (sealed["hidden_device"], sealed["hidden_inode"]), identity)
            self.assertEqual(
                ((stage / STAGE_CREATION_MARKER).parent.stat().st_dev,
                 (stage / STAGE_CREATION_MARKER).parent.stat().st_ino),
                identity,
            )
            self.assertFalse((campaign / STAGE_CREATION_PUBLISHED).exists())
            before = self.snapshot(campaign)
            with self.patch_home(campaign), self.assertRaisesRegex(
                    ValueError, "not PUBLISHED"):
                validate_stage_creation(stage, sha256(receipt_raw))
            scheduler = []

            def forbidden_scheduler(command, **_kwargs):
                scheduler.append(command)
                raise AssertionError("unpublished stage contacted scheduler")

            with self.patch_home(campaign), \
                    patch("progressive_submit.subprocess.check_output",
                          side_effect=forbidden_scheduler), \
                    self.assertRaisesRegex(ValueError, "not PUBLISHED"):
                submit_one(
                    stage, SCRIPT_PATH, {"PATH": "/usr/bin"},
                    pin_sha256="0" * 64,
                    stage_creation_sha256=sha256(receipt_raw),
                )
            self.assertEqual(scheduler, [])
            self.assertEqual(self.snapshot(campaign), before)

    def test_publish_boundaries_reject_namespace_swaps(self):
        for route in ("normal", "sealed-recovery"):
            for swapped in (
                    "stage", "creation-lock", "submission-lock",
                    "prepared", "sealed", "ledger",
                    "predecessor-campaign", "predecessor-stage",
                    "predecessor-archive", "predecessor-root",
                    "predecessor-extra"):
                with self.subTest(route=route, swapped=swapped), \
                        tempfile.TemporaryDirectory() as folder:
                    campaign = self.campaign(folder)
                    payload, contract = self.payload(folder, campaign)
                    stage = campaign / "stages" / ACTIVE_STAGE_NAME
                    if route == "sealed-recovery":
                        def stop_after_sealed(checkpoint, _context):
                            if checkpoint == "after_sealed":
                                raise RuntimeError("sealed recovery sentinel")

                        with self.patch_home(campaign), self.assertRaisesRegex(
                                RuntimeError, "sealed recovery sentinel"):
                            create_fixed_stage(
                                campaign, payload, contract,
                                _fault=stop_after_sealed)
                        self.assertTrue(
                            (campaign / STAGE_CREATION_SEALED).is_file())
                        self.assertFalse(
                            (campaign / STAGE_CREATION_PUBLISHED).exists())

                    paths = {
                        "stage": stage,
                        "creation-lock": campaign
                            / progressive_submit.STAGE_CREATION_LOCK,
                        "submission-lock": campaign
                            / ".atlas-progressive-submit.lock",
                        "prepared": campaign / STAGE_CREATION_PREPARED,
                        "sealed": campaign / STAGE_CREATION_SEALED,
                        "ledger": campaign / ".atlas-progressive-submit.json",
                        "predecessor-campaign": campaign
                            / ".atlas-stage-creation-prepared.json",
                        "predecessor-stage": campaign / "stages"
                            / PREDECESSOR_STAGE_NAME
                            / ".atlas-stage-creation.json",
                        "predecessor-root": campaign / "stages"
                            / PREDECESSOR_STAGE_NAME,
                        "predecessor-archive": next(
                            campaign / name
                            for name in contract["predecessor_state"][
                                "campaign_files"]
                            if name.startswith(
                                ".atlas-stage-creation-failure-")),
                        "predecessor-extra": campaign / "stages"
                            / PREDECESSOR_STAGE_NAME
                            / "unexpected-boundary-file",
                    }
                    original = paths[swapped]
                    displaced = original.with_name(original.name + ".displaced")
                    injected = {}

                    def swap_before_published(checkpoint, _context):
                        if checkpoint != "before_published":
                            return
                        if swapped == "predecessor-extra":
                            write_frozen(original, b"late predecessor file\n")
                        else:
                            original.rename(displaced)
                        if swapped == "predecessor-extra":
                            pass
                        elif swapped in ("stage", "predecessor-root"):
                            original.mkdir(mode=0o755)
                            (original / "replacement").write_text(
                                "must remain\n")
                        elif swapped in (
                                "prepared", "sealed", "ledger",
                                "predecessor-campaign", "predecessor-stage",
                                "predecessor-archive"):
                            original.write_text("{}\n")
                            original.chmod(0o444)
                        else:
                            descriptor = os.open(
                                original,
                                os.O_WRONLY | os.O_CREAT | os.O_EXCL,
                                0o600,
                            )
                            os.close(descriptor)
                        injected["displaced"] = self.snapshot(displaced)
                        injected["replacement"] = self.snapshot(original)

                    with self.patch_home(campaign), self.assertRaises(ValueError):
                        create_fixed_stage(
                            campaign, payload, contract,
                            _fault=swap_before_published)
                    self.assertEqual(set(injected), {
                        "displaced", "replacement",
                    })
                    self.assertEqual(
                        self.snapshot(displaced), injected["displaced"])
                    self.assertEqual(
                        self.snapshot(original), injected["replacement"])
                    self.assertFalse(
                        (campaign / STAGE_CREATION_PUBLISHED).exists())
                    self.assertEqual(
                        [name for name in os.listdir(campaign)
                         if name.startswith(".atlas-publish-")],
                        [],
                    )
                    if swapped == "ledger":
                        original.unlink()
                        displaced.rename(original)
                        with self.patch_home(campaign):
                            recovered_stage, recovered_sha = create_fixed_stage(
                                campaign, payload, contract)
                            validate_stage_creation(
                                recovered_stage, recovered_sha)

        for swapped in (
                "stage", "creation-lock", "submission-lock",
                "prepared", "sealed", "published", "ledger",
                "predecessor-campaign", "predecessor-stage",
                "predecessor-archive", "predecessor-root",
                "predecessor-extra"):
            with self.subTest(route="after-published", swapped=swapped), \
                    tempfile.TemporaryDirectory() as folder:
                campaign = self.campaign(folder)
                payload, contract = self.payload(folder, campaign)
                stage = campaign / "stages" / ACTIVE_STAGE_NAME
                paths = {
                    "stage": stage,
                    "creation-lock": campaign
                        / progressive_submit.STAGE_CREATION_LOCK,
                    "submission-lock": campaign
                        / ".atlas-progressive-submit.lock",
                    "prepared": campaign / STAGE_CREATION_PREPARED,
                    "sealed": campaign / STAGE_CREATION_SEALED,
                    "published": campaign / STAGE_CREATION_PUBLISHED,
                    "ledger": campaign / ".atlas-progressive-submit.json",
                    "predecessor-campaign": campaign
                        / ".atlas-stage-creation-prepared.json",
                    "predecessor-stage": campaign / "stages"
                        / PREDECESSOR_STAGE_NAME
                        / ".atlas-stage-creation.json",
                    "predecessor-root": campaign / "stages"
                        / PREDECESSOR_STAGE_NAME,
                    "predecessor-archive": next(
                        campaign / name
                        for name in contract["predecessor_state"][
                            "campaign_files"]
                        if name.startswith(
                            ".atlas-stage-creation-failure-")),
                    "predecessor-extra": campaign / "stages"
                        / PREDECESSOR_STAGE_NAME
                        / "unexpected-boundary-file",
                }
                original = paths[swapped]
                injected = {}

                def swap_after_published(checkpoint, _context):
                    if checkpoint != "after_published":
                        return
                    displaced = original.with_name(
                        original.name + ".displaced")
                    if swapped == "predecessor-extra":
                        write_frozen(original, b"late predecessor file\n")
                    else:
                        original.rename(displaced)
                    if swapped == "predecessor-extra":
                        pass
                    elif swapped in ("stage", "predecessor-root"):
                        original.mkdir(mode=0o755)
                        (original / "replacement").write_text(
                            "must remain\n")
                    elif swapped in (
                            "prepared", "sealed", "published", "ledger",
                            "predecessor-campaign", "predecessor-stage",
                            "predecessor-archive"):
                        original.write_text("{}\n")
                        original.chmod(0o444)
                    else:
                        descriptor = os.open(
                            original,
                            os.O_WRONLY | os.O_CREAT | os.O_EXCL,
                            0o600,
                        )
                        os.close(descriptor)
                    injected["displaced"] = self.snapshot(displaced)
                    injected["replacement"] = self.snapshot(original)

                with self.patch_home(campaign), self.assertRaises(ValueError):
                    create_fixed_stage(
                        campaign, payload, contract,
                        _fault=swap_after_published)
                self.assertEqual(set(injected), {
                    "displaced", "replacement",
                })
                self.assertEqual(
                    self.snapshot(paths[swapped].with_name(
                        paths[swapped].name + ".displaced")),
                    injected["displaced"],
                )
                self.assertEqual(
                    self.snapshot(paths[swapped]), injected["replacement"])
                self.assertEqual(
                    [name for name in os.listdir(campaign)
                     if name.startswith(".atlas-publish-")],
                    [],
                )
                if swapped == "ledger":
                    displaced = original.with_name(
                        original.name + ".displaced")
                    original.unlink()
                    displaced.rename(original)
                    with self.patch_home(campaign):
                        recovered_stage, recovered_sha = create_fixed_stage(
                            campaign, payload, contract)
                        validate_stage_creation(
                            recovered_stage, recovered_sha)

    def test_published_raw_bytes_and_numeric_types_are_exact(self):
        recovery_cases = (
            ("prepared", "after_prepared", STAGE_CREATION_PREPARED,
             "sequence"),
            ("receipt", "after_receipt", STAGE_CREATION_RECEIPT,
             "stage_device"),
            ("sealed", "after_sealed", STAGE_CREATION_SEALED,
             "sequence"),
        )
        for name, checkpoint, leaf, field in recovery_cases:
            with self.subTest(state=name), \
                    tempfile.TemporaryDirectory() as folder:
                campaign = self.campaign(folder)
                payload, contract = self.payload(folder, campaign)

                def stop_here(observed, _context):
                    if observed == checkpoint:
                        raise RuntimeError(checkpoint)

                with self.patch_home(campaign), self.assertRaisesRegex(
                        RuntimeError, checkpoint):
                    create_fixed_stage(
                        campaign, payload, contract, _fault=stop_here)
                path = campaign / leaf if name != "receipt" else (
                    campaign / "stages" / ACTIVE_STAGE_NAME / leaf)
                value = json.loads(path.read_text())
                value[field] = float(value[field])
                path.chmod(0o644)
                path.write_bytes(json_bytes(value))
                path.chmod(0o444)
                before = self.snapshot(campaign)
                with self.patch_home(campaign), self.assertRaises(ValueError):
                    create_fixed_stage(campaign, payload, contract)
                self.assertEqual(self.snapshot(campaign), before)

        with tempfile.TemporaryDirectory() as folder:
            campaign = self.campaign(folder)
            payload, contract = self.payload(folder, campaign)
            with self.patch_home(campaign):
                stage, receipt_sha256 = create_fixed_stage(
                    campaign, payload, contract)
            marker_path = stage / STAGE_CREATION_MARKER
            marker = json.loads(marker_path.read_text())
            noncanonical = (
                json.dumps(marker, separators=(",", ":")) + "\n").encode()
            self.assertNotEqual(noncanonical, marker_path.read_bytes())
            marker_path.chmod(0o644)
            marker_path.write_bytes(noncanonical)
            marker_path.chmod(0o444)
            before = self.snapshot(campaign)
            with self.patch_home(campaign), self.assertRaises(ValueError):
                validate_stage_creation(stage, receipt_sha256)
            self.assertEqual(self.snapshot(campaign), before)

        for case in ("noncanonical", "numeric-type"):
            with self.subTest(case=case), \
                    tempfile.TemporaryDirectory() as folder:
                campaign = self.campaign(folder)
                payload, contract = self.payload(folder, campaign)
                with self.patch_home(campaign):
                    stage, receipt_sha256 = create_fixed_stage(
                        campaign, payload, contract)
                published_path = campaign / STAGE_CREATION_PUBLISHED
                canonical = published_path.read_bytes()
                value = json.loads(canonical)
                if case == "noncanonical":
                    changed = (
                        json.dumps(value, separators=(",", ":")) + "\n").encode()
                    self.assertEqual(json.loads(changed), value)
                else:
                    value["sequence"] = 3.0
                    changed = json_bytes(value)
                    self.assertEqual(json.loads(changed)["sequence"], 3.0)
                    self.assertEqual(json.loads(changed)["sequence"], 3)
                self.assertNotEqual(changed, canonical)
                published_path.chmod(0o644)
                published_path.write_bytes(changed)
                published_path.chmod(0o444)
                before = self.snapshot(campaign)
                with self.patch_home(campaign), self.assertRaises(ValueError):
                    validate_stage_creation(stage, receipt_sha256)
                self.assertEqual(self.snapshot(campaign), before)

    def test_published_hardlink_pair_is_read_only_rejected_before_scheduler(self):
        with tempfile.TemporaryDirectory() as folder:
            campaign = self.campaign(folder)
            payload, contract = self.payload(folder, campaign)

            def stop_at_published_link(checkpoint, context):
                if (checkpoint == "after_file_link"
                        and context.get("name") == STAGE_CREATION_PUBLISHED):
                    raise RuntimeError("PUBLISHED link sentinel")

            with self.patch_home(campaign), self.assertRaisesRegex(
                    RuntimeError, "PUBLISHED link sentinel"):
                create_fixed_stage(
                    campaign, payload, contract,
                    _fault=stop_at_published_link)
            stage = campaign / "stages" / ACTIVE_STAGE_NAME
            receipt_raw = (stage / STAGE_CREATION_RECEIPT).read_bytes()
            receipt_sha256 = sha256(receipt_raw)
            published = campaign / STAGE_CREATION_PUBLISHED
            published_raw = published.read_bytes()
            temporary = campaign / transaction_temp_name(
                STAGE_CREATION_PUBLISHED, published_raw)
            self.assertTrue(temporary.is_file())
            self.assertEqual(temporary.read_bytes(), published_raw)
            self.assertEqual(
                (temporary.stat().st_dev, temporary.stat().st_ino),
                (published.stat().st_dev, published.stat().st_ino),
            )
            self.assertEqual(temporary.stat().st_nlink, 2)
            self.assertEqual(published.stat().st_nlink, 2)
            pair_before = {
                "published": self.snapshot(published),
                "temporary": self.snapshot(temporary),
            }

            with self.patch_home(campaign), self.assertRaises(ValueError):
                validate_stage_creation(stage, receipt_sha256)
            self.assertEqual(self.snapshot(published), pair_before["published"])
            self.assertEqual(self.snapshot(temporary), pair_before["temporary"])

            with self.patch_home(campaign), \
                    patch("progressive_submit.subprocess.check_output") \
                        as scheduler, \
                    self.assertRaises(ValueError):
                submit_one(
                    stage, SCRIPT_PATH, {"PATH": "/usr/bin"},
                    pin_sha256="0" * 64,
                    stage_creation_sha256=receipt_sha256,
                )
            scheduler.assert_not_called()
            self.assertEqual(self.snapshot(published), pair_before["published"])
            self.assertEqual(self.snapshot(temporary), pair_before["temporary"])

    def run_competitors(self, campaign, arms):
        context = multiprocessing.get_context("spawn")
        start = context.Event()
        results = context.Queue()
        processes = [
            context.Process(
                target=create_worker,
                args=(str(campaign), str(payload), contract, start, results),
            )
            for payload, contract in arms
        ]
        for process in processes:
            process.start()
        start.set()
        for process in processes:
            process.join(20)
        try:
            for process in processes:
                if process.is_alive():
                    process.terminate()
                    process.join(5)
                    self.fail("concurrent stage creator did not terminate")
                self.assertEqual(process.exitcode, 0)
            return [results.get(timeout=5) for _ in processes]
        finally:
            results.close()
            results.join_thread()

    def test_real_processes_same_spec_publish_once(self):
        with tempfile.TemporaryDirectory() as folder:
            campaign = self.campaign(folder)
            payload, contract = self.payload(folder, campaign)
            outcomes = self.run_competitors(
                campaign, [(payload, contract), (payload, contract)])
            self.assertEqual([row[0] for row in outcomes], ["ok", "ok"])
            self.assertEqual(len({tuple(row[1:]) for row in outcomes}), 1)
            self.assertEqual(
                {path.name for path in (campaign / "stages").iterdir()},
                ({stage for stage, _job in PREDECESSOR_LINEAGE}
                 | {ACTIVE_STAGE_NAME}),
            )
            self.assertTrue((campaign / STAGE_CREATION_PREPARED).is_file())
            self.assertTrue((campaign / STAGE_CREATION_PUBLISHED).is_file())

    def test_real_processes_different_specs_cannot_mix_payloads(self):
        with tempfile.TemporaryDirectory() as folder:
            campaign = self.campaign(folder)
            first_payload, first_contract = self.payload(folder, campaign)
            second_payload, second_contract = self.payload(
                folder, campaign, suffix=b"-other")
            outcomes = self.run_competitors(
                campaign,
                [(first_payload, first_contract),
                 (second_payload, second_contract)],
            )
            self.assertEqual(sorted(row[0] for row in outcomes), ["error", "ok"])
            stage, receipt_raw, receipt = self.published(campaign)
            winner = first_contract if receipt["contract"] == first_contract \
                else second_contract
            self.assertEqual(receipt["contract"], winner)
            self.assertEqual(sha256(receipt_raw),
                             next(row[2] for row in outcomes if row[0] == "ok"))
            self.assertEqual(
                {
                    str(path.relative_to(stage / "overrides")): sha256(path.read_bytes())
                    for path in (stage / "overrides").rglob("*")
                    if path.is_file() and path.name != "overrides.json"
                },
                winner["inputs"],
            )

    def install_submission_inputs(self, stage, receipt_sha256, receipt):
        script_source = stage / "overrides" / SCRIPT_PATH
        script_target = stage / SCRIPT_PATH
        script_target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(script_source, script_target)
        script_target.chmod(0o444)
        pin = {
            "schema": "atlas-weyl-context-g2-pin-v1",
            "stage_creation": {
                "receipt_sha256": receipt_sha256,
                "contract_sha256": receipt["contract_sha256"],
            },
        }
        pin_raw = json_bytes(pin)
        (stage / PIN_PATH).write_bytes(pin_raw)
        (stage / PIN_PATH).chmod(0o444)
        return sha256(pin_raw)

    def prepare_submission(self, folder):
        campaign = self.campaign(folder)
        payload, contract = self.payload(folder, campaign)
        with self.patch_home(campaign):
            stage, receipt_sha256 = create_fixed_stage(
                campaign, payload, contract)
            receipt = validate_stage_creation(stage, receipt_sha256)
            pin_sha256 = self.install_submission_inputs(
                stage, receipt_sha256, receipt)
        return campaign, stage, receipt_sha256, pin_sha256

    def test_lock_name_swap_after_squeue_never_reaches_sbatch(self):
        with tempfile.TemporaryDirectory() as folder:
            campaign, stage, creation_sha, pin_sha = self.prepare_submission(
                folder)
            predecessor = json.loads(
                (campaign / ".atlas-progressive-submit.json").read_text())
            lock = campaign / ".atlas-progressive-submit.lock"
            displaced = campaign / ".atlas-progressive-submit.lock.displaced"

            def scheduler(_command, **_options):
                lock.rename(displaced)
                descriptor = os.open(lock, os.O_WRONLY | os.O_CREAT | os.O_EXCL,
                                     0o600)
                os.close(descriptor)
                return ""

            with self.patch_home(campaign), \
                    patch("progressive_submit.subprocess.check_output",
                          side_effect=scheduler) as scheduler_call, \
                    self.assertRaisesRegex(ValueError, "lock path changed"):
                submit_one(
                    stage, SCRIPT_PATH, {}, pin_sha256=pin_sha,
                    stage_creation_sha256=creation_sha)
            self.assertEqual(scheduler_call.call_count, 1)
            self.assertFalse((stage / "submission-intent.json").exists())
            self.assertEqual(
                json.loads((campaign / ".atlas-progressive-submit.json").read_text()),
                predecessor,
            )

    def test_intent_racer_is_preserved_and_never_reaches_sbatch(self):
        with tempfile.TemporaryDirectory() as folder:
            campaign, stage, creation_sha, pin_sha = self.prepare_submission(
                folder)
            predecessor = json.loads(
                (campaign / ".atlas-progressive-submit.json").read_text())
            intent = stage / "submission-intent.json"
            racer_raw = b'{"owner":"racer"}\n'

            def scheduler(_command, **_options):
                intent.write_bytes(racer_raw)
                intent.chmod(0o444)
                return ""

            with self.patch_home(campaign), \
                    patch("progressive_submit.subprocess.check_output",
                          side_effect=scheduler) as scheduler_call, \
                    self.assertRaisesRegex(ValueError, "existing submission intent"):
                submit_one(
                    stage, SCRIPT_PATH, {}, pin_sha256=pin_sha,
                    stage_creation_sha256=creation_sha)
            self.assertEqual(scheduler_call.call_count, 1)
            self.assertEqual(intent.read_bytes(), racer_raw)
            self.assertEqual(
                json.loads((campaign / ".atlas-progressive-submit.json").read_text()),
                predecessor,
            )

    def test_mode_tampering_fails_before_squeue(self):
        cases = ("ledger", "stage", "overrides", "script", "pin")
        for case in cases:
            with self.subTest(case=case), \
                    tempfile.TemporaryDirectory() as folder:
                campaign, stage, creation_sha, pin_sha = self.prepare_submission(
                    folder)
                targets = {
                    "ledger": campaign / ".atlas-progressive-submit.json",
                    "stage": stage,
                    "overrides": stage / "overrides",
                    "script": stage / SCRIPT_PATH,
                    "pin": stage / PIN_PATH,
                }
                target = targets[case]
                target.chmod(0o777 if target.is_dir() else 0o666)
                with self.patch_home(campaign), \
                        patch("progressive_submit.subprocess.check_output") as scheduler, \
                        self.assertRaises(ValueError):
                    submit_one(
                        stage, SCRIPT_PATH, {}, pin_sha256=pin_sha,
                        stage_creation_sha256=creation_sha)
                scheduler.assert_not_called()

    def test_submit_one_requires_published_receipt_before_squeue(self):
        with tempfile.TemporaryDirectory() as folder:
            campaign = self.campaign(folder)
            raw = campaign / "stages" / ACTIVE_STAGE_NAME
            raw.mkdir()
            (raw / "one.sbatch").write_text("#SBATCH --nodes=1\n")
            with self.patch_home(campaign), \
                    patch("progressive_submit.subprocess.check_output") as scheduler:
                for creation_sha256 in (None, "0" * 64):
                    with self.subTest(creation_sha256=creation_sha256), \
                            self.assertRaises(ValueError):
                        submit_one(
                            raw, "one.sbatch", {},
                            pin_sha256="a" * 64,
                            stage_creation_sha256=creation_sha256,
                        )
                scheduler.assert_not_called()

        with tempfile.TemporaryDirectory() as source_folder, \
                tempfile.TemporaryDirectory() as target_folder:
            source_campaign = self.campaign(source_folder)
            payload, contract = self.payload(source_folder, source_campaign)
            with self.patch_home(source_campaign):
                source_stage, source_sha = create_fixed_stage(
                    source_campaign, payload, contract)
            target_campaign = self.campaign(target_folder)
            target = target_campaign / "stages" / ACTIVE_STAGE_NAME
            target.mkdir()
            shutil.copyfile(
                source_stage / STAGE_CREATION_RECEIPT,
                target / STAGE_CREATION_RECEIPT,
            )
            (target / "one.sbatch").write_text("#SBATCH --nodes=1\n")
            with self.patch_home(target_campaign), \
                    patch("progressive_submit.subprocess.check_output") as scheduler, \
                    self.assertRaises(ValueError):
                submit_one(
                    target, "one.sbatch", {},
                    pin_sha256="a" * 64,
                    stage_creation_sha256=source_sha,
                )
            scheduler.assert_not_called()

    def test_real_published_stage_reaches_sbatch_after_durable_intent(self):
        with tempfile.TemporaryDirectory() as folder:
            campaign, stage, creation_sha, pin_sha = self.prepare_submission(
                folder)
            ledger_path = campaign / ".atlas-progressive-submit.json"
            predecessor = json.loads(ledger_path.read_text())
            intent_path = stage / "submission-intent.json"
            script_contents = (stage / SCRIPT_PATH).read_text(encoding="utf-8")
            expected_unconfirmed = {
                "stage": str(stage.resolve()),
                "script": SCRIPT_PATH,
                "queue_before": [],
                "status": "SUBMISSION_INTENT_NOT_CONFIRMED",
                "max_outstanding": 10,
                "pin_sha256": pin_sha,
                "stage_creation_sha256": creation_sha,
            }
            commands = []
            stage_value = stage.stat()
            stage_identity = (stage_value.st_dev, stage_value.st_ino)
            displaced = stage.with_name(stage.name + ".displaced")
            descriptor_cwds = []

            def scheduler(command, **options):
                commands.append(command)
                if command[0] == "squeue":
                    self.assertEqual(command, [
                        "squeue", "-h", "-r", "-u", "guard-test", "-o", "%i",
                    ])
                    self.assertEqual(options, {"text": True, "timeout": 20})
                    self.assertEqual(
                        json.loads(ledger_path.read_text()), predecessor)
                    self.assertFalse(intent_path.exists())
                    return ""
                self.assertEqual(
                    command, ["sbatch", "--parsable", "--export=ALL"])
                descriptors = options.get("pass_fds")
                self.assertIsInstance(descriptors, tuple)
                self.assertEqual(len(descriptors), 1)
                descriptor = descriptors[0]
                self.assertNotIn("preexec_fn", options)
                self.assertEqual(options["env"], {})
                self.assertEqual(options["input"], script_contents)
                self.assertIs(options["text"], True)
                self.assertEqual(options["timeout"], 25)
                ledger = json.loads(ledger_path.read_text())
                intent = json.loads(intent_path.read_text())
                self.assertEqual(ledger, predecessor + [expected_unconfirmed])
                self.assertEqual(intent, expected_unconfirmed)
                self.assertNotIn("job", intent)
                stage.rename(displaced)
                stage.mkdir(mode=0o755)
                try:
                    held = os.fstat(descriptor)
                    bound = os.stat(options["cwd"])
                    replacement = stage.stat()
                    self.assertEqual(
                        (held.st_dev, held.st_ino), stage_identity)
                    self.assertEqual(
                        (bound.st_dev, bound.st_ino), stage_identity)
                    self.assertNotEqual(
                        (replacement.st_dev, replacement.st_ino),
                        stage_identity,
                    )
                    descriptor_cwds.append(options["cwd"])
                finally:
                    stage.rmdir()
                    displaced.rename(stage)
                return "812345;cluster"

            with self.patch_home(campaign), \
                    patch("progressive_submit.pwd.getpwuid",
                          side_effect=KeyError), \
                    patch("progressive_submit.subprocess.check_output") \
                          as unresolved_scheduler, \
                    self.assertRaisesRegex(ValueError, "scheduler user"):
                submit_one(
                    stage, SCRIPT_PATH, {}, pin_sha256=pin_sha,
                    stage_creation_sha256=creation_sha,
                )
            unresolved_scheduler.assert_not_called()
            self.assertFalse(intent_path.exists())
            self.assertEqual(json.loads(ledger_path.read_text()), predecessor)

            account = type("Account", (), {"pw_name": "guard-test"})()
            with self.patch_home(campaign), \
                    patch("progressive_submit.os.geteuid",
                          return_value=4242), \
                    patch("progressive_submit.pwd.getpwuid",
                          return_value=account) as lookup, \
                    patch("progressive_submit.subprocess.check_output",
                          side_effect=scheduler) as scheduler_call:
                record = submit_one(
                    stage, SCRIPT_PATH, {}, pin_sha256=pin_sha,
                    stage_creation_sha256=creation_sha,
                )
            lookup.assert_called_once_with(4242)

            confirmed = dict(
                expected_unconfirmed, status="SUBMITTED", job="812345")
            self.assertEqual(scheduler_call.call_count, 2)
            self.assertEqual(commands, [
                ["squeue", "-h", "-r", "-u", "guard-test", "-o", "%i"],
                ["sbatch", "--parsable", "--export=ALL"],
            ])
            self.assertEqual(len(descriptor_cwds), 1)
            self.assertRegex(descriptor_cwds[0], r"^/proc/self/fd/[0-9]+$")
            self.assertEqual(record, confirmed)
            self.assertEqual(
                json.loads(ledger_path.read_text()), predecessor + [confirmed])
            self.assertEqual(json.loads(intent_path.read_text()), confirmed)
            with self.patch_home(campaign):
                receipt = validate_stage_creation(
                    stage, creation_sha, pin_sha256=pin_sha)
            self.assertEqual(receipt["contract"]["script"]["path"], SCRIPT_PATH)

    def test_post_ledger_receipt_tampering_blocks_sbatch_and_retry(self):
        for tamper in ("mode", "content"):
            with self.subTest(tamper=tamper), \
                    tempfile.TemporaryDirectory() as folder:
                campaign, stage, creation_sha, pin_sha = self.prepare_submission(
                    folder)
                ledger_path = campaign / ".atlas-progressive-submit.json"
                predecessor = json.loads(ledger_path.read_text())
                intent_path = stage / "submission-intent.json"
                receipt_path = stage / STAGE_CREATION_RECEIPT
                original_receipt = receipt_path.read_bytes()
                expected_unconfirmed = {
                    "stage": str(stage.resolve()),
                    "script": SCRIPT_PATH,
                    "queue_before": [],
                    "status": "SUBMISSION_INTENT_NOT_CONFIRMED",
                    "max_outstanding": 10,
                    "pin_sha256": pin_sha,
                    "stage_creation_sha256": creation_sha,
                }
                real_save = progressive_submit.save
                tampered = []

                def tamper_after_unconfirmed_ledger(path, value):
                    real_save(path, value)
                    if (Path(path) == ledger_path
                            and isinstance(value, list) and value
                            and value[-1].get("status")
                               == "SUBMISSION_INTENT_NOT_CONFIRMED"):
                        if tamper == "mode":
                            receipt_path.chmod(0o644)
                        else:
                            receipt_path.chmod(0o644)
                            receipt_path.write_bytes(b'{"changed":true}\n')
                            receipt_path.chmod(0o444)
                        tampered.append(tamper)

                def scheduler(command, **_options):
                    if command[0] != "squeue":
                        self.fail("sbatch was reached after receipt tampering")
                    return ""

                with self.patch_home(campaign), \
                        patch.dict(os.environ, {"USER": "guard-test"}), \
                        patch.object(
                            progressive_submit, "save",
                            side_effect=tamper_after_unconfirmed_ledger), \
                        patch("progressive_submit.subprocess.check_output",
                              side_effect=scheduler) as scheduler_call, \
                        self.assertRaises(ValueError):
                    submit_one(
                        stage, SCRIPT_PATH, {}, pin_sha256=pin_sha,
                        stage_creation_sha256=creation_sha,
                    )

                self.assertEqual(tampered, [tamper])
                self.assertEqual(scheduler_call.call_count, 1)
                ledger = json.loads(ledger_path.read_text())
                intent = json.loads(intent_path.read_text())
                self.assertEqual(ledger, predecessor + [expected_unconfirmed])
                self.assertEqual(intent, expected_unconfirmed)
                self.assertNotIn("job", ledger[-1])

                receipt_path.chmod(0o644)
                receipt_path.write_bytes(original_receipt)
                receipt_path.chmod(0o444)
                with self.patch_home(campaign), \
                        patch.dict(os.environ, {"USER": "guard-test"}), \
                        patch("progressive_submit.subprocess.check_output") \
                            as retry_scheduler, \
                        self.assertRaises(ValueError):
                    submit_one(
                        stage, SCRIPT_PATH, {}, pin_sha256=pin_sha,
                        stage_creation_sha256=creation_sha,
                    )
                retry_scheduler.assert_not_called()

    def test_uncertain_submission_retains_creation_sha_everywhere(self):
        with tempfile.TemporaryDirectory() as folder:
            campaign = self.campaign(folder)
            payload, contract = self.payload(folder, campaign)
            with self.patch_home(campaign):
                stage, receipt_sha256 = create_fixed_stage(
                    campaign, payload, contract)
                receipt = validate_stage_creation(stage, receipt_sha256)
                pin_sha256 = self.install_submission_inputs(
                    stage, receipt_sha256, receipt)
                with patch(
                        "progressive_submit.subprocess.check_output",
                        side_effect=[
                            "",
                            subprocess.TimeoutExpired("sbatch", 25),
                        ]) as scheduler, self.assertRaises(
                            subprocess.TimeoutExpired):
                    submit_one(
                        stage, SCRIPT_PATH, {}, pin_sha256=pin_sha256,
                        stage_creation_sha256=receipt_sha256,
                    )
                self.assertEqual(scheduler.call_count, 2)
            ledger = json.loads(
                (campaign / ".atlas-progressive-submit.json").read_text())
            intent = json.loads(
                (stage / "submission-intent.json").read_text())
            self.assertEqual(ledger[-1], intent)
            self.assertEqual(
                ledger[-1]["stage_creation_sha256"], receipt_sha256)
            self.assertEqual(
                ledger[-1]["status"], "SUBMISSION_INTENT_NOT_CONFIRMED")
            with self.patch_home(campaign), \
                    patch("progressive_submit.subprocess.check_output") as scheduler, \
                    self.assertRaisesRegex(ValueError, "unresolved submission"):
                submit_one(
                    stage, SCRIPT_PATH, {}, pin_sha256=pin_sha256,
                    stage_creation_sha256=receipt_sha256,
                )
            scheduler.assert_not_called()


if __name__ == "__main__":
    unittest.main()

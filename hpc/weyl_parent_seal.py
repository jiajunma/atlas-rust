"""Path-independent contract for the complete Weyl parent evidence seal."""
import hashlib
import json
from pathlib import PurePosixPath
import re

from campaign_blob import SCHEMA as BLOB_SCHEMA, read_blob, verify_blob
from campaign_source import SCHEMA as SOURCE_SCHEMA, verify_source_archive
from campaign_workspace import ACTIVE_CAMPAIGN, HPC_HOME


SCHEMA = "atlas-weyl-context-parent-seal-v1"
STAGE_NAME = "weyl-parent-seal-v1"
PIN_NAME = "weyl-parent-seal-pin.json"
SBATCH = "hpc/math_weyl_parent_seal.sbatch"
STAGE_LOCK = ".parent-seal-stage.lock"
CHECKER_TESTS = 64
SUBMISSION_ATTESTATION_SCHEMA = "atlas-weyl-parent-seal-submission-attestation-v1"
SUBMISSION_STAGE = "stages/" + STAGE_NAME
SUBMISSION_ABSOLUTE_STAGE = (HPC_HOME / ACTIVE_CAMPAIGN / SUBMISSION_STAGE).as_posix()
SUBMISSION_RECORD_KEYS = {
    "stage", "script", "queue_before", "status", "max_outstanding", "job",
    "pin_sha256",
}
TRACE_MARKER = "WEYL_PARENT_TRACE_JSON="
MIGRATION_OVERRIDE_NAMES = {
    "hpc/" + name for name in (
        "campaign_workspace.py", "test_campaign_workspace.py",
        "campaign_source.py", "test_campaign_source.py",
        "campaign_blob.py", "test_campaign_blob.py",
        "progressive_submit.py", "test_progressive_submit.py",
        "weyl_parent_seal.py", "test_weyl_parent_seal.py",
        "weyl_parent_legacy_trace.py", "math_weyl_parent_seal.py",
        "math_weyl_parent_seal.sbatch", "stage_weyl_parent_seal.py",
    )
}
PARENT_SEAL_KEYS = {
    "schema", "status", "migration", "migration_inputs", "legacy_gate_runs",
    "trace_sets_equal", "archival_closure_verified", "legacy", "capture",
    "report_hashes", "core_inventory", "core_inventory_sha256",
    "domain_inventory", "domain_inventory_sha256", "source_file_count",
    "source_manifest", "source_manifest_sha256", "source_object", "binaries",
    "harness", "scripts", "execution_artifacts", "legacy_files", "closure",
}
CAPTURE_COMMANDS = ["test_math_loading_followup", "test_math_overload_command_verified",
                    "test_math_weyl_context_capture"]
CAPTURE_CASES = [
    dict(file="weyl_context_history.atlas",
         fixture_sha256="abbe9a88e474407a7aedbea33a661ded98404d6cd6889692d87bec27ec11f864",
         id="generic_weyl_context_history", intent="accept",
         source_sha256="225381360965b99a470f3dd3c69b409ec60fb9dc159aa727121599652676f3ea"),
    dict(file="weyl_context_rejected.atlas",
         fixture_sha256="9a4a2b567933cfcb58e2ccbd328aeb2630a0e62ee86791de8c5a1529881ed4cf",
         id="generic_weyl_context_rejected", intent="reject",
         source_sha256="8c9c34bb83f322281373a40decaff0da601f6e9b3558837084ec0674207c6374"),
    dict(file="root_ladder_coordinate_boundary.atlas",
         fixture_sha256="dc88d6606ae855b618dcf12589ecde82edcbe482873a94bcff1001163691ec65",
         id="generic_root_ladder_coordinate_boundary", intent="accept",
         source_sha256="dde2e5c1f84d255ef66e01c83deef114f70a22dc22224e87cb186359077df95f"),
]
LADDER_LABELS = ["small-root-true", "below-root-true", "above-root-true",
                 "below-root-false", "above-root-false", "below-coroot-true",
                 "above-coroot-true", "below-coroot-false", "above-coroot-false",
                 "max-root", "max-coroot"]
CAPTURE_COMPARISONS = [
    dict(status="ORIGINAL_PROVISIONAL_INTENT_NOT_CONFIRMED",
         categories=dict(oracle="REJECTED_PROGRAM", rust="REJECTED_NAME"),
         complete_frames=dict(oracle=False, rust=False),
         details=dict(oracle=dict(labels=["START", "RECOVERY"]),
                      rust=dict(labels=["START", "RECOVERY"])),
         full_stderr_equal=False, full_stdout_equal=True,
         original_matches_provisional_intent=False, positive_full_stream_match=False),
    dict(status="ORIGINAL_HISTORY_INCOMPLETE",
         categories=dict(oracle="REJECTED_PROGRAM+RUNTIME", rust="REJECTED_NAME+RUNTIME"),
         complete_frames=dict(oracle=False, rust=False),
         details=dict(oracle=dict(labels=["AFTER_ROOT", "RECOVERY"]),
                      rust=dict(labels=["AFTER_ROOT", "RECOVERY"])),
         full_stderr_equal=False, full_stdout_equal=True,
         original_matches_provisional_intent=True, positive_full_stream_match=False),
    dict(status="RUST_POSITIVE_DIFFERENCE",
         categories=dict(oracle="ACCEPTED", rust="REJECTED_RUNTIME"),
         complete_frames=dict(oracle=True, rust=False),
         details=dict(oracle=dict(datums=11, labels=LADDER_LABELS, recovery=1, rows=22),
                      rust=dict(datums=11, labels=LADDER_LABELS, recovery=1, rows=10)),
         full_stderr_equal=False, full_stdout_equal=False,
         original_matches_provisional_intent=True, positive_full_stream_match=False),
]
EXECUTION_ARTIFACTS = {
    "capture-report", "capture-pin", "command-after-report", "profile-report",
    "boundary-input", "boundary-oracle-stdout", "boundary-oracle-stderr",
    "boundary-rust-stdout", "boundary-rust-stderr",
    "legacy-gate-first-log", "legacy-gate-first-time",
    "legacy-gate-second-log", "legacy-gate-second-time",
}
RUST_BINARY_SHA256 = "7aa4350d3f4dac4b250608b1fa67fe111fb5442029e32db69906b5dbe8893cf6"
ORACLE_BINARY_SHA256 = "d4f0f3dc3a82102529aa2ec562db0601e99b368dee25d5539ca52dae2fd37a5a"
REPORT_HASHES = {
    "capture": "bdee12811fe6ce4160ffbc2a25df18d9a9338a7b29f22d5fcd19352cf970f832",
    "capture_pin": "459a5147e8546bf7e61d865cdd71e74a414db7c74d3612ba1a5893723c3cdc34",
    "command_after": "fbc615d23c5d111aeea83ece1fecb6c67f7b37d6c0435c6af21e062a86d787fc",
    "profile": "1d2baaddbfcb2bf09801a995ed9a6702646dd3cf9a8f396b1ddfd6d22c693915",
}
FIXED_ARTIFACT_HASHES = {
    "capture-report": REPORT_HASHES["capture"],
    "capture-pin": REPORT_HASHES["capture_pin"],
    "command-after-report": REPORT_HASHES["command_after"],
    "profile-report": REPORT_HASHES["profile"],
    "boundary-input": CAPTURE_CASES[2]["source_sha256"],
    "boundary-oracle-stdout": "3a7fdade43c46cf4f3048b52cf81f282db060012951ee559296f017f7d3eab80",
    "boundary-oracle-stderr": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
    "boundary-rust-stdout": "86d2a6e7b6d346c42ed42484a2141836982fc55d30d63925adfd7e88a40e0664",
    "boundary-rust-stderr": "819d2264c925c1849bc86aaf5a88b84fee99c47dea9ed03b2499a9619185c7d3",
}
SOURCE_MANIFEST_SHA256 = "b027eee2efe72d0f8a20848f99cd6368c29e3884ce9be16a2e7b5c63577dd0da"
CORE_INVENTORY_SHA256 = "d14812d037526cea52bce0b2358ffbafe1edfad6a7fe2e3dd067dce913973e84"


def _json_digest(value):
    raw = json.dumps(value, sort_keys=True, separators=(",", ":")).encode()
    return hashlib.sha256(raw).hexdigest()


def _is_sha(value):
    return (isinstance(value, str) and len(value) == 64
            and all(character in "0123456789abcdef" for character in value))


def submission_receipt(record, pin_sha):
    if (not _is_sha(pin_sha) or not isinstance(record, dict)
            or record.get("pin_sha256") != pin_sha):
        raise ValueError("parent-seal submission record is not bound to its raw pin")
    return dict(record, schema="atlas-weyl-parent-seal-submission-v1",
                status="SUBMITTED_NOT_VERIFIED", pin_sha256=pin_sha,
                checker_tests=CHECKER_TESTS, legacy_gate_runs=2,
                core_inventory=629, domain_inventory=519,
                storage_policy="Single campaign stage. Login staging stores the frozen legacy harness only as an opaque CAS archive; compute expands it in an auto-cleaned workspace, seals the observed closure, and removes temporary source/target on ordinary exits.",
                scope="Infrastructure-only path-independent evidence seal; no Atlas runtime, mathematical result, rank or performance change.")


def _submission_record(record, stage):
    """Validate one exact submission record for its requested stage spelling."""
    if not isinstance(record, dict) or set(record) != SUBMISSION_RECORD_KEYS:
        raise ValueError("parent-seal submission record schema changed")
    queue = record.get("queue_before")
    if (record.get("stage") != stage
            or record.get("script") != SBATCH
            or record.get("status") != "SUBMITTED"
            or type(record.get("max_outstanding")) is not int
            or record["max_outstanding"] != 10
            or not isinstance(queue, list) or len(queue) >= 10
            or any(not isinstance(item, str)
                   or re.fullmatch(r"[0-9]+(?:_[0-9]+)?", item) is None
                   for item in queue)
            or len(queue) != len(set(queue))
            or not isinstance(record.get("job"), str)
            or not record["job"].isdecimal()
            or any(item.partition("_")[0] == record["job"] for item in queue)
            or not _is_sha(record.get("pin_sha256"))):
        raise ValueError("parent-seal submission record is invalid")
    return record


def _canonical_submission_record(record):
    """Validate the immutable path-free projection stored in the seal."""
    return _submission_record(record, SUBMISSION_STAGE)


def _absolute_submission_record(record):
    """Validate or reconstruct the exact pinned-campaign ledger preimage."""
    return _submission_record(record, SUBMISSION_ABSOLUTE_STAGE)


def submission_attestation(record, pin_sha, receipt):
    """Freeze independently checkable record/receipt preimages without paths."""
    absolute_record = _absolute_submission_record(record)
    if not _is_sha(pin_sha) or receipt != submission_receipt(absolute_record, pin_sha):
        raise ValueError("parent-seal submission receipt is not derived from its record")
    canonical_record = dict(absolute_record, stage=SUBMISSION_STAGE)
    canonical_receipt = submission_receipt(canonical_record, pin_sha)
    return {
        "schema": SUBMISSION_ATTESTATION_SCHEMA,
        "pin_sha256": pin_sha,
        "record": canonical_record,
        "record_sha256": _json_digest(absolute_record),
        "receipt": canonical_receipt,
        "receipt_sha256": _json_digest(receipt),
    }


def validate_submission_attestation(attestation):
    """Validate a submission binding using only path-free seal bytes."""
    expected_keys = {
        "schema", "pin_sha256", "record", "record_sha256", "receipt",
        "receipt_sha256",
    }
    if (not isinstance(attestation, dict) or set(attestation) != expected_keys
            or attestation.get("schema") != SUBMISSION_ATTESTATION_SCHEMA
            or not _is_sha(attestation.get("pin_sha256"))):
        raise ValueError("parent-seal submission attestation schema changed")
    record = _canonical_submission_record(attestation.get("record"))
    receipt = attestation.get("receipt")
    absolute_record = _absolute_submission_record(
        dict(record, stage=SUBMISSION_ABSOLUTE_STAGE))
    absolute_receipt = submission_receipt(
        absolute_record, attestation["pin_sha256"])
    if (record != attestation["record"]
            or attestation.get("record_sha256") != _json_digest(absolute_record)
            or receipt != submission_receipt(record, attestation["pin_sha256"])
            or attestation.get("receipt_sha256") != _json_digest(absolute_receipt)):
        raise ValueError("parent-seal submission attestation is not self-verifying")
    return record, receipt


def _blob_reference(reference, role):
    return (isinstance(reference, dict)
            and set(reference) == {"schema", "role", "sha256", "bytes"}
            and reference.get("schema") == BLOB_SCHEMA and reference.get("role") == role
            and _is_sha(reference.get("sha256"))
            and type(reference.get("bytes")) is int and reference["bytes"] >= 0)


def _source_reference(reference):
    return (isinstance(reference, dict)
            and set(reference) == {"schema", "role", "sha256", "bytes", "files", "source_bytes"}
            and reference.get("schema") == SOURCE_SCHEMA and reference.get("role") == "rust-source"
            and _is_sha(reference.get("sha256"))
            and all(type(reference.get(name)) is int and reference[name] >= 0
                    for name in ("bytes", "files", "source_bytes")))


def _string_inventory(value, count):
    return (isinstance(value, list) and len(value) == count and value == sorted(value)
            and all(isinstance(item, str) and item for item in value)
            and len(set(value)) == count)


def accepted_capture(report):
    commands = [dict(name=name, exit_status=0) for name in CAPTURE_COMMANDS]
    captures = [dict(case=case, comparison=comparison)
                for case, comparison in zip(CAPTURE_CASES, CAPTURE_COMPARISONS)]
    expected = dict(status="WEYL_CONTEXT_DISCOVERY_CAPTURED", integrity_rechecked=True,
                    production_unchanged=True, commands=commands, captures=captures)
    if report != expected:
        raise ValueError("complete unchanged original discovery required")


def _aggregate(files):
    if not isinstance(files, list):
        raise ValueError("invalid legacy evidence inventory")
    unique = {}
    try:
        for row in files:
            reference = row["object"]
            unique.setdefault(reference["sha256"], reference["bytes"])
            if unique[reference["sha256"]] != reference["bytes"]:
                raise ValueError("one digest has conflicting blob sizes")
        return dict(file_count=len(files), logical_bytes=sum(row["object"]["bytes"] for row in files),
                    unique_object_count=len(unique), unique_bytes=sum(unique.values()),
                    manifest_sha256=_json_digest(files))
    except (KeyError, TypeError) as error:
        raise ValueError("invalid legacy evidence object reference") from error


def _reject_absolute_paths(value):
    if value is None or isinstance(value, (bool, int, float)):
        return
    if isinstance(value, str):
        if PurePosixPath(value).is_absolute():
            raise ValueError("seal embeds an absolute path")
        return
    if isinstance(value, list):
        for child in value:
            _reject_absolute_paths(child)
        return
    if isinstance(value, dict):
        for key, child in value.items():
            if not isinstance(key, str):
                raise ValueError("seal has a non-string JSON key")
            if key == "path" or PurePosixPath(key).is_absolute():
                raise ValueError("seal embeds a path-bearing reference")
            _reject_absolute_paths(child)
        return
    raise ValueError("seal contains a non-JSON value")


def _json_artifact(campaign, reference, label):
    try:
        value = json.loads(read_blob(campaign, reference))
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise ValueError("invalid " + label + " JSON") from error
    if not isinstance(value, dict):
        raise ValueError("invalid " + label + " JSON")
    return value


def _trace_artifact(campaign, reference):
    raw = read_blob(campaign, reference).decode(errors="strict")
    rows = [line.removeprefix(TRACE_MARKER) for line in raw.splitlines()
            if line.startswith(TRACE_MARKER)]
    if len(rows) != 1:
        raise ValueError("legacy verifier log lacks one trace record")
    try:
        value = json.loads(rows[0])
    except json.JSONDecodeError as error:
        raise ValueError("invalid legacy trace JSON") from error
    if not isinstance(value, dict):
        raise ValueError("invalid legacy trace JSON")
    return value


def _capture_summary(report):
    try:
        return dict(status=report["status"], integrity_rechecked=report["integrity_rechecked"],
                    production_unchanged=report["production_unchanged"],
                    commands=[dict(name=row["name"], exit_status=row["exit_status"])
                              for row in report["commands"]],
                    captures=[dict(case=row["case"], comparison=row["comparison"])
                              for row in report["captures"]])
    except (KeyError, TypeError) as error:
        raise ValueError("invalid full capture report") from error


def _verify_execution_links(campaign, seal):
    artifacts = seal["execution_artifacts"]
    capture = _json_artifact(campaign, artifacts["capture-report"], "capture report")
    capture_pin = _json_artifact(campaign, artifacts["capture-pin"], "capture pin")
    command = _json_artifact(campaign, artifacts["command-after-report"], "command report")
    profile = _json_artifact(campaign, artifacts["profile-report"], "profile report")
    if (_capture_summary(capture) != seal["capture"]
            or command.get("source_files") != seal["source_manifest"]
            or command.get("inventory") != seal["core_inventory"]
            or _json_digest(command.get("source_files")) != SOURCE_MANIFEST_SHA256
            or _json_digest(command.get("inventory")) != CORE_INVENTORY_SHA256
            or command.get("binary", {}).get("sha256") != RUST_BINARY_SHA256
            or capture_pin.get("inputs") != seal["harness"]["manifest"]):
        raise ValueError("sealed reports do not reproduce source, inventory, capture, or harness")
    command_summary = dict(status=command.get("status"),
                           correctness_accepted=command.get("correctness_accepted"),
                           integrity_rechecked=command.get("integrity_rechecked"),
                           source_integrity_rechecked=command.get("source_integrity_rechecked"),
                           inventory=len(command.get("inventory", [])),
                           histories=len(command.get("histories", [])),
                           retained=len(command.get("retained", [])),
                           benchmarks=len(command.get("benchmarks", [])),
                           improvement_on_every_input=command.get("summary", {}).get("improvement_on_every_input"))
    profile_summary = dict(status=profile.get("status"),
                           integrity_rechecked=profile.get("integrity_rechecked"),
                           source_integrity_rechecked=profile.get("source_integrity_rechecked"),
                           commands=len(profile.get("commands", [])),
                           controls=len(profile.get("controls", [])),
                           profiles=len(profile.get("profiles", [])))
    if command_summary != seal["legacy"]["command_after"] or profile_summary != seal["legacy"]["profile"]:
        raise ValueError("sealed report summaries are not derived from the fixed reports")
    try:
        boundary = capture["captures"][2]
        boundary_hashes = {
            "boundary-input": boundary["case"]["source_sha256"],
            "boundary-oracle-stdout": boundary["observations"]["oracle"]["artifacts"]["stdout"]["sha256"],
            "boundary-oracle-stderr": boundary["observations"]["oracle"]["artifacts"]["stderr"]["sha256"],
            "boundary-rust-stdout": boundary["observations"]["rust"]["artifacts"]["stdout"]["sha256"],
            "boundary-rust-stderr": boundary["observations"]["rust"]["artifacts"]["stderr"]["sha256"],
        }
    except (IndexError, KeyError, TypeError) as error:
        raise ValueError("capture report lacks the complete boundary evidence") from error
    if any(artifacts[name]["sha256"] != sha for name, sha in boundary_hashes.items()):
        raise ValueError("boundary objects are not the capture's raw streams")

    first = _trace_artifact(campaign, artifacts["legacy-gate-first-log"])
    second = _trace_artifact(campaign, artifacts["legacy-gate-second-log"])
    if first != second:
        raise ValueError("complete legacy traces differ")
    expected_trace = dict(source_files=seal["source_manifest"], core_inventory=seal["core_inventory"],
                          legacy_inputs=seal["harness"]["manifest"],
                          command_after=seal["legacy"]["command_after"], profile=seal["legacy"]["profile"],
                          capture=seal["capture"], report_hashes=seal["report_hashes"],
                          scripts_manifest=seal["scripts"]["manifest"])
    if any(first.get(name) != value for name, value in expected_trace.items()):
        raise ValueError("legacy trace summary is not the sealed parent")
    try:
        traced = [dict(legacy_key=row["legacy_key"], sha256=row["sha256"], bytes=row["bytes"])
                  for row in first["files"]]
    except (KeyError, TypeError) as error:
        raise ValueError("legacy trace has an invalid file inventory") from error
    sealed = [dict(legacy_key=row["legacy_key"], sha256=row["object"]["sha256"],
                   bytes=row["object"]["bytes"]) for row in seal["legacy_files"]]
    if traced != sealed:
        raise ValueError("archival closure is not exactly the traced file set")
    for name, sha in (("rust", RUST_BINARY_SHA256), ("oracle", ORACLE_BINARY_SHA256)):
        if first.get("binaries", {}).get(name, {}).get("sha256") != sha:
            raise ValueError("legacy trace binary changed")


def validate_parent_seal(campaign, seal, *, verify_archival=False, verify_execution=True):
    if verify_archival and not verify_execution:
        raise ValueError("archival verification requires execution-object verification")
    if not isinstance(seal, dict):
        raise ValueError("invalid Weyl parent seal")
    if set(seal) != PARENT_SEAL_KEYS:
        raise ValueError("Weyl parent seal schema fields changed")
    _reject_absolute_paths(seal)
    source_manifest = seal.get("source_manifest")
    core_inventory = seal.get("core_inventory")
    domain_inventory = seal.get("domain_inventory")
    migration = seal.get("migration")
    migration_inputs = seal.get("migration_inputs")
    if not isinstance(migration, dict):
        raise ValueError("incomplete Weyl parent seal")
    try:
        submission_record, submission_receipt_value = validate_submission_attestation(
            migration.get("submission"))
    except ValueError as error:
        raise ValueError("incomplete Weyl parent seal") from error
    if (seal.get("schema") != SCHEMA or seal.get("status") != "WEYL_CONTEXT_PARENT_SEALED"
            or seal.get("legacy_gate_runs") != 2 or seal.get("trace_sets_equal") is not True
            or seal.get("archival_closure_verified") is not True
            or set(migration) != {"job", "node", "checker_tests",
                                  "toolchain_commands", "submission"}
            or not isinstance(migration.get("job"), str)
            or not migration["job"].isdecimal()
            or not isinstance(migration.get("node"), str)
            or not migration["node"]
            or type(migration.get("checker_tests")) is not int
            or migration["checker_tests"] != CHECKER_TESTS
            or type(migration.get("toolchain_commands")) is not int
            or migration["toolchain_commands"] != 2
            or migration.get("job") != submission_record["job"]
            or migration.get("checker_tests") != submission_receipt_value["checker_tests"]
            or not isinstance(migration_inputs, dict)
            or any(not isinstance(name, str) or not _is_sha(sha) for name, sha in migration_inputs.items())
            or not _string_inventory(domain_inventory, 519)
            or not _string_inventory(core_inventory, 629)
            or seal.get("core_inventory_sha256") != _json_digest(core_inventory)
            or seal.get("domain_inventory_sha256") != _json_digest(domain_inventory)
            or not isinstance(source_manifest, dict) or len(source_manifest) != 1558
            or seal.get("source_file_count") != 1558
            or seal.get("source_manifest_sha256") != _json_digest(source_manifest)
            or seal.get("report_hashes") != REPORT_HASHES
            or not _source_reference(seal.get("source_object"))
            or seal["source_object"]["files"] != 1558):
        raise ValueError("incomplete Weyl parent seal")
    legacy = seal.get("legacy") if isinstance(seal.get("legacy"), dict) else {}
    command, profile = legacy.get("command_after", {}), legacy.get("profile", {})
    if (command != dict(status="OVERLOAD_COMMAND_RETAINED_RANK1_AB_IMPROVES",
                        correctness_accepted=True, integrity_rechecked=True,
                        source_integrity_rechecked=True, inventory=629,
                        histories=26, retained=72, benchmarks=16,
                        improvement_on_every_input=True)
            or profile != dict(status="OVERLOAD_COMMAND_PROFILE_CAPTURED",
                               integrity_rechecked=True, source_integrity_rechecked=True,
                               commands=4, controls=2, profiles=4)):
        raise ValueError("accepted parent summary changed")
    accepted_capture(seal.get("capture"))

    files = seal.get("legacy_files")
    if (not isinstance(files, list) or len(files) < 1600
            or any(not isinstance(row, dict) or set(row) != {"legacy_key", "object"}
                   or not isinstance(row.get("legacy_key"), str)
                   or row["legacy_key"] == "." or not PurePosixPath(row["legacy_key"]).parts
                   or PurePosixPath(row["legacy_key"]).is_absolute()
                   or PurePosixPath(row["legacy_key"]).as_posix() != row["legacy_key"]
                   or any(part in ("", ".", "..") for part in PurePosixPath(row["legacy_key"]).parts)
                   or not _blob_reference(row.get("object"), "legacy-evidence") for row in files)
            or files != sorted(files, key=lambda row: row["legacy_key"])
            or len({row["legacy_key"] for row in files}) != len(files)
            or _aggregate(files) != seal.get("closure")):
        raise ValueError("legacy evidence closure changed")
    by_reference = {(row["object"]["sha256"], row["object"]["bytes"]) for row in files}

    artifacts = seal.get("execution_artifacts")
    if not isinstance(artifacts, dict) or set(artifacts) != EXECUTION_ARTIFACTS:
        raise ValueError("execution artifact inventory changed")
    for name, reference in artifacts.items():
        role = "legacy-attestation" if name.startswith("legacy-gate-") else "legacy-evidence"
        if not _blob_reference(reference, role):
            raise ValueError("invalid execution artifact reference")
        if name in FIXED_ARTIFACT_HASHES and reference["sha256"] != FIXED_ARTIFACT_HASHES[name]:
            raise ValueError("fixed execution artifact identity changed")
        if verify_execution:
            verify_blob(campaign, reference)
        if not name.startswith("legacy-gate-") and (reference["sha256"], reference["bytes"]) not in by_reference:
            raise ValueError("execution artifact absent from archival closure")

    binaries = seal.get("binaries")
    if (not isinstance(binaries, dict) or set(binaries) != {"rust", "oracle"}
            or not _blob_reference(binaries.get("rust"), "legacy-evidence")
            or not _blob_reference(binaries.get("oracle"), "legacy-evidence")
            or binaries["rust"]["sha256"] != RUST_BINARY_SHA256
            or binaries["oracle"]["sha256"] != ORACLE_BINARY_SHA256):
        raise ValueError("accepted binary identity changed")
    for reference in binaries.values():
        if verify_execution:
            verify_blob(campaign, reference)
        if (reference["sha256"], reference["bytes"]) not in by_reference:
            raise ValueError("binary absent from archival closure")

    for group in ("harness", "scripts"):
        bundle = seal.get(group)
        if not isinstance(bundle, dict) or set(bundle) != {"manifest", "objects"}:
            raise ValueError(group + " tree closure changed")
        manifest, objects = bundle["manifest"], bundle["objects"]
        if (not isinstance(manifest, dict) or not manifest or not isinstance(objects, dict)
                or set(manifest) != set(objects)
                or any(not isinstance(name, str) or not _is_sha(sha)
                       or not _blob_reference(objects.get(name), "legacy-evidence")
                       or objects[name]["sha256"] != sha for name, sha in manifest.items())):
            raise ValueError(group + " tree closure changed")
        for reference in objects.values():
            if (reference["sha256"], reference["bytes"]) not in by_reference:
                raise ValueError(group + " object absent from archival closure")
    if len(seal["harness"]["manifest"]) != 341:
        raise ValueError("legacy harness input inventory changed")
    if (set(migration_inputs) != MIGRATION_OVERRIDE_NAMES
            or {name for name in migration_inputs if name.startswith("hpc/stage_")}
               != {"hpc/stage_weyl_parent_seal.py"}):
        raise ValueError("migration root contains an unsafe or incomplete stager set")

    if verify_execution:
        verify_source_archive(campaign, seal["source_object"], source_manifest)
        _verify_execution_links(campaign, seal)
    if verify_archival:
        seen = set()
        for row in files:
            identity = (row["object"]["sha256"], row["object"]["bytes"])
            if identity not in seen:
                verify_blob(campaign, row["object"])
                seen.add(identity)
    return seal


def load_parent_seal(campaign, reference, *, verify_archival=False):
    if not _blob_reference(reference, "weyl-parent-seal"):
        raise ValueError("wrong parent seal role")
    try:
        seal = json.loads(read_blob(campaign, reference))
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise ValueError("invalid parent seal JSON") from error
    return validate_parent_seal(campaign, seal, verify_archival=verify_archival, verify_execution=True)


def boundary_bytes(campaign, seal):
    artifacts = seal["execution_artifacts"]
    return {
        "input": read_blob(campaign, artifacts["boundary-input"]),
        "oracle_stdout": read_blob(campaign, artifacts["boundary-oracle-stdout"]),
        "oracle_stderr": read_blob(campaign, artifacts["boundary-oracle-stderr"]),
        "rust_stdout": read_blob(campaign, artifacts["boundary-rust-stdout"]),
        "rust_stderr": read_blob(campaign, artifacts["boundary-rust-stderr"]),
    }

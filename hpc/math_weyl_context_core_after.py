"""Run one tests-first Weyl-context regression AFTER gate.

The driver reruns the same frozen gate on the repaired source and proves that
the two known Rust regressions now pass, with the repaired Rust matching every
frozen original golden.  Expanded source, Cargo output, binaries, and scripts
remain inside one disposable compute-node workspace.  This gate records the
repaired passes and releases no mathematical, cache, performance, memory, or
rank claim.
"""

import copy
import hashlib
import json
import math
import os
from pathlib import Path, PurePosixPath
import platform
import re
import signal
import stat
import subprocess
import sys
import time
import traceback


# This hook constrains Python-level opens made by this driver.  It cannot see
# syscalls made inside Cargo, Rust, or original-Atlas child processes; that
# limitation is retained explicitly in every successful report.
_ACTIVE_CAMPAIGN = "atlas-rust-campaign-20260930"
_HPC_HOME = Path("/public/home/majj")
LEGACY_PATH_OPEN_ATTEMPTS = []


def forbidden_legacy_path(value):
    if not isinstance(value, (str, bytes, os.PathLike)):
        return False
    try:
        path = Path(os.path.abspath(os.fsdecode(value)))
        relative = path.relative_to(_HPC_HOME)
    except (TypeError, ValueError, UnicodeError):
        return False
    return (bool(relative.parts)
            and relative.parts[0].startswith("atlas-")
            and relative.parts[0] != _ACTIVE_CAMPAIGN)


def _path_audit(event, arguments):
    if event == "open" and arguments and forbidden_legacy_path(arguments[0]):
        LEGACY_PATH_OPEN_ATTEMPTS.append(os.fsdecode(arguments[0]))
        raise RuntimeError("retired top-level Atlas path access rejected")


sys.addaudithook(_path_audit)

from campaign_blob import read_blob
from campaign_source import digest, file_manifest, materialize_source_archive
from campaign_workspace import (
    ACTIVE_CAMPAIGN, campaign_stage, create_result_folder,
    ephemeral_job_workspace,
)
from progressive_submit import read_json_file, validate_stage_creation
from stage_weyl_context_core_after import (
    ACCEPTED_SOURCE,
    CATALOG_CASES,
    CATALOG_PATH,
    CATALOG_SHA256,
    EXPECTED_TEST_COUNTS,
    FINAL_HASHES,
    PATCH_HASHES,
    PIN_NAME,
    PIN_SCHEMA,
    PARENT_SEAL_REFERENCE,
    PARENT_SOURCE_OBJECT,
    RETIRED_STAGER_BUNDLE_REFERENCE,
    SBATCH,
    SEALED_FIXTURE_HASHES,
    STAGE_LOCK,
    STAGE_NAME,
    SUBMISSION_ENABLED as STAGER_SUBMISSION_ENABLED,
    TESTS_ONLY_HASHES,
    REPAIR_PATCH_HASHES,
    REPAIR_PATCH_PATH,
    REPAIRED_SOURCE_HASHES,
    AFTER_SOURCE_FILES,
    AFTER_SOURCE_MANIFEST_SHA256,
    REGRESSION_CATALOG_BYTES,
    REGRESSION_CATALOG_PATH,
    REGRESSION_CATALOG_SHA256,
    REGRESSION_FIXTURE_HASHES,
    REGRESSION_INSPECTION_BYTES,
    REGRESSION_INSPECTION_PATH,
    REGRESSION_INSPECTION_SHA256,
    REGRESSION_PATCH_HASHES,
    REGRESSION_PATCH_PATH,
    REGRESSION_SOURCE,
    REGRESSION_SOURCE_HASHES,
    confirmed_record,
    existing_lock,
    frozen_stage_inputs,
    saved_json_sha,
    submission_receipt,
    validate_catalog as validate_staged_catalog,
    validate_capture_v1_failure,
    validate_capture_v2_failure,
    validate_capture_v3_failure,
    validate_capture_v4_failure,
    validate_capture_v5_failure,
    validate_capture_v6_failure,
    validate_capture_v7_failure,
    validate_capture_v8,
    validate_before_v1_failure,
    validate_before_v2_failure,
    validate_before_v3_failure,
    validate_after_v1_failure,
    validate_after_v2_failure,
    validate_after_v3_failure,
    validate_after_v4_failure,
    validate_before_v4_result,
    validate_parent_objects,
    validate_pin as validate_stage_pin,
    validate_predecessor,
    validate_prior_creation_failure,
    validate_regression_inputs,
    validate_stage_topology,
)
from weyl_context_core_contract import (
    CAPTURE_MATURITY,
    NONACCEPTING_STATUSES,
    classify_capture,
    decode_catalog as decode_discovery_catalog,
    validate_catalog as validate_contract_catalog,
)
from weyl_context_core_regression import (
    AFTER_HARNESS_FAILURE,
    AFTER_PROVENANCE_FAILURE,
    AFTER_REPRODUCED,
    AFTER_STILL_FAILING,
    AFTER_OBSERVATION_SCHEMA,
    classify_after,
    decode_catalog as decode_regression_catalog,
    decode_inspection as decode_regression_inspection,
    expected_provenance as expected_regression_provenance,
    validate_artifacts as validate_regression_artifacts,
)
from weyl_parent_seal import boundary_bytes, load_parent_seal


if ACTIVE_CAMPAIGN != _ACTIVE_CAMPAIGN:
    raise RuntimeError("Weyl core after campaign policy changed")


# This exact driver is enabled only with its matching sole-active stager.  The
# guard in main() precedes environment parsing and filesystem I/O.
SUBMISSION_ENABLED = False
EXPECTED_STAGE = (
    "/public/home/majj/atlas-rust-campaign-20260930/stages/"
    "weyl-context-core-after-v5"
)

REPORT_SCHEMA = "atlas-weyl-context-core-after-v5"
SUCCESS_STATUS = "WEYL_CONTEXT_AFTER_REGRESSIONS_PASS"
INCOMPLETE_STATUS = "WEYL_CONTEXT_CORE_AFTER_INCOMPLETE"
REPORT_SCOPE = (
    "Tests-first AFTER verification of the Weyl-owner repair on the unchanged "
    "frozen inputs. It applies the reviewed repair patch, retains the four "
    "discovery captures, exact oracle goldens, the retained ladder control, "
    "the complete test inventory and resource metrics, and requires both "
    "regressions to pass with the repaired Rust matching every golden; it "
    "releases no gate."
)
COMMAND_TIMEOUT_SECONDS = 1200
COMMAND_KILL_AFTER_SECONDS = 30
COMMAND_EXIT_GRACE_SECONDS = 5
CHECKER_PYTHON = "/public/software/anaconda/anaconda3-2022.5/bin/python3.9"
EXPECTED_INVOCATIONS = (
    ("weyl_context_core_cold_dual", "oracle"),
    ("weyl_context_core_cold_dual", "rust"),
    ("weyl_context_core_prewarmed_dual", "rust"),
    ("weyl_context_core_prewarmed_dual", "oracle"),
)
COMMAND_NAMES = (
    "test-campaign-stage-creation",
    "test-progressive-submit",
    "test-weyl-context-core-contract",
    "test-weyl-context-core-regression-contract",
    "test-math-weyl-context-core-after",
    "test-stager-allowlist",
    "rustc-version",
    "cargo-version",
    "source-reconstruction",
    "release-build",
    "atlas-core-test-inventory",
    "weyl-context-regressions",
    "root-ladder-control",
)
CHECK_COMMANDS = (
    ("test-campaign-stage-creation", "test_campaign_stage_creation.py"),
    ("test-progressive-submit", "test_progressive_submit.py"),
    ("test-weyl-context-core-contract", "test_weyl_context_core_contract.py"),
    (
        "test-weyl-context-core-regression-contract",
        "test_weyl_context_core_regression.py",
    ),
    ("test-math-weyl-context-core-after", "test_math_weyl_context_core_after.py"),
    ("test-stager-allowlist", "test_stager_allowlist.py"),
)
REPORT_KEYS = {
    "schema", "job", "node", "status", "evidence_maturity", "scope",
    "pin", "accepted_source", "catalog", "commands", "invocations",
    "captures", "complete", "acceptance_eligible", "math_gate_released",
    "cache_gate_released", "integrity_rechecked",
    "ephemeral_workspace_removed", "source", "binaries", "limitations",
    "legacy_path_open_attempts", "thread_settings",
    "source_integrity_rechecked", "provenance", "environment", "regression",
    "performance_gate_released", "rank_gate_released",
}
PROVENANCE_KEYS = {
    "pin_sha256", "submission_receipt", "harness_inputs", "campaign_record",
}
INVOCATION_KEYS = {
    "sequence", "case_id", "engine", "invocation_id", "pid",
    "process_group_id", "fresh_process", "observation", "stdout", "stderr",
    "time", "input", "executable_sha256", "executable_bytes",
}
OBSERVATION_KEYS = {
    "engine", "exit_status", "timed_out", "termination_uncertain", "signal",
    "seconds", "user_cpu_seconds", "system_cpu_seconds", "maxrss_kb",
    "maxrss_approximate",
}
ARTIFACT_KEYS = {"path", "sha256", "bytes"}
COMMAND_KEYS = {
    "name", "argv", "cwd", "exit_status", "timed_out",
    "termination_uncertain", "signal", "seconds", "user_cpu_seconds",
    "system_cpu_seconds", "maxrss_kb", "maxrss_approximate", "stdout",
    "stderr", "time",
}
CATALOG_RECORD = {
    "artifact": {
        "path": "catalog.json",
        "sha256": CATALOG_SHA256,
        "bytes": 1283,
    },
    "cases": 2,
    "fresh_processes": 4,
}
REGRESSION_CATALOG_RECORD = {
    "path": "regression-catalog.json",
    "sha256": REGRESSION_CATALOG_SHA256,
    "bytes": REGRESSION_CATALOG_BYTES,
}
REGRESSION_INSPECTION_RECORD = {
    "path": "capture-v8-inspection.json",
    "sha256": REGRESSION_INSPECTION_SHA256,
    "bytes": REGRESSION_INSPECTION_BYTES,
}
REGRESSION_KEYS = {
    "catalog", "inspection", "artifacts", "classification",
    "original_rerun_invocations", "selector_command",
    "retained_control_command", "inventory_command",
}
CLAIM_KEYS = {
    "acceptance_eligible", "math_gate_released", "cache_gate_released",
    "performance_gate_released", "rank_gate_released",
}
EXPECTED_COMMAND_EXITS = {
    "test-campaign-stage-creation": {0},
    "test-progressive-submit": {0},
    "test-weyl-context-core-contract": {0},
    "test-weyl-context-core-regression-contract": {0},
    "test-math-weyl-context-core-after": {0},
    "test-stager-allowlist": {0},
    "rustc-version": {0},
    "cargo-version": {0},
    "source-reconstruction": {0},
    "release-build": {0},
    "atlas-core-test-inventory": {0},
    "weyl-context-regressions": {0, 101},
    "root-ladder-control": {0},
}
THREAD_SETTINGS = {
    "RAYON_NUM_THREADS": "1",
    "OMP_NUM_THREADS": "1",
    "OPENBLAS_NUM_THREADS": "1",
}
ENVIRONMENT_KEYS = {
    "PATH", "HOME", "LD_LIBRARY_PATH", "CARGO_HOME", "RUSTUP_HOME",
    "CARGO_TARGET_DIR", "CARGO_BUILD_JOBS", "CARGO_INCREMENTAL",
    "CARGO_NET_OFFLINE", "CARGO_TERM_COLOR", "RAYON_NUM_THREADS",
    "OMP_NUM_THREADS", "OPENBLAS_NUM_THREADS", "LC_ALL", "LANG",
    "PYTHONDONTWRITEBYTECODE", "TMPDIR", "TMP", "TEMP",
}
LIMITATIONS = [
    (
        "The Python audit hook covers only this driver process; it does not "
        "prove that Cargo, Rust, or original-Atlas child system calls avoided "
        "retired top-level paths."
    ),
    (
        "This is tests-first AFTER evidence that the reviewed repair makes "
        "the two regressions pass and matches the goldens; it is not a "
        "mathematical acceptance beyond that exact scope."
    ),
    (
        "This AFTER releases no mathematical, cache, rank, operation, "
        "parallel, memory, or performance gate."
    ),
    (
        "The source-reconstruction command times only the pinned manifest "
        "verifier after untimed archive extraction and patching; its metrics "
        "are approximate for that phase and are not used for a speed claim."
    ),
    (
        "Capture resource metrics include one fixed bash exec wrapper used "
        "to execute the already-open, pre-hashed executable descriptor; "
        "they are not accepted performance measurements."
    ),
    (
        "Cargo-test timing may include release test-harness compilation; it "
        "is legitimate AFTER resource evidence, not an accepted benchmark."
    ),
]
FORBIDDEN_REPORT_KEYS = {
    "speed_ratio", "speedup", "acceptance_index", "review_evidence", "accepted",
}
COMPLETE_CAPTURE_STATUSES = {
    "ORIGINAL_SOURCE_PREDICTION_DIFFERED",
    "RUST_SOURCE_PREDICTION_DIFFERED",
    "BOTH_SOURCE_PREDICTIONS_DIFFERED",
    "SOURCE_PREDICTIONS_OBSERVED_UNREVIEWED",
}

ROOT_SYSTEM = "crates/atlas-real-group/src/root_system.rs"
SESSION = "crates/atlas-core/src/session.rs"
DOMAIN_BUILTINS = "crates/atlas-core/src/domain_builtins.rs"
TYPED = "crates/atlas-core/src/typed.rs"
WEYL_SUBGROUP = "crates/atlas-core/src/domain_builtins/weyl_subgroup.rs"
TEST_PATCH = "hpc/patches/ladder_boundary_tests.patch"
PRODUCTION_PATCH = "hpc/patches/ladder_boundary_fix.patch"
BOUNDARY_FIXTURE = "tests/math/generics/root_ladder_coordinate_boundary.atlas"
BOUNDARY_STDOUT = (
    "tests/math/generics/root_ladder_coordinate_boundary.oracle.stdout"
)
BOUNDARY_STDERR = (
    "tests/math/generics/root_ladder_coordinate_boundary.oracle.stderr"
)
SHA256_PATTERN = r"[0-9a-f]{64}\Z"
INVOCATION_ID_PATTERN = r"[A-Za-z0-9][A-Za-z0-9_.-]{0,127}\Z"

MANIFEST_PROGRAM = r"""
import hashlib
import json
import pathlib
import stat
import sys

root = pathlib.Path(sys.argv[2])
expected = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
observed = {}
for path in sorted(root.rglob("*")):
    value = path.lstat()
    if stat.S_ISLNK(value.st_mode):
        raise SystemExit("source contains a symlink")
    if stat.S_ISDIR(value.st_mode):
        continue
    if not stat.S_ISREG(value.st_mode):
        raise SystemExit("source contains a non-regular file")
    sha = hashlib.sha256()
    with path.open("rb") as handle:
        for block in iter(lambda: handle.read(1024 * 1024), b""):
            sha.update(block)
    observed[path.relative_to(root).as_posix()] = sha.hexdigest()
if observed != expected:
    raise SystemExit("reconstructed source manifest differs")
canonical = json.dumps(observed, sort_keys=True, separators=(",", ":")).encode()
print("SOURCE_RECONSTRUCTION_OK", len(observed), hashlib.sha256(canonical).hexdigest())
"""

COMMAND_CONTRACTS = {
    "test-campaign-stage-creation": {
        "argv": [
            CHECKER_PYTHON, "-I", "-S", "-B", "-m", "unittest", "discover",
            "-s", "hpc", "-p", "test_campaign_stage_creation.py", "-v",
        ],
        "cwd": ".",
    },
    "test-progressive-submit": {
        "argv": [
            CHECKER_PYTHON, "-I", "-S", "-B", "-m", "unittest", "discover",
            "-s", "hpc", "-p", "test_progressive_submit.py", "-v",
        ],
        "cwd": ".",
    },
    "test-weyl-context-core-contract": {
        "argv": [
            CHECKER_PYTHON, "-I", "-S", "-B", "-m", "unittest", "discover",
            "-s", "hpc", "-p", "test_weyl_context_core_contract.py", "-v",
        ],
        "cwd": ".",
    },
    "test-weyl-context-core-regression-contract": {
        "argv": [
            CHECKER_PYTHON, "-I", "-S", "-B", "-m", "unittest", "discover",
            "-s", "hpc", "-p", "test_weyl_context_core_regression.py", "-v",
        ],
        "cwd": ".",
    },
    "test-math-weyl-context-core-after": {
        "argv": [
            CHECKER_PYTHON, "-I", "-S", "-B", "-m", "unittest", "discover",
            "-s", "hpc", "-p", "test_math_weyl_context_core_after.py", "-v",
        ],
        "cwd": ".",
    },
    "test-stager-allowlist": {
        "argv": [
            CHECKER_PYTHON, "-I", "-S", "-B", "-m", "unittest", "discover",
            "-s", "hpc", "-p", "test_stager_allowlist.py", "-v",
        ],
        "cwd": ".",
    },
    "rustc-version": {"argv": ["rustc", "-vV"], "cwd": "."},
    "cargo-version": {"argv": ["cargo", "-vV"], "cwd": "."},
    "source-reconstruction": {
        "argv": [
            CHECKER_PYTHON, "-I", "-S", "-B", "-c", MANIFEST_PROGRAM,
            "expected-source-manifest.json", "source",
        ],
        "cwd": "workspace",
    },
    "release-build": {
        "argv": [
            "cargo", "build", "--offline", "--locked", "--release", "-p",
            "atlas-cli",
        ],
        "cwd": "workspace/source",
    },
    "atlas-core-test-inventory": {
        "argv": [
            "cargo", "test", "--offline", "--locked", "--release", "-p",
            "atlas-core", "--lib", "--", "--list",
        ],
        "cwd": "workspace/source",
    },
    "weyl-context-regressions": {
        "argv": [
            "cargo", "test", "--offline", "--locked", "--release", "-p",
            "atlas-core", "--lib", "weyl_context_core_", "--",
            "--test-threads=1", "--nocapture",
        ],
        "cwd": "workspace/source",
    },
    "root-ladder-control": {
        "argv": [
            "cargo", "test", "--offline", "--locked", "--release", "-p",
            "atlas-core", "--lib",
            "session::tests::root_ladder_coordinate_boundary_original", "--",
            "--exact", "--test-threads=1", "--nocapture",
        ],
        "cwd": "workspace/source",
    },
}


def _sha(raw):
    return hashlib.sha256(raw).hexdigest()


def _number(value, *, integral=False, nonnegative=False):
    if integral:
        valid = type(value) is int
    else:
        valid = type(value) in (int, float) and math.isfinite(value)
    return valid and (not nonnegative or value >= 0)


def _safe_relative(name):
    if not isinstance(name, str) or not name:
        raise ValueError("artifact path must be a nonempty relative POSIX path")
    pure = PurePosixPath(name)
    if (pure.is_absolute() or pure.as_posix() != name
            or any(part in ("", ".", "..") for part in pure.parts)):
        raise ValueError("unsafe artifact path")
    return pure.parts


def _stable_relative_bytes(root, name):
    """Read one result artifact without following any path component."""
    parts = _safe_relative(name)
    nofollow = getattr(os, "O_NOFOLLOW", None)
    directory = getattr(os, "O_DIRECTORY", None)
    if nofollow is None or directory is None:
        raise ValueError("safe no-follow artifact traversal is unavailable")
    descriptors = []
    leaf = current_leaf = None
    try:
        current = os.open(root, os.O_RDONLY | os.O_CLOEXEC | directory | nofollow)
        descriptors.append(current)
        for part in parts[:-1]:
            current = os.open(
                part, os.O_RDONLY | os.O_CLOEXEC | directory | nofollow,
                dir_fd=current,
            )
            descriptors.append(current)
        leaf = os.open(
            parts[-1], os.O_RDONLY | os.O_CLOEXEC | nofollow, dir_fd=current,
        )
        before = os.fstat(leaf)
        if (not stat.S_ISREG(before.st_mode)
                or stat.S_IMODE(before.st_mode) != 0o444
                or before.st_nlink != 1):
            raise ValueError("artifact is not one immutable single-link file")
        chunks = []
        while True:
            block = os.read(leaf, 64 * 1024)
            if not block:
                break
            chunks.append(block)
        after = os.fstat(leaf)
        current_leaf = os.open(
            parts[-1], os.O_RDONLY | os.O_CLOEXEC | nofollow, dir_fd=current,
        )
        present = os.fstat(current_leaf)
        identity = lambda value: (
            value.st_dev, value.st_ino, value.st_size, value.st_mtime_ns,
            value.st_ctime_ns, stat.S_IFMT(value.st_mode), value.st_nlink,
        )
        raw = b"".join(chunks)
        if (identity(before) != identity(after)
                or identity(before) != identity(present)
                or len(raw) != before.st_size):
            raise ValueError("artifact changed while it was read")
        return raw
    except OSError as error:
        raise ValueError("artifact could not be read safely") from error
    finally:
        if current_leaf is not None:
            os.close(current_leaf)
        if leaf is not None:
            os.close(leaf)
        for descriptor in reversed(descriptors):
            os.close(descriptor)


def _write_immutable(path, raw, mode=0o444):
    path = Path(path)
    if type(raw) is not bytes or path.name in ("", ".", ".."):
        raise ValueError("invalid immutable artifact")
    flags = (os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_CLOEXEC
             | getattr(os, "O_NOFOLLOW", 0))
    descriptor = None
    try:
        descriptor = os.open(path, flags, 0o600)
        remaining = memoryview(raw)
        while remaining:
            written = os.write(descriptor, remaining)
            if written <= 0:
                raise OSError("short artifact write")
            remaining = remaining[written:]
        os.fchmod(descriptor, mode)
        os.fsync(descriptor)
    finally:
        if descriptor is not None:
            os.close(descriptor)
    directory = os.open(
        path.parent,
        os.O_RDONLY | os.O_CLOEXEC | getattr(os, "O_DIRECTORY", 0)
        | getattr(os, "O_NOFOLLOW", 0),
    )
    try:
        os.fsync(directory)
    finally:
        os.close(directory)


def _publish_artifact(out, name, raw):
    _safe_relative(name)
    if len(PurePosixPath(name).parts) != 1:
        raise ValueError("capture artifacts must be direct result children")
    _write_immutable(Path(out) / name, raw)
    return {"path": name, "sha256": _sha(raw), "bytes": len(raw)}


def _validate_artifact(reference, result_dir=None):
    if (type(reference) is not dict or frozenset(reference) != ARTIFACT_KEYS
            or not isinstance(reference.get("sha256"), str)
            or re.fullmatch(SHA256_PATTERN, reference["sha256"]) is None
            or type(reference.get("bytes")) is not int
            or reference["bytes"] < 0):
        raise ValueError("invalid raw-artifact reference")
    _safe_relative(reference.get("path"))
    if result_dir is not None:
        raw = _stable_relative_bytes(Path(result_dir), reference["path"])
        if len(raw) != reference["bytes"] or _sha(raw) != reference["sha256"]:
            raise ValueError("raw artifact changed")
        return raw
    return None


def _elapsed_seconds(value):
    pieces = value.split(":")
    if len(pieces) not in (2, 3):
        return None
    try:
        seconds = float(pieces[-1])
        minutes = int(pieces[-2])
        hours = int(pieces[0]) if len(pieces) == 3 else 0
    except ValueError:
        return None
    if (hours < 0 or minutes < 0
            or (len(pieces) == 3 and minutes >= 60) or seconds < 0
            or seconds >= 60 or not math.isfinite(seconds)):
        return None
    return hours * 3600 + minutes * 60 + seconds


def _parse_time(raw):
    try:
        text = raw.decode("utf-8", errors="strict")
    except UnicodeDecodeError:
        return None
    patterns = {
        "user_cpu_seconds":
            r"(?m)^\s*User time \(seconds\):\s*([0-9.]+)\s*$",
        "system_cpu_seconds":
            r"(?m)^\s*System time \(seconds\):\s*([0-9.]+)\s*$",
        "maxrss_kb":
            r"(?m)^\s*Maximum resident set size \(kbytes\):\s*(\d+)\s*$",
        "elapsed": (
            r"(?m)^\s*Elapsed \(wall clock\) time "
            r"\(h:mm:ss or m:ss\):\s*([0-9:.]+)\s*$"
        ),
        "exit_status": r"(?m)^\s*Exit status:\s*(\d+)\s*$",
    }
    matches = {name: re.findall(pattern, text)
               for name, pattern in patterns.items()}
    if any(len(values) != 1 for values in matches.values()):
        return None
    signals = re.findall(
        r"(?m)^\s*Command terminated by signal ([0-9]+)\s*$", text,
    )
    if len(signals) > 1:
        return None
    try:
        result = {
            "seconds": _elapsed_seconds(matches["elapsed"][0]),
            "user_cpu_seconds": float(matches["user_cpu_seconds"][0]),
            "system_cpu_seconds": float(matches["system_cpu_seconds"][0]),
            "maxrss_kb": int(matches["maxrss_kb"][0]),
            "exit_status": int(matches["exit_status"][0]),
            "signal": int(signals[0]) if signals else None,
        }
    except ValueError:
        return None
    if (result["seconds"] is None
            or any(not _number(result[name], nonnegative=True)
                   for name in ("seconds", "user_cpu_seconds",
                                "system_cpu_seconds"))
            or result["maxrss_kb"] < 0 or result["exit_status"] < 0):
        return None
    return result


def _time_matches_record(raw, record):
    metrics = _parse_time(raw)
    return (
        metrics is not None
        and record.get("seconds") == metrics["seconds"]
        and record.get("user_cpu_seconds") == metrics["user_cpu_seconds"]
        and record.get("system_cpu_seconds") == metrics["system_cpu_seconds"]
        and record.get("maxrss_kb") == metrics["maxrss_kb"]
        and record.get("exit_status") == metrics["exit_status"]
        and record.get("signal") == metrics["signal"]
    )


def _forbid_claim_keys(value):
    if isinstance(value, dict):
        for key, child in value.items():
            if key in FORBIDDEN_REPORT_KEYS:
                raise ValueError("capture report contains a forbidden claim key")
            _forbid_claim_keys(child)
    elif isinstance(value, list):
        for child in value:
            _forbid_claim_keys(child)


def invocation_plan(catalog):
    validate_contract_catalog(catalog)
    cases = {case["id"]: case for case in catalog["cases"]}
    plan = []
    for sequence, (case_id, engine) in enumerate(EXPECTED_INVOCATIONS):
        case = cases.get(case_id)
        if case is None:
            raise ValueError("capture case is absent from the catalog")
        plan.append({
            "sequence": sequence,
            "case_id": case_id,
            "engine": engine,
            "timeout_seconds": case["timeout_seconds"],
        })
    return plan


def command_environment(inherited, work):
    if not isinstance(inherited, dict):
        raise ValueError("command environment must be a mapping")
    required = ("PATH", "HOME", "LD_LIBRARY_PATH")
    if any(not isinstance(inherited.get(key), str) or not inherited[key]
           for key in required):
        raise ValueError("required controlled command environment is absent")
    work = Path(work)
    env = {key: inherited[key] for key in required}
    env.update(
        CARGO_HOME=str(Path(env["HOME"]) / ".cargo"),
        RUSTUP_HOME=str(Path(env["HOME"]) / ".rustup"),
        CARGO_TARGET_DIR=str(work / "target"),
        CARGO_BUILD_JOBS="2",
        CARGO_INCREMENTAL="0",
        CARGO_NET_OFFLINE="true",
        CARGO_TERM_COLOR="never",
        RAYON_NUM_THREADS="1",
        OMP_NUM_THREADS="1",
        OPENBLAS_NUM_THREADS="1",
        LC_ALL="C",
        LANG="C",
        PYTHONDONTWRITEBYTECODE="1",
        TMPDIR=str(work / "tmp"),
        TMP=str(work / "tmp"),
        TEMP=str(work / "tmp"),
    )
    return env


def validate_environment_record(env):
    if (not isinstance(env, dict) or set(env) != ENVIRONMENT_KEYS
            or any(not isinstance(value, str) or not value
                   for value in env.values())):
        raise ValueError("command environment record changed")
    home = Path(env["HOME"])
    target = Path(env["CARGO_TARGET_DIR"])
    temporary = Path(env["TMPDIR"])
    if (not home.is_absolute()
            or env["CARGO_HOME"] != str(home / ".cargo")
            or env["RUSTUP_HOME"] != str(home / ".rustup")
            or not target.is_absolute() or target.name != "target"
            or not temporary.is_absolute() or temporary.name != "tmp"
            or target.parent != temporary.parent
            or env["TMP"] != env["TMPDIR"] or env["TEMP"] != env["TMPDIR"]
            or env["CARGO_BUILD_JOBS"] != "2"
            or env["CARGO_INCREMENTAL"] != "0"
            or env["CARGO_NET_OFFLINE"] != "true"
            or env["CARGO_TERM_COLOR"] != "never"
            or env["LC_ALL"] != "C" or env["LANG"] != "C"
            or env["PYTHONDONTWRITEBYTECODE"] != "1"
            or any(env[key] != value for key, value in THREAD_SETTINGS.items())):
        raise ValueError("command environment is not the controlled capture environment")
    return copy.deepcopy(env)


def validate_pin(pin, current_inputs):
    value = validate_stage_pin(pin)
    if (not isinstance(current_inputs, dict)
            or current_inputs != value["inputs"]
            or value["schema"] != PIN_SCHEMA
            or value["test_counts"] != EXPECTED_TEST_COUNTS
            or value["accepted_source"] != ACCEPTED_SOURCE
            or value["parent_seal_object"] != PARENT_SEAL_REFERENCE
            or value["retired_stager_object"]
               != RETIRED_STAGER_BUNDLE_REFERENCE
            or value["catalog"].get("cases") != CATALOG_CASES
            or value["catalog"].get("fresh_processes") != 4):
        raise ValueError("Weyl core capture pin differs from installed inputs")
    return value


def _validate_observation(observation, engine, *, require_complete=False):
    if (type(observation) is not dict
            or frozenset(observation) != OBSERVATION_KEYS
            or observation.get("engine") != engine
            or type(observation.get("exit_status")) is not int
            or type(observation.get("timed_out")) is not bool
            or type(observation.get("termination_uncertain")) is not bool
            or type(observation.get("maxrss_approximate")) is not bool):
        raise ValueError("invalid capture observation")
    captured_signal = observation.get("signal")
    if captured_signal is not None and type(captured_signal) is not int:
        raise ValueError("invalid capture signal")
    for name in ("seconds", "user_cpu_seconds", "system_cpu_seconds"):
        if not _number(observation.get(name)):
            raise ValueError("invalid capture metric")
    if type(observation.get("maxrss_kb")) is not int:
        raise ValueError("invalid capture RSS")
    if require_complete and (
            observation["exit_status"] not in (0, 1)
            or observation["timed_out"] is not False
            or observation["termination_uncertain"] is not False
            or observation["signal"] not in (None, 0)
            or any(not _number(observation[name], nonnegative=True)
                   for name in ("seconds", "user_cpu_seconds",
                                "system_cpu_seconds"))
            or not _number(observation["maxrss_kb"], integral=True,
                           nonnegative=True)
            or observation["maxrss_approximate"] is not False):
        raise ValueError("capture observation is incomplete")
    return observation


def validate_invocation_records(records, result_dir=None):
    if type(records) is not list or len(records) != len(EXPECTED_INVOCATIONS):
        raise ValueError("exactly four invocation records are required")
    identifiers = set()
    pids = set()
    process_groups = set()
    for sequence, (record, expected) in enumerate(zip(records, EXPECTED_INVOCATIONS)):
        if type(record) is not dict or frozenset(record) != INVOCATION_KEYS:
            raise ValueError("capture invocation schema changed")
        case_id, engine = expected
        pid, process_group = record.get("pid"), record.get("process_group_id")
        invocation_id = record.get("invocation_id")
        if (type(record.get("sequence")) is not int
                or record["sequence"] != sequence
                or record.get("case_id") != case_id
                or record.get("engine") != engine
                or type(pid) is not int or pid <= 0
                or type(process_group) is not int or process_group <= 0
                or pid != process_group
                or record.get("fresh_process") is not True
                or not isinstance(invocation_id, str)
                or re.fullmatch(INVOCATION_ID_PATTERN, invocation_id) is None
                or invocation_id in identifiers or pid in pids
                or process_group in process_groups):
            raise ValueError("capture invocation is not one unique fresh process")
        identifiers.add(invocation_id)
        pids.add(pid)
        process_groups.add(process_group)
        stem = "capture-%02d-%s-%s" % (sequence, case_id, engine)
        expected_artifacts = {
            "input": stem + ".input.atlas",
            "stdout": stem + ".stdout",
            "stderr": stem + ".stderr",
            "time": stem + ".time",
        }
        if any(not isinstance(record.get(key), dict)
               or record[key].get("path") != name
               for key, name in expected_artifacts.items()):
            raise ValueError("capture artifact name changed")
        _validate_observation(record["observation"], engine, require_complete=True)
        if (not isinstance(record.get("executable_sha256"), str)
                or re.fullmatch(SHA256_PATTERN, record["executable_sha256"]) is None
                or record["executable_sha256"] == "0" * 64
                or type(record.get("executable_bytes")) is not int
                or record["executable_bytes"] <= 0
                or (engine == "oracle"
                    and record["executable_sha256"]
                    != ACCEPTED_SOURCE["oracle"]["binary_sha256"])):
            raise ValueError("capture executable identity is invalid")
        for key in ("input", "stdout", "stderr", "time"):
            _validate_artifact(record[key], result_dir)
        if result_dir is not None:
            raw_input = _validate_artifact(record["input"], result_dir)
            expected_case = next(
                case for case in CATALOG_CASES if case["id"] == case_id
            )
            if (len(raw_input) != expected_case["input_bytes"]
                    or _sha(raw_input) != expected_case["input_sha256"]):
                raise ValueError("capture input artifact changed")
            raw_time = _validate_artifact(record["time"], result_dir)
            if not _time_matches_record(raw_time, record["observation"]):
                raise ValueError("capture observation differs from GNU time")
    return records


def _validate_command_records(records, result_dir):
    if type(records) is not list or len(records) != len(COMMAND_NAMES):
        raise ValueError("exactly thirteen command records are required")
    for expected_name, record in zip(COMMAND_NAMES, records):
        expected_command = COMMAND_CONTRACTS[expected_name]
        if (type(record) is not dict or frozenset(record) != COMMAND_KEYS
                or record.get("name") != expected_name
                or record.get("argv") != expected_command["argv"]
                or record.get("cwd") != expected_command["cwd"]
                or type(record.get("exit_status")) is not int
                or record["exit_status"] not in EXPECTED_COMMAND_EXITS[
                    expected_name]
                or record.get("timed_out") is not False
                or record.get("termination_uncertain") is not False
                or record.get("signal") not in (None, 0)
                or any(not _number(record.get(key), nonnegative=True)
                       for key in ("seconds", "user_cpu_seconds",
                                   "system_cpu_seconds"))
                or not _number(record.get("maxrss_kb"), integral=True,
                               nonnegative=True)
                or record.get("maxrss_approximate")
                   is not (expected_name == "source-reconstruction")):
            raise ValueError("capture command record is incomplete")
        if any(not isinstance(record.get(key), dict)
               or record[key].get("path") != expected_name + "." + key
               for key in ("stdout", "stderr", "time")):
            raise ValueError("capture command artifact name changed")
        stdout = _validate_artifact(record["stdout"], result_dir)
        stderr = _validate_artifact(record["stderr"], result_dir)
        raw_time = _validate_artifact(record["time"], result_dir)
        if not _time_matches_record(raw_time, record):
            raise ValueError("command record differs from GNU time")
        combined = stdout + stderr
        if expected_name in dict(CHECK_COMMANDS):
            count = EXPECTED_TEST_COUNTS[expected_name]
            if re.search(
                    rb"Ran " + str(count).encode()
                    + rb" tests in [0-9.]+s\s+OK\s*$", combined) is None:
                raise ValueError("checker output is not a complete passing run")
        elif expected_name in ("rustc-version", "cargo-version"):
            if not combined.strip():
                raise ValueError("toolchain identity output is empty")
        elif expected_name == "source-reconstruction":
            expected = (
                "SOURCE_RECONSTRUCTION_OK %d %s\n"
                % (AFTER_SOURCE_FILES,
                   AFTER_SOURCE_MANIFEST_SHA256)
            ).encode()
            if combined != expected:
                raise ValueError("source reconstruction evidence changed")
    return records


def _capture_runs(catalog, records, result_dir):
    by_case = {case["id"]: [] for case in catalog["cases"]}
    for record in records:
        by_case[record["case_id"]].append({
            "engine": record["engine"],
            "observation": record["observation"],
            "stdout": _validate_artifact(record["stdout"], result_dir),
            "stderr": _validate_artifact(record["stderr"], result_dir),
            "fresh_process": record["fresh_process"],
            "invocation_id": record["invocation_id"],
        })
    return [
        classify_capture(case, by_case[case["id"]])
        for case in catalog["cases"]
    ]


def regression_source_manifest(accepted_manifest):
    if (type(accepted_manifest) is not dict
            or len(accepted_manifest) != ACCEPTED_SOURCE["final_files"]
            or _sha(json.dumps(
                accepted_manifest, sort_keys=True, separators=(",", ":"),
            ).encode()) != ACCEPTED_SOURCE["source_manifest_sha256"]):
        raise ValueError("accepted source manifest changed before regression patch")
    manifest = copy.deepcopy(accepted_manifest)
    manifest.update(REGRESSION_SOURCE_HASHES)
    manifest.update(REGRESSION_FIXTURE_HASHES)
    canonical = json.dumps(
        manifest, sort_keys=True, separators=(",", ":"),
    ).encode()
    if (len(manifest) != REGRESSION_SOURCE["files"]
            or _sha(canonical) != REGRESSION_SOURCE["source_manifest_sha256"]):
        raise ValueError("tests-first regression source manifest changed")
    return manifest


def repaired_source_manifest(regression_manifest):
    """Derive the repaired source manifest from the tests-first manifest."""
    if (type(regression_manifest) is not dict
            or len(regression_manifest) != REGRESSION_SOURCE["files"]
            or _sha(json.dumps(
                regression_manifest, sort_keys=True, separators=(",", ":"),
            ).encode()) != REGRESSION_SOURCE["source_manifest_sha256"]):
        raise ValueError("tests-first source manifest changed before repair patch")
    manifest = copy.deepcopy(regression_manifest)
    manifest.update(REPAIRED_SOURCE_HASHES)
    canonical = json.dumps(
        manifest, sort_keys=True, separators=(",", ":"),
    ).encode()
    if (len(manifest) != AFTER_SOURCE_FILES
            or _sha(canonical) != AFTER_SOURCE_MANIFEST_SHA256):
        raise ValueError("repaired source manifest changed")
    return manifest


def _regression_inputs(root):
    root = Path(root)
    catalog_raw = _stable_relative_bytes(root, REGRESSION_CATALOG_PATH)
    inspection_raw = _stable_relative_bytes(root, REGRESSION_INSPECTION_PATH)
    catalog = decode_regression_catalog(catalog_raw)
    decode_regression_inspection(inspection_raw, catalog)
    artifacts = {
        PurePosixPath(name).name: _stable_relative_bytes(root, name)
        for name in sorted(REGRESSION_FIXTURE_HASHES)
    }
    if len(artifacts) != len(REGRESSION_FIXTURE_HASHES):
        raise ValueError("regression artifact basenames are not unique")
    validate_regression_artifacts(catalog, artifacts)
    return catalog_raw, inspection_raw, catalog, artifacts


def _publish_regression_inputs(out, catalog_raw, inspection_raw, artifacts):
    catalog_reference = _publish_artifact(
        out, REGRESSION_CATALOG_RECORD["path"], catalog_raw,
    )
    inspection_reference = _publish_artifact(
        out, REGRESSION_INSPECTION_RECORD["path"], inspection_raw,
    )
    if (catalog_reference != REGRESSION_CATALOG_RECORD
            or inspection_reference != REGRESSION_INSPECTION_RECORD):
        raise ValueError("published regression provenance changed")
    references = {
        name: _publish_artifact(out, "regression-" + name, raw)
        for name, raw in sorted(artifacts.items())
    }
    return catalog_reference, inspection_reference, references


def _after_metrics(record):
    return {
        "format": "gnu-time-v",
        "seconds": record["seconds"],
        "user_cpu_seconds": record["user_cpu_seconds"],
        "system_cpu_seconds": record["system_cpu_seconds"],
        "maxrss_kb": record["maxrss_kb"],
        "maxrss_approximate": record["maxrss_approximate"],
        "timed_out": record["timed_out"],
        "termination_uncertain": record["termination_uncertain"],
        "signal": record["signal"],
    }


def _after_observation(report, result_dir, regression_catalog):
    invocations = report.get("invocations")
    commands = report.get("commands")
    if type(invocations) is not list or type(commands) is not list:
        raise ValueError("BEFORE evidence records are absent")
    original_reruns = []
    invocation_ids = []
    for case in regression_catalog["cases"]:
        selected = [
            record for record in invocations
            if record.get("case_id") == case["id"]
            and record.get("engine") == "oracle"
        ]
        if len(selected) != 1:
            raise ValueError("each regression needs one exact oracle rerun")
        record = selected[0]
        invocation_ids.append(record["invocation_id"])
        original_reruns.append({
            "case_id": case["id"],
            "engine": "oracle",
            "exit_status": record["observation"]["exit_status"],
            "stdout": _validate_artifact(record["stdout"], result_dir),
            "stderr": _validate_artifact(record["stderr"], result_dir),
            "fresh_process": record["fresh_process"],
            "timed_out": record["observation"]["timed_out"],
            "termination_uncertain":
                record["observation"]["termination_uncertain"],
            "metrics": _after_metrics(record["observation"]),
        })
    if len(set(invocation_ids)) != 2:
        raise ValueError("oracle rerun invocation identities are not unique")

    command_map = {record.get("name"): record for record in commands}
    if len(command_map) != len(commands):
        raise ValueError("command records contain duplicate names")

    def command_observation(name):
        record = command_map.get(name)
        if not isinstance(record, dict):
            raise ValueError("BEFORE command record is absent: " + name)
        return {
            "name": name,
            "exit_status": record["exit_status"],
            "log": (
                _validate_artifact(record["stdout"], result_dir)
                + _validate_artifact(record["stderr"], result_dir)
            ),
            "metrics": _after_metrics(record),
        }

    claims = {key: False for key in CLAIM_KEYS}
    observation = {
        "schema": AFTER_OBSERVATION_SCHEMA,
        "provenance": expected_regression_provenance(),
        "original_reruns": original_reruns,
        "selector_command": command_observation("weyl-context-regressions"),
        "retained_control_command": command_observation(
            "root-ladder-control"),
        "inventory_command": command_observation("atlas-core-test-inventory"),
        "claims": claims,
    }
    return observation, invocation_ids


def _regression_classification(report, result_dir):
    regression = report.get("regression")
    if type(regression) is not dict or frozenset(regression) != REGRESSION_KEYS:
        raise ValueError("regression report closure changed")
    if (regression.get("catalog") != REGRESSION_CATALOG_RECORD
            or regression.get("inspection") != REGRESSION_INSPECTION_RECORD):
        raise ValueError("regression catalog or inspection reference changed")
    catalog_raw = _validate_artifact(regression["catalog"], result_dir)
    inspection_raw = _validate_artifact(regression["inspection"], result_dir)
    catalog = decode_regression_catalog(catalog_raw)
    decode_regression_inspection(inspection_raw, catalog)
    references = regression.get("artifacts")
    expected_names = {
        PurePosixPath(name).name for name in REGRESSION_FIXTURE_HASHES
    }
    if (type(references) is not dict or set(references) != expected_names
            or any(
                type(references[name]) is not dict
                or references[name].get("path") != "regression-" + name
                for name in expected_names
            )):
        raise ValueError("regression artifact references changed")
    artifacts = {
        name: _validate_artifact(reference, result_dir)
        for name, reference in references.items()
    }
    validate_regression_artifacts(catalog, artifacts)
    observation, invocation_ids = _after_observation(
        report, result_dir, catalog,
    )
    classification = classify_after(
        catalog_raw, inspection_raw, artifacts, observation,
    )
    if (regression.get("original_rerun_invocations") != invocation_ids
            or regression.get("selector_command") != "weyl-context-regressions"
            or regression.get("retained_control_command")
               != "root-ladder-control"
            or regression.get("inventory_command")
               != "atlas-core-test-inventory"):
        raise ValueError("regression evidence bindings changed")
    return classification


def validate_report(report, result_dir):
    if type(report) is not dict or frozenset(report) != REPORT_KEYS:
        raise ValueError("successful capture report schema changed")
    _forbid_claim_keys(report)
    if (report.get("schema") != REPORT_SCHEMA
            or not isinstance(report.get("job"), str)
            or not report["job"].isdecimal()
            or not isinstance(report.get("node"), str) or not report["node"]
            or report.get("status") != SUCCESS_STATUS
            or report.get("evidence_maturity") != "tests_first_after"
            or report.get("scope") != REPORT_SCOPE
            or report.get("accepted_source")
               != report.get("pin", {}).get("accepted_source")
            or report.get("complete") is not True
            or report.get("acceptance_eligible") is not False
            or report.get("math_gate_released") is not False
            or report.get("cache_gate_released") is not False
            or report.get("performance_gate_released") is not False
            or report.get("rank_gate_released") is not False
            or report.get("integrity_rechecked") is not True
            or report.get("source_integrity_rechecked") is not True
            or report.get("ephemeral_workspace_removed") is not True):
        raise ValueError("capture report makes an invalid success claim")
    validate_stage_pin(report["pin"])
    if report["accepted_source"] != ACCEPTED_SOURCE:
        raise ValueError("accepted source summary changed")
    provenance = report.get("provenance")
    if (not isinstance(provenance, dict)
            or set(provenance) != PROVENANCE_KEYS
            or provenance.get("pin_sha256") != saved_json_sha(report["pin"])
            or provenance.get("harness_inputs") != report["pin"]["inputs"]
            or not isinstance(provenance.get("campaign_record"), dict)
            or provenance["campaign_record"].get("job") != report["job"]
            or provenance["campaign_record"].get("stage") != EXPECTED_STAGE
            or provenance.get("submission_receipt")
               != submission_receipt(
                   provenance["campaign_record"], report["pin"],
                   root=EXPECTED_STAGE)):
        raise ValueError("capture provenance closure changed")
    if report.get("catalog") != CATALOG_RECORD:
        raise ValueError("capture catalog reference changed")
    source = report.get("source")
    if (type(source) is not dict
            or set(source) != {"files", "manifest_sha256", "manifest"}
            or source.get("files") != AFTER_SOURCE_FILES
            or source.get("manifest_sha256")
               != AFTER_SOURCE_MANIFEST_SHA256
            or not isinstance(source.get("manifest"), dict)
            or len(source["manifest"]) != source["files"]
            or _sha(json.dumps(
                source["manifest"], sort_keys=True, separators=(",", ":"),
            ).encode()) != source["manifest_sha256"]):
        raise ValueError("reconstructed source evidence changed")
    binaries = report.get("binaries")
    rust_binary = binaries.get("rust") if isinstance(binaries, dict) else None
    oracle_binary = binaries.get("oracle") if isinstance(binaries, dict) else None
    if (not isinstance(binaries, dict) or set(binaries) != {"rust", "oracle"}
            or not isinstance(rust_binary, dict)
            or set(rust_binary) != {"sha256", "bytes"}
            or not isinstance(rust_binary.get("sha256"), str)
            or re.fullmatch(SHA256_PATTERN, rust_binary["sha256"]) is None
            or type(rust_binary.get("bytes")) is not int
            or rust_binary["bytes"] <= 0
            or not isinstance(oracle_binary, dict)
            or set(oracle_binary) != {"sha256", "bytes", "commit"}
            or oracle_binary.get("sha256")
               != ACCEPTED_SOURCE["oracle"]["binary_sha256"]
            or oracle_binary.get("commit")
               != ACCEPTED_SOURCE["oracle"]["commit"]
            or type(oracle_binary.get("bytes")) is not int
            or oracle_binary["bytes"] <= 0
            or report.get("limitations") != LIMITATIONS
            or report.get("legacy_path_open_attempts") != []
            or report.get("thread_settings") != THREAD_SETTINGS):
        raise ValueError("capture execution identity or limitations changed")
    validate_environment_record(report.get("environment"))
    catalog_raw = _validate_artifact(report["catalog"]["artifact"], result_dir)
    catalog = decode_discovery_catalog(catalog_raw)
    _validate_command_records(report["commands"], result_dir)
    validate_invocation_records(report["invocations"], result_dir)
    regression = report.get("regression")
    if type(regression) is not dict or frozenset(regression) != REGRESSION_KEYS:
        raise ValueError("regression report closure changed")
    references = [
        report["catalog"]["artifact"], regression["catalog"],
        regression["inspection"],
    ]
    if type(regression.get("artifacts")) is not dict:
        raise ValueError("regression artifact map changed")
    references.extend(regression["artifacts"].values())
    references.extend(
        record[key]
        for record in report["commands"]
        for key in ("stdout", "stderr", "time")
    )
    references.extend(
        record[key]
        for record in report["invocations"]
        for key in ("input", "stdout", "stderr", "time")
    )
    artifact_paths = [reference["path"] for reference in references]
    expected_artifacts = (
        3 + len(REGRESSION_FIXTURE_HASHES)
        + 3 * len(COMMAND_NAMES) + 4 * len(EXPECTED_INVOCATIONS))
    if (len(artifact_paths) != expected_artifacts
            or len(set(artifact_paths)) != len(artifact_paths)
            or any(len(PurePosixPath(name).parts) != 1
                   for name in artifact_paths)):
        raise ValueError("capture raw artifacts are aliased or nested")
    for record in report["invocations"]:
        expected_binary = binaries[record["engine"]]
        if (record["executable_sha256"] != expected_binary["sha256"]
                or record["executable_bytes"] != expected_binary["bytes"]):
            raise ValueError("capture invocation names another executable")
    captures = _capture_runs(catalog, report["invocations"], result_dir)
    if (report.get("captures") != captures
            or len(captures) != 2
            or any(capture.get("status") not in COMPLETE_CAPTURE_STATUSES
                   or capture.get("evidence_maturity") != CAPTURE_MATURITY
                   or capture.get("acceptance_eligible") is not False
                   or capture.get("math_gate_released") is not False
                   or capture.get("cache_gate_released") is not False
                   or capture.get("fresh_process_complete") is not True
                   or capture.get("resource_complete") is not True
                   or capture.get("metrics_complete") is not True
                   or capture.get("stream_complete") is not True
                   # The AFTER gate: the repaired Rust must match every
                   # frozen original golden byte-for-byte.
                   or capture.get("full_stdout_equal") is not True
                   or capture.get("full_stderr_equal") is not True
                   or capture.get("exit_status_equal") is not True
                   for capture in captures)):
        raise ValueError("capture classifier result is incomplete or changed")
    classification = _regression_classification(report, result_dir)
    if (classification != regression.get("classification")
            or classification.get("status") != AFTER_REPRODUCED
            or classification.get("evidence_maturity") != "tests_first_after"
            or any(classification.get(key) is not False
                   for key in CLAIM_KEYS)):
        raise ValueError("tests-first AFTER classification changed")
    return report


def gates(root):
    root = Path(root).resolve()
    validate_stage_topology(root)
    campaign = campaign_stage(root)
    if root.name != STAGE_NAME:
        raise ValueError("unexpected Weyl core capture stage")
    pin_sha = os.environ.get("WEYL_CONTEXT_CORE_CAPTURE_PIN_SHA256")
    spool = os.environ.get("WEYL_CONTEXT_CORE_CAPTURE_SPOOL")
    job = os.environ.get("SLURM_JOB_ID")
    if (not isinstance(pin_sha, str) or re.fullmatch(SHA256_PATTERN, pin_sha) is None
            or not isinstance(spool, str) or not spool
            or not isinstance(job, str) or not job.isdecimal()):
        raise ValueError("missing or invalid Weyl core capture environment")
    with existing_lock(root, STAGE_LOCK):
        inputs = frozen_stage_inputs(root)
        pin = validate_pin(read_json_file(root / PIN_NAME, pin_sha), inputs)
        if digest(spool) != inputs[SBATCH]:
            raise ValueError("submitted Weyl core capture script changed")
        record = confirmed_record(
            root, pin, repair_intent=True, expected_job=job,
        )
        validate_stage_creation(
            root, record["stage_creation_sha256"], pin_sha256=pin_sha)
        receipt = read_json_file(root / "submission.json")
        if receipt != submission_receipt(record, pin, root=root):
            raise ValueError("Weyl core capture submission receipt changed")
    accepted_source_manifest = validate_predecessor(root, inputs)
    validate_prior_creation_failure(root, inputs)
    validate_capture_v1_failure(root, inputs)
    validate_capture_v2_failure(root, inputs)
    validate_capture_v3_failure(root, inputs)
    validate_capture_v4_failure(root, inputs)
    validate_capture_v5_failure(root, inputs)
    validate_capture_v6_failure(root, inputs)
    validate_capture_v7_failure(root, inputs)
    validate_capture_v8(root, inputs)
    validate_before_v1_failure(root, inputs)
    validate_before_v2_failure(root, inputs)
    validate_before_v3_failure(root, inputs)
    validate_before_v4_result(root, inputs)
    validate_after_v1_failure(root, inputs)
    validate_after_v2_failure(root, inputs)
    validate_after_v3_failure(root, inputs)
    validate_after_v4_failure(root, inputs)
    catalog = validate_staged_catalog(root, inputs)
    regression_source_manifest = validate_regression_inputs(
        root, inputs, accepted_source_manifest,
    )
    repaired_manifest = repaired_source_manifest(regression_source_manifest)
    if validate_parent_objects(campaign) != PARENT_SOURCE_OBJECT:
        raise ValueError("Weyl core parent source object changed")
    seal = load_parent_seal(campaign, PARENT_SEAL_REFERENCE)
    read_blob(campaign, RETIRED_STAGER_BUNDLE_REFERENCE)
    if (seal["source_object"] != PARENT_SOURCE_OBJECT
            or seal["source_manifest"] is None
            or len(seal["source_manifest"]) != ACCEPTED_SOURCE["parent_files"]):
        raise ValueError("Weyl core parent seal changed")
    return {
        "pin": pin,
        "pin_sha256": pin_sha,
        "inputs": inputs,
        "record": record,
        "receipt": receipt,
        "accepted_source_manifest": accepted_source_manifest,
        "regression_source_manifest": regression_source_manifest,
        "repaired_source_manifest": repaired_manifest,
        "catalog": catalog,
        "seal": seal,
    }


def _parse_header_path(line, prefix):
    if not line.startswith(prefix):
        raise ValueError("invalid unified patch header")
    raw = line[len(prefix):].rstrip(b"\r\n").split(b"\t", 1)[0]
    try:
        value = raw.decode("utf-8", errors="strict")
    except UnicodeDecodeError as error:
        raise ValueError("non-UTF-8 unified patch path") from error
    if not value.startswith(("a/", "b/")):
        raise ValueError("unified patch path lacks a side prefix")
    name = value[2:]
    _safe_relative(name)
    return name


def _apply_unified_patch(source, patch_path):
    lines = Path(patch_path).read_bytes().splitlines(keepends=True)
    position = 0
    changed = []
    header = (
        rb"^@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@(?: .*)?(?:\r?\n)?$"
    )
    while position < len(lines):
        if not lines[position].startswith(b"--- "):
            raise ValueError("unified patch contains unexpected leading data")
        old_name = _parse_header_path(lines[position], b"--- ")
        position += 1
        if position >= len(lines):
            raise ValueError("unified patch lacks a new-file header")
        new_name = _parse_header_path(lines[position], b"+++ ")
        position += 1
        if old_name != new_name or new_name in changed:
            raise ValueError("unified patch changes an unexpected file")
        path = Path(source) / new_name
        original = path.read_bytes().splitlines(keepends=True)
        output = []
        cursor = 0
        hunks = 0
        while position < len(lines) and lines[position].startswith(b"@@ "):
            match = re.fullmatch(header, lines[position])
            if match is None:
                raise ValueError("invalid unified patch hunk header")
            old_start = int(match.group(1))
            old_count = int(match.group(2) or b"1")
            new_count = int(match.group(4) or b"1")
            position += 1
            start = old_start - 1
            if start < cursor or start > len(original):
                raise ValueError("unified patch hunk is out of range")
            output.extend(original[cursor:start])
            cursor = start
            consumed_old = produced_new = 0
            while (position < len(lines)
                   and not lines[position].startswith((b"@@ ", b"--- "))):
                line = lines[position]
                position += 1
                if line.startswith(b"\\ No newline at end of file"):
                    raise ValueError("unsupported no-newline unified patch")
                if not line or line[:1] not in (b" ", b"+", b"-"):
                    raise ValueError("invalid unified patch hunk line")
                marker, contents = line[:1], line[1:]
                if marker in (b" ", b"-"):
                    if cursor >= len(original) or original[cursor] != contents:
                        raise ValueError("unified patch context changed")
                    cursor += 1
                    consumed_old += 1
                if marker in (b" ", b"+"):
                    output.append(contents)
                    produced_new += 1
            if consumed_old != old_count or produced_new != new_count:
                raise ValueError("unified patch hunk count changed")
            hunks += 1
        if not hunks:
            raise ValueError("unified patch file lacks hunks")
        output.extend(original[cursor:])
        path.write_bytes(b"".join(output))
        changed.append(new_name)
    if not changed:
        raise ValueError("empty unified patch")
    return changed


def _sealed_boundary_fixtures(campaign, seal):
    raw = boundary_bytes(campaign, seal)
    case_id = "generic_root_ladder_coordinate_boundary"
    begin = ("prints(\"MATH_BEGIN " + case_id + "\")\n").encode()
    finish = ("\nprints(\"MATH_END " + case_id + "\")\nquit\n").encode()
    stdout_begin = ("MATH_BEGIN " + case_id + "\n").encode()
    stdout_finish = ("MATH_END " + case_id + "\nBye.\n").encode()
    if (not raw["input"].startswith(begin) or not raw["input"].endswith(finish)
            or not raw["oracle_stdout"].startswith(stdout_begin)
            or not raw["oracle_stdout"].endswith(stdout_finish)
            or raw["oracle_stderr"] != b""):
        raise ValueError("sealed ladder fixture frame changed")
    fixtures = {
        BOUNDARY_FIXTURE: raw["input"][len(begin):-len(finish)],
        BOUNDARY_STDOUT: raw["oracle_stdout"],
        BOUNDARY_STDERR: raw["oracle_stderr"],
    }
    if any(_sha(fixtures[name]) != SEALED_FIXTURE_HASHES[name]
           for name in fixtures):
        raise ValueError("sealed ladder fixture bytes changed")
    return fixtures


def _group_alive(process_group):
    try:
        os.killpg(process_group, 0)
    except ProcessLookupError:
        return False
    except PermissionError:
        return True
    return True


def _wait_group_exit(process_group, timeout_seconds):
    if (type(process_group) is not int or process_group <= 0
            or isinstance(timeout_seconds, bool)
            or not isinstance(timeout_seconds, (int, float))
            or not math.isfinite(timeout_seconds) or timeout_seconds <= 0):
        raise ValueError("invalid process-group exit wait")
    deadline = time.monotonic() + timeout_seconds
    while _group_alive(process_group):
        remaining = deadline - time.monotonic()
        if remaining <= 0:
            return False
        time.sleep(min(0.05, remaining))
    return True


def _terminate_group(process, process_group):
    if process is None:
        return False
    uncertain = False
    if process_group is None:
        process_group = process.pid
    if process.poll() is None or _group_alive(process_group):
        try:
            os.killpg(process_group, signal.SIGTERM)
        except ProcessLookupError:
            pass
        try:
            process.wait(timeout=COMMAND_KILL_AFTER_SECONDS)
        except subprocess.TimeoutExpired:
            uncertain = True
            try:
                os.killpg(process_group, signal.SIGKILL)
            except ProcessLookupError:
                pass
            process.wait(timeout=COMMAND_KILL_AFTER_SECONDS)
    else:
        process.wait()
    if _group_alive(process_group):
        uncertain = True
        try:
            os.killpg(process_group, signal.SIGKILL)
        except ProcessLookupError:
            pass
        if not _wait_group_exit(process_group, COMMAND_KILL_AFTER_SECONDS):
            uncertain = True
    return uncertain


def _execute_timed(argv, cwd, env, input_bytes, timeout_seconds, metrics_path,
                   active, pass_fds=()):
    if (type(timeout_seconds) is not int or timeout_seconds < 1
            or type(input_bytes) is not bytes
            or type(pass_fds) is not tuple
            or any(type(value) is not int or value < 0 for value in pass_fds)
            or len(set(pass_fds)) != len(pass_fds)
            or type(active) is not dict
            or set(active) != {"process", "process_group"}
            or active["process"] is not None
            or active["process_group"] is not None):
        raise ValueError("invalid timed-command request")
    started = time.monotonic()
    process = None
    process_group = None
    timed_out = False
    termination_uncertain = False
    sent_signal = None
    group_alive_after_communicate = False
    natural_exit_grace_attempted = False
    natural_exit_grace_succeeded = False
    cleanup_attempted = False
    cleanup_uncertain = False
    group_alive_after_cleanup = None
    try:
        process = subprocess.Popen(
            ["/usr/bin/time", "-v", "-o", str(metrics_path), *argv],
            cwd=cwd,
            env=env,
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            start_new_session=True,
            close_fds=True,
            pass_fds=pass_fds,
        )
        process_group = os.getpgid(process.pid)
        active.update(process=process, process_group=process_group)
        try:
            stdout, stderr = process.communicate(
                input=input_bytes, timeout=timeout_seconds,
            )
        except subprocess.TimeoutExpired:
            timed_out = True
            sent_signal = signal.SIGTERM
            try:
                os.killpg(process_group, signal.SIGTERM)
            except ProcessLookupError:
                pass
            try:
                stdout, stderr = process.communicate(
                    timeout=COMMAND_KILL_AFTER_SECONDS,
                )
            except subprocess.TimeoutExpired:
                sent_signal = signal.SIGKILL
                termination_uncertain = True
                try:
                    os.killpg(process_group, signal.SIGKILL)
                except ProcessLookupError:
                    pass
                stdout, stderr = process.communicate(
                    timeout=COMMAND_KILL_AFTER_SECONDS,
                )
        returncode = process.returncode
        group_alive_after_communicate = _group_alive(process_group)
        if group_alive_after_communicate:
            if not timed_out and returncode == 0:
                natural_exit_grace_attempted = True
                natural_exit_grace_succeeded = _wait_group_exit(
                    process_group, COMMAND_EXIT_GRACE_SECONDS,
                )
            if (timed_out or returncode != 0
                    or not natural_exit_grace_succeeded):
                termination_uncertain = True
                cleanup_attempted = True
                cleanup_uncertain = _terminate_group(process, process_group)
                termination_uncertain = (
                    cleanup_uncertain or termination_uncertain
                )
    except BaseException:
        termination_uncertain = (
            _terminate_group(process, process_group) or termination_uncertain
        )
        if process_group is None or not _group_alive(process_group):
            active.update(process=None, process_group=None)
        raise
    else:
        group_alive_after_cleanup = (
            process_group is not None and _group_alive(process_group)
        )
        if group_alive_after_cleanup:
            raise RuntimeError("timed command process group remains alive")
        active.update(process=None, process_group=None)
    try:
        metrics = Path(metrics_path).read_bytes()
    except FileNotFoundError:
        metrics = b""
    parsed_metrics = _parse_time(metrics)
    measured_seconds = time.monotonic() - started
    if parsed_metrics is None:
        parsed_metrics = {
            "seconds": -1.0,
            "user_cpu_seconds": -1.0,
            "system_cpu_seconds": -1.0,
            "maxrss_kb": -1,
            "exit_status": -1,
            "signal": None,
        }
    derived_return_signal = (
        -returncode if returncode is not None and returncode < 0 else None
    )
    captured_signal = (
        sent_signal if sent_signal is not None else derived_return_signal
    )
    return {
        "pid": process.pid,
        "process_group_id": process_group,
        "exit_status": returncode,
        "timed_out": timed_out,
        "termination_uncertain": termination_uncertain,
        "signal": captured_signal,
        "sent_signal": sent_signal,
        "derived_return_signal": derived_return_signal,
        "group_alive_after_communicate": group_alive_after_communicate,
        "natural_exit_grace_attempted": natural_exit_grace_attempted,
        "natural_exit_grace_succeeded": natural_exit_grace_succeeded,
        "cleanup_attempted": cleanup_attempted,
        "cleanup_uncertain": cleanup_uncertain,
        "group_alive_after_cleanup": group_alive_after_cleanup,
        "seconds": parsed_metrics["seconds"],
        "measured_seconds": measured_seconds,
        "user_cpu_seconds": parsed_metrics["user_cpu_seconds"],
        "system_cpu_seconds": parsed_metrics["system_cpu_seconds"],
        "maxrss_kb": parsed_metrics["maxrss_kb"],
        "maxrss_approximate": False,
        "stdout_raw": stdout,
        "stderr_raw": stderr,
        "time_raw": metrics,
    }


def _command_working_directory(label, root, work):
    root = Path(root)
    work = Path(work)
    if (not root.is_absolute() or not work.is_absolute()
            or Path(os.path.abspath(root)) != root
            or Path(os.path.abspath(work)) != work
            or root == work):
        raise ValueError("command roots are not one bounded stage workspace")
    locations = {
        ".": root,
        "workspace": work,
        "workspace/source": work / "source",
    }
    if label not in locations:
        raise ValueError("command contract has an unknown working directory")
    selected = locations[label]
    try:
        anchor_statuses = (os.lstat(root), os.lstat(work))
        selected_status = os.lstat(selected)
    except OSError as error:
        raise ValueError("command working directory is unavailable") from error
    if (any(not stat.S_ISDIR(value.st_mode) for value in anchor_statuses)
            or not stat.S_ISDIR(selected_status.st_mode)):
        raise ValueError("command working directory is indirect or not a directory")
    return selected


def _command_failed_checks(name, record):
    checks = []
    if (name not in EXPECTED_COMMAND_EXITS
            or type(record.get("exit_status")) is not int
            or record["exit_status"] not in EXPECTED_COMMAND_EXITS[name]):
        checks.append("exit_status")
    if record.get("timed_out") is not False:
        checks.append("timed_out")
    if record.get("termination_uncertain") is not False:
        checks.append("termination_uncertain")
    if record.get("signal") not in (None, 0):
        checks.append("signal")
    for key in ("seconds", "user_cpu_seconds", "system_cpu_seconds"):
        if not _number(record.get(key), nonnegative=True):
            checks.append(key)
    if not _number(record.get("maxrss_kb"), integral=True, nonnegative=True):
        checks.append("maxrss_kb")
    return checks


class CommandRecordFailure(ValueError):
    def __init__(self, name, record, result, failed_checks):
        self.failed_checks = list(failed_checks)
        diagnostic_keys = (
            "pid", "process_group_id", "exit_status", "timed_out",
            "termination_uncertain", "signal", "sent_signal",
            "derived_return_signal", "group_alive_after_communicate",
            "natural_exit_grace_attempted", "natural_exit_grace_succeeded",
            "cleanup_attempted", "cleanup_uncertain",
            "group_alive_after_cleanup", "measured_seconds",
        )
        self.evidence = {
            "name": name,
            "failed_checks": list(failed_checks),
            "record": copy.deepcopy(record),
            "execution": {
                key: result.get(key) for key in diagnostic_keys
            },
        }
        super().__init__(
            "command failed or lacks exact GNU time metrics: " + name
            + " failed_checks=" + json.dumps(self.failed_checks)
        )


def _command_record(out, name, root, work, env, metrics_path, active):
    if name not in COMMAND_NAMES or name not in COMMAND_CONTRACTS:
        raise ValueError("unknown capture command")
    contract = COMMAND_CONTRACTS[name]
    if (type(contract) is not dict or set(contract) != {"argv", "cwd"}
            or type(contract["argv"]) is not list
            or any(not isinstance(value, str) or not value
                   for value in contract["argv"])
            or not isinstance(contract["cwd"], str)):
        raise ValueError("capture command contract changed")
    argv = list(contract["argv"])
    cwd_label = contract["cwd"]
    cwd = _command_working_directory(cwd_label, root, work)
    result = _execute_timed(
        argv, cwd, env, b"", COMMAND_TIMEOUT_SECONDS, metrics_path, active,
    )
    record = {
        "name": name,
        "argv": list(argv),
        "cwd": cwd_label,
        "exit_status": result["exit_status"],
        "timed_out": result["timed_out"],
        "termination_uncertain": result["termination_uncertain"],
        "signal": result["signal"],
        "seconds": result["seconds"],
        "user_cpu_seconds": result["user_cpu_seconds"],
        "system_cpu_seconds": result["system_cpu_seconds"],
        "maxrss_kb": result["maxrss_kb"],
        "maxrss_approximate": result["maxrss_approximate"],
        "stdout": _publish_artifact(out, name + ".stdout", result["stdout_raw"]),
        "stderr": _publish_artifact(out, name + ".stderr", result["stderr_raw"]),
        "time": _publish_artifact(out, name + ".time", result["time_raw"]),
    }
    failed_checks = _command_failed_checks(name, record)
    if failed_checks:
        raise CommandRecordFailure(name, record, result, failed_checks)
    return record, result["stdout_raw"] + result["stderr_raw"]


def _stat_identity(value):
    return (
        value.st_dev, value.st_ino, value.st_size, value.st_mtime_ns,
        value.st_ctime_ns, stat.S_IFMT(value.st_mode),
        stat.S_IMODE(value.st_mode), value.st_nlink, value.st_uid, value.st_gid,
    )


def _descriptor_snapshot(descriptor):
    try:
        os.lseek(descriptor, 0, os.SEEK_SET)
        before = os.fstat(descriptor)
        if (not stat.S_ISREG(before.st_mode)
                or stat.S_IMODE(before.st_mode) != 0o555
                or before.st_nlink != 1):
            raise ValueError("executable is not one immutable single-link file")
        sha = hashlib.sha256()
        size = 0
        while True:
            block = os.read(descriptor, 64 * 1024)
            if not block:
                break
            sha.update(block)
            size += len(block)
        after = os.fstat(descriptor)
        fingerprint = _stat_identity(before)
        if fingerprint != _stat_identity(after) or size != before.st_size:
            raise ValueError("executable changed while it was inspected")
        return {"sha256": sha.hexdigest(), "bytes": size}, fingerprint
    except OSError as error:
        raise ValueError("executable descriptor could not be inspected") from error


def _open_executable(path):
    path = Path(path)
    nofollow = getattr(os, "O_NOFOLLOW", None)
    nonblock = getattr(os, "O_NONBLOCK", None)
    if nofollow is None or nonblock is None:
        raise ValueError("safe no-follow executable inspection is unavailable")
    if (not path.is_absolute()
            or Path(os.path.abspath(path)) != path):
        raise ValueError("executable path is not one normalized absolute path")
    descriptor = current = None
    try:
        descriptor = os.open(
            path, os.O_RDONLY | os.O_CLOEXEC | nofollow | nonblock,
        )
        identity, fingerprint = _descriptor_snapshot(descriptor)
        current = os.open(
            path, os.O_RDONLY | os.O_CLOEXEC | nofollow | nonblock,
        )
        if _stat_identity(os.fstat(current)) != fingerprint:
            raise ValueError("executable pathname changed during inspection")
        return descriptor, identity, fingerprint
    except OSError as error:
        if descriptor is not None:
            os.close(descriptor)
            descriptor = None
        raise ValueError("executable could not be inspected safely") from error
    except Exception:
        if descriptor is not None:
            os.close(descriptor)
            descriptor = None
        raise
    finally:
        if current is not None:
            os.close(current)


def _executable_snapshot(path):
    descriptor = None
    try:
        descriptor, identity, fingerprint = _open_executable(path)
        return identity, fingerprint
    finally:
        if descriptor is not None:
            os.close(descriptor)


def _materialize_built_executable(source, destination):
    source = Path(source)
    destination = Path(destination)
    nofollow = getattr(os, "O_NOFOLLOW", None)
    nonblock = getattr(os, "O_NONBLOCK", None)
    if nofollow is None or nonblock is None:
        raise ValueError("safe no-follow build output inspection is unavailable")
    if (not source.is_absolute() or Path(os.path.abspath(source)) != source
            or not destination.is_absolute()
            or Path(os.path.abspath(destination)) != destination
            or source == destination):
        raise ValueError("built executable paths are not normalized and distinct")
    descriptor = current = None
    try:
        descriptor = os.open(
            source, os.O_RDONLY | os.O_CLOEXEC | nofollow | nonblock,
        )
        before = os.fstat(descriptor)
        if (not stat.S_ISREG(before.st_mode) or before.st_size <= 0
                or not stat.S_IMODE(before.st_mode) & 0o111):
            raise ValueError("Cargo output is not one executable regular file")
        chunks = []
        size = 0
        while True:
            block = os.read(descriptor, 64 * 1024)
            if not block:
                break
            chunks.append(block)
            size += len(block)
        after = os.fstat(descriptor)
        current = os.open(
            source, os.O_RDONLY | os.O_CLOEXEC | nofollow | nonblock,
        )
        fingerprint = _stat_identity(before)
        if (fingerprint != _stat_identity(after)
                or fingerprint != _stat_identity(os.fstat(current))
                or size != before.st_size):
            raise ValueError("Cargo output changed while it was copied")
    except OSError as error:
        raise ValueError("Cargo output could not be copied safely") from error
    finally:
        if current is not None:
            os.close(current)
        if descriptor is not None:
            os.close(descriptor)
    raw = b"".join(chunks)
    _write_immutable(destination, raw, mode=0o555)
    identity, _ = _executable_snapshot(destination)
    if identity != {"sha256": _sha(raw), "bytes": len(raw)}:
        raise ValueError("private Rust executable identity changed")
    return destination, identity


def _capture_record(out, plan, executables, binary_records, scripts, env,
                    input_bytes, metrics_path, active, job):
    if (type(executables) is not dict
            or set(executables) != {"rust", "oracle"}
            or type(binary_records) is not dict
            or set(binary_records) != {"rust", "oracle"}):
        raise ValueError("capture executable map changed")
    engine = plan.get("engine") if isinstance(plan, dict) else None
    expected_record = binary_records.get(engine)
    if (engine not in executables or not isinstance(expected_record, dict)
            or not isinstance(expected_record.get("sha256"), str)
            or re.fullmatch(SHA256_PATTERN, expected_record["sha256"]) is None
            or type(expected_record.get("bytes")) is not int
            or expected_record["bytes"] <= 0):
        raise ValueError("capture executable expectation changed")
    binary = Path(executables[engine])
    expected_identity = {
        "sha256": expected_record["sha256"],
        "bytes": expected_record["bytes"],
    }
    descriptor = None
    try:
        descriptor, before_identity, before_fingerprint = _open_executable(binary)
        if before_identity != expected_identity:
            raise ValueError("capture executable differs from its declared identity")
        wrapper = [
            "/bin/bash", "--noprofile", "--norc", "-c",
            'exec -a "$1" "/proc/self/fd/$2"',
            "atlas-capture-exec", str(binary), str(descriptor),
        ]
        result = _execute_timed(
            wrapper, scripts, env, input_bytes, plan["timeout_seconds"],
            metrics_path, active, pass_fds=(descriptor,),
        )
        after_identity, after_fingerprint = _descriptor_snapshot(descriptor)
        current_identity, current_fingerprint = _executable_snapshot(binary)
        if (after_identity != before_identity
                or after_fingerprint != before_fingerprint
                or current_identity != before_identity
                or current_fingerprint != before_fingerprint):
            raise ValueError("capture executable changed during execution")
    finally:
        if descriptor is not None:
            os.close(descriptor)
    sequence = plan["sequence"]
    stem = "capture-%02d-%s-%s" % (
        sequence, plan["case_id"], plan["engine"],
    )
    return {
        "sequence": sequence,
        "case_id": plan["case_id"],
        "engine": plan["engine"],
        "invocation_id": "%s-%02d-%s-%d" % (
            job, sequence, plan["engine"], result["pid"],
        ),
        "pid": result["pid"],
        "process_group_id": result["process_group_id"],
        "fresh_process": result["pid"] == result["process_group_id"],
        "executable_sha256": before_identity["sha256"],
        "executable_bytes": before_identity["bytes"],
        "observation": {
            "engine": plan["engine"],
            "exit_status": result["exit_status"],
            "timed_out": result["timed_out"],
            "termination_uncertain": result["termination_uncertain"],
            "signal": result["signal"],
            "seconds": result["seconds"],
            "user_cpu_seconds": result["user_cpu_seconds"],
            "system_cpu_seconds": result["system_cpu_seconds"],
            "maxrss_kb": result["maxrss_kb"],
            "maxrss_approximate": result["maxrss_approximate"],
        },
        "input": _publish_artifact(out, stem + ".input.atlas", input_bytes),
        "stdout": _publish_artifact(out, stem + ".stdout", result["stdout_raw"]),
        "stderr": _publish_artifact(out, stem + ".stderr", result["stderr_raw"]),
        "time": _publish_artifact(out, stem + ".time", result["time_raw"]),
    }


def _materialize_blob(path, raw, mode):
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    _write_immutable(path, raw, mode=mode)
    return path


def _materialize_scripts(campaign, seal, destination):
    destination = Path(destination)
    destination.mkdir()
    for name, reference in sorted(seal["scripts"]["objects"].items()):
        _safe_relative(name)
        target = destination / name
        target.parent.mkdir(parents=True, exist_ok=True)
        _materialize_blob(target, read_blob(campaign, reference), 0o444)
    if file_manifest(destination) != seal["scripts"]["manifest"]:
        raise ValueError("materialized original scripts changed")
    return destination


def _write_workspace_file(path, raw):
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    if os.path.lexists(path):
        raise ValueError("workspace file already exists")
    path.write_bytes(raw)


def _publish_report(out, report):
    raw = (json.dumps(report, indent=2, sort_keys=True) + "\n").encode("utf-8")
    _write_immutable(Path(out) / "report.sha256", (_sha(raw) + "\n").encode())
    _write_immutable(Path(out) / "report.json", raw)


def _base_report(job):
    return {
        "schema": REPORT_SCHEMA,
        "job": job,
        "node": platform.node(),
        "status": AFTER_HARNESS_FAILURE,
        "evidence_maturity": "tests_first_after",
        "scope": REPORT_SCOPE,
        "pin": {},
        "accepted_source": {},
        "catalog": copy.deepcopy(CATALOG_RECORD),
        "commands": [],
        "invocations": [],
        "captures": [],
        "complete": False,
        "acceptance_eligible": False,
        "math_gate_released": False,
        "cache_gate_released": False,
        "performance_gate_released": False,
        "rank_gate_released": False,
        "integrity_rechecked": False,
        "source_integrity_rechecked": False,
        "ephemeral_workspace_removed": False,
        "source": {},
        "binaries": {},
        "limitations": copy.deepcopy(LIMITATIONS),
        "legacy_path_open_attempts": [],
        "thread_settings": copy.deepcopy(THREAD_SETTINGS),
        "provenance": {},
        "environment": {},
        "regression": {},
    }


def main():
    if not SUBMISSION_ENABLED:
        raise SystemExit("Weyl core BEFORE compute driver is not enabled")
    if STAGER_SUBMISSION_ENABLED is not True:
        raise SystemExit("Weyl core BEFORE stager is not enabled")
    job = os.environ.get("SLURM_JOB_ID")
    if not isinstance(job, str) or not job.isdecimal():
        raise SystemExit("compute nodes only")
    root = Path.cwd().resolve()
    preflight = gates(root)
    out = create_result_folder(root, job)
    report = _base_report(job)
    report["pin"] = preflight["pin"]
    report["accepted_source"] = preflight["pin"]["accepted_source"]
    report["provenance"] = {
        "pin_sha256": preflight["pin_sha256"],
        "submission_receipt": preflight["receipt"],
        "harness_inputs": preflight["inputs"],
        "campaign_record": preflight["record"],
    }
    active = {"process": None, "process_group": None}
    work = None
    capture_incomplete = False

    def interrupted(signum, _frame):
        process_group = active.get("process_group")
        _terminate_group(active.get("process"), process_group)
        if process_group is None or not _group_alive(process_group):
            active.update(process=None, process_group=None)
        raise RuntimeError("received signal " + str(signum))

    try:
        signal.signal(signal.SIGTERM, interrupted)
        signal.signal(signal.SIGINT, interrupted)
        catalog_raw = _stable_relative_bytes(root, CATALOG_PATH)
        catalog = decode_discovery_catalog(catalog_raw)
        if catalog != preflight["catalog"] or _sha(catalog_raw) != CATALOG_SHA256:
            raise ValueError("Weyl core catalog changed after preflight")
        catalog_reference = _publish_artifact(
            out, CATALOG_RECORD["artifact"]["path"], catalog_raw,
        )
        if catalog_reference != CATALOG_RECORD["artifact"]:
            raise ValueError("published catalog artifact identity changed")
        (regression_catalog_raw, regression_inspection_raw,
         regression_catalog, regression_artifacts) = _regression_inputs(root)
        (regression_catalog_reference, regression_inspection_reference,
         regression_artifact_references) = _publish_regression_inputs(
             out, regression_catalog_raw, regression_inspection_raw,
             regression_artifacts,
         )
        report["regression"] = {
            "catalog": regression_catalog_reference,
            "inspection": regression_inspection_reference,
            "artifacts": regression_artifact_references,
            "classification": {},
            "original_rerun_invocations": [],
            "selector_command": "weyl-context-regressions",
            "retained_control_command": "root-ladder-control",
            "inventory_command": "atlas-core-test-inventory",
        }

        campaign = campaign_stage(root)
        with ephemeral_job_workspace(out, "weyl-core-after-v3") as work:
            (work / "tmp").mkdir(mode=0o700)
            env = command_environment(dict(os.environ), work)
            report["environment"] = validate_environment_record(env)
            for command_name, _pattern in CHECK_COMMANDS:
                record, raw = _command_record(
                    out, command_name, root, work, env,
                    work / (command_name + ".time"), active,
                )
                report["commands"].append(record)
                count = EXPECTED_TEST_COUNTS[command_name]
                if re.search(
                        rb"Ran " + str(count).encode()
                        + rb" tests in [0-9.]+s\s+OK\s*$", raw) is None:
                    raise ValueError("checker inventory changed: " + command_name)

            for command_name in ("rustc-version", "cargo-version"):
                record, raw = _command_record(
                    out, command_name, root, work, env,
                    work / (command_name + ".time"), active,
                )
                report["commands"].append(record)
                if not raw.strip():
                    raise ValueError("empty toolchain identity: " + command_name)

            source = work / "source"
            materialize_source_archive(
                campaign, preflight["seal"]["source_object"],
                preflight["seal"]["source_manifest"], source,
            )
            for name, raw in _sealed_boundary_fixtures(
                    campaign, preflight["seal"]).items():
                _write_workspace_file(source / name, raw)
            if (_sha(_stable_relative_bytes(root, TEST_PATCH))
                    != PATCH_HASHES[TEST_PATCH]
                    or _sha(_stable_relative_bytes(root, PRODUCTION_PATCH))
                    != PATCH_HASHES[PRODUCTION_PATCH]):
                raise ValueError("ladder repair patches changed")
            _apply_unified_patch(source, root / TEST_PATCH)
            if (digest(source / SESSION) != TESTS_ONLY_HASHES[SESSION]
                    or digest(source / ROOT_SYSTEM)
                       != TESTS_ONLY_HASHES[ROOT_SYSTEM]):
                raise ValueError("tests-only reconstructed source changed")
            _apply_unified_patch(source, root / PRODUCTION_PATCH)
            if any(digest(source / name) != wanted
                   for name, wanted in FINAL_HASHES.items()):
                raise ValueError("final reconstructed source changed")
            accepted_manifest = preflight["accepted_source_manifest"]
            if file_manifest(source) != accepted_manifest:
                raise ValueError("reconstructed accepted source changed")

            regression_patch = _stable_relative_bytes(
                root, REGRESSION_PATCH_PATH,
            )
            if (_sha(regression_patch)
                    != REGRESSION_PATCH_HASHES[REGRESSION_PATCH_PATH]):
                raise ValueError("Weyl regression tests-only patch changed")
            changed = _apply_unified_patch(
                source, root / REGRESSION_PATCH_PATH,
            )
            if changed != [SESSION]:
                raise ValueError("Weyl regression patch changed production scope")
            for name, wanted in sorted(REGRESSION_FIXTURE_HASHES.items()):
                raw_fixture = _stable_relative_bytes(root, name)
                if _sha(raw_fixture) != wanted:
                    raise ValueError("Weyl regression fixture changed: " + name)
                _write_workspace_file(source / name, raw_fixture)
            if any(digest(source / name) != wanted
                   for name, wanted in REGRESSION_SOURCE_HASHES.items()):
                raise ValueError("tests-first source hashes changed")
            final_manifest = regression_source_manifest(accepted_manifest)
            if (final_manifest != preflight["regression_source_manifest"]
                    or file_manifest(source) != final_manifest):
                raise ValueError("reconstructed tests-first source changed")

            repair_patch = _stable_relative_bytes(root, REPAIR_PATCH_PATH)
            if _sha(repair_patch) != REPAIR_PATCH_HASHES[REPAIR_PATCH_PATH]:
                raise ValueError("Weyl repair patch changed")
            changed = _apply_unified_patch(source, root / REPAIR_PATCH_PATH)
            if set(changed) != {DOMAIN_BUILTINS, TYPED, WEYL_SUBGROUP}:
                raise ValueError("Weyl repair patch scope changed")
            if any(digest(source / name) != wanted
                   for name, wanted in REPAIRED_SOURCE_HASHES.items()):
                raise ValueError("repaired source hashes changed")
            final_manifest = repaired_source_manifest(final_manifest)
            if (final_manifest != preflight["repaired_source_manifest"]
                    or file_manifest(source) != final_manifest):
                raise ValueError("reconstructed repaired source changed")

            manifest_sha = _sha(json.dumps(
                final_manifest, sort_keys=True,
                separators=(",", ":"),
            ).encode())
            report["source"] = {
                "files": len(final_manifest),
                "manifest_sha256": manifest_sha,
                "manifest": final_manifest,
            }

            manifest_file = work / "expected-source-manifest.json"
            manifest_file.write_text(
                json.dumps(final_manifest, sort_keys=True),
                encoding="utf-8",
            )
            record, raw = _command_record(
                out, "source-reconstruction", root, work, env,
                work / "source-reconstruction.time", active,
            )
            # The timed verifier excludes the preceding archive extraction and
            # patch application, so its metrics only approximate the complete
            # reconstruction phase and never support a speed claim.
            record["maxrss_approximate"] = True
            report["commands"].append(record)
            expected_line = (
                "SOURCE_RECONSTRUCTION_OK %d %s\n"
                % (len(final_manifest), manifest_sha)
            ).encode()
            if (raw != expected_line
                    or manifest_sha
                       != AFTER_SOURCE_MANIFEST_SHA256):
                raise ValueError("repaired source manifest digest changed")

            record, _ = _command_record(
                out, "release-build", root, work, env,
                work / "release-build.time", active,
            )
            report["commands"].append(record)
            rust_binary, rust_identity = _materialize_built_executable(
                work / "target/release/atlas-cli", work / "rust-atlas",
            )
            oracle_raw = read_blob(
                campaign, preflight["seal"]["binaries"]["oracle"],
            )
            if _sha(oracle_raw) != ACCEPTED_SOURCE["oracle"]["binary_sha256"]:
                raise ValueError("original Atlas binary changed")
            oracle_binary = _materialize_blob(
                work / "oracle-atlas", oracle_raw, 0o555,
            )
            oracle_identity, _ = _executable_snapshot(oracle_binary)
            report["binaries"] = {
                "rust": rust_identity,
                "oracle": {
                    "sha256": oracle_identity["sha256"],
                    "bytes": oracle_identity["bytes"],
                    "commit": ACCEPTED_SOURCE["oracle"]["commit"],
                },
            }
            executables = {"rust": rust_binary, "oracle": oracle_binary}
            scripts = _materialize_scripts(
                campaign, preflight["seal"], work / "atlas-scripts",
            )

            cases = {case["id"]: case for case in catalog["cases"]}
            pinned_cases = {
                case["id"]: case for case in preflight["pin"]["catalog"]["cases"]
            }
            for plan in invocation_plan(catalog):
                case = cases[plan["case_id"]]
                fixture_name = "tests/math/generics/" + case["file"]
                fixture = _stable_relative_bytes(root, fixture_name)
                input_bytes = (
                    ("prints(\"MATH_BEGIN " + case["id"] + "\")\n").encode()
                    + fixture
                    + ("prints(\"MATH_END " + case["id"] + "\")\nquit\n").encode()
                )
                pinned = pinned_cases[case["id"]]
                if (len(input_bytes) != pinned["input_bytes"]
                        or _sha(input_bytes) != pinned["input_sha256"]):
                    raise ValueError("capture input envelope changed")
                report["invocations"].append(_capture_record(
                    out, plan, executables, report["binaries"], scripts, env,
                    input_bytes,
                    work / ("capture-%02d.time" % plan["sequence"]),
                    active, job,
                ))

            validate_invocation_records(report["invocations"], out)
            report["captures"] = _capture_runs(
                catalog, report["invocations"], out,
            )
            capture_incomplete = any(
                capture["status"] not in COMPLETE_CAPTURE_STATUSES
                for capture in report["captures"]
            )
            if any(capture["status"] not in NONACCEPTING_STATUSES
                   for capture in report["captures"]):
                raise ValueError("capture classifier returned an unknown status")

            for command_name in (
                    "atlas-core-test-inventory", "weyl-context-regressions",
                    "root-ladder-control"):
                record, _raw = _command_record(
                    out, command_name, root, work, env,
                    work / (command_name + ".time"), active,
                )
                report["commands"].append(record)
            _observation, rerun_ids = _after_observation(
                report, out, regression_catalog,
            )
            report["regression"]["original_rerun_invocations"] = rerun_ids
            classification = _regression_classification(report, out)
            report["regression"]["classification"] = classification
            classification_status = classification.get("status")
            allowed_after_statuses = (
                AFTER_REPRODUCED, AFTER_STILL_FAILING,
                AFTER_PROVENANCE_FAILURE, AFTER_HARNESS_FAILURE,
            )
            if (classification_status not in allowed_after_statuses
                    or any(classification.get(key) is not False
                           for key in CLAIM_KEYS)):
                raise ValueError("AFTER classifier returned an invalid result")
            after_complete = classification_status == AFTER_REPRODUCED

            final_preflight = gates(root)
            if final_preflight != preflight:
                raise ValueError("Weyl core capture preflight changed during execution")
            final_rust_identity, _ = _executable_snapshot(rust_binary)
            final_oracle_identity, _ = _executable_snapshot(oracle_binary)
            if (file_manifest(source) != final_manifest
                    or final_rust_identity != report["binaries"]["rust"]
                    or final_oracle_identity != {
                        "sha256": report["binaries"]["oracle"]["sha256"],
                        "bytes": report["binaries"]["oracle"]["bytes"],
                    }
                    or file_manifest(scripts)
                       != preflight["seal"]["scripts"]["manifest"]):
                raise ValueError("ephemeral execution inputs changed")
            report["source_integrity_rechecked"] = True
            _validate_command_records(report["commands"], out)
            validate_invocation_records(report["invocations"], out)
            if _capture_runs(catalog, report["invocations"], out) != report["captures"]:
                raise ValueError("raw capture artifacts changed")
            if (_regression_classification(report, out)
                    != report["regression"]["classification"]):
                raise ValueError("raw BEFORE artifacts changed")
            report["integrity_rechecked"] = True
            report["legacy_path_open_attempts"] = list(LEGACY_PATH_OPEN_ATTEMPTS)
            if report["legacy_path_open_attempts"]:
                raise ValueError("retired top-level path access was attempted")
            report["complete"] = not capture_incomplete and after_complete
            report["status"] = (
                INCOMPLETE_STATUS if capture_incomplete
                else classification_status
            )
        if work.exists():
            raise ValueError("ephemeral Weyl core capture workspace was retained")
        report["ephemeral_workspace_removed"] = True
        if not capture_incomplete and report["status"] == SUCCESS_STATUS:
            validate_report(report, out)
    except Exception as error:
        if isinstance(error, CommandRecordFailure):
            report["failed_command"] = copy.deepcopy(error.evidence)
        report.update(
            status=AFTER_HARNESS_FAILURE,
            complete=False,
            error=traceback.format_exc(),
        )
    finally:
        try:
            process_group = active.get("process_group")
            cleanup_uncertain = _terminate_group(
                active.get("process"), process_group,
            )
            group_alive = (
                process_group is not None and _group_alive(process_group)
            )
            if not group_alive:
                active.update(process=None, process_group=None)
            if cleanup_uncertain or group_alive:
                report.update(
                    status=AFTER_HARNESS_FAILURE,
                    complete=False,
                    cleanup_error=(
                        "timed command process-group cleanup was uncertain"
                    ),
                )
        except Exception:
            report.update(
                status=AFTER_HARNESS_FAILURE,
                complete=False,
                cleanup_error=traceback.format_exc(),
            )
        if work is not None and not work.exists():
            report["ephemeral_workspace_removed"] = True
    try:
        _publish_report(out, report)
        published = True
    except Exception:
        published = False
        print(traceback.format_exc(), file=sys.stderr, flush=True)
    print(report["status"], out, flush=True)
    return int(
        not published or report["status"] != SUCCESS_STATUS
        or report["complete"] is not True
    )


if __name__ == "__main__":
    raise SystemExit(main())

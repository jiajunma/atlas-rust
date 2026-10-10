import ast
import builtins
from contextlib import ExitStack
import copy
import dis
import hashlib
import inspect
import json
import math
import os
from pathlib import Path, PurePosixPath
import signal
import subprocess
import tempfile
import types
import unittest
from unittest.mock import patch

import math_weyl_context_core_capture as driver
import stage_weyl_context_core_capture as stager
import weyl_context_core_contract as contract
import weyl_context_core_regression as regression


EXPECTED_ORDER = (
    ("weyl_context_g2_cold_dual", "oracle"),
    ("weyl_context_g2_cold_dual", "rust"),
    ("weyl_context_g2_prewarmed_dual", "rust"),
    ("weyl_context_g2_prewarmed_dual", "oracle"),
)
CAUSE_MESSAGES = {
    contract.MISMATCH: "Weyl group mismatch",
    contract.HIGH_WORD: "Illegal Weyl word entry 1 (should be <1)",
    contract.NEGATIVE_WORD: "Negative integer where unsigned is required",
    contract.HIGH_WORD_G2: "Illegal Weyl word entry 2 (should be <2)",
}
REPORT_KEYS = {
    "schema", "job", "node", "status", "evidence_maturity", "scope",
    "pin", "accepted_source", "catalog", "commands", "invocations",
    "captures", "complete", "acceptance_eligible", "math_gate_released",
    "cache_gate_released", "integrity_rechecked",
    "ephemeral_workspace_removed", "source", "binaries", "limitations",
    "legacy_path_open_attempts", "thread_settings",
    "source_integrity_rechecked", "provenance", "environment",
    "performance_gate_released", "rank_gate_released",
}
INVOCATION_KEYS = {
    "sequence", "case_id", "engine", "invocation_id", "pid",
    "process_group_id", "fresh_process", "observation", "input", "stdout",
    "stderr", "time", "executable_sha256", "executable_bytes",
}
COMMAND_KEYS = {
    "name", "argv", "cwd", "exit_status", "timed_out",
    "termination_uncertain", "signal", "seconds", "user_cpu_seconds",
    "system_cpu_seconds", "maxrss_kb", "maxrss_approximate", "stdout",
    "stderr", "time",
}
ARTIFACT_KEYS = {"path", "sha256", "bytes"}
PROVENANCE_KEYS = {
    "pin_sha256", "submission_receipt", "harness_inputs", "campaign_record",
}
ENVIRONMENT_KEYS = {
    "PATH", "HOME", "LD_LIBRARY_PATH", "CARGO_HOME", "RUSTUP_HOME",
    "CARGO_TARGET_DIR", "CARGO_BUILD_JOBS", "CARGO_INCREMENTAL",
    "CARGO_NET_OFFLINE", "CARGO_TERM_COLOR", "RAYON_NUM_THREADS",
    "OMP_NUM_THREADS", "OPENBLAS_NUM_THREADS", "LC_ALL", "LANG",
    "PYTHONDONTWRITEBYTECODE", "TMPDIR", "TMP", "TEMP",
}
ORACLE_SHA256 = (
    "d4f0f3dc3a82102529aa2ec562db0601e99b368dee25d5539ca52dae2fd37a5a"
)
ORACLE_COMMIT = "7e1b958c7aa9456769cc9cf09ac1542814b4800a"
RUST_SHA256 = "c" * 64
ORACLE_BYTES = 654321
RUST_BYTES = 123456


def unresolved_globals(code, namespace):
    missing = {
        instruction.argval
        for instruction in dis.get_instructions(code)
        if instruction.opname == "LOAD_GLOBAL"
        and instruction.argval not in namespace
        and not hasattr(builtins, instruction.argval)
    }
    for constant in code.co_consts:
        if isinstance(constant, types.CodeType):
            missing.update(unresolved_globals(constant, namespace))
    return missing


def catalog_value():
    return {
        "schema": contract.G2_CATALOG_SCHEMA,
        "evidence_maturity": contract.CATALOG_MATURITY,
        "scope": contract.G2_CATALOG_SCOPE,
        "cases": [dict(row) for row in contract.G2_EXPECTED_CASES],
    }


def predictions_for(case_id):
    found = contract.PREDICTIONS.get(case_id)
    return found if found is not None else contract.G2_PREDICTIONS[case_id]


def observation(engine, exit_status):
    return {
        "engine": engine,
        "exit_status": exit_status,
        "timed_out": False,
        "termination_uncertain": False,
        "signal": None,
        "seconds": 1.25,
        "user_cpu_seconds": 1.0,
        "system_cpu_seconds": 0.25,
        "maxrss_kb": 4096,
        "maxrss_approximate": False,
    }


def stdout_bytes(case_id, engine):
    golden = (
        Path(__file__).resolve().parents[1] / "tests" / "math"
        / "generics" / (case_id + ".oracle.stdout")
    )
    if engine == "oracle" and golden.exists():
        return golden.read_bytes()
    prediction = predictions_for(case_id)[engine]
    lines = ["MATH_BEGIN " + case_id]
    lines.extend(prediction["payload_lines"])
    lines.extend(["MATH_END " + case_id, "Bye."])
    return ("\n".join(lines) + "\n").encode()


def stderr_bytes(case_id, engine):
    golden = (
        Path(__file__).resolve().parents[1] / "tests" / "math"
        / "generics" / (case_id + ".oracle.stderr")
    )
    if engine == "oracle" and golden.exists():
        return golden.read_bytes()
    causes = predictions_for(case_id)[engine]["causes"]
    if engine == "oracle":
        return "".join(
            "Runtime error:\n  " + CAUSE_MESSAGES[cause]
            + "\nEvaluation aborted.\n"
            for cause in causes
        ).encode()
    return "".join(
        "Runtime error at <stdin>:%d:1: %s\n"
        "  | expression_%d\n"
        "  | ^^^^^^^^^^^^\n" % (index, CAUSE_MESSAGES[cause], index)
        for index, cause in enumerate(causes, start=1)
    ).encode()


def artifact(root, relative, raw):
    path = root / relative
    path.parent.mkdir(parents=True, exist_ok=True)
    if os.path.lexists(path):
        if not path.is_symlink():
            path.chmod(0o600)
        path.unlink()
    path.write_bytes(raw)
    path.chmod(0o444)
    return {
        "path": PurePosixPath(relative).as_posix(),
        "sha256": hashlib.sha256(raw).hexdigest(),
        "bytes": len(raw),
    }


def invocation_records(root):
    records = []
    raw_by_id = {}
    for sequence, (case_id, engine) in enumerate(EXPECTED_ORDER):
        prediction = predictions_for(case_id)[engine]
        case = next(row for row in contract.G2_EXPECTED_CASES
                    if row["id"] == case_id)
        fixture = (
            Path(__file__).resolve().parents[1] / "tests" / "math"
            / "generics" / case["file"]
        ).read_bytes()
        input_raw = (
            ('prints("MATH_BEGIN ' + case_id + '")\n').encode()
            + fixture
            + ('prints("MATH_END ' + case_id + '")\nquit\n').encode()
        )
        invocation_id = "%02d-%s-%s" % (sequence, case_id, engine)
        stdout = stdout_bytes(case_id, engine)
        stderr = stderr_bytes(case_id, engine)
        time_raw = (
            "\tUser time (seconds): 1.00\n"
            "\tSystem time (seconds): 0.25\n"
            "\tElapsed (wall clock) time (h:mm:ss or m:ss): 0:01.25\n"
            "\tMaximum resident set size (kbytes): 4096\n"
            "\tExit status: %d\n" % prediction["exit_status"]
        ).encode()
        prefix = "capture-%02d-%s-%s" % (sequence, case_id, engine)
        records.append({
            "sequence": sequence,
            "case_id": case_id,
            "engine": engine,
            "invocation_id": invocation_id,
            "pid": 1000 + sequence,
            "process_group_id": 1000 + sequence,
            "fresh_process": True,
            "executable_sha256": (
                ORACLE_SHA256 if engine == "oracle" else RUST_SHA256),
            "executable_bytes": (
                ORACLE_BYTES if engine == "oracle" else RUST_BYTES),
            "observation": observation(engine, prediction["exit_status"]),
            "input": artifact(root, prefix + ".input.atlas", input_raw),
            "stdout": artifact(root, prefix + ".stdout", stdout),
            "stderr": artifact(root, prefix + ".stderr", stderr),
            "time": artifact(root, prefix + ".time", time_raw),
        })
        raw_by_id[invocation_id] = (stdout, stderr)
    return records, raw_by_id


def command_records(root):
    records = []
    for name in driver.COMMAND_NAMES:
        command = driver.COMMAND_CONTRACTS[name]
        argv = copy.deepcopy(command["argv"])
        cwd = command["cwd"]
        if name in stager.EXPECTED_TEST_COUNTS:
            stdout = (
                "Ran %d tests in 0.001s\n\nOK\n"
                % stager.EXPECTED_TEST_COUNTS[name]
            ).encode()
        elif name == "source-reconstruction":
            stdout = (
                "SOURCE_RECONSTRUCTION_OK %d %s\n"
                % (stager.G2_SOURCE["files"],
                   stager.G2_SOURCE["source_manifest_sha256"])
            ).encode()
        elif name == "atlas-core-test-inventory":
            names = list(regression.SELECTOR_TESTS) + [regression.CONTROL_TEST]
            names.extend(
                "session::tests::frozen_baseline_%03d" % index
                for index in range(
                    regression.EXPECTED_INVENTORY_COUNT - len(names))
            )
            stdout = (
                "\n".join(test + ": test" for test in names)
                + "\n\n632 tests, 0 benchmarks\n"
            ).encode()
        elif name == "weyl-context-regressions":
            stdout = (
                "running 2 tests\n"
                "WEYL_CONTEXT_CORE_READY weyl_context_core_cold_dual "
                "diagnostics=2\n"
                "WEYL_CONTEXT_CORE_READY weyl_context_core_prewarmed_dual "
                "diagnostics=4\n"
                "test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; "
                "630 filtered out; finished in 0.01s\n"
            ).encode()
        elif name == "root-ladder-control":
            stdout = (
                "running 1 test\n"
                "test session::tests::root_ladder_coordinate_boundary_original "
                "... ok\n"
                "test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; "
                "631 filtered out; finished in 0.01s\n"
            ).encode()
        else:
            stdout = (name + " complete\n").encode()
        stderr = b""
        exit_status = 0
        time_raw = (
            b"\tUser time (seconds): 0.50\n"
            b"\tSystem time (seconds): 0.25\n"
            b"\tElapsed (wall clock) time (h:mm:ss or m:ss): 0:01.00\n"
            b"\tMaximum resident set size (kbytes): 2048\n"
            + ("\tExit status: %d\n" % exit_status).encode()
        )
        prefix = name
        records.append({
            "name": name,
            "argv": argv,
            "cwd": cwd,
            "exit_status": exit_status,
            "timed_out": False,
            "termination_uncertain": False,
            "signal": None,
            "seconds": 1.0,
            "user_cpu_seconds": 0.5,
            "system_cpu_seconds": 0.25,
            "maxrss_kb": 2048,
            "maxrss_approximate": name == "source-reconstruction",
            "stdout": artifact(root, prefix + ".stdout", stdout),
            "stderr": artifact(root, prefix + ".stderr", stderr),
            "time": artifact(root, prefix + ".time", time_raw),
        })
    return records


def valid_pin():
    inputs = {name: "a" * 64 for name in stager.STAGE_INPUT_NAMES}
    inputs.update(stager.PATCH_HASHES)
    inputs.update(stager.REGRESSION_PATCH_HASHES)
    inputs.update(stager.REPAIR_PATCH_HASHES)
    inputs.update(stager.REGRESSION_FIXTURE_HASHES)
    inputs.update(stager.G2_FIXTURE_HASHES)
    inputs[stager.REGRESSION_CATALOG_PATH] = stager.REGRESSION_CATALOG_SHA256
    inputs[stager.REGRESSION_INSPECTION_PATH] = (
        stager.REGRESSION_INSPECTION_SHA256)
    inputs[stager.V8_SUBMISSION_EVIDENCE_PATH] = (
        stager.V8_SUBMISSION_EVIDENCE_SHA256)
    for evidence in (
            stager.V1_FAILURE_EVIDENCE,
            stager.V2_FAILURE_EVIDENCE,
            stager.V3_FAILURE_EVIDENCE,
            stager.V4_FAILURE_EVIDENCE,
            stager.V5_FAILURE_EVIDENCE,
            stager.V6_FAILURE_EVIDENCE,
            stager.V7_FAILURE_EVIDENCE,
            stager.BEFORE_V1_FAILURE_EVIDENCE,
            stager.BEFORE_V2_FAILURE_EVIDENCE,
            stager.BEFORE_V3_FAILURE_EVIDENCE,
            stager.BEFORE_V4_RESULT_EVIDENCE,
            stager.AFTER_V1_FAILURE_EVIDENCE,
            stager.AFTER_V2_FAILURE_EVIDENCE,
            stager.AFTER_V3_FAILURE_EVIDENCE,
            stager.AFTER_V4_FAILURE_EVIDENCE,
            stager.AFTER_V5_RESULT_EVIDENCE,
            stager.AFTER_V5_SUBMISSION_EVIDENCE,
    ):
        inputs[evidence["file"]] = evidence["sha256"]
    inputs[stager.PRIOR_CREATION_FAILURE_EVIDENCE["file"]] = (
        stager.PRIOR_CREATION_FAILURE_EVIDENCE["sha256"])
    inputs[stager.V3_FAILURE_EVIDENCE["file"]] = (
        stager.V3_FAILURE_EVIDENCE["sha256"])
    inputs[stager.V4_FAILURE_EVIDENCE["file"]] = (
        stager.V4_FAILURE_EVIDENCE["sha256"])
    inputs[stager.V5_FAILURE_EVIDENCE["file"]] = (
        stager.V5_FAILURE_EVIDENCE["sha256"])
    inputs[stager.V6_FAILURE_EVIDENCE["file"]] = (
        stager.V6_FAILURE_EVIDENCE["sha256"])
    inputs[stager.V7_FAILURE_EVIDENCE["file"]] = (
        stager.V7_FAILURE_EVIDENCE["sha256"])
    return stager.build_pin(
        inputs,
        "b" * 64,
        copy.deepcopy(stager.EXPECTED_TEST_COUNTS),
        {
            "receipt_sha256": "c" * 64,
            "contract_sha256": "d" * 64,
        },
    )


def recovery_predecessor():
    history = [
        {
            "stage": "/campaign/stages/predecessor-%d" % index,
            "script": "hpc/predecessor-%d.sbatch" % index,
            "queue_before": [],
            "status": "SUBMITTED",
            "max_outstanding": 10,
            "job": str(7000 + index),
        }
        for index in range(7)
    ]
    history.append({
        "stage": "/campaign/stages/weyl-context-core-capture-v1",
        "script": stager.SBATCH,
        "queue_before": [],
        "status": "SUBMITTED",
        "max_outstanding": 10,
        "job": "7007",
        "pin_sha256": "1" * 64,
        "stage_creation_sha256": "2" * 64,
    })
    history.append({
        "stage": "/campaign/stages/weyl-context-core-capture-v2",
        "script": stager.SBATCH,
        "queue_before": [],
        "status": "SUBMITTED",
        "max_outstanding": 10,
        "job": "7008",
        "pin_sha256": "3" * 64,
        "stage_creation_sha256": "4" * 64,
    })
    history.append({
        "stage": "/campaign/stages/weyl-context-core-capture-v3",
        "script": stager.SBATCH,
        "queue_before": [],
        "status": "SUBMITTED",
        "max_outstanding": 10,
        "job": "7009",
        "pin_sha256": "5" * 64,
        "stage_creation_sha256": "6" * 64,
    })
    history.append({
        "stage": "/campaign/stages/weyl-context-core-capture-v4",
        "script": stager.SBATCH,
        "queue_before": [],
        "status": "SUBMITTED",
        "max_outstanding": 10,
        "job": "7010",
        "pin_sha256": "7" * 64,
        "stage_creation_sha256": "8" * 64,
    })
    history.append({
        "stage": "/campaign/stages/weyl-context-core-capture-v5",
        "script": stager.SBATCH,
        "queue_before": [],
        "status": "SUBMITTED",
        "max_outstanding": 10,
        "job": "7011",
        "pin_sha256": "9" * 64,
        "stage_creation_sha256": "a" * 64,
    })
    history.append({
        "stage": "/campaign/stages/weyl-context-core-capture-v6",
        "script": stager.SBATCH,
        "queue_before": [],
        "status": "SUBMITTED",
        "max_outstanding": 10,
        "job": "7012",
        "pin_sha256": "b" * 64,
        "stage_creation_sha256": "c" * 64,
    })
    history.append({
        "stage": "/campaign/stages/weyl-context-core-capture-v7",
        "script": stager.SBATCH,
        "queue_before": [],
        "status": "SUBMITTED",
        "max_outstanding": 10,
        "job": "7013",
        "pin_sha256": "d" * 64,
        "stage_creation_sha256": "e" * 64,
    })
    history.append({
        "stage": "/campaign/stages/weyl-context-core-capture-v8",
        "script": stager.SBATCH,
        "queue_before": [],
        "status": "SUBMITTED",
        "max_outstanding": 10,
        "job": "7014",
        "pin_sha256": "f" * 64,
        "stage_creation_sha256": "0" * 64,
    })
    history.append({
        "stage": "/campaign/stages/weyl-context-core-before-v1",
        "script": stager.SBATCH,
        "queue_before": [],
        "status": "SUBMITTED",
        "max_outstanding": 10,
        "job": "3884862",
        "pin_sha256": "1" * 64,
        "stage_creation_sha256": "2" * 64,
    })
    history.append({
        "stage": "/campaign/stages/weyl-context-core-before-v2",
        "script": stager.SBATCH,
        "queue_before": [],
        "status": "SUBMITTED",
        "max_outstanding": 10,
        "job": "3884880",
        "pin_sha256": (
            "bc98d9a033ab1fb0829456ef8a852b0db504a2eeffff6a6e6c377ddad045b750"
        ),
        "stage_creation_sha256": (
            "82b88fcc782c540055f11ef07e37244e73bec13fd1179c7d0b3c40747978dec9"
        ),
    })
    history.append({
        "stage": "/campaign/stages/weyl-context-core-before-v3",
        "script": stager.SBATCH,
        "queue_before": [],
        "status": "SUBMITTED",
        "max_outstanding": 10,
        "job": "3884903",
        "pin_sha256": (
            "a56d700e8b9aff4fea37ae6fc8f8281b9a3447befa20ce5ec4b6290a50857eb7"
        ),
        "stage_creation_sha256": (
            "72884ef76fca402fee932098abefe4a8d9ce0e9593b6229c0cb514399aeee214"
        ),
    })
    history.append({
        "stage": "/campaign/stages/weyl-context-core-before-v4",
        "script": stager.SBATCH,
        "queue_before": [],
        "status": "SUBMITTED",
        "max_outstanding": 10,
        "job": "3886748",
        "pin_sha256": (
            "54221cf4545875ec768c09d92753ceba5c5140faeacd895cbf8af36327e18e78"
        ),
        "stage_creation_sha256": (
            "c9197be0f005da73d110a3aac6d9b0531fe92728dd0f8db37d7c2a8b992307c7"
        ),
    })
    history.append({
        "stage": "/campaign/stages/weyl-context-core-after-v1",
        "script": stager.SBATCH,
        "queue_before": [],
        "status": "SUBMITTED",
        "max_outstanding": 10,
        "job": "3890328",
        "pin_sha256": (
            "396f30f2dae9e52637fbfeb0c8b5510286090b0801da4787742c0f2d2b9d7bb4"
        ),
        "stage_creation_sha256": (
            "ba22b03c4f87171d1ec1b5eb7aa0b8d6018e15d03054e9d5822eabc1e58771ad"
        ),
    })
    history.append({
        "stage": "/campaign/stages/weyl-context-core-after-v2",
        "script": stager.SBATCH,
        "queue_before": [],
        "status": "SUBMITTED",
        "max_outstanding": 10,
        "job": "3890580",
        "pin_sha256": (
            "3820fd83e64a9a9b8503eedc379e3a4e9dc62a583ca778d232c358bd1940918a"
        ),
        "stage_creation_sha256": (
            "e468bf226f31f35177703d4b5ed88ef230a03396a5be5dc94671d6bee94ec29b"
        ),
    })
    history.append({
        "stage": "/campaign/stages/weyl-context-core-after-v3",
        "script": stager.SBATCH,
        "queue_before": [],
        "status": "SUBMITTED",
        "max_outstanding": 10,
        "job": "3899303",
        "pin_sha256": (
            "b7f683d27b21fa1f8b98d22f54445e7d8edb82b22ff78a7c89d8420dcc70b52c"
        ),
        "stage_creation_sha256": (
            "387ecb1f71635488d9dd59d10f8d685120941807e9df768268d340d14b42a2bc"
        ),
    })
    history.append({
        "stage": "/campaign/stages/weyl-context-core-after-v4",
        "script": stager.SBATCH,
        "queue_before": [],
        "status": "SUBMITTED",
        "max_outstanding": 10,
        "job": "3899885",
        "pin_sha256": (
            "b1855bcab3c70cfb2a83943bd484de4c58e4b5266442b6c61b6a05bd0b825f25"
        ),
        "stage_creation_sha256": (
            "1b3e32a192c6e2e7136d7ee7bc04f5aeb7c0b9d2df06077933832fa3c4829508"
        ),
    })
    history.append({
        "stage": "/campaign/stages/weyl-context-core-after-v5",
        "script": stager.SBATCH,
        "queue_before": [],
        "status": "SUBMITTED",
        "max_outstanding": 10,
        "job": "3900050",
        "pin_sha256": (
            "d13190716f1996cc71c068bbf9589beec0553749c4d805fb3e02894cb83ff4d2"
        ),
        "stage_creation_sha256": (
            "c968e621a0265026999c0962b321e5c253f0fa68a5cf8fb6191a10de42e4cc98"
        ),
    })
    return history


def unconfirmed_capture_attempt(root, pin):
    return {
        "stage": str(Path(root).resolve()),
        "script": stager.SBATCH,
        "queue_before": ["6999"],
        "status": "SUBMISSION_INTENT_NOT_CONFIRMED",
        "max_outstanding": 10,
        "pin_sha256": stager.saved_json_sha(pin),
        "stage_creation_sha256": pin["stage_creation"]["receipt_sha256"],
    }


def capture_results(records, raw_by_id):
    results = []
    for case in contract.G2_EXPECTED_CASES:
        selected = []
        for row in records:
            if row["case_id"] != case["id"]:
                continue
            stdout, stderr = raw_by_id[row["invocation_id"]]
            selected.append({
                "engine": row["engine"],
                "observation": copy.deepcopy(row["observation"]),
                "stdout": stdout,
                "stderr": stderr,
                "fresh_process": row["fresh_process"],
                "invocation_id": row["invocation_id"],
            })
        results.append(contract.classify_capture(dict(case), selected))
    return results


def regression_references(root):
    repository = Path(__file__).resolve().parents[1]
    catalog_raw = (repository / stager.REGRESSION_CATALOG_PATH).read_bytes()
    inspection_raw = (
        repository / stager.REGRESSION_INSPECTION_PATH
    ).read_bytes()
    catalog = regression.decode_catalog(catalog_raw)
    artifacts = {}
    references = {}
    for case in catalog["cases"]:
        for name in (
                case["fixture"], case["oracle_stdout"]["file"],
                case["oracle_stderr"]["file"]):
            raw = repository.joinpath("tests", "math", "generics", name).read_bytes()
            artifacts[name] = raw
            references[name] = artifact(root, "regression-" + name, raw)
    regression.validate_artifacts(catalog, artifacts)
    return {
        "catalog": artifact(root, "regression-catalog.json", catalog_raw),
        "inspection": artifact(root, "capture-v8-inspection.json", inspection_raw),
        "artifacts": references,
        "classification": {},
        "original_rerun_invocations": [],
        "selector_command": "weyl-context-regressions",
        "retained_control_command": "root-ladder-control",
        "inventory_command": "atlas-core-test-inventory",
    }


def valid_report(root):
    pin = valid_pin()
    pin_sha256 = stager.saved_json_sha(pin)
    stage = str(Path(stager.PREDECESSOR["stage"]).parent / stager.STAGE_NAME)
    campaign_record = {
        "stage": stage,
        "script": stager.SBATCH,
        "queue_before": [],
        "status": "SUBMITTED",
        "max_outstanding": 10,
        "job": "12345",
        "pin_sha256": pin_sha256,
        "stage_creation_sha256": pin["stage_creation"]["receipt_sha256"],
    }
    accepted_report = json.loads(
        (Path(__file__).resolve().parents[1]
         / stager.AFTER_REPORT_PATH).read_text())
    accepted_manifest = accepted_report["source_files"]
    manifest = driver.g2_source_manifest(accepted_manifest)
    catalog_raw = (
        Path(__file__).resolve().parents[1]
        / stager.CATALOG_PATH
    ).read_bytes()
    catalog_artifact = artifact(root, "catalog.json", catalog_raw)
    invocations, raw_by_id = invocation_records(root)
    report = {
        "schema": driver.REPORT_SCHEMA,
        "job": "12345",
        "node": "compute-node",
        "status": driver.SUCCESS_STATUS,
        "evidence_maturity": contract.CAPTURE_MATURITY,
        "scope": driver.REPORT_SCOPE,
        "pin": pin,
        "accepted_source": copy.deepcopy(pin["accepted_source"]),
        "provenance": {
            "pin_sha256": pin_sha256,
            "submission_receipt": stager.submission_receipt(
                campaign_record, pin),
            "harness_inputs": copy.deepcopy(pin["inputs"]),
            "campaign_record": copy.deepcopy(campaign_record),
        },
        "source": {
            "files": stager.G2_SOURCE["files"],
            "manifest_sha256":
                stager.G2_SOURCE["source_manifest_sha256"],
            "manifest": manifest,
        },
        "binaries": {
            "rust": {"sha256": RUST_SHA256, "bytes": RUST_BYTES},
            "oracle": {
                "sha256": ORACLE_SHA256,
                "bytes": ORACLE_BYTES,
                "commit": ORACLE_COMMIT,
            },
        },
        "catalog": {
            "artifact": catalog_artifact,
            "cases": 2,
            "fresh_processes": 4,
        },
        "commands": command_records(root),
        "invocations": invocations,
        "captures": capture_results(invocations, raw_by_id),
        "complete": True,
        "acceptance_eligible": False,
        "math_gate_released": False,
        "cache_gate_released": False,
        "performance_gate_released": False,
        "rank_gate_released": False,
        "integrity_rechecked": True,
        "source_integrity_rechecked": True,
        "ephemeral_workspace_removed": True,
        "limitations": list(driver.LIMITATIONS),
        "legacy_path_open_attempts": [],
        "thread_settings": {
            "RAYON_NUM_THREADS": "1",
            "OMP_NUM_THREADS": "1",
            "OPENBLAS_NUM_THREADS": "1",
        },
        "environment": driver.command_environment({
            "PATH": "/toolchain/bin",
            "HOME": "/home/capture",
            "LD_LIBRARY_PATH": "/toolchain/lib",
        }, root / "workspace"),
    }
    return report


class WeylContextCoreCaptureTests(unittest.TestCase):
    def assert_rejected_report(self, report, root):
        with self.assertRaises(ValueError):
            driver.validate_report(report, root)

    def test_deferred_globals_are_bound_and_legacy_helpers_absent(self):
        banned = {
            "artifact", "classify", "load_cases", "math_suite", "observe",
            "stage_merged_math", "streams", "verified_gates",
            "weyl_context_contract",
        }
        self.assertTrue(banned.isdisjoint(vars(driver)))
        for module in (driver, stager):
            for value in vars(module).values():
                if (isinstance(value, types.FunctionType)
                        and value.__module__ == module.__name__):
                    target = inspect.unwrap(value)
                    self.assertEqual(
                        unresolved_globals(target.__code__, target.__globals__),
                        set(), target.__name__)

    def test_path_audit_precedes_every_project_import(self):
        source = Path(driver.__file__).read_text()
        tree = ast.parse(source)
        hooks = [
            node.lineno for node in ast.walk(tree)
            if isinstance(node, ast.Call)
            and isinstance(node.func, ast.Attribute)
            and isinstance(node.func.value, ast.Name)
            and node.func.value.id == "sys"
            and node.func.attr == "addaudithook"
        ]
        self.assertEqual(len(hooks), 1)
        project_modules = {
            "campaign_blob", "campaign_source", "campaign_workspace",
            "progressive_submit", "stage_weyl_context_core_capture",
            "weyl_context_core_contract", "weyl_context_core_regression",
            "weyl_parent_seal",
        }
        imports = []
        for node in ast.walk(tree):
            if isinstance(node, ast.Import):
                names = {alias.name.split(".", 1)[0] for alias in node.names}
            elif isinstance(node, ast.ImportFrom) and node.module:
                names = {node.module.split(".", 1)[0]}
            else:
                continue
            if names & project_modules:
                imports.append(node.lineno)
        self.assertTrue(imports)
        self.assertTrue(all(line > hooks[0] for line in imports))
        self.assertTrue(driver.forbidden_legacy_path(
            "/public/home/majj/atlas-obsolete-stage"))
        self.assertFalse(driver.forbidden_legacy_path(
            "/public/home/majj/atlas-rust-campaign-20260930/stages/current"))

    def test_compute_node_guard_precedes_result_or_source_creation(self):
        with patch.object(driver, "SUBMISSION_ENABLED", True), \
                patch.object(driver, "STAGER_SUBMISSION_ENABLED", True), \
                patch.dict(driver.os.environ, {}, clear=True), \
                patch.object(driver, "create_result_folder") as create, \
                patch.object(
                    driver, "materialize_source_archive") as materialize:
            with self.assertRaises(SystemExit):
                driver.main()
        create.assert_not_called()
        materialize.assert_not_called()

    def test_stager_uses_receipt_bound_creator_before_submission(self):
        source = Path(stager.__file__).read_text()
        tree = ast.parse(source)
        functions = {
            node.name: node for node in tree.body
            if isinstance(node, ast.FunctionDef)
        }
        self.assertIn("run_enabled", functions)
        self.assertIn("submit_pinned", functions)

        def call_name(node):
            if isinstance(node.func, ast.Name):
                return node.func.id
            if isinstance(node.func, ast.Attribute):
                return node.func.attr
            return None

        run_calls = sorted(
            (node.lineno, call_name(node))
            for node in ast.walk(functions["run_enabled"])
            if isinstance(node, ast.Call)
        )
        creator_lines = [
            line for line, name in run_calls if name == "create_fixed_stage"
        ]
        submit_lines = [
            line for line, name in run_calls if name == "submit_pinned"
        ]
        self.assertEqual(len(creator_lines), 1)
        self.assertEqual(len(submit_lines), 1)
        self.assertLess(creator_lines[0], submit_lines[0])
        self.assertFalse([
            line for line, name in run_calls
            if name in {"mkdir", "makedirs", "copytree", "rename", "replace"}
        ])

        submit_calls = [
            node for node in ast.walk(functions["submit_pinned"])
            if isinstance(node, ast.Call) and call_name(node) == "submit_one"
        ]
        self.assertEqual(len(submit_calls), 1)
        self.assertIn(
            "stage_creation_sha256",
            {keyword.arg for keyword in submit_calls[0].keywords},
        )

    def test_stage_identity_catalog_and_test_counts_are_exact(self):
        self.assertEqual(driver.STAGE_NAME, stager.STAGE_NAME)
        self.assertEqual(driver.STAGE_NAME,
                         "weyl-context-g2-v2")
        self.assertEqual(driver.PIN_NAME, stager.PIN_NAME)
        self.assertEqual(driver.PIN_NAME,
                         "weyl-context-g2-v2-pin.json")
        self.assertEqual(driver.PIN_SCHEMA, stager.PIN_SCHEMA)
        self.assertEqual(driver.PIN_SCHEMA,
                         "atlas-weyl-context-g2-pin-v2")
        self.assertEqual(driver.SBATCH, stager.SBATCH)
        self.assertEqual(driver.CATALOG_PATH, stager.CATALOG_PATH)
        self.assertEqual(driver.CATALOG_SHA256, stager.CATALOG_SHA256)
        self.assertEqual(driver.REPORT_SCHEMA,
                         "atlas-weyl-context-g2-v2")
        self.assertEqual(driver.SUCCESS_STATUS,
                         "WEYL_CONTEXT_G2_CAPTURE_COMPLETE")
        self.assertEqual(driver.EXPECTED_TEST_COUNTS,
                         stager.EXPECTED_TEST_COUNTS)
        self.assertEqual(
            driver.EXPECTED_TEST_COUNTS[
                "test-math-weyl-context-core-capture"], 28)
        self.assertEqual(sum(driver.EXPECTED_TEST_COUNTS.values()), 129)
        self.assertEqual(tuple(driver.COMMAND_NAMES[:6]),
                         tuple(stager.EXPECTED_TEST_COUNTS))
        self.assertEqual(len(driver.COMMAND_NAMES), 13)
        self.assertEqual(tuple(driver.COMMAND_CONTRACTS),
                         tuple(driver.COMMAND_NAMES))
        self.assertTrue(all(
            set(driver.COMMAND_CONTRACTS[name]) == {"argv", "cwd"}
            for name in driver.COMMAND_NAMES))
        self.assertEqual(set(driver.REPORT_KEYS), REPORT_KEYS)
        self.assertEqual(set(driver.INVOCATION_KEYS), INVOCATION_KEYS)
        self.assertEqual(set(driver.COMMAND_KEYS), COMMAND_KEYS)
        self.assertEqual(set(driver.ARTIFACT_KEYS), ARTIFACT_KEYS)
        self.assertEqual(set(driver.PROVENANCE_KEYS), PROVENANCE_KEYS)
        self.assertEqual(set(driver.ENVIRONMENT_KEYS), ENVIRONMENT_KEYS)
        self.assertEqual(driver.CATALOG_RECORD, {
            "artifact": {
                "path": "catalog.json",
                "sha256": driver.CATALOG_SHA256,
                "bytes": 1436,
            },
            "cases": 2,
            "fresh_processes": 4,
        })
        self.assertEqual(driver.EXPECTED_COMMAND_EXITS[
            "weyl-context-regressions"], {0})
        self.assertTrue(all(
            exits == {0} for name, exits in driver.EXPECTED_COMMAND_EXITS.items()
            if name != "weyl-context-regressions"
        ))

    def test_invocation_plan_is_exactly_four_in_alternating_order(self):
        plan = driver.invocation_plan(catalog_value())
        self.assertEqual(tuple(driver.EXPECTED_INVOCATIONS), EXPECTED_ORDER)
        self.assertEqual(
            [(row["case_id"], row["engine"]) for row in plan],
            list(EXPECTED_ORDER))
        self.assertEqual([row["sequence"] for row in plan], list(range(4)))
        self.assertTrue(all(set(row) == {
            "sequence", "case_id", "engine", "timeout_seconds"
        } for row in plan))
        self.assertTrue(all(type(row["timeout_seconds"]) is int
                            and row["timeout_seconds"] == 45 for row in plan))
        plan[0]["case_id"] = "mutated"
        self.assertEqual(contract.G2_EXPECTED_CASES[0]["id"],
                         EXPECTED_ORDER[0][0])

    def test_invocation_plan_rejects_changed_catalog(self):
        changes = (
            lambda value: value["cases"].reverse(),
            lambda value: value["cases"][0].update(timeout_seconds=True),
            lambda value: value["cases"][0].update(fixture_sha256="0" * 64),
            lambda value: value.update(evidence_maturity="captured"),
            lambda value: value.update(extra=True),
        )
        for change in changes:
            changed = catalog_value()
            change(changed)
            with self.subTest(changed=changed):
                with self.assertRaises(ValueError):
                    driver.invocation_plan(changed)

    def test_sealed_boundary_fixtures_preserve_accepted_generic_frame(self):
        case_id = "generic_root_ladder_coordinate_boundary"
        fixture = b"accepted fixture body\n"
        stdout = (
            ("MATH_BEGIN " + case_id + "\n").encode()
            + b"accepted oracle body\n"
            + ("MATH_END " + case_id + "\nBye.\n").encode()
        )
        stderr = b""
        raw = {
            "input": (
                ("prints(\"MATH_BEGIN " + case_id + "\")\n").encode()
                + fixture
                + ("\nprints(\"MATH_END " + case_id
                   + "\")\nquit\n").encode()
            ),
            "oracle_stdout": stdout,
            "oracle_stderr": stderr,
            "rust_stdout": b"not used by fixture reconstruction",
            "rust_stderr": b"not used by fixture reconstruction",
        }
        expected = {
            driver.BOUNDARY_FIXTURE: fixture,
            driver.BOUNDARY_STDOUT: stdout,
            driver.BOUNDARY_STDERR: stderr,
        }
        sealed_hashes = {
            name: hashlib.sha256(value).hexdigest()
            for name, value in expected.items()
        }
        campaign = Path("/campaign")
        seal = {"sealed": True}
        with patch.object(
                driver, "boundary_bytes", return_value=raw) as boundary, \
                patch.object(
                    driver, "SEALED_FIXTURE_HASHES", sealed_hashes):
            self.assertEqual(
                driver._sealed_boundary_fixtures(campaign, seal), expected)
        boundary.assert_called_once_with(campaign, seal)

    def test_invocation_records_require_fresh_unique_process_groups(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            records, _ = invocation_records(root)
            self.assertIs(driver.validate_invocation_records(records), records)
            self.assertIs(driver.validate_invocation_records(records, root),
                          records)
            self.assertEqual([row["pid"] for row in records],
                             [row["process_group_id"] for row in records])
            self.assertEqual(len({row["pid"] for row in records}), 4)
            self.assertTrue(all(row["fresh_process"] is True for row in records))
            for key, value in (
                    ("fresh_process", False),
                    ("pid", True),
                    ("pid", 0),
                    ("process_group_id", -1),
                    ("invocation_id", ""),
                    ("executable_sha256", ""),
                    ("executable_bytes", 0),
                    ("executable_bytes", True)):
                changed = copy.deepcopy(records)
                changed[0][key] = value
                with self.subTest(key=key, value=value):
                    with self.assertRaises(ValueError):
                        driver.validate_invocation_records(changed)
            changed = copy.deepcopy(records)
            changed[0]["process_group_id"] += 1
            with self.assertRaises(ValueError):
                driver.validate_invocation_records(changed)
            changed = copy.deepcopy(records)
            changed[0]["executable_sha256"] = "0" * 64
            with self.assertRaises(ValueError):
                driver.validate_invocation_records(changed)

    def test_invocation_records_reject_missing_duplicate_or_reordered_rows(self):
        with tempfile.TemporaryDirectory() as directory:
            records, _ = invocation_records(Path(directory))
            mutations = [
                records[:-1],
                [records[0], records[0], records[2], records[3]],
                [records[1], records[0], records[2], records[3]],
                copy.deepcopy(records),
                copy.deepcopy(records),
                copy.deepcopy(records),
            ]
            mutations[3][1]["sequence"] = 0
            mutations[4][1]["invocation_id"] = mutations[0][0]["invocation_id"]
            mutations[5][1]["pid"] = mutations[5][0]["pid"]
            mutations[5][1]["process_group_id"] = mutations[5][0]["pid"]
            for changed in mutations:
                with self.subTest(changed=changed):
                    with self.assertRaises(ValueError):
                        driver.validate_invocation_records(changed)

    def test_invocation_resources_fail_closed(self):
        mutations = (
            ("timed_out", True),
            ("termination_uncertain", True),
            ("signal", 9),
            ("exit_status", 124),
            ("exit_status", True),
        )
        with tempfile.TemporaryDirectory() as directory:
            records, _ = invocation_records(Path(directory))
            for key, value in mutations:
                changed = copy.deepcopy(records)
                changed[0]["observation"][key] = value
                with self.subTest(key=key, value=value):
                    with self.assertRaises(ValueError):
                        driver.validate_invocation_records(changed)

        self.assertEqual(
            tuple(inspect.signature(driver._capture_record).parameters),
            ("out", "plan", "executables", "binary_records", "scripts",
             "env", "input_bytes", "metrics_path", "active", "job"),
        )
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            out = root / "results"
            scripts = root / "scripts"
            out.mkdir()
            scripts.mkdir()
            cargo_binary = root / "target/release/atlas-cli"
            cargo_peer = root / "target/release/deps/atlas-cli-built"
            cargo_peer.parent.mkdir(parents=True)
            cargo_binary.write_bytes(b"rust executable bytes")
            cargo_binary.chmod(0o755)
            os.link(cargo_binary, cargo_peer)
            rust, rust_identity = driver._materialize_built_executable(
                cargo_binary, root / "rust-atlas",
            )
            oracle = root / "oracle-atlas"
            oracle.write_bytes(b"oracle executable bytes")
            oracle.chmod(0o555)
            oracle_identity, _ = driver._executable_snapshot(oracle)
            self.assertGreaterEqual(cargo_binary.stat().st_nlink, 2)
            self.assertEqual(rust.stat().st_nlink, 1)
            self.assertEqual(rust.stat().st_mode & 0o777, 0o555)

            real_open = os.open
            opened = []

            def fail_reopen(path, flags, *arguments, **keywords):
                if not opened:
                    descriptor = real_open(path, flags, *arguments, **keywords)
                    opened.append(descriptor)
                    return descriptor
                raise OSError("forced pathname reopen failure")

            with patch.object(driver.os, "open", side_effect=fail_reopen):
                with self.assertRaises(ValueError):
                    driver._open_executable(oracle)
            self.assertEqual(len(opened), 1)
            with self.assertRaises(OSError):
                os.fstat(opened[0])

            executables = {"rust": rust, "oracle": oracle}
            binary_records = {
                "rust": dict(rust_identity),
                "oracle": {
                    **oracle_identity,
                    "commit": ORACLE_COMMIT,
                },
            }
            plan = {
                "sequence": 0,
                "case_id": EXPECTED_ORDER[0][0],
                "engine": "rust",
                "timeout_seconds": 45,
            }
            result = {
                "pid": 2000,
                "process_group_id": 2000,
                "exit_status": 0,
                "timed_out": False,
                "termination_uncertain": False,
                "signal": None,
                "seconds": 1.0,
                "measured_seconds": 1.0,
                "user_cpu_seconds": 0.5,
                "system_cpu_seconds": 0.25,
                "maxrss_kb": 4096,
                "maxrss_approximate": False,
                "stdout_raw": b"stdout\n",
                "stderr_raw": b"",
                "time_raw": b"time\n",
            }
            observed_descriptor = {}

            def observe_execute(*arguments, **keywords):
                descriptor = keywords["pass_fds"][0]
                observed_descriptor["value"] = descriptor
                os.fstat(descriptor)
                self.assertEqual(arguments[1], scripts)
                self.assertEqual(arguments[2], {})
                self.assertEqual(arguments[3], b"quit\n")
                return result

            with patch.object(
                    driver, "_execute_timed",
                    side_effect=observe_execute) as execute:
                record = driver._capture_record(
                    out, plan, executables, binary_records, scripts, {},
                    b"quit\n", root / "capture.time", {}, "12345",
                )
            argv = execute.call_args[0][0]
            descriptor = execute.call_args.kwargs["pass_fds"][0]
            self.assertEqual(descriptor, observed_descriptor["value"])
            with self.assertRaises(OSError):
                os.fstat(descriptor)
            self.assertEqual(argv, [
                "/bin/bash", "--noprofile", "--norc", "-c",
                'exec -a "$1" "/proc/self/fd/$2"',
                "atlas-capture-exec", str(rust), str(descriptor),
            ])
            self.assertEqual(record["executable_sha256"],
                             rust_identity["sha256"])
            self.assertEqual(record["executable_bytes"],
                             rust_identity["bytes"])

            probe = root / "fd-chain-true"
            driver._write_immutable(
                probe, Path("/bin/true").read_bytes(), mode=0o555,
            )
            probe_descriptor, _, _ = driver._open_executable(probe)
            probe_active = {"process": None, "process_group": None}
            try:
                probe_result = driver._execute_timed(
                    [
                        "/bin/bash", "--noprofile", "--norc", "-c",
                        'exec -a "$1" "/proc/self/fd/$2"',
                        "atlas-capture-exec", str(probe),
                        str(probe_descriptor),
                    ],
                    scripts,
                    {"PATH": "/usr/bin:/bin", "LC_ALL": "C"},
                    b"", 30, root / "fd-chain.time", probe_active,
                    pass_fds=(probe_descriptor,),
                )
            finally:
                os.close(probe_descriptor)
            self.assertEqual(probe_result["exit_status"], 0)
            self.assertFalse(probe_result["timed_out"])
            self.assertFalse(probe_result["termination_uncertain"])
            self.assertEqual(probe_result["signal"], None)
            self.assertEqual(
                probe_active, {"process": None, "process_group": None},
            )

            for key, value in (
                    ("sha256", "0" * 64),
                    ("bytes", rust_identity["bytes"] + 1)):
                changed_records = copy.deepcopy(binary_records)
                changed_records["rust"][key] = value
                with patch.object(driver, "_execute_timed") as execute:
                    with self.assertRaises(ValueError):
                        driver._capture_record(
                            out, plan, executables, changed_records, scripts,
                            {}, b"quit\n", root / ("changed-" + key + ".time"),
                            {}, "12345",
                        )
                    execute.assert_not_called()

            swapped = {"rust": oracle, "oracle": rust}
            with patch.object(driver, "_execute_timed") as execute:
                with self.assertRaises(ValueError):
                    driver._capture_record(
                        out, plan, swapped, binary_records, scripts, {},
                        b"quit\n", root / "swapped.time", {}, "12345",
                    )
                execute.assert_not_called()

            symbolic = root / "symbolic-atlas"
            symbolic.symlink_to(rust)
            with self.assertRaises(ValueError):
                driver._executable_snapshot(symbolic)
            linked = root / "linked-atlas"
            os.link(rust, linked)
            with self.assertRaises(ValueError):
                driver._executable_snapshot(rust)
            linked.unlink()

            changed_out = root / "changed-results"
            changed_out.mkdir()

            def mutate_binary(*_arguments, **_keywords):
                rust.chmod(0o755)
                rust.write_bytes(b"changed executable bytes")
                rust.chmod(0o555)
                return result

            with patch.object(
                    driver, "_execute_timed", side_effect=mutate_binary):
                with self.assertRaises(ValueError):
                    driver._capture_record(
                        changed_out, plan, executables, binary_records,
                        scripts, {}, b"quit\n", root / "mutated.time", {},
                        "12345",
                    )

    def test_invocation_metrics_must_be_exact_finite_and_nonnegative(self):
        v3_first_checker_time = (
            b'\tCommand being timed: "'
            b"/public/software/anaconda/anaconda3-2022.5/bin/python3.9 "
            b"-I -S -B -m unittest discover -s hpc -p "
            b'test_campaign_stage_creation.py -v"\n'
            b"\tUser time (seconds): 3.38\n"
            b"\tSystem time (seconds): 5.19\n"
            b"\tPercent of CPU this job got: 30%\n"
            b"\tElapsed (wall clock) time (h:mm:ss or m:ss): 0:28.52\n"
            b"\tAverage shared text size (kbytes): 0\n"
            b"\tAverage unshared data size (kbytes): 0\n"
            b"\tAverage stack size (kbytes): 0\n"
            b"\tAverage total size (kbytes): 0\n"
            b"\tMaximum resident set size (kbytes): 29096\n"
            b"\tAverage resident set size (kbytes): 0\n"
            b"\tMajor (requiring I/O) page faults: 49\n"
            b"\tMinor (reclaiming a frame) page faults: 54468\n"
            b"\tVoluntary context switches: 128977\n"
            b"\tInvoluntary context switches: 5\n"
            b"\tSwaps: 0\n"
            b"\tFile system inputs: 9928\n"
            b"\tFile system outputs: 24928\n"
            b"\tSocket messages sent: 0\n"
            b"\tSocket messages received: 0\n"
            b"\tSignals delivered: 0\n"
            b"\tPage size (bytes): 4096\n"
            b"\tExit status: 0\n"
        )
        self.assertEqual(len(v3_first_checker_time), 860)
        self.assertEqual(
            hashlib.sha256(v3_first_checker_time).hexdigest(),
            "624f8ea4714c371560f51da4dede4ec55afdd05e6cf05c4c6c433ec25acad4b2",
        )
        self.assertEqual(driver._parse_time(v3_first_checker_time), {
            "seconds": 28.52,
            "user_cpu_seconds": 3.38,
            "system_cpu_seconds": 5.19,
            "maxrss_kb": 29096,
            "exit_status": 0,
            "signal": None,
        })

        mutations = (
            ("seconds", float("nan")),
            ("seconds", -0.1),
            ("user_cpu_seconds", float("inf")),
            ("system_cpu_seconds", "0"),
            ("maxrss_kb", -1),
            ("maxrss_kb", True),
            ("maxrss_approximate", True),
        )
        with tempfile.TemporaryDirectory() as directory:
            records, _ = invocation_records(Path(directory))
            for key, value in mutations:
                changed = copy.deepcopy(records)
                changed[0]["observation"][key] = value
                with self.subTest(key=key, value=value):
                    with self.assertRaises(ValueError):
                        driver.validate_invocation_records(changed)

    def test_raw_artifacts_are_relative_rehashed_and_sized(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            records, _ = invocation_records(root)
            driver.validate_invocation_records(records, root)
            expected_inputs = {
                "weyl_context_g2_cold_dual": (
                    "9ea25f2d0f38c79d439d210ee2e467f10b74891b0593e71bd2aad440f3b82e51",
                    2025,
                ),
                "weyl_context_g2_prewarmed_dual": (
                    "3907ff5b0deb8ffde2ef2b12d20b0132b3831b98674e42bdb62d67a47fcba44f",
                    1561,
                ),
            }
            for row in records:
                self.assertEqual(
                    row["executable_sha256"],
                    ORACLE_SHA256 if row["engine"] == "oracle"
                    else RUST_SHA256)
                self.assertEqual(
                    (row["input"]["sha256"], row["input"]["bytes"]),
                    expected_inputs[row["case_id"]])
                for name in ("input", "stdout", "stderr", "time"):
                    reference = row[name]
                    self.assertEqual(set(reference), ARTIFACT_KEYS)
                    self.assertFalse(PurePosixPath(reference["path"]).is_absolute())
                    raw = (root / reference["path"]).read_bytes()
                    self.assertEqual(len(raw), reference["bytes"])
                    self.assertEqual(hashlib.sha256(raw).hexdigest(),
                                     reference["sha256"])

    def test_raw_artifact_missing_symlink_or_mutation_fails_closed(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            records, _ = invocation_records(root)
            target = root / records[0]["input"]["path"]
            target.chmod(0o600)
            target.write_bytes(target.read_bytes() + b"changed")
            target.chmod(0o444)
            with self.assertRaises(ValueError):
                driver.validate_invocation_records(records, root)

            records, _ = invocation_records(root)
            (root / records[1]["stderr"]["path"]).unlink()
            with self.assertRaises(ValueError):
                driver.validate_invocation_records(records, root)

            records, _ = invocation_records(root)
            link = root / records[2]["input"]["path"]
            actual = link.with_suffix(".real")
            link.rename(actual)
            link.symlink_to(actual.name)
            with self.assertRaises(ValueError):
                driver.validate_invocation_records(records, root)
            link.unlink()
            actual.rename(link)

            records, _ = invocation_records(root)
            records[3]["input"]["path"] = "/absolute/mutable.atlas"
            with self.assertRaises(ValueError):
                driver.validate_invocation_records(records, root)

            records, _ = invocation_records(root)
            writable = root / records[0]["stdout"]["path"]
            writable.chmod(0o644)
            with self.assertRaises(ValueError):
                driver.validate_invocation_records(records, root)
            writable.chmod(0o444)

            records, _ = invocation_records(root)
            original = root / records[1]["stdout"]["path"]
            hardlink = original.with_suffix(".hardlink")
            os.link(original, hardlink)
            with self.assertRaises(ValueError):
                driver.validate_invocation_records(records, root)
            hardlink.unlink()

    def test_complete_report_is_capture_only_and_releases_no_gate(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            report = valid_report(root)
            self.assertIs(driver.validate_report(report, root), report)
            self.assertEqual(set(report), REPORT_KEYS)
            self.assertEqual(report["evidence_maturity"],
                             contract.CAPTURE_MATURITY)
            self.assertTrue(report["complete"])
            self.assertTrue(report["integrity_rechecked"])
            self.assertTrue(report["source_integrity_rechecked"])
            self.assertTrue(report["ephemeral_workspace_removed"])
            self.assertFalse(report["acceptance_eligible"])
            self.assertFalse(report["math_gate_released"])
            self.assertFalse(report["cache_gate_released"])
            self.assertFalse(report["performance_gate_released"])
            self.assertFalse(report["rank_gate_released"])
            self.assertEqual(len(report["invocations"]), 4)
            self.assertEqual(len(report["captures"]), 2)
            self.assertEqual(
                len(report["commands"]), len(driver.COMMAND_NAMES))
            self.assertEqual(report["catalog"], driver.CATALOG_RECORD)
            self.assertEqual(
                [capture["status"] for capture in report["captures"]],
                ["SOURCE_PREDICTIONS_OBSERVED_UNREVIEWED"] * 2)
            self.assertTrue(all(
                capture["full_stdout_equal"] and capture["exit_status_equal"]
                and capture["full_stderr_equal"]
                for capture in report["captures"]))
            self.assertEqual(
                tuple(row["name"] for row in report["commands"]),
                tuple(driver.COMMAND_NAMES))
            command_by_name = {
                row["name"]: row for row in report["commands"]
            }
            self.assertEqual(command_by_name["rustc-version"]["argv"],
                             ["rustc", "-vV"])
            self.assertEqual(command_by_name["cargo-version"]["argv"],
                             ["cargo", "-vV"])
            self.assertEqual(command_by_name["source-reconstruction"]["cwd"],
                             "workspace")
            self.assertTrue(command_by_name[
                "source-reconstruction"]["maxrss_approximate"])
            for name, record in command_by_name.items():
                self.assertEqual(record["argv"],
                                 driver.COMMAND_CONTRACTS[name]["argv"])
                self.assertEqual(record["cwd"],
                                 driver.COMMAND_CONTRACTS[name]["cwd"])
            for name in set(driver.COMMAND_NAMES) - {"source-reconstruction"}:
                self.assertFalse(command_by_name[name]["maxrss_approximate"])
            self.assertEqual(report["thread_settings"], {
                "RAYON_NUM_THREADS": "1",
                "OMP_NUM_THREADS": "1",
                "OPENBLAS_NUM_THREADS": "1",
            })
            self.assertEqual(set(report["environment"]), ENVIRONMENT_KEYS)
            self.assertEqual(
                driver.validate_environment_record(report["environment"]),
                report["environment"])
            self.assertEqual(report["environment"]["CARGO_TARGET_DIR"],
                             str(root / "workspace" / "target"))
            self.assertEqual(report["environment"]["TMPDIR"],
                             str(root / "workspace" / "tmp"))
            self.assertEqual(report["legacy_path_open_attempts"], [])
            self.assertEqual(report["limitations"], list(driver.LIMITATIONS))
            self.assertEqual(set(report["provenance"]), PROVENANCE_KEYS)
            self.assertEqual(
                report["provenance"]["pin_sha256"],
                stager.saved_json_sha(report["pin"]))
            self.assertEqual(report["provenance"]["harness_inputs"],
                             report["pin"]["inputs"])
            self.assertEqual(
                report["provenance"]["campaign_record"]["job"],
                report["job"])
            self.assertEqual(
                report["provenance"]["campaign_record"]["stage"],
                driver.EXPECTED_STAGE)
            self.assertEqual(
                report["provenance"]["submission_receipt"],
                stager.submission_receipt(
                    report["provenance"]["campaign_record"], report["pin"]))
            for record in report["commands"]:
                for name in ("stdout", "stderr", "time"):
                    reference = record[name]
                    self.assertEqual(set(reference), ARTIFACT_KEYS)
                    raw = (root / reference["path"]).read_bytes()
                    self.assertEqual(reference["bytes"], len(raw))
                    self.assertEqual(
                        reference["sha256"],
                        hashlib.sha256(raw).hexdigest())
            self.assertTrue(all(
                row["observation"]["maxrss_approximate"] is False
                for row in report["invocations"]))
            self.assertEqual(report["source"]["files"], 1569)
            self.assertEqual(len(report["source"]["manifest"]), 1569)
            self.assertEqual(
                stager.canonical_json_sha(report["source"]["manifest"]),
                "dff0e90d2830ba3e5dfd95cee950ba47f7683a20fe829c43850ee77095ca34bd")
            self.assertEqual(report["source"]["manifest_sha256"],
                             stager.G2_SOURCE[
                                 "source_manifest_sha256"])
            self.assertEqual(report["binaries"]["oracle"]["sha256"],
                             ORACLE_SHA256)
            self.assertEqual(report["binaries"]["oracle"]["commit"],
                             ORACLE_COMMIT)
            self.assertGreater(report["binaries"]["oracle"]["bytes"], 0)
            self.assertEqual(report["binaries"]["rust"]["sha256"],
                             RUST_SHA256)
            self.assertGreater(report["binaries"]["rust"]["bytes"], 0)
            limitations = " ".join(report["limitations"]).lower()
            self.assertIn("driver", limitations)
            self.assertIn("child", limitations)
            self.assertIn("provisional", limitations)
            self.assertIn("performance", limitations)
            self.assertIn("approximate", limitations)
            self.assertIn("speed", limitations)

    def test_report_rejects_status_maturity_completeness_or_release_mutation(self):
        def mutate_command(value, name, **changes):
            matches = [
                record for record in value["commands"]
                if record.get("name") == name
            ]
            self.assertEqual(len(matches), 1)
            matches[0].update(changes)

        def move_campaign_and_recompute_receipt(value):
            provenance = value["provenance"]
            provenance["campaign_record"]["stage"] = (
                "/public/home/majj/atlas-rust-campaign-20990101/stages/"
                "weyl-context-g2-v2"
            )
            provenance["submission_receipt"] = stager.submission_receipt(
                provenance["campaign_record"], value["pin"])

        changes = (
            lambda value: value.update(status="MATH_PASS"),
            lambda value: value.update(evidence_maturity="accepted"),
            lambda value: value.update(complete=False),
            lambda value: value.update(acceptance_eligible=True),
            lambda value: value.update(math_gate_released=True),
            lambda value: value.update(cache_gate_released=True),
            lambda value: value.update(performance_gate_released=True),
            lambda value: value.update(rank_gate_released=True),
            lambda value: value.update(integrity_rechecked=False),
            lambda value: value.update(source_integrity_rechecked=False),
            lambda value: value.update(ephemeral_workspace_removed=False),
            lambda value: value.update(legacy_path_open_attempts=["blocked"]),
            lambda value: value.update(limitations=[]),
            lambda value: value.update(thread_settings={}),
            lambda value: value["environment"].update(
                LD_PRELOAD="/hostile/wrapper.so"),
            lambda value: value["environment"].pop("PATH"),
            lambda value: value["environment"].update(
                CARGO_TARGET_DIR="/durable/target"),
            lambda value: mutate_command(
                value, "source-reconstruction", maxrss_approximate=False),
            lambda value: mutate_command(
                value, "release-build", maxrss_approximate=True),
            lambda value: mutate_command(
                value, "test-campaign-stage-creation",
                argv=["python3", "-m", "unittest"]),
            lambda value: mutate_command(
                value, "test-stager-allowlist", argv=["rustc", "--version"]),
            lambda value: mutate_command(
                value, "rustc-version", cwd="workspace"),
            lambda value: mutate_command(
                value, "cargo-version", argv=["true"]),
            lambda value: mutate_command(
                value, "cargo-version", cwd="workspace/source"),
            lambda value: mutate_command(
                value, "source-reconstruction",
                argv=["cargo", "build", "--release"]),
            lambda value: value["provenance"].update(pin_sha256="0" * 64),
            lambda value: value["provenance"].update(harness_inputs={}),
            lambda value: value["provenance"]["campaign_record"].update(
                job="99999"),
            lambda value: value["provenance"]["submission_receipt"].update(
                job="99999"),
            lambda value: value["captures"][0].update(
                status="RUST_SOURCE_PREDICTION_DIFFERED"),
            lambda value: value["captures"][1].update(
                classification={}),
            move_campaign_and_recompute_receipt,
            lambda value: value.update(extra=True),
        )
        for change in changes:
            with tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                report = valid_report(root)
                change(report)
                with self.subTest(report=report):
                    self.assert_rejected_report(report, root)

    def test_report_rejects_speed_ratio_acceptance_and_review_fields_recursively(self):
        locations = (
            lambda value, key: value.update({key: 1}),
            lambda value, key: value["catalog"].update({key: 1}),
            lambda value, key: value["captures"][0].update({key: 1}),
            lambda value, key: value["commands"][0].update({key: 1}),
        )
        forbidden = (
            "speed_ratio", "speedup", "acceptance_index", "review_evidence",
            "accepted",
        )
        for location in locations:
            for key in forbidden:
                with tempfile.TemporaryDirectory() as directory:
                    root = Path(directory)
                    report = valid_report(root)
                    location(report, key)
                    with self.subTest(key=key, location=location):
                        self.assert_rejected_report(report, root)

    def test_report_requires_exact_catalog_source_and_case_completeness(self):
        changes = (
            lambda value: value["catalog"]["artifact"].update(
                sha256="0" * 64),
            lambda value: value["catalog"].update(cases=1),
            lambda value: value["catalog"].update(fresh_processes=3),
            lambda value: value["catalog"]["artifact"].update(
                path="../catalog.json"),
            lambda value: value.update(accepted_source={}),
            lambda value: value["source"].update(files=1566),
            lambda value: value["source"].update(manifest_sha256="0" * 64),
            lambda value: value["source"]["manifest"].update(
                {"crates/atlas-core/src/lib.rs": "0" * 64}),
            lambda value: value["binaries"]["rust"].update(sha256="0" * 64),
            lambda value: value["binaries"]["rust"].update(bytes=True),
            lambda value: value["binaries"]["oracle"].update(
                sha256="0" * 64),
            lambda value: value["binaries"]["oracle"].update(commit="0" * 40),
            lambda value: value["invocations"][1].update(
                executable_sha256="0" * 64),
            lambda value: value["invocations"][1].update(
                executable_bytes=1),
            lambda value: value["captures"].pop(),
            lambda value: value["invocations"].pop(),
            lambda value: value["commands"].pop(),
            lambda value: value["catalog"]["artifact"].update(
                sha256="0" * 64),
            lambda value: value["catalog"]["artifact"].update(bytes=1),
            lambda value: value["catalog"]["artifact"].update(
                path="renamed-catalog.json"),
        )
        for change in changes:
            with tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                report = valid_report(root)
                change(report)
                with self.subTest(report=report):
                    self.assert_rejected_report(report, root)

    def test_report_recomputes_both_capture_classifications(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            report = valid_report(root)
            self.assertEqual(
                [row["case"]["id"] for row in report["captures"]],
                [row["id"] for row in contract.G2_EXPECTED_CASES])
            report["captures"][0]["arms"]["oracle"]["stdout"]["bytes"] += 1
            self.assert_rejected_report(report, root)

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            report = valid_report(root)
            report["captures"].reverse()
            self.assert_rejected_report(report, root)

    def test_report_accepts_prediction_difference_but_rejects_incomplete_capture(self):
        def replace_raw(report, root, index, stream, raw):
            row = report["invocations"][index]
            row[stream] = artifact(root, row[stream]["path"], raw)
            return raw

        def reclassify(report, root):
            raw_by_id = {
                row["invocation_id"]: (
                    (root / row["stdout"]["path"]).read_bytes(),
                    (root / row["stderr"]["path"]).read_bytes(),
                )
                for row in report["invocations"]
            }
            report["captures"] = capture_results(
                report["invocations"], raw_by_id)
            return report["captures"]

        def assert_capture_only(report):
            self.assertFalse(report["acceptance_eligible"])
            self.assertFalse(report["math_gate_released"])
            self.assertFalse(report["cache_gate_released"])
            self.assertFalse(report["performance_gate_released"])
            self.assertFalse(report["rank_gate_released"])

        self.assertIn(
            "RUST_SOURCE_PREDICTION_DIFFERED",
            driver.COMPLETE_CAPTURE_STATUSES,
        )
        self.assertNotIn(
            "CAPTURE_INCOMPLETE_STREAM", driver.COMPLETE_CAPTURE_STATUSES)
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            report = valid_report(root)
            row = report["invocations"][1]
            path = root / row["stdout"]["path"]
            changed = path.read_bytes().replace(
                b"WG_RECOVERY|727", b"WG_RECOVERY|728")
            replace_raw(report, root, 1, "stdout", changed)
            reclassify(report, root)
            self.assertTrue(report["captures"][0]["stream_complete"])
            self.assertIs(driver.validate_report(report, root), report)
            assert_capture_only(report)

        # The G2 capture has no frozen golden yet: a complete original
        # stream that differs from the provisional source prediction is a
        # recorded prediction difference, not a rejection.
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            report = valid_report(root)
            row = report["invocations"][0]
            path = root / row["stdout"]["path"]
            changed = path.read_bytes().replace(
                b"WG_RECOVERY|727", b"WG_RECOVERY|728")
            replace_raw(report, root, 0, "stdout", changed)
            reclassify(report, root)
            self.assertTrue(report["captures"][0]["stream_complete"])
            self.assertEqual(report["captures"][0]["status"],
                             "ORIGINAL_SOURCE_PREDICTION_DIFFERED")
            self.assertIs(driver.validate_report(report, root), report)
            assert_capture_only(report)

        unknown_complete = (
            (
                1,
                0,
                "rust",
                "RUST_SOURCE_PREDICTION_DIFFERED",
                lambda raw: raw.replace(
                    b"MATH_BEGIN weyl_context_g2_cold_dual\n",
                    b"future rust stdout prefix\n"
                    b"MATH_BEGIN weyl_context_g2_cold_dual\n",
                    1,
                ).replace(
                    b"MATH_END weyl_context_g2_cold_dual\n",
                    b"WG_FUTURE_MARKER|opaque\n"
                    b"future complete payload line\n"
                    b"MATH_END weyl_context_g2_cold_dual\n"
                    b"future rust stdout suffix\n",
                    1,
                ),
                lambda raw: raw.replace(
                    b"Weyl group mismatch",
                    b"Previously unseen Weyl owner diagnostic",
                    1,
                ) + (
                    b"Runtime error at <stdin>:99: Future complete diagnostic\n"
                    b"  | future_expression\n"
                    b"  | ^^^^^^^^^^^^^^^^^\n"
                ),
            ),
            (
                3,
                1,
                "oracle",
                "ORIGINAL_SOURCE_PREDICTION_DIFFERED",
                lambda raw: raw.replace(
                    b"MATH_BEGIN weyl_context_g2_prewarmed_dual\n",
                    b"future original stdout prefix\n"
                    b"MATH_BEGIN weyl_context_g2_prewarmed_dual\n",
                    1,
                ).replace(
                    b"MATH_END weyl_context_g2_prewarmed_dual\n",
                    b"WGN_FUTURE_MARKER|opaque\n"
                    b"future complete original payload line\n"
                    b"MATH_END weyl_context_g2_prewarmed_dual\n"
                    b"future original stdout suffix\n",
                    1,
                ),
                lambda raw: raw.replace(
                    b"Weyl group mismatch",
                    b"Previously unseen original Weyl diagnostic",
                    1,
                ) + (
                    b"Runtime error:\n"
                    b"  Future complete original diagnostic\n"
                    b"Evaluation aborted.\n"
                ),
            ),
        )
        for (index, capture_index, engine, expected_status,
             stdout_change, stderr_change) in unknown_complete:
            with tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                report = valid_report(root)
                row = report["invocations"][index]
                changed_stdout = stdout_change(
                    (root / row["stdout"]["path"]).read_bytes())
                changed_stderr = stderr_change(
                    (root / row["stderr"]["path"]).read_bytes())
                replace_raw(
                    report, root, index, "stdout", changed_stdout)
                replace_raw(
                    report, root, index, "stderr", changed_stderr)
                reclassify(report, root)
                capture = report["captures"][capture_index]
                arm = capture["arms"][engine]
                self.assertTrue(capture["stream_complete"])
                self.assertEqual(capture["status"], expected_status)
                self.assertEqual(arm["stdout"], {
                    "sha256": hashlib.sha256(changed_stdout).hexdigest(),
                    "bytes": len(changed_stdout),
                })
                self.assertEqual(arm["stderr"], {
                    "sha256": hashlib.sha256(changed_stderr).hexdigest(),
                    "bytes": len(changed_stderr),
                })
                self.assertIs(driver.validate_report(report, root), report)
                assert_capture_only(report)

        incomplete_observations = (
            ("process", "fresh_process", False,
             "CAPTURE_INCOMPLETE_FRESH_PROCESS"),
            ("resource", "timed_out", True,
             "CAPTURE_INCOMPLETE_RESOURCE_OR_TIMEOUT"),
            ("metrics", "maxrss_approximate", True,
             "CAPTURE_INCOMPLETE_METRICS"),
        )
        for incomplete, key, value, expected_status in incomplete_observations:
            with tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                report = valid_report(root)
                if incomplete == "process":
                    report["invocations"][0][key] = value
                else:
                    report["invocations"][0]["observation"][key] = value
                reclassify(report, root)
                with self.subTest(incomplete=incomplete):
                    self.assertEqual(report["captures"][0]["status"],
                                     expected_status)
                    assert_capture_only(report)
                    self.assert_rejected_report(report, root)

        case_id = "weyl_context_g2_cold_dual"
        begin = ("MATH_BEGIN " + case_id + "\n").encode()
        end = ("MATH_END " + case_id + "\n").encode()
        malformed_streams = (
            ("missing-delimiter", "stdout", 1, 0,
             lambda raw: raw.replace(end, b"", 1)),
            ("duplicate-delimiter", "stdout", 1, 0,
             lambda raw: raw.replace(begin, begin + begin, 1)),
            ("reversed-delimiters", "stdout", 1, 0,
             lambda raw: end + begin + b"Bye.\n"),
            ("foreign-delimiter", "stdout", 1, 0,
             lambda raw: b"MATH_BEGIN foreign_case\n" + raw),
            ("recognized-diagnostic-truncation", "stderr", 2, 1,
             lambda raw: b"\n".join(raw.splitlines()[:-1]) + b"\n"),
            ("non-utf8", "stdout", 1, 0,
             lambda raw: raw.replace(begin, begin + b"\xff\n", 1)),
            ("missing-diagnostic-final-newline", "stderr", 2, 1,
             lambda raw: raw[:-1]),
        )
        for reason, stream, index, capture_index, mutation in \
                malformed_streams:
            with tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                report = valid_report(root)
                row = report["invocations"][index]
                path = root / row[stream]["path"]
                changed = mutation(path.read_bytes())
                replace_raw(report, root, index, stream, changed)
                reclassify(report, root)
                capture = report["captures"][capture_index]
                arm = capture["arms"]["rust"]
                with self.subTest(reason=reason):
                    self.assertFalse(capture["stream_complete"])
                    self.assertEqual(capture["status"],
                                     "CAPTURE_INCOMPLETE_STREAM")
                    self.assertEqual(arm[stream], {
                        "sha256": hashlib.sha256(changed).hexdigest(),
                        "bytes": len(changed),
                    })
                    assert_capture_only(report)
                    self.assert_rejected_report(report, root)

    def test_command_environment_uses_only_ephemeral_target_and_scrubs_inheritance(self):
        inherited = {
            "PATH": "/toolchain/bin",
            "HOME": "/home/capture",
            "LD_LIBRARY_PATH": "/toolchain/lib",
            "KEEP_ME": "forbidden",
            "LD_PRELOAD": "/hostile/preload.so",
            "DYLD_INSERT_LIBRARIES": "/hostile/dylib",
            "PYTHONPATH": "/hostile/python",
            "PYTHONHOME": "/hostile/python-home",
            "BASH_ENV": "/hostile/bash-env",
            "ENV": "/hostile/sh-env",
            "RUSTC": "/hostile/rustc",
            "RUSTC_WRAPPER": "/hostile/wrapper",
            "RUSTC_WORKSPACE_WRAPPER": "/hostile/workspace-wrapper",
            "CARGO_TARGET_DIR": "/durable/target",
            "CARGO_HOME": "/hostile/cargo",
            "RUSTUP_HOME": "/hostile/rustup",
            "TMPDIR": "/durable/tmp",
            "RUST_TEST_NOCAPTURE": "1",
            "RUSTFLAGS": "forbidden",
            "ATLAS_LEGACY": "forbidden",
        }
        original = dict(inherited)
        work = Path("/node-local/weyl-core")
        environment = driver.command_environment(inherited, work)
        self.assertEqual(inherited, original)
        self.assertEqual(set(environment), ENVIRONMENT_KEYS)
        self.assertEqual(environment["PATH"], "/toolchain/bin")
        self.assertEqual(environment["HOME"], "/home/capture")
        self.assertEqual(environment["LD_LIBRARY_PATH"], "/toolchain/lib")
        self.assertEqual(environment["CARGO_HOME"], "/home/capture/.cargo")
        self.assertEqual(environment["RUSTUP_HOME"], "/home/capture/.rustup")
        self.assertEqual(environment["CARGO_TARGET_DIR"],
                         "/node-local/weyl-core/target")
        self.assertEqual(environment["TMPDIR"],
                         "/node-local/weyl-core/tmp")
        self.assertEqual(environment["TMP"], environment["TMPDIR"])
        self.assertEqual(environment["TEMP"], environment["TMPDIR"])
        self.assertEqual(environment["CARGO_BUILD_JOBS"], "2")
        self.assertEqual(environment["CARGO_INCREMENTAL"], "0")
        self.assertEqual(environment["CARGO_NET_OFFLINE"], "true")
        self.assertEqual(environment["CARGO_TERM_COLOR"], "never")
        self.assertEqual(environment["LC_ALL"], "C")
        self.assertEqual(environment["LANG"], "C")
        self.assertEqual(environment["PYTHONDONTWRITEBYTECODE"], "1")
        self.assertEqual(environment["RAYON_NUM_THREADS"], "1")
        self.assertEqual(environment["OMP_NUM_THREADS"], "1")
        self.assertEqual(environment["OPENBLAS_NUM_THREADS"], "1")
        for key in set(inherited) - ENVIRONMENT_KEYS:
            self.assertNotIn(key, environment)
        self.assertEqual(
            driver.validate_environment_record(environment), environment)
        for mutation in (
                lambda value: value.update(
                    LD_PRELOAD="/hostile/preload.so"),
                lambda value: value.pop("PATH"),
                lambda value: value.update(
                    CARGO_TARGET_DIR="/durable/target"),
                lambda value: value.update(HOME="relative/home")):
            changed = copy.deepcopy(environment)
            mutation(changed)
            with self.assertRaises(ValueError):
                driver.validate_environment_record(changed)
        for missing in ({}, {"PATH": "/bin", "HOME": "/home/capture"}):
            with self.assertRaises(ValueError):
                driver.command_environment(missing, work)

    def test_driver_materializes_source_and_target_only_in_ephemeral_workspace(self):
        source = Path(driver.__file__).read_text()
        self.assertIn("ephemeral_job_workspace", source)
        self.assertIn("materialize_source_archive", source)
        self.assertIn("CARGO_TARGET_DIR", source)
        self.assertNotIn("tempfile.mkdtemp", source)
        self.assertNotIn("TemporaryDirectory(", source)
        self.assertNotIn("/tmp/atlas", source)
        self.assertNotIn("shutil.copytree", source)
        self.assertNotRegex(source, r"root\s*/\s*[\"']target[\"']")
        main_source = inspect.getsource(driver.main)
        workspace_at = main_source.index("ephemeral_job_workspace")
        materialize_at = main_source.index("materialize_source_archive")
        build_at = main_source.index('"release-build"')
        self.assertLess(workspace_at, materialize_at)
        self.assertLess(materialize_at, build_at)

    def test_driver_command_and_process_group_boundaries_are_static(self):
        source = Path(driver.__file__).read_text()
        self.assertIn("subprocess.Popen", source)
        self.assertIn("start_new_session=True", source)
        self.assertIn("close_fds=True", source)
        self.assertIn("os.getpgid", source)
        self.assertNotIn("shell=True", source)
        self.assertIn("COMMAND_TIMEOUT_SECONDS = 1200", source)
        self.assertIn("COMMAND_KILL_AFTER_SECONDS = 30", source)
        self.assertIn("COMMAND_EXIT_GRACE_SECONDS =", source)
        self.assertGreater(driver.COMMAND_EXIT_GRACE_SECONDS, 0)
        self.assertLess(
            driver.COMMAND_EXIT_GRACE_SECONDS,
            driver.COMMAND_KILL_AFTER_SECONDS,
        )
        self.assertEqual(
            tuple(inspect.signature(driver._wait_group_exit).parameters),
            ("process_group", "timeout_seconds"),
        )
        exact_build = [
            "cargo", "build", "--offline", "--locked", "--release", "-p",
            "atlas-cli",
        ]
        exact_inventory = [
            "cargo", "test", "--offline", "--locked", "--release", "-p",
            "atlas-core", "--lib", "--", "--list",
        ]
        exact_selector = [
            "cargo", "test", "--offline", "--locked", "--release", "-p",
            "atlas-core", "--lib", "weyl_context_core_", "--",
            "--test-threads=1", "--nocapture",
        ]
        exact_control = [
            "cargo", "test", "--offline", "--locked", "--release", "-p",
            "atlas-core", "--lib",
            "session::tests::root_ladder_coordinate_boundary_original", "--",
            "--exact", "--test-threads=1", "--nocapture",
        ]
        literal_lists = []
        for node in ast.walk(ast.parse(source)):
            if (isinstance(node, ast.List)
                    and all(isinstance(item, ast.Constant)
                            and isinstance(item.value, str)
                            for item in node.elts)):
                literal_lists.append([item.value for item in node.elts])
        self.assertIn(exact_build, literal_lists)
        self.assertEqual(
            driver.COMMAND_CONTRACTS["atlas-core-test-inventory"]["argv"],
            exact_inventory,
        )
        self.assertEqual(
            driver.COMMAND_CONTRACTS["weyl-context-regressions"]["argv"],
            exact_selector,
        )
        self.assertNotIn("--exact", exact_selector)
        self.assertEqual(
            driver.COMMAND_CONTRACTS["root-ladder-control"]["argv"],
            exact_control,
        )
        self.assertNotIn("cargo fetch", source)
        self.assertNotIn("cargo install", source)
        self.assertEqual(
            tuple(inspect.signature(driver._command_record).parameters),
            ("out", "name", "root", "work", "env", "metrics_path",
             "active"),
        )
        self.assertEqual(
            tuple(inspect.signature(driver._command_failed_checks).parameters),
            ("name", "record"),
        )
        fixed_cwds = {
            "test-campaign-stage-creation": ".",
            "test-progressive-submit": ".",
            "test-weyl-context-core-contract": ".",
            "test-weyl-context-core-regression-contract": ".",
            "test-math-weyl-context-core-capture": ".",
            "test-stager-allowlist": ".",
            "rustc-version": ".",
            "cargo-version": ".",
            "source-reconstruction": "workspace",
            "release-build": "workspace/source",
            "atlas-core-test-inventory": "workspace/source",
            "weyl-context-regressions": "workspace/source",
            "root-ladder-control": "workspace/source",
        }
        self.assertEqual(
            {name: value["cwd"]
             for name, value in driver.COMMAND_CONTRACTS.items()},
            fixed_cwds,
        )
        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory).resolve()
            root = base / "stage"
            out = root / "results" / "12345"
            work = base / "node-local" / "workspace"
            source_dir = work / "source"
            out.mkdir(parents=True)
            source_dir.mkdir(parents=True)
            result = {
                "pid": 3000,
                "process_group_id": 3000,
                "exit_status": 0,
                "timed_out": False,
                "termination_uncertain": False,
                "signal": None,
                "seconds": 1.0,
                "measured_seconds": 1.0,
                "user_cpu_seconds": 0.5,
                "system_cpu_seconds": 0.25,
                "maxrss_kb": 2048,
                "maxrss_approximate": False,
                "stdout_raw": b"ok\n",
                "stderr_raw": b"",
                "time_raw": b"time\n",
            }
            actual_cwds = {
                ".": root,
                "workspace": work,
                "workspace/source": source_dir,
            }
            with patch.object(
                    driver, "_execute_timed", return_value=result) as execute:
                for name in driver.COMMAND_NAMES:
                    execute.reset_mock()
                    record, _ = driver._command_record(
                        out, name, root, work, {}, work / (name + ".time"),
                        {},
                    )
                    arguments = execute.call_args[0]
                    label = fixed_cwds[name]
                    self.assertEqual(arguments[0],
                                     driver.COMMAND_CONTRACTS[name]["argv"])
                    self.assertEqual(arguments[1], actual_cwds[label])
                    self.assertEqual(record["cwd"], label)
                    self.assertNotIn(str(root), json.dumps(record))
                    self.assertNotIn(str(work), json.dumps(record))

            self.assertEqual(
                driver._command_failed_checks("root-ladder-control", record),
                [],
            )
            multiply_invalid = dict(record)
            multiply_invalid.update({
                "exit_status": 1,
                "timed_out": True,
                "termination_uncertain": True,
                "signal": signal.SIGTERM,
                "seconds": -1.0,
                "user_cpu_seconds": float("nan"),
                "system_cpu_seconds": "5.19",
                "maxrss_kb": True,
            })
            self.assertEqual(driver._command_failed_checks(
                "root-ladder-control", multiply_invalid,
            ), [
                "exit_status",
                "timed_out",
                "termination_uncertain",
                "signal",
                "seconds",
                "user_cpu_seconds",
                "system_cpu_seconds",
                "maxrss_kb",
            ])

            expected_failure_out = root / "results" / "expected-failure"
            expected_failure_out.mkdir()
            expected_failure = dict(result, exit_status=101)
            with patch.object(
                    driver, "_execute_timed", return_value=expected_failure):
                with self.assertRaises(driver.CommandRecordFailure) as caught:
                    driver._command_record(
                        expected_failure_out, "weyl-context-regressions",
                        root, work, {}, work / "expected-failure.time", {},
                    )
            # The repaired-source stage admits only a clean pass: the
            # historical expected-failure exit 101 fails closed.
            self.assertEqual(
                caught.exception.evidence["record"]["exit_status"], 101)
            self.assertEqual(caught.exception.failed_checks, ["exit_status"])

            rejected_out = root / "results" / "rejected-command"
            rejected_out.mkdir()
            uncertain_result = dict(result, termination_uncertain=True)
            with patch.object(
                    driver, "_execute_timed",
                    return_value=uncertain_result):
                with self.assertRaises(ValueError) as rejected:
                    driver._command_record(
                        rejected_out, "cargo-version", root, work, {},
                        work / "rejected-command.time", {},
                    )
            self.assertEqual(
                rejected.exception.failed_checks,
                ["termination_uncertain"],
            )
            self.assertIn(
                'failed_checks=["termination_uncertain"]', str(rejected.exception),
            )
            self.assertEqual(
                rejected.exception.evidence["failed_checks"],
                ["termination_uncertain"],
            )
            self.assertEqual(
                rejected.exception.evidence["record"]["name"],
                "cargo-version",
            )
            self.assertTrue(
                rejected.exception.evidence["execution"]
                ["termination_uncertain"],
            )
            self.assertIn(
                'report["failed_command"] = copy.deepcopy(error.evidence)',
                inspect.getsource(driver.main),
            )

            for invalid in (
                    "/absolute/path", "workspace/../outside", "unknown"):
                changed = copy.deepcopy(driver.COMMAND_CONTRACTS["cargo-version"])
                changed["cwd"] = invalid
                with patch.dict(
                        driver.COMMAND_CONTRACTS,
                        {"cargo-version": changed}, clear=False):
                    with patch.object(driver, "_execute_timed") as execute:
                        with self.assertRaises(ValueError):
                            driver._command_record(
                                out, "cargo-version", root, work, {},
                                work / "invalid.time", {},
                            )
                        execute.assert_not_called()

            occupied = {"process": object(), "process_group": 3999}
            with patch.object(driver.subprocess, "Popen") as popen:
                with self.assertRaises(ValueError):
                    driver._execute_timed(
                        ["/bin/true"], root, {}, b"", 1,
                        work / "occupied.time", occupied,
                    )
            popen.assert_not_called()

            class ExitedLeaderProcess:
                pid = 3998
                returncode = 0

                @staticmethod
                def communicate(input=None, timeout=None):
                    return b"", b""

            exited_leader = ExitedLeaderProcess()
            naturally_reaped_active = {
                "process": None,
                "process_group": None,
            }
            with patch.object(
                    driver.subprocess, "Popen", return_value=exited_leader), \
                    patch.object(driver.os, "getpgid", return_value=3998), \
                    patch.object(
                        driver, "_group_alive",
                        side_effect=[True, False]), \
                    patch.object(
                        driver, "_wait_group_exit", return_value=True,
                    ) as wait_for_natural_exit, \
                    patch.object(
                        driver, "_terminate_group",
                    ) as terminate_naturally_reaped:
                naturally_reaped = driver._execute_timed(
                    ["/bin/true"], root, {}, b"", 1,
                    work / "naturally-reaped.time", naturally_reaped_active,
                )
            wait_for_natural_exit.assert_called_once_with(
                3998, driver.COMMAND_EXIT_GRACE_SECONDS,
            )
            terminate_naturally_reaped.assert_not_called()
            self.assertFalse(naturally_reaped["termination_uncertain"])
            self.assertEqual(
                naturally_reaped_active,
                {"process": None, "process_group": None},
            )

            cleanup_active = {"process": None, "process_group": None}
            with patch.object(
                    driver.subprocess, "Popen", return_value=exited_leader), \
                    patch.object(driver.os, "getpgid", return_value=3998), \
                    patch.object(
                        driver, "_group_alive",
                        side_effect=[True, False]), \
                    patch.object(
                        driver, "_wait_group_exit", return_value=False,
                    ) as expired_grace, \
                    patch.object(
                        driver, "_terminate_group", return_value=False,
                    ) as terminate_after_grace:
                cleaned_after_grace = driver._execute_timed(
                    ["/bin/true"], root, {}, b"", 1,
                    work / "cleaned-after-grace.time", cleanup_active,
                )
            expired_grace.assert_called_once_with(
                3998, driver.COMMAND_EXIT_GRACE_SECONDS,
            )
            terminate_after_grace.assert_called_once_with(exited_leader, 3998)
            self.assertTrue(cleaned_after_grace["termination_uncertain"])
            self.assertEqual(
                cleanup_active, {"process": None, "process_group": None},
            )

            class FailedLeaderProcess(ExitedLeaderProcess):
                returncode = 1

            failed_leader = FailedLeaderProcess()
            failed_active = {"process": None, "process_group": None}
            with patch.object(
                    driver.subprocess, "Popen", return_value=failed_leader), \
                    patch.object(driver.os, "getpgid", return_value=3997), \
                    patch.object(
                        driver, "_group_alive", side_effect=[True, False]), \
                    patch.object(driver, "_wait_group_exit") as no_failed_grace, \
                    patch.object(
                        driver, "_terminate_group", return_value=False,
                    ) as terminate_failed_group:
                failed_result = driver._execute_timed(
                    ["/bin/false"], root, {}, b"", 1,
                    work / "failed-leader.time", failed_active,
                )
            no_failed_grace.assert_not_called()
            terminate_failed_group.assert_called_once_with(failed_leader, 3997)
            self.assertEqual(failed_result["exit_status"], 1)
            self.assertTrue(failed_result["termination_uncertain"])

            class TimedOutLeaderProcess(ExitedLeaderProcess):
                returncode = 124
                attempts = 0

                @classmethod
                def communicate(cls, input=None, timeout=None):
                    cls.attempts += 1
                    if cls.attempts == 1:
                        raise subprocess.TimeoutExpired(["/bin/sleep"], timeout)
                    return b"", b""

            timed_out_leader = TimedOutLeaderProcess()
            timeout_active = {"process": None, "process_group": None}
            with patch.object(
                    driver.subprocess, "Popen", return_value=timed_out_leader), \
                    patch.object(driver.os, "getpgid", return_value=3996), \
                    patch.object(driver.os, "killpg"), \
                    patch.object(
                        driver, "_group_alive", side_effect=[True, False]), \
                    patch.object(driver, "_wait_group_exit") as no_timeout_grace, \
                    patch.object(
                        driver, "_terminate_group", return_value=False,
                    ) as terminate_timeout_group:
                timeout_result = driver._execute_timed(
                    ["/bin/sleep"], root, {}, b"", 1,
                    work / "timed-out-leader.time", timeout_active,
                )
            no_timeout_grace.assert_not_called()
            terminate_timeout_group.assert_called_once_with(
                timed_out_leader, 3996,
            )
            self.assertTrue(timeout_result["timed_out"])
            self.assertTrue(timeout_result["termination_uncertain"])

            lingering_active = {"process": None, "process_group": None}
            with patch.object(
                    driver.subprocess, "Popen", return_value=exited_leader), \
                    patch.object(driver.os, "getpgid", return_value=3998), \
                    patch.object(
                        driver, "_group_alive",
                        side_effect=[True, True]), \
                    patch.object(
                        driver, "_wait_group_exit", return_value=False,
                    ) as failed_grace, \
                    patch.object(
                        driver, "_terminate_group", return_value=True,
                    ) as failed_cleanup:
                with self.assertRaisesRegex(
                        RuntimeError, "process group remains alive"):
                    driver._execute_timed(
                        ["/bin/true"], root, {}, b"", 1,
                        work / "lingering.time", lingering_active,
                    )
            failed_grace.assert_called_once_with(
                3998, driver.COMMAND_EXIT_GRACE_SECONDS,
            )
            failed_cleanup.assert_called_once_with(exited_leader, 3998)
            self.assertIs(lingering_active["process"], exited_leader)
            self.assertEqual(lingering_active["process_group"], 3998)

            class BrokenProcess:
                pid = 4000
                returncode = None

                def communicate(self, input=None, timeout=None):
                    raise RuntimeError("forced communicate failure")

            broken = BrokenProcess()
            active = {"process": None, "process_group": None}
            with patch.object(
                    driver.subprocess, "Popen", return_value=broken), \
                    patch.object(driver.os, "getpgid", return_value=4000), \
                    patch.object(
                        driver, "_terminate_group",
                        side_effect=RuntimeError("forced cleanup failure")):
                with self.assertRaises(RuntimeError):
                    driver._execute_timed(
                        ["/bin/true"], root, {}, b"", 1,
                        work / "broken.time", active,
                    )
            self.assertIs(active["process"], broken)
            self.assertEqual(active["process_group"], 4000)

            class ExitedProcess:
                pid = 5000

                @staticmethod
                def poll():
                    return 0

                @staticmethod
                def wait(timeout=None):
                    return 0

            with patch.object(
                    driver, "_group_alive",
                    side_effect=[False, True, True, False]) as alive, \
                    patch.object(driver.os, "killpg") as killpg, \
                    patch.object(
                        driver.time, "monotonic",
                        side_effect=[10.0, 10.0]), \
                    patch.object(driver.time, "sleep") as sleep:
                self.assertTrue(driver._terminate_group(ExitedProcess(), 5000))
            self.assertEqual(alive.call_count, 4)
            killpg.assert_called_once_with(5000, signal.SIGKILL)
            sleep.assert_called_once_with(0.05)

    def test_stager_pin_counts_predecessor_and_source_are_exact(self):
        pin = valid_pin()
        self.assertEqual(set(pin), stager.PIN_KEYS)
        self.assertEqual(stager.validate_pin(pin), pin)
        self.assertEqual(stager.validate_test_counts(
            stager.EXPECTED_TEST_COUNTS), stager.EXPECTED_TEST_COUNTS)
        self.assertEqual(
            stager.EXPECTED_TEST_COUNTS[
                "test-math-weyl-context-core-capture"], 28)
        self.assertEqual(
            stager.EXPECTED_TEST_COUNTS[
                "test-weyl-context-core-regression-contract"], 21)
        self.assertEqual(stager.CHECKER_TESTS, 129)
        self.assertEqual(len(stager.STAGE_INPUT_NAMES), 65)
        self.assertEqual(stager.PREDECESSOR["campaign_ledger_records"], 24)
        self.assertEqual(
            stager.PREDECESSOR["campaign_ledger_sha256"],
            "6bdf33d7c1e4dcad1515632000a966223bed50537f25ebb7cdc672bc02401f99",
        )
        self.assertEqual(stager.PREDECESSOR["job"], "3900050")
        self.assertEqual(stager.PREDECESSOR["stage_tree_sha256"],
                         stager.PREDECESSOR_STATE["stage_tree_sha256"])
        self.assertEqual(
            stager.PREDECESSOR_STATE["stage_tree_sha256"],
            "7096f5f08486e1fc9a8af4cf2af7a7e07b04570d3ea31e23ccb55a84e399bce5",
        )
        self.assertEqual(stager.PREDECESSOR_STATE["stage_tree_files"], 198)
        self.assertEqual(stager.PREDECESSOR_STATE["stage_tree_directories"], 18)
        self.assertEqual(stager.PREDECESSOR_STATE["stage_tree_bytes"], 5222128)
        self.assertEqual(stager.BEFORE_V2_PREDECESSOR["job"], "3884880")
        self.assertEqual(
            stager.BEFORE_V2_PREDECESSOR["campaign_ledger_records"], 17)
        self.assertEqual(stager.BEFORE_V2_PREDECESSOR_STATE["schema"],
                         "atlas-stage-creation-predecessor-v11")
        self.assertEqual(stager.BEFORE_V1_PREDECESSOR["job"], "3884862")
        self.assertEqual(
            stager.BEFORE_V1_PREDECESSOR["campaign_ledger_records"], 16)
        self.assertEqual(stager.BEFORE_V1_PREDECESSOR_STATE["schema"],
                         "atlas-stage-creation-predecessor-v10")
        self.assertEqual(stager.V8_PREDECESSOR["job"], "3884807")
        self.assertEqual(stager.V8_PREDECESSOR["campaign_ledger_records"], 15)
        self.assertEqual(stager.BEFORE_V1_FAILURE_EVIDENCE, {
            "file": (
                "tests/reference/hpc/"
                "math_weyl_context_core_before_v1_failure_2026_10_02.json"
            ),
            "sha256": (
                "4ea80b67f853789f345911f2a61d71610bf37e3a2d99b8c77805cde3497bf427"
            ),
        })
        self.assertEqual(stager.BEFORE_V2_FAILURE_EVIDENCE, {
            "file": (
                "tests/reference/hpc/"
                "math_weyl_context_core_before_v2_failure_2026_10_02.json"
            ),
            "sha256": (
                "ee4cc4db5b4c1bb64d280ea9e47d85ea9ca8b99356312bfce5c312f30359e80f"
            ),
        })
        self.assertEqual(stager.BEFORE_V3_FAILURE_EVIDENCE, {
            "file": (
                "tests/reference/hpc/"
                "math_weyl_context_core_before_v3_failure_2026_10_02.json"
            ),
            "sha256": (
                "fd6dd4ce66b743c2db36429dcb609e67f4e945fc1422b3fd82b24de7bb8654fd"
            ),
        })
        self.assertEqual(
            pin["inputs"][stager.PRIOR_CREATION_FAILURE_EVIDENCE["file"]],
            stager.PRIOR_CREATION_FAILURE_EVIDENCE["sha256"],
        )
        self.assertEqual(
            pin["inputs"][stager.V1_FAILURE_EVIDENCE["file"]],
            stager.V1_FAILURE_EVIDENCE["sha256"],
        )
        self.assertEqual(
            pin["inputs"][stager.V2_FAILURE_EVIDENCE["file"]],
            stager.V2_FAILURE_EVIDENCE["sha256"],
        )
        self.assertEqual(
            pin["inputs"][stager.V3_FAILURE_EVIDENCE["file"]],
            stager.V3_FAILURE_EVIDENCE["sha256"],
        )
        self.assertEqual(
            pin["inputs"][stager.V4_FAILURE_EVIDENCE["file"]],
            stager.V4_FAILURE_EVIDENCE["sha256"],
        )
        self.assertEqual(
            pin["inputs"][stager.V5_FAILURE_EVIDENCE["file"]],
            stager.V5_FAILURE_EVIDENCE["sha256"],
        )
        self.assertEqual(
            pin["inputs"][stager.V6_FAILURE_EVIDENCE["file"]],
            stager.V6_FAILURE_EVIDENCE["sha256"],
        )
        self.assertEqual(
            pin["inputs"][stager.V7_FAILURE_EVIDENCE["file"]],
            stager.V7_FAILURE_EVIDENCE["sha256"],
        )
        self.assertEqual(
            pin["inputs"][stager.BEFORE_V1_FAILURE_EVIDENCE["file"]],
            stager.BEFORE_V1_FAILURE_EVIDENCE["sha256"],
        )
        self.assertEqual(
            pin["inputs"][stager.BEFORE_V2_FAILURE_EVIDENCE["file"]],
            stager.BEFORE_V2_FAILURE_EVIDENCE["sha256"],
        )
        self.assertEqual(
            pin["inputs"][stager.BEFORE_V3_FAILURE_EVIDENCE["file"]],
            stager.BEFORE_V3_FAILURE_EVIDENCE["sha256"],
        )
        self.assertEqual(set(stager.PRIOR_CREATION_FAILURE), {
            "schema", "temporary", "archive", "sha256", "bytes", "mode",
            "destination", "contract_sha256", "transaction",
        })
        repository = Path(__file__).resolve().parents[1]
        failure = stager.validate_capture_v1_failure(
            repository, pin["inputs"])
        self.assertEqual(failure["submission"]["job"], "3884124")
        self.assertEqual(failure["report"]["atlas_invocations"], 0)
        self.assertIn("mathematical comparison", failure["not_reached"])
        for field, changed in (
                (("submission", "job"), "3884125"),
                (("report", "math_gate_released"), True),
                (("failed_gate", "observed_tests"), 30)):
            with self.subTest(failure_field=field), \
                    tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                mutated = copy.deepcopy(failure)
                mutated[field[0]][field[1]] = changed
                raw = (json.dumps(mutated, indent=2, sort_keys=True)
                       + "\n").encode()
                destination = root / stager.V1_FAILURE_EVIDENCE["file"]
                destination.parent.mkdir(parents=True)
                destination.write_bytes(raw)
                destination.chmod(0o444)
                digest = hashlib.sha256(raw).hexdigest()
                changed_inputs = copy.deepcopy(pin["inputs"])
                changed_inputs[stager.V1_FAILURE_EVIDENCE["file"]] = digest
                with patch.object(
                        stager, "V1_FAILURE_EVIDENCE",
                        {"file": stager.V1_FAILURE_EVIDENCE["file"],
                         "sha256": digest}), self.assertRaises(ValueError):
                    stager.validate_capture_v1_failure(root, changed_inputs)
        failure = stager.validate_capture_v2_failure(
            repository, pin["inputs"])
        self.assertEqual(failure["submission"]["job"], "3884371")
        self.assertEqual(failure["report"]["commands_recorded"], 3)
        self.assertEqual(failure["report"]["atlas_invocations"], 0)
        self.assertEqual(failure["failed_gate"]["observed_tests"], 27)
        self.assertFalse(failure["mathematical_regression_required"])
        for field, changed in (
                (("submission", "job"), "3884372"),
                (("report", "math_gate_released"), True),
                (("failed_gate", "passed"), 25)):
            with self.subTest(v2_failure_field=field), \
                    tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                mutated = copy.deepcopy(failure)
                mutated[field[0]][field[1]] = changed
                raw = (json.dumps(mutated, indent=2, sort_keys=True)
                       + "\n").encode()
                destination = root / stager.V2_FAILURE_EVIDENCE["file"]
                destination.parent.mkdir(parents=True)
                destination.write_bytes(raw)
                destination.chmod(0o444)
                digest = hashlib.sha256(raw).hexdigest()
                changed_inputs = copy.deepcopy(pin["inputs"])
                changed_inputs[stager.V2_FAILURE_EVIDENCE["file"]] = digest
                with patch.object(
                        stager, "V2_FAILURE_EVIDENCE",
                        {"file": stager.V2_FAILURE_EVIDENCE["file"],
                         "sha256": digest}), self.assertRaises(ValueError):
                    stager.validate_capture_v2_failure(root, changed_inputs)
        failure = stager.validate_capture_v3_failure(
            repository, pin["inputs"])
        self.assertEqual(failure["submission"]["job"], "3884456")
        self.assertEqual(failure["report"]["commands_recorded"], 0)
        self.assertEqual(failure["report"]["atlas_invocations"], 0)
        self.assertEqual(failure["passed_but_unrecorded_gate"]["tests"], 32)
        self.assertEqual(
            failure["failed_gate"]["independent_diagnosis"]
            ["most_likely_remaining_predicate"],
            "termination_uncertain",
        )
        self.assertFalse(failure["mathematical_regression_required"])
        for field, changed in (
                (("submission", "job"), "3884457"),
                (("report", "math_gate_released"), True),
                (("passed_but_unrecorded_gate", "tests"), 31),
                (("failed_gate", "traceback_driver_line"), 1324)):
            with self.subTest(v3_failure_field=field), \
                    tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                mutated = copy.deepcopy(failure)
                mutated[field[0]][field[1]] = changed
                raw = (json.dumps(mutated, indent=2, sort_keys=True)
                       + "\n").encode()
                destination = root / stager.V3_FAILURE_EVIDENCE["file"]
                destination.parent.mkdir(parents=True)
                destination.write_bytes(raw)
                destination.chmod(0o444)
                digest = hashlib.sha256(raw).hexdigest()
                changed_inputs = copy.deepcopy(pin["inputs"])
                changed_inputs[stager.V3_FAILURE_EVIDENCE["file"]] = digest
                with patch.object(
                        stager, "V3_FAILURE_EVIDENCE",
                        {"file": stager.V3_FAILURE_EVIDENCE["file"],
                         "sha256": digest}), self.assertRaises(ValueError):
                    stager.validate_capture_v3_failure(root, changed_inputs)
        failure = stager.validate_capture_v4_failure(
            repository, pin["inputs"])
        self.assertEqual(failure["submission"]["job"], "3884494")
        self.assertEqual(failure["report"]["commands_recorded"], 3)
        self.assertEqual(failure["report"]["atlas_invocations"], 0)
        self.assertEqual(failure["failed_gate"]["tests_started"], 0)
        self.assertEqual(failure["failed_gate"]["failed_checks"],
                         ["exit_status"])
        self.assertFalse(failure["failed_gate"]["termination_uncertain"])
        self.assertFalse(failure["mathematical_regression_required"])
        for field, changed in (
                (("submission", "job"), "3884495"),
                (("report", "math_gate_released"), True),
                (("failed_gate", "tests_started"), 1),
                (("failed_gate", "termination_uncertain"), True)):
            with self.subTest(v4_failure_field=field), \
                    tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                mutated = copy.deepcopy(failure)
                mutated[field[0]][field[1]] = changed
                raw = (json.dumps(mutated, indent=2, sort_keys=True)
                       + "\n").encode()
                destination = root / stager.V4_FAILURE_EVIDENCE["file"]
                destination.parent.mkdir(parents=True)
                destination.write_bytes(raw)
                destination.chmod(0o444)
                digest = hashlib.sha256(raw).hexdigest()
                changed_inputs = copy.deepcopy(pin["inputs"])
                changed_inputs[stager.V4_FAILURE_EVIDENCE["file"]] = digest
                with patch.object(
                        stager, "V4_FAILURE_EVIDENCE",
                        {"file": stager.V4_FAILURE_EVIDENCE["file"],
                         "sha256": digest}), self.assertRaises(ValueError):
                    stager.validate_capture_v4_failure(root, changed_inputs)
        failure = stager.validate_capture_v5_failure(
            repository, pin["inputs"])
        self.assertEqual(failure["submission"]["job"], "3884727")
        self.assertEqual(failure["report"]["commands_recorded"], 3)
        self.assertEqual(failure["report"]["atlas_invocations"], 0)
        self.assertEqual(failure["failed_gate"]["tests_run"], 27)
        self.assertEqual(failure["failed_gate"]["tests_passed"], 26)
        self.assertTrue(
            failure["failed_gate"]
            ["v3_process_group_remediation_regressions_verified"])
        self.assertFalse(failure["mathematical_regression_required"])
        for field, changed in (
                (("submission", "job"), "3884728"),
                (("report", "math_gate_released"), True),
                (("failed_gate", "tests_passed"), 27),
                (("independent_review", "read_only_audits"), 0)):
            with self.subTest(v5_failure_field=field), \
                    tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                mutated = copy.deepcopy(failure)
                mutated[field[0]][field[1]] = changed
                raw = (json.dumps(mutated, indent=2, sort_keys=True)
                       + "\n").encode()
                destination = root / stager.V5_FAILURE_EVIDENCE["file"]
                destination.parent.mkdir(parents=True)
                destination.write_bytes(raw)
                destination.chmod(0o444)
                digest = hashlib.sha256(raw).hexdigest()
                changed_inputs = copy.deepcopy(pin["inputs"])
                changed_inputs[stager.V5_FAILURE_EVIDENCE["file"]] = digest
                with patch.object(
                        stager, "V5_FAILURE_EVIDENCE",
                        {"file": stager.V5_FAILURE_EVIDENCE["file"],
                         "sha256": digest}), self.assertRaises(ValueError):
                    stager.validate_capture_v5_failure(root, changed_inputs)
        failure = stager.validate_capture_v6_failure(
            repository, pin["inputs"])
        self.assertEqual(failure["submission"]["job"], "3884751")
        self.assertEqual(failure["report"]["atlas_invocations"], 0)
        self.assertFalse(failure["mathematical_regression_required"])
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            mutated = copy.deepcopy(failure)
            mutated["submission"]["job"] = "3884752"
            raw = (json.dumps(mutated, indent=2, sort_keys=True)
                   + "\n").encode()
            destination = root / stager.V6_FAILURE_EVIDENCE["file"]
            destination.parent.mkdir(parents=True)
            destination.write_bytes(raw)
            destination.chmod(0o444)
            digest = hashlib.sha256(raw).hexdigest()
            changed_inputs = copy.deepcopy(pin["inputs"])
            changed_inputs[stager.V6_FAILURE_EVIDENCE["file"]] = digest
            with patch.object(
                    stager, "V6_FAILURE_EVIDENCE",
                    {"file": stager.V6_FAILURE_EVIDENCE["file"],
                     "sha256": digest}), self.assertRaises(ValueError):
                stager.validate_capture_v6_failure(root, changed_inputs)
        failure = stager.validate_capture_v7_failure(
            repository, pin["inputs"])
        self.assertEqual(failure["submission"]["job"], "3884780")
        self.assertEqual(failure["report"]["commands_recorded"], 1)
        self.assertEqual(failure["report"]["atlas_invocations"], 0)
        self.assertEqual(failure["passed_test_gates"][0]["tests"], 32)
        self.assertEqual(failure["failed_gate"]["tests"], 17)
        self.assertEqual(failure["failed_gate"]["passed"], 13)
        self.assertEqual(failure["failed_gate"]["failures"], 0)
        self.assertEqual(failure["failed_gate"]["errors"], 4)
        self.assertEqual(
            failure["failed_gate"]["actual_stage_creation_tests_passed"],
            32,
        )
        self.assertFalse(failure["mathematical_regression_required"])
        for field, changed in (
                (("submission", "job"), "3884781"),
                (("report", "math_gate_released"), True),
                (("failed_gate", "passed"), 14),
                (("failed_gate", "actual_stage_creation_tests_passed"), 31)):
            with self.subTest(v7_failure_field=field), \
                    tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                mutated = copy.deepcopy(failure)
                mutated[field[0]][field[1]] = changed
                raw = (json.dumps(mutated, indent=2, sort_keys=True)
                       + "\n").encode()
                destination = root / stager.V7_FAILURE_EVIDENCE["file"]
                destination.parent.mkdir(parents=True)
                destination.write_bytes(raw)
                destination.chmod(0o444)
                digest = hashlib.sha256(raw).hexdigest()
                changed_inputs = copy.deepcopy(pin["inputs"])
                changed_inputs[stager.V7_FAILURE_EVIDENCE["file"]] = digest
                with patch.object(
                        stager, "V7_FAILURE_EVIDENCE",
                        {"file": stager.V7_FAILURE_EVIDENCE["file"],
                         "sha256": digest}), self.assertRaises(ValueError):
                    stager.validate_capture_v7_failure(root, changed_inputs)
        inspection = stager.validate_capture_v8(repository, pin["inputs"])
        self.assertEqual(inspection["job"]["id"], "3884807")
        self.assertEqual(inspection["job"]["state"], "COMPLETED")
        self.assertEqual(inspection["job"]["outstanding_jobs_after_inspection"],
                         0)
        self.assertEqual(
            inspection["artifacts"]["stage_tree"]["sha256"],
            stager.V8_PREDECESSOR["stage_tree_sha256"],
        )
        before_failure = stager.validate_before_v1_failure(
            repository, pin["inputs"])
        self.assertEqual(before_failure["schema"],
                         "atlas-weyl-context-core-before-failure-v1")
        self.assertEqual(before_failure["submission"]["job"], "3884862")
        self.assertEqual(before_failure["submission"]["state"], "FAILED")
        self.assertEqual(before_failure["submission"]["ledger_records_after"],
                         16)
        self.assertEqual(
            [gate["tests"] for gate in before_failure["passed_test_gates"]],
            [32, 17, 18, 17],
        )
        self.assertEqual(before_failure["failed_gate"]["tests"], 28)
        self.assertEqual(before_failure["failed_gate"]["passed"], 26)
        self.assertEqual(before_failure["failed_gate"]["failures"], 1)
        self.assertEqual(before_failure["failed_gate"]["errors"], 1)
        self.assertEqual(
            before_failure["failed_gate"]["failure"]["test"],
            "test_report_accepts_prediction_difference_but_rejects_"
            "incomplete_capture",
        )
        self.assertEqual(
            before_failure["failed_gate"]["error"]["test"],
            "test_stager_submission_is_prerequisite_bound_single_job_and_"
            "fail_closed",
        )
        self.assertEqual(before_failure["report"]["commands_recorded"], 4)
        self.assertEqual(before_failure["report"]["bytes"], 51145)
        self.assertEqual(
            before_failure["report"]["sha256"],
            "1972be6c1bb893153d7765b3f778dbb96707f1f90ad327f54a07311c2b57c21c",
        )
        self.assertEqual(before_failure["report"]["source_files_recorded"], 0)
        self.assertEqual(before_failure["report"]["binaries_recorded"], 0)
        self.assertEqual(before_failure["report"]["atlas_invocations"], 0)
        self.assertEqual(before_failure["report"]["captures_recorded"], 0)
        self.assertTrue(before_failure["report"]["ephemeral_workspace_removed"])
        self.assertFalse(before_failure["mathematical_regression_required"])
        self.assertTrue(all(
            before_failure["report"][key] is False
            for key in (
                "acceptance_eligible", "math_gate_released",
                "cache_gate_released", "performance_gate_released",
                "rank_gate_released",
            )
        ))
        for field, changed in (
                (("submission", "job"), "3884863"),
                (("report", "math_gate_released"), True),
                (("failed_gate", "passed"), 27)):
            with self.subTest(before_v1_failure_field=field), \
                    tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                mutated = copy.deepcopy(before_failure)
                mutated[field[0]][field[1]] = changed
                raw = (json.dumps(mutated, indent=2, sort_keys=True)
                       + "\n").encode()
                destination = root / stager.BEFORE_V1_FAILURE_EVIDENCE["file"]
                destination.parent.mkdir(parents=True)
                destination.write_bytes(raw)
                destination.chmod(0o444)
                digest = hashlib.sha256(raw).hexdigest()
                changed_inputs = copy.deepcopy(pin["inputs"])
                changed_inputs[
                    stager.BEFORE_V1_FAILURE_EVIDENCE["file"]
                ] = digest
                with patch.object(
                        stager, "BEFORE_V1_FAILURE_EVIDENCE",
                        {
                            "file": stager.BEFORE_V1_FAILURE_EVIDENCE["file"],
                            "sha256": digest,
                        }), self.assertRaises(ValueError):
                    stager.validate_before_v1_failure(root, changed_inputs)
        before_v2_failure = stager.validate_before_v2_failure(
            repository, pin["inputs"])
        self.assertEqual(before_v2_failure["schema"],
                         "atlas-weyl-context-core-before-failure-v2")
        self.assertEqual(before_v2_failure["submission"]["job"], "3884880")
        self.assertEqual(before_v2_failure["submission"]["state"], "FAILED")
        self.assertEqual(
            before_v2_failure["submission"]["ledger_records_after"], 17)
        self.assertEqual(before_v2_failure["test_totals"], {
            "tests": 119, "passed": 117, "failures": 0,
            "errors": 2, "ignored": 0,
        })
        self.assertEqual(
            [gate["tests"] for gate in before_v2_failure["passed_test_gates"]],
            [32, 17, 18, 17, 28],
        )
        self.assertEqual(before_v2_failure["failed_gate"]["name"],
                         "test-stager-allowlist")
        self.assertEqual(before_v2_failure["failed_gate"]["tests"], 7)
        self.assertEqual(before_v2_failure["failed_gate"]["passed"], 5)
        self.assertEqual(before_v2_failure["failed_gate"]["failures"], 0)
        self.assertEqual(before_v2_failure["failed_gate"]["errors"], 2)
        self.assertEqual(
            [record["test"]
             for record in before_v2_failure["failed_gate"]["error_records"]],
            [
                "test_current_filename_and_role_snapshot",
                "test_module_actions_and_canonical_entries_are_bounded",
            ],
        )
        self.assertEqual(
            before_v2_failure["failed_gate"][
                "observed_first_rejected_action"
            ]["assignment"],
            "PREDECESSOR_CAMPAIGN_FILES",
        )
        self.assertIn(
            "dictionary display with **V8_PREDECESSOR_CAMPAIGN_FILES",
            before_v2_failure["failed_gate"][
                "observed_first_rejected_action"
            ]["form"],
        )
        self.assertEqual(
            before_v2_failure["failed_gate"][
                "latent_next_rejected_action"
            ]["assignment"],
            "STAGE_INPUT_NAMES",
        )
        self.assertIn(
            'BEFORE_V1_FAILURE_EVIDENCE["file"]',
            before_v2_failure["failed_gate"][
                "latent_next_rejected_action"
            ]["form"],
        )
        self.assertEqual(before_v2_failure["report"]["commands_recorded"], 5)
        self.assertTrue(before_v2_failure["report"]["failed_command_recorded"])
        self.assertEqual(before_v2_failure["report"]["bytes"], 52518)
        self.assertEqual(
            before_v2_failure["report"]["sha256"],
            "d38a7ace107be965212a46a4edfd54b1327b5429f0893f3a347320cbc553f827",
        )
        self.assertEqual(before_v2_failure["report"]["source_files_recorded"], 0)
        self.assertEqual(before_v2_failure["report"]["binaries_recorded"], 0)
        self.assertEqual(before_v2_failure["report"]["atlas_invocations"], 0)
        self.assertEqual(before_v2_failure["report"]["captures_recorded"], 0)
        self.assertTrue(
            before_v2_failure["report"]["ephemeral_workspace_removed"])
        self.assertFalse(before_v2_failure["mathematical_regression_required"])
        self.assertTrue(all(
            before_v2_failure["report"][key] is False
            for key in (
                "acceptance_eligible", "math_gate_released",
                "cache_gate_released", "performance_gate_released",
                "rank_gate_released",
            )
        ))
        for field, changed in (
                (("submission", "job"), "3884881"),
                (("report", "math_gate_released"), True),
                (("failed_gate", "passed"), 6)):
            with self.subTest(before_v2_failure_field=field), \
                    tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                mutated = copy.deepcopy(before_v2_failure)
                mutated[field[0]][field[1]] = changed
                raw = (json.dumps(mutated, indent=2, sort_keys=True)
                       + "\n").encode()
                destination = root / stager.BEFORE_V2_FAILURE_EVIDENCE["file"]
                destination.parent.mkdir(parents=True)
                destination.write_bytes(raw)
                destination.chmod(0o444)
                digest = hashlib.sha256(raw).hexdigest()
                changed_inputs = copy.deepcopy(pin["inputs"])
                changed_inputs[
                    stager.BEFORE_V2_FAILURE_EVIDENCE["file"]
                ] = digest
                with patch.object(
                        stager, "BEFORE_V2_FAILURE_EVIDENCE",
                        {
                            "file": stager.BEFORE_V2_FAILURE_EVIDENCE["file"],
                            "sha256": digest,
                        }), self.assertRaises(ValueError):
                    stager.validate_before_v2_failure(root, changed_inputs)
        before_v3_failure = stager.validate_before_v3_failure(
            repository, pin["inputs"])
        self.assertEqual(before_v3_failure["schema"],
                         "atlas-weyl-context-core-before-failure-v3")
        self.assertEqual(before_v3_failure["submission"]["job"], "3884903")
        self.assertEqual(before_v3_failure["submission"]["state"], "FAILED")
        self.assertEqual(
            before_v3_failure["submission"]["ledger_records_after"], 18)
        self.assertEqual(before_v3_failure["test_totals"], {
            "tests": 119, "passed": 117, "failures": 0,
            "errors": 2, "ignored": 0,
        })
        self.assertEqual(
            [(gate["name"], gate["tests"], gate["exit_status"])
             for gate in before_v3_failure["passed_test_gates"]],
            [
                ("test-campaign-stage-creation", 32, 0),
                ("test-progressive-submit", 17, 0),
                ("test-weyl-context-core-contract", 18, 0),
                ("test-weyl-context-core-regression-contract", 17, 0),
                ("test-math-weyl-context-core-capture", 28, 0),
            ],
        )
        self.assertEqual(before_v3_failure["failed_gate"]["name"],
                         "test-stager-allowlist")
        self.assertEqual(before_v3_failure["failed_gate"]["tests"], 7)
        self.assertEqual(before_v3_failure["failed_gate"]["passed"], 5)
        self.assertEqual(before_v3_failure["failed_gate"]["failures"], 0)
        self.assertEqual(before_v3_failure["failed_gate"]["errors"], 2)
        self.assertEqual(
            [record["test"]
             for record in before_v3_failure["failed_gate"]["error_records"]],
            [
                "test_current_filename_and_role_snapshot",
                "test_module_actions_and_canonical_entries_are_bounded",
            ],
        )
        self.assertEqual(before_v3_failure["root_cause"], {
            "rejected_launcher": "hpc/math_ladder_boundary_before.py",
            "rejected_node": "top-level campaign guard",
            "rejected_node_lines": [55, 56],
            "historical_driver_actual_message":
                "ladder BEFORE v2 campaign policy changed",
            "allowlist_path": "hpc/test_stager_allowlist.py",
            "allowlist_expected_message_line": 658,
            "allowlist_incorrect_expected_message":
                "ladder BEFORE v3 campaign policy changed",
            "mechanism": (
                "A broad BEFORE-v2 to BEFORE-v3 identifier migration changed "
                "the allowlist's expected literal for the immutable historical "
                "ladder driver. The historical driver correctly retained its "
                "v2 message, so exact_campaign_guard returned false and "
                "bounded_module rejected that top-level If node."
            ),
            "active_v3_stager_non_pure_module_assignments": [],
            "active_v3_driver_non_pure_module_assignments": [],
            "not_a_rust_mathematical_failure": True,
        })
        self.assertEqual(before_v3_failure["report"]["commands_recorded"], 5)
        self.assertTrue(before_v3_failure["report"]["failed_command_recorded"])
        self.assertEqual(before_v3_failure["report"]["bytes"], 52866)
        self.assertEqual(
            before_v3_failure["report"]["sha256"],
            "7918ed04b11d2719b59916b6fe59d146bc6c622e43eac39c5c4f1643d6d571a1",
        )
        self.assertEqual(before_v3_failure["report"]["source_files_recorded"], 0)
        self.assertEqual(before_v3_failure["report"]["binaries_recorded"], 0)
        self.assertEqual(before_v3_failure["report"]["atlas_invocations"], 0)
        self.assertEqual(before_v3_failure["report"]["captures_recorded"], 0)
        self.assertTrue(
            before_v3_failure["report"]["ephemeral_workspace_removed"])
        self.assertFalse(before_v3_failure["mathematical_regression_required"])
        self.assertTrue(all(
            before_v3_failure["report"][key] is False
            for key in (
                "acceptance_eligible", "math_gate_released",
                "cache_gate_released", "performance_gate_released",
                "rank_gate_released",
            )
        ))
        for field, changed in (
                (("submission", "job"), "3884904"),
                (("report", "math_gate_released"), True),
                (("failed_gate", "passed"), 6),
                (("root_cause", "allowlist_incorrect_expected_message"),
                 "ladder BEFORE v2 campaign policy changed")):
            with self.subTest(before_v3_failure_field=field), \
                    tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                mutated = copy.deepcopy(before_v3_failure)
                mutated[field[0]][field[1]] = changed
                raw = (json.dumps(mutated, indent=2, sort_keys=True)
                       + "\n").encode()
                destination = root / stager.BEFORE_V3_FAILURE_EVIDENCE["file"]
                destination.parent.mkdir(parents=True)
                destination.write_bytes(raw)
                destination.chmod(0o444)
                digest = hashlib.sha256(raw).hexdigest()
                changed_inputs = copy.deepcopy(pin["inputs"])
                changed_inputs[
                    stager.BEFORE_V3_FAILURE_EVIDENCE["file"]
                ] = digest
                with patch.object(
                        stager, "BEFORE_V3_FAILURE_EVIDENCE",
                        {
                            "file": stager.BEFORE_V3_FAILURE_EVIDENCE["file"],
                            "sha256": digest,
                        }), self.assertRaises(ValueError):
                    stager.validate_before_v3_failure(root, changed_inputs)
        accepted_report = json.loads(
            (repository / stager.AFTER_REPORT_PATH).read_text())
        regression_manifest = stager.validate_regression_inputs(
            repository, pin["inputs"], accepted_report["source_files"])
        self.assertEqual(len(regression_manifest), 1567)
        self.assertEqual(
            stager.canonical_json_sha(regression_manifest),
            stager.REGRESSION_SOURCE["source_manifest_sha256"],
        )
        g2_manifest = stager.validate_g2_inputs(
            repository, pin["inputs"], accepted_report["source_files"])
        self.assertEqual(len(g2_manifest), 1569)
        self.assertEqual(
            stager.canonical_json_sha(g2_manifest),
            stager.G2_SOURCE["source_manifest_sha256"],
        )
        changed_inputs = copy.deepcopy(pin["inputs"])
        changed_inputs[stager.REGRESSION_CATALOG_PATH] = "0" * 64
        with self.assertRaises(ValueError):
            stager.validate_capture_v8(repository, changed_inputs)
        self.assertEqual(pin["predecessor"], stager.PREDECESSOR)
        self.assertEqual(pin["accepted_source"], stager.ACCEPTED_SOURCE)
        self.assertEqual(pin["regression"], stager.REGRESSION_RECORD)
        self.assertEqual(pin["parent_seal_object"],
                         stager.PARENT_SEAL_REFERENCE)
        self.assertEqual(pin["catalog"]["fresh_processes"], 4)
        self.assertEqual(pin["catalog"]["cases"], stager.CATALOG_CASES)
        absolutes = []

        def visit(value, pointer=""):
            if isinstance(value, dict):
                for key, item in value.items():
                    visit(item, pointer + "/" + key)
            elif isinstance(value, list):
                for index, item in enumerate(value):
                    visit(item, pointer + "/" + str(index))
            elif isinstance(value, str) and value.startswith("/"):
                absolutes.append(pointer)

        visit(pin)
        self.assertEqual(absolutes, ["/predecessor/stage"])
        self.assertRegex(pin["predecessor"]["pin_sha256"], r"^[0-9a-f]{64}$")
        self.assertRegex(pin["predecessor"]["stage_creation_sha256"],
                         r"^[0-9a-f]{64}$")
        self.assertRegex(pin["predecessor"]["submission_receipt_sha256"],
                         r"^[0-9a-f]{64}$")
        for changed in (
                dict(stager.EXPECTED_TEST_COUNTS,
                     **{"test-math-weyl-context-core-capture": 23}),
                dict(stager.EXPECTED_TEST_COUNTS,
                     **{"test-math-weyl-context-core-capture": True})):
            with self.assertRaises(ValueError):
                stager.validate_test_counts(changed)

        with tempfile.TemporaryDirectory() as directory:
            parent = Path(directory) / "parent"
            child = parent / "child"
            child.mkdir(parents=True)
            parent_descriptor = os.open(
                parent, os.O_RDONLY | getattr(os, "O_DIRECTORY", 0))
            parent_identity = (
                parent.stat().st_dev, parent.stat().st_ino)
            synced = []
            real_fsync = os.fsync

            def observe_sync(descriptor):
                value = os.fstat(descriptor)
                synced.append((value.st_dev, value.st_ino))
                return real_fsync(descriptor)

            child_descriptor = None
            try:
                with patch(
                        "stage_weyl_context_core_capture.os.fsync",
                        side_effect=observe_sync):
                    child_descriptor = stager._open_child_directory(
                        parent_descriptor, "child", create=True)
            finally:
                if child_descriptor is not None:
                    os.close(child_descriptor)
                os.close(parent_descriptor)
            self.assertIn(parent_identity, synced)

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "stage"
            incoming = root / ".incoming"
            destination = root / "nested"
            incoming.mkdir(parents=True)
            destination.mkdir()
            raw = b"already linked and immutable\n"
            target = destination / "input.txt"
            target.write_bytes(raw)
            target.chmod(0o444)
            destination_identity = (
                destination.stat().st_dev, destination.stat().st_ino)
            synced = []
            real_fsync = os.fsync

            def observe_install_sync(descriptor):
                value = os.fstat(descriptor)
                synced.append((value.st_dev, value.st_ino))
                return real_fsync(descriptor)

            with patch(
                    "stage_weyl_context_core_capture.os.fsync",
                    side_effect=observe_install_sync):
                stager._atomic_install(
                    root, incoming, "nested/input.txt", raw,
                    hashlib.sha256(raw).hexdigest())
            self.assertIn(destination_identity, synced)

    def test_running_job_recovers_exact_twenty_fifth_attempt_and_missing_receipt(self):
        predecessor = recovery_predecessor()
        self.assertEqual(len(predecessor), 24)
        self.assertTrue(predecessor[-1]["stage"].endswith(
            "/weyl-context-core-after-v5"))
        predecessor_contract = copy.deepcopy(stager.PREDECESSOR)
        predecessor_contract["campaign_ledger_sha256"] = (
            stager.saved_json_sha(predecessor)
        )
        with tempfile.TemporaryDirectory() as directory, \
                patch.object(stager, "PREDECESSOR", predecessor_contract):
            campaign = Path(directory) / "campaign"
            root = campaign / "stages" / stager.STAGE_NAME
            root.mkdir(parents=True)
            (campaign / ".atlas-progressive-submit.lock").touch()
            (root / stager.STAGE_LOCK).touch()
            pin = valid_pin()
            pin_sha256 = stager.saved_json_sha(pin)
            attempt = unconfirmed_capture_attempt(root, pin)
            history = predecessor + [copy.deepcopy(attempt)]
            self.assertEqual(len(history), 25)
            ledger_path = campaign / ".atlas-progressive-submit.json"
            intent_path = root / "submission-intent.json"
            receipt_path = root / "submission.json"
            stager.save(ledger_path, history)
            stager.save(intent_path, attempt)
            stager.save(root / stager.PIN_NAME, pin)
            spool = root / "submitted.sbatch"
            spool.write_text("#!/bin/bash\n", encoding="utf-8")
            job = "812345"
            confirmed = dict(attempt, status="SUBMITTED", job=job)
            expected_receipt = stager.submission_receipt(
                confirmed, pin, root=root)
            parent_manifest = {
                "source-%04d" % index: "a" * 64
                for index in range(stager.ACCEPTED_SOURCE["parent_files"])
            }
            seal = {
                "source_object": stager.PARENT_SOURCE_OBJECT,
                "source_manifest": parent_manifest,
            }

            with ExitStack() as stack:
                stack.enter_context(patch.object(
                    stager, "submission_scope", return_value=campaign))
                stack.enter_context(patch.object(
                    driver, "campaign_stage", return_value=campaign))
                stack.enter_context(
                    patch.object(driver, "validate_stage_topology"))
                creation_gate = stack.enter_context(patch.object(
                    driver, "validate_stage_creation"))
                stack.enter_context(patch.object(
                    driver, "frozen_stage_inputs",
                    return_value=copy.deepcopy(pin["inputs"])))
                stack.enter_context(patch.object(
                    driver, "digest",
                    return_value=pin["inputs"][stager.SBATCH]))
                stack.enter_context(patch.object(
                    driver, "validate_predecessor",
                    return_value={"source": True}))
                stack.enter_context(
                    patch.object(driver, "validate_prior_creation_failure"))
                stack.enter_context(
                    patch.object(driver, "validate_capture_v1_failure"))
                stack.enter_context(
                    patch.object(driver, "validate_capture_v2_failure"))
                v3_gate = stack.enter_context(patch.object(
                    driver, "validate_capture_v3_failure"))
                v4_gate = stack.enter_context(patch.object(
                    driver, "validate_capture_v4_failure"))
                v5_gate = stack.enter_context(patch.object(
                    driver, "validate_capture_v5_failure"))
                v6_gate = stack.enter_context(patch.object(
                    driver, "validate_capture_v6_failure"))
                v7_gate = stack.enter_context(patch.object(
                    driver, "validate_capture_v7_failure"))
                v8_gate = stack.enter_context(patch.object(
                    driver, "validate_capture_v8"))
                before_v1_gate = stack.enter_context(patch.object(
                    driver, "validate_before_v1_failure"))
                before_v2_gate = stack.enter_context(patch.object(
                    driver, "validate_before_v2_failure"))
                before_v3_gate = stack.enter_context(patch.object(
                    driver, "validate_before_v3_failure"))
                stack.enter_context(patch.object(
                    driver, "validate_staged_catalog",
                    return_value={"catalog": True}))
                g2_gate = stack.enter_context(patch.object(
                    driver, "validate_g2_inputs",
                    return_value={"g2": True}))
                before_v4_gate = stack.enter_context(patch.object(
                    driver, "validate_before_v4_result"))
                after_v1_gate = stack.enter_context(patch.object(
                    driver, "validate_after_v1_failure"))
                after_v2_gate = stack.enter_context(patch.object(
                    driver, "validate_after_v2_failure"))
                after_v3_gate = stack.enter_context(patch.object(
                    driver, "validate_after_v3_failure"))
                after_v4_gate = stack.enter_context(patch.object(
                    driver, "validate_after_v4_failure"))
                after_v5_gate = stack.enter_context(patch.object(
                    driver, "validate_after_v5_result"))
                stack.enter_context(patch.object(
                    driver, "validate_parent_objects",
                    return_value=stager.PARENT_SOURCE_OBJECT))
                stack.enter_context(patch.object(
                    driver, "load_parent_seal", return_value=seal))
                stack.enter_context(patch.object(driver, "read_blob"))
                submit_one = stack.enter_context(
                    patch.object(stager, "submit_one"))
                submit_pinned = stack.enter_context(
                    patch.object(stager, "submit_pinned"))
                persist = stack.enter_context(patch.object(
                    stager, "save", wraps=stager.save))
                stack.enter_context(patch.dict(driver.os.environ, {
                    "WEYL_CONTEXT_CORE_CAPTURE_PIN_SHA256": pin_sha256,
                    "WEYL_CONTEXT_CORE_CAPTURE_SPOOL": str(spool),
                    "SLURM_JOB_ID": job,
                }, clear=True))
                result = driver.gates(root)

            submit_one.assert_not_called()
            submit_pinned.assert_not_called()
            creation_gate.assert_called_once_with(
                root, confirmed["stage_creation_sha256"],
                pin_sha256=pin_sha256,
            )
            v3_gate.assert_called_once_with(root, pin["inputs"])
            v4_gate.assert_called_once_with(root, pin["inputs"])
            v5_gate.assert_called_once_with(root, pin["inputs"])
            v6_gate.assert_called_once_with(root, pin["inputs"])
            v7_gate.assert_called_once_with(root, pin["inputs"])
            v8_gate.assert_called_once_with(root, pin["inputs"])
            before_v1_gate.assert_called_once_with(root, pin["inputs"])
            before_v2_gate.assert_called_once_with(root, pin["inputs"])
            before_v3_gate.assert_called_once_with(root, pin["inputs"])
            g2_gate.assert_called_once_with(
                root, pin["inputs"], {"source": True})
            before_v4_gate.assert_called_once_with(root, pin["inputs"])
            after_v1_gate.assert_called_once_with(root, pin["inputs"])
            after_v2_gate.assert_called_once_with(root, pin["inputs"])
            after_v3_gate.assert_called_once_with(root, pin["inputs"])
            after_v4_gate.assert_called_once_with(root, pin["inputs"])
            after_v5_gate.assert_called_once_with(root, pin["inputs"])
            self.assertEqual(result["record"], confirmed)
            self.assertEqual(result["receipt"], expected_receipt)
            self.assertEqual(result["g2_source_manifest"],
                             {"g2": True})
            self.assertEqual(stager.read_json_file(ledger_path),
                             predecessor + [confirmed])
            self.assertEqual(stager.read_json_file(intent_path), confirmed)
            self.assertEqual(stager.read_json_file(receipt_path),
                             expected_receipt)

            # The login-side re-entry has no SLURM_JOB_ID argument, but it
            # must replay the same ledger durability before fixing intent;
            # run_enabled writes its receipt only after this function returns.
            stager.save(intent_path, attempt)
            receipt_path.unlink()
            with patch.object(
                    stager, "submission_scope", return_value=campaign), \
                    patch.object(
                        stager, "save", wraps=stager.save) as login_persist:
                login_replayed = stager.confirmed_record(
                    root, pin, repair_intent=True)
            self.assertEqual(login_replayed, confirmed)
            self.assertEqual(
                [Path(call.args[0]).name
                 for call in login_persist.call_args_list],
                [ledger_path.name, intent_path.name],
            )
            self.assertEqual(stager.read_json_file(ledger_path),
                             predecessor + [confirmed])
            self.assertEqual(stager.read_json_file(intent_path), confirmed)
            self.assertFalse(receipt_path.exists())
            stager.save(receipt_path, expected_receipt)
            self.assertEqual(
                [Path(call.args[0]).name for call in persist.call_args_list],
                [ledger_path.name, intent_path.name, receipt_path.name],
            )

            # Exercise the separate state where submit_one's confirmed ledger
            # is visible but its parent sync may have been interrupted.  The
            # compute recovery must replay ledger durability before repairing
            # the still-unconfirmed intent and missing receipt.
            stager.save(intent_path, attempt)
            receipt_path.unlink()
            with patch.object(
                    stager, "submission_scope", return_value=campaign), \
                    patch.object(
                        stager, "save", wraps=stager.save) as replay_persist:
                replayed = stager.confirmed_record(
                    root, pin, repair_intent=True, expected_job=job)
            self.assertEqual(replayed, confirmed)
            self.assertEqual(
                [Path(call.args[0]).name
                 for call in replay_persist.call_args_list],
                [ledger_path.name, intent_path.name, receipt_path.name],
            )
            self.assertEqual(stager.read_json_file(ledger_path),
                             predecessor + [confirmed])
            self.assertEqual(stager.read_json_file(intent_path), confirmed)
            self.assertEqual(stager.read_json_file(receipt_path),
                             expected_receipt)

    def test_running_job_recovery_rejects_every_unconfirmed_contract_drift(self):
        cases = (
            "missing-job", "nondecimal-job", "short-history",
            "non-tail-attempt", "predecessor-drift", "extra-ledger-key",
            "ledger-already-has-job", "wrong-status", "wrong-stage",
            "wrong-script", "wrong-queue", "wrong-maximum", "wrong-pin",
            "wrong-creation", "intent-stage", "intent-script",
            "intent-queue", "intent-maximum", "intent-pin",
            "intent-creation", "intent-status", "missing-intent",
            "wrong-pin-argument", "changed-receipt",
        )
        for case in cases:
            with self.subTest(case=case), \
                    tempfile.TemporaryDirectory() as directory:
                predecessor = recovery_predecessor()
                predecessor_contract = copy.deepcopy(stager.PREDECESSOR)
                predecessor_contract["campaign_ledger_sha256"] = (
                    stager.saved_json_sha(predecessor)
                )
                with patch.object(
                        stager, "PREDECESSOR", predecessor_contract):
                    campaign = Path(directory) / "campaign"
                    root = campaign / "stages" / stager.STAGE_NAME
                    root.mkdir(parents=True)
                    (campaign / ".atlas-progressive-submit.lock").touch()
                    pin = valid_pin()
                    supplied_pin = pin
                    attempt = unconfirmed_capture_attempt(root, pin)
                    history = predecessor + [copy.deepcopy(attempt)]
                    intent = copy.deepcopy(attempt)
                    expected_job = "812345"
                    receipt_path = root / "submission.json"

                    if case == "missing-job":
                        expected_job = None
                    elif case == "nondecimal-job":
                        expected_job = "812345_1"
                    elif case == "short-history":
                        history = predecessor[:-1] + [history[-1]]
                    elif case == "non-tail-attempt":
                        history = predecessor[:-1] + [
                            history[-1], predecessor[-1],
                        ]
                    elif case == "predecessor-drift":
                        history[0] = dict(history[0], job="7999")
                    elif case == "extra-ledger-key":
                        history[-1]["extra"] = False
                        intent["extra"] = False
                    elif case == "ledger-already-has-job":
                        history[-1]["job"] = expected_job
                        intent["job"] = expected_job
                    elif case == "wrong-status":
                        history[-1]["status"] = "SUBMITTED"
                        intent["status"] = "SUBMITTED"
                    elif case == "wrong-stage":
                        history[-1]["stage"] += "-other"
                        intent["stage"] += "-other"
                    elif case == "wrong-script":
                        history[-1]["script"] = "hpc/other.sbatch"
                        intent["script"] = "hpc/other.sbatch"
                    elif case == "wrong-queue":
                        history[-1]["queue_before"] = [expected_job]
                        intent["queue_before"] = [expected_job]
                    elif case == "wrong-maximum":
                        history[-1]["max_outstanding"] = 9
                        intent["max_outstanding"] = 9
                    elif case == "wrong-pin":
                        history[-1]["pin_sha256"] = "e" * 64
                        intent["pin_sha256"] = "e" * 64
                    elif case == "wrong-creation":
                        history[-1]["stage_creation_sha256"] = "e" * 64
                        intent["stage_creation_sha256"] = "e" * 64
                    elif case.startswith("intent-"):
                        field = {
                            "intent-stage": "stage",
                            "intent-script": "script",
                            "intent-queue": "queue_before",
                            "intent-maximum": "max_outstanding",
                            "intent-pin": "pin_sha256",
                            "intent-creation": "stage_creation_sha256",
                            "intent-status": "status",
                        }[case]
                        intent[field] = {
                            "stage": intent["stage"] + "-other",
                            "script": "hpc/other.sbatch",
                            "queue_before": [],
                            "max_outstanding": 9,
                            "pin_sha256": "e" * 64,
                            "stage_creation_sha256": "e" * 64,
                            "status": "SUBMITTED",
                        }[field]
                    elif case == "wrong-pin-argument":
                        supplied_pin = copy.deepcopy(pin)
                        supplied_pin["scope"] += " changed"

                    ledger_path = campaign / ".atlas-progressive-submit.json"
                    intent_path = root / "submission-intent.json"
                    stager.save(ledger_path, history)
                    if case != "missing-intent":
                        stager.save(intent_path, intent)
                    if case == "changed-receipt":
                        stager.save(receipt_path, {"changed": True})
                    ledger_before = ledger_path.read_bytes()
                    intent_before = (
                        intent_path.read_bytes() if intent_path.exists() else None)
                    receipt_before = (
                        receipt_path.read_bytes() if receipt_path.exists() else None)

                    with patch.object(
                            stager, "submission_scope", return_value=campaign), \
                            patch.object(stager, "submit_one") as submit_one, \
                            patch.object(stager, "submit_pinned") as submit_pinned:
                        with self.assertRaises((OSError, ValueError)):
                            stager.confirmed_record(
                                root, supplied_pin, repair_intent=True,
                                expected_job=expected_job,
                            )

                    submit_one.assert_not_called()
                    submit_pinned.assert_not_called()
                    self.assertEqual(ledger_path.read_bytes(), ledger_before)
                    self.assertEqual(
                        intent_path.read_bytes() if intent_path.exists() else None,
                        intent_before,
                    )
                    self.assertEqual(
                        receipt_path.read_bytes() if receipt_path.exists() else None,
                        receipt_before,
                    )

    def test_stager_submission_is_prerequisite_bound_single_job_and_fail_closed(self):
        source = Path(stager.__file__).read_text()
        tree = ast.parse(source)
        functions = {
            node.name: node for node in tree.body
            if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef))
        }
        module_assignments = {
            node.targets[0].id: node.value
            for node in tree.body
            if isinstance(node, ast.Assign)
            and len(node.targets) == 1
            and isinstance(node.targets[0], ast.Name)
        }
        before_v1_campaign_files = module_assignments[
            "BEFORE_V1_PREDECESSOR_CAMPAIGN_FILES"
        ]
        self.assertIsInstance(before_v1_campaign_files, ast.BinOp)
        self.assertIsInstance(before_v1_campaign_files.op, ast.BitOr)
        before_v2_campaign_files = module_assignments[
            "BEFORE_V2_PREDECESSOR_CAMPAIGN_FILES"
        ]
        self.assertIsInstance(before_v2_campaign_files, ast.BinOp)
        self.assertIsInstance(before_v2_campaign_files.op, ast.BitOr)
        predecessor_campaign_files = module_assignments[
            "PREDECESSOR_CAMPAIGN_FILES"
        ]
        self.assertIsInstance(predecessor_campaign_files, ast.BinOp)
        self.assertIsInstance(predecessor_campaign_files.op, ast.BitOr)
        stage_inputs = module_assignments["STAGE_INPUT_NAMES"]
        self.assertIsInstance(stage_inputs, ast.Set)
        self.assertFalse(any(
            isinstance(node, ast.Subscript) for node in ast.walk(stage_inputs)
        ))
        stage_input_literals = {
            node.value for node in ast.walk(stage_inputs)
            if isinstance(node, ast.Constant)
            and isinstance(node.value, str)
        }
        self.assertTrue({
            "tests/reference/hpc/"
            "math_weyl_context_core_before_v1_failure_2026_10_02.json",
            "tests/reference/hpc/"
            "math_weyl_context_core_before_v2_failure_2026_10_02.json",
            "tests/reference/hpc/"
            "math_weyl_context_core_before_v3_failure_2026_10_02.json",
        } <= stage_input_literals)

        def executable_body(function):
            body = list(function.body)
            if (body and isinstance(body[0], ast.Expr)
                    and isinstance(body[0].value, ast.Constant)
                    and isinstance(body[0].value.value, str)):
                body = body[1:]
            return body

        run = functions["run_enabled"]
        first = executable_body(run)[0]
        self.assertIsInstance(first, ast.Assign)
        self.assertIsInstance(first.value, ast.Call)
        self.assertIsInstance(first.value.func, ast.Name)
        self.assertEqual(first.value.func.id, "require_enabled_launcher")
        main = functions["main"]
        first_main = executable_body(main)[0]
        self.assertIsInstance(first_main, ast.If)
        self.assertIn("SUBMISSION_ENABLED", ast.unparse(first_main.test))
        self.assertIn("validate_predecessor", source)
        self.assertIn("validate_prior_creation_failure", source)
        self.assertIn("validate_capture_v1_failure", source)
        self.assertIn("validate_capture_v2_failure", source)
        self.assertIn("validate_capture_v3_failure", source)
        self.assertIn("validate_capture_v4_failure", source)
        self.assertIn("validate_capture_v5_failure", source)
        self.assertIn("validate_capture_v6_failure", source)
        self.assertIn("validate_capture_v7_failure", source)
        self.assertIn("validate_capture_v8", source)
        self.assertIn("validate_before_v1_failure", source)
        self.assertIn("validate_before_v2_failure", source)
        self.assertIn("validate_before_v3_failure", source)
        self.assertIn("validate_regression_inputs", source)
        self.assertIn("validate_catalog", source)
        self.assertIn("validate_launcher_transition", source)
        self.assertIn("validate_campaign_history", source)
        self.assertIn("submit_one", inspect.getsource(stager.submit_pinned))
        self.assertNotIn("subprocess", source)
        self.assertNotIn("--array", source)
        self.assertNotIn("--dependency", source)
        self.assertEqual(stager.PREDECESSOR_REFERENCE, stager.PREDECESSOR)
        self.assertEqual(stager.PREDECESSOR_STAGE,
                         stager.PREDECESSOR["stage"])
        self.assertTrue(stager.PREDECESSOR_STAGE.endswith(
            "/stages/weyl-context-core-after-v5"))
        self.assertEqual(stager.PREDECESSOR_STATE["schema"],
                         "atlas-stage-creation-predecessor-v14")
        self.assertTrue(stager.BEFORE_V2_PREDECESSOR_STAGE.endswith(
            "/stages/weyl-context-core-before-v2"))
        self.assertEqual(stager.BEFORE_V2_PREDECESSOR_REFERENCE,
                         stager.BEFORE_V2_PREDECESSOR)
        self.assertTrue(stager.BEFORE_V1_PREDECESSOR_STAGE.endswith(
            "/stages/weyl-context-core-before-v1"))
        self.assertEqual(stager.BEFORE_V1_PREDECESSOR_REFERENCE,
                         stager.BEFORE_V1_PREDECESSOR)
        self.assertTrue(stager.V8_PREDECESSOR_STAGE.endswith(
            "/stages/weyl-context-core-capture-v8"))
        self.assertEqual(stager.V8_PREDECESSOR_REFERENCE,
                         stager.V8_PREDECESSOR)
        self.assertTrue(stager.V7_PREDECESSOR_STAGE.endswith(
            "/stages/weyl-context-core-capture-v7"))
        self.assertEqual(stager.V7_PREDECESSOR_REFERENCE,
                         stager.V7_PREDECESSOR)
        self.assertTrue(stager.V6_PREDECESSOR_STAGE.endswith(
            "/stages/weyl-context-core-capture-v6"))
        self.assertEqual(stager.V6_PREDECESSOR_REFERENCE,
                         stager.V6_PREDECESSOR)
        self.assertTrue(stager.V5_PREDECESSOR_STAGE.endswith(
            "/stages/weyl-context-core-capture-v5"))
        self.assertEqual(stager.V5_PREDECESSOR_REFERENCE,
                         stager.V5_PREDECESSOR)
        self.assertTrue(stager.V4_PREDECESSOR_STAGE.endswith(
            "/stages/weyl-context-core-capture-v4"))
        self.assertEqual(stager.V4_PREDECESSOR_REFERENCE,
                         stager.V4_PREDECESSOR)
        self.assertTrue(stager.V3_PREDECESSOR_STAGE.endswith(
            "/stages/weyl-context-core-capture-v3"))
        self.assertEqual(stager.V3_PREDECESSOR_REFERENCE,
                         stager.V3_PREDECESSOR)

        def local_aliases(function_name):
            return {
                node.targets[0].id: node.value.id
                for node in ast.walk(functions[function_name])
                if isinstance(node, ast.Assign)
                and len(node.targets) == 1
                and isinstance(node.targets[0], ast.Name)
                and isinstance(node.value, ast.Name)
                and node.targets[0].id in {
                    "PREDECESSOR_STAGE", "PREDECESSOR_STATE", "PREDECESSOR",
                }
            }

        self.assertEqual(local_aliases("validate_capture_v8"), {
            "PREDECESSOR_STAGE": "V8_PREDECESSOR_STAGE",
            "PREDECESSOR_STATE": "V8_PREDECESSOR_STATE",
            "PREDECESSOR": "V8_PREDECESSOR",
        })
        self.assertEqual(local_aliases("validate_before_v1_failure"), {
            "PREDECESSOR_STAGE": "BEFORE_V1_PREDECESSOR_STAGE",
            "PREDECESSOR_STATE": "BEFORE_V1_PREDECESSOR_STATE",
            "PREDECESSOR": "BEFORE_V1_PREDECESSOR",
        })
        self.assertEqual(local_aliases("validate_before_v2_failure"), {
            "PREDECESSOR_STAGE": "BEFORE_V2_PREDECESSOR_STAGE",
            "PREDECESSOR_STATE": "BEFORE_V2_PREDECESSOR_STATE",
            "PREDECESSOR": "BEFORE_V2_PREDECESSOR",
        })
        self.assertEqual(
            set(stager.FROZEN_LAUNCHER_HASHES)
            | {stager.CURRENT_STAGER_PATH},
            set(stager.FROZEN_LAUNCHER_STATES),
        )
        run_enabled_source = inspect.getsource(stager.run_enabled)
        self.assertLess(
            run_enabled_source.index(
                'validate_launcher_transition(payload_root / "overrides", manifest)'),
            run_enabled_source.index("create_fixed_stage("),
        )
        self.assertLess(
            run_enabled_source.index(
                'validate_prior_creation_failure(payload_root / "overrides", manifest)'),
            run_enabled_source.index("create_fixed_stage("),
        )
        self.assertLess(
            run_enabled_source.index(
                'validate_capture_v1_failure(payload_root / "overrides", manifest)'),
            run_enabled_source.index("create_fixed_stage("),
        )
        self.assertLess(
            run_enabled_source.index(
                'validate_capture_v2_failure(payload_root / "overrides", manifest)'),
            run_enabled_source.index("create_fixed_stage("),
        )
        self.assertLess(
            run_enabled_source.index(
                'validate_capture_v3_failure(payload_root / "overrides", manifest)'),
            run_enabled_source.index("create_fixed_stage("),
        )
        self.assertLess(
            run_enabled_source.index(
                'validate_capture_v4_failure(payload_root / "overrides", manifest)'),
            run_enabled_source.index("create_fixed_stage("),
        )
        self.assertLess(
            run_enabled_source.index(
                'validate_capture_v5_failure(payload_root / "overrides", manifest)'),
            run_enabled_source.index("create_fixed_stage("),
        )
        self.assertLess(
            run_enabled_source.index(
                'validate_capture_v6_failure(payload_root / "overrides", manifest)'),
            run_enabled_source.index("create_fixed_stage("),
        )
        self.assertLess(
            run_enabled_source.index(
                'validate_capture_v7_failure(payload_root / "overrides", manifest)'),
            run_enabled_source.index("create_fixed_stage("),
        )
        self.assertLess(
            run_enabled_source.index(
                'validate_capture_v8(payload_root / "overrides", manifest)'),
            run_enabled_source.index("create_fixed_stage("),
        )
        self.assertLess(
            run_enabled_source.index(
                'validate_before_v3_failure(payload_root / "overrides", manifest)'),
            run_enabled_source.index("create_fixed_stage("),
        )
        direct_calls = sorted(
            [
                [node.lineno, node]
                for node in ast.walk(run)
                if isinstance(node, ast.Call)
                and isinstance(node.func, ast.Name)
            ],
            key=lambda item: item[0],
        )

        def calls_named(name):
            return [
                (line, call) for line, call in direct_calls
                if call.func.id == name
            ]

        predecessor_assignments = sorted(
            [
                [node.lineno, node.targets[0].id, node.value]
                for node in ast.walk(run)
                if isinstance(node, ast.Assign)
                and len(node.targets) == 1
                and isinstance(node.targets[0], ast.Name)
                and isinstance(node.value, ast.Call)
                and isinstance(node.value.func, ast.Name)
                and node.value.func.id == "validate_predecessor"
            ],
            key=lambda item: item[0],
        )
        regression_calls = calls_named("validate_g2_inputs")
        v8_calls = calls_named("validate_capture_v8")
        before_v1_calls = calls_named("validate_before_v1_failure")
        before_v2_calls = calls_named("validate_before_v2_failure")
        before_v3_calls = calls_named("validate_before_v3_failure")
        catalog_calls = calls_named("validate_catalog")
        create_calls = calls_named("create_fixed_stage")
        install_calls = calls_named("install_inputs")
        pin_calls = calls_named("build_pin")
        self.assertEqual(len(predecessor_assignments), 2)
        self.assertEqual(len(regression_calls), 2)
        self.assertEqual(len(v8_calls), 2)
        self.assertEqual(len(before_v1_calls), 2)
        self.assertEqual(len(before_v2_calls), 2)
        self.assertEqual(len(before_v3_calls), 2)
        self.assertEqual(len(catalog_calls), 2)
        self.assertEqual(len(create_calls), 1)
        self.assertEqual(len(install_calls), 1)
        self.assertEqual(len(pin_calls), 1)
        precreation_source_line, precreation_source_name, precreation_source = (
            predecessor_assignments[0]
        )
        installed_source_line, installed_source_name, installed_source = (
            predecessor_assignments[1]
        )
        precreation_regression_line, precreation_regression = regression_calls[0]
        installed_regression_line, installed_regression = regression_calls[1]

        def argument_shapes(expressions):
            return [
                ast.dump(expression, include_attributes=False)
                for expression in expressions
            ]

        override_arguments = argument_shapes(ast.parse(
            'probe(payload_root / "overrides", manifest)'
        ).body[0].value.args)
        installed_arguments = argument_shapes(ast.parse(
            "probe(root, manifest)"
        ).body[0].value.args)
        self.assertEqual(
            argument_shapes(precreation_source.args), override_arguments)
        self.assertEqual(
            argument_shapes(installed_source.args), installed_arguments)
        self.assertEqual(
            argument_shapes(precreation_regression.args[:2]),
            override_arguments,
        )
        self.assertEqual(
            argument_shapes(installed_regression.args[:2]),
            installed_arguments,
        )
        self.assertIsInstance(precreation_regression.args[2], ast.Name)
        self.assertIsInstance(installed_regression.args[2], ast.Name)
        self.assertEqual(
            precreation_regression.args[2].id, precreation_source_name)
        self.assertEqual(
            installed_regression.args[2].id, installed_source_name)
        self.assertLess(precreation_source_line, v8_calls[0][0])
        self.assertLess(v8_calls[0][0], before_v1_calls[0][0])
        self.assertLess(before_v1_calls[0][0], before_v2_calls[0][0])
        self.assertLess(before_v2_calls[0][0], before_v3_calls[0][0])
        self.assertLess(before_v3_calls[0][0], precreation_regression_line)
        self.assertLess(before_v3_calls[0][0], catalog_calls[0][0])
        self.assertLess(precreation_regression_line, create_calls[0][0])
        self.assertLess(install_calls[0][0], installed_source_line)
        self.assertLess(installed_source_line, v8_calls[1][0])
        self.assertLess(v8_calls[1][0], before_v1_calls[1][0])
        self.assertLess(before_v1_calls[1][0], before_v2_calls[1][0])
        self.assertLess(before_v2_calls[1][0], before_v3_calls[1][0])
        self.assertLess(installed_source_line, installed_regression_line)
        self.assertLess(before_v3_calls[1][0], installed_regression_line)
        self.assertLess(before_v3_calls[1][0], catalog_calls[1][0])
        self.assertLess(installed_regression_line, pin_calls[0][0])
        self.assertLess(
            run_enabled_source.index(
                'validate_capture_v6_failure(payload_root / "overrides", manifest)'),
            run_enabled_source.index(
                'validate_capture_v7_failure(payload_root / "overrides", manifest)'),
        )
        self.assertNotIn("prior_failure=PRIOR_CREATION_FAILURE",
                         run_enabled_source)
        repository = Path(__file__).resolve().parents[1]
        with tempfile.TemporaryDirectory() as directory:
            launch_root = Path(directory)
            manifest = {}
            original = {}
            for name in stager.FROZEN_LAUNCHER_STATES:
                raw = (repository / name).read_bytes()
                destination = launch_root / name
                destination.parent.mkdir(parents=True, exist_ok=True)
                destination.write_bytes(raw)
                manifest[name] = hashlib.sha256(raw).hexdigest()
                original[name] = raw
            self.assertEqual(
                stager.validate_launcher_transition(launch_root, manifest),
                stager.FROZEN_LAUNCHER_STATES,
            )
            for name in stager.FROZEN_LAUNCHER_STATES:
                changed = original[name] + b"# self-consistent mutation\n"
                (launch_root / name).write_bytes(changed)
                manifest[name] = hashlib.sha256(changed).hexdigest()
                with self.subTest(launcher=name):
                    with self.assertRaises(ValueError):
                        stager.validate_launcher_transition(
                            launch_root, manifest,
                        )
                (launch_root / name).write_bytes(original[name])
                manifest[name] = hashlib.sha256(original[name]).hexdigest()
        with tempfile.TemporaryDirectory() as directory:
            stage = Path(directory)
            stager.validate_stage_topology(stage)
            incoming = stage / ".incoming"
            incoming.mkdir()
            stager.validate_stage_topology(stage)
            hidden_file = incoming / "target"
            hidden_file.write_text("durable junk\n")
            with self.assertRaises(ValueError):
                stager.validate_stage_topology(stage)
            hidden_file.unlink()
            hidden_directory = incoming / "build"
            hidden_directory.mkdir()
            with self.assertRaises(ValueError):
                stager.validate_stage_topology(stage)
            hidden_directory.rmdir()
            hidden_link = incoming / "source"
            hidden_link.symlink_to("missing")
            with self.assertRaises(ValueError):
                stager.validate_stage_topology(stage)
            hidden_link.unlink()
            (stage / "results").mkdir()
            stager.validate_stage_topology(stage)
            (stage / "results" / "12345").mkdir()
            stager.validate_stage_topology(stage)
            sibling_result = stage / "results" / "54321"
            sibling_result.mkdir()
            with self.assertRaises(ValueError):
                stager.validate_stage_topology(stage)
            sibling_result.rmdir()
            for name in (
                    "source", "source-r2", "target", "target-retry",
                    "workspace", "workspace-old", "build", ".atlas-cache",
                    "unknown"):
                forbidden = stage / name
                forbidden.mkdir()
                with self.subTest(name=name):
                    with self.assertRaises(ValueError):
                        stager.validate_stage_topology(stage)
                forbidden.rmdir()
            unknown_file = stage / "unknown.txt"
            unknown_file.write_text("not a lifecycle artifact")
            with self.assertRaises(ValueError):
                stager.validate_stage_topology(stage)
            unknown_file.unlink()
            (stage / "results" / "12345").rmdir()
            nonnumeric_result = stage / "results" / "latest"
            nonnumeric_result.mkdir()
            with self.assertRaises(ValueError):
                stager.validate_stage_topology(stage)
            nonnumeric_result.rmdir()
            numeric_file = stage / "results" / "54321"
            numeric_file.write_text("not a result directory")
            with self.assertRaises(ValueError):
                stager.validate_stage_topology(stage)
            numeric_file.unlink()
            with tempfile.TemporaryDirectory() as target_directory:
                numeric_link = stage / "results" / "54321"
                numeric_link.symlink_to(target_directory,
                                        target_is_directory=True)
                with self.assertRaises(ValueError):
                    stager.validate_stage_topology(stage)
                numeric_link.unlink()
                results = stage / "results"
                results.rmdir()
                results.symlink_to(target_directory, target_is_directory=True)
                with self.assertRaises(ValueError):
                    stager.validate_stage_topology(stage)
                results.unlink()
        run_source = run_enabled_source
        self.assertLess(run_source.index("_read_override_manifest"),
                        run_source.index("create_fixed_stage("))
        self.assertLess(run_source.index("create_fixed_stage("),
                        run_source.index("validate_stage_topology"))
        installed_v7_at = run_source.index(
            "validate_capture_v7_failure(root, manifest)")
        installed_v8_at = run_source.index(
            "validate_capture_v8(root, manifest)")
        installed_before_v1_at = run_source.index(
            "validate_before_v1_failure(root, manifest)")
        installed_before_v2_at = run_source.index(
            "validate_before_v2_failure(root, manifest)")
        installed_before_v3_at = run_source.index(
            "validate_before_v3_failure(root, manifest)")
        installed_g2_at = run_source.index(
            "validate_g2_inputs(root, manifest")
        installed_v6_at = run_source.index(
            "validate_capture_v6_failure(root, manifest)")
        self.assertLess(run_source.index("install_inputs(root, manifest)"),
                        installed_v7_at)
        self.assertLess(installed_v6_at, installed_v7_at)
        self.assertLess(installed_v7_at, installed_v8_at)
        self.assertLess(installed_v8_at, installed_before_v1_at)
        self.assertLess(installed_before_v1_at, installed_before_v2_at)
        self.assertLess(installed_before_v2_at, installed_before_v3_at)
        self.assertLess(installed_before_v3_at, installed_g2_at)
        self.assertLess(installed_g2_at, run_source.index("build_pin("))
        gates_source = inspect.getsource(driver.gates)
        self.assertLess(gates_source.index("validate_stage_topology"),
                        gates_source.index("frozen_stage_inputs"))
        self.assertLess(gates_source.index("confirmed_record("),
                        gates_source.index("validate_stage_creation("))
        self.assertLess(gates_source.index("validate_stage_creation("),
                        gates_source.index('read_json_file(root / "submission.json")'))
        self.assertLess(
            gates_source.index("validate_capture_v7_failure(root, inputs)"),
            gates_source.index("validate_staged_catalog(root, inputs)"),
        )
        self.assertLess(
            gates_source.index("validate_capture_v8(root, inputs)"),
            gates_source.index("validate_before_v1_failure(root, inputs)"),
        )
        self.assertLess(
            gates_source.index("validate_before_v1_failure(root, inputs)"),
            gates_source.index("validate_before_v2_failure(root, inputs)"),
        )
        self.assertLess(
            gates_source.index("validate_before_v2_failure(root, inputs)"),
            gates_source.index("validate_before_v3_failure(root, inputs)"),
        )
        self.assertLess(
            gates_source.index("validate_before_v3_failure(root, inputs)"),
            gates_source.index("validate_staged_catalog(root, inputs)"),
        )
        self.assertLess(
            gates_source.index("validate_staged_catalog(root, inputs)"),
            gates_source.index("validate_g2_inputs("),
        )
        self.assertLess(
            gates_source.index("validate_capture_v5_failure(root, inputs)"),
            gates_source.index("validate_capture_v6_failure(root, inputs)"),
        )
        self.assertLess(
            gates_source.index("validate_capture_v6_failure(root, inputs)"),
            gates_source.index("validate_capture_v7_failure(root, inputs)"),
        )

    def test_sbatch_is_exact_two_cpu_eight_gib_two_hour_exec_boundary(self):
        root = Path(__file__).resolve().parents[1]
        raw = (root / stager.SBATCH).read_text()
        self.assertIn("#!/bin/bash -p", raw)
        self.assertIn("#SBATCH --job-name=atlas-weyl-g2-v2", raw)
        self.assertIn(
            "#SBATCH --output=weyl-context-g2-v2-%j.out", raw)
        self.assertIn("#SBATCH --nodes=1", raw)
        self.assertIn("#SBATCH --ntasks=1", raw)
        self.assertIn("#SBATCH --cpus-per-task=2", raw)
        self.assertIn("#SBATCH --mem=8G", raw)
        self.assertIn("#SBATCH --time=02:00:00", raw)
        self.assertIn("#SBATCH --signal=B:TERM@300", raw)
        self.assertNotIn("#SBATCH --array", raw)
        self.assertNotIn("#SBATCH --dependency", raw)
        self.assertIn('cd "${SLURM_SUBMIT_DIR:?}"', raw)
        self.assertIn("PYTHONDONTWRITEBYTECODE=1", raw)
        self.assertIn("WEYL_CONTEXT_CORE_CAPTURE_SPOOL", raw)
        self.assertIn(
            "exec /public/software/anaconda/anaconda3-2022.5/bin/python3.9 \\",
            raw)
        self.assertIn("-E -s -S -B hpc/math_weyl_context_core_capture.py", raw)
        self.assertNotRegex(raw, r"(?m)^[^#\n]*\bsbatch\b")


if __name__ == "__main__":
    unittest.main()

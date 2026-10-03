"""Pure validator for the tests-first Weyl-context regression BEFORE gate.

The module performs no filesystem or process I/O.  Callers supply the frozen
catalog, independent capture inspection, fixtures, goldens, and fresh HPC
observations as bytes/data.  A successful classification records only that the
known regressions were reproduced before a fix; it grants no mathematical,
cache, performance, memory, or rank acceptance.
"""

import hashlib
import json
import math
import re


CATALOG_SCHEMA = "atlas-weyl-context-core-regression-v1"
CATALOG_BYTES = 5208
CATALOG_SHA256 = "6a9f960753bcd2afb575cb5207d4310f21a5694505d02a83f4e4142211eaa634"
CATALOG_MATURITY = "original_captured_regression_pending_before"

INSPECTION_SCHEMA = "atlas-weyl-context-core-capture-independent-inspection-v1"
INSPECTION_FILE = (
    "tests/reference/hpc/"
    "math_weyl_context_core_capture_v8_inspection_2026_10_02.json"
)
INSPECTION_BYTES = 11902
INSPECTION_SHA256 = "b2c7f4ece709df3506c3224a57abad896f3ecfcd6625fea97731619f629e1da3"
CAPTURE_CLASSIFICATION = (
    "CAPTURE_COMPLETE_RUST_SEMANTIC_MISMATCH_CONFIRMED_REGRESSION_REQUIRED"
)
CAPTURE_JOB = "3884807"
CAPTURE_REPORT_BYTES = 324940
CAPTURE_REPORT_SHA256 = (
    "e644017f6c040691c91ee56fdaeac09a3e8d834ec9956939c2ba2530c80c5c73"
)
ORACLE_COMMIT = "7e1b958c7aa9456769cc9cf09ac1542814b4800a"
ORACLE_BINARY_SHA256 = (
    "d4f0f3dc3a82102529aa2ec562db0601e99b368dee25d5539ca52dae2fd37a5a"
)

PARENT_SESSION_SHA256 = (
    "cef588d9afc7fcd513ee1660758b229a8767c252adacdec7feea9737675de6aa"
)
TESTS_ONLY_SESSION_SHA256 = (
    "969cdb27ccae61ca4fe3e36219801037d475517bee3614fead553024818307e7"
)
TEST_PATCH_SHA256 = (
    "ccd3009892dbeae2f146dff9924efa6d6bede3dca910d026c8f3ab3e1b4c49cf"
)

OBSERVATION_SCHEMA = "atlas-weyl-context-core-regression-before-observation-v1"
CLASSIFICATION_SCHEMA = (
    "atlas-weyl-context-core-regression-before-classification-v1"
)
BEFORE_REPRODUCED = "WEYL_CONTEXT_BEFORE_EXPECTED_FAILURES_OBSERVED"
UNEXPECTED_PASS = "WEYL_CONTEXT_BEFORE_UNEXPECTED_REGRESSIONS_PASSED"
PROVENANCE_FAILURE = "WEYL_CONTEXT_BEFORE_PROVENANCE_FAILURE"
HARNESS_FAILURE = "WEYL_CONTEXT_BEFORE_HARNESS_FAILURE"

COLD_CASE = "weyl_context_core_cold_dual"
PREWARM_CASE = "weyl_context_core_prewarmed_dual"
CASE_IDS = (COLD_CASE, PREWARM_CASE)
SELECTOR_TESTS = (
    "session::tests::weyl_context_core_cold_dual_original",
    "session::tests::weyl_context_core_prewarmed_dual_original",
)
CONTROL_TEST = "session::tests::root_ladder_coordinate_boundary_original"
EXPECTED_INVENTORY_COUNT = 632

_EMPTY_SHA256 = hashlib.sha256(b"").hexdigest()
_CATALOG_KEYS = frozenset(
    {
        "schema",
        "evidence_maturity",
        "scope",
        "source_capture",
        "cases",
        "required_gate_sequence",
        "claims_not_granted",
    }
)
_SOURCE_CAPTURE_KEYS = frozenset(
    {
        "job",
        "report_bytes",
        "report_sha256",
        "inspection_file",
        "inspection_bytes",
        "inspection_sha256",
        "oracle_commit",
        "oracle_binary_sha256",
        "prediction_arrays_are_goldens",
        "raw_oracle_streams_are_goldens",
    }
)
_CASE_KEYS = frozenset(
    {
        "id",
        "fixture",
        "fixture_bytes",
        "fixture_sha256",
        "capture_input_bytes",
        "capture_input_sha256",
        "oracle_exit_status",
        "oracle_stdout",
        "oracle_stderr",
        "regression_assertions",
        "before_requirement",
    }
)
_STREAM_KEYS = frozenset({"file", "bytes", "sha256"})
_INSPECTION_KEYS = frozenset(
    {
        "schema",
        "observed_at_utc",
        "classification",
        "campaign",
        "stage",
        "job",
        "submission_evidence",
        "report",
        "checker_and_build_commands",
        "artifacts",
        "binaries_and_execution",
        "captures",
        "prediction_interpretation",
        "regression_requirement",
        "performance_interpretation",
        "transport",
        "claims_not_granted",
    }
)
_PROVENANCE_KEYS = frozenset(
    {
        "catalog_bytes",
        "catalog_sha256",
        "inspection_bytes",
        "inspection_sha256",
        "capture_job",
        "capture_report_bytes",
        "capture_report_sha256",
        "oracle_commit",
        "oracle_binary_sha256",
        "parent_session_sha256",
        "tests_only_session_sha256",
        "test_patch_sha256",
        "production_unchanged",
    }
)
_OBSERVATION_KEYS = frozenset(
    {
        "schema",
        "provenance",
        "original_reruns",
        "selector_command",
        "retained_control_command",
        "inventory_command",
        "claims",
    }
)
_ORIGINAL_RUN_KEYS = frozenset(
    {
        "case_id",
        "engine",
        "exit_status",
        "stdout",
        "stderr",
        "fresh_process",
        "timed_out",
        "termination_uncertain",
        "metrics",
    }
)
_COMMAND_KEYS = frozenset({"name", "exit_status", "log", "metrics"})
_METRIC_KEYS = frozenset(
    {
        "format",
        "seconds",
        "user_cpu_seconds",
        "system_cpu_seconds",
        "maxrss_kb",
        "maxrss_approximate",
        "timed_out",
        "termination_uncertain",
        "signal",
    }
)
_CLAIM_KEYS = frozenset(
    {
        "acceptance_eligible",
        "math_gate_released",
        "cache_gate_released",
        "performance_gate_released",
        "rank_gate_released",
    }
)

_CASE_METADATA = {
    COLD_CASE: {
        "fixture": "weyl_context_core_cold_dual.atlas",
        "fixture_bytes": 1815,
        "fixture_sha256": (
            "4eff8fa08f8490e242f07282125b83cde1daf24a875fe8df4dc7e73e75a0ae99"
        ),
        "capture_input_bytes": 1916,
        "capture_input_sha256": (
            "d94ae61b72215cda32b4ef040221dd81d0dd127866bc334c2ca7f43a23b820ba"
        ),
        "oracle_exit_status": 0,
        "oracle_stdout": {
            "file": "weyl_context_core_cold_dual.oracle.stdout",
            "bytes": 911,
            "sha256": (
                "7a614e47b75469c441e774cbe46769dcd769c5cc0a2a31cf1868ff9b59a780dc"
            ),
        },
        "oracle_stderr": {
            "file": "weyl_context_core_cold_dual.oracle.stderr",
            "bytes": 0,
            "sha256": _EMPTY_SHA256,
        },
        "recovery": b"WC_RECOVERY|727\n",
        "ordered_diagnostics": (),
    },
    PREWARM_CASE: {
        "fixture": "weyl_context_core_prewarmed_dual.atlas",
        "fixture_bytes": 1275,
        "fixture_sha256": (
            "f13d704175f702b966790bf82e56c837dd80a594f0dc2dcd0d6b81ba281799a2"
        ),
        "capture_input_bytes": 1386,
        "capture_input_sha256": (
            "3f3bb95cf514aee419678543e135e405285b1d6db10d63eb590e27a94880ee4f"
        ),
        "oracle_exit_status": 1,
        "oracle_stdout": {
            "file": "weyl_context_core_prewarmed_dual.oracle.stdout",
            "bytes": 572,
            "sha256": (
                "5074aab3290d4ae404bb7abb0b24ccc99036f616abec518c6479e59ec2db021f"
            ),
        },
        "oracle_stderr": {
            "file": "weyl_context_core_prewarmed_dual.oracle.stderr",
            "bytes": 501,
            "sha256": (
                "ef4404d85f7252a611f9f4e4a5205fe6bbac2c66f48b7378a30a8053f986e157"
            ),
        },
        "recovery": b"WCN_RECOVERY|733\n",
        "ordered_diagnostics": (
            "Runtime:Weyl group mismatch",
            "Runtime:Weyl group mismatch",
            "Runtime:Weyl group mismatch",
            "Runtime:Illegal Weyl word entry 1 (should be <1)",
            "Runtime:Negative integer where unsigned is required",
            "Runtime:Weyl group mismatch",
            "Runtime:Weyl group mismatch",
            "Runtime:Weyl group mismatch",
        ),
    },
}


class ProvenanceError(ValueError):
    """The frozen source, oracle, catalog, inspection, or golden changed."""


class HarnessError(ValueError):
    """The tests-first execution is incomplete or has the wrong shape."""


def _sha(raw):
    return hashlib.sha256(raw).hexdigest()


def _is_int(value):
    return type(value) is int


def _is_number(value):
    return type(value) in (int, float) and math.isfinite(value)


def _unique_json_object(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError("duplicate JSON key: " + key)
        result[key] = value
    return result


def _reject_nonfinite_json(token):
    raise ValueError("non-finite JSON number: " + token)


def _decode_frozen_json(raw, expected_bytes, expected_sha256, label):
    if type(raw) is not bytes:
        raise ProvenanceError(label + " must be bytes")
    if len(raw) != expected_bytes or _sha(raw) != expected_sha256:
        raise ProvenanceError(label + " bytes changed")
    try:
        value = json.loads(
            raw,
            object_pairs_hook=_unique_json_object,
            parse_constant=_reject_nonfinite_json,
        )
    except (UnicodeDecodeError, json.JSONDecodeError, ValueError) as error:
        raise ProvenanceError(label + " is not strict JSON") from error
    if type(value) is not dict:
        raise ProvenanceError(label + " root must be an object")
    return value


def decode_catalog(raw):
    """Decode and validate the byte-frozen original-backed regression catalog."""
    catalog = _decode_frozen_json(
        raw, CATALOG_BYTES, CATALOG_SHA256, "regression catalog"
    )
    validate_catalog(catalog)
    return catalog


def validate_catalog(catalog):
    if type(catalog) is not dict or frozenset(catalog) != _CATALOG_KEYS:
        raise ProvenanceError("regression catalog schema changed")
    if (
        catalog.get("schema") != CATALOG_SCHEMA
        or catalog.get("evidence_maturity") != CATALOG_MATURITY
        or type(catalog.get("scope")) is not str
        or not catalog["scope"]
        or type(catalog.get("required_gate_sequence")) is not list
        or len(catalog["required_gate_sequence"]) != 4
        or not all(type(item) is str and item for item in catalog["required_gate_sequence"])
        or catalog.get("claims_not_granted")
        != [
            "Weyl-context mathematical acceptance",
            "cache correctness",
            "rank escalation",
            "performance improvement",
            "memory improvement",
        ]
    ):
        raise ProvenanceError("regression catalog contract changed")

    source = catalog.get("source_capture")
    expected_source = {
        "job": CAPTURE_JOB,
        "report_bytes": CAPTURE_REPORT_BYTES,
        "report_sha256": CAPTURE_REPORT_SHA256,
        "inspection_file": INSPECTION_FILE,
        "inspection_bytes": INSPECTION_BYTES,
        "inspection_sha256": INSPECTION_SHA256,
        "oracle_commit": ORACLE_COMMIT,
        "oracle_binary_sha256": ORACLE_BINARY_SHA256,
        "prediction_arrays_are_goldens": False,
        "raw_oracle_streams_are_goldens": True,
    }
    if (
        type(source) is not dict
        or frozenset(source) != _SOURCE_CAPTURE_KEYS
        or source != expected_source
    ):
        raise ProvenanceError("catalog source-capture binding changed")

    cases = catalog.get("cases")
    if (
        type(cases) is not list
        or any(type(case) is not dict for case in cases)
        or [case.get("id") for case in cases] != list(CASE_IDS)
    ):
        raise ProvenanceError("regression cases changed")
    for case in cases:
        case_id = case["id"]
        expected = _CASE_METADATA[case_id]
        keys = _CASE_KEYS | ({"ordered_oracle_diagnostics"} if case_id == PREWARM_CASE else set())
        if type(case) is not dict or frozenset(case) != frozenset(keys):
            raise ProvenanceError("regression case schema changed")
        for key in (
            "fixture",
            "fixture_bytes",
            "fixture_sha256",
            "capture_input_bytes",
            "capture_input_sha256",
            "oracle_exit_status",
            "oracle_stdout",
            "oracle_stderr",
        ):
            if case.get(key) != expected[key]:
                raise ProvenanceError("regression case metadata changed")
        if (
            type(case["fixture_bytes"]) is not int
            or type(case["capture_input_bytes"]) is not int
            or type(case["oracle_exit_status"]) is not int
            or type(case["oracle_stdout"]) is not dict
            or frozenset(case["oracle_stdout"]) != _STREAM_KEYS
            or type(case["oracle_stderr"]) is not dict
            or frozenset(case["oracle_stderr"]) != _STREAM_KEYS
            or type(case["regression_assertions"]) is not list
            or not case["regression_assertions"]
            or not all(type(item) is str and item for item in case["regression_assertions"])
            or type(case["before_requirement"]) is not str
            or not case["before_requirement"]
        ):
            raise ProvenanceError("regression case types changed")
        if case_id == PREWARM_CASE:
            if tuple(case["ordered_oracle_diagnostics"]) != expected["ordered_diagnostics"]:
                raise ProvenanceError("ordered oracle diagnostics changed")


def decode_inspection(raw, catalog):
    """Decode the exact independent v8 inspection and bind it to the catalog."""
    validate_catalog(catalog)
    inspection = _decode_frozen_json(
        raw, INSPECTION_BYTES, INSPECTION_SHA256, "capture inspection"
    )
    validate_inspection(inspection, catalog)
    return inspection


def validate_inspection(inspection, catalog):
    validate_catalog(catalog)
    if type(inspection) is not dict or frozenset(inspection) != _INSPECTION_KEYS:
        raise ProvenanceError("capture inspection schema changed")
    report = inspection.get("report")
    binaries = inspection.get("binaries_and_execution")
    job = inspection.get("job")
    performance = inspection.get("performance_interpretation")
    oracle_binary = binaries.get("oracle") if type(binaries) is dict else None
    if (
        inspection.get("schema") != INSPECTION_SCHEMA
        or inspection.get("classification") != CAPTURE_CLASSIFICATION
        or type(job) is not dict
        or job.get("id") != CAPTURE_JOB
        or job.get("state") != "COMPLETED"
        or job.get("exit_code") != "0:0"
        or job.get("outstanding_jobs_after_inspection") != 0
        or type(report) is not dict
        or report.get("bytes") != CAPTURE_REPORT_BYTES
        or report.get("sha256") != CAPTURE_REPORT_SHA256
        or report.get("complete") is not True
        or report.get("integrity_rechecked") is not True
        or report.get("source_integrity_rechecked") is not True
        or report.get("ephemeral_workspace_removed") is not True
        or report.get("acceptance_eligible") is not False
        or report.get("math_gate_released") is not False
        or report.get("cache_gate_released") is not False
        or type(binaries) is not dict
        or type(oracle_binary) is not dict
        or oracle_binary.get("commit") != ORACLE_COMMIT
        or oracle_binary.get("sha256") != ORACLE_BINARY_SHA256
        or binaries.get("fresh_processes") is not True
        or binaries.get("unique_pid_equals_process_group_id") is not True
        or binaries.get("timed_out") is not False
        or binaries.get("termination_uncertain") is not False
        or type(performance) is not dict
        or performance.get("valid_speed_ratio") is not False
        or performance.get("rust_vs_original_speedup_claimed") is not False
        or performance.get("memory_improvement_claimed") is not False
    ):
        raise ProvenanceError("capture inspection provenance changed")

    source = catalog["source_capture"]
    if (
        source["job"] != job["id"]
        or source["report_bytes"] != report["bytes"]
        or source["report_sha256"] != report["sha256"]
        or source["oracle_commit"] != oracle_binary["commit"]
        or source["oracle_binary_sha256"] != oracle_binary["sha256"]
    ):
        raise ProvenanceError("catalog and inspection do not bind one capture")

    captures = inspection.get("captures")
    if (
        type(captures) is not list
        or any(type(row) is not dict for row in captures)
        or [row.get("case_id") for row in captures] != list(CASE_IDS)
    ):
        raise ProvenanceError("inspection capture set changed")
    cases = {case["id"]: case for case in catalog["cases"]}
    for capture in captures:
        case = cases[capture["case_id"]]
        oracle = capture.get("oracle")
        input_record = capture.get("input")
        if (
            type(oracle) is not dict
            or type(input_record) is not dict
            or capture.get("fresh_process_complete") is not True
            or capture.get("resource_complete") is not True
            or capture.get("metrics_complete") is not True
            or capture.get("stream_complete") is not True
            or input_record
            != {
                "bytes": case["capture_input_bytes"],
                "sha256": case["capture_input_sha256"],
            }
            or oracle.get("exit_status") != case["oracle_exit_status"]
            or oracle.get("stdout_bytes") != case["oracle_stdout"]["bytes"]
            or oracle.get("stdout_sha256") != case["oracle_stdout"]["sha256"]
            or oracle.get("stderr_bytes") != case["oracle_stderr"]["bytes"]
            or oracle.get("stderr_sha256") != case["oracle_stderr"]["sha256"]
        ):
            raise ProvenanceError("inspection raw-capture binding changed")


def _diagnostic_blocks(raw):
    if raw == b"":
        return ()
    try:
        text = raw.decode("utf-8", errors="strict")
    except UnicodeDecodeError as error:
        raise ProvenanceError("oracle stderr is not UTF-8") from error
    if not text.endswith("\n"):
        raise ProvenanceError("oracle diagnostic stream lacks final newline")
    lines = text[:-1].split("\n")
    if len(lines) % 3:
        raise ProvenanceError("oracle diagnostics are not complete blocks")
    result = []
    for offset in range(0, len(lines), 3):
        header, message, ending = lines[offset : offset + 3]
        if header != "Runtime error:" or ending != "Evaluation aborted.":
            raise ProvenanceError("oracle diagnostic block changed")
        if not message.startswith("  ") or message == "  ":
            raise ProvenanceError("oracle diagnostic message changed")
        result.append("Runtime:" + message[2:])
    return tuple(result)


def validate_artifacts(catalog, artifacts):
    """Validate both fixtures and all four byte-exact oracle goldens."""
    validate_catalog(catalog)
    if type(artifacts) is not dict:
        raise ProvenanceError("regression artifacts must be an object")
    expected_names = set()
    for case in catalog["cases"]:
        expected_names.add(case["fixture"])
        expected_names.add(case["oracle_stdout"]["file"])
        expected_names.add(case["oracle_stderr"]["file"])
    if set(artifacts) != expected_names or not all(type(name) is str for name in artifacts):
        raise ProvenanceError("regression artifact set changed")
    if not all(type(raw) is bytes for raw in artifacts.values()):
        raise ProvenanceError("regression artifacts must contain bytes")

    for case in catalog["cases"]:
        case_id = case["id"]
        expected = _CASE_METADATA[case_id]
        fixture = artifacts[case["fixture"]]
        stdout = artifacts[case["oracle_stdout"]["file"]]
        stderr = artifacts[case["oracle_stderr"]["file"]]
        for raw, record in (
            (fixture, {"bytes": case["fixture_bytes"], "sha256": case["fixture_sha256"]}),
            (stdout, case["oracle_stdout"]),
            (stderr, case["oracle_stderr"]),
        ):
            if len(raw) != record["bytes"] or _sha(raw) != record["sha256"]:
                raise ProvenanceError("regression artifact bytes changed")

        capture_input = (
            ("prints(\"MATH_BEGIN " + case_id + "\")\n").encode()
            + fixture
            + ("prints(\"MATH_END " + case_id + "\")\nquit\n").encode()
        )
        if (
            len(capture_input) != case["capture_input_bytes"]
            or _sha(capture_input) != case["capture_input_sha256"]
        ):
            raise ProvenanceError("capture input envelope changed")

        begin = ("MATH_BEGIN " + case_id + "\n").encode()
        end = ("MATH_END " + case_id + "\nBye.\n").encode()
        if (
            not stdout.startswith(begin)
            or not stdout.endswith(end)
            or stdout.count(begin) != 1
            or stdout.count(("MATH_END " + case_id + "\n").encode()) != 1
            or stdout.count(expected["recovery"]) != 1
            or b"\r" in stdout
        ):
            raise ProvenanceError("oracle stdout envelope or recovery changed")
        if _diagnostic_blocks(stderr) != expected["ordered_diagnostics"]:
            raise ProvenanceError("ordered oracle diagnostics changed")
    if artifacts[_CASE_METADATA[COLD_CASE]["oracle_stderr"]["file"]] != b"":
        raise ProvenanceError("cold-dual oracle stderr must be empty")
    return artifacts


def expected_provenance():
    """Return a new exact tests-only source and capture provenance record."""
    return {
        "catalog_bytes": CATALOG_BYTES,
        "catalog_sha256": CATALOG_SHA256,
        "inspection_bytes": INSPECTION_BYTES,
        "inspection_sha256": INSPECTION_SHA256,
        "capture_job": CAPTURE_JOB,
        "capture_report_bytes": CAPTURE_REPORT_BYTES,
        "capture_report_sha256": CAPTURE_REPORT_SHA256,
        "oracle_commit": ORACLE_COMMIT,
        "oracle_binary_sha256": ORACLE_BINARY_SHA256,
        "parent_session_sha256": PARENT_SESSION_SHA256,
        "tests_only_session_sha256": TESTS_ONLY_SESSION_SHA256,
        "test_patch_sha256": TEST_PATCH_SHA256,
        "production_unchanged": True,
    }


def _validate_metrics(metrics):
    if type(metrics) is not dict or frozenset(metrics) != _METRIC_KEYS:
        raise HarnessError("GNU-time metric schema changed")
    if (
        metrics.get("format") != "gnu-time-v"
        or not _is_number(metrics.get("seconds"))
        or metrics["seconds"] < 0
        or not _is_number(metrics.get("user_cpu_seconds"))
        or metrics["user_cpu_seconds"] < 0
        or not _is_number(metrics.get("system_cpu_seconds"))
        or metrics["system_cpu_seconds"] < 0
        or not _is_int(metrics.get("maxrss_kb"))
        or metrics["maxrss_kb"] <= 0
        or metrics.get("maxrss_approximate") is not False
        or metrics.get("timed_out") is not False
        or metrics.get("termination_uncertain") is not False
        or metrics.get("signal") is not None
    ):
        raise HarnessError("GNU-time metrics are incomplete or uncertain")
    return metrics


def _validate_command(command, expected_name):
    if type(command) is not dict or frozenset(command) != _COMMAND_KEYS:
        raise HarnessError("command record schema changed")
    if (
        command.get("name") != expected_name
        or type(command.get("exit_status")) is not int
        or type(command.get("log")) is not bytes
    ):
        raise HarnessError("command identity or stream changed")
    _validate_metrics(command.get("metrics"))
    try:
        command["log"].decode("utf-8", errors="strict")
    except UnicodeDecodeError as error:
        raise HarnessError("command log is not UTF-8") from error
    return command


def _validate_observation_shape(
    observation, schema=OBSERVATION_SCHEMA, label="BEFORE"
):
    if type(observation) is not dict or frozenset(observation) != _OBSERVATION_KEYS:
        raise ProvenanceError(label + " observation schema changed")
    if observation.get("schema") != schema:
        raise ProvenanceError(label + " observation version changed")
    provenance = observation.get("provenance")
    if (
        type(provenance) is not dict
        or frozenset(provenance) != _PROVENANCE_KEYS
        or provenance != expected_provenance()
    ):
        raise ProvenanceError(label + " provenance changed")
    claims = observation.get("claims")
    if (
        type(claims) is not dict
        or frozenset(claims) != _CLAIM_KEYS
        or any(value is not False for value in claims.values())
    ):
        raise HarnessError(label + " observation attempts to release a gate")


def _validate_original_reruns(runs, catalog, artifacts):
    if type(runs) is not list or len(runs) != len(CASE_IDS):
        raise ProvenanceError("original rerun set changed")
    if (
        any(type(run) is not dict for run in runs)
        or [run.get("case_id") for run in runs] != list(CASE_IDS)
    ):
        raise ProvenanceError("original rerun order changed")
    cases = {case["id"]: case for case in catalog["cases"]}
    for run in runs:
        if type(run) is not dict or frozenset(run) != _ORIGINAL_RUN_KEYS:
            raise ProvenanceError("original rerun schema changed")
        case = cases[run["case_id"]]
        if (
            run.get("engine") != "oracle"
            or type(run.get("exit_status")) is not int
            or run["exit_status"] != case["oracle_exit_status"]
            or type(run.get("stdout")) is not bytes
            or type(run.get("stderr")) is not bytes
            or run["stdout"] != artifacts[case["oracle_stdout"]["file"]]
            or run["stderr"] != artifacts[case["oracle_stderr"]["file"]]
            or run.get("fresh_process") is not True
            or run.get("timed_out") is not False
            or run.get("termination_uncertain") is not False
        ):
            raise ProvenanceError("original rerun differs from frozen golden")
        try:
            _validate_metrics(run.get("metrics"))
        except HarnessError as error:
            raise ProvenanceError("original rerun metrics are incomplete") from error


def _test_result(log, outcome):
    pattern = re.compile(
        rb"(?m)^test result: " + outcome.encode()
        + rb"\. (\d+) passed; (\d+) failed; (\d+) ignored; "
          rb"(\d+) measured; (\d+) filtered out(?:;[^\r\n]*)?$"
    )
    matches = pattern.findall(log)
    if len(matches) != 1:
        raise HarnessError("test result summary is missing or duplicated")
    return tuple(int(value) for value in matches[0])


def _ready_records(log):
    matches = re.findall(
        rb"(?m)^WEYL_CONTEXT_CORE_READY ([a-z0-9_]+) diagnostics=(\d+)$",
        log,
    )
    if len(matches) != 2:
        raise HarnessError("Weyl-context READY records changed")
    records = [(case.decode("ascii"), int(count)) for case, count in matches]
    if [case for case, _count in records] != list(CASE_IDS):
        raise HarnessError("Weyl-context READY order changed")
    return dict(records)


def _selector_state(command):
    _validate_command(command, "weyl-context-regressions")
    log = command["log"]
    ready = _ready_records(log)
    panics = re.findall(rb"(?m)^thread '([^']+)' (?:\(\d+\) )?panicked at", log)
    panic_names = tuple(name.decode("utf-8") for name in panics)

    if command["exit_status"] == 101:
        if (
            _test_result(log, "FAILED") != (0, 2, 0, 0, 630)
            or set(panic_names) != set(SELECTOR_TESTS)
            or len(panic_names) != 2
            or ready != {COLD_CASE: 2, PREWARM_CASE: 4}
        ):
            raise HarnessError("selector did not reproduce exactly two regressions")
        return "expected_failures"

    if command["exit_status"] == 0:
        if (
            _test_result(log, "ok") != (2, 0, 0, 0, 630)
            or panic_names
            or ready != {COLD_CASE: 0, PREWARM_CASE: 8}
        ):
            raise HarnessError("zero-exit selector is not an exact regression pass")
        return "unexpected_pass"
    raise HarnessError("selector exit status changed")


def _validate_control(command):
    _validate_command(command, "root-ladder-control")
    log = command["log"]
    names = tuple(
        item.decode("utf-8")
        for item in re.findall(rb"(?m)^test ([^\r\n ]+) \.\.\. ok$", log)
    )
    if (
        command["exit_status"] != 0
        or names != (CONTROL_TEST,)
        or _test_result(log, "ok") != (1, 0, 0, 0, 631)
        or b"FAILED" in log
        or b"panicked at" in log
    ):
        raise HarnessError("retained root-ladder control did not pass exactly")


def _validate_inventory(command):
    _validate_command(command, "atlas-core-test-inventory")
    log = command["log"]
    if command["exit_status"] != 0:
        raise HarnessError("test inventory command failed")
    names = [
        item.decode("utf-8")
        for item in re.findall(rb"(?m)^([^\r\n]+): test$", log)
    ]
    required = set(SELECTOR_TESTS) | {CONTROL_TEST}
    if (
        len(names) != EXPECTED_INVENTORY_COUNT
        or len(set(names)) != EXPECTED_INVENTORY_COUNT
        or not required.issubset(names)
        or log.count(b"632 tests, 0 benchmarks") != 1
        or re.search(rb"(?m): benchmark$", log) is not None
    ):
        raise HarnessError("compiled atlas-core test inventory changed")
    return names


def _classification(status, reason):
    reproduced = status == BEFORE_REPRODUCED
    unexpected = status == UNEXPECTED_PASS
    return {
        "schema": CLASSIFICATION_SCHEMA,
        "status": status,
        "evidence_maturity": "tests_first_before",
        "reason": reason,
        "original_goldens_matched": reproduced or unexpected,
        "expected_regressions_observed": reproduced,
        "unexpected_regressions_passed": unexpected,
        "retained_control_passed": reproduced or unexpected,
        "inventory_complete": reproduced or unexpected,
        "metrics_complete": reproduced or unexpected,
        "acceptance_eligible": False,
        "math_gate_released": False,
        "cache_gate_released": False,
        "performance_gate_released": False,
        "rank_gate_released": False,
    }


def classify_before(catalog_raw, inspection_raw, artifacts, observation):
    """Classify one complete tests-first BEFORE observation.

    Frozen-byte and original-rerun failures are provenance failures.  Command,
    inventory, timing, control, and expected-failure-shape errors are harness
    failures.  A clean two-test pass is kept distinct: it means this source is
    not a valid failing BEFORE, even though the regression assertions pass.
    """
    try:
        catalog = decode_catalog(catalog_raw)
        decode_inspection(inspection_raw, catalog)
        validate_artifacts(catalog, artifacts)
        _validate_observation_shape(observation)
        _validate_original_reruns(
            observation["original_reruns"], catalog, artifacts
        )
    except ProvenanceError as error:
        return _classification(PROVENANCE_FAILURE, str(error))
    except HarnessError as error:
        return _classification(HARNESS_FAILURE, str(error))

    try:
        _validate_inventory(observation["inventory_command"])
        _validate_control(observation["retained_control_command"])
        selector = _selector_state(observation["selector_command"])
    except HarnessError as error:
        return _classification(HARNESS_FAILURE, str(error))

    if selector == "unexpected_pass":
        return _classification(
            UNEXPECTED_PASS,
            "both original-backed regressions passed, so this is not a failing BEFORE",
        )
    return _classification(
        BEFORE_REPRODUCED,
        "original goldens matched and exactly the two known Rust regressions failed",
    )


# --- AFTER classification -------------------------------------------------
#
# The AFTER gate reruns the same tests-first gate on the repaired source.  A
# successful AFTER proves that exactly the two known regressions now pass and
# that the repaired Rust matches every frozen original golden; it releases no
# mathematical, cache, performance, memory, or rank claim either.

AFTER_OBSERVATION_SCHEMA = (
    "atlas-weyl-context-core-regression-after-observation-v1"
)
AFTER_CLASSIFICATION_SCHEMA = (
    "atlas-weyl-context-core-regression-after-classification-v1"
)
AFTER_REPRODUCED = "WEYL_CONTEXT_AFTER_REGRESSIONS_PASS"
AFTER_STILL_FAILING = "WEYL_CONTEXT_AFTER_REGRESSIONS_STILL_FAIL"
AFTER_PROVENANCE_FAILURE = "WEYL_CONTEXT_AFTER_PROVENANCE_FAILURE"
AFTER_HARNESS_FAILURE = "WEYL_CONTEXT_AFTER_HARNESS_FAILURE"


def _selector_state_after(command):
    _validate_command(command, "weyl-context-regressions")
    log = command["log"]
    ready = _ready_records(log)
    panics = re.findall(rb"(?m)^thread '([^']+)' (?:\(\d+\) )?panicked at", log)
    panic_names = tuple(name.decode("utf-8") for name in panics)

    if command["exit_status"] == 101:
        if (
            _test_result(log, "FAILED") != (0, 2, 0, 0, 630)
            or set(panic_names) != set(SELECTOR_TESTS)
            or len(panic_names) != 2
            or ready != {COLD_CASE: 2, PREWARM_CASE: 4}
        ):
            raise HarnessError("AFTER selector failure shape changed")
        return "still_failing"

    if command["exit_status"] == 0:
        if (
            _test_result(log, "ok") != (2, 0, 0, 0, 630)
            or panic_names
            or ready != {COLD_CASE: 0, PREWARM_CASE: 8}
        ):
            raise HarnessError("AFTER selector pass shape changed")
        return "expected_passes"
    raise HarnessError("AFTER selector exit status changed")


def _classification_after(status, reason):
    reproduced = status == AFTER_REPRODUCED
    still_failing = status == AFTER_STILL_FAILING
    return {
        "schema": AFTER_CLASSIFICATION_SCHEMA,
        "status": status,
        "evidence_maturity": "tests_first_after",
        "reason": reason,
        "original_goldens_matched": reproduced or still_failing,
        "expected_regressions_passed": reproduced,
        "regressions_still_failing": still_failing,
        "retained_control_passed": reproduced or still_failing,
        "inventory_complete": reproduced or still_failing,
        "metrics_complete": reproduced or still_failing,
        "acceptance_eligible": False,
        "math_gate_released": False,
        "cache_gate_released": False,
        "performance_gate_released": False,
        "rank_gate_released": False,
    }


def classify_after(catalog_raw, inspection_raw, artifacts, observation):
    """Classify one complete tests-first AFTER observation.

    Frozen-byte and original-rerun failures are provenance failures.  Command,
    inventory, timing, control, and pass/failure-shape errors are harness
    failures.  A still-failing selector is a genuine result: the repair
    candidate did not fix the two regressions, so nothing is released.
    """
    try:
        catalog = decode_catalog(catalog_raw)
        decode_inspection(inspection_raw, catalog)
        validate_artifacts(catalog, artifacts)
        _validate_observation_shape(
            observation, schema=AFTER_OBSERVATION_SCHEMA, label="AFTER"
        )
        _validate_original_reruns(
            observation["original_reruns"], catalog, artifacts
        )
    except ProvenanceError as error:
        return _classification_after(AFTER_PROVENANCE_FAILURE, str(error))
    except HarnessError as error:
        return _classification_after(AFTER_HARNESS_FAILURE, str(error))

    try:
        _validate_inventory(observation["inventory_command"])
        _validate_control(observation["retained_control_command"])
        selector = _selector_state_after(observation["selector_command"])
    except HarnessError as error:
        return _classification_after(AFTER_HARNESS_FAILURE, str(error))

    if selector == "still_failing":
        return _classification_after(
            AFTER_STILL_FAILING,
            "original goldens matched but the two regressions still fail, "
            "so the repair candidate is not effective",
        )
    return _classification_after(
        AFTER_REPRODUCED,
        "original goldens matched and exactly the two known regressions "
        "now pass",
    )

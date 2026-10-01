"""Validate the append-only mathematical acceptance index; compute nodes only."""

from __future__ import annotations

import copy
import hashlib
import json
import os
import re
import stat
import unittest
from pathlib import Path, PurePosixPath
from typing import Any
from unittest import mock


REPO_ROOT = Path(__file__).resolve().parents[1]
INDEX_PATH = REPO_ROOT / "tests/reference/hpc/math_acceptance_index_2026_10_01.json"
SCHEMA = "atlas-math-acceptance-index-v1"
FROZEN_PREFIX_CHECKPOINTS = {
    2: "e44a8b6d8ed6d2a7781ca9f9e2ba4b2e57c06f118a5a288d7a61c0d77d022c6c",
    3: "1628ee21c71a91376a02982c38404cee958cb6183c57f791a42c4a668a1efc12",
}
SHA256_RE = re.compile(r"[0-9a-f]{64}\Z")
ENTRY_ID_RE = re.compile(r"[0-9]{4,}-[a-z0-9]+(?:-[a-z0-9]+)*\Z")
CLAIM_ID_RE = re.compile(r"[a-z0-9]+(?:_[a-z0-9]+)*\Z")
OPERATION_RE = re.compile(r"[a-z0-9]+(?:_[a-z0-9]+)*\Z")

TOP_LEVEL_KEYS = {"schema", "entries"}
ENTRY_KEYS = {
    "sequence",
    "entry_id",
    "previous_entry_sha256",
    "entry_sha256",
    "supersedes",
    "operation",
    "claim_id",
    "acceptance",
    "status",
    "source",
    "report",
    "review_evidence",
    "scope",
    "assertions",
    "limitations",
}
SOURCE_KEYS = {"kind", "sha256", "evidence"}
SOURCE_EVIDENCE_KEYS = {"file", "sha256", "json_pointer"}
FILE_REFERENCE_KEYS = {"file", "sha256"}
SCOPE_KEYS = {
    "collection_pointer",
    "where",
    "id_pointer",
    "count",
    "ordered_ids_sha256",
}
WHERE_KEYS = {"pointer", "values"}
ACCEPTANCE_VALUES = {"accepted", "review_pending"}
STATUS_VALUES = {
    "math_pass",
    "inventory_only",
    "rejected",
    "shared_failure",
    "math_mismatch",
    "rust_failure",
    "oracle_failure",
    "timeout",
    "harness_failure",
    "pending",
}

CLAIM_CONTRACTS = {
    ("sl2_finite_module_av_ann_zero_orbit", "math_pass"): {
        "operation": "av_ann",
        "scope": {
            "collection_pointer": "/results",
            "where": [
                {
                    "pointer": "/case/id",
                    "values": ["av_ann_rank1_finite_cycle"],
                },
                {"pointer": "/status", "values": ["FINITE_ANCHOR_MATCH"]},
            ],
            "id_pointer": "/case/id",
            "count": 1,
            "ordered_ids_sha256": "22e93eb8ce15429ce6322cc1b93cdc20cbfd169d0b10fbca60eedf2165c45bbf",
        },
        "assertions": [
            "annihilator_orbit_dimension_zero",
            "complete_tail_equal",
            "finite_module_dimensions_1_2_3_5",
            "gk_dimension_zero",
        ],
        "limitations": [
            "finite_dimensional_rank1_only",
            "not_generic_annihilator_variety",
            "not_higher_rank",
            "startup_prefix_not_equal",
        ],
    },
    ("sl2_finite_module_point_cycle_multiplicity", "math_pass"): {
        "operation": "associated_cycle",
        "scope": {
            "collection_pointer": "/results",
            "where": [
                {
                    "pointer": "/case/id",
                    "values": ["av_ann_rank1_finite_cycle"],
                },
                {"pointer": "/status", "values": ["FINITE_ANCHOR_MATCH"]},
            ],
            "id_pointer": "/case/id",
            "count": 1,
            "ordered_ids_sha256": "22e93eb8ce15429ce6322cc1b93cdc20cbfd169d0b10fbca60eedf2165c45bbf",
        },
        "assertions": [
            "complete_tail_equal",
            "derived_point_orbit_support",
            "finite_module_dimension_derivation",
            "point_cycle_multiplicities_1_2_3_5",
        ],
        "limitations": [
            "derived_not_interpreter_cycle_output",
            "finite_dimensional_point_orbit_only",
            "not_generic_associated_cycle",
            "not_higher_rank",
        ],
    },
    ("a1_torus_root_coroot_ladder_coordinate_boundary", "math_pass"): {
        "operation": "root_ladder",
        "scope": {
            "collection_pointer": "/commands",
            "where": [
                {
                    "pointer": "/name",
                    "values": ["core-focused", "domain-focused"],
                },
                {"pointer": "/exit_status", "values": [0]},
            ],
            "id_pointer": "/name",
            "count": 2,
            "ordered_ids_sha256": "38cb6b3bd9f184c50ab4698e7e514ea2fee89ace05a60363c2e431dd24246c16",
        },
        "assertions": [
            "captured_original_accepts_11_cases_22_rows",
            "coroot_ladder_coordinate_boundary_kernel_passes",
            "full_atlas_core_lib_suite_passes_630",
            "full_atlas_real_group_lib_suite_passes_521",
            "root_ladder_coordinate_boundary_kernel_passes",
            "rust_complete_stream_matches_captured_original",
            "tests_first_three_regressions_failed_before_fix",
        ],
        "limitations": [
            "a1_plus_central_torus_coordinate_boundary_fixture_only",
            "after_job_did_not_rerun_the_original_oracle",
            "not_generic_root_system_correctness",
            "not_higher_rank",
            "not_klv_unitarity_hodge_associated_cycle_or_av_ann",
            "not_performance_or_parallel_acceptance",
            "rust_full_suites_are_regression_guards_not_oracle_proofs",
        ],
    },
}

FROZEN_SEED_ENTRY_IDS = [
    "0001-finite-rank1-av-ann-anchor",
    "0002-finite-rank1-point-cycle-anchor",
]
FROZEN_SEED_OPERATIONS = [
    "av_ann",
    "associated_cycle",
]
FROZEN_SEED_CLAIMS = [
    "sl2_finite_module_av_ann_zero_orbit",
    "sl2_finite_module_point_cycle_multiplicity",
]


class ValidationError(ValueError):
    pass


def _object_without_duplicate_keys(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise ValidationError(f"duplicate JSON key: {key}")
        result[key] = value
    return result


def strict_json_loads(raw: str) -> Any:
    def reject_constant(value: str) -> None:
        raise ValidationError(f"non-finite JSON number: {value}")

    return json.loads(
        raw,
        object_pairs_hook=_object_without_duplicate_keys,
        parse_constant=reject_constant,
    )


def load_json(path: Path) -> Any:
    return strict_json_loads(path.read_text(encoding="utf-8"))


def canonical_json(value: Any) -> bytes:
    return json.dumps(
        value,
        ensure_ascii=False,
        allow_nan=False,
        sort_keys=True,
        separators=(",", ":"),
    ).encode("utf-8")


def sha256_bytes(raw: bytes) -> str:
    return hashlib.sha256(raw).hexdigest()


def require_exact_keys(value: Any, expected: set[str], label: str) -> None:
    if not isinstance(value, dict):
        raise ValidationError(f"{label} must be an object")
    actual = set(value)
    if actual != expected:
        raise ValidationError(
            f"{label} keys differ: missing={sorted(expected - actual)}, "
            f"extra={sorted(actual - expected)}"
        )


def require_sha256(value: Any, label: str) -> str:
    if not isinstance(value, str) or SHA256_RE.fullmatch(value) is None:
        raise ValidationError(f"{label} is not a lowercase SHA-256")
    return value


def resolve_repository_file(relative: Any, label: str) -> Path:
    if not isinstance(relative, str) or not relative:
        raise ValidationError(f"{label} must be a nonempty string")
    pure = PurePosixPath(relative)
    if pure.is_absolute() or pure.as_posix() != relative or ".." in pure.parts:
        raise ValidationError(f"{label} is not a repository-relative POSIX path")
    candidate = REPO_ROOT
    for part in pure.parts:
        candidate = candidate / part
        if candidate.is_symlink():
            raise ValidationError(f"{label} traverses a symbolic link")
    if candidate.is_symlink() or not candidate.is_file():
        raise ValidationError(f"{label} must name a regular non-symlink file")
    if os.path.commonpath((str(REPO_ROOT), str(candidate.resolve()))) != str(REPO_ROOT):
        raise ValidationError(f"{label} escapes the repository")
    return candidate


def verify_file_reference(reference: Any, label: str) -> tuple[Path, dict[str, Any]]:
    require_exact_keys(reference, FILE_REFERENCE_KEYS, label)
    path = resolve_repository_file(reference["file"], f"{label}.file")
    expected = require_sha256(reference["sha256"], f"{label}.sha256")
    flags = os.O_RDONLY | getattr(os, "O_CLOEXEC", 0) | getattr(os, "O_NOFOLLOW", 0)
    descriptor = os.open(path, flags)
    try:
        before = os.fstat(descriptor)
        if not stat.S_ISREG(before.st_mode):
            raise ValidationError(f"{label} must resolve to a regular file")
        with os.fdopen(os.dup(descriptor), "rb") as stream:
            raw = stream.read()
        after = os.fstat(descriptor)
    finally:
        os.close(descriptor)
    stable_fields = ("st_dev", "st_ino", "st_size", "st_mtime_ns", "st_ctime_ns")
    if any(getattr(before, field) != getattr(after, field) for field in stable_fields):
        raise ValidationError(f"{label} changed while it was read")
    actual = sha256_bytes(raw)
    if actual != expected:
        raise ValidationError(f"{label} hash drift: expected {expected}, got {actual}")
    return path, strict_json_loads(raw.decode("utf-8"))


def json_pointer(document: Any, pointer: Any, label: str) -> Any:
    if not isinstance(pointer, str) or (pointer and not pointer.startswith("/")):
        raise ValidationError(f"{label} is not a JSON pointer")
    current = document
    if pointer == "":
        return current
    for raw_part in pointer[1:].split("/"):
        part = raw_part.replace("~1", "/").replace("~0", "~")
        if isinstance(current, dict) and part in current:
            current = current[part]
        elif isinstance(current, list) and part.isdigit() and int(part) < len(current):
            current = current[int(part)]
        else:
            raise ValidationError(f"{label} does not resolve: {pointer}")
    return current


def entry_digest(entry: dict[str, Any]) -> str:
    payload = {key: value for key, value in entry.items() if key != "entry_sha256"}
    return sha256_bytes(canonical_json(payload))


def verify_scope(scope: Any, review: dict[str, Any], label: str) -> list[str]:
    require_exact_keys(scope, SCOPE_KEYS, label)
    collection = json_pointer(review, scope["collection_pointer"], f"{label}.collection")
    if not isinstance(collection, list):
        raise ValidationError(f"{label}.collection must resolve to an array")
    where = scope["where"]
    if not isinstance(where, list) or not where:
        raise ValidationError(f"{label}.where must be a nonempty array")
    selected = collection
    for index, condition in enumerate(where):
        require_exact_keys(condition, WHERE_KEYS, f"{label}.where[{index}]")
        values = condition["values"]
        if (
            not isinstance(values, list)
            or not values
            or values != sorted(values, key=lambda item: canonical_json(item))
            or len({canonical_json(item) for item in values}) != len(values)
        ):
            raise ValidationError(f"{label}.where[{index}].values must be sorted and unique")
        allowed = {canonical_json(item) for item in values}
        selected = [
            item
            for item in selected
            if canonical_json(
                json_pointer(item, condition["pointer"], f"{label}.where[{index}]")
            )
            in allowed
        ]
    if type(scope["count"]) is not int or scope["count"] < 1:
        raise ValidationError(f"{label}.count must be a positive integer")
    if len(selected) != scope["count"]:
        raise ValidationError(
            f"{label}.count differs: expected {scope['count']}, selected {len(selected)}"
        )
    ids = [json_pointer(item, scope["id_pointer"], f"{label}.id_pointer") for item in selected]
    if not all(isinstance(item, str) and item for item in ids) or len(set(ids)) != len(ids):
        raise ValidationError(f"{label} selected IDs must be nonempty and unique")
    expected_digest = require_sha256(
        scope["ordered_ids_sha256"], f"{label}.ordered_ids_sha256"
    )
    actual_digest = sha256_bytes(canonical_json(ids))
    if actual_digest != expected_digest:
        raise ValidationError(
            f"{label} ordered ID digest differs: expected {expected_digest}, got {actual_digest}"
        )
    return ids


def validate_finite_anchor_review(
    entry: dict[str, Any],
    report: dict[str, Any],
    review: dict[str, Any],
    source_document: dict[str, Any],
) -> None:
    if report.get("schema") != "atlas-cycle-rank1-v1" or report.get("job") != "3855999":
        raise ValidationError("finite-anchor execution report identity differs")
    if report.get("status") != "FINITE_CYCLE_RANK1_CAPTURE_REVIEW_REQUIRED":
        raise ValidationError("finite-anchor capture classification differs")
    if report.get("checker", {}).get("exit_status") != 0:
        raise ValidationError("finite-anchor capture checker did not pass")
    if report.get("integrity_rechecked") is not True:
        raise ValidationError("finite-anchor capture did not recheck integrity")
    if review.get("schema") != "atlas-cycle-rank1-review-v1":
        raise ValidationError("finite-anchor review schema differs")
    if review.get("job") != "3856006":
        raise ValidationError("finite-anchor review job differs")
    if review.get("status") != "FINITE_ANCHORS_REVIEWED_NOT_GENERAL_CYCLES":
        raise ValidationError("finite-anchor review classification differs")
    if review.get("mathematics_rerun") is not False:
        raise ValidationError("finite-anchor review must remain a review of the pinned capture")
    if review.get("checker", {}).get("exit_status") != 0:
        raise ValidationError("finite-anchor independent checker did not pass")
    if review.get("integrity_rechecked") is not True:
        raise ValidationError("finite-anchor review did not recheck integrity")
    report_sha256 = entry["report"]["sha256"]
    if review.get("pin", {}).get("capture", {}).get("sha256") != report_sha256:
        raise ValidationError("review does not bind the execution-report SHA-256")
    if review.get("capture", {}).get("sha256") != report_sha256:
        raise ValidationError("review capture identity differs from the indexed report")
    source_evidence_sha256 = entry["source"]["evidence"]["sha256"]
    review_source = review.get("pin", {}).get("deformation_after", {})
    report_source = report.get("pin", {}).get("deformation_after", {})
    if review_source.get("sha256") != source_evidence_sha256:
        raise ValidationError("review does not bind the executed source-evidence report")
    if report_source.get("sha256") != source_evidence_sha256:
        raise ValidationError("execution report does not bind the source-evidence report")
    if (
        source_document.get("schema") != "atlas-full-deform-after-v1"
        or source_document.get("job") != "3855872"
        or source_document.get("source_integrity_rechecked") is not True
    ):
        raise ValidationError("finite-anchor source-evidence identity differs")
    executed_binary = report.get("binaries", {}).get("rust", {}).get("sha256")
    source_binary = source_document.get("binary", {}).get("sha256")
    require_sha256(executed_binary, "finite-anchor executed Rust binary")
    require_sha256(source_binary, "finite-anchor source-evidence Rust binary")
    if executed_binary != source_binary:
        raise ValidationError("executed Rust binary differs from the source-evidence binary")

    capture_rows = report.get("results", [])
    review_rows = review.get("results", [])
    if not isinstance(capture_rows, list) or not isinstance(review_rows, list):
        raise ValidationError("finite-anchor results must be arrays")
    try:
        capture_results = {result["case"]["id"]: result for result in capture_rows}
        results = {result["case"]["id"]: result for result in review_rows}
    except (KeyError, TypeError) as error:
        raise ValidationError("finite-anchor result identity is malformed") from error
    if len(capture_results) != len(capture_rows) or len(results) != len(review_rows):
        raise ValidationError("finite-anchor result IDs are duplicated")
    expected_ids = {"av_ann_rank1_finite_cycle", "av_ann_rank1_scaling_control"}
    if set(capture_results) != expected_ids or set(results) != expected_ids:
        raise ValidationError("finite-anchor review result IDs differ")
    for case_id in sorted(expected_ids):
        captured = capture_results[case_id]
        reviewed = results[case_id]
        if reviewed.get("previous_status") != captured.get("status"):
            raise ValidationError(f"finite-anchor {case_id} previous status is not bound")
        for field in ("case", "observations", "order"):
            if reviewed.get(field) != captured.get(field):
                raise ValidationError(f"finite-anchor {case_id} {field} differs")
    if any(result.get("status") != "FINITE_ANCHOR_MATCH" for result in results.values()):
        raise ValidationError("finite-anchor review contains a non-match")
    if any(result.get("complete_tail_equal") is not True for result in results.values()):
        raise ValidationError("finite-anchor complete tail differs")

    finite = results["av_ann_rank1_finite_cycle"]
    captured_finite = capture_results["av_ann_rank1_finite_cycle"]
    if captured_finite.get("status") != "FINITE_ANCHOR_MATCH":
        raise ValidationError("finite-module capture was not already a mathematical match")
    if captured_finite.get("complete_tail_equal") is not True:
        raise ValidationError("finite-module capture complete tail differs")
    if finite.get("inventories") != captured_finite.get("inventories"):
        raise ValidationError("finite-module inventories changed during review")
    oracle = finite.get("inventories", {}).get("oracle")
    rust = finite.get("inventories", {}).get("rust")
    if oracle != rust or not isinstance(oracle, list) or len(oracle) != 4:
        raise ValidationError("finite-module inventories are not four equal rows")
    expected_weights = [0, 1, 2, 4]
    expected_dimensions = [1, 2, 3, 5]
    if [row.get("highest_weight") for row in oracle] != expected_weights:
        raise ValidationError("finite-module highest weights differ")
    if [row.get("dimension") for row in oracle] != expected_dimensions:
        raise ValidationError("finite-module dimensions differ")
    if [row.get("derived_module_cycle_point_multiplicity") for row in oracle] != expected_dimensions:
        raise ValidationError("derived point-cycle multiplicities differ")
    if any(row.get("annihilator_orbit_dimension") != 0 for row in oracle):
        raise ValidationError("finite-module annihilator orbit is not the point orbit")
    if any(row.get("gk_dimension") != 0 for row in oracle):
        raise ValidationError("finite-module GK dimension differs from zero")


def validate_finite_anchor_capture(
    entry: dict[str, Any],
    report: dict[str, Any],
    source_document: dict[str, Any],
) -> None:
    if report.get("schema") != "atlas-cycle-rank1-v1" or report.get("job") != "3855999":
        raise ValidationError("finite-anchor execution report identity differs")
    if report.get("status") != "FINITE_CYCLE_RANK1_CAPTURE_REVIEW_REQUIRED":
        raise ValidationError("finite-anchor capture classification differs")
    if report.get("checker", {}).get("exit_status") != 0:
        raise ValidationError("finite-anchor capture checker did not pass")
    if report.get("integrity_rechecked") is not True:
        raise ValidationError("finite-anchor capture did not recheck integrity")
    source_evidence_sha256 = entry["source"]["evidence"]["sha256"]
    if report.get("pin", {}).get("deformation_after", {}).get("sha256") != source_evidence_sha256:
        raise ValidationError("execution report does not bind the source-evidence report")
    if (
        source_document.get("schema") != "atlas-full-deform-after-v1"
        or source_document.get("job") != "3855872"
        or source_document.get("source_integrity_rechecked") is not True
    ):
        raise ValidationError("finite-anchor source-evidence identity differs")
    executed_binary = report.get("binaries", {}).get("rust", {}).get("sha256")
    source_binary = source_document.get("binary", {}).get("sha256")
    require_sha256(executed_binary, "finite-anchor executed Rust binary")
    require_sha256(source_binary, "finite-anchor source-evidence Rust binary")
    if executed_binary != source_binary:
        raise ValidationError("executed Rust binary differs from the source-evidence binary")

    capture_rows = report.get("results", [])
    if not isinstance(capture_rows, list):
        raise ValidationError("finite-anchor capture results must be an array")
    try:
        capture_results = {result["case"]["id"]: result for result in capture_rows}
    except (KeyError, TypeError) as error:
        raise ValidationError("finite-anchor capture result identity is malformed") from error
    if len(capture_results) != len(capture_rows):
        raise ValidationError("finite-anchor capture result IDs are duplicated")
    expected_ids = {"av_ann_rank1_finite_cycle", "av_ann_rank1_scaling_control"}
    if set(capture_results) != expected_ids:
        raise ValidationError("finite-anchor capture result IDs differ")
    finite = capture_results["av_ann_rank1_finite_cycle"]
    if finite.get("status") != "FINITE_ANCHOR_MATCH":
        raise ValidationError("finite-module capture was not a mathematical match")
    if finite.get("complete_tail_equal") is not True:
        raise ValidationError("finite-module capture complete tail differs")
    scaling = capture_results["av_ann_rank1_scaling_control"]
    if scaling.get("status") != "ORACLE_INCOMPLETE_CONTROL":
        raise ValidationError("scaling-control capture status differs")
    if scaling.get("complete_tail_equal") is not False:
        raise ValidationError("scaling-control capture was incorrectly upgraded")

    oracle = finite.get("inventories", {}).get("oracle")
    rust = finite.get("inventories", {}).get("rust")
    if oracle != rust or not isinstance(oracle, list) or len(oracle) != 4:
        raise ValidationError("finite-module capture inventories are not four equal rows")
    expected_weights = [0, 1, 2, 4]
    expected_dimensions = [1, 2, 3, 5]
    if [row.get("highest_weight") for row in oracle] != expected_weights:
        raise ValidationError("finite-module capture highest weights differ")
    if [row.get("dimension") for row in oracle] != expected_dimensions:
        raise ValidationError("finite-module capture dimensions differ")
    if [row.get("derived_module_cycle_point_multiplicity") for row in oracle] != expected_dimensions:
        raise ValidationError("finite-module capture point-cycle multiplicities differ")
    if any(row.get("annihilator_orbit_dimension") != 0 for row in oracle):
        raise ValidationError("finite-module capture annihilator orbit is not the point orbit")
    if any(row.get("gk_dimension") != 0 for row in oracle):
        raise ValidationError("finite-module capture GK dimension differs from zero")


LADDER_CLAIM = (
    "a1_torus_root_coroot_ladder_coordinate_boundary",
    "math_pass",
)
LADDER_REPORT_FILE = (
    "tests/reference/hpc/math_ladder_boundary_after_v3_report_2026_10_01.json"
)
LADDER_REPORT_SHA256 = (
    "771fc790dd4340f408235e50c3f6eee754ebe4c850cbad36d4af4f902a24627c"
)
LADDER_INSPECTION_FILE = (
    "tests/reference/hpc/math_ladder_boundary_after_v3_2026_10_01.json"
)
LADDER_INSPECTION_SHA256 = (
    "a459fa08117ff8a721181d349467ebdd9267e798cd25b5bcb1380996c2d15e15"
)
LADDER_BEFORE_FILE = (
    "tests/reference/hpc/math_ladder_boundary_before_v3_2026_10_01.json"
)
LADDER_BEFORE_SHA256 = (
    "608e996acac10ea1bacca177468fcd83f3ec39c3e579899440e11aab674fc37f"
)
LADDER_ORIGINAL_FILE = (
    "tests/reference/hpc/math_weyl_context_capture_2026_09_30.json"
)
LADDER_ORIGINAL_SHA256 = (
    "bdee12811fe6ce4160ffbc2a25df18d9a9338a7b29f22d5fcd19352cf970f832"
)
LADDER_SOURCE_HASHES = {
    "crates/atlas-core/src/session.rs": (
        "cef588d9afc7fcd513ee1660758b229a8767c252adacdec7feea9737675de6aa"
    ),
    "crates/atlas-real-group/src/root_system.rs": (
        "cc6a1764e1c2425f7de8b4c27ca34a8bdc855c764d6db2a6ab9d6092e7b8cfe9"
    ),
    "tests/math/generics/root_ladder_coordinate_boundary.atlas": (
        "dc88d6606ae855b618dcf12589ecde82edcbe482873a94bcff1001163691ec65"
    ),
    "tests/math/generics/root_ladder_coordinate_boundary.oracle.stdout": (
        "3a7fdade43c46cf4f3048b52cf81f282db060012951ee559296f017f7d3eab80"
    ),
    "tests/math/generics/root_ladder_coordinate_boundary.oracle.stderr": (
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    ),
}
LADDER_PATCH_HASHES = {
    "hpc/patches/ladder_boundary_fix.patch": (
        "cada2e341bfba80ee587acb966413ce8cb6297593d6394b93890e02cdd730e75"
    ),
    "hpc/patches/ladder_boundary_tests.patch": (
        "69676d60b16591851570f58bc62ddc3bfe3a80b0ec6dad64a8eff36a9f777ca9"
    ),
}


def ladder_focused_commands(document: dict[str, Any], label: str) -> list[dict[str, Any]]:
    commands = document.get("commands")
    if not isinstance(commands, list):
        raise ValidationError(f"{label} commands must be an array")
    selected = [
        command
        for command in commands
        if isinstance(command, dict)
        and command.get("name") in {"domain-focused", "core-focused"}
    ]
    if [command.get("name") for command in selected] != [
        "domain-focused",
        "core-focused",
    ]:
        raise ValidationError(f"{label} focused command order differs")
    return selected


def validate_ladder_capture(
    entry: dict[str, Any],
    report: dict[str, Any],
    source_document: dict[str, Any],
) -> None:
    if entry.get("report") != {
        "file": LADDER_REPORT_FILE,
        "sha256": LADDER_REPORT_SHA256,
    }:
        raise ValidationError("ladder indexed execution report reference differs")
    if source_document != report:
        raise ValidationError("ladder source evidence differs from the execution report")
    if (
        report.get("schema") != "atlas-ladder-boundary-after-v3"
        or report.get("job") != "3875239"
        or report.get("node") != "cu105"
        or report.get("status") != "LADDER_BOUNDARY_AFTER_GATES_PASS"
    ):
        raise ValidationError("ladder execution report identity differs")
    if (
        report.get("integrity_rechecked") is not True
        or report.get("source_integrity_rechecked") is not True
        or report.get("production_patch_only") is not True
        or report.get("ephemeral_workspace_verified") is not True
        or type(report.get("legacy_path_open_attempts")) is not int
        or report.get("legacy_path_open_attempts") != 0
    ):
        raise ValidationError("ladder execution integrity boundary differs")
    if report.get("patch_hashes") != LADDER_PATCH_HASHES:
        raise ValidationError("ladder repair patch identity differs")
    source_files = report.get("source_files")
    if not isinstance(source_files, dict) or len(source_files) != 1561:
        raise ValidationError("ladder final source manifest differs")
    if any(source_files.get(name) != sha for name, sha in LADDER_SOURCE_HASHES.items()):
        raise ValidationError("ladder source or original fixture identity differs")
    if entry["source"]["sha256"] != sha256_bytes(canonical_json(source_files)):
        raise ValidationError("ladder indexed source digest differs")
    expected_after_regressions = {
        "core": {"failed": 0, "filtered_out": 629, "ignored": 0, "passed": 1},
        "domain": {"failed": 0, "filtered_out": 519, "ignored": 0, "passed": 2},
    }
    if canonical_json(report.get("after_regressions")) != canonical_json(
        expected_after_regressions
    ):
        raise ValidationError("ladder focused regression result differs")
    expected_full_suites = {
        "core": {"failed": 0, "filtered_out": 0, "ignored": 0, "passed": 630},
        "domain": {"failed": 0, "filtered_out": 0, "ignored": 0, "passed": 521},
    }
    if canonical_json(report.get("full_suites")) != canonical_json(
        expected_full_suites
    ):
        raise ValidationError("ladder complete-suite result differs")
    inventories = report.get("inventories")
    if (
        not isinstance(inventories, dict)
        or not isinstance(inventories.get("core"), list)
        or not isinstance(inventories.get("domain"), list)
        or len(inventories["core"]) != 630
        or len(set(inventories["core"])) != 630
        or len(inventories["domain"]) != 521
        or len(set(inventories["domain"])) != 521
        or "session::tests::root_ladder_coordinate_boundary_original"
        not in inventories["core"]
        or "root_system::tests::ladder_coordinate_boundary_roots"
        not in inventories["domain"]
        or "root_system::tests::ladder_coordinate_boundary_coroots"
        not in inventories["domain"]
    ):
        raise ValidationError("ladder test inventory differs")
    commands = report.get("commands")
    if (
        not isinstance(commands, list)
        or len(commands) != 16
        or len({command.get("name") for command in commands if isinstance(command, dict)})
        != 16
        or any(
            not isinstance(command, dict)
            or type(command.get("exit_status")) is not int
            or command.get("exit_status") != 0
            or type(command.get("maxrss_kb")) is not int
            or command.get("maxrss_kb") < 0
            for command in commands
        )
    ):
        raise ValidationError("ladder command evidence differs")
    ladder_focused_commands(report, "ladder execution")
    before_reference = {
        "file": LADDER_BEFORE_FILE,
        "sha256": LADDER_BEFORE_SHA256,
    }
    _, before = verify_file_reference(before_reference, "ladder BEFORE evidence")
    if (
        before.get("schema") != "atlas-ladder-boundary-before-inspection-v1"
        or before.get("status") != "LADDER_BOUNDARY_BEFORE_ACCEPTED"
        or before.get("accounting", {}).get("job") != "3873400"
        or before.get("accounting", {}).get("state") != "COMPLETED"
        or before.get("accounting", {}).get("exit_code") != "0:0"
        or canonical_json(before.get("before_regressions"))
        != canonical_json({
            "core": {
                "exit_status": 101,
                "failed": 1,
                "filtered_out": 629,
                "ignored": 0,
                "name": "session::tests::root_ladder_coordinate_boundary_original",
                "passed": 0,
            },
            "domain": {
                "exit_status": 101,
                "failed": 2,
                "filtered_out": 519,
                "ignored": 0,
                "names": [
                    "root_system::tests::ladder_coordinate_boundary_coroots",
                    "root_system::tests::ladder_coordinate_boundary_roots",
                ],
                "passed": 0,
            },
        })
        or before.get("inspection", {}).get("production_unchanged") is not True
        or before.get("inspection", {}).get("source_integrity_rechecked") is not True
    ):
        raise ValidationError("ladder tests-first BEFORE evidence differs")
    original_reference = {
        "file": LADDER_ORIGINAL_FILE,
        "sha256": LADDER_ORIGINAL_SHA256,
    }
    _, original = verify_file_reference(
        original_reference, "ladder original discovery capture"
    )
    captures = original.get("captures")
    if (
        original.get("schema") != "atlas-weyl-context-capture-v1"
        or original.get("status") != "WEYL_CONTEXT_DISCOVERY_CAPTURED"
        or original.get("job") != "3868832"
        or original.get("integrity_rechecked") is not True
        or not isinstance(captures, list)
    ):
        raise ValidationError("ladder original discovery identity differs")
    selected = [
        row
        for row in captures
        if isinstance(row, dict)
        and row.get("case", {}).get("id")
        == "generic_root_ladder_coordinate_boundary"
    ]
    if len(selected) != 1:
        raise ValidationError("ladder original discovery case is not unique")
    original_case = selected[0]
    comparison = original_case.get("comparison", {})
    oracle = original_case.get("observations", {}).get("oracle", {})
    if (
        original_case.get("case", {}).get("fixture_sha256")
        != LADDER_SOURCE_HASHES[
            "tests/math/generics/root_ladder_coordinate_boundary.atlas"
        ]
        or comparison.get("categories", {}).get("oracle") != "ACCEPTED"
        or comparison.get("complete_frames", {}).get("oracle") is not True
        or type(comparison.get("details", {}).get("oracle", {}).get("datums"))
        is not int
        or comparison.get("details", {}).get("oracle", {}).get("datums") != 11
        or type(comparison.get("details", {}).get("oracle", {}).get("rows"))
        is not int
        or comparison.get("details", {}).get("oracle", {}).get("rows") != 22
        or type(oracle.get("exit_status")) is not int
        or oracle.get("exit_status") != 0
        or oracle.get("timed_out") is not False
        or oracle.get("termination_uncertain") is not False
        or oracle.get("artifacts", {}).get("stdout", {}).get("sha256")
        != LADDER_SOURCE_HASHES[
            "tests/math/generics/root_ladder_coordinate_boundary.oracle.stdout"
        ]
        or oracle.get("artifacts", {}).get("stderr", {}).get("sha256")
        != LADDER_SOURCE_HASHES[
            "tests/math/generics/root_ladder_coordinate_boundary.oracle.stderr"
        ]
    ):
        raise ValidationError("ladder original accepted boundary differs")


def validate_ladder_review(
    entry: dict[str, Any],
    report: dict[str, Any],
    review: dict[str, Any],
    source_document: dict[str, Any],
) -> None:
    validate_ladder_capture(entry, report, source_document)
    if (
        review.get("schema") != "atlas-root-ladder-acceptance-review-v1"
        or review.get("status") != "ROOT_LADDER_COORDINATE_BOUNDARY_REVIEWED"
        or review.get("job") != "3875239"
        or review.get("mathematics_rerun") is not False
        or review.get("operation") != entry["operation"]
        or review.get("claim_id") != entry["claim_id"]
        or review.get("execution_report") != entry["report"]
        or review.get("source_manifest_sha256") != entry["source"]["sha256"]
        or review.get("assertions") != entry["assertions"]
        or review.get("limitations") != entry["limitations"]
    ):
        raise ValidationError("ladder independent review identity differs")
    expected_inspection_reference = {
        "file": LADDER_INSPECTION_FILE,
        "sha256": LADDER_INSPECTION_SHA256,
    }
    if review.get("independent_inspection") != expected_inspection_reference:
        raise ValidationError("ladder independent inspection reference differs")
    if review.get("before_evidence") != {
        "file": LADDER_BEFORE_FILE,
        "sha256": LADDER_BEFORE_SHA256,
    }:
        raise ValidationError("ladder review BEFORE evidence reference differs")
    if review.get("original_capture") != {
        "file": LADDER_ORIGINAL_FILE,
        "sha256": LADDER_ORIGINAL_SHA256,
    }:
        raise ValidationError("ladder review original capture reference differs")
    _, inspection = verify_file_reference(
        expected_inspection_reference, "ladder independent inspection"
    )
    if (
        inspection.get("schema") != "atlas-ladder-boundary-after-inspection-v3"
        or inspection.get("status") != "LADDER_BOUNDARY_AFTER_ACCEPTED"
        or inspection.get("accounting", {}).get("job") != "3875239"
        or inspection.get("accounting", {}).get("state") != "COMPLETED"
        or inspection.get("accounting", {}).get("exit_code") != "0:0"
        or inspection.get("artifacts", {}).get("report", {}).get("file_sha256")
        != entry["report"]["sha256"]
        or inspection.get("inspection", {}).get("source_integrity_rechecked")
        is not True
        or inspection.get("inspection", {}).get("ephemeral_workspace_absent")
        is not True
        or type(
            inspection.get("inspection", {}).get("legacy_path_open_attempts")
        ) is not int
        or inspection.get("inspection", {}).get("legacy_path_open_attempts") != 0
        or inspection.get("math", {}).get("final_status")
        != "LADDER_BOUNDARY_AFTER_GATES_PASS"
        or inspection.get("production_hashes")
        != {
            "crates/atlas-core/src/session.rs": LADDER_SOURCE_HASHES[
                "crates/atlas-core/src/session.rs"
            ],
            "crates/atlas-real-group/src/root_system.rs": LADDER_SOURCE_HASHES[
                "crates/atlas-real-group/src/root_system.rs"
            ],
        }
    ):
        raise ValidationError("ladder independent inspection content differs")
    if review.get("commands") != ladder_focused_commands(report, "ladder execution"):
        raise ValidationError("ladder reviewed focused commands differ")
    expected_math = {
        "original_case_id": "generic_root_ladder_coordinate_boundary",
        "original_datums": 11,
        "original_rows": 22,
        "original_complete_frame": True,
        "original_stdout_sha256": LADDER_SOURCE_HASHES[
            "tests/math/generics/root_ladder_coordinate_boundary.oracle.stdout"
        ],
        "before_failed_regressions": 3,
        "after_domain_regressions_passed": 2,
        "after_core_regressions_passed": 1,
        "after_domain_full_passed": 521,
        "after_core_full_passed": 630,
    }
    if canonical_json(review.get("math")) != canonical_json(expected_math):
        raise ValidationError("ladder reviewed mathematical scope differs")


FINITE_ANCHOR_CLAIMS = {
    ("sl2_finite_module_av_ann_zero_orbit", "math_pass"),
    ("sl2_finite_module_point_cycle_multiplicity", "math_pass"),
}
CLAIM_CAPTURE_VALIDATORS = {
    **{claim: validate_finite_anchor_capture for claim in FINITE_ANCHOR_CLAIMS},
    LADDER_CLAIM: validate_ladder_capture,
}
CLAIM_REVIEW_VALIDATORS = {
    **{claim: validate_finite_anchor_review for claim in FINITE_ANCHOR_CLAIMS},
    LADDER_CLAIM: validate_ladder_review,
}


def validate_index(
    document: Any,
    *,
    enforce_frozen_head: bool = True,
    frozen_prefix_checkpoints: dict[int, str] | None = None,
    allow_unpublished_tail: bool = False,
) -> None:
    require_exact_keys(document, TOP_LEVEL_KEYS, "index")
    if document["schema"] != SCHEMA:
        raise ValidationError("index schema differs")
    entries = document["entries"]
    if not isinstance(entries, list) or not entries:
        raise ValidationError("index entries must be a nonempty array")
    operations: list[str] = []
    claims: list[str] = []
    entry_ids: set[str] = set()
    seen_entries: dict[str, dict[str, Any]] = {}
    latest_by_claim: dict[str, str] = {}
    previous: str | None = None
    for offset, entry in enumerate(entries, start=1):
        label = f"entries[{offset - 1}]"
        require_exact_keys(entry, ENTRY_KEYS, label)
        if type(entry["sequence"]) is not int or entry["sequence"] != offset:
            raise ValidationError(f"{label}.sequence is not contiguous")
        entry_id = entry["entry_id"]
        if not isinstance(entry_id, str) or ENTRY_ID_RE.fullmatch(entry_id) is None:
            raise ValidationError(f"{label}.entry_id is invalid")
        if not entry_id.startswith(f"{offset:04d}-"):
            raise ValidationError(f"{label}.entry_id does not encode its sequence")
        if entry_id in entry_ids:
            raise ValidationError(f"{label}.entry_id is duplicated")
        entry_ids.add(entry_id)
        if entry["previous_entry_sha256"] != previous:
            raise ValidationError(f"{label}.previous_entry_sha256 breaks the chain")
        actual_entry_digest = entry_digest(entry)
        expected_entry_digest = require_sha256(entry["entry_sha256"], f"{label}.entry_sha256")
        if actual_entry_digest != expected_entry_digest:
            raise ValidationError(
                f"{label} digest differs: expected {expected_entry_digest}, got {actual_entry_digest}"
            )
        if entry["acceptance"] not in ACCEPTANCE_VALUES:
            raise ValidationError(f"{label}.acceptance is invalid")
        if entry["status"] not in STATUS_VALUES:
            raise ValidationError(f"{label}.status is invalid")
        operation = entry["operation"]
        if not isinstance(operation, str) or OPERATION_RE.fullmatch(operation) is None:
            raise ValidationError(f"{label}.operation is invalid")
        operations.append(operation)
        claim_id = entry["claim_id"]
        if not isinstance(claim_id, str) or CLAIM_ID_RE.fullmatch(claim_id) is None:
            raise ValidationError(f"{label}.claim_id is invalid")
        claims.append(claim_id)
        supersedes = entry["supersedes"]
        if not isinstance(supersedes, list) or supersedes != sorted(set(supersedes)):
            raise ValidationError(f"{label}.supersedes must be a sorted unique array")
        for superseded in supersedes:
            require_sha256(superseded, f"{label}.supersedes")
            if superseded not in seen_entries:
                raise ValidationError(f"{label}.supersedes does not reference an earlier entry")
            if seen_entries[superseded]["claim_id"] != claim_id:
                raise ValidationError(f"{label}.supersedes crosses claim lineages")
        prior_same_claim = latest_by_claim.get(claim_id)
        if prior_same_claim is not None:
            prior_entry = seen_entries[prior_same_claim]
            if prior_entry["operation"] != operation:
                raise ValidationError(f"{label} changes operation within a claim lineage")
            if prior_same_claim not in supersedes:
                raise ValidationError(
                    f"{label} repeats a claim without superseding its latest entry"
                )

        require_exact_keys(entry["source"], SOURCE_KEYS, f"{label}.source")
        source = entry["source"]
        if source["kind"] != "canonical_source_files_v1":
            raise ValidationError(f"{label}.source.kind differs")
        require_exact_keys(
            source["evidence"], SOURCE_EVIDENCE_KEYS, f"{label}.source.evidence"
        )
        evidence_ref = {
            "file": source["evidence"]["file"],
            "sha256": source["evidence"]["sha256"],
        }
        _, source_document = verify_file_reference(evidence_ref, f"{label}.source.evidence")
        source_manifest = json_pointer(
            source_document,
            source["evidence"]["json_pointer"],
            f"{label}.source.evidence.json_pointer",
        )
        actual_source_digest = sha256_bytes(canonical_json(source_manifest))
        expected_source_digest = require_sha256(source["sha256"], f"{label}.source.sha256")
        if actual_source_digest != expected_source_digest:
            raise ValidationError(
                f"{label}.source digest differs: expected {expected_source_digest}, "
                f"got {actual_source_digest}"
            )

        report_path, report = verify_file_reference(entry["report"], f"{label}.report")
        review_reference = entry["review_evidence"]
        if entry["acceptance"] == "accepted":
            if review_reference is None:
                raise ValidationError(f"{label} accepted without independent review evidence")
            review_path, review = verify_file_reference(
                review_reference, f"{label}.review_evidence"
            )
            if review_path == report_path:
                raise ValidationError(f"{label} uses the execution report as its own review")
        else:
            if review_reference is not None:
                raise ValidationError(f"{label} review_pending entry already has review evidence")
            review = report

        if (
            not isinstance(entry["assertions"], list)
            or not all(isinstance(item, str) and item for item in entry["assertions"])
            or entry["assertions"] != sorted(set(entry["assertions"]))
        ):
            raise ValidationError(f"{label}.assertions must be a sorted unique array")
        if (
            not isinstance(entry["limitations"], list)
            or not entry["limitations"]
            or not all(isinstance(item, str) and item for item in entry["limitations"])
        ):
            raise ValidationError(f"{label}.limitations must be nonempty")
        if entry["limitations"] != sorted(set(entry["limitations"])):
            raise ValidationError(f"{label}.limitations must be sorted and unique")

        claim_outcome = (claim_id, entry["status"])
        contract = CLAIM_CONTRACTS.get(claim_outcome)
        if contract is None or claim_outcome not in CLAIM_CAPTURE_VALIDATORS:
            raise ValidationError(f"{label} has no registered claim/outcome contract")
        for field in ("operation", "scope", "assertions", "limitations"):
            if entry[field] != contract[field]:
                raise ValidationError(
                    f"{label}.{field} differs from the registered claim contract"
                )
        report_ids = verify_scope(entry["scope"], report, f"{label}.report_scope")
        review_ids = verify_scope(entry["scope"], review, f"{label}.review_scope")
        if report_ids != review_ids:
            raise ValidationError(f"{label} report/review selected IDs differ")

        CLAIM_CAPTURE_VALIDATORS[claim_outcome](entry, report, source_document)
        if entry["acceptance"] == "accepted":
            if claim_outcome not in CLAIM_REVIEW_VALIDATORS:
                raise ValidationError(f"{label} accepted without a registered claim reviewer")
            CLAIM_REVIEW_VALIDATORS[claim_outcome](
                entry, report, review, source_document
            )
        seen_entries[expected_entry_digest] = entry
        latest_by_claim[claim_id] = expected_entry_digest
        previous = expected_entry_digest

    seed_count = len(FROZEN_SEED_ENTRY_IDS)
    if [entry["entry_id"] for entry in entries[:seed_count]] != FROZEN_SEED_ENTRY_IDS:
        raise ValidationError("reviewed v1 seed IDs are missing, duplicated, or reordered")
    if operations[:seed_count] != FROZEN_SEED_OPERATIONS:
        raise ValidationError("reviewed v1 seed operations are missing, duplicated, or reordered")
    if claims[:seed_count] != FROZEN_SEED_CLAIMS:
        raise ValidationError("reviewed v1 seed claims are missing, duplicated, or reordered")
    if enforce_frozen_head:
        checkpoints = dict(FROZEN_PREFIX_CHECKPOINTS)
        if frozen_prefix_checkpoints is not None:
            for length, expected_head in frozen_prefix_checkpoints.items():
                if length in checkpoints and checkpoints[length] != expected_head:
                    raise ValidationError(
                        f"frozen prefix checkpoint {length} cannot be replaced"
                    )
                checkpoints[length] = expected_head
        for length, expected_head in checkpoints.items():
            if type(length) is not int or length < 1:
                raise ValidationError("append-only prefix checkpoint length is invalid")
            require_sha256(expected_head, f"append-only prefix checkpoint {length}")
            if len(entries) < length or entries[length - 1]["entry_sha256"] != expected_head:
                raise ValidationError(f"append-only prefix checkpoint {length} differs")
        if not allow_unpublished_tail and max(checkpoints) != len(entries):
            raise ValidationError("ledger tail has no published prefix checkpoint")


def rehash_chain(document: dict[str, Any]) -> None:
    previous: str | None = None
    for sequence, entry in enumerate(document["entries"], start=1):
        entry["sequence"] = sequence
        entry["previous_entry_sha256"] = previous
        entry["entry_sha256"] = entry_digest(entry)
        previous = entry["entry_sha256"]


class MathAcceptanceIndex(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.index = load_json(INDEX_PATH)

    def test_published_index(self) -> None:
        validate_index(self.index)

    def test_duplicate_json_key_is_rejected(self) -> None:
        with self.assertRaises(ValidationError):
            strict_json_loads('{"schema":"a","schema":"b","entries":[]}')

    def test_entry_mutation_breaks_hash_chain(self) -> None:
        altered = copy.deepcopy(self.index)
        altered["entries"][0]["status"] = "pending"
        with self.assertRaises(ValidationError):
            validate_index(altered)

    def test_report_hash_drift_is_rejected(self) -> None:
        altered = copy.deepcopy(self.index)
        altered["entries"][0]["report"]["sha256"] = "0" * 64
        rehash_chain(altered)
        with self.assertRaises(ValidationError):
            validate_index(altered, enforce_frozen_head=False)

    def test_review_hash_drift_is_rejected(self) -> None:
        altered = copy.deepcopy(self.index)
        altered["entries"][0]["review_evidence"]["sha256"] = "0" * 64
        rehash_chain(altered)
        with self.assertRaises(ValidationError):
            validate_index(altered, enforce_frozen_head=False)

    def test_source_manifest_drift_is_rejected(self) -> None:
        altered = copy.deepcopy(self.index)
        altered["entries"][0]["source"]["sha256"] = "0" * 64
        rehash_chain(altered)
        with self.assertRaises(ValidationError):
            validate_index(altered, enforce_frozen_head=False)

    def test_accepted_entry_requires_distinct_review(self) -> None:
        missing = copy.deepcopy(self.index)
        missing["entries"][0]["review_evidence"] = None
        rehash_chain(missing)
        with self.assertRaises(ValidationError):
            validate_index(missing, enforce_frozen_head=False)

        self_reviewed = copy.deepcopy(self.index)
        self_reviewed["entries"][0]["review_evidence"] = copy.deepcopy(
            self_reviewed["entries"][0]["report"]
        )
        rehash_chain(self_reviewed)
        with self.assertRaises(ValidationError):
            validate_index(self_reviewed, enforce_frozen_head=False)

    def test_scope_expansion_is_rejected(self) -> None:
        altered = copy.deepcopy(self.index)
        altered["entries"][1]["scope"]["where"] = [
            {"pointer": "/status", "values": ["FINITE_ANCHOR_MATCH"]}
        ]
        altered["entries"][1]["scope"]["count"] = 2
        altered["entries"][1]["scope"]["ordered_ids_sha256"] = (
            "e6cf0ce85254c36ee24598ea76a0ca019b6d9f22645d06c0e0bd5b57583faa2c"
        )
        rehash_chain(altered)
        with self.assertRaises(ValidationError):
            validate_index(altered, enforce_frozen_head=False)

    def test_scope_matching_does_not_equate_boolean_false_with_zero(self) -> None:
        scope = {
            "collection_pointer": "/rows",
            "where": [{"pointer": "/exit_status", "values": [0]}],
            "id_pointer": "/id",
            "count": 1,
            "ordered_ids_sha256": (
                "5bf47690cfac2bb35c843b8e41e04e5b2cbdcf10a39721023670ff22b7ec4fba"
            ),
        }
        with self.assertRaises(ValidationError):
            verify_scope(
                scope,
                {"rows": [{"id": "false-is-not-zero", "exit_status": False}]},
                "test.boolean_scope",
            )

    def test_limitation_removal_is_rejected(self) -> None:
        altered = copy.deepcopy(self.index)
        altered["entries"][1]["limitations"].remove("not_generic_associated_cycle")
        rehash_chain(altered)
        with self.assertRaises(ValidationError):
            validate_index(altered, enforce_frozen_head=False)

    def test_path_escape_is_rejected(self) -> None:
        altered = copy.deepcopy(self.index)
        altered["entries"][0]["report"]["file"] = "../report.json"
        rehash_chain(altered)
        with self.assertRaises(ValidationError):
            validate_index(altered, enforce_frozen_head=False)

    def test_duplicate_entry_id_is_rejected(self) -> None:
        altered = copy.deepcopy(self.index)
        duplicate = copy.deepcopy(altered["entries"][0])
        duplicate["supersedes"] = [altered["entries"][0]["entry_sha256"]]
        altered["entries"].append(duplicate)
        rehash_chain(altered)
        with self.assertRaises(ValidationError):
            validate_index(altered, allow_unpublished_tail=True)

    def test_unknown_supersedes_target_is_rejected(self) -> None:
        altered = copy.deepcopy(self.index)
        successor = copy.deepcopy(altered["entries"][0])
        successor["entry_id"] = "0004-finite-rank1-av-ann-anchor-review"
        successor["supersedes"] = ["0" * 64]
        altered["entries"].append(successor)
        rehash_chain(altered)
        with self.assertRaises(ValidationError):
            validate_index(altered, allow_unpublished_tail=True)

    def test_unpublished_valid_suffix_requires_explicit_draft_mode(self) -> None:
        altered = copy.deepcopy(self.index)
        successor = copy.deepcopy(altered["entries"][0])
        successor["entry_id"] = "0004-finite-rank1-av-ann-anchor-review"
        successor["supersedes"] = [altered["entries"][0]["entry_sha256"]]
        altered["entries"].append(successor)
        rehash_chain(altered)
        with self.assertRaises(ValidationError):
            validate_index(altered)
        validate_index(altered, allow_unpublished_tail=True)

    def test_unregistered_pending_classifications_are_rejected(self) -> None:
        for status in ("inventory_only", "shared_failure", "pending"):
            with self.subTest(status=status):
                altered = copy.deepcopy(self.index)
                successor = copy.deepcopy(altered["entries"][0])
                successor["entry_id"] = f"0004-av-ann-{status.replace('_', '-')}"
                successor["claim_id"] = f"sl2_second_av_ann_{status}"
                successor["acceptance"] = "review_pending"
                successor["status"] = status
                successor["review_evidence"] = None
                successor["supersedes"] = []
                altered["entries"].append(successor)
                rehash_chain(altered)
                with self.assertRaises(ValidationError):
                    validate_index(altered, allow_unpublished_tail=True)

    def test_registered_math_pass_can_remain_review_pending(self) -> None:
        altered = copy.deepcopy(self.index)
        successor = copy.deepcopy(altered["entries"][0])
        successor["entry_id"] = "0004-finite-rank1-av-ann-review-pending"
        successor["acceptance"] = "review_pending"
        successor["review_evidence"] = None
        successor["supersedes"] = [altered["entries"][0]["entry_sha256"]]
        altered["entries"].append(successor)
        rehash_chain(altered)
        validate_index(altered, allow_unpublished_tail=True)

    def test_accepted_entry_runs_capture_and_review_validators(self) -> None:
        claim_outcome = (
            "sl2_finite_module_av_ann_zero_orbit",
            "math_pass",
        )
        original_capture = CLAIM_CAPTURE_VALIDATORS[claim_outcome]
        original_review = CLAIM_REVIEW_VALIDATORS[claim_outcome]
        calls: list[str] = []

        def capture(*args: Any) -> None:
            calls.append("capture")
            original_capture(*args)

        def review(*args: Any) -> None:
            calls.append("review")
            original_review(*args)

        with mock.patch.dict(CLAIM_CAPTURE_VALIDATORS, {claim_outcome: capture}):
            with mock.patch.dict(CLAIM_REVIEW_VALIDATORS, {claim_outcome: review}):
                validate_index(self.index)
        self.assertEqual(calls, ["capture", "review"])

    def test_unregistered_claim_cannot_be_accepted(self) -> None:
        altered = copy.deepcopy(self.index)
        successor = copy.deepcopy(altered["entries"][0])
        successor["entry_id"] = "0004-unregistered-av-ann-claim"
        successor["claim_id"] = "sl2_unregistered_av_ann_claim"
        successor["supersedes"] = []
        altered["entries"].append(successor)
        rehash_chain(altered)
        with self.assertRaises(ValidationError):
            validate_index(altered, allow_unpublished_tail=True)

    def test_supersedes_cannot_cross_claim_lineages(self) -> None:
        altered = copy.deepcopy(self.index)
        successor = copy.deepcopy(altered["entries"][0])
        successor["entry_id"] = "0004-other-av-ann-claim"
        successor["claim_id"] = "sl2_other_av_ann_claim"
        successor["acceptance"] = "review_pending"
        successor["status"] = "pending"
        successor["review_evidence"] = None
        successor["supersedes"] = [altered["entries"][0]["entry_sha256"]]
        altered["entries"].append(successor)
        rehash_chain(altered)
        with self.assertRaises(ValidationError):
            validate_index(altered, allow_unpublished_tail=True)

    def test_pending_inventory_and_shared_failure_cannot_relabel_seed(self) -> None:
        for field, value in (
            ("acceptance", "review_pending"),
            ("status", "inventory_only"),
            ("status", "shared_failure"),
        ):
            with self.subTest(field=field, value=value):
                altered = copy.deepcopy(self.index)
                altered["entries"][0][field] = value
                rehash_chain(altered)
                with self.assertRaises(ValidationError):
                    validate_index(altered, enforce_frozen_head=False)

    def test_boolean_scope_count_is_rejected(self) -> None:
        altered = copy.deepcopy(self.index)
        altered["entries"][0]["scope"]["count"] = True
        rehash_chain(altered)
        with self.assertRaises(ValidationError):
            validate_index(altered, enforce_frozen_head=False)

    def test_boolean_sequence_is_rejected(self) -> None:
        altered = copy.deepcopy(self.index)
        altered["entries"][0]["sequence"] = True
        altered["entries"][0]["entry_sha256"] = entry_digest(altered["entries"][0])
        altered["entries"][1]["previous_entry_sha256"] = altered["entries"][0][
            "entry_sha256"
        ]
        altered["entries"][1]["entry_sha256"] = entry_digest(altered["entries"][1])
        with self.assertRaises(ValidationError):
            validate_index(altered, enforce_frozen_head=False)

    def test_duplicate_review_result_id_is_rejected(self) -> None:
        entry = self.index["entries"][0]
        _, report = verify_file_reference(entry["report"], "test.report")
        _, review = verify_file_reference(entry["review_evidence"], "test.review")
        source_reference = {
            "file": entry["source"]["evidence"]["file"],
            "sha256": entry["source"]["evidence"]["sha256"],
        }
        _, source_document = verify_file_reference(source_reference, "test.source")
        review["results"].append(copy.deepcopy(review["results"][0]))
        with self.assertRaises(ValidationError):
            validate_finite_anchor_review(entry, report, review, source_document)

    def test_ladder_capture_result_tampering_is_rejected(self) -> None:
        entry = self.index["entries"][2]
        _, report = verify_file_reference(entry["report"], "test.ladder.report")
        report["after_regressions"]["domain"]["passed"] = 1
        with self.assertRaises(ValidationError):
            validate_ladder_capture(entry, report, report)

    def test_ladder_boolean_count_tampering_is_rejected(self) -> None:
        entry = self.index["entries"][2]
        _, report = verify_file_reference(entry["report"], "test.ladder.report")
        report["after_regressions"]["core"]["passed"] = True
        with self.assertRaises(ValidationError):
            validate_ladder_capture(entry, report, report)

    def test_ladder_report_and_review_cannot_be_rebound_together(self) -> None:
        entry = copy.deepcopy(self.index["entries"][2])
        _, report = verify_file_reference(entry["report"], "test.ladder.report")
        _, review = verify_file_reference(
            entry["review_evidence"], "test.ladder.review"
        )
        source_reference = {
            "file": entry["source"]["evidence"]["file"],
            "sha256": entry["source"]["evidence"]["sha256"],
        }
        _, source_document = verify_file_reference(
            source_reference, "test.ladder.source"
        )
        entry["report"] = {
            "file": "tests/reference/hpc/rebound-ladder-report.json",
            "sha256": "0" * 64,
        }
        review["execution_report"] = copy.deepcopy(entry["report"])
        with self.assertRaises(ValidationError):
            validate_ladder_review(entry, report, review, source_document)

    def test_ladder_review_result_tampering_is_rejected(self) -> None:
        entry = self.index["entries"][2]
        _, report = verify_file_reference(entry["report"], "test.ladder.report")
        _, review = verify_file_reference(
            entry["review_evidence"], "test.ladder.review"
        )
        source_reference = {
            "file": entry["source"]["evidence"]["file"],
            "sha256": entry["source"]["evidence"]["sha256"],
        }
        _, source_document = verify_file_reference(
            source_reference, "test.ladder.source"
        )
        review["math"]["original_rows"] = 21
        with self.assertRaises(ValidationError):
            validate_ladder_review(entry, report, review, source_document)

    def test_ladder_before_reference_tampering_is_rejected(self) -> None:
        entry = self.index["entries"][2]
        _, report = verify_file_reference(entry["report"], "test.ladder.report")
        _, review = verify_file_reference(
            entry["review_evidence"], "test.ladder.review"
        )
        review["before_evidence"]["sha256"] = "0" * 64
        with self.assertRaises(ValidationError):
            validate_ladder_review(entry, report, review, report)

    def test_ladder_original_reference_tampering_is_rejected(self) -> None:
        entry = self.index["entries"][2]
        _, report = verify_file_reference(entry["report"], "test.ladder.report")
        _, review = verify_file_reference(
            entry["review_evidence"], "test.ladder.review"
        )
        review["original_capture"]["sha256"] = "0" * 64
        with self.assertRaises(ValidationError):
            validate_ladder_review(entry, report, review, report)

    def test_ladder_focused_command_order_tampering_is_rejected(self) -> None:
        entry = self.index["entries"][2]
        _, report = verify_file_reference(entry["report"], "test.ladder.report")
        report["commands"].reverse()
        with self.assertRaises(ValidationError):
            validate_ladder_capture(entry, report, report)

    def test_ladder_duplicate_command_tampering_is_rejected(self) -> None:
        entry = self.index["entries"][2]
        _, report = verify_file_reference(entry["report"], "test.ladder.report")
        report["commands"].append(copy.deepcopy(report["commands"][0]))
        with self.assertRaises(ValidationError):
            validate_ladder_capture(entry, report, report)

    def test_ladder_claim_cannot_be_widened_to_a_rank_gate(self) -> None:
        altered = copy.deepcopy(self.index)
        altered["entries"][2]["limitations"].remove("not_higher_rank")
        rehash_chain(altered)
        with self.assertRaises(ValidationError):
            validate_index(altered, enforce_frozen_head=False)

    def test_builtin_prefix_checkpoint_cannot_be_replaced(self) -> None:
        with self.assertRaises(ValidationError):
            validate_index(
                self.index,
                frozen_prefix_checkpoints={2: "0" * 64},
            )

    def test_removing_published_tail_is_rejected(self) -> None:
        published = copy.deepcopy(self.index)
        successor = copy.deepcopy(published["entries"][0])
        successor["entry_id"] = "0004-finite-rank1-av-ann-anchor-review"
        successor["supersedes"] = [published["entries"][0]["entry_sha256"]]
        published["entries"].append(successor)
        rehash_chain(published)
        checkpoints = {4: published["entries"][3]["entry_sha256"]}
        validate_index(published, frozen_prefix_checkpoints=checkpoints)

        mutated = copy.deepcopy(published)
        mutated["entries"][3]["entry_id"] = (
            "0004-finite-rank1-av-ann-anchor-review-amended"
        )
        rehash_chain(mutated)
        with self.assertRaises(ValidationError):
            validate_index(mutated, frozen_prefix_checkpoints=checkpoints)

        altered = copy.deepcopy(published)
        altered["entries"].pop()
        with self.assertRaises(ValidationError):
            validate_index(altered, frozen_prefix_checkpoints=checkpoints)


if __name__ == "__main__":
    unittest.main()

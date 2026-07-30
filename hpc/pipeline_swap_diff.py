#!/usr/bin/env python3
"""Compare atlas-cli with the frozen typed-pipeline Atlas event fixtures.

The checked-in event files are the oracle.  This driver deliberately runs
only the Rust interpreter.  Fixture lines whose Rust builtins are not yet
implemented are omitted from the runnable input and recorded as explicit
pending coverage, so a partial port can never be reported as a full pass.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import pathlib
import re
import subprocess
import time
from dataclasses import dataclass
from typing import Any


PINNED_ATLAS_REVISION = "4d3e9449062a07c1c85f4e6df215eb6ccc0eeae9"
COMMIT_TOKEN = re.compile(r"^(?:[0-9a-fA-F]{40}|unversioned)$")
DIRTY_TREE_TOKENS = {"true", "false", "unknown"}


@dataclass(frozen=True)
class PendingCase:
    feature: str
    source_line: int
    reference_event: int
    reason: str


@dataclass(frozen=True)
class FixturePlan:
    name: str
    runnable_lines: tuple[int, ...] | None = None
    runnable_events: tuple[int, ...] | None = None
    pending: tuple[PendingCase, ...] = ()
    # Lines that are executed as part of the runnable input but produce no
    # observable event (for example output redirected to a file).
    silent_lines: tuple[int, ...] = ()


# These overloads are present in the Atlas oracle but are intentionally not
# registered until their owning Rust domain types and semantics are ported.
# They are coverage gaps, not fixture failures.
PENDING_OVERLOADS = (
    {
        "feature": "involution",
        "signature": "(LieType,[int],string) -> mat",
        "reason": "Rust overload is not implemented",
    },
    {
        "feature": "involution",
        "signature": "(LieType,mat,string) -> mat",
        "reason": "Rust overload is not implemented",
    },
    {
        "feature": "real_form",
        "signature": "(InnerClass,mat,ratvec) -> RealForm",
        "reason": "Rust overload is not implemented",
    },
)


FIXTURE_PLANS = (
    FixturePlan(name="eval/pipeline_swap_constructors"),
    # RootDatum/InnerClass/RealForm construction, full domain renderings,
    # KGBElt display, and the equality/inequality relations are all ported.
    FixturePlan(name="eval/pipeline_swap_domain_equality"),
    FixturePlan(name="eval/pipeline_swap_linear_values"),
    FixturePlan(name="eval/pipeline_swap_rejected"),
    FixturePlan(name="eval/pipeline_swap_void_reports"),
    # B3a non-recursive functions: typed lambdas, closure capture, return at
    # the call boundary, and identifier selectors.
    FixturePlan(name="eval/functions_b3"),
    FixturePlan(name="eval/functions_b3_rejected"),
    # B3b recursive functions and let-declaration definition sugar.
    FixturePlan(name="eval/functions_b3b"),
    FixturePlan(name="eval/functions_b3b_rejected"),
    # B3c parameter patterns: tuple destructuring, discard, and const patterns.
    FixturePlan(name="eval/patterns_b3c"),
    FixturePlan(name="eval/patterns_b3c_rejected"),
    # B3d selectors: unit selector and operator selectors.
    FixturePlan(name="eval/selectors_b3d"),
    FixturePlan(name="eval/selectors_b3d_rejected"),
    # B4 loops: while/for value collection, break, and loop rejections.
    FixturePlan(name="eval/loops_b4"),
    FixturePlan(name="eval/loops_b4_rejected"),
    # B5 set_type: user-defined types, union display, and case discrimination.
    FixturePlan(name="eval/settype_b5"),
    FixturePlan(name="eval/settype_b5_rejected"),
    # B6 case and counted for: integer case selection, union case, and
    # from/downto loops.
    FixturePlan(name="eval/casefor_b6"),
    FixturePlan(name="eval/casefor_b6_rejected"),
    # B7 misc commands: forget, die, and coercion after overload removal.
    FixturePlan(name="eval/commands_b7"),
    FixturePlan(name="eval/commands_b7_rejected"),
    # B8 user overloads: definition accumulation, redefinition, listing, and
    # wrong-arity rejection.
    FixturePlan(name="eval/overloads_b8"),
    FixturePlan(name="eval/overloads_b8b"),
    FixturePlan(name="eval/overloads_b8_rejected"),
    FixturePlan(name="eval/overloads_ops_b8c"),
    FixturePlan(name="eval/overloads_ops_b8c_rejected"),
    FixturePlan(name="eval/whattype_ops_b8d"),
    # B13 do-expression termination: `dont` is admitted only after a
    # semicolon in a while condition, not as a plain expression after `do`.
    FixturePlan(name="eval/dont_b13"),
    FixturePlan(name="eval/dont_b13_rejected"),
    # B9 file commands: tofile/addtofile redirection and its rejections.
    # The two redirect lines run but produce no stdout event.
    FixturePlan(
        name="eval/file_commands_b9",
        runnable_lines=(3,),
        runnable_events=(0,),
        silent_lines=(1, 2),
    ),
    FixturePlan(name="eval/file_commands_b9_rejected"),
    # B10 fromfile inclusion errors and quit semantics. The quit line and
    # the unreachable line after it run but produce no event.
    FixturePlan(name="eval/fromfile_b10"),
    # B10 accepted inclusion: line 3 is a silent skip (file already seen).
    FixturePlan(
        name="eval/fromfile_accepted_b10",
        runnable_lines=(1, 2, 4),
        runnable_events=(0, 1, 2),
        silent_lines=(3,),
    ),
    FixturePlan(
        name="eval/quit_b10",
        runnable_lines=(1,),
        runnable_events=(0,),
        silent_lines=(2, 3),
    ),
    # B11 precedence/associativity corpus and B12 runtime-error corpus.
    FixturePlan(name="eval/precedence_b11"),
    FixturePlan(name="eval/runtime_errors_b12"),
    # RootDatum root/coroot queries: oracle presentation order, negative-index
    # negation, long/short flags, rank, and the illegal-index rejection.
    FixturePlan(name="domain/root_coroot"),
    FixturePlan(name="domain/root_coroot_rejected"),
    # KGB headline observables: per-form KGB sizes and root statuses across
    # the A1/C2/A2 families, plus the inexistent-element and type rejections.
    FixturePlan(name="domain/kgb_generation"),
    FixturePlan(name="domain/kgb_generation_rejected"),
    # Real-form numbering, form names, and dual real-form construction for
    # A1/C2/A2, including the exact illegal external-number diagnostic.
    FixturePlan(name="domain/real_group"),
    FixturePlan(name="domain/real_group_rejected"),
    # KGB element operations: cross/Cayley/status/length, torus_factor,
    # equality, the `%` decompose, the distinguished twist, and the
    # illegal-generator rejection.
    FixturePlan(name="domain/kgb_operations"),
    FixturePlan(name="domain/kgb_operations_rejected"),
    # Tits twists: distinguished twist on KGB elements and the outer twist
    # by a matrix, including the unbased-involution rejection.
    FixturePlan(name="domain/tits_operations"),
    FixturePlan(name="domain/tits_operations_rejected"),
    # Grading slice: base_grading_vector/initial_torus_bits per real form
    # and torus_bits per KGB element, plus the RootDatum-argument rejection.
    FixturePlan(name="domain/grading"),
    FixturePlan(name="domain/grading_rejected"),
    # WeylElt surface: W_elt canonical words (A2/B2 braid anchors), word,
    # length, relations, product/inverse/generator-product, root_datum,
    # plus the illegal-entry and negative-entry rejections.
    FixturePlan(name="domain/weyl_element"),
    FixturePlan(name="domain/weyl_element_rejected"),
    # CartanClass surface: per-class occurrence counts and display,
    # involution, most-split, (dual) real-form sweeps, square classes,
    # fiber partition, per-form numbering, and the illegal-number rejection.
    FixturePlan(name="domain/cartan_aggregation"),
    FixturePlan(name="domain/cartan_aggregation_rejected"),
    # Synthetic KGB seed: KGB_elt(RealForm,mat,ratvec) symmetrizes the torus
    # factor, factors theta as a twisted involution, and looks the Tits
    # element up per form; rejections cover the cocharacter-coset and
    # non-involution diagnostics.
    FixturePlan(name="domain/seed_x0"),
    FixturePlan(name="domain/seed_x0_rejected"),
    # Involution-table printers: print_KGB's full table (statuses, crosses,
    # Cayleys, torus parts, canonical-involution words) and
    # print_strong_real's single-class layout on A1, plus the two-overload
    # match failure on a RootDatum argument.
    FixturePlan(name="domain/involution_table"),
    FixturePlan(name="domain/involution_table_rejected"),
    # Adjoint-fiber stabilizer: central_fiber(RealForm->[vec]) on split
    # SL(2,R), compact SU(2), and quasisplit SU(2,1), plus the InnerClass
    # argument conform rejection.
    FixturePlan(name="domain/adjoint_fiber"),
    FixturePlan(name="domain/adjoint_fiber_rejected"),
    # Real-form label matrices: occurrence/dual_occurrence, block_sizes and
    # block_size, and Cartan_order on the A2 compact inner class, plus the
    # real-form-number out-of-bounds rejection.
    FixturePlan(name="domain/real_form_labels"),
    FixturePlan(name="domain/real_form_labels_rejected"),
    # Early scalar-era fixtures: verified verbatim locally and included so
    # the HPC differential upgrades their reference metadata.
    FixturePlan(name="eval/scalars"),
    FixturePlan(name="eval/scalar_overloads"),
    FixturePlan(name="eval/scalar_error_fraction_zero"),
    FixturePlan(name="eval/scalar_error_int_power_large"),
    FixturePlan(name="eval/scalar_error_int_power_negative"),
    FixturePlan(name="eval/scalar_error_rat_divide_zero"),
    FixturePlan(name="eval/scalar_error_rat_modulo_zero"),
    FixturePlan(name="eval/scalar_error_rat_power_negative"),
    FixturePlan(name="eval/scalar_error_rat_quotient_zero"),
)


DIAGNOSTIC_HEADER = re.compile(
    r"^(Lexical|Syntax|Name|Type|Runtime|Io) error(?: at .*?:\d+:\d+)?: (.*)$"
)


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def command_output(command: list[str]) -> str:
    try:
        return subprocess.check_output(
            command, text=True, stderr=subprocess.DEVNULL
        ).strip()
    except (OSError, subprocess.CalledProcessError):
        return "unavailable"


def render_value(value: dict[str, Any]) -> str:
    if "display" in value:
        return str(value["display"])
    value_type = value["type"]
    if value_type == "integer":
        return str(value["value"])
    if value_type == "rational":
        return f"{value['numerator']}/{value['denominator']}"
    if value_type == "boolean":
        return "true" if value["value"] else "false"
    if value_type == "string":
        return f'"{value["value"]}"'
    if value_type == "tuple":
        return "(" + ",".join(render_value(item) for item in value["values"]) + ")"
    raise ValueError(f"cannot render expected value type {value_type!r}")


def expected_cli_observation(events: list[dict[str, Any]]) -> dict[str, Any]:
    stdout_parts: list[str] = []
    diagnostics: list[dict[str, str]] = []
    for event in events:
        kind = event.get("kind")
        if kind == "Value":
            stdout_parts.append(f"Value: {render_value(event['value'])}\n")
        elif kind in ("ReportLine", "Output"):
            stdout_parts.append(event["text"])
        elif kind == "Diagnostic":
            diagnostics.append(
                {
                    "category": event["category"].lower(),
                    "message": event["message"],
                }
            )
        else:
            raise ValueError(f"unsupported expected event kind {kind!r}")
    stdout_parts.append("Bye.\n")
    return {
        "stdout": "".join(stdout_parts),
        "diagnostics": diagnostics,
        "exit_status": 1 if diagnostics else 0,
    }


def parse_cli_diagnostics(stderr: str) -> tuple[list[dict[str, str]], list[str]]:
    diagnostics: list[dict[str, str]] = []
    unparsed: list[str] = []
    for line in stderr.splitlines():
        match = DIAGNOSTIC_HEADER.match(line)
        if match:
            diagnostics.append(
                {"category": match.group(1).lower(), "message": match.group(2)}
            )
        elif line.startswith("  | ") or not line.strip():
            continue
        else:
            unparsed.append(line)
    return diagnostics, unparsed


def selected_fixture_source(source: str, line_numbers: tuple[int, ...]) -> str:
    lines = source.splitlines()
    selected = [lines[line_number - 1] for line_number in line_numbers]
    return "\n".join(selected) + "\n"


def validate_plan(
    plan: FixturePlan,
    source: str,
    events: list[dict[str, Any]],
) -> tuple[tuple[int, ...], tuple[int, ...], list[str]]:
    errors: list[str] = []
    source_lines = source.splitlines()
    nonempty_lines = tuple(
        index for index, line in enumerate(source_lines, start=1) if line.strip()
    )
    runnable_lines = (
        nonempty_lines if plan.runnable_lines is None else plan.runnable_lines
    )
    runnable_events = (
        tuple(range(len(events)))
        if plan.runnable_events is None
        else plan.runnable_events
    )

    if len(runnable_lines) != len(runnable_events):
        errors.append("runnable source/event selection lengths differ")
    if any(line < 1 or line > len(source_lines) for line in runnable_lines):
        errors.append("runnable source line is outside the fixture")
    if any(index < 0 or index >= len(events) for index in runnable_events):
        errors.append("runnable event index is outside the expectation")
    if tuple(sorted(set(runnable_lines))) != runnable_lines:
        errors.append("runnable source lines are not unique and increasing")
    if tuple(sorted(set(runnable_events))) != runnable_events:
        errors.append("runnable event indices are not unique and increasing")

    pending_lines = tuple(case.source_line for case in plan.pending)
    pending_events = tuple(case.reference_event for case in plan.pending)
    if tuple(sorted(set(pending_lines))) != pending_lines:
        errors.append("pending source lines are not unique and increasing")
    if tuple(sorted(set(pending_events))) != pending_events:
        errors.append("pending event indices are not unique and increasing")
    silent_lines = plan.silent_lines
    if tuple(sorted(set(silent_lines))) != silent_lines:
        errors.append("silent source lines are not unique and increasing")
    if any(line < 1 or line > len(source_lines) for line in silent_lines):
        errors.append("silent source line is outside the fixture")
    if set(runnable_lines).intersection(pending_lines):
        errors.append("a source line is both runnable and pending")
    if set(runnable_lines).intersection(silent_lines):
        errors.append("a source line is both runnable and silent")
    if set(silent_lines).intersection(pending_lines):
        errors.append("a source line is both silent and pending")
    if set(runnable_events).intersection(pending_events):
        errors.append("an event is both runnable and pending")
    if tuple(sorted(runnable_lines + silent_lines + pending_lines)) != nonempty_lines:
        errors.append("source selection does not cover every nonempty fixture line")
    if tuple(sorted(runnable_events + pending_events)) != tuple(range(len(events))):
        errors.append("event selection does not cover every expected event")
    return runnable_lines, runnable_events, errors


def run_fixture(
    plan: FixturePlan,
    cli_bin: pathlib.Path,
    output_dir: pathlib.Path,
    fixture_root: pathlib.Path,
    reference_root: pathlib.Path,
    workspace_root: pathlib.Path,
    expected_revision: str,
    timeout: int,
) -> tuple[dict[str, Any], bool]:
    fixture = fixture_root / f"{plan.name}.atlas"
    expectation_path = reference_root / f"{plan.name}.events.json"
    metadata_path = reference_root / f"{plan.name}.meta.json"
    source_bytes = fixture.read_bytes()
    source = source_bytes.decode("utf-8")
    expectation = json.loads(expectation_path.read_text(encoding="utf-8"))
    metadata = json.loads(metadata_path.read_text(encoding="utf-8"))
    events = expectation.get("events", [])
    runnable_lines, runnable_events, configuration_errors = validate_plan(
        plan, source, events
    )

    fixture_sha = sha256(source_bytes)
    if metadata.get("fixture_sha256") != fixture_sha:
        configuration_errors.append("fixture checksum differs from oracle metadata")
    expected_fixture_name = plan.name
    # Older capture jobs recorded the fixture name with its ".atlas" suffix
    # (for example domain/root_coroot.atlas); the events file and the plan
    # both use the bare name, so normalize only that suffix away.
    metadata_fixture = str(metadata.get("fixture", "")).removesuffix(".atlas")
    if metadata_fixture != expected_fixture_name:
        configuration_errors.append("metadata names a different fixture")
    if expectation.get("fixture") != expected_fixture_name:
        configuration_errors.append("event expectation names a different fixture")
    if metadata.get("reference_status") != "verified_hpc_reference":
        configuration_errors.append("reference metadata is not HPC-verified")
    if expectation.get("status") != "verified_hpc_reference":
        configuration_errors.append("event expectation is not HPC-verified")
    if metadata.get("reference_atlas_revision") != expected_revision:
        configuration_errors.append("reference revision differs from requested revision")
    if metadata.get("oracle") != "atlas":
        configuration_errors.append("metadata does not name Atlas as the oracle")
    if metadata.get("stage") != "typed-pipeline-swap":
        configuration_errors.append("metadata belongs to a different stage")

    artifact_dir = output_dir / plan.name
    artifact_dir.mkdir(parents=True, exist_ok=True)
    selected_lines = tuple(sorted(runnable_lines + plan.silent_lines))
    selected_source = selected_fixture_source(source, selected_lines)
    selected_path = artifact_dir / "runnable.atlas"
    selected_path.write_text(selected_source, encoding="utf-8")
    expected_events = [events[index] for index in runnable_events]
    expected = expected_cli_observation(expected_events)
    if not plan.pending:
        if "oracle_exit_status" not in metadata:
            configuration_errors.append("oracle metadata has no exit status")
        else:
            expected["exit_status"] = metadata["oracle_exit_status"]

    timed_out = False
    started = time.monotonic()
    if configuration_errors:
        stdout = b""
        stderr = b""
        exit_status = None
    else:
        try:
            completed = subprocess.run(
                [str(cli_bin), str(selected_path.resolve())],
                cwd=workspace_root,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                timeout=timeout,
            )
            stdout = completed.stdout
            stderr = completed.stderr
            exit_status = completed.returncode
        except subprocess.TimeoutExpired as error:
            timed_out = True
            stdout = error.stdout or b""
            stderr = error.stderr or b""
            exit_status = None
    elapsed = round(time.monotonic() - started, 3)

    stdout_path = artifact_dir / "rust.stdout"
    stderr_path = artifact_dir / "rust.stderr"
    stdout_path.write_bytes(stdout)
    stderr_path.write_bytes(stderr)
    stdout_text = stdout.decode("utf-8", errors="replace")
    stderr_text = stderr.decode("utf-8", errors="replace")
    actual_diagnostics, unparsed_stderr = parse_cli_diagnostics(stderr_text)
    checks = {
        "configuration_valid": not configuration_errors,
        "completed_before_timeout": not timed_out,
        "stdout_exact": stdout_text == expected["stdout"],
        "diagnostics_exact": actual_diagnostics == expected["diagnostics"],
        "stderr_fully_parsed": not unparsed_stderr,
        "exit_status_exact": exit_status == expected["exit_status"],
    }
    runnable_passed = all(checks.values())
    fixture_status = (
        "FAIL" if not runnable_passed else "PARTIAL" if plan.pending else "PASS"
    )

    def relative(path: pathlib.Path) -> str:
        try:
            return path.resolve().relative_to(workspace_root).as_posix()
        except ValueError:
            return str(path.resolve())

    def artifact_relative(path: pathlib.Path) -> str:
        return path.resolve().relative_to(output_dir.resolve()).as_posix()

    entry = {
        "fixture": relative(fixture),
        "fixture_sha256": fixture_sha,
        "expectation": {
            "path": relative(expectation_path),
            "sha256": sha256(expectation_path.read_bytes()),
            "event_indices": list(runnable_events),
            "stdout": {
                "sha256": sha256(expected["stdout"].encode()),
                "text": expected["stdout"],
            },
            "diagnostics": expected["diagnostics"],
            "exit_status": expected["exit_status"],
        },
        "metadata": {
            "path": relative(metadata_path),
            "sha256": sha256(metadata_path.read_bytes()),
            "reference_job": metadata.get("reference_job"),
            "reference_atlas_revision": metadata.get("reference_atlas_revision"),
            "reference_binary_sha256": metadata.get("reference_binary_sha256"),
        },
        "runnable": {
            "source_lines": list(runnable_lines),
            "silent_source_lines": list(plan.silent_lines),
            "input_path": artifact_relative(selected_path),
            "input_sha256": sha256(selected_source.encode()),
        },
        "pending": [
            {
                "feature": case.feature,
                "source_line": case.source_line,
                "reference_event": case.reference_event,
                "reason": case.reason,
            }
            for case in plan.pending
        ],
        "rust": {
            "stdout": {
                "path": artifact_relative(stdout_path),
                "sha256": sha256(stdout),
                "text": stdout_text,
            },
            "stderr": {
                "path": artifact_relative(stderr_path),
                "sha256": sha256(stderr),
                "text": stderr_text,
            },
            "diagnostics": actual_diagnostics,
            "unparsed_stderr": unparsed_stderr,
            "exit_status": exit_status,
            "timed_out": timed_out,
            "seconds": elapsed,
        },
        "configuration_errors": configuration_errors,
        "checks": checks,
        "runnable_status": "PASS" if runnable_passed else "FAIL",
        "status": fixture_status,
    }
    return entry, runnable_passed


def parse_dirty_tree(value: str) -> bool | str:
    return {"true": True, "false": False}.get(value.lower(), value)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("atlas_cli", type=pathlib.Path)
    parser.add_argument("output_dir", type=pathlib.Path)
    parser.add_argument("--workspace-root", type=pathlib.Path, required=True)
    parser.add_argument("--fixture-root", type=pathlib.Path, required=True)
    parser.add_argument("--reference-root", type=pathlib.Path, required=True)
    parser.add_argument("--commit", required=True)
    parser.add_argument("--dirty-tree", required=True)
    parser.add_argument("--detected-commit", required=True)
    parser.add_argument("--detected-dirty-tree", required=True)
    parser.add_argument("--job-id", required=True)
    parser.add_argument("--source-snapshot-sha256", required=True)
    parser.add_argument("--timeout", type=int, default=30)
    parser.add_argument(
        "--expected-reference-revision", default=PINNED_ATLAS_REVISION
    )
    args = parser.parse_args()

    cli_bin = args.atlas_cli.resolve()
    if not os.access(cli_bin, os.X_OK):
        parser.error(f"atlas-cli is not executable: {cli_bin}")
    workspace_root = args.workspace_root.resolve()
    fixture_root = args.fixture_root.resolve()
    reference_root = args.reference_root.resolve()
    output_dir = args.output_dir.resolve()
    output_dir.mkdir(parents=True, exist_ok=True)

    source_state_checks = {
        "declared_commit_valid": bool(COMMIT_TOKEN.fullmatch(args.commit)),
        "detected_commit_valid": bool(COMMIT_TOKEN.fullmatch(args.detected_commit)),
        "commit_exact": args.commit == args.detected_commit,
        "declared_dirty_tree_valid": args.dirty_tree in DIRTY_TREE_TOKENS,
        "detected_dirty_tree_valid": (
            args.detected_dirty_tree in DIRTY_TREE_TOKENS
        ),
        "dirty_tree_exact": args.dirty_tree == args.detected_dirty_tree,
    }
    source_state_verified = all(source_state_checks.values())

    entries = []
    all_runnable_passed = True
    for plan in FIXTURE_PLANS:
        entry, passed = run_fixture(
            plan,
            cli_bin,
            output_dir,
            fixture_root,
            reference_root,
            workspace_root,
            args.expected_reference_revision,
            args.timeout,
        )
        entries.append(entry)
        all_runnable_passed = all_runnable_passed and passed

    pending = [
        {
            "fixture": entry["fixture"],
            **case,
        }
        for entry in entries
        for case in entry["pending"]
    ]
    pending.extend(
        {"scope": "uncovered_overload", **overload}
        for overload in PENDING_OVERLOADS
    )
    status = (
        "FAIL"
        if not source_state_verified or not all_runnable_passed
        else "PARTIAL"
        if pending
        else "PASS"
    )
    report = {
        "schema": "atlas-pipeline-swap-diff-v1",
        "stage": "typed-pipeline-swap-rust-vs-frozen-atlas",
        "status": status,
        "runnable_status": "PASS" if all_runnable_passed else "FAIL",
        "compatibility_claim": status == "PASS",
        "pending_features": pending,
        "commit": args.commit,
        "dirty_tree": parse_dirty_tree(args.dirty_tree),
        "source_state": {
            "declared_commit": args.commit,
            "detected_commit": args.detected_commit,
            "declared_dirty_tree": parse_dirty_tree(args.dirty_tree),
            "detected_dirty_tree": parse_dirty_tree(args.detected_dirty_tree),
            "verified": source_state_verified,
            "checks": source_state_checks,
        },
        "source_snapshot_sha256": args.source_snapshot_sha256,
        "source_snapshot_scope": (
            "provided snapshot (exact scope annotated by the batch job)"
        ),
        "harness_sha256": sha256(pathlib.Path(__file__).read_bytes()),
        "atlas_cli": str(cli_bin),
        "atlas_cli_sha256": sha256(cli_bin.read_bytes()),
        "reference_atlas_revision": args.expected_reference_revision,
        "diagnostic_comparison": {
            "scope": "category and message only",
            "source_path_line_column_caret_compared": False,
            "position_context_lines_ignored": "lines beginning with '  | '",
            "other_stderr_is_failure": True,
        },
        "rustc": command_output(["rustc", "--version"]),
        "cargo": command_output(["cargo", "--version"]),
        "slurm": {
            "job_id": args.job_id,
            "node_list": os.environ.get("SLURM_JOB_NODELIST", "unavailable"),
            "hostname": command_output(["hostname"]),
        },
        "fixtures": entries,
    }
    report_path = output_dir / "pipeline_swap_diff_report.json"
    report_path.write_text(
        json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    print(
        f"pipeline swap: {len(entries)} fixtures, "
        f"{len(pending)} pending cases, {status}"
    )
    print(f"report: {report_path}")
    return 0 if source_state_verified and all_runnable_passed else 1


if __name__ == "__main__":
    raise SystemExit(main())

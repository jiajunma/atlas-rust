"""Pure, non-accepting classifier for the core Weyl-context discovery.

This module deliberately performs no I/O and imports no project helpers.  A
capture runner owns fresh subprocess creation and raw-artifact persistence; a
later, independent reviewer owns any golden or mathematical acceptance.
"""

import hashlib
import json
import math
import re


CATALOG_SCHEMA = "atlas-weyl-context-core-discovery-v1"
CATALOG_MATURITY = "source_predicted_not_captured"
CAPTURE_MATURITY = "capture_only_unreviewed"
CATALOG_SCOPE = (
    "Fresh-process core-only rank-one discovery of Weyl owner compatibility, "
    "cold-dual sharing, prewarmed-dual separation, invalid-word ordering and "
    "recovery. Intent describes the original oracle only; no Rust compatibility, "
    "mathematical acceptance or cache acceptance is claimed."
)

EXPECTED_CASES = (
    {
        "id": "weyl_context_core_cold_dual",
        "file": "weyl_context_core_cold_dual.atlas",
        "fixture_sha256": "4eff8fa08f8490e242f07282125b83cde1daf24a875fe8df4dc7e73e75a0ae99",
        "intent": "accept",
        "timeout_seconds": 45,
        "scope": (
            "Same owner, alias, independently interned equal datum, both "
            "warm-source/cold-target dual directions, rebound owner and "
            "saved-value lifetime."
        ),
    },
    {
        "id": "weyl_context_core_prewarmed_dual",
        "file": "weyl_context_core_prewarmed_dual.atlas",
        "fixture_sha256": "f13d704175f702b966790bf82e56c837dd80a594f0dc2dcd0d6b81ba281799a2",
        "intent": "reject",
        "timeout_seconds": 45,
        "scope": (
            "Cold source with independently prewarmed canonical dual, plus a "
            "preference-distinct owner, incompatible equality/product, invalid "
            "words and recovery after every rejection."
        ),
    },
)

NONACCEPTING_STATUSES = frozenset(
    {
        "CAPTURE_INCOMPLETE_FRESH_PROCESS",
        "CAPTURE_INCOMPLETE_RESOURCE_OR_TIMEOUT",
        "CAPTURE_INCOMPLETE_METRICS",
        "CAPTURE_INCOMPLETE_STREAM",
        "ORIGINAL_SOURCE_PREDICTION_DIFFERED",
        "RUST_SOURCE_PREDICTION_DIFFERED",
        "BOTH_SOURCE_PREDICTIONS_DIFFERED",
        "SOURCE_PREDICTIONS_OBSERVED_UNREVIEWED",
    }
)

_COLD_ORACLE_MARKERS = [
    "WC_SAME|[0]|1|[1,0]|[]",
    "WC_ALIAS|true|true|true|[]",
    "WC_EQUAL|true|true|true|[]",
    "WC_DUAL_COLD_OWNER|true",
    "WC_DUAL_COLD_EQ|true",
    "WC_DUAL_COLD_NEQ|false",
    "WC_DUAL_COLD_MUL|[]",
    "WC_DUAL_REVERSE_OWNER|true",
    "WC_DUAL_REVERSE_EQ|true",
    "WC_DUAL_REVERSE_NEQ|false",
    "WC_DUAL_REVERSE_MUL|[]|true",
    "WC_REBOUND|false|[1,0]|true|false|[0]|[1,0]",
    "WC_RECOVERY|727",
]
_COLD_RUST_MARKERS = [
    "WC_SAME|[0]|1|[1,0]|[]",
    "WC_ALIAS|true|true|true|[]",
    "WC_EQUAL|true|true|true|[]",
    "WC_DUAL_COLD_OWNER|true",
    "WC_DUAL_COLD_EQ|false",
    "WC_DUAL_COLD_NEQ|true",
    "WC_DUAL_REVERSE_OWNER|true",
    "WC_DUAL_REVERSE_EQ|false",
    "WC_DUAL_REVERSE_NEQ|true",
    "WC_REBOUND|false|[1,0]|true|false|[0]|[1,0]",
    "WC_RECOVERY|727",
]
_PREWARM_ORACLE_MARKERS = [
    "WCN_AFTER_OWNER_EQ|[0]|[1,0]",
    "WCN_AFTER_OWNER_NEQ|[0]|[1,0]",
    "WCN_AFTER_OWNER_MUL|[0]|[1,0]",
    "WCN_AFTER_HIGH|[0]",
    "WCN_AFTER_NEGATIVE|[0]",
    "WCN_AFTER_DUAL_EQ|[0]|true",
    "WCN_AFTER_DUAL_NEQ|[0]|true",
    "WCN_AFTER_DUAL_MUL|[0]|[0]|true",
    "WCN_RECOVERY|733",
]
_PREWARM_RUST_MARKERS = [
    "WCN_OWNER_EQ|false",
    "WCN_AFTER_OWNER_EQ|[0]|[1,0]",
    "WCN_OWNER_NEQ|true",
    "WCN_AFTER_OWNER_NEQ|[0]|[1,0]",
    "WCN_AFTER_OWNER_MUL|[0]|[1,0]",
    "WCN_AFTER_HIGH|[0]",
    "WCN_AFTER_NEGATIVE|[0]",
    "WCN_DUAL_PREWARM_EQ|false",
    "WCN_AFTER_DUAL_EQ|[0]|true",
    "WCN_DUAL_PREWARM_NEQ|true",
    "WCN_AFTER_DUAL_NEQ|[0]|true",
    "WCN_AFTER_DUAL_MUL|[0]|[0]|true",
    "WCN_RECOVERY|733",
]

_COLD_DECLARATIONS = [
    "Variable wc_rd: RootDatum",
    "Variable wc_saved: WeylElt",
    "Variable wc_alias: RootDatum",
    "Variable wc_alias_w: WeylElt",
    "Variable wc_equal: RootDatum",
    "Variable wc_equal_w: WeylElt",
    "Variable wc_dual: RootDatum",
    "Variable wc_dual_w: WeylElt",
    "Variable wc_reverse_target: RootDatum",
    "Variable wc_reverse_target_w: WeylElt",
    "Variable wc_reverse_source: RootDatum",
    "Variable wc_reverse_source_w: WeylElt",
    (
        "Variable wc_rd: RootDatum (overriding previous instance, which had "
        "type RootDatum)"
    ),
    "Variable wc_rebound_w: WeylElt",
]
_PREWARM_DECLARATIONS = [
    "Variable wcn_true: RootDatum",
    "Variable wcn_false: RootDatum",
    "Variable wcn_false_w: WeylElt",
    "Variable wcn_target: RootDatum",
    "Variable wcn_target_w: WeylElt",
    "Variable wcn_dual: RootDatum",
    "Variable wcn_saved: WeylElt",
    "Variable wcn_dual_w: WeylElt",
]


def _cold_payload(marker_lines, include_products):
    declarations = iter(_COLD_DECLARATIONS)
    markers = iter(marker_lines)
    payload = [next(declarations), next(declarations), next(markers)]
    payload.extend([next(declarations), next(declarations), next(markers)])
    payload.extend([next(declarations), next(declarations), next(markers)])
    payload.extend([next(declarations), next(declarations)])
    payload.extend([next(markers), next(markers), next(markers)])
    if include_products:
        payload.append(next(markers))
    payload.extend(
        [
            next(declarations),
            next(declarations),
            next(declarations),
            next(declarations),
            next(markers),
            next(markers),
            next(markers),
        ]
    )
    if include_products:
        payload.append(next(markers))
    payload.extend([next(declarations), next(declarations), next(markers), next(markers)])
    return payload


def _prewarm_payload(marker_lines):
    return [*_PREWARM_DECLARATIONS, *marker_lines]

MISMATCH = "weyl_group_mismatch"
HIGH_WORD = "illegal_weyl_word_entry"
NEGATIVE_WORD = "negative_integer_where_unsigned_required"

PREDICTIONS = {
    "weyl_context_core_cold_dual": {
        "oracle": {
            "exit_status": 0,
            "marker_lines": _COLD_ORACLE_MARKERS,
            "payload_lines": _cold_payload(_COLD_ORACLE_MARKERS, True),
            "causes": [],
            "diagnostic_categories": [],
            "stderr_empty": True,
        },
        "rust": {
            "exit_status": 1,
            "marker_lines": _COLD_RUST_MARKERS,
            "payload_lines": _cold_payload(_COLD_RUST_MARKERS, False),
            "causes": [MISMATCH, MISMATCH],
            "diagnostic_categories": ["runtime", "runtime"],
            "stderr_empty": False,
        },
    },
    "weyl_context_core_prewarmed_dual": {
        "oracle": {
            "exit_status": 1,
            "marker_lines": _PREWARM_ORACLE_MARKERS,
            "payload_lines": _prewarm_payload(_PREWARM_ORACLE_MARKERS),
            "causes": [MISMATCH, MISMATCH, MISMATCH, HIGH_WORD, NEGATIVE_WORD,
                       MISMATCH, MISMATCH, MISMATCH],
            "diagnostic_categories": ["runtime"] * 8,
            "stderr_empty": False,
        },
        "rust": {
            "exit_status": 1,
            "marker_lines": _PREWARM_RUST_MARKERS,
            "payload_lines": _prewarm_payload(_PREWARM_RUST_MARKERS),
            "causes": [MISMATCH, HIGH_WORD, NEGATIVE_WORD, MISMATCH],
            "diagnostic_categories": ["runtime"] * 4,
            "stderr_empty": False,
        },
    },
}

G2_CATALOG_SCHEMA = "atlas-weyl-context-g2-discovery-v1"
G2_CATALOG_SCOPE = (
    "Fresh-process core-only G2 discovery of Weyl owner compatibility across "
    "the two asymmetric numberings, cold-dual sharing in both directions, "
    "prewarmed-dual separation, noncommuting words, the order-6 braid "
    "relation, invalid-word ordering and recovery. Intent describes the "
    "original oracle only; no Rust compatibility, mathematical acceptance "
    "or cache acceptance is claimed."
)

G2_EXPECTED_CASES = (
    {
        "id": "weyl_context_g2_cold_dual",
        "file": "weyl_context_g2_cold_dual.atlas",
        "fixture_sha256": (
            "70c3e09e3678d149f46d844d2a47c3024050452a1383116784e577d953fa8c50"
        ),
        "intent": "accept",
        "timeout_seconds": 45,
        "scope": (
            "G2 same owner, noncommuting words, order-6 braid relation, "
            "alias, independently interned equal datum, both cold dual "
            "directions with the generator-order witness, rebound owner and "
            "saved-value lifetime."
        ),
    },
    {
        "id": "weyl_context_g2_prewarmed_dual",
        "file": "weyl_context_g2_prewarmed_dual.atlas",
        "fixture_sha256": (
            "06d3816d4636f2d25d46f96f1efa5417c598c4b6511d42f37d2fec61c8e05921"
        ),
        "intent": "reject",
        "timeout_seconds": 45,
        "scope": (
            "G2 numbering-incompatible owners, independently prewarmed "
            "canonical dual, incompatible equality/inequality/product, high "
            "and negative words and recovery after every rejection."
        ),
    },
)

_G2_COLD_DECLARATIONS = [
    "Variable wg_rd: RootDatum",
    "Variable wg_s0: WeylElt",
    "Variable wg_s1: WeylElt",
    "Variable wg_alias: RootDatum",
    "Variable wg_alias_w: WeylElt",
    "Variable wg_equal: RootDatum",
    "Variable wg_equal_w: WeylElt",
    "Variable wg_dual: RootDatum",
    "Variable wg_dual_w: WeylElt",
    "Variable wg_reverse_target: RootDatum",
    "Variable wg_reverse_target_w: WeylElt",
    "Variable wg_reverse_source: RootDatum",
    "Variable wg_reverse_source_w: WeylElt",
    (
        "Variable wg_rd: RootDatum (overriding previous instance, which had "
        "type RootDatum)"
    ),
    "Variable wg_rebound_w: WeylElt",
]
_G2_COLD_MARKERS = [
    "WG_SAME|[0]|1|1|[0,1]",
    "WG_NONCOMMUTE|[0,1]|[1,0]",
    "WG_BRAID|true|6",
    "WG_ALIAS|true|true",
    "WG_EQUAL|true|true",
    "WG_DUAL_OWNER|true",
    "WG_DUAL_EQ|true",
    "WG_DUAL_NEQ|false",
    "WG_DUAL_MUL|[]",
    "WG_REVERSE_OWNER|true",
    "WG_REVERSE_EQ|true",
    "WG_REVERSE_NEQ|false",
    "WG_REVERSE_MUL|[]|true",
    "WG_REBOUND|false|true|false|[0]|[1]",
    "WG_RECOVERY|727",
]


def _g2_cold_payload():
    declarations = iter(_G2_COLD_DECLARATIONS)
    markers = iter(_G2_COLD_MARKERS)
    payload = [next(declarations) for _ in range(3)]
    payload.extend([next(markers), next(markers), next(markers)])
    payload.extend([next(declarations), next(declarations), next(markers)])
    payload.extend([next(declarations), next(declarations), next(markers)])
    payload.extend([next(declarations), next(declarations)])
    payload.extend([next(markers) for _ in range(4)])
    payload.extend([next(declarations) for _ in range(4)])
    payload.extend([next(markers) for _ in range(4)])
    payload.extend([next(declarations), next(declarations)])
    payload.extend([next(markers), next(markers)])
    return payload


_G2_PREWARM_DECLARATIONS = [
    "Variable wgn_true: RootDatum",
    "Variable wgn_false: RootDatum",
    "Variable wgn_false_w: WeylElt",
    "Variable wgn_target: RootDatum",
    "Variable wgn_target_w: WeylElt",
    "Variable wgn_dual: RootDatum",
    "Variable wgn_saved: WeylElt",
    "Variable wgn_dual_w: WeylElt",
]
_G2_PREWARM_MARKERS = [
    "WGN_AFTER_OWNER_EQ|[0]|1",
    "WGN_AFTER_OWNER_NEQ|[0]|1",
    "WGN_AFTER_OWNER_MUL|[0]|1",
    "WGN_AFTER_HIGH|[0]",
    "WGN_AFTER_NEGATIVE|[0]",
    "WGN_AFTER_DUAL_EQ|[1]|true",
    "WGN_AFTER_DUAL_NEQ|[1]|true",
    "WGN_AFTER_DUAL_MUL|[0]|[1]|true",
    "WGN_RECOVERY|733",
]


def _g2_prewarm_payload():
    return [*_G2_PREWARM_DECLARATIONS, *_G2_PREWARM_MARKERS]


HIGH_WORD_G2 = "illegal_weyl_word_entry_2"
_G2_PREWARM_CAUSES = [
    MISMATCH, MISMATCH, MISMATCH, HIGH_WORD_G2, NEGATIVE_WORD,
    MISMATCH, MISMATCH, MISMATCH,
]


def _g2_cold_prediction():
    return {
        "exit_status": 0,
        "marker_lines": _G2_COLD_MARKERS,
        "payload_lines": _g2_cold_payload(),
        "causes": [],
        "diagnostic_categories": [],
        "stderr_empty": True,
    }


def _g2_prewarm_prediction():
    return {
        "exit_status": 1,
        "marker_lines": _G2_PREWARM_MARKERS,
        "payload_lines": _g2_prewarm_payload(),
        "causes": _G2_PREWARM_CAUSES,
        "diagnostic_categories": ["runtime"] * 8,
        "stderr_empty": False,
    }


G2_PREDICTIONS = {
    "weyl_context_g2_cold_dual": {
        "oracle": _g2_cold_prediction(),
        "rust": _g2_cold_prediction(),
    },
    "weyl_context_g2_prewarmed_dual": {
        "oracle": _g2_prewarm_prediction(),
        "rust": _g2_prewarm_prediction(),
    },
}


def _g2_expected_catalog():
    return {
        "schema": G2_CATALOG_SCHEMA,
        "evidence_maturity": CATALOG_MATURITY,
        "scope": G2_CATALOG_SCOPE,
        "cases": [dict(case) for case in G2_EXPECTED_CASES],
    }


def decode_g2_catalog(raw):
    """Decode strict JSON bytes/text and require the frozen G2 schema."""
    if type(raw) not in (bytes, str):
        raise ValueError("catalog JSON must be bytes or text")
    try:
        catalog = json.loads(
            raw,
            object_pairs_hook=_unique_json_object,
            parse_constant=_reject_nonfinite_json,
        )
    except (UnicodeDecodeError, json.JSONDecodeError, ValueError) as error:
        raise ValueError("invalid Weyl-context G2 catalog JSON") from error
    validate_g2_catalog(catalog)
    return catalog


def validate_g2_catalog(catalog):
    """Require the exact G2 source-prediction catalog; never read fixtures."""
    if not _same_exact(_g2_expected_catalog(), catalog):
        raise ValueError("Weyl-context G2 catalog changed")


_CASE_PREFIXES = {
    "weyl_context_core_cold_dual": b"WC_",
    "weyl_context_core_prewarmed_dual": b"WCN_",
    "weyl_context_g2_cold_dual": b"WG_",
    "weyl_context_g2_prewarmed_dual": b"WGN_",
}
_MARKER_PREFIXES = tuple(_CASE_PREFIXES.values())

_ALL_PREDICTIONS = {**PREDICTIONS, **G2_PREDICTIONS}

_RUN_KEYS = frozenset(
    {"engine", "observation", "stdout", "stderr", "fresh_process", "invocation_id"}
)
_OBSERVATION_KEYS = frozenset(
    {
        "engine",
        "exit_status",
        "timed_out",
        "termination_uncertain",
        "signal",
        "seconds",
        "user_cpu_seconds",
        "system_cpu_seconds",
        "maxrss_kb",
        "maxrss_approximate",
    }
)
_ENGINES = ("oracle", "rust")
_FRAME_LINE = re.compile(rb"(?m)^MATH_(?:BEGIN|END) [^\r\n]*\n")
_ORACLE_DIAGNOSTIC_HEADER = re.compile(
    rb"^([A-Za-z][A-Za-z ]*) error:$"
)
_RUST_DIAGNOSTIC_HEADER = re.compile(
    rb"^([A-Za-z][A-Za-z ]*) error at <stdin>:[0-9]+:[0-9]+: (.*)$"
)
_RUST_CARET_LINE = re.compile(rb"^  \| +\^+ *$")
_CAUSE_MESSAGES = {
    MISMATCH: b"Weyl group mismatch",
    HIGH_WORD: b"Illegal Weyl word entry 1 (should be <1)",
    NEGATIVE_WORD: b"Negative integer where unsigned is required",
    HIGH_WORD_G2: b"Illegal Weyl word entry 2 (should be <2)",
}


def _same_exact(expected, actual):
    if type(expected) is not type(actual):
        return False
    if isinstance(expected, dict):
        return (set(expected) == set(actual)
                and all(_same_exact(expected[key], actual[key]) for key in expected))
    if isinstance(expected, list):
        return (len(expected) == len(actual)
                and all(_same_exact(left, right)
                        for left, right in zip(expected, actual)))
    return expected == actual


def _expected_catalog():
    return {
        "schema": CATALOG_SCHEMA,
        "evidence_maturity": CATALOG_MATURITY,
        "scope": CATALOG_SCOPE,
        "cases": [dict(case) for case in EXPECTED_CASES],
    }


def _unique_json_object(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError(f"duplicate JSON key: {key}")
        result[key] = value
    return result


def _reject_nonfinite_json(token):
    raise ValueError(f"non-finite JSON number: {token}")


def decode_catalog(raw):
    """Decode strict JSON bytes/text and require the frozen catalog schema."""
    if type(raw) not in (bytes, str):
        raise ValueError("catalog JSON must be bytes or text")
    try:
        catalog = json.loads(
            raw,
            object_pairs_hook=_unique_json_object,
            parse_constant=_reject_nonfinite_json,
        )
    except (UnicodeDecodeError, json.JSONDecodeError, ValueError) as error:
        raise ValueError("invalid Weyl-context core catalog JSON") from error
    validate_catalog(catalog)
    return catalog


def validate_catalog(catalog):
    """Require the exact source-prediction catalog; never read fixture files."""
    if not _same_exact(_expected_catalog(), catalog):
        raise ValueError("Weyl-context core catalog changed")


def _validated_case(case):
    for expected in (*EXPECTED_CASES, *G2_EXPECTED_CASES):
        if _same_exact(expected, case):
            return expected
    raise ValueError("unknown or changed Weyl-context case")


def _sha(raw):
    return hashlib.sha256(raw).hexdigest()


def _line_record(line, line_number):
    return {
        "line_number": line_number,
        "line": line.decode("utf-8", errors="replace"),
        "line_sha256": _sha(line),
        "line_bytes": len(line),
        "line_hex": line.hex(),
    }


def _region_record(raw):
    return {
        "text": raw.decode("utf-8", errors="replace"),
        "sha256": _sha(raw),
        "bytes": len(raw),
        "hex": raw.hex(),
        "lines": [
            _line_record(line, number)
            for number, line in enumerate(raw.splitlines(), start=1)
        ],
    }


def _incomplete_frame(reason, stdout):
    return {
        "complete": False,
        "reason": reason,
        "markers": [],
        "unclassified_markers": [],
        "payload_lines": [],
        "payload_sha256": None,
        "payload_bytes": None,
        "prefix": _region_record(b""),
        "suffix": _region_record(b""),
        "stdout_sha256": _sha(stdout),
        "stdout_bytes": len(stdout),
    }


def _parse_frame(stdout, case_id, prefix):
    begin = ("MATH_BEGIN " + case_id + "\n").encode()
    end = ("MATH_END " + case_id + "\n").encode()
    try:
        stdout.decode("utf-8", errors="strict")
    except UnicodeDecodeError:
        return _incomplete_frame("non_utf8_stdout", stdout)
    frame_lines = _FRAME_LINE.findall(stdout)
    if (frame_lines.count(begin) != 1 or frame_lines.count(end) != 1
            or len(frame_lines) != 2):
        return _incomplete_frame(
            "missing_duplicate_or_foreign_delimiter", stdout)
    begin_at = stdout.find(begin)
    finish = stdout.find(end)
    start = begin_at + len(begin)
    if begin_at < 0 or finish < start:
        return _incomplete_frame("reversed_delimiters", stdout)

    payload = stdout[start:finish]
    payload_lines = [line.decode("utf-8", errors="strict")
                     for line in payload.splitlines()]
    markers = []
    unclassified_markers = []
    marker_pattern = re.compile(rb"^" + re.escape(prefix) + rb"[A-Z0-9_]+\|[^\r\n]*$")
    for line_number, line in enumerate(payload.splitlines(), start=1):
        if not line.startswith(_MARKER_PREFIXES):
            continue
        if marker_pattern.fullmatch(line) is None:
            unclassified_markers.append(_line_record(line, line_number))
            continue
        name, value = line.split(b"|", 1)
        name_text = name.decode("ascii", errors="strict")
        line_text = line.decode("utf-8", errors="strict")
        value_text = value.decode("utf-8", errors="strict")
        markers.append(
            {
                "name": name_text,
                "payload": value_text,
                "line": line_text,
                "line_sha256": _sha(line),
                "line_bytes": len(line),
                "line_hex": line.hex(),
            }
        )
    return {
        "complete": True,
        "reason": "complete",
        "markers": markers,
        "unclassified_markers": unclassified_markers,
        "payload_lines": payload_lines,
        "payload_sha256": _sha(payload),
        "payload_bytes": len(payload),
        "prefix": _region_record(stdout[:begin_at]),
        "suffix": _region_record(stdout[finish + len(end):]),
        "stdout_sha256": _sha(stdout),
        "stdout_bytes": len(stdout),
    }


def _cause_for_message(message):
    matches = [token for token, expected in _CAUSE_MESSAGES.items()
               if message == expected]
    return matches[0] if len(matches) == 1 else None


def _parse_diagnostics(stderr, engine):
    headers = []
    causes = []
    blocks = []
    unclassified = []
    if stderr and not stderr.endswith(b"\n"):
        return {
            "complete": False,
            "reason": "missing_diagnostic_final_newline",
            "line_count": len(stderr.splitlines()),
            "headers": headers,
            "causes": causes,
            "blocks": blocks,
            "unclassified": unclassified,
        }
    try:
        stderr.decode("utf-8", errors="strict")
    except UnicodeDecodeError:
        return {
            "complete": False,
            "reason": "non_utf8_diagnostic",
            "line_count": len(stderr.splitlines()),
            "headers": headers,
            "causes": causes,
            "blocks": blocks,
            "unclassified": unclassified,
        }
    lines = stderr.splitlines()
    position = 0
    while position < len(lines):
        line_number = position + 1
        if engine == "oracle":
            header_match = _ORACLE_DIAGNOSTIC_HEADER.fullmatch(lines[position])
            if header_match is None:
                unclassified.append(_line_record(lines[position], line_number))
                position += 1
                continue
            if (position + 2 >= len(lines)
                    or lines[position + 2] != b"Evaluation aborted."):
                return {
                    "complete": False,
                    "reason": "incomplete_oracle_diagnostic",
                    "line_count": len(lines),
                    "headers": headers,
                    "causes": causes,
                    "blocks": blocks,
                    "unclassified": unclassified,
                }
            message = (
                lines[position + 1][2:]
                if lines[position + 1].startswith(b"  ")
                else lines[position + 1]
            )
            cause = _cause_for_message(message)
            category = header_match.group(1).decode("ascii").lower().replace(" ", "_")
            header = {
                "category": category,
                **_line_record(lines[position], line_number),
            }
            cause_record = {
                "cause": cause,
                "message": message.decode("utf-8", errors="strict"),
                **_line_record(lines[position + 1], line_number + 1),
            }
            headers.append(header)
            causes.append(cause_record)
            blocks.append(
                {
                    "category": category,
                    "cause": cause,
                    "line_numbers": [line_number, line_number + 1, line_number + 2],
                    "lines_hex": [line.hex() for line in lines[position:position + 3]],
                }
            )
            position += 3
            continue

        header_match = _RUST_DIAGNOSTIC_HEADER.fullmatch(lines[position])
        if header_match is None:
            unclassified.append(_line_record(lines[position], line_number))
            position += 1
            continue
        if (position + 2 >= len(lines)
                or not lines[position + 1].startswith(b"  | ")
                or _RUST_CARET_LINE.fullmatch(lines[position + 2]) is None):
            return {
                "complete": False,
                "reason": "incomplete_rust_diagnostic",
                "line_count": len(lines),
                "headers": headers,
                "causes": causes,
                "blocks": blocks,
                "unclassified": unclassified,
            }
        category = header_match.group(1).decode("ascii").lower().replace(" ", "_")
        message = header_match.group(2)
        cause = _cause_for_message(message)
        header = {
            "category": category,
            **_line_record(lines[position], line_number),
        }
        cause_record = {
            "cause": cause,
            "message": message.decode("utf-8", errors="strict"),
            **_line_record(lines[position], line_number),
        }
        headers.append(header)
        causes.append(cause_record)
        blocks.append(
            {
                "category": category,
                "cause": cause,
                "line_numbers": [line_number, line_number + 1, line_number + 2],
                "lines_hex": [line.hex() for line in lines[position:position + 3]],
            }
        )
        position += 3
    return {
        "complete": True,
        "reason": "complete",
        "line_count": len(lines),
        "headers": headers,
        "causes": causes,
        "blocks": blocks,
        "unclassified": unclassified,
    }


def _resource_healthy(observation, engine):
    code = observation.get("exit_status")
    signal = observation.get("signal")
    return (
        type(code) is int
        and code in (0, 1)
        and observation.get("timed_out") is False
        and observation.get("termination_uncertain") is False
        and (signal is None or (type(signal) is int and signal == 0))
        and observation["engine"] == engine
    )


def _exact_metrics(observation):
    for name in ("seconds", "user_cpu_seconds", "system_cpu_seconds"):
        value = observation.get(name)
        if type(value) is int:
            if value < 0:
                return False
        elif type(value) is float:
            if not math.isfinite(value) or value < 0:
                return False
        else:
            return False
    rss = observation.get("maxrss_kb")
    return (type(rss) is int and rss >= 0
            and observation.get("maxrss_approximate") is False)


def _json_metric(value):
    if type(value) is int:
        return value
    if type(value) is float and math.isfinite(value):
        return value
    return None


def _validate_observation_schema(observation, engine):
    if type(observation) is not dict or frozenset(observation) != _OBSERVATION_KEYS:
        raise ValueError("capture observation schema changed")
    if observation["engine"] != engine:
        raise ValueError("capture observation engine changed")
    if (type(observation["exit_status"]) is not int
            or type(observation["timed_out"]) is not bool
            or type(observation["termination_uncertain"]) is not bool
            or type(observation["maxrss_approximate"]) is not bool):
        raise ValueError("capture observation values have invalid types")
    signal = observation["signal"]
    if signal is not None and type(signal) is not int:
        raise ValueError("capture signal has invalid type")
    for name in ("seconds", "user_cpu_seconds", "system_cpu_seconds"):
        if type(observation[name]) not in (int, float):
            raise ValueError("capture metric has invalid type")
    if type(observation["maxrss_kb"]) is not int:
        raise ValueError("capture RSS has invalid type")


def _runs_by_engine(runs):
    if type(runs) not in (list, tuple) or len(runs) != 2:
        raise ValueError("exactly two engine runs are required")
    result = {}
    for run in runs:
        if type(run) is not dict or frozenset(run) != _RUN_KEYS:
            raise ValueError("capture run schema changed")
        engine = run.get("engine")
        if engine not in _ENGINES or engine in result:
            raise ValueError("oracle and Rust are required exactly once")
        if (type(run.get("stdout")) is not bytes
                or type(run.get("stderr")) is not bytes
                or type(run.get("observation")) is not dict
                or type(run.get("fresh_process")) is not bool
                or type(run.get("invocation_id")) is not str):
            raise ValueError("capture run values have invalid types")
        _validate_observation_schema(run["observation"], engine)
        result[engine] = run
    if set(result) != set(_ENGINES):
        raise ValueError("oracle and Rust are required exactly once")
    return result


def _arm_result(case_id, engine, run):
    prefix = _CASE_PREFIXES[case_id]
    frame = _parse_frame(run["stdout"], case_id, prefix)
    diagnostic_stream = _parse_diagnostics(run["stderr"], engine)
    headers = diagnostic_stream["headers"]
    causes = diagnostic_stream["causes"]
    observation = run["observation"]
    prediction = _ALL_PREDICTIONS[case_id][engine]
    marker_lines = [marker["line"] for marker in frame["markers"]]
    cause_names = [cause["cause"] for cause in causes]
    categories = [header["category"] for header in headers]
    prediction_shape_matches = (
        frame["complete"]
        and diagnostic_stream["complete"]
        and frame["prefix"]["bytes"] == 0
        and frame["suffix"]["text"] == "Bye.\n"
        and frame["unclassified_markers"] == []
        and diagnostic_stream["unclassified"] == []
        and observation.get("exit_status") == prediction["exit_status"]
        and frame["payload_lines"] == prediction["payload_lines"]
        and marker_lines == prediction["marker_lines"]
        and cause_names == prediction["causes"]
        and categories == prediction["diagnostic_categories"]
        and ((run["stderr"] == b"") == prediction["stderr_empty"])
    )
    return {
        "engine": engine,
        "invocation_id": run["invocation_id"],
        "fresh_process": run["fresh_process"],
        "resource_healthy": _resource_healthy(observation, engine),
        "metrics_exact": _exact_metrics(observation),
        "exit_status": observation.get("exit_status"),
        "observation": {
            "engine": observation["engine"],
            "exit_status": observation["exit_status"],
            "timed_out": observation["timed_out"],
            "termination_uncertain": observation["termination_uncertain"],
            "signal": observation["signal"],
            "seconds": _json_metric(observation["seconds"]),
            "user_cpu_seconds": _json_metric(observation["user_cpu_seconds"]),
            "system_cpu_seconds": _json_metric(observation["system_cpu_seconds"]),
            "maxrss_kb": _json_metric(observation["maxrss_kb"]),
            "maxrss_approximate": observation["maxrss_approximate"],
        },
        "metrics": {
            "seconds": _json_metric(observation.get("seconds")),
            "user_cpu_seconds": _json_metric(observation.get("user_cpu_seconds")),
            "system_cpu_seconds": _json_metric(observation.get("system_cpu_seconds")),
            "maxrss_kb": _json_metric(observation.get("maxrss_kb")),
            "maxrss_approximate": observation.get("maxrss_approximate"),
        },
        "stdout": {"sha256": _sha(run["stdout"]), "bytes": len(run["stdout"])},
        "stderr": {"sha256": _sha(run["stderr"]), "bytes": len(run["stderr"])},
        "frame": frame,
        "diagnostic_stream": diagnostic_stream,
        "diagnostic_headers": headers,
        "diagnostic_causes": causes,
        "source_prediction": {
            "exit_status": prediction["exit_status"],
            "marker_lines": list(prediction["marker_lines"]),
            "payload_lines": list(prediction["payload_lines"]),
            "causes": list(prediction["causes"]),
            "diagnostic_categories": list(prediction["diagnostic_categories"]),
            "stderr_empty": prediction["stderr_empty"],
            "shape_matches": prediction_shape_matches,
        },
    }


def _error_summaries(text):
    """Ordered error messages of one engine's stderr, envelope-free.

    The oracle renders each error as `Runtime error:` + indented message +
    `Evaluation aborted.`; the Rust CLI renders it as
    `Runtime error at <stdin>:LINE:COL: message` + a source line and a
    caret line.  Byte-equality across the two renderers is unattainable for
    reject cases, so only the ordered message contents are compared.
    Returns None when any block deviates from its engine's envelope.
    """
    summaries = []
    lines = text.splitlines()
    index = 0
    while index < len(lines):
        line = lines[index]
        if line == "Runtime error:":
            if (index + 2 >= len(lines)
                    or not lines[index + 1].startswith("  ")
                    or lines[index + 2] != "Evaluation aborted."):
                return None
            summaries.append(lines[index + 1].strip())
            index += 3
            continue
        match = re.fullmatch(
            r"Runtime error at <stdin>:\d+:\d+: (\S.*)", line)
        if match is not None:
            if (index + 2 >= len(lines)
                    or not lines[index + 1].startswith("  | ")
                    or not lines[index + 2].startswith("  | ")):
                return None
            summaries.append(match.group(1))
            index += 3
            continue
        return None
    return summaries


def _stderr_equal(intent, oracle_raw, rust_raw):
    """Reject cases compare ordered error messages; accept stays byte-exact."""
    if intent != "reject":
        return oracle_raw == rust_raw
    oracle_summaries = _error_summaries(
        oracle_raw.decode("utf-8", errors="replace"))
    rust_summaries = _error_summaries(
        rust_raw.decode("utf-8", errors="replace"))
    return (oracle_summaries is not None
            and rust_summaries is not None
            and oracle_summaries == rust_summaries)


def classify_capture(case, runs):
    """Classify one two-process capture without ever releasing a gate."""
    expected_case = _validated_case(case)
    by_engine = _runs_by_engine(runs)
    arms = {
        engine: _arm_result(expected_case["id"], engine, by_engine[engine])
        for engine in _ENGINES
    }

    invocation_ids = [by_engine[engine]["invocation_id"] for engine in _ENGINES]
    fresh = (all(arms[engine]["fresh_process"] for engine in _ENGINES)
             and all(invocation_ids) and len(set(invocation_ids)) == 2)
    resources = all(arms[engine]["resource_healthy"] for engine in _ENGINES)
    metrics = all(arms[engine]["metrics_exact"] for engine in _ENGINES)
    streams = all(
        arms[engine]["frame"]["complete"]
        and arms[engine]["diagnostic_stream"]["complete"]
        for engine in _ENGINES
    )
    capture_complete = fresh and resources and metrics and streams
    prediction_shapes = {
        engine: arms[engine]["source_prediction"]["shape_matches"]
        for engine in _ENGINES
    }
    predictions = {
        engine: capture_complete and prediction_shapes[engine]
        for engine in _ENGINES
    }

    if not fresh:
        status = "CAPTURE_INCOMPLETE_FRESH_PROCESS"
    elif not resources:
        status = "CAPTURE_INCOMPLETE_RESOURCE_OR_TIMEOUT"
    elif not metrics:
        status = "CAPTURE_INCOMPLETE_METRICS"
    elif not streams:
        status = "CAPTURE_INCOMPLETE_STREAM"
    elif not predictions["oracle"] and not predictions["rust"]:
        status = "BOTH_SOURCE_PREDICTIONS_DIFFERED"
    elif not predictions["oracle"]:
        status = "ORIGINAL_SOURCE_PREDICTION_DIFFERED"
    elif not predictions["rust"]:
        status = "RUST_SOURCE_PREDICTION_DIFFERED"
    else:
        status = "SOURCE_PREDICTIONS_OBSERVED_UNREVIEWED"

    oracle = by_engine["oracle"]
    rust = by_engine["rust"]
    result = {
        "status": status,
        "evidence_maturity": CAPTURE_MATURITY,
        "acceptance_eligible": False,
        "math_gate_released": False,
        "cache_gate_released": False,
        "case": dict(expected_case),
        "fresh_process_complete": fresh,
        "resource_complete": resources,
        "metrics_complete": metrics,
        "stream_complete": streams,
        "prediction_shape_matches": prediction_shapes,
        "source_predictions_observed": predictions,
        "full_stdout_equal": oracle["stdout"] == rust["stdout"],
        "full_stderr_equal": _stderr_equal(
            expected_case["intent"], oracle["stderr"], rust["stderr"]),
        "exit_status_equal": (
            oracle["observation"].get("exit_status")
            == rust["observation"].get("exit_status")
        ),
        "arms": arms,
    }
    if status not in NONACCEPTING_STATUSES or any(
        result[name]
        for name in ("acceptance_eligible", "math_gate_released", "cache_gate_released")
    ):
        raise AssertionError("capture classifier attempted to release a gate")
    return result

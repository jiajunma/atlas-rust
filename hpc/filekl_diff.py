#!/usr/bin/env python3
"""Differential check: Rust-written KL binary files vs upstream KLread oracle.

Pipeline:
  1. Run the #[ignore]d dump test (`filekl_dump`) in atlas-real-group with
     FILEKL_DUMP_DIR set; it writes <name>.block / <name>.matrix / <name>.kl
     plus <name>.json expectations (block size, rank, per-pair KL polynomial
     coefficients, pool size) for a fixed set of small blocks.
  2. For each block, drive the upstream KLread binary (built from
     sources/stand-alone/KLread.cpp) with a scripted stdin: header lines are
     captured, then every listed (x, y) pair is queried and `quit` sent.
  3. Parse KLread output semantically (coefficient vector per polynomial,
     value at q=1) and compare against the JSON expectations.
  4. Emit results/<sha>/<job>/filekl_report.json and a human summary.

Exit status: 0 when every block matches, 1 otherwise.
"""
from __future__ import annotations

import json
import os
import re
import subprocess
import sys
from pathlib import Path

KLREAD = os.environ.get(
    "KLREAD_BIN", "/public/home/majj/atlasofliegroups/sources/stand-alone/KLread"
)

# KLread prints polynomials with terms in DECREASING degree, joined by
# " + ": ``2q^3 + q^2 + 3.`` (braced exponents ``q^{12}`` for degree >= 10,
# coefficient omitted when it is 1 except for the constant term). Parse into
# a degree -> coefficient map. Coefficients are unsigned; only ``+`` appears.
def parse_poly(text: str) -> dict[int, int]:
    """Parse a KLread polynomial string into {degree: coeff}."""
    text = text.strip().rstrip(".").strip()
    if text in ("0", ""):
        return {}
    coeffs: dict[int, int] = {}
    for term in text.split("+"):
        term = term.strip()
        qm = re.fullmatch(r"(\d*)\s*q(?:\^\{?(\d+)\}?)?", term)
        if qm:
            coef = int(qm.group(1)) if qm.group(1) else 1
            deg = int(qm.group(2)) if qm.group(2) else 1
        elif re.fullmatch(r"\d+", term):
            coef, deg = int(term), 0
        else:
            raise ValueError(f"unparseable term {term!r} in {text!r}")
        coeffs[deg] = coeffs.get(deg, 0) + coef
    return {d: c for d, c in coeffs.items() if c != 0}


def poly_value_at_one(coeffs: dict[int, int]) -> int:
    return sum(coeffs.values())


def coeffs_to_list(coeffs: dict[int, int]) -> list[int]:
    if not coeffs:
        return []
    top = max(coeffs)
    return [coeffs.get(d, 0) for d in range(top + 1)]


def run_klread(block: Path, matrix: Path, kl: Path, queries: list[str]) -> str:
    stdin = "".join(q + "\n" for q in queries) + "quit\n"
    proc = subprocess.run(
        [KLREAD, str(block), str(matrix), str(kl)],
        input=stdin,
        capture_output=True,
        text=True,
        timeout=300,
    )
    if proc.returncode != 0:
        raise RuntimeError(
            f"KLread failed ({proc.returncode}) for {block.name}: {proc.stderr.strip()}"
        )
    return proc.stdout


# Per-query stdout shape (KLread.cpp main loop):
#   P_{x,y}=P_{x_prim,y}=polynomial #i:
#   <terms joined by " + ">[; value at q=1: N].
# Errors (triangularity, real non-parity, index too large) go to stderr, so
# failed queries simply produce no stdout block.
PAIR_RE = re.compile(
    r"P_\{(\d+),(\d+)\}=P_\{(\d+),(\d+)\}=polynomial #(\d+):\s*\n"
    r"([^\n;.]+(?:\s*\+\s*[^\n;.]+)*)(?:; value at q=1: (\d+))?\."
)
RANK_RE = re.compile(r"rank=(\d+)")
SIZE_RE = re.compile(r"block size=(\d+)")


def compare_block(dump_dir: Path, name: str, max_pairs: int) -> dict:
    block = dump_dir / f"{name}.block"
    matrix = dump_dir / f"{name}.matrix"
    kl = dump_dir / f"{name}.kl"
    expected = json.loads((dump_dir / f"{name}.json").read_text())
    size = expected["size"]

    expected_map = {}
    for entry in expected["polynomials"]:
        expected_map[(entry["x"], entry["y"])] = entry["coeffs"]

    # Only probe pairs that carry a stored expectation; others (triangular
    # zeroes, real non-parity ascents) legitimately error out in KLread.
    pairs = sorted(expected_map)
    if max_pairs and len(pairs) > max_pairs:
        step = max(1, len(pairs) // max_pairs)
        pairs = pairs[::step]
    queries = [f"{x},{y}" for (x, y) in pairs]
    out = run_klread(block, matrix, kl, queries)

    result = {
        "name": name,
        "size": size,
        "rank_expected": expected["rank"],
        "pairs_checked": 0,
        "mismatches": [],
        "status": "pass",
    }

    m = RANK_RE.search(out)
    if not m or int(m.group(1)) != expected["rank"]:
        result["mismatches"].append(
            {"kind": "rank", "expected": expected["rank"], "got": m.group(1) if m else None}
        )
    m = SIZE_RE.search(out)
    if not m or int(m.group(1)) != size:
        result["mismatches"].append(
            {"kind": "block_size", "expected": size, "got": m.group(1) if m else None}
        )

    got_pairs = {}
    for pm in PAIR_RE.finditer(out):
        x, y = int(pm.group(1)), int(pm.group(2))
        try:
            coeffs = parse_poly(pm.group(6))
        except ValueError as exc:
            result["mismatches"].append({"kind": "parse", "detail": str(exc)})
            continue
        if pm.group(7) is not None and int(pm.group(7)) != poly_value_at_one(coeffs):
            result["mismatches"].append(
                {
                    "kind": "value_at_one",
                    "x": x,
                    "y": y,
                    "expected": poly_value_at_one(coeffs),
                    "got": int(pm.group(7)),
                }
            )
        got_pairs[(x, y)] = coeffs

    for (x, y) in pairs:
        exp = expected_map.get((x, y))
        got = got_pairs.get((x, y))
        if exp is None:
            continue  # pair probed but no stored expectation
        if got is None:
            if exp:
                result["mismatches"].append({"kind": "missing_pair", "x": x, "y": y})
            else:
                # Zero polynomial: KLread refuses the query ("Result null by
                # triangularity." or the real non-parity message) on stderr
                # instead of printing a polynomial. That IS the zero answer.
                result["pairs_checked"] += 1
            continue
        if coeffs_to_list(got) != exp:
            result["mismatches"].append(
                {
                    "kind": "poly",
                    "x": x,
                    "y": y,
                    "expected": exp,
                    "got": coeffs_to_list(got),
                }
            )
        else:
            result["pairs_checked"] += 1

    if result["mismatches"]:
        result["status"] = "fail"
    return result


def main() -> int:
    if len(sys.argv) < 3:
        print("usage: filekl_diff.py <dump_dir> <report_path> [max_pairs]", file=sys.stderr)
        return 2
    dump_dir = Path(sys.argv[1])
    report_path = Path(sys.argv[2])
    max_pairs = int(sys.argv[3]) if len(sys.argv) > 3 else 0

    names = sorted(p.stem for p in dump_dir.glob("*.json"))
    if not names:
        print(f"no expectation JSON files in {dump_dir}", file=sys.stderr)
        return 2

    blocks = []
    for name in names:
        try:
            blocks.append(compare_block(dump_dir, name, max_pairs))
        except Exception as exc:  # noqa: BLE001 - report and continue
            blocks.append({"name": name, "status": "error", "detail": str(exc)})

    passed = sum(1 for b in blocks if b.get("status") == "pass")
    report = {"blocks": blocks, "passed": passed, "total": len(blocks)}
    report_path.parent.mkdir(parents=True, exist_ok=True)
    report_path.write_text(json.dumps(report, indent=2))

    for b in blocks:
        line = f"{b['name']}: {b['status'].upper()}"
        if b.get("pairs_checked"):
            line += f" ({b['pairs_checked']} pairs)"
        if b.get("mismatches"):
            line += f" first mismatch: {json.dumps(b['mismatches'][0])}"
        if b.get("detail"):
            line += f" {b['detail']}"
        print(line)
    print(f"filekl diff: {passed}/{len(blocks)} blocks PASS")
    return 0 if passed == len(blocks) else 1


if __name__ == "__main__":
    sys.exit(main())

#!/usr/bin/env python3
"""Locate differences in already-reviewed full KL history output; do not rerun Atlas."""
import hashlib
import json
import os
from pathlib import Path
import re


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def history_records(raw, case_id):
    start, end = "MATH_BEGIN " + case_id + "\n", "MATH_END " + case_id + "\n"
    if raw.count(start) != 1 or raw.count(end) != 1:
        raise ValueError("missing or duplicate history boundary")
    body = raw.split(start, 1)[1].split(end, 1)[0]
    chunks = re.split(r"(?m)^PARAMETER", body)
    if chunks[0] or len(chunks) != 4:
        raise ValueError("expected all three parameter histories")
    result = []
    for chunk in chunks[1:]:
        parameter, rest = chunk.split("\n", 1)
        cold, rest = rest.split("\nFULL", 1)
        if not cold.startswith("COLD"):
            raise ValueError("missing cold result")
        cold, expected = cold[4:].split("EXPECTED_PARAMETERS", 1)
        full, warm = rest.split("WARM", 1)
        result.append(dict(parameter=parameter, cold=cold, expected=expected, full=full, warm=warm))
    return result


def section_difference(left, right):
    result = {"equal": left == right, "oracle_chars": len(left), "rust_chars": len(right)}
    if left == right:
        return result
    first = next((i for i, pair in enumerate(zip(left, right)) if pair[0] != pair[1]),
                 min(len(left), len(right)))
    result.update(first_differing_character=first,
                  oracle_context=left[max(0, first - 100):first + 200],
                  rust_context=right[max(0, first - 100):first + 200])
    rows = [list(re.finditer(r"(?m)^\|([ ,0-9+-]*)\|$", text)) for text in (left, right)]
    result["matrix_row_counts"] = [len(x) for x in rows]
    if rows[0] and rows[1]:
        result["parameter_prefix_equal"] = left[:rows[0][0].start()] == right[:rows[1][0].start()]
        differences = []
        for i, pair in enumerate(zip(*rows)):
            values = [[int(x) for x in re.findall(r"-?\d+", row[1])] for row in pair]
            if len(values[0]) != len(values[1]):
                differences.append({"row": i, "column_counts": list(map(len, values))})
            differences.extend({"row": i, "column": j, "oracle": a, "rust": b}
                               for j, (a, b) in enumerate(zip(*values)) if a != b)
        result["matrix_differing_entries"] = len(differences)
        result["first_matrix_differences"] = differences[:12]
        result["polynomial_suffix_equal"] = left[rows[0][-1].end():] == right[rows[1][-1].end():]
        pools = [[[int(x) for x in re.findall(r"-?\d+", vector)]
                  for vector in re.findall(r"\[([ ,0-9+-]*)\]", text[matches[-1].end():])]
                 for text, matches in zip((left, right), rows)]
        result["polynomial_pool_sizes"] = list(map(len, pools))
        changed = []
        for index, (a, b) in enumerate(zip(*pools)):
            if a == b:
                continue
            references = []
            for i, row in enumerate(rows[0]):
                references.extend([i, j] for j, value in enumerate(re.findall(r"-?\d+", row[1]))
                                  if int(value) == index)
            changed.append(dict(index=index, oracle=a, rust=b, oracle_matrix_references=references))
        result["changed_polynomials"] = changed
    return result


def main():
    if not os.environ.get("SLURM_JOB_ID"):
        raise SystemExit("Stored-output diagnosis runs on an HPC compute node")
    root = Path.cwd()
    if digest(root / "diagnose.json") != os.environ["DIAGNOSE_SHA256"]:
        raise ValueError("diagnosis pin changed")
    pin = json.loads((root / "diagnose.json").read_text())
    for name, sha in pin["inputs"].items():
        if digest(root / name) != sha:
            raise ValueError("diagnostic input changed")
    if digest(os.environ["MATH_DIAGNOSE_SPOOL"]) != pin["inputs"]["hpc/math_kl_history_diagnose.sbatch"]:
        raise ValueError("submitted script changed")
    if digest(pin["review"]) != pin["review_sha256"] or digest(pin["report"]) != pin["report_sha256"]:
        raise ValueError("reviewed evidence changed")
    review = json.loads(Path(pin["review"]).read_text())
    report = json.loads(Path(pin["report"]).read_text())
    case = next(x for x in review["cases"] if x["id"] == report["case"]["id"])
    if case["status"] != "MATH_MISMATCH" or case["report_sha256"] != pin["report_sha256"]:
        raise ValueError("diagnosis requires a previously reviewed mathematical mismatch")
    records = {}
    for engine, observation in report["observations"].items():
        if observation["exit_status"] != 0 or observation["timed_out"]:
            raise ValueError("execution did not complete")
        for artifact in observation["artifacts"].values():
            if digest(artifact["path"]) != artifact["sha256"]:
                raise ValueError("stored stream changed")
        if Path(observation["artifacts"]["stderr"]["path"]).read_bytes():
            raise ValueError("execution had diagnostics")
        records[engine] = history_records(
            Path(observation["artifacts"]["stdout"]["path"]).read_text(), report["case"]["id"])
    comparisons = [{name: section_difference(a[name], b[name]) for name in a}
                   for a, b in zip(records["oracle"], records["rust"])]
    result = {"schema": "atlas-kl-history-diagnosis-v1", "job": os.environ["SLURM_JOB_ID"],
              "status": "DIFFERENCES_LOCATED_NOT_REPAIRED", "pin": pin,
              "comparisons": comparisons,
              "scope": "Exact strings compared without changing the original failure classification. Matrix summaries only locate differing entries; raw streams remain authoritative."}
    out = root / "results" / os.environ["SLURM_JOB_ID"]
    out.mkdir(parents=True, exist_ok=False)
    path = out / "diagnosis.json"
    path.write_text(json.dumps(result, indent=2, sort_keys=True) + "\n")
    path.with_suffix(".sha256").write_text(digest(path) + "\n")
    print(result["status"], path, flush=True)


if __name__ == "__main__":
    main()

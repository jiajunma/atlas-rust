#!/usr/bin/env python3
"""KGB differential driver: upstream C++ atlas vs the Rust port.

For each group in the battery, runs (a) the upstream `atlas` interpreter with
a generated script printing per-real-form KGB sizes and sorted length
multisets in the EXTERNAL numbering, and (b) the Rust `kgb_bench` example
printing the same observables in the same order. Compares POSITIONALLY —
this validates the port's ExternalFormOrder against the live oracle — and
records wall-clock times for both sides.

Usage: kgb_differential.py <atlas-binary> <kgb-bench-binary> [groups...]
Writes a JSON report to stdout's directory-side file given by $REPORT (or
kgb_differential_report.json).
"""

import json
import os
import re
import subprocess
import sys
import time

DEFAULT_BATTERY = [
    "A1", "A2", "A3", "A4", "B2", "B3", "B4", "C3", "C4", "D4", "G2", "F4",
]


def atlas_script(group: str) -> str:
    # Equal-rank inner class ("e" per simple factor) of the simply connected
    # group; print size and the length multiset per external form number.
    loop = (
        f"for i:n do let rf = real_form(ic,i) in "
        f'prints("{group} form ", i, " size ", KGB_size(rf), " lengths ", '
        "for x:KGB_size(rf) do length(KGB(rf,x)) od) od"
    )
    return (
        "<basic.at\n"
        f'set rd = simply_connected(Lie_type("{group}"))\n'
        'set ic = inner_class(rd,"e")\n'
        "set n = nr_of_real_forms(ic)\n"
        f"{loop}\n"
        "quit\n"
    )


def parse_lines(text: str, group: str):
    forms = {}
    pattern = re.compile(
        rf"^{re.escape(group)} form (\d+) size (\d+) lengths \[?([0-9,\s]*)\]?\s*$"
    )
    for line in text.splitlines():
        match = pattern.match(line.strip())
        if not match:
            continue
        external = int(match.group(1))
        size = int(match.group(2))
        lengths = sorted(
            int(part) for part in match.group(3).replace(" ", "").split(",") if part
        )
        forms[external] = {"size": size, "lengths": lengths}
    return forms


def run_group(atlas_bin: str, bench_bin: str, group: str):
    entry = {"group": group}

    script = atlas_script(group)
    start = time.monotonic()
    cpp = subprocess.run(
        [atlas_bin],
        input=script,
        text=True,
        capture_output=True,
        timeout=1800,
        cwd=os.path.join(os.path.dirname(atlas_bin) or ".", "atlas-scripts"),
    )
    entry["cpp_seconds"] = round(time.monotonic() - start, 3)
    cpp_forms = parse_lines(cpp.stdout, group)

    start = time.monotonic()
    rust = subprocess.run(
        [bench_bin, group], text=True, capture_output=True, timeout=1800
    )
    entry["rust_seconds"] = round(time.monotonic() - start, 3)
    rust_forms = parse_lines(rust.stdout, group)
    timing = re.search(
        rf"^{re.escape(group)} time_ms (\d+) kgb_ms (\d+)", rust.stdout, re.M
    )
    if timing:
        entry["rust_pipeline_ms"] = int(timing.group(1))
        entry["rust_kgb_ms"] = int(timing.group(2))

    entry["cpp_forms"] = cpp_forms
    entry["rust_forms"] = rust_forms
    entry["form_count_match"] = len(cpp_forms) == len(rust_forms)
    mismatches = []
    for external in sorted(set(cpp_forms) | set(rust_forms)):
        left = cpp_forms.get(external)
        right = rust_forms.get(external)
        if left != right:
            mismatches.append(
                {"external": external, "cpp": left, "rust": right}
            )
    entry["mismatches"] = mismatches
    entry["match"] = entry["form_count_match"] and not mismatches
    if rust.returncode != 0:
        entry["rust_error"] = rust.stderr[-2000:]
        entry["match"] = False
    if cpp.returncode != 0 and not cpp_forms:
        entry["cpp_error"] = (cpp.stderr or cpp.stdout)[-2000:]
        entry["match"] = False
    return entry


def main() -> int:
    atlas_bin, bench_bin = sys.argv[1], sys.argv[2]
    battery = sys.argv[3:] or DEFAULT_BATTERY
    report = {
        "schema": "atlas-kgb-differential-v1",
        "battery": battery,
        "groups": [],
    }
    for group in battery:
        entry = run_group(atlas_bin, bench_bin, group)
        report["groups"].append(entry)
        status = "MATCH" if entry["match"] else "MISMATCH"
        print(
            f"{group}: {status} cpp={entry['cpp_seconds']}s "
            f"rust={entry['rust_seconds']}s",
            flush=True,
        )
    report["all_match"] = all(entry["match"] for entry in report["groups"])
    path = os.environ.get("REPORT", "kgb_differential_report.json")
    with open(path, "w", encoding="utf-8") as handle:
        json.dump(report, handle, indent=2)
    print(f"report: {path} all_match={report['all_match']}")
    return 0 if report["all_match"] else 1


if __name__ == "__main__":
    raise SystemExit(main())

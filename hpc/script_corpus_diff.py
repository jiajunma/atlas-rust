#!/usr/bin/env python3
"""Corpus differential: run the upstream atlas-scripts .at corpus through
both interpreters and score compatibility.

For every .at file: feed it to the upstream `atlas` interpreter (cwd =
atlas-scripts so `<file` includes resolve) and to the Rust `atlas-cli`.
Classify each file:

  MATCH             both load cleanly and stdout agrees
  OUTPUT_DIFF       both load cleanly, stdout differs
  RUST_PARSE_FAIL   the Rust side reports lexical/syntax diagnostics
  RUST_EVAL_FAIL    the Rust side reports name/type/runtime diagnostics
  CPP_FAIL          the upstream side itself fails to load
  SKIPPED_LARGE     data files above the size cap (listed, not scored)

The report includes a histogram of first-error messages on the Rust side —
the corpus-driven priority list for the next language features.

Usage: script_corpus_diff.py <atlas-binary> <atlas-cli-binary> [globs...]
Env: REPORT (output json), SIZE_CAP bytes (default 2 MiB), TIMEOUT seconds.
"""

import glob
import json
import os
import re
import subprocess
import sys
import time


def classify_rust(stderr: str) -> str:
    first = stderr.splitlines()[0] if stderr else ""
    if first.startswith(("Lexical", "Syntax")):
        return "RUST_PARSE_FAIL"
    return "RUST_EVAL_FAIL"


def first_error(stderr: str) -> str:
    for line in stderr.splitlines():
        line = line.strip()
        if line:
            # Collapse identifiers/numbers so the histogram groups shapes.
            line = re.sub(r"`[^`]*`", "`_`", line)
            line = re.sub(r"\d+", "N", line)
            return line[:120]
    return "(no diagnostic)"


def run_corpus(atlas_bin, cli_bin, files, size_cap, timeout):
    scripts_dir = os.path.join(os.path.dirname(atlas_bin) or ".", "atlas-scripts")
    entries = []
    for path in files:
        name = os.path.basename(path)
        size = os.path.getsize(path)
        entry = {"script": name, "bytes": size}
        if size > size_cap:
            entry["category"] = "SKIPPED_LARGE"
            entries.append(entry)
            continue
        text = open(path, encoding="utf-8", errors="replace").read()

        start = time.monotonic()
        try:
            cpp = subprocess.run(
                [atlas_bin],
                input=text + "\nquit\n",
                text=True,
                capture_output=True,
                timeout=timeout,
                cwd=scripts_dir,
            )
            cpp_ok = cpp.returncode == 0 and "error" not in cpp.stderr.lower()
            cpp_out = cpp.stdout
        except subprocess.TimeoutExpired:
            cpp_ok, cpp_out = False, ""
        entry["cpp_seconds"] = round(time.monotonic() - start, 3)

        start = time.monotonic()
        try:
            # Mirror the C++ invocation: cwd = atlas-scripts so `<basic.at`
            # resolves (and prints) the same cwd-relative spelling.
            rust = subprocess.run(
                [cli_bin, os.path.abspath(path)],
                text=True,
                capture_output=True,
                timeout=timeout,
                cwd=scripts_dir,
            )
        except subprocess.TimeoutExpired:
            entry["category"] = "RUST_EVAL_FAIL"
            entry["rust_first_error"] = "(timeout)"
            entries.append(entry)
            continue
        entry["rust_seconds"] = round(time.monotonic() - start, 3)

        if not cpp_ok:
            entry["category"] = "CPP_FAIL"
        elif rust.returncode == 0:
            entry["category"] = (
                "MATCH" if rust.stdout == cpp_out else "OUTPUT_DIFF"
            )
        else:
            entry["category"] = classify_rust(rust.stderr)
            entry["rust_first_error"] = first_error(rust.stderr)
        entries.append(entry)
    return entries


def main() -> int:
    atlas_bin, cli_bin = sys.argv[1], sys.argv[2]
    patterns = sys.argv[3:] or [
        os.path.join(
            os.path.dirname(atlas_bin) or ".", "atlas-scripts", "*.at"
        )
    ]
    files = sorted({p for pattern in patterns for p in glob.glob(pattern)})
    size_cap = int(os.environ.get("SIZE_CAP", 2 * 1024 * 1024))
    timeout = int(os.environ.get("TIMEOUT", 120))
    entries = run_corpus(atlas_bin, cli_bin, files, size_cap, timeout)

    counts = {}
    histogram = {}
    for entry in entries:
        counts[entry["category"]] = counts.get(entry["category"], 0) + 1
        if "rust_first_error" in entry:
            key = entry["rust_first_error"]
            histogram[key] = histogram.get(key, 0) + 1
    report = {
        "schema": "atlas-script-corpus-diff-v1",
        "total": len(entries),
        "counts": counts,
        "first_error_histogram": dict(
            sorted(histogram.items(), key=lambda kv: -kv[1])
        ),
        "scripts": entries,
    }
    path = os.environ.get("REPORT", "script_corpus_report.json")
    with open(path, "w", encoding="utf-8") as handle:
        json.dump(report, handle, indent=2)
    print(f"corpus: {len(entries)} scripts")
    for category in sorted(counts):
        print(f"  {category}: {counts[category]}")
    print("top blockers:")
    for message, count in list(report["first_error_histogram"].items())[:12]:
        print(f"  {count:4d}  {message}")
    print(f"report: {path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

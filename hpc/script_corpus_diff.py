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
import pathlib
import re
import resource
import shutil
import subprocess
import sys
import tempfile
import time


TIME_BIN = shutil.which("time")
USE_GNU_TIME = bool(TIME_BIN and sys.platform != "darwin")


def parse_time_metrics(path):
    if not path.exists():
        return None
    for line in path.read_text(encoding="utf-8", errors="replace").splitlines():
        if "maximum resident set size" in line.lower():
            digits = re.sub(r"\D", "", line.split(":", 1)[-1])
            if digits:
                return int(digits)
    return None


def measure_command(argv, *, cwd, timeout, input_text=None):
    """Run one interpreter and return output plus wall time and peak RSS."""
    started = time.monotonic()
    timed_out = False
    command = list(argv)
    temporary = None
    metric_path = None
    if USE_GNU_TIME:
        temporary = tempfile.TemporaryDirectory()
        metric_path = os.path.join(temporary.name, "time.metrics")
        command = [TIME_BIN, "-v", "-o", metric_path, *command]

    def limit_memory():
        # Contain runaway allocations (e.g. a diverging lazy list) so one
        # script cannot OOM-kill the whole SLURM job.
        cap = int(os.environ.get("MEM_CAP_GB", "6")) * 1024**3
        resource.setrlimit(resource.RLIMIT_AS, (cap, cap))

    try:
        completed = subprocess.run(
            command,
            input=input_text,
            text=True,
            capture_output=True,
            timeout=timeout,
            cwd=cwd,
            preexec_fn=limit_memory,
        )
        stdout, stderr, exit_status = (
            completed.stdout,
            completed.stderr,
            completed.returncode,
        )
    except subprocess.TimeoutExpired as error:
        timed_out = True
        stdout = error.stdout or ""
        stderr = error.stderr or ""
        if isinstance(stdout, bytes):
            stdout = stdout.decode("utf-8", errors="replace")
        if isinstance(stderr, bytes):
            stderr = stderr.decode("utf-8", errors="replace")
        exit_status = None
    seconds = round(time.monotonic() - started, 3)
    if metric_path is not None:
        maxrss_kb = parse_time_metrics(pathlib.Path(metric_path))
        temporary.cleanup()
        approximate = False
    else:
        approximate = True
        try:
            maxrss_kb = int(resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss)
            if sys.platform == "darwin":
                maxrss_kb //= 1024
        except (AttributeError, ValueError):
            maxrss_kb = None
    return {
        "stdout": stdout,
        "stderr": stderr,
        "exit_status": exit_status,
        "timed_out": timed_out,
        "seconds": seconds,
        "maxrss_kb": maxrss_kb,
        "maxrss_approximate": approximate,
    }


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

        cpp = measure_command(
            [atlas_bin], input_text=text + "\nquit\n",
            timeout=timeout, cwd=scripts_dir,
        )
        cpp_ok = (
            not cpp["timed_out"] and cpp["exit_status"] == 0
            and "error" not in cpp["stderr"].lower()
        )
        cpp_out = cpp["stdout"]
        for key in ("seconds", "maxrss_kb", "maxrss_approximate", "timed_out"):
            entry[f"cpp_{key}"] = cpp[key]
        entry["cpp_exit_status"] = cpp["exit_status"]
        entry["cpp_stderr"] = cpp["stderr"]

        # Mirror the C++ invocation: cwd = atlas-scripts so `<basic.at`
        # resolves (and prints) the same cwd-relative spelling.
        rust = measure_command(
            [cli_bin], input_text=text + "\nquit\n",
            timeout=timeout, cwd=scripts_dir,
        )
        for key in ("seconds", "maxrss_kb", "maxrss_approximate", "timed_out"):
            entry[f"rust_{key}"] = rust[key]
        entry["rust_exit_status"] = rust["exit_status"]
        entry["rust_stderr"] = rust["stderr"]
        if cpp["seconds"] > 0:
            entry["rust_to_cpp_seconds"] = round(
                rust["seconds"] / cpp["seconds"], 3
            )
        if cpp["maxrss_kb"] and rust["maxrss_kb"]:
            entry["rust_to_cpp_maxrss"] = round(
                rust["maxrss_kb"] / cpp["maxrss_kb"], 3
            )

        if not cpp_ok:
            entry["category"] = "CPP_FAIL"
        elif rust["timed_out"]:
            entry["category"] = "RUST_EVAL_FAIL"
            entry["rust_first_error"] = "(timeout)"
        elif rust["exit_status"] == 0:
            entry["category"] = (
                "MATCH" if rust["stdout"] == cpp_out else "OUTPUT_DIFF"
            )
        else:
            entry["category"] = classify_rust(rust["stderr"])
            entry["rust_first_error"] = first_error(rust["stderr"])
        entries.append(entry)
    return entries


def main() -> int:
    # The Rust CLI runs with cwd=scripts_dir, so its path must be absolute.
    atlas_bin, cli_bin = sys.argv[1], os.path.abspath(sys.argv[2])
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
    comparable = [
        entry for entry in entries
        if entry.get("category") in {"MATCH", "OUTPUT_DIFF"}
        and "rust_to_cpp_seconds" in entry
    ]
    report["benchmark_summary"] = {
        "comparable_scripts": len(comparable),
        "rust_faster": sum(
            entry["rust_to_cpp_seconds"] < 1 for entry in comparable
        ),
        "within_2x": sum(
            entry["rust_to_cpp_seconds"] <= 2 for entry in comparable
        ),
        "over_5x_slower": sum(
            entry["rust_to_cpp_seconds"] > 5 for entry in comparable
        ),
        "slowest": [
            {
                "script": entry["script"],
                "rust_to_cpp_seconds": entry["rust_to_cpp_seconds"],
                "rust_seconds": entry["rust_seconds"],
                "cpp_seconds": entry["cpp_seconds"],
            }
            for entry in sorted(
                comparable,
                key=lambda item: item["rust_to_cpp_seconds"],
                reverse=True,
            )[:10]
        ],
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

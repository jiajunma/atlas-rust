#!/bin/bash
# Freeze the first lexer corpus on HPC. Execution is intentionally HPC-only.
# The Atlas binary path and Rust command are site-specific job parameters.
#SBATCH --job-name=atlas-lexer
#SBATCH --output=results/%x-%j.log
set -euo pipefail

commit="$(git rev-parse HEAD)"
mkdir -p "results/$commit/$SLURM_JOB_ID"
report="results/$commit/$SLURM_JOB_ID/lexer.json"
python3 - "$report" "$commit" <<'PY'
import json, pathlib, sys
report, commit = sys.argv[1:]
fixtures = sorted(str(p) for p in pathlib.Path("tests/fixtures/lex").glob("*.atlas"))
pathlib.Path(report).write_text(json.dumps({
    "schema": "atlas-differential-v0",
    "stage": "lexer",
    "commit": commit,
    "fixtures": fixtures,
    "status": "reference_command_pending",
}, indent=2) + "\n")
PY

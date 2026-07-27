#!/bin/bash
# Run the first lexer verification on HPC. Execution is intentionally HPC-only.
#SBATCH --job-name=atlas-lexer
#SBATCH --output=atlas-lexer-%j.log
set -euo pipefail
export PATH="$HOME/.cargo/bin:$PATH"

commit="${ATLAS_COMMIT:-$(git rev-parse HEAD)}"
mkdir -p "results/$commit/$SLURM_JOB_ID"
report="results/$commit/$SLURM_JOB_ID/lexer.json"
rustc_version="$(rustc --version)"
cargo test --workspace --locked
python3 - "$report" "$commit" "$rustc_version" "$SLURM_JOB_ID" <<'PY'
import json, pathlib, sys
report, commit, rustc_version, job_id = sys.argv[1:]
fixtures = sorted(str(p) for p in pathlib.Path("tests/fixtures/lex").glob("*.atlas"))
pathlib.Path(report).write_text(json.dumps({
    "schema": "atlas-differential-v0",
    "stage": "lexer",
    "commit": commit,
    "rustc": rustc_version,
    "slurm_job_id": job_id,
    "fixtures": fixtures,
    "rust_status": "passed",
    "reference_status": "pending_reference_token_adapter",
}, indent=2) + "\n")
PY

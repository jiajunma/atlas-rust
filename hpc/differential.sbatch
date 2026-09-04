#!/usr/bin/env bash
# Run the lexer-stage verification on XMU HPC. This is deliberately not yet an
# Atlas differential job: no reference token adapter has been implemented.
#SBATCH --job-name=atlas-lexer
#SBATCH --partition=cpu
#SBATCH --account=yushilingroup
#SBATCH --qos=normal
#SBATCH --nodes=1
#SBATCH --ntasks=1
#SBATCH --cpus-per-task=2
#SBATCH --mem=8G
#SBATCH --time=00:20:00
#SBATCH --output=atlas-lexer-%j.out

set -euo pipefail
cd "$SLURM_SUBMIT_DIR"
export PATH="$HOME/.cargo/bin:$PATH"

if [[ -n "${ATLAS_COMMIT:-}" ]]; then
  commit="$ATLAS_COMMIT"
elif git rev-parse --verify HEAD >/dev/null 2>&1; then
  commit="$(git rev-parse HEAD)"
else
  commit="unversioned"
fi
job_id="${SLURM_JOB_ID:?SLURM_JOB_ID is required}"
run_dir="results/$commit/$job_id"
report="$run_dir/lexer.json"
rust_status="not_started"
started_at="$(date --iso-8601=seconds)"

if [[ -n "${ATLAS_DIRTY_TREE:-}" ]]; then
  dirty_tree="$ATLAS_DIRTY_TREE"
elif git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  if git diff --quiet && git diff --cached --quiet; then
    dirty_tree=false
  else
    dirty_tree=true
  fi
else
  dirty_tree=unknown
fi

mkdir -p "$run_dir"

write_report() {
  local exit_status="$1"
  local ended_at
  ended_at="$(date --iso-8601=seconds)"
  python3 - "$report" "$commit" "$job_id" "$started_at" "$ended_at" \
    "$dirty_tree" "$rust_status" "$exit_status" \
    "${ATLAS_REFERENCE_REVISION:-not-provided}" \
    "${ATLAS_CWEB_VERSION:-not-provided}" <<'PY'
import hashlib
import json
import os
import pathlib
import socket
import subprocess
import sys

(
    report,
    commit,
    job_id,
    started_at,
    ended_at,
    dirty_tree,
    rust_status,
    exit_status,
    reference_revision,
    cweb_version,
) = sys.argv[1:]

fixtures = []
for path in sorted(pathlib.Path("tests/fixtures/lex").glob("*.atlas")):
    fixtures.append(
        {
            "path": str(path),
            "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
        }
    )

def command_output(command):
    try:
        return subprocess.check_output(command, text=True).strip()
    except (OSError, subprocess.CalledProcessError):
        return "unavailable"

payload = {
    "schema": "atlas-differential-v1",
    "stage": "lexer-preflight",
    "differential_status": "not-run",
    "reason": "reference token adapter is not implemented",
    "commit": commit,
    "dirty_tree": {"true": True, "false": False}.get(dirty_tree),
    "source_tree_state": dirty_tree,
    "rustc": command_output(["rustc", "--version"]),
    "cargo": command_output(["cargo", "--version"]),
    "reference_atlas_revision": reference_revision,
    "cweb_version": cweb_version,
    "slurm": {
        "job_id": job_id,
        "node_list": os.environ.get("SLURM_JOB_NODELIST", "unavailable"),
        "cpus_per_task": os.environ.get("SLURM_CPUS_PER_TASK", "unavailable"),
        "hostname": socket.gethostname(),
    },
    "started_at": started_at,
    "ended_at": ended_at,
    "exit_status": int(exit_status),
    "rust_status": rust_status,
    "reference_status": "pending_reference_token_adapter",
    "fixtures": fixtures,
}
pathlib.Path(report).write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n")
PY
  sha256sum "$report" >"$report.sha256"
}

on_exit() {
  local exit_status=$?
  set +e
  write_report "$exit_status"
  exit "$exit_status"
}
trap on_exit EXIT

echo "host=$(hostname) job=$job_id cpus=${SLURM_CPUS_PER_TASK:-unavailable} date=$(date)"
echo "== lexer-stage Rust suite =="
/usr/bin/time -v cargo test --workspace --locked
rust_status="passed"

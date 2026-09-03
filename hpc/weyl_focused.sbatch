#!/usr/bin/env bash
# Focused Weyl transducer gate. This deliberately avoids the full-workspace
# format and Clippy gates because unrelated dirty files may block them.
#SBATCH --job-name=atlas-weyl
#SBATCH --partition=cpu
#SBATCH --account=yushilingroup
#SBATCH --qos=normal
#SBATCH --nodes=1
#SBATCH --ntasks=1
#SBATCH --cpus-per-task=2
#SBATCH --mem=4G
#SBATCH --time=00:20:00
#SBATCH --output=atlas-weyl-%j.out

set -euo pipefail
submit_dir="${SLURM_SUBMIT_DIR:?SLURM_SUBMIT_DIR is required}"
cd "$submit_dir"
export PATH="$HOME/.cargo/bin:$PATH"
echo "WEYL_FOCUSED_START job=${SLURM_JOB_ID:-unknown} submit_dir=$submit_dir host=$(hostname)"
export RUST_MIN_STACK=33554432

started_at="$(date --iso-8601=seconds)"
commit="${ATLAS_COMMIT:-$(git rev-parse HEAD 2>/dev/null || echo unversioned)}"
source_tree_state="${ATLAS_DIRTY_TREE:-unknown}"
report="${WEYL_REPORT:-$submit_dir/weyl_focused_${SLURM_JOB_ID}.json}"
log_dir="${WEYL_LOG_DIR:-$submit_dir/weyl_focused_${SLURM_JOB_ID}}"
mkdir -p "$log_dir"
status=not_started

write_report() {
  local final_status="$1"
  local final_exit="$2"
  local ended_at
  ended_at="$(date --iso-8601=seconds)"
  python3 - "$report" "$commit" "$source_tree_state" "$started_at" "$ended_at" "$final_status" "$final_exit" "$log_dir" <<'PY'
import json
import pathlib
import subprocess
import sys

report, commit, tree_state, started, ended, status, exit_status, log_dir = sys.argv[1:]
root = pathlib.Path(log_dir)
files = {path.name: str(path) for path in sorted(root.iterdir()) if path.is_file()}
payload = {
    "schema": "atlas-weyl-focused-v1",
    "stage": "weyl-transducer-focused-gate",
    "commit": commit,
    "source_tree_state": tree_state,
    "started_at": started,
    "ended_at": ended,
    "status": status,
    "exit_status": int(exit_status),
    "rustc": subprocess.check_output(["rustc", "--version"], text=True).strip(),
    "cargo": subprocess.check_output(["cargo", "--version"], text=True).strip(),
    "logs": files,
}
pathlib.Path(report).write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n")
PY
  echo "WEYL_FOCUSED_DONE report=$report status=$final_status"
}

on_exit() {
  local code=$?
  set +e
  if [[ "$code" -eq 0 ]]; then
    write_report passed 0
  else
    write_report "$status" "$code"
  fi
  exit "$code"
}
trap on_exit EXIT

run_timed() {
  local label="$1"
  shift
  status="$label"
  /usr/bin/time -v -o "$log_dir/${label}.time" "$@" \
    >"$log_dir/${label}.stdout" 2>"$log_dir/${label}.stderr"
}

run_timed check cargo check -p atlas-real-group
run_timed weyl_debug cargo test -p atlas-real-group --lib weyl
run_timed involution_debug cargo test -p atlas-real-group --lib involution_table
run_timed kgb_debug cargo test -p atlas-real-group --lib kgb_graph
run_timed weyl_release cargo test -p atlas-real-group --release --lib weyl
run_timed involution_release cargo test -p atlas-real-group --release --lib involution_table
run_timed kgb_release cargo test -p atlas-real-group --release --lib kgb_graph

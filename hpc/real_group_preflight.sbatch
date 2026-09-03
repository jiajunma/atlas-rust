#!/usr/bin/env bash
# Exercise the Rust structural layer on XMU HPC without claiming Atlas
# compatibility. A true differential job needs a language-level constructor
# fixture plus adapters for both the reference executable and Rust events.
#SBATCH --job-name=atlas-real-group
#SBATCH --partition=cpu
#SBATCH --account=yushilingroup
#SBATCH --qos=normal
#SBATCH --nodes=1
#SBATCH --ntasks=1
#SBATCH --cpus-per-task=2
#SBATCH --mem=4G
#SBATCH --time=00:20:00
#SBATCH --output=atlas-real-group-%j.out

set -euo pipefail
submit_dir="${SLURM_SUBMIT_DIR:?SLURM_SUBMIT_DIR is required}"
cd "$submit_dir"
export PATH="$HOME/.cargo/bin:$PATH"

if [[ -n "${ATLAS_COMMIT:-}" ]]; then
  commit="$ATLAS_COMMIT"
elif git rev-parse --verify HEAD >/dev/null 2>&1; then
  commit="$(git rev-parse HEAD)"
else
  commit="unversioned"
fi
job_id="${SLURM_JOB_ID:?SLURM_JOB_ID is required}"
run_dir="$submit_dir/results/$commit/$job_id"
snapshot_dir="$run_dir/source"
report="$run_dir/real_group_preflight.json"
rust_status="not_started"
source_snapshot_sha256="unavailable"
started_at="$(date --iso-8601=seconds)"

if [[ -n "${ATLAS_DIRTY_TREE:-}" ]]; then
  dirty_tree="$ATLAS_DIRTY_TREE"
elif git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  if [[ -z "$(git status --porcelain --untracked-files=all)" ]]; then
    dirty_tree=false
  else
    dirty_tree=true
  fi
else
  dirty_tree=unknown
fi

mkdir -p "$snapshot_dir"

snapshot_hash() {
  python3 - "$1" <<'PY'
import hashlib
import pathlib
import sys

root = pathlib.Path(sys.argv[1])
hasher = hashlib.sha256()
for path in sorted((item for item in root.rglob("*") if item.is_file()), key=lambda item: item.relative_to(root).as_posix()):
    relative = path.relative_to(root).as_posix()
    hasher.update(relative.encode() + b"\0")
    hasher.update(hashlib.sha256(path.read_bytes()).digest())
print(hasher.hexdigest())
PY
}

write_report() {
  local exit_status="$1"
  local ended_at
  ended_at="$(date --iso-8601=seconds)"
  python3 - "$report" "$commit" "$job_id" "$started_at" "$ended_at" \
    "$dirty_tree" "$rust_status" "$exit_status" \
    "${ATLAS_REFERENCE_REVISION:-not-provided}" "$source_snapshot_sha256" \
    "$snapshot_dir" <<'PY'
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
    source_snapshot_sha256,
    snapshot_dir,
) = sys.argv[1:]

snapshot_root = pathlib.Path(snapshot_dir)
fixture_root = snapshot_root / "tests/fixtures/domain"
fixtures = []
if fixture_root.is_dir():
    for fixture in sorted(fixture_root.glob("*.atlas")):
        fixtures.append(
            {
                "path": fixture.relative_to(snapshot_root).as_posix(),
                "sha256": hashlib.sha256(fixture.read_bytes()).hexdigest(),
                "role": "declared_future_differential_fixture",
                "executed": False,
            }
        )

def command_output(command):
    try:
        return subprocess.check_output(command, text=True).strip()
    except (OSError, subprocess.CalledProcessError):
        return "unavailable"

payload = {
    "schema": "atlas-real-group-preflight-v1",
    "stage": "real-group-rust-preflight",
    "compatibility_status": "not-claimed",
    "reason": "Atlas constructor syntax and oracle event adapters are pending",
    "commit": commit,
    "source_snapshot_sha256": source_snapshot_sha256,
    "source_snapshot_scope": "frozen submit directory excluding .git, target, results, and Slurm stdout",
    "dirty_tree": {"true": True, "false": False}.get(dirty_tree),
    "source_tree_state": dirty_tree,
    "rustc": command_output(["rustc", "--version"]),
    "cargo": command_output(["cargo", "--version"]),
    "reference_atlas_revision": reference_revision,
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

# Freeze the complete workspace before executing Cargo. The submitted Slurm
# script is copied from the actual job source so the snapshot cannot be changed
# by a later edit to the submit directory.
job_script="${BASH_SOURCE[0]}"
if [[ "$job_script" != /* ]]; then
  job_script="$submit_dir/$job_script"
fi
tar -C "$submit_dir" \
  --exclude=.git \
  --exclude=target \
  --exclude=results \
  --exclude='atlas-real-group-*.out' \
  -cf - . | tar -C "$snapshot_dir" -xf -
cp "$job_script" "$snapshot_dir/hpc/real_group_preflight.sbatch"
source_snapshot_sha256="$(snapshot_hash "$snapshot_dir")"
export CARGO_TARGET_DIR="$run_dir/target"
cd "$snapshot_dir"

echo "host=$(hostname) job=$job_id cpus=${SLURM_CPUS_PER_TASK:-unavailable} date=$(date)"
echo "== atlas-real-group formatting =="
rust_status="formatting"
cargo fmt -p atlas-real-group --check
echo "== atlas-real-group lint =="
rust_status="linting"
cargo clippy -p atlas-real-group --all-targets --locked -- -D warnings
echo "== atlas-real-group Rust unit suite =="
rust_status="testing"
/usr/bin/time -v cargo test -p atlas-real-group --lib --locked
rust_status="passed"

if [[ "$(snapshot_hash "$snapshot_dir")" != "$source_snapshot_sha256" ]]; then
  rust_status="snapshot_mutated"
  echo "ERROR: frozen source snapshot changed during preflight" >&2
  exit 1
fi

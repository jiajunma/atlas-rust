#!/usr/bin/env bash
# Compare the Rust typed pipeline with checked-in, HPC-frozen Atlas events.
# Uncovered upstream overloads remain explicit pending coverage in the report.
#SBATCH --job-name=atlas-pipeline-swap
#SBATCH --partition=cpu
#SBATCH --account=yushilingroup
#SBATCH --qos=normal
#SBATCH --nodes=1
#SBATCH --ntasks=1
#SBATCH --cpus-per-task=2
#SBATCH --mem=8G
#SBATCH --time=00:20:00
#SBATCH --output=atlas-pipeline-swap-%j.out

set -euo pipefail
submit_dir="${SLURM_SUBMIT_DIR:?SLURM_SUBMIT_DIR is required}"
cd "$submit_dir"
export PATH="$HOME/.cargo/bin:$PATH"

job_id="${SLURM_JOB_ID:?SLURM_JOB_ID is required}"
if [[ ! "$job_id" =~ ^[0-9]+$ ]]; then
  echo "SLURM_JOB_ID must contain only decimal digits" >&2
  exit 2
fi
slurm_stdout="atlas-pipeline-swap-${job_id}.out"

# This small bootstrap must not trust a mutable helper before we know whether
# the submit tree is clean. The checked helper is loaded only after binding it.
detect_submit_dirty_tree() {
  local stdout_path="${1:?Slurm stdout path is required}"

  if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    printf 'unknown\n'
    return 0
  fi

  local status
  status="$(git status --porcelain --untracked-files=all)" || return 2
  local line
  while IFS= read -r line; do
    if [[ -n "$line" && "$line" != "?? $stdout_path" ]]; then
      printf 'true\n'
      return 0
    fi
  done <<<"$status"
  printf 'false\n'
}

if git rev-parse --verify HEAD >/dev/null 2>&1; then
  detected_commit="$(git rev-parse HEAD)"
else
  detected_commit="unversioned"
fi
commit="${ATLAS_COMMIT:-$detected_commit}"
if [[ "$detected_commit" != "unversioned" && ! "$detected_commit" =~ ^[0-9a-fA-F]{40}$ ]]; then
  echo "detected Git commit is not a 40-digit hexadecimal token" >&2
  exit 2
fi
if [[ "$commit" != "unversioned" && ! "$commit" =~ ^[0-9a-fA-F]{40}$ ]]; then
  echo "ATLAS_COMMIT must be a 40-digit hexadecimal commit or 'unversioned'" >&2
  exit 2
fi
detected_dirty_tree="$(detect_submit_dirty_tree "$slurm_stdout")" || {
  echo "could not inspect submit checkout status" >&2
  exit 2
}
dirty_tree="${ATLAS_DIRTY_TREE:-$detected_dirty_tree}"
if [[ ! "$dirty_tree" =~ ^(true|false|unknown)$ ]]; then
  echo "ATLAS_DIRTY_TREE must be true, false, or unknown" >&2
  exit 2
fi

run_dir="$submit_dir/results/$commit/$job_id"
snapshot_dir="$run_dir/source"
artifact_dir="$run_dir/pipeline_swap"
mkdir -p "$snapshot_dir" "$artifact_dir"
report="$artifact_dir/pipeline_swap_diff_report.json"
source_snapshot_sha256="unavailable"
source_snapshot_provenance="not_created"
source_state_helper_provenance="not_loaded"
source_state_helper_blob_oid="unavailable"
failure_phase="initialization"

write_checksum() {
  local input="$1"
  local output="$2"
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$input" >"$output"
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$input" >"$output"
  else
    return 1
  fi
}

write_fallback_report() {
  local process_status="$1"
  python3 - "$report" "$commit" "$detected_commit" "$dirty_tree" \
    "$detected_dirty_tree" "$job_id" \
    "$source_snapshot_sha256" "$source_snapshot_provenance" \
    "$source_state_helper_provenance" "$source_state_helper_blob_oid" \
    "$failure_phase" "$process_status" <<'PY'
import json
import pathlib
import sys

(
    report_path,
    commit,
    detected_commit,
    dirty_tree,
    detected_dirty_tree,
    job_id,
    source_snapshot_sha256,
    source_snapshot_provenance,
    source_state_helper_provenance,
    source_state_helper_blob_oid,
    failure_phase,
    process_status,
) = sys.argv[1:]
report = {
    "schema": "atlas-pipeline-swap-diff-v1",
    "stage": "typed-pipeline-swap-rust-vs-frozen-atlas",
    "status": "FAIL",
    "runnable_status": "FAIL",
    "compatibility_claim": False,
    "commit": commit,
    "dirty_tree": {"true": True, "false": False}.get(dirty_tree, dirty_tree),
    "source_state": {
        "declared_commit": commit,
        "detected_commit": detected_commit,
        "declared_dirty_tree": {"true": True, "false": False}.get(dirty_tree, dirty_tree),
        "detected_dirty_tree": {"true": True, "false": False}.get(detected_dirty_tree, detected_dirty_tree),
        "verified": commit == detected_commit and dirty_tree == detected_dirty_tree,
        "helper": {
            "provenance": source_state_helper_provenance,
            "blob_oid": source_state_helper_blob_oid,
        },
    },
    "source_snapshot_sha256": source_snapshot_sha256,
    "source_snapshot_provenance": source_snapshot_provenance,
    "slurm": {"job_id": job_id},
    "failure_phase": failure_phase,
    "process_exit_status": int(process_status),
    "reason": "job failed before the differential harness produced a report",
}
path = pathlib.Path(report_path)
path.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n")
PY
}

on_exit() {
  local process_status=$?
  set +e
  if [[ $process_status -eq 0 && ! -f "$report" ]]; then
    process_status=1
    failure_phase="missing_report"
  fi
  if [[ $process_status -ne 0 && ! -f "$report" ]]; then
    write_fallback_report "$process_status" || \
      printf '{"schema":"atlas-pipeline-swap-diff-v1","status":"FAIL","compatibility_claim":false}\n' >"$report"
  fi
  if [[ -f "$report" ]]; then
    write_checksum "$report" "$report.sha256" || process_status=1
  fi
  trap - EXIT
  exit "$process_status"
}
trap on_exit EXIT

failure_phase="validating_declared_source_state"
if [[ "$commit" != "$detected_commit" || "$dirty_tree" != "$detected_dirty_tree" ]]; then
  echo "declared Atlas-Rust source state does not match the submit checkout" >&2
  exit 2
fi

failure_phase="validating_source_state_helper"
source_state_path="$submit_dir/hpc/source_state.sh"
if [[ ! -f "$source_state_path" ]]; then
  echo "missing source-state helper: $source_state_path" >&2
  exit 2
fi
if [[ "$detected_commit" != "unversioned" && "$detected_dirty_tree" == "false" ]]; then
  committed_source_state_oid="$(git rev-parse --verify "$detected_commit:hpc/source_state.sh" 2>/dev/null || true)"
  submitted_source_state_oid="$(git hash-object "$source_state_path" 2>/dev/null || true)"
  if [[ -z "$committed_source_state_oid" || \
        "$submitted_source_state_oid" != "$committed_source_state_oid" ]]; then
    echo "source-state helper differs from the detected commit" >&2
    exit 2
  fi
  source <(git show "$detected_commit:hpc/source_state.sh")
  source_state_helper_provenance="git_commit:$detected_commit"
  source_state_helper_blob_oid="$committed_source_state_oid"
else
  frozen_source_state_helper="$run_dir/source_state.sh"
  cp -- "$source_state_path" "$frozen_source_state_helper"
  submitted_source_state_oid="$(git hash-object "$source_state_path" 2>/dev/null || true)"
  frozen_source_state_oid="$(git hash-object "$frozen_source_state_helper" 2>/dev/null || true)"
  if [[ -z "$submitted_source_state_oid" || \
        "$submitted_source_state_oid" != "$frozen_source_state_oid" ]]; then
    echo "source-state helper changed while it was frozen" >&2
    exit 2
  fi
  source "$frozen_source_state_helper"
  source_state_helper_provenance="${detected_dirty_tree}_submit_tree"
  source_state_helper_blob_oid="$frozen_source_state_oid"
fi
helper_dirty_tree="$(atlas_detect_dirty_tree "$slurm_stdout")" || {
  echo "could not inspect source state through the frozen helper" >&2
  exit 2
}
if [[ "$helper_dirty_tree" != "$detected_dirty_tree" ]]; then
  echo "source-state helper disagrees with bootstrap dirty-tree detection" >&2
  exit 2
fi

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

failure_phase="snapshotting"
job_script="${BASH_SOURCE[0]}"
if [[ "$job_script" != /* ]]; then
  job_script="$submit_dir/$job_script"
fi
if [[ "$detected_dirty_tree" == "false" && "$detected_commit" != "unversioned" ]]; then
  source_snapshot_provenance="git_archive:$detected_commit"
  git archive --format=tar "$detected_commit" | tar -C "$snapshot_dir" -xf -
else
  source_snapshot_provenance="${detected_dirty_tree}_submit_tree"
  tar -C "$submit_dir" \
    --exclude=.git \
    --exclude=target \
    --exclude=results \
    --exclude='__pycache__' \
    --exclude='*.pyc' \
    --exclude='atlas-pipeline-swap-*.out' \
    -cf - . | tar -C "$snapshot_dir" -xf -
fi
snapshot_job_script="$snapshot_dir/hpc/pipeline_swap_diff.sbatch"
failure_phase="validating_submitted_job_script"
submitted_job_script_oid="$(git hash-object "$job_script" 2>/dev/null || true)"
snapshot_job_script_oid="$(git hash-object "$snapshot_job_script" 2>/dev/null || true)"
if [[ -z "$submitted_job_script_oid" || \
      "$submitted_job_script_oid" != "$snapshot_job_script_oid" ]]; then
  echo "submitted pipeline-swap script differs from the frozen checkout" >&2
  exit 2
fi
if [[ "$detected_commit" != "unversioned" && "$detected_dirty_tree" == "false" ]]; then
  committed_job_script_oid="$(git rev-parse --verify "$detected_commit:hpc/pipeline_swap_diff.sbatch" 2>/dev/null || true)"
  if [[ -z "$committed_job_script_oid" || \
        "$submitted_job_script_oid" != "$committed_job_script_oid" ]]; then
    echo "submitted pipeline-swap script differs from the declared commit" >&2
    exit 2
  fi
fi
snapshot_source_state_oid="$(git hash-object "$snapshot_dir/hpc/source_state.sh" 2>/dev/null || true)"
if [[ -z "$snapshot_source_state_oid" || \
      "$snapshot_source_state_oid" != "$source_state_helper_blob_oid" ]]; then
  echo "frozen source-state helper differs from the helper used for source detection" >&2
  exit 2
fi
source_snapshot_sha256="$(snapshot_hash "$snapshot_dir")"

failure_phase="verifying_submit_source_state"
post_snapshot_commit="$(git rev-parse HEAD 2>/dev/null || printf 'unversioned')"
post_snapshot_dirty_tree="$(atlas_detect_dirty_tree "$slurm_stdout")" || {
  echo "could not re-check submit checkout status after snapshot" >&2
  exit 2
}
if [[ "$post_snapshot_commit" != "$detected_commit" || \
      "$post_snapshot_dirty_tree" != "$detected_dirty_tree" ]]; then
  echo "submit checkout changed while the frozen source snapshot was created" >&2
  exit 2
fi

annotate_report_provenance() {
  local snapshot_scope
  if [[ "$source_snapshot_provenance" == git_archive:* ]]; then
    snapshot_scope="full tracked Git archive"
  else
    snapshot_scope="live submit-tree snapshot (generated paths excluded)"
  fi
  python3 - "$report" "$source_snapshot_provenance" \
    "$source_state_helper_provenance" "$source_state_helper_blob_oid" \
    "$snapshot_scope" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
report = json.loads(path.read_text())
report["source_snapshot_provenance"] = sys.argv[2]
report["source_snapshot_scope"] = sys.argv[5]
report.setdefault("source_state", {})["helper"] = {
    "provenance": sys.argv[3],
    "blob_oid": sys.argv[4],
}
path.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n")
PY
}

export CARGO_TARGET_DIR="$run_dir/target"
workspace_dir="$run_dir/workspace"
mkdir -p "$workspace_dir"
cd "$snapshot_dir"
echo "host=$(hostname) job=$job_id cpus=${SLURM_CPUS_PER_TASK:-unavailable} date=$(date)"
echo "== build frozen atlas-cli =="
failure_phase="building_atlas_cli"
cargo build --release -p atlas-cli --locked
cli_bin="$CARGO_TARGET_DIR/release/atlas-cli"

failure_phase="running_differential"
set +e
python3 hpc/pipeline_swap_diff.py \
  "$cli_bin" "$artifact_dir" \
  --workspace-root "$workspace_dir" \
  --fixture-root "$snapshot_dir/tests/fixtures" \
  --reference-root "$snapshot_dir/tests/reference" \
  --commit "$commit" \
  --dirty-tree "$dirty_tree" \
  --detected-commit "$detected_commit" \
  --detected-dirty-tree "$detected_dirty_tree" \
  --job-id "$job_id" \
  --source-snapshot-sha256 "$source_snapshot_sha256" \
  --expected-reference-revision "${ATLAS_REFERENCE_REVISION:-4d3e9449062a07c1c85f4e6df215eb6ccc0eeae9}" \
  --timeout "${TIMEOUT:-30}"
job_status=$?
set -e

failure_phase="verifying_snapshot"
if [[ "$(snapshot_hash "$snapshot_dir")" != "$source_snapshot_sha256" ]]; then
  echo "frozen pipeline-swap snapshot changed during execution" >&2
  python3 - "$report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
if path.is_file():
    report = json.loads(path.read_text())
    report["status"] = "FAIL"
    report["runnable_status"] = "FAIL"
    report["compatibility_claim"] = False
    report["source_snapshot_mutated"] = True
    path.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n")
PY
  job_status=1
fi
test -f "$report"
annotate_report_provenance
failure_phase="complete"
exit "$job_status"

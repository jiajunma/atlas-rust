#!/usr/bin/env bash
# Authoritative upstream Atlas capture for the typed scalar overload fixtures.
# This job produces oracle artifacts only; it does not make a Rust
# compatibility claim until a later differential job consumes the report.
#SBATCH --job-name=atlas-scalar-reference
#SBATCH --partition=cpu
#SBATCH --account=yushilingroup
#SBATCH --qos=normal
#SBATCH --nodes=1
#SBATCH --ntasks=1
#SBATCH --cpus-per-task=1
#SBATCH --mem=2G
#SBATCH --time=00:10:00
#SBATCH --output=atlas-scalar-reference-%j.out

# The upstream binary was built with a newer gcc than the node baseline.
source /etc/profile >/dev/null 2>&1 || true
module load misc/gcc/12.1 >/dev/null 2>&1 || true
gcc_lib="$(dirname "$(gcc -print-file-name=libstdc++.so.6)")"
export LD_LIBRARY_PATH="$gcc_lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"

set -euo pipefail
submit_dir="${SLURM_SUBMIT_DIR:?SLURM_SUBMIT_DIR is required}"
cd "$submit_dir"
export PATH="$HOME/.cargo/bin:$PATH"

atlas_bin="${ATLAS_BIN:-$HOME/atlasofliegroups/atlas}"
test -x "$atlas_bin" || { echo "missing atlas binary at $atlas_bin" >&2; exit 2; }

if [[ -n "${ATLAS_COMMIT:-}" ]]; then
  commit="$ATLAS_COMMIT"
elif git rev-parse --verify HEAD >/dev/null 2>&1; then
  commit="$(git rev-parse HEAD)"
else
  commit="unversioned"
fi
if [[ -n "${ATLAS_DIRTY_TREE:-}" ]]; then
  dirty_tree="$ATLAS_DIRTY_TREE"
elif git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  [[ -z "$(git status --porcelain --untracked-files=all)" ]] && dirty_tree=false || dirty_tree=true
else
  dirty_tree=unknown
fi

job_id="${SLURM_JOB_ID:?SLURM_JOB_ID is required}"
run_dir="$submit_dir/results/$commit/$job_id"
snapshot_dir="$run_dir/source"
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

# Execute from a frozen copy so later synchronization cannot change the
# harness, fixtures, or expectations after submission.
job_script="${BASH_SOURCE[0]}"
if [[ "$job_script" != /* ]]; then
  job_script="$submit_dir/$job_script"
fi
tar -C "$submit_dir" --exclude='hpc/__pycache__' --exclude='*.pyc' \
  -cf - hpc tests/fixtures/eval tests/reference/eval \
  | tar -C "$snapshot_dir" -xf -
cp "$job_script" "$snapshot_dir/hpc/scalar_reference.sbatch"
source_snapshot_sha256="$(snapshot_hash "$snapshot_dir")"
cd "$snapshot_dir"

fixtures=(
  tests/fixtures/eval/scalar_overloads.atlas
  tests/fixtures/eval/scalar_error_*.atlas
)
for fixture in "${fixtures[@]}"; do
  test -f "$fixture" || { echo "missing scalar fixture: $fixture" >&2; exit 2; }
done

set +e
python3 hpc/scalar_reference.py \
  "$atlas_bin" "$run_dir" "${fixtures[@]}" \
  --reference-root tests/reference/eval \
  --commit "$commit" \
  --dirty-tree "$dirty_tree" \
  --job-id "$job_id" \
  --reference-revision "${ATLAS_REFERENCE_REVISION:-not-provided}" \
  --cweb-version "${ATLAS_CWEB_VERSION:-not-provided}" \
  --source-snapshot-sha256 "$source_snapshot_sha256" \
  --timeout "${TIMEOUT:-30}"
job_status=$?
set -e

if [[ "$(snapshot_hash "$snapshot_dir")" != "$source_snapshot_sha256" ]]; then
  echo "frozen scalar reference snapshot changed during execution" >&2
  python3 - "$run_dir/scalar_reference_report.json" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
if path.is_file():
    report = json.loads(path.read_text())
    report["status"] = "FAIL"
    report["source_snapshot_mutated"] = True
    path.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n")
PY
  job_status=1
fi
sha256sum "$run_dir/scalar_reference_report.json" >"$run_dir/scalar_reference_report.json.sha256"
exit "$job_status"

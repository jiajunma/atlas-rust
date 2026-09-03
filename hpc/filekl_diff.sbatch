#!/usr/bin/env bash
# Differential check for the KL binary file formats (filekl.w): the Rust
# writer in atlas-real-group dumps block/matrix/KL-store files for a fixed set
# of small blocks; the upstream KLread stand-alone tool reads them back and
# its polynomials are compared with Rust-side expectations.
#SBATCH --job-name=atlas-filekl-diff
#SBATCH --partition=cpu
#SBATCH --account=yushilingroup
#SBATCH --qos=normal
#SBATCH --nodes=1
#SBATCH --ntasks=1
#SBATCH --cpus-per-task=2
#SBATCH --mem=4G
#SBATCH --time=00:30:00
#SBATCH --output=atlas-filekl-diff-%j.out

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
dump_dir="$run_dir/dump"
report="$run_dir/filekl_report.json"
mkdir -p "$dump_dir"

echo "== filekl differential: commit=$commit job=$job_id"

echo "== building atlas-real-group (release)"
cargo build --release -p atlas-real-group

echo "== dumping KL binary files via filekl_dump test"
FILEKL_DUMP_DIR="$dump_dir" \
  cargo test --release -p atlas-real-group filekl_dump -- --ignored --nocapture

ls -la "$dump_dir"

echo "== comparing against upstream KLread oracle"
python3 hpc/filekl_diff.py "$dump_dir" "$report" "${MAX_PAIRS:-200}"

echo "== report written to $report"

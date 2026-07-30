#!/usr/bin/env bash
# KGB differential: the upstream C++ atlas interpreter vs the Rust port,
# comparing per-real-form KGB sizes and length multisets in the EXTERNAL
# numbering, with wall-clock timings for both sides.
#SBATCH --job-name=atlas-kgb-diff
#SBATCH --partition=cpu
#SBATCH --account=yushilingroup
#SBATCH --qos=normal
#SBATCH --nodes=1
#SBATCH --ntasks=1
#SBATCH --cpus-per-task=4
#SBATCH --mem=16G
#SBATCH --time=01:00:00
#SBATCH --output=atlas-kgb-diff-%j.out

# The upstream binary was built with a newer gcc than the node baseline;
# profile/module setup runs before strict mode (set -u breaks profile.d).
source /etc/profile >/dev/null 2>&1 || true
module load misc/gcc/12.1 >/dev/null 2>&1 || true
gcc_lib="$(dirname "$(gcc -print-file-name=libstdc++.so.6)")"
export LD_LIBRARY_PATH="$gcc_lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"

set -euo pipefail
submit_dir="${SLURM_SUBMIT_DIR:?SLURM_SUBMIT_DIR is required}"
cd "$submit_dir"
export PATH="$HOME/.cargo/bin:$PATH"

atlas_bin="${ATLAS_BIN:-$HOME/atlasofliegroups-4d3e9449/atlas}"
test -x "$atlas_bin" || { echo "missing atlas binary at $atlas_bin"; exit 2; }

commit="${ATLAS_COMMIT:-$(git rev-parse HEAD 2>/dev/null || echo unversioned)}"
job_id="${SLURM_JOB_ID:?SLURM_JOB_ID is required}"
run_dir="$submit_dir/results/$commit/$job_id"
mkdir -p "$run_dir"

echo "building kgb_bench (release)..."
cargo build --release --example kgb_bench -p atlas-real-group
bench_bin="$submit_dir/target/release/examples/kgb_bench"

export REPORT="$run_dir/kgb_differential_report.json"
python3 hpc/kgb_differential.py "$atlas_bin" "$bench_bin" "$@" \
  | tee "$run_dir/kgb_differential.log"

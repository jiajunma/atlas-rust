#!/usr/bin/env bash
# Corpus differential: every upstream atlas-scripts/*.at file through the
# upstream C++ interpreter and the Rust atlas-cli, scored per file with a
# first-error histogram (the corpus-driven language priority list).
#SBATCH --job-name=atlas-corpus-diff
#SBATCH --partition=cpu
#SBATCH --account=yushilingroup
#SBATCH --qos=normal
#SBATCH --nodes=1
#SBATCH --ntasks=1
#SBATCH --cpus-per-task=4
#SBATCH --mem=4G
#SBATCH --time=04:00:00
#SBATCH --output=atlas-corpus-diff-%j.out

# The upstream binary was built with a newer gcc than the node baseline;
# profile/module setup runs before strict mode (set -u breaks profile.d).
source /etc/profile >/dev/null 2>&1 || true
# Bind the same GCC runtime used by the reference-capture job.  A compute
# node may silently retain the system GCC 8 runtime even when module loading
# reports success, which makes every oracle script look like CPP_FAIL.
gcc_root="${ATLAS_GCC_ROOT:-/public/software/gcc/gcc-12.1}"
gcc_bin="$gcc_root/bin/gcc"
if [[ ! -x "$gcc_bin" ]]; then
  echo "required GCC executable is absent or not executable: $gcc_bin" >&2
  exit 2
fi
gcc_lib="$(dirname "$($gcc_bin -print-file-name=libstdc++.so.6)")"
if ! strings "$gcc_lib/libstdc++.so.6" | grep -q '^GLIBCXX_3\.4\.29$'; then
  echo "required GLIBCXX_3.4.29 is absent from $gcc_lib/libstdc++.so.6" >&2
  exit 2
fi
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

echo "building atlas-cli (release)..."
cargo build --release -p atlas-cli
cli_bin="$submit_dir/target/release/atlas-cli"

export REPORT="$run_dir/script_corpus_report.json"
export TIMEOUT="${TIMEOUT:-60}"
python3 hpc/script_corpus_diff.py "$atlas_bin" "$cli_bin" "$@" \
  | tee "$run_dir/script_corpus.log"

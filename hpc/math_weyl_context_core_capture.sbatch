#!/bin/bash -p
#SBATCH --job-name=atlas-weyl-g2-v1
#SBATCH --partition=cpu
#SBATCH --account=yushilingroup
#SBATCH --qos=normal
#SBATCH --nodes=1
#SBATCH --ntasks=1
#SBATCH --cpus-per-task=2
#SBATCH --mem=8G
#SBATCH --time=02:00:00
#SBATCH --signal=B:TERM@300
#SBATCH --output=weyl-context-g2-v1-%j.out
set -euo pipefail
cd "${SLURM_SUBMIT_DIR:?}"
export PATH="/public/software/gcc/gcc-12.1/bin:${HOME:?}/.cargo/bin:${PATH:?}"
weyl_core_gcc_runtime="$(g++ -print-file-name=libstdc++.so.6)"
[[ "$weyl_core_gcc_runtime" = /* ]] || {
  echo "g++ did not resolve an absolute libstdc++ runtime path" >&2
  exit 1
}
weyl_core_gcc_lib="$(dirname -- "$weyl_core_gcc_runtime")"
export LD_LIBRARY_PATH="$weyl_core_gcc_lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
# These bytes have no standalone launch contract.  Only the reviewed campaign
# stager may pin and submit them through the shared one-job ledger boundary.
export PYTHONDONTWRITEBYTECODE=1
export WEYL_CONTEXT_CORE_CAPTURE_SPOOL="${BASH_SOURCE[0]}"
exec /public/software/anaconda/anaconda3-2022.5/bin/python3.9 \
  -E -s -S -B hpc/math_weyl_context_core_capture.py

#!/usr/bin/env bash
#SBATCH --job-name=atlas-language-replay
#SBATCH --partition=cpu
#SBATCH --account=yushilingroup
#SBATCH --qos=normal
#SBATCH --nodes=1
#SBATCH --ntasks=1
#SBATCH --cpus-per-task=1
#SBATCH --mem=2G
#SBATCH --time=00:10:00
#SBATCH --output=language-replay-%j.out
set -euo pipefail
cd "${SLURM_SUBMIT_DIR:?}"
export PATH="/public/software/gcc/gcc-12.1/bin:$HOME/.cargo/bin:$PATH"
atlas_gcc_lib="$(dirname "$(g++ -print-file-name=libstdc++.so.6)")"
export LD_LIBRARY_PATH="$atlas_gcc_lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
export PYTHONDONTWRITEBYTECODE=1 BRIDGE_SPOOL="${BASH_SOURCE[0]}"
python3 hpc/math_language_bridge.py

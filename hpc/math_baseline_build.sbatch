#!/usr/bin/env bash
#SBATCH --job-name=atlas-math-baseline
#SBATCH --partition=cpu
#SBATCH --account=yushilingroup
#SBATCH --qos=normal
#SBATCH --nodes=1
#SBATCH --ntasks=1
#SBATCH --cpus-per-task=2
#SBATCH --mem=8G
#SBATCH --time=00:55:00
#SBATCH --output=math-baseline-%j.out
set -euo pipefail
cd "${SLURM_SUBMIT_DIR:?}"
export PATH="/public/software/gcc/gcc-12.1/bin:$HOME/.cargo/bin:$PATH"
gcc_lib="$(dirname "$(g++ -print-file-name=libstdc++.so.6)")"
export LD_LIBRARY_PATH="$gcc_lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
export PYTHONDONTWRITEBYTECODE=1 MATH_BUILD_SPOOL="${BASH_SOURCE[0]}"
python3 hpc/math_baseline_build.py

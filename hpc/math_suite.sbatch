#!/usr/bin/env bash
#SBATCH --job-name=atlas-math-survey
#SBATCH --partition=cpu
#SBATCH --account=yushilingroup
#SBATCH --qos=normal
#SBATCH --nodes=1
#SBATCH --ntasks=1
#SBATCH --cpus-per-task=2
#SBATCH --mem=8G
#SBATCH --time=00:25:00
#SBATCH --output=math-survey-%A_%a.out
set -euo pipefail
cd "${SLURM_SUBMIT_DIR:?}"
export PATH="/public/software/gcc/gcc-12.1/bin:$PATH"
gcc_lib="$(dirname "$(g++ -print-file-name=libstdc++.so.6)")"
export LD_LIBRARY_PATH="$gcc_lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
export PYTHONDONTWRITEBYTECODE=1 MATH_SUITE_SPOOL="${BASH_SOURCE[0]}"
python3 -m unittest discover -s hpc -p test_math_suite.py
python3 hpc/math_suite.py --case "${SLURM_ARRAY_TASK_ID:?}" --build "${MATH_BUILD_REPORT:?}" \
  --timeout "${MATH_CASE_TIMEOUT:-300}"

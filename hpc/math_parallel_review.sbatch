#!/usr/bin/env bash
#SBATCH --job-name=atlas-parallel-review
#SBATCH --partition=cpu
#SBATCH --account=yushilingroup
#SBATCH --qos=normal
#SBATCH --nodes=1
#SBATCH --ntasks=1
#SBATCH --cpus-per-task=1
#SBATCH --mem=2G
#SBATCH --time=00:10:00
#SBATCH --output=parallel-review-%j.out
set -euo pipefail
cd "${SLURM_SUBMIT_DIR:?}"
export PYTHONDONTWRITEBYTECODE=1 PARALLEL_REVIEW_SPOOL="${BASH_SOURCE[0]}"
python3 hpc/math_parallel_review.py

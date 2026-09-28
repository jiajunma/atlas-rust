#!/usr/bin/env bash
#SBATCH --job-name=atlas-kl-history-diagnose
#SBATCH --partition=cpu
#SBATCH --account=yushilingroup
#SBATCH --qos=normal
#SBATCH --nodes=1
#SBATCH --ntasks=1
#SBATCH --cpus-per-task=1
#SBATCH --mem=2G
#SBATCH --time=00:05:00
#SBATCH --output=history-diagnose-%j.out
set -euo pipefail
cd "${SLURM_SUBMIT_DIR:?}"
export PYTHONDONTWRITEBYTECODE=1 MATH_DIAGNOSE_SPOOL="${BASH_SOURCE[0]}"
python3 -m unittest discover -s hpc -p test_math_history_diagnose.py -v
python3 hpc/math_kl_history_diagnose.py

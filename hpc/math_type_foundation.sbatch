#!/usr/bin/env bash
#SBATCH --job-name=atlas-type-foundation
#SBATCH --partition=cpu
#SBATCH --account=yushilingroup
#SBATCH --qos=normal
#SBATCH --nodes=1
#SBATCH --ntasks=1
#SBATCH --cpus-per-task=2
#SBATCH --mem=8G
#SBATCH --time=00:35:00
#SBATCH --output=type-foundation-%j.out
set -euo pipefail
cd "${SLURM_SUBMIT_DIR:?}"
export PATH="/public/software/gcc/gcc-12.1/bin:$HOME/.cargo/bin:$PATH"
export PYTHONDONTWRITEBYTECODE=1 TYPE_SPOOL="${BASH_SOURCE[0]}"
python3 hpc/math_type_foundation.py

#!/bin/bash -p
#SBATCH --job-name=atlas-ladder-index-v1
#SBATCH --partition=cpu
#SBATCH --account=yushilingroup
#SBATCH --qos=normal
#SBATCH --nodes=1
#SBATCH --ntasks=1
#SBATCH --cpus-per-task=1
#SBATCH --mem=2G
#SBATCH --time=00:15:00
#SBATCH --signal=B:TERM@60
#SBATCH --output=ladder-boundary-index-v1-%j.out
set -euo pipefail
cd "${SLURM_SUBMIT_DIR:?}"
# This spool has no standalone launch contract. The reviewed index stager must
# pin and submit these exact bytes through the one-job submit_one boundary.
export LADDER_BOUNDARY_INDEX_SPOOL="${BASH_SOURCE[0]}"
exec /public/software/anaconda/anaconda3-2022.5/bin/python3.9 \
  -E -s -S -B hpc/math_ladder_boundary_index.py

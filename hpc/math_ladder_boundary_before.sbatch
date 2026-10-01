#!/usr/bin/env bash
#SBATCH --job-name=atlas-ladder-boundary-before-v3
#SBATCH --partition=cpu
#SBATCH --account=yushilingroup
#SBATCH --qos=normal
#SBATCH --nodes=1
#SBATCH --ntasks=1
#SBATCH --cpus-per-task=2
#SBATCH --mem=8G
#SBATCH --time=02:00:00
#SBATCH --signal=B:TERM@300
#SBATCH --output=ladder-boundary-before-v3-%j.out
set -euo pipefail
cd "${SLURM_SUBMIT_DIR:?}"
export PATH="/public/software/gcc/gcc-12.1/bin:$HOME/.cargo/bin:$PATH"
ladder_gcc_lib="$(dirname "$(g++ -print-file-name=libstdc++.so.6)")"
export LD_LIBRARY_PATH="$ladder_gcc_lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
# This script has no standalone launch contract. Only the reviewed v3-attempt,
# v2-protocol stager may submit it against the accepted parent seal and ledger.
export PYTHONDONTWRITEBYTECODE=1 LADDER_BOUNDARY_BEFORE_SPOOL="${BASH_SOURCE[0]}"
exec python3 hpc/math_ladder_boundary_before.py

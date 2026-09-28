#!/usr/bin/env bash
#SBATCH --job-name=atlas-math-review
#SBATCH --partition=cpu
#SBATCH --account=yushilingroup
#SBATCH --qos=normal
#SBATCH --nodes=1
#SBATCH --ntasks=1
#SBATCH --cpus-per-task=1
#SBATCH --mem=2G
#SBATCH --time=00:10:00
#SBATCH --output=math-review-%j.out
set -euo pipefail
cd "${SLURM_SUBMIT_DIR:?}"
export PYTHONDONTWRITEBYTECODE=1
test "$(sha256sum review-code.sha256 | cut -d' ' -f1)" = "${REVIEW_CODE_SHA256:?}"
sha256sum --check review-code.sha256
test "$(sha256sum "${BASH_SOURCE[0]}" | cut -d' ' -f1)" = "${REVIEW_SCRIPT_SHA256:?}"
python3 -m unittest discover -s hpc -p 'test_math*.py' -v
python3 hpc/math_suite_review.py --stage "${REVIEW_STAGE:?}" \
  --inputs-sha "${REVIEW_INPUTS_SHA256:?}" --build "${MATH_BUILD_REPORT:?}" \
  --build-sha "${MATH_BUILD_SHA256:?}" --indices "${REVIEW_INDICES:?}" \
  --output "results/${SLURM_JOB_ID}/review.json"

#!/usr/bin/env bash
# Fast compile check of the pushed HEAD in a throwaway worktree, so it can
# run alongside a corpus job that owns the main checkout's target dir.
#SBATCH --job-name=atlas-check
#SBATCH --partition=cpu
#SBATCH --account=yushilingroup
#SBATCH --qos=normal
#SBATCH --nodes=1
#SBATCH --ntasks=1
#SBATCH --cpus-per-task=4
#SBATCH --mem=8G
#SBATCH --time=00:20:00
#SBATCH --output=atlas-check-%j.out

set -euo pipefail
submit_dir="${SLURM_SUBMIT_DIR:?SLURM_SUBMIT_DIR is required}"
cd "$submit_dir"
export PATH="$HOME/.cargo/bin:$PATH"

check_dir="$submit_dir-check"
git worktree remove --force "$check_dir" 2>/dev/null || true
git worktree prune
rm -rf "$check_dir"
git worktree add "$check_dir" HEAD
cd "$check_dir"
export CARGO_TARGET_DIR="$check_dir/target"
cargo check --workspace --all-targets 2>&1 | grep -E '^(error|warning: unused)' -A 12 || true
check_status=${PIPESTATUS[0]}
echo "CHECK_DONE status=$check_status"
if [[ "$check_status" -eq 0 ]]; then
  cargo test --workspace 2>&1 | grep -E '^(error|test result|failures:|---- .* stdout)' -A 6 || true
  echo "TEST_DONE status=${PIPESTATUS[0]}"
fi
cd "$submit_dir"
git worktree remove --force "$check_dir"

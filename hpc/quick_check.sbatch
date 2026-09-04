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

check_dir="$submit_dir-check-${SLURM_JOB_ID}"
# Per-job worktree: a concurrent quick_check must never remove this one
# (job 3621023's cleanup deleted 3621032's checkout mid-test).
git worktree prune
rm -rf "$check_dir"
git worktree add "$check_dir" HEAD
cd "$check_dir"
export CARGO_TARGET_DIR="$check_dir/target"
cargo check --workspace --all-targets 2>&1 | grep -E '^(error|warning: unused)' -A 12 || true
check_status=${PIPESTATUS[0]}
echo "CHECK_DONE status=$check_status"
if [[ "$check_status" -eq 0 ]]; then
  test_log="$submit_dir/atlas-test-${SLURM_JOB_ID}.log"
  # The evaluator recurses deeper than the 2MB default test-thread stack
  # (typed::tests::let_function_sugar_and_recursion_evaluate SIGABRTs
  # without this; with 32M it passes in 0.01s — depth, not runaway).
  RUST_MIN_STACK=33554432 cargo test --workspace > "$test_log" 2>&1 && test_status=0 || test_status=$?
  grep -E '^(test result|error|failures:)' "$test_log" | tail -25 || true
  echo "TEST_DONE status=$test_status log=$test_log"
fi
cd "$submit_dir"
# A concurrent quick-check may prune this worktree's metadata while the test
# process is still running. In that case the directory is no longer a known
# worktree, but it is still safe to remove this job-owned checkout directly.
if git worktree list --porcelain | grep -Fqx "worktree $check_dir"; then
  git worktree remove --force "$check_dir"
else
  rm -rf "$check_dir"
fi

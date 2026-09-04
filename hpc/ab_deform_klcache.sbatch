#!/bin/bash
#SBATCH --job-name=ab-deform-klcache
#SBATCH --output=ab-deform-klcache-%j.out
#SBATCH --partition=cpu
#SBATCH --account=yushilingroup
#SBATCH --qos=normal
#SBATCH --nodes=1
#SBATCH --ntasks=1
#SBATCH --cpus-per-task=4
#SBATCH --mem=4012M
#SBATCH --time=00:30:00
# A/B wall-time comparison for 83a78bf (deform KL table routed through
# the per-block record cache): BASE commit (pre-cache) vs the checkout's
# release binary, alternating order, REPS each, on a deform-heavy probe.
# Output correctness is irrelevant here (probe_deform_e7_only prints
# deform(p), identical across the two commits); wall time is the gate.
# Env: BASE (default b7ed9e5), PROBE, REPS (default 3).
set -euo pipefail
submit_dir="${SLURM_SUBMIT_DIR:?SLURM_SUBMIT_DIR is required}"
cd "$submit_dir"
export PATH="$HOME/.cargo/bin:$PATH"
base="${BASE:-b7ed9e5}"
probe="${PROBE:-hpc/workloads/probe_deform_e7_only.atlas}"
[[ "$probe" = /* ]] || probe="$submit_dir/$probe"
reps="${REPS:-3}"

# Build the base commit in a throwaway per-job worktree.
base_dir="$submit_dir-ab-base-${SLURM_JOB_ID}"
git worktree prune
rm -rf "$base_dir"
git worktree add "$base_dir" "$base"
( cd "$base_dir" && CARGO_TARGET_DIR="$base_dir/target" cargo build --release -p atlas-cli 2>&1 | tail -1 )
base_bin="$base_dir/target/release/atlas-cli"
tip_bin="$submit_dir/target/release/atlas-cli"

cd /public/home/majj/atlasofliegroups-4d3e9449/atlas-scripts
run_one() { # label, binary, rep
  local tf="$submit_dir/ab-${1}-${3}.time"
  /usr/bin/time -v "$2" < "$probe" > /dev/null 2> "$tf" || true
  echo "AB $1 rep$3 wall=$(grep 'wall clock' "$tf" | awk '{print $NF}') rss_kb=$(grep 'Maximum resident' "$tf" | awk '{print $NF}')"
}
for rep in $(seq 1 "$reps"); do
  # alternate leg order each rep (lane-C phantom-regression guard)
  if (( rep % 2 == 1 )); then
    run_one base "$base_bin" "$rep"; run_one tip "$tip_bin" "$rep"
  else
    run_one tip "$tip_bin" "$rep"; run_one base "$base_bin" "$rep"
  fi
done
cd "$submit_dir"
git worktree remove --force "$base_dir" 2>/dev/null || rm -rf "$base_dir"
echo "AB_DONE"

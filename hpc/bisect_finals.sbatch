#!/bin/bash
#SBATCH --job-name=bisect-finals
#SBATCH --output=bisect-finals-%j.out
#SBATCH --partition=cpu
#SBATCH --account=yushilingroup
#SBATCH --qos=normal
#SBATCH --nodes=1
#SBATCH --ntasks=1
#SBATCH --cpus-per-task=8
#SBATCH --mem=16G
#SBATCH --time=01:30:00
# Bisect the "common deformation finals" regression in
# probe_unitary_e7_heavy across three commits: c09e5b3 (pre-klfill),
# 6fd186c (pre-dynkin), 0835205 (pre-root-sum session baseline).
# Builds each in a throwaway worktree; does NOT touch the channel
# checkout or its target/release.
set -uo pipefail
submit_dir="${SLURM_SUBMIT_DIR:?SLURM_SUBMIT_DIR is required}"
cd "$submit_dir"
export PATH="$HOME/.cargo/bin:$PATH"
probe="$submit_dir/hpc/workloads/probe_unitary_e7_heavy.atlas"
for commit in c09e5b3 6fd186c 0835205; do
  wt="$submit_dir-bisect-$commit-${SLURM_JOB_ID}"
  rm -rf "$wt"
  git worktree add --force "$wt" "$commit" >/dev/null 2>&1
  ( cd "$wt" && CARGO_TARGET_DIR="$wt/target" cargo build --release -p atlas-cli 2>&1 | tail -1 )
  bin="$wt/target/release/atlas-cli"
  ( cd /public/home/majj/atlasofliegroups-4d3e9449/atlas-scripts && /usr/bin/time -v "$bin" < "$probe" > "$submit_dir/bisect-$commit.out" 2> "$submit_dir/bisect-$commit.time" )
  st=$?
  wall=$(grep 'wall clock' "$submit_dir/bisect-$commit.time" 2>/dev/null | awk '{print $NF}')
  errs=$(grep -c 'Runtime error' "$submit_dir/bisect-$commit.time" 2>/dev/null)
  echo "BISECT $commit exit=$st wall=$wall runtime_errors=$errs"
  git worktree remove --force "$wt" 2>/dev/null || rm -rf "$wt"
done
echo BISECT_DONE

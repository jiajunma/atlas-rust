#!/bin/bash
#SBATCH --job-name=ab-avann
#SBATCH --output=ab-avann-%j.out
#SBATCH --partition=cpu
#SBATCH --nodes=1
#SBATCH --ntasks=1
#SBATCH --cpus-per-task=2
#SBATCH --mem=8G
#SBATCH --time=00:30:00
# Interleaved A/B of probe_av_ann_e7 across the last three slice frontier
# commits (pre-fastgj, fastgj, elimops merge) to tell a real regression
# from machine drift: av_ann moved 15.44 (swar gate) -> 16.29 (fastgj
# gate) -> 16.50 (elimops gate) while its oracle stayed 14.1-14.5.
# Builds each in a throwaway worktree; does NOT touch the channel
# checkout or its target/release. Alternating order, REPS each.
set -uo pipefail
submit_dir="${SLURM_SUBMIT_DIR:?SLURM_SUBMIT_DIR is required}"
cd "$submit_dir"
export PATH="$HOME/.cargo/bin:$PATH"
probe="$submit_dir/hpc/workloads/probe_av_ann_e7.atlas"
reps="${REPS:-3}"
commits="a869688 2ce2d15 76cc35e"
for commit in $commits; do
  wt="$submit_dir-ab-avann-$commit-${SLURM_JOB_ID}"
  rm -rf "$wt"
  git worktree add --force "$wt" "$commit" >/dev/null 2>&1
  ( cd "$wt" && CARGO_TARGET_DIR="$wt/target" cargo build --release -p atlas-cli 2>&1 | tail -1 )
done
for rep in $(seq 1 "$reps"); do
  for commit in $commits; do
    bin="$submit_dir-ab-avann-$commit-${SLURM_JOB_ID}/target/release/atlas-cli"
    out="$submit_dir/ab-avann-$commit.$rep.out"
    ( cd /public/home/majj/atlasofliegroups-4d3e9449/atlas-scripts && /usr/bin/time -v "$bin" < "$probe" > "$out" 2> "$submit_dir/ab-avann-$commit.$rep.time" )
    st=$?
    wall=$(grep 'wall clock' "$submit_dir/ab-avann-$commit.$rep.time" 2>/dev/null | awk '{print $NF}')
    rss=$(grep 'Maximum resident' "$submit_dir/ab-avann-$commit.$rep.time" 2>/dev/null | awk '{print $NF}')
    echo "AB_AVANN $commit rep=$rep exit=$st wall=$wall rss_kb=$rss"
  done
done
for commit in $commits; do
  ref="ab-avann-$commit.1.out"
  for rep in $(seq 2 "$reps"); do
    diff <(sort "$ref") <(sort "ab-avann-$commit.$rep.out") >/dev/null \
      && echo "AB_AVANN_DIFF $commit rep=$rep IDENTICAL" \
      || echo "AB_AVANN_DIFF $commit rep=$rep DIFFERS"
  done
  diff <(sort ab-avann-a869688.1.out) <(sort "ab-avann-$commit.1.out") >/dev/null \
    && echo "AB_AVANN_CROSS $commit vs a869688 IDENTICAL" \
    || echo "AB_AVANN_CROSS $commit vs a869688 DIFFERS"
  wt="$submit_dir-ab-avann-$commit-${SLURM_JOB_ID}"
  git worktree remove --force "$wt" 2>/dev/null || rm -rf "$wt"
done
echo AB_AVANN_DONE

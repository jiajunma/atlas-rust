#!/usr/bin/env bash
# Heap attribution: run atlas-cli on the given script(s) under valgrind
# massif, then ms_print the profile. Usage:
#   sbatch hpc/massif_profile.sbatch /path/to/script.at [more.at ...]
# Output lands in $SLURM_SUBMIT_DIR: massif.<jobid>.out (raw) and
# massif.<jobid>.txt (ms_print), plus a no-valgrind /usr/bin/time -v
# baseline in time.<jobid>.txt so heap-vs-maxrss arena slack is visible.
#SBATCH --job-name=atlas-massif
#SBATCH --partition=cpu
#SBATCH --account=yushilingroup
#SBATCH --qos=normal
#SBATCH --nodes=1
#SBATCH --ntasks=1
#SBATCH --cpus-per-task=4
#SBATCH --mem=8G
#SBATCH --time=02:00:00
#SBATCH --output=massif-%j.out
set -euo pipefail
cd "$SLURM_SUBMIT_DIR"
export PATH="$HOME/.cargo/bin:$PATH"

echo "building atlas-cli (release)..."
cargo build --release -p atlas-cli

SCRIPTS="${ATLAS_SCRIPTS_DIR:-/public/home/majj/atlasofliegroups-4d3e9449/atlas-scripts}"
cli=./target/release/atlas-cli
test "$#" -ge 1 || { echo "usage: sbatch $0 script.at [...]" >&2; exit 2; }

echo "== baseline maxrss without valgrind =="
/usr/bin/time -v "$cli" --path="$SCRIPTS" "$@" > /dev/null 2> "time.$SLURM_JOB_ID.txt" || true
grep -E "Maximum resident|Elapsed" "time.$SLURM_JOB_ID.txt"

echo "== massif heap profile =="
valgrind --tool=massif --threshold=0.2 --max-snapshots=200 --detailed-freq=10 \
  --massif-out-file="massif.$SLURM_JOB_ID.out" \
  "$cli" --path="$SCRIPTS" "$@" > /dev/null 2> "massif.$SLURM_JOB_ID.stderr"
ms_print "massif.$SLURM_JOB_ID.out" > "massif.$SLURM_JOB_ID.txt"
grep -n "(peak)" "massif.$SLURM_JOB_ID.txt" || true

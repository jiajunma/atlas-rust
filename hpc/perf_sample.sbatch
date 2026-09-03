#!/bin/bash
#SBATCH --job-name=atlas-perf-sample
#SBATCH --output=perf-sample-%j.out
#SBATCH --partition=cpu
#SBATCH --account=yushilingroup
#SBATCH --qos=normal
#SBATCH --nodes=1
#SBATCH --ntasks=1
#SBATCH --cpus-per-task=2
#SBATCH --mem=4G
#SBATCH --time=00:30:00
# Sample the rust atlas-cli running a slow corpus script with gdb to find
# the remaining per-script hot spot (post orbit-construction fix).
# Recipe gotchas (docs/HANDOFF.md): `A && B &` backgrounds the whole list;
# CLI stdout is block-buffered, so a hung run leaves an empty file.
# Usage: sbatch --export=ALL,SCRIPT=springer_table_E8.at hpc/perf_sample.sbatch
set -u
SCRIPT=${SCRIPT:-springer_table_E8.at}
SAMPLES=${SAMPLES:-12}
INTERVAL=${INTERVAL:-2}
BIN=/public/home/majj/atlas-rust/target/release/atlas-cli
cd /public/home/majj/atlasofliegroups-4d3e9449/atlas-scripts
echo "script=$SCRIPT samples=$SAMPLES interval=$INTERVAL"
echo "binary commit: $(cd /public/home/majj/atlas-rust && git log --oneline -1)"
bash -c "exec $BIN < $SCRIPT > /dev/null 2>&1" &
PID=$!
for i in $(seq 1 "$SAMPLES"); do
  sleep "$INTERVAL"
  if ! kill -0 "$PID" 2>/dev/null; then echo "process exited after ~$((i*INTERVAL))s"; break; fi
  echo "=== sample $i (t=${i}x${INTERVAL}s) ==="
  gdb -p "$PID" -batch -ex "bt 22" 2>/dev/null | grep -E "^#" | head -22
done
wait "$PID" 2>/dev/null
echo "SAMPLE_DONE"

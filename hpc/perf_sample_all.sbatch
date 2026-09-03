#!/bin/bash
#SBATCH --job-name=atlas-perf-sample-all
#SBATCH --output=perf-sample-all-%j.out
#SBATCH --partition=cpu
#SBATCH --account=yushilingroup
#SBATCH --qos=normal
#SBATCH --nodes=1
#SBATCH --ntasks=1
#SBATCH --cpus-per-task=4
#SBATCH --mem=4G
#SBATCH --time=00:30:00
# perf_sample.sbatch variant that samples EVERY thread (the KGB BFS's rayon
# workers are invisible to main-thread-only sampling; post-2026-08-24j the
# main thread is mostly parked in LockLatch). One "=== sample i, thread t ==="
# section per thread per round; aggregate with:
#   grep -oE "in [a-zA-Z0-9_:<>., ]+\(\)" perf-sample-all-<id>.out | sort | uniq -c | sort -rn
# Usage: sbatch --export=ALL,SCRIPT=springer_table_E8.at hpc/perf_sample_all.sbatch
set -u
SCRIPT=${SCRIPT:-springer_table_E8.at}
SAMPLES=${SAMPLES:-12}
INTERVAL=${INTERVAL:-2}
BIN=/public/home/majj/atlas-rust/target/release/atlas-cli
cd /public/home/majj/atlasofliegroups-4d3e9449/atlas-scripts
echo "script=$SCRIPT samples=$SAMPLES interval=$INTERVAL (all threads)"
echo "binary commit: $(cd /public/home/majj/atlas-rust && git log --oneline -1)"
bash -c "exec $BIN < $SCRIPT > /dev/null 2>&1" &
PID=$!
for i in $(seq 1 "$SAMPLES"); do
  sleep "$INTERVAL"
  if ! kill -0 "$PID" 2>/dev/null; then echo "process exited after ~$((i*INTERVAL))s"; break; fi
  gdb -p "$PID" -batch -ex "thread apply all bt 12" 2>/dev/null \
    | grep -E "^Thread|^#" | head -80
  echo "=== sample $i done (t=${i}x${INTERVAL}s) ==="
done
wait "$PID" 2>/dev/null
echo "SAMPLE_DONE"

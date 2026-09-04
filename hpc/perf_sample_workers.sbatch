#!/bin/bash
#SBATCH --job-name=atlas-perf-workers
#SBATCH --output=perf-workers-%j.out
#SBATCH --partition=cpu
#SBATCH --account=yushilingroup
#SBATCH --qos=normal
#SBATCH --nodes=1
#SBATCH --ntasks=1
#SBATCH --cpus-per-task=4
#SBATCH --mem=8G
#SBATCH --time=00:40:00
# Sample ALL threads (incl. rayon KGB BFS workers) of the rust atlas-cli
# running a slow corpus script. Main-thread sampling (perf_sample.sbatch)
# only sees LockLatch parking; the workers' own cost needs
# `thread apply all bt`. Usage:
#   sbatch --export=ALL,SCRIPT=unipotent_representations_exceptional.at hpc/perf_sample_workers.sbatch
set -u
SCRIPT=${SCRIPT:-unipotent_representations_exceptional.at}
SAMPLES=${SAMPLES:-10}
INTERVAL=${INTERVAL:-2}
export PATH="$HOME/.cargo/bin:$PATH"
cd /public/home/majj/atlas-rust
echo "checkout commit: $(git log --oneline -1)"
cargo build --release -p atlas-cli 2>&1 | tail -2
BIN=/public/home/majj/atlas-rust/target/release/atlas-cli
cd /public/home/majj/atlasofliegroups-4d3e9449/atlas-scripts
echo "script=$SCRIPT samples=$SAMPLES interval=$INTERVAL"
bash -c "exec $BIN < $SCRIPT > /dev/null 2>&1" &
PID=$!
for i in $(seq 1 "$SAMPLES"); do
  sleep "$INTERVAL"
  if ! kill -0 "$PID" 2>/dev/null; then echo "process exited after ~$((i*INTERVAL))s"; break; fi
  echo "=== sample $i (t=${i}x${INTERVAL}s) ==="
  gdb -p "$PID" -batch -ex "thread apply all bt 14" 2>/dev/null | grep -E "^(Thread|#)" | head -60
done
wait "$PID" 2>/dev/null
echo "SAMPLE_DONE"

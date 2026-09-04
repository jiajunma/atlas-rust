#!/bin/bash
#SBATCH --job-name=perf-unitary
#SBATCH --output=perf-unitary-%j.out
#SBATCH --partition=fat
#SBATCH --account=yushilingroup
#SBATCH --qos=normal
#SBATCH --nodes=1
#SBATCH --ntasks=1
#SBATCH --cpus-per-task=4
#SBATCH --mem=32G
#SBATCH --time=01:30:00
# Frame-pointer perf profile of a heavy unitarity workload (default:
# probe_unitary_e7_heavy.atlas). Builds atlas-cli into a SEPARATE
# target-prof dir with frame pointers and LTO off (under the default
# lto=fat+cgu=1 the deform path inlines away and perf srcline is
# useless — lane C note), then perf-records the probe under a timeout.
# Env overrides: PROBE, PERF_SECONDS (default 1500), FREQ (default 499).
# Gotcha (lane C): perf script headers embed the binary path, which
# contains the worktree name — parse frame lines only.
set -euo pipefail
submit_dir="${SLURM_SUBMIT_DIR:?SLURM_SUBMIT_DIR is required}"
cd "$submit_dir"
export PATH="$HOME/.cargo/bin:$PATH"
probe="${PROBE:-hpc/workloads/probe_unitary_e7_heavy.atlas}"
[[ "$probe" = /* ]] || probe="$submit_dir/$probe"
seconds="${PERF_SECONDS:-1500}"
freq="${FREQ:-499}"

export CARGO_TARGET_DIR="$submit_dir/target-prof"
export CARGO_PROFILE_RELEASE_LTO=off
export CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16
export RUSTFLAGS="-C force-frame-pointers=yes"
cargo build --release -p atlas-cli 2>&1 | tail -2
bin="$CARGO_TARGET_DIR/release/atlas-cli"

cd /public/home/majj/atlasofliegroups-4d3e9449/atlas-scripts
data="$submit_dir/perf-unitary-${SLURM_JOB_ID}.data"
timeout "${seconds}s" perf record -F "$freq" -g -o "$data" -- "$bin" < "$probe" > "$submit_dir/perf-unitary-${SLURM_JOB_ID}.stdout" 2>&1 || true
echo "=== flat top 40 (overhead, command-shared-object-symbol) ==="
perf report -i "$data" --stdio --no-children 2>/dev/null | grep -E '^\s*[0-9]+\.[0-9]+%' | head -40 || true
echo "PERF_UNITARY_DONE data=$data"

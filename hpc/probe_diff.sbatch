#!/bin/bash
#SBATCH --job-name=probe-diff
#SBATCH --output=probe-diff-%j.out
#SBATCH --partition=cpu
#SBATCH --account=yushilingroup
#SBATCH --qos=normal
#SBATCH --nodes=1
#SBATCH --ntasks=1
#SBATCH --cpus-per-task=1
#SBATCH --mem=4012M
#SBATCH --time=01:00:00
# Generic rust-vs-oracle probe differential with benchmark capture.
# Both legs run regardless of failure; outputs land in the submit dir.
# Env overrides (via sbatch --export=ALL,... or --partition/--mem/--time):
#   PROBE   atlas script path (absolute, or relative to submit dir)
#   RUSTBIN atlas-cli binary (default: $SLURM_SUBMIT_DIR/target/release/atlas-cli)
#   TAG     output label (default: probe basename)
#   ATLAS_BIN  oracle binary (default: $HOME/atlasofliegroups-4d3e9449/atlas)
set -u
submit_dir="${SLURM_SUBMIT_DIR:?SLURM_SUBMIT_DIR is required}"
probe="${PROBE:?set PROBE to the atlas script path}"
[[ "$probe" = /* ]] || probe="$submit_dir/$probe"
[[ -f "$probe" ]] || { echo "PROBE_MISSING $probe"; exit 1; }
tag="${TAG:-$(basename "$probe" .atlas)}"
rustbin="${RUSTBIN:-$submit_dir/target/release/atlas-cli}"
gcc_root="${ATLAS_GCC_ROOT:-/public/software/gcc/gcc-12.1}"
gcc_lib="$(dirname "$("$gcc_root/bin/gcc" -print-file-name=libstdc++.so.6)")"
export LD_LIBRARY_PATH="$gcc_lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
atlas_bin="${ATLAS_BIN:-$HOME/atlasofliegroups-4d3e9449/atlas}"
cd /public/home/majj/atlasofliegroups-4d3e9449/atlas-scripts

stat_line() { # label, time-file
  echo "$1 exit=$2 wall=$(grep 'wall clock' "$3" 2>/dev/null | awk '{print $NF}') rss_kb=$(grep 'Maximum resident' "$3" 2>/dev/null | awk '{print $NF}')"
}

/usr/bin/time -v "$rustbin" < "$probe" > "$submit_dir/$tag.rust.out" 2> "$submit_dir/$tag.rust.time"
rust_status=$?
/usr/bin/time -v "$atlas_bin" < "$probe" > "$submit_dir/$tag.cpp.out" 2> "$submit_dir/$tag.cpp.time"
cpp_status=$?
stat_line "rust" "$rust_status" "$submit_dir/$tag.rust.time"
stat_line "oracle" "$cpp_status" "$submit_dir/$tag.cpp.time"
# A printed polynomial's last term carries no trailing comma, so a
# term-order difference shifts a comma between otherwise identical
# lines; strip line-final commas before the sorted comparison.
if diff <(sed 's/,$//' "$submit_dir/$tag.cpp.out" | sort) <(sed 's/,$//' "$submit_dir/$tag.rust.out" | sort) > /dev/null 2>&1; then
  echo "PROBE_DIFF $tag: SORTED_MATCH ($(wc -l < "$submit_dir/$tag.rust.out") lines)"
else
  echo "PROBE_DIFF $tag: SORTED_DIFF diff_lines=$(diff <(sed 's/,$//' "$submit_dir/$tag.cpp.out" | sort) <(sed 's/,$//' "$submit_dir/$tag.rust.out" | sort) | grep -c '^[<>]')"
fi
echo "PROBE_DONE $tag"

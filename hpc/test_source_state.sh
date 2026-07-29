#!/usr/bin/env bash

set -euo pipefail

hpc_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$hpc_dir/source_state.sh"

test_dir="$(mktemp -d)"
trap 'rm -rf "$test_dir"' EXIT
cd "$test_dir"

git init -q
git config user.email atlas-test@example.invalid
git config user.name atlas-test
printf 'tracked\n' >tracked.txt
git add tracked.txt
git commit -qm initial

test "$(atlas_detect_dirty_tree atlas-pipeline-swap-123.out)" = false

printf 'slurm output\n' >atlas-pipeline-swap-123.out
test "$(atlas_detect_dirty_tree atlas-pipeline-swap-123.out)" = false

printf 'other output\n' >atlas-pipeline-swap-456.out
test "$(atlas_detect_dirty_tree atlas-pipeline-swap-123.out)" = true
rm atlas-pipeline-swap-456.out

printf 'source edit\n' >>tracked.txt
test "$(atlas_detect_dirty_tree atlas-pipeline-swap-123.out)" = true
git checkout -q -- tracked.txt

git add atlas-pipeline-swap-123.out
git commit -qm 'track log name'
printf 'changed tracked log\n' >>atlas-pipeline-swap-123.out
test "$(atlas_detect_dirty_tree atlas-pipeline-swap-123.out)" = true

printf 'source-state checks passed\n'

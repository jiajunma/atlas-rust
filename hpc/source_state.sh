#!/usr/bin/env bash

# Slurm opens the configured stdout file before the job body starts. Ignore
# only that exact untracked file when measuring the submitted source tree.
atlas_detect_dirty_tree() {
  local slurm_stdout="${1:?Slurm stdout path is required}"

  if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    printf 'unknown\n'
    return 0
  fi

  local status
  status="$(git status --porcelain --untracked-files=all)" || return 2

  local line
  while IFS= read -r line; do
    if [[ -n "$line" && "$line" != "?? $slurm_stdout" ]]; then
      printf 'true\n'
      return 0
    fi
  done <<<"$status"

  printf 'false\n'
}

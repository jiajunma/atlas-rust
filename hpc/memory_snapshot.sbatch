#!/usr/bin/env bash
# Capture scheduler and cgroup memory facts from the node running this step.
# This is an operations probe, not a benchmark, and contains no credentials.
#SBATCH --job-name=atlas-memory-snapshot
#SBATCH --partition=cpu
#SBATCH --account=yushilingroup
#SBATCH --qos=normal
#SBATCH --nodes=1
#SBATCH --ntasks=1
#SBATCH --cpus-per-task=1
#SBATCH --mem=1G
#SBATCH --time=00:05:00
#SBATCH --output=atlas-memory-snapshot-%j.out

set -euo pipefail

printf '== identity ==\n'
hostname
date -Is
printf 'job_id=%s\n' "${SLURM_JOB_ID:-unknown}"
printf 'job_node=%s\n' "${SLURMD_NODENAME:-unknown}"
printf 'cpus_per_task_requested=%s\n' "${SLURM_CPUS_PER_TASK:-unknown}"
printf 'job_cpus_per_node_allocated=%s\n' "${SLURM_JOB_CPUS_PER_NODE:-unknown}"
printf 'mem_per_node_requested=%s\n' "${SLURM_MEM_PER_NODE:-unknown}"
printf 'mem_per_cpu_requested=%s\n' "${SLURM_MEM_PER_CPU:-unknown}"

printf '\n== partition ==\n'
scontrol show partition cpu -o
scontrol show partition fat -o

printf '\n== node ==\n'
sinfo -Nel -n "${SLURMD_NODENAME:?SLURMD_NODENAME is required}"

printf '\n== slurm config ==\n'
scontrol show config | grep -E '^(AccountingStorageEnforce|AccountingStorageTRES|DefMemPerNode|MaxMemPerNode|SelectType|SelectTypeParameters|TaskPlugin|TaskPluginParam|ProctrackType|ConstrainRAMSpace|ConstrainSwapSpace|JobAcctGatherType|JobAcctGatherFrequency|JobSubmitPlugins)'
printf 'compute_node_cgroup_conf:\n'
if [[ -r /etc/slurm/cgroup.conf ]]; then
  grep -Ev '^[[:space:]]*(#|$)' /etc/slurm/cgroup.conf
else
  printf 'not readable\n'
fi

printf '\n== qos and association ==\n'
sacctmgr show qos normal,long -P -o Name,Flags,MaxWall,MaxTRES,MaxTRESPerNode,MaxTRESPU,MaxJobsPU,MaxSubmitPU
sacctmgr show assoc where user="${USER:?USER is required}" account=yushilingroup -Pn

printf '\n== job ==\n'
scontrol show job "${SLURM_JOB_ID:?SLURM_JOB_ID is required}"

printf '\n== proc meminfo ==\n'
grep -E '^(MemTotal|MemFree|MemAvailable|SwapTotal|SwapFree|CommitLimit|Committed_AS):' /proc/meminfo
printf '\n== vm policy ==\n'
for name in overcommit_memory overcommit_ratio swappiness; do
  path="/proc/sys/vm/$name"
  if [[ -r "$path" ]]; then
    printf '%s=' "$path"
    tr -d '\n' < "$path"
    printf '\n'
  fi
done
printf '\n== numa and affinity ==\n'
if command -v numactl >/dev/null 2>&1; then
  numactl --show
  numactl --hardware
else
  printf 'numactl=unavailable\n'
fi
grep -E '^(Cpus_allowed_list|Mems_allowed_list):' /proc/self/status || true

printf '\n== cgroup ==\n'
printf 'proc_self_cgroup:\n'
cat /proc/self/cgroup
printf 'memory_files:\n'
print_cgroup_file() {
  local path="${1:?cgroup control-file path is required}"
  local line_count
  line_count="$(wc -l < "$path")"
  if [[ "$line_count" -le 1 ]]; then
    printf '%s=' "$path"
    tr -d '\n' < "$path"
    printf '\n'
  else
    printf '%s:\n' "$path"
    sed 's/^/  /' "$path"
  fi
}

# Read this task's ancestors only through its own job cgroup. Higher ancestors
# aggregate other jobs on a shared node and are outside this probe's scope.
while IFS=: read -r hierarchy controllers relative; do
  if [[ "$hierarchy" == 0 && -z "$controllers" ]]; then
    cgroup_dir="/sys/fs/cgroup${relative}"
    while [[ "$cgroup_dir" == /sys/fs/cgroup* ]]; do
      printf '[v2] %s\n' "$cgroup_dir"
      for name in memory.max memory.current memory.stat memory.events memory.swap.max; do
        path="$cgroup_dir/$name"
        if [[ -r "$path" ]]; then
          print_cgroup_file "$path"
        fi
      done
      if [[ "$cgroup_dir" == */job_"${SLURM_JOB_ID}" || \
          "$cgroup_dir" == */job_"${SLURM_JOB_ID}".scope ]]; then
        break
      fi
      [[ "$cgroup_dir" == /sys/fs/cgroup ]] && break
      cgroup_dir="${cgroup_dir%/*}"
      [[ -n "$cgroup_dir" ]] || break
    done
  elif [[ "$controllers" == *memory* ]]; then
    # XMU currently mounts the Slurm memory controller as cgroup v1. Locate
    # the controller mount by its conventional path, then append the exact
    # relative group from /proc/self/cgroup; never walk sibling groups.
    for mount in /sys/fs/cgroup/memory /sys/fs/cgroup; do
      cgroup_dir="$mount${relative}"
      if [[ -d "$cgroup_dir" ]]; then
        while [[ "$cgroup_dir" == "$mount"* ]]; do
          printf '[v1] %s\n' "$cgroup_dir"
          for name in memory.limit_in_bytes memory.soft_limit_in_bytes \
              memory.usage_in_bytes \
              memory.max_usage_in_bytes memory.failcnt memory.oom_control \
              memory.memsw.limit_in_bytes memory.memsw.usage_in_bytes \
              memory.stat memory.swappiness; do
            path="$cgroup_dir/$name"
            if [[ -r "$path" ]]; then
              print_cgroup_file "$path"
            fi
          done
          [[ "$cgroup_dir" == */job_"${SLURM_JOB_ID}" ]] && break
          [[ "$cgroup_dir" == "$mount" ]] && break
          cgroup_dir="${cgroup_dir%/*}"
          [[ -n "$cgroup_dir" ]] || break
        done
        break
      fi
    done
  fi
done < /proc/self/cgroup

snapshot_sleep="${SNAPSHOT_SLEEP_SECONDS:-0}"
if [[ "$snapshot_sleep" =~ ^[0-9]+$ && "$snapshot_sleep" -gt 0 ]]; then
  printf '\n== hold ==\nseconds=%s\n' "$snapshot_sleep"
  sleep "$snapshot_sleep"
fi

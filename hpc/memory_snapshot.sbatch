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
printf 'cpus_allocated=%s\n' "${SLURM_CPUS_PER_TASK:-unknown}"
printf 'mem_per_node_requested=%s\n' "${SLURM_MEM_PER_NODE:-unknown}"
printf 'mem_per_cpu_requested=%s\n' "${SLURM_MEM_PER_CPU:-unknown}"

printf '\n== partition ==\n'
scontrol show partition cpu -o
scontrol show partition fat -o

printf '\n== node ==\n'
sinfo -Nel -n "${SLURMD_NODENAME:?SLURMD_NODENAME is required}"

printf '\n== slurm config ==\n'
scontrol show config | grep -E '^(DefMemPerNode|MaxMemPerNode|SelectTypeParameters|TaskPlugin|TaskPluginParam|ProctrackType|ConstrainRAMSpace|ConstrainSwapSpace|JobAcctGatherType|JobAcctGatherFrequency)'

printf '\n== qos and association ==\n'
sacctmgr show qos normal,long -P -o Name,Flags,MaxWall,MaxTRES,MaxTRESPerNode,MaxTRESPU,MaxJobsPU,MaxSubmitPU
sacctmgr show assoc where user="${USER:?USER is required}" account=yushilingroup -Pn

printf '\n== job ==\n'
scontrol show job "${SLURM_JOB_ID:?SLURM_JOB_ID is required}"

printf '\n== proc meminfo ==\n'
grep -E '^(MemTotal|MemFree|MemAvailable|SwapTotal|SwapFree|CommitLimit|Committed_AS):' /proc/meminfo

printf '\n== cgroup ==\n'
printf 'proc_self_cgroup:\n'
cat /proc/self/cgroup
printf 'memory_files:\n'
# Read only this step's cgroup. Recursing through /sys/fs/cgroup would include
# unrelated users' jobs on a shared node and is unnecessary for this probe.
while IFS=: read -r hierarchy controllers relative; do
  if [[ "$hierarchy" == 0 && -z "$controllers" ]]; then
    cgroup_dir="/sys/fs/cgroup${relative}"
    for name in memory.max memory.current memory.events memory.swap.max; do
      path="$cgroup_dir/$name"
      if [[ -r "$path" ]]; then
        printf '%s=' "$path"
        tr -d '\n' < "$path"
        printf '\n'
      fi
    done
  fi
done < /proc/self/cgroup

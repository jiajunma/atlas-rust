# HPC Memory Policy Audit Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Correct the repository's XMU Slurm memory model and restore safe job defaults after the false `4G per job` assumption.

**Architecture:** Treat requested Slurm memory, allocated CPUs, cgroup-enforced resident memory, process virtual address space, and measured RSS as separate quantities. Encode stable checked-in defaults in tests, preserve live scheduler/cgroup evidence in the handoff, and keep heavy E7/E8 overrides on `fat` explicit.

**Tech Stack:** Slurm 21.08, cgroup v1, Bash sbatch scripts, Python `unittest`, GNU `time -v`.

---

### Task 1: Freeze the corrected resource policy in tests

**Files:**
- Modify: `hpc/test_slurm_memory_defaults.py`
- Test: `hpc/test_slurm_memory_defaults.py`

- [x] **Step 1: Add expected-resource and cgroup-headroom assertions**

Define the known resource defaults for the jobs reduced by commit `5c525db`, distinguish the 4096 MiB submit boundary from `AllowedRAMSpace=90`, and assert that `script_corpus_diff.DEFAULT_MEM_CAP_GB` fits below the resulting corpus cgroup budget with driver headroom.

- [x] **Step 2: Run the focused test and verify RED**

Run: `python3 -m unittest hpc/test_slurm_memory_defaults.py`

Expected: failures show the current 4G job defaults and 3 GiB corpus child limit do not match the corrected policy.

### Task 2: Restore valid job and child defaults

**Files:**
- Modify: `hpc/differential.sbatch`
- Modify: `hpc/filekl_diff.sbatch`
- Modify: `hpc/kgb_differential.sbatch`
- Modify: `hpc/massif_profile.sbatch`
- Modify: `hpc/perf_sample_workers.sbatch`
- Modify: `hpc/pipeline_swap_diff.sbatch`
- Modify: `hpc/quick_check.sbatch`
- Modify: `hpc/real_group_preflight.sbatch`
- Modify: `hpc/script_corpus.sbatch`
- Modify: `hpc/weyl_focused.sbatch`
- Modify: `hpc/script_corpus_diff.py`

- [x] **Step 1: Restore the pre-regression memory totals**

Use 8G for the affected two-CPU jobs, 16G for KGB and corpus four-CPU jobs, and the prior 8G totals for Massif, worker profiling, and quick-check. Set `probe_diff.sbatch` to the exact one-CPU `MaxMemPerCPU=4012M` boundary so Slurm does not allocate a second CPU merely to satisfy a 4G request; heavy probes continue to override both partition and memory.

- [x] **Step 2: Restore the corpus runaway guard to 6 GiB**

Set `DEFAULT_MEM_CAP_GB = 6`. This remains a per-child `RLIMIT_AS`, leaves more than half of the CPU corpus job's observed 14.4 GiB cgroup budget for the driver and mappings, and still prevents the historical 12.4 GiB runaway from killing the entire job.

- [x] **Step 3: Run the focused test and verify GREEN**

Run: `python3 -m unittest hpc/test_slurm_memory_defaults.py`

Expected: all memory-policy tests pass.

### Task 3: Record the live memory model accurately

**Files:**
- Modify: `hpc/README.md`
- Modify: `docs/HANDOFF.md`

- [x] **Step 1: Document scheduler allocation behavior**

Record that `--test-only` validates the request but a real one-CPU job may receive two logical CPUs because complete cores are allocated. Use `sacct AllocTRES` and `scontrol NumCPUs`, not only `--cpus-per-task`, when reporting the actual allocation.

- [x] **Step 2: Document partition-specific cgroup enforcement**

Record CPU job evidence (`AllowedRAMSpace=90`, 1G to 966365184 bytes, 4G to 3865468928 bytes, 8G to 7730937856 bytes) and fat evidence (no `AllowedRAMSpace` override, 32G to 34359738368 bytes). State that the task subgroup's unlimited sentinel is not the job limit.

- [x] **Step 3: Document measurement meanings**

Keep `RLIMIT_AS`, process peak RSS, Slurm MaxRSS, physical node RAM, and job cgroup hard limit distinct. Explain that larger node RAM is shared scheduler capacity and is not automatically available to a job.

### Task 4: Verify the corrected repository state

**Files:**
- Test: all `hpc/test_*.py`
- Test: all `hpc/*.sbatch`

- [x] **Step 1: Run bounded local checks**

Run `python3 -m unittest discover -s hpc -p 'test_*.py'`, `bash -n` for every sbatch file, and `git diff --check`.

Expected: all checks pass with no syntax or whitespace errors.

- [ ] **Step 2: Inspect the final diff and preserve unrelated files**

Verify that `hpc/workloads/probe_associated_variety_a2.atlas` and `rust_out` remain untouched and untracked.

- [ ] **Step 3: Commit the correction**

Commit only the memory-policy source, test, plan, and documentation changes using `fix: correct HPC memory allocation defaults`.

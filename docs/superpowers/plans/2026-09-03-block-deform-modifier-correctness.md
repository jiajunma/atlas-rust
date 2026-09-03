# Modifier-Aware Block Deformation Correctness Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Restore `block_deform` parity with the Atlas oracle before any further deformation or unitarity optimization.

**Architecture:** Treat the `LocatedBlock` returned by `RepTable::lookup_full_block` as the authoritative source for transported row representatives and singular subsystem generators. Keep the independently built `BlockGraph` only while its full topology and row numbering are proved identical; otherwise route the KL/deformation topology through the located block itself.

**Tech Stack:** Rust, Atlas source/CWEB oracle, existing fixture pipeline, XMU SLURM jobs with wall-time and peak-RSS capture.

---

### Task 1: Freeze The Singular-Subsystem Regression

**Files:**
- Create: `tests/fixtures/domain/block_deform_integral_singular.atlas`
- Create: `tests/reference/domain/block_deform_integral_singular.meta.json`
- Modify after capture: `tests/reference/domain/block_deform_integral_singular.events.json`
- Modify after capture: `hpc/pipeline_swap_diff.py`

- [x] **Step 1: Capture the A2 oracle result on HPC**

Run `hpc/reference_capture.sbatch tests/fixtures/domain/block_deform_integral_singular.atlas` from a clean worktree. Expected: a verified reference capture with wall time, peak RSS, and exact output for `deform(p)` plus `block_deform(p,d,-1)`.

- [x] **Step 2: Confirm the current Rust build is RED**

Run `hpc/probe_diff.sbatch` on the fixture at `bf0c43f`. Observed in job 3674819: `SORTED_DIFF`, Rust exit 1 at the cross-construction row-alignment guard, oracle exit 0.

- [x] **Step 3: Add the captured events and register the fixture**

Generate the exact event expectation from the pinned capture, update the meta file with capture job/checksums/timing, and add a `FixturePlan` beside `domain/block_deform` in `hpc/pipeline_swap_diff.py`.

### Task 2: Prove The Failing Boundary

**Files:**
- Modify: `crates/atlas-core/src/domain_builtins.rs`
- Test: `crates/atlas-core/src/domain_builtins.rs`

- [x] **Step 1: Add a focused internal test**

Add a high-level test for the exact captured tuple. It must fail on bf0c43f at the row-alignment invariant. Separately compare the A2 located block with the independent BlockGraph to pin the first row/topology mismatch; after using one topology, compare modifier-aware and ambient-simple survivor sets.

- [x] **Step 2: Run the RED test on HPC**

Run the exact test through `hpc/quick_check.sbatch` or a focused preflight. Expected: failure at `rep-context block deformation full-block row alignment invariant was violated`.

- [x] **Step 3: Verify topology alignment separately**

The A2 fixture already proved that the two row numberings differ, so the stronger E6 field dump was no longer needed to choose the architecture: any mismatch blocks the singular-only fix and requires one authoritative topology.

### Task 3: Implement One Root-Cause Fix

**Files:**
- Modify: `crates/atlas-real-group/src/deform.rs`
- Modify: `crates/atlas-core/src/domain_builtins.rs`

- [x] **Step 1: Pass singular flags into deformation**

Change `block_deformation_to_height` and its internal body to take the `LocatedBlock`. Build/fill the dual KL table from `located.block().dual()`, reconstruct rows with `sr_with_modifier`, and filter with modifier-aware singular flags. Remove the independent `BlockGraph`, x-alignment guard, and internal `simple_singular_flags(rc, gamma)` call.

- [x] **Step 2: Preserve small test semantics explicitly**

Existing A2 direct tests pass their known simple flags. Add the captured regression as the high-level proof that the wrapper uses `located_singular_flags`.

- [x] **Step 3: Stop if topology alignment failed**

If Task 2 found any mismatch, do not add more row guards. Generalize the dual-KL cache/deformation body over `PartialBlock` and derive both topology and row parameters from the same `LocatedBlock`.

### Task 4: Verify Before Optimizing

**Files:**
- Modify after successful jobs: `docs/HANDOFF.md`
- Modify after successful jobs: `docs/REMAINING_BUILTINS.md`
- Modify after successful jobs: `docs/BENCHMARKS.md`

- [ ] **Step 1: Run focused and full correctness gates**

Submit quick check and the 240-fixture corpus. Then run sorted E6 x=1790 differential and the A2 regression. Expected: all exact matches.

- [ ] **Step 2: Run the E7 correctness gate**

Submit `probe_bd_e7_single.atlas` to the `fat` partition with 32GB and a generous timeout. Expected: sorted match against the pinned oracle output; record both timings and RSS.

- [ ] **Step 3: Resume performance work only after all gates pass**

Profile `probe_unitary_e7_heavy.atlas` with frame pointers before considering an `ExtKlTable` cache. Build an associated-cycle workload and locate its Rust execution boundary before optimizing script-level enumeration, K-type indexing, or matrix operations.

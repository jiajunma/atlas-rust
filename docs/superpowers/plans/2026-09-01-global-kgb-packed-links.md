# GlobalKGB Packed Links Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Reduce GlobalKGB link-table memory by storing element identifiers as `u32` while preserving the public Atlas-compatible `usize` API and all invalid-input behavior.

**Architecture:** Introduce private checked pack/unpack helpers and a two-word packed inverse-Cayley slot in `global_kgb.rs`. `ElementStore` and `GlobalKgb` retain flat `element * rank + generator` indexing, but their targets use `u32::MAX` as the absent/unwritten sentinel and decode only at public/API and invariant boundaries.

**Tech Stack:** Rust standard library, atlas-real-group unit tests, XMU SLURM focused tests and differential drivers.

---

### Task 1: Pin Packed Target Semantics

**Files:**
- Modify: `crates/atlas-real-group/src/global_kgb.rs`
- Test: `crates/atlas-real-group/src/global_kgb.rs` (`#[cfg(test)]` module)

- [ ] **Step 1: Write failing private representation tests**

Add tests that require private helpers with these exact semantics:

```rust
#[test]
fn packed_global_kgb_target_round_trips_and_reserves_sentinel() {
    assert_eq!(unpack_target(pack_target(0).unwrap()), Some(0));
    assert_eq!(unpack_target(pack_target(u32::MAX as usize - 1).unwrap()), Some(u32::MAX as usize - 1));
    assert_eq!(unpack_target(NO_TARGET), None);
    assert_eq!(pack_target(u32::MAX as usize), Err(StructureError::ArithmeticOverflow));
}

#[test]
fn packed_inverse_cayley_preserves_outer_and_inner_none() {
    assert_eq!(PackedInverseCayley::NONE.unpack(), None);
    assert_eq!(PackedInverseCayley::one(7).unwrap().unpack(), Some((7, None)));
    assert_eq!(PackedInverseCayley::two(7, 11).unwrap().unpack(), Some((7, Some(11))));
}
```

- [ ] **Step 2: Run the RED test on HPC**

Sync the isolated branch into its own HPC worktree, then run:

```bash
sbatch --partition=cpu --mem=8G hpc/real_group_preflight.sbatch
```

Expected: compile failure because `NO_TARGET`, `pack_target`, `unpack_target`, and `PackedInverseCayley` do not exist. Record the job ID and compiler failure in `docs/HANDOFF.md` after integration.

- [ ] **Step 3: Commit the RED test**

```bash
git add crates/atlas-real-group/src/global_kgb.rs
git commit -m "test: specify packed global KGB links"
```

### Task 2: Pack GlobalKGB Link Storage

**Files:**
- Modify: `crates/atlas-real-group/src/global_kgb.rs`

- [ ] **Step 1: Add checked target packing**

Add a private `NO_TARGET: u32 = u32::MAX`, a checked `pack_target(usize) -> Result<u32, StructureError>`, and `unpack_target(u32) -> Option<usize>`. `u32::MAX` must never encode an element.

- [ ] **Step 2: Add the inverse-Cayley packed slot**

Add a private `#[derive(Clone, Copy, Debug, Eq, PartialEq)] struct PackedInverseCayley { first: u32, second: u32 }`. `NONE` uses both sentinels; a present first target with sentinel second represents `Some((first, None))`. Constructors must call `pack_target`.

- [ ] **Step 3: Change only internal vectors**

Change both `GlobalKgb` and `ElementStore` to:

```rust
cross: Vec<u32>,
cayley: Vec<u32>,
inverse_cayley: Vec<PackedInverseCayley>,
```

Keep `statuses`, `elements`, `element_packet`, packet metadata, and every public method signature unchanged. Initialize cross and Cayley slots with `NO_TARGET` and inverse slots with `PackedInverseCayley::NONE`.

- [ ] **Step 4: Encode every write and decode every read**

Use `pack_target` at BFS writes. Decode in `cross`, `cayley`, `inverse_cayley`, `print_layout`, and internal invariant checks. A missing cross after successful construction remains an invariant violation; invalid element or generator indexes retain their current `None` result and error precedence.

- [ ] **Step 5: Commit the minimal GREEN implementation**

```bash
git add crates/atlas-real-group/src/global_kgb.rs
git commit -m "perf: pack global KGB link targets"
```

### Task 3: Verify Behavior And Measure Memory

**Files:**
- Modify after successful jobs: `docs/HANDOFF.md`
- Modify after successful jobs: `docs/BENCHMARKS.md`

- [ ] **Step 1: Run focused debug and release gates on HPC**

```bash
sbatch --partition=cpu --mem=8G hpc/real_group_preflight.sbatch
sbatch --partition=cpu --mem=8G hpc/weyl_focused.sbatch
```

Expected: atlas-real-group check passes; GlobalKGB, Weyl, InvolutionTable, and KGB tests pass in both profiles.

- [ ] **Step 2: Run the full differential pipeline on HPC**

```bash
sbatch --partition=fat --mem=32G --export=ALL,TIMEOUT=1200 hpc/pipeline_swap_diff.sbatch
```

Expected: all registered fixtures match the original Atlas oracle, with zero undeclared pending cases and complete wall/RSS fields.

- [ ] **Step 3: Run targeted print_X and heavy anchors**

Use `hpc/script_corpus.sbatch` for the full-KGB `print_X` fixture and `unipotent_representations_exceptional.at`, always recording Rust/oracle wall time and peak RSS. The `print_X` workload should show the direct GlobalKGB storage effect; the unipotent workload is a control because it uses `KgbGraph`, not `GlobalKgb`.

- [ ] **Step 4: Record exact evidence**

Add commit, job IDs, pass counts, report hashes, Rust/oracle seconds, Rust/oracle MaxRSS, and both ratios to `docs/HANDOFF.md` and `docs/BENCHMARKS.md`. Do not claim an unmeasured improvement.

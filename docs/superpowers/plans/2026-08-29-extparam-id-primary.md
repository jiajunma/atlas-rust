# ExtParam ID-Primary Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remove the owned `WeylElement` from every `ExtParam` and make the table-scoped `InvolutionId` its sole involution representation.

**Architecture:** `ExtParam` stores `InvolutionId`; theta/classification/KGB lookups consume it directly. Simple twisted conjugation replays existing `InvolutionTable::cross` links. Arbitrary reflection words use `weyl_left_word_lookup`, which allows non-table intermediate Weyl elements and resolves only the final result. Legacy permutations remain only inside compatibility tests and `InvolutionRecord` until the later record cleanup.

**Tech Stack:** Safe Rust, existing `InvolutionTable` compact transitions, HPC SLURM tests, Atlas differential fixtures.

---

### Task 1: Specify ID-primary ExtParam behavior

**Files:**
- Modify: `crates/atlas-real-group/src/ext_param.rs` tests

- [ ] Add an `involution_id()` accessor test for default extensions in A1, split A2, and B2.
- [ ] Exercise representative `star`/`complex_cross` links and assert every returned parameter's stored ID agrees with the table record selected by its observable `theta` and `x`.
- [ ] Submit an HPC RED test and require failure only because the accessor/storage does not exist.

### Task 2: Replace ExtParam storage and direct lookups

**Files:**
- Modify: `crates/atlas-real-group/src/ext_param.rs`

- [ ] Replace `tw: WeylElement` with `involution: InvolutionId`.
- [ ] Change `ExtParam::at`, `new`, `theta`, `theta_id`, `x`, `same_standard_reps`, and debug `validate` to use the stored ID.
- [ ] Change shifted-involution helpers to accept IDs and remove `table_lookup`.
- [ ] Preserve same-context/table ownership as an explicit API invariant.

### Task 3: Replace Weyl mutations with compact table transitions

**Files:**
- Modify: `crates/atlas-real-group/src/ext_param.rs`

- [ ] Replace simple twisted-conjugation loops with `InvolutionTable::cross` in their existing iteration order.
- [ ] Replace `word_product` call sites with `weyl_left_word_lookup`; map a missing final ID to the existing `RepInvariantViolation { invariant: "extended parameter Cartan lookup" }`.
- [ ] Preserve nested reflection-word concatenation order, October-surprise flips, and all existing `ExtParam::new` target choices.
- [ ] Remove `word_product`, `inner_twist`, and the `WeylElement` import when no production caller remains.

### Task 4: Verify and benchmark

**Files:**
- Modify after verification: `docs/HANDOFF.md`
- Modify after verification: `docs/BENCHMARKS.md`
- Modify after verification: `docs/WEYL_ELEMENT_MIGRATION_PLAN.md`

- [ ] Run local `cargo fmt --all -- --check` and scoped `git diff --check`; do not compile locally.
- [ ] Run HPC `ext_param` debug/release tests, then the focused real-group gate.
- [ ] Differential-test `default_extended`, `ext_block`, `ext_block_proper`, `shift_flip`, `ext_finalise`, and `twisted_family` through the full registered pipeline.
- [ ] Run the fat unipotent benchmark and record wall time plus peak RSS for Rust and C++.
- [ ] Dispatch spec and code-quality reviews before committing the production slice.

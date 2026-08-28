# BlockGraph Dual-Packet Compact Lookup Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remove per-packet legacy `WeylElement` cloning and hashing from `BlockGraph::build` while preserving Atlas dual-involution pairing and all existing error semantics.

**Architecture:** Add an `InvolutionTable` helper that starts from the compact dual longest element, replays the external involution word in reverse with the dual twist, and resolves the result through the table's compact index. In `BlockGraph::build`, index dual packets by `InvolutionId` using `Vec<Option<usize>>`, then pair primal packets through compact IDs; retain the existing `longest_action` budget gate and legacy `dual_involution` only for compatibility tests.

**Tech Stack:** Safe Rust, existing `CompactWeyl`/`WeylElt`, `InvolutionTable::compact_index`, HPC SLURM verification, Atlas C++ oracle.

---

### Task 1: Freeze compact dual lookup behavior with tests

**Files:**
- Modify: `crates/atlas-real-group/src/involution_table.rs` (`#[cfg(test)]` module)

- [ ] **Step 1: Write the failing test**

Add a test that builds A1, A2, and the existing nontrivial-twist A2 table contexts, computes the dual longest compact element, and asserts a wished-for helper returns the same `InvolutionId` as the existing `dual_involution` materialization for every table record. Include an invalid generator word case and assert the same `IndexOutOfRange` precedence as `dual_involution`.

- [ ] **Step 2: Run the focused HPC test to verify RED**

Submit a clean detached HPC worktree job running `cargo test -p atlas-real-group --lib involution_table compact_dual_lookup`. Expected result: compilation fails because the helper is not yet defined.

- [ ] **Step 3: Commit the RED fixture/test only**

Commit the test with `test: specify compact dual involution lookup` after confirming the failure is the missing helper, not a fixture/setup error.

### Task 2: Implement the compact dual lookup helper

**Files:**
- Modify: `crates/atlas-real-group/src/involution_table.rs` (production `impl InvolutionTable`)

- [ ] **Step 1: Implement the minimal helper**

Add `pub(crate) fn weyl_dual_lookup(&self, word: &[usize], dual_twist: &[usize]) -> Result<Option<InvolutionId>, StructureError>`. Start with `self.compact_weyl.longest()`, iterate `word.iter().rev()`, validate the external generator and twisted generator indices in the same order as `dual_involution`, apply compact `inner_mult`, then return `self.compact_index.get(&current).copied()`.

- [ ] **Step 2: Run the focused HPC test to verify GREEN**

Submit the same focused command from the exact RED worktree with an external `CARGO_TARGET_DIR`. Expected result: all compact dual lookup cases pass, including missing compact IDs and invalid generator diagnostics.

- [ ] **Step 3: Refactor only after green**

Keep the helper allocation-free apart from the input word and avoid materializing a `WeylElement`. Preserve `longest_action` outside this helper so its budget/error contract remains unchanged.

### Task 3: Migrate BlockGraph pairing to compact IDs

**Files:**
- Modify: `crates/atlas-real-group/src/block.rs` (`BlockGraph::build`)
- Test: `crates/atlas-real-group/src/block.rs` existing block tests plus one compact-vs-legacy pairing assertion

- [ ] **Step 1: Write the failing integration assertion**

Add an assertion in the block test helper that the packet pairing produced by the new compact path equals the current legacy path for the A1 SL(2,R)/PGL(2,R) anchor and the nontrivial A2 block anchor.

- [ ] **Step 2: Implement the minimal migration**

Build `dual_position: Vec<Option<usize>>` sized to `dual_table.involution_count()`, fill it from `dual_graph.packet_involution(position)`, compute `dual_longest` as a compact `WeylElt` from the existing `longest_action`, and call `dual_table.weyl_dual_lookup(&word, &dual_twist)`. Use the returned `InvolutionId` to read `dual_graph.tau_packet`; preserve `None => 0`, overflow checks, and all existing invariant errors. Remove only the `HashMap<WeylElement, usize>` and `Vec<WeylElement>` hot-path storage; leave the public free `dual_involution` function and its tests intact.

- [ ] **Step 3: Run block-focused HPC tests**

Run debug and release `cargo test -p atlas-real-group --lib block` in a clean HPC worktree with separate target directories. Expected result: all existing block tests pass and packet counts/coordinates remain unchanged.

### Task 4: Review and differential verification

**Files:**
- Modify: `docs/HANDOFF.md`
- Modify: `docs/BENCHMARKS.md`

- [ ] **Step 1: Run static checks locally**

Run `cargo fmt --all -- --check` and `git diff --check`; do not compile locally.

- [ ] **Step 2: Dispatch independent code review**

Review the exact diff for generator-order, twist-order, invalid-ID precedence, packet numbering, and accidental legacy materialization.

- [ ] **Step 3: Submit HPC gates**

Submit focused real-group, block debug/release, the fat unipotent corpus, and the full 360-fixture `pipeline_swap_diff.sbatch`, each from its own clean detached worktree and with benchmark fields enabled.

- [ ] **Step 4: Record evidence**

Record job IDs, PASS counts, source-state verification, wall time, and peak RSS. Claim a performance change only if repeated benchmark evidence exceeds run variance; otherwise record this as a correctness/memory migration.

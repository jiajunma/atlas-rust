# InvolutionTable Cayley Compact Lookup Implementation Plan

> For agentic workers: REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox syntax for tracking.

Goal: Make InvolutionTable::cayley use the record-owned compact WeylElt as its production input and retain legacy multiplication only in tests.

Architecture: Add a small internal compact lookup that copies record.element, applies CompactWeyl::inner_left_mult, and probes compact_index. Keep public error ordering and Ok(None) semantics unchanged; the existing index owns packed/full key handling.

Tech Stack: Safe Rust, CompactWeyl, existing DedupIndex, HPC SLURM gates.

---

### Task 1: Specify compact Cayley behavior

Files:
- Modify: crates/atlas-real-group/src/involution_table.rs

- [ ] Add b2_compact_cayley_matches_legacy_for_all_records comparing the compact lookup contract with WeylElement::multiply for every stored B2 record and generator, including None targets.
- [ ] Assert invalid source IDs are rejected before invalid generators, and valid sources reject out-of-range generators with the exact bound.
- [ ] Submit the focused HPC test and confirm the only failure is the missing compact lookup API.

### Task 2: Implement the compact lookup

Files:
- Modify: crates/atlas-real-group/src/involution_table.rs

- [ ] Add the minimal internal helper that validates source and generator, copies record.element, applies inner_left_mult, and probes compact_index.
- [ ] Make public cayley delegate to the helper without reading legacy_element, reflection, or image_permutation.
- [ ] Keep lookup, record construction, and the nonpacked fallback unchanged.

### Task 3: Verify and record

Files:
- Modify after verification: docs/HANDOFF.md
- Modify after verification: docs/WEYL_ELEMENT_MIGRATION_PLAN.md
- Modify after verification: docs/BENCHMARKS.md

- [ ] Run local formatting and diff checks only.
- [ ] Run the focused Weyl/InvolutionTable/KGB HPC gate in debug and release.
- [ ] Run the exact-commit full differential and fat unipotent benchmark; record seconds, peak RSS, ratios, and source-state verification.
- [ ] Obtain spec and code-quality review before promoting the slice.

# Weyl Element C++ Transducer Migration Plan

## 1. Goal

Migrate the Atlas-Rust Weyl element implementation toward the upstream Atlas
C++ design:

- `WeylElt` is a fixed-size, allocation-free piece array;
- transducers, generator ordering, piece words, and lengths live in shared
  group context;
- multiplication, inverse, simple left/right multiplication, longest-element
  construction, and canonical words use the compact transducer path;
- root permutations, PyCox-style simple-root images, and lattice matrices are
  materialized only when required;
- Atlas-visible numbering, ordering, and output remain unchanged.

The PyCox `coxelm` representation is not the primary group-operation
representation. It remains a possible derived cache or index representation.

## 2. Current Baseline

The repository already contains two relevant layers:

- `crates/atlas-real-group/src/weyl_transducer.rs` contains `WeylElt = [u8; 8]`,
  transducer tables, internal/external generator maps, piece words, and compact
  enumeration;
- `crates/atlas-real-group/src/weyl_element.rs` stores a Weyl element as a
  boxed forward root permutation plus its inverse, and performs most operations
  by allocating and traversing both arrays;
- `crates/atlas-real-group/src/weyl.rs` stores lattice actions as two matrix
  values and is the matrix-level representation;
- `crates/atlas-real-group/src/root_system.rs` caches simple-reflection root
  permutations.

The key architectural mismatch is that the compact transducer layer is used
for some enumeration, while the permutation-owning `WeylElement` remains the
main representation for many real-group and KGB operations.

The upstream C++ implementation instead uses:

```text
WeylGroup context + fixed-size WeylElt value
```

The value contains only parabolic-subquotient pieces. Group operations mutate
that value using shared transducer tables.

## 3. Phase 0: Freeze the Baseline and Behavior Contract

### Work

1. Pin the Rust commit, Rust toolchain, C++ oracle commit, compiler, and
   machine/partition used for measurements.
2. Establish Weyl-focused workloads covering:
   - A2, B2, G2;
   - A3, D4, F4;
   - D6 and E6;
   - E7/E8 KGB or involution workloads on the `fat` partition.
3. Record for every workload:
   - Rust/C++ output or event-stream hash;
   - wall time;
   - peak RSS;
   - Weyl-group order;
   - longest element;
   - inverse, left-descent, right-descent, and canonical-word results.
4. Add Weyl-layer property tests for:
   - `w * identity = w`;
   - `identity * w = w`;
   - `w * inverse(w) = identity`;
   - Coxeter braid relations;
   - simple left/right multiplication length changes of `+1` or `-1`;
   - reduced-word reconstruction;
   - canonical-word agreement with upstream ordering.

### Gate

A baseline report with wall time and RSS must exist on HPC before the primary
representation is changed. Small local checks do not replace this baseline.

## 4. Phase 1: Complete and Freeze the CompactWeyl API

### Work

Complete the C++-aligned operations in `weyl_transducer.rs`:

- `identity()`;
- `inner_gen()`;
- `longest()`;
- `max_length()`;
- `order()`;
- `length(&WeylElt)`;
- `inner_mult(&mut WeylElt, generator)`;
- C++-style local `inner_left_mult(&mut WeylElt, generator)`;
- in-place `multiply(&mut WeylElt, &WeylElt)`;
- `inverse(&WeylElt)`;
- `canonical_word(&WeylElt)`;
- `piece_key(&WeylElt)`.

### Requirements

1. `WeylElt` remains `Copy`, `Eq`, `Hash`, and `Ord`. It must not contain a
   `Vec`, `Box`, `Arc`, or matrix.
2. `inner_left_mult` follows upstream `min_neighbor`/`min_star` logic and only
   updates the affected piece interval. It must not materialize a root
   permutation.
3. `mult_by_piece` must not allocate a temporary `Vec` per call. Use a fixed
   stack buffer or precomputed piece words.
4. Length changes come from transducer piece lengths or shift direction, not a
   full root scan.
5. External/internal generator conversion is owned by `CompactWeyl`; callers do
   not perform their own reordering.
6. `longest()` directly selects the maximum valid piece in every transducer,
   matching upstream `WeylGroup::longest()`.

### Tests

- Unit tests for A2, B2, G2, D4, and F4;
- randomized word and multiplication tests;
- C++ oracle captures for order, longest, inverse, canonical word, and piece
  ordering;
- `size_of::<WeylElt>()` checks and allocation instrumentation to verify that
  element values do not allocate.

### HPC Gate

- compact preflight compiles successfully;
- all Weyl-focused tests pass;
- compact enumeration equals matrix-enumeration as a set;
- at least one E6 workload is no slower than the current baseline and remains
  output-identical.

## 5. Phase 2: Establish Explicit Materialization Boundaries

Add explicit conversions without immediately migrating every caller:

```text
WeylElt --materialize--> CoxElm/simple-root images
WeylElt --materialize--> full root permutation
WeylElt --materialize--> WeylAction/lattice matrix
```

Recommended responsibilities:

- `CompactWeyl::materialize_coxelm`: produce images of simple roots only;
- `RootSystem`/`CompactWeyl`: produce a complete root permutation;
- `WeylAction`: produce the lattice action;
- materialization functions have explicit names and are not hidden inside
  ordinary compact operations.

### Compatibility Strategy

Temporarily retain `WeylElement` as a compatibility facade while callers are
migrated. The old permutation implementation remains an independent oracle for
conversion tests, not the primary hot-path representation.

Verify that:

- compact-to-permutation equals the current implementation;
- compact-to-matrix equals `WeylAction` composition;
- simple-root images agree with the PyCox/rustcox `CoxElm` definition;
- full permutation inverse and group laws remain correct;
- rank-zero, reducible groups, and B/C/D internal reversal are covered.

### Tests

- random finite-type elements converted between all three representations;
- complete E6 compact/materialized set comparison;
- sampled E7/E8 conversion checks on HPC;
- explicit tests for datum identity and generator ordering.

### Current Progress

- `CompactWeyl::materialize_action` is now an explicit compact-to-lattice-action
  boundary. It composes cached simple-reflection actions from the compact piece
  word and does not alter the compact element.
- The existing matrix-enumeration test uses this API and passes.
- B3/C3/D4 exhaustive left-multiplication tests compare compact materialization
  against matrix multiplication and pass.
- `materialize_coxelm` and full root-permutation materialization remain to be
  added before production callers are migrated. The existing permutation-based
  `WeylElement` remains the oracle for those conversions.
- HPC preflight job 3634955 did not reach compilation/tests because the dirty
  snapshot failed the initial full-workspace format check. It is not a Weyl
  correctness result; the next HPC gate must use a clean snapshot or a focused
  Weyl-only runner.

## 6. Phase 3: Migrate InvolutionTable and Cartan Classification

These are the first production callers because they already have compact
enumeration and permutation-level optimizations.

### Work

1. Store the primary Weyl value in involution records as `WeylElt`.
2. Check twisted involutions using compact inverse and twist operations.
3. Use the compact piece key for primary indexing and ordering.
4. Keep root-involution data separate from the compact Weyl value.
5. Use compact elements for Cartan candidate scans, twisted-conjugacy
   partitioning, and orbit sweeps.
6. Materialize root actions only for root involution construction, theta/root
   image operations, or external output.

### Invariants

- Atlas involution and Cartan numbering is unchanged;
- piece ordering agrees with the C++ `WeylElt::operator<` behavior;
- root-level classification is unchanged;
- cache keys include complete datum identity, not only the Cartan matrix.

### Tests

- existing involution, Cartan, and real-form differential fixtures;
- full A2/B2/G2/F4/D6/E6 coverage;
- E7/E8 workloads on `fat`;
- old/new intermediate record comparison for compact key, length, and root-action
  hashes.

### Current Progress

- `InvolutionRecord` now stores `WeylElt` as its primary Weyl value; the
  parallel table-level compact vector has been removed. Compact length,
  descents, words, twists, lookup, and ordering read the record value.
- The seed is encoded through the checked `encode_element` boundary.
- Every newly discovered cross-action neighbor is computed through compact
  `inner_mult` plus C++-style local `inner_left_mult`. Record insertion then
  compares all simple-root images against the legacy composed permutation in
  debug and release builds, without allocating a word or full permutation.
- `legacy_element: WeylElement` remains as the compatibility/oracle field for
  downstream consumers not yet migrated. Removing it is the remaining memory
  step; the compact-primary move alone intentionally does not change RSS.
- Exact-commit focused job 3646031 passed Weyl 62/62, InvolutionTable 16/16,
  and KGB 11/11 in both debug and release. Unipotent differential 3646032
  matched, and full pipeline differential 3646033 passed 360/360 fixtures.
- `minimal_torus_part` now uses compact identity and ordered left-descent table
  operations instead of reading `record.weyl_element()`. Exact-commit focused
  job 3646081 passed Weyl 62/62, InvolutionTable 17/17, and KGB 11/11 in both
  debug and release; minimal-torus job 3646082 passed 4/4; full pipeline
  differential 3646083 passed 360/360 fixtures with zero pending cases.
- GlobalKGB twisted commutation now runs on the compact record value. Exact
  focused job 3646940 passed Weyl 62/62, InvolutionTable 18/18, and KGB 11/11
  in debug and release; unipotent differential 3646938 matched at 1.748x wall
  time and 4.204x RSS; full pipeline differential 3647072 passed 360/360.
- GlobalKGB printed canonical involution words now reduce the compact record
  directly. Exact focused job 3647184 passed Weyl 62/62, InvolutionTable
  19/19, and KGB 11/11 in debug and release; GlobalKGB job 3647185 passed 4/4
  in both profiles; unipotent differential 3647186 matched at 1.786x wall
  time and 4.207x RSS; full pipeline 3647187 passed 360/360.
- The next production consumer is BlockGraph dual-packet pairing. Keep its
  `longest_action` budget/error gate, but replace full-permutation packet keys
  with compact table IDs. External APIs keep explicit materialization
  boundaries until their remaining consumers are migrated.
- BlockGraph dual-packet pairing now resolves compact dual IDs and stores only
  packet positions. Exact focused job 3647607 passed Weyl 62/62,
  InvolutionTable 20/20, and KGB 11/11 in debug and release; block job 3647608
  passed 60/60 in both profiles; unipotent differential 3647609 matched at
  1.773x wall time and 4.206x RSS; full pipeline 3647610 passed 360/360.
- The next migration target is `ExtParam`: make its involution ID primary and
  materialize a legacy permutation only at explicit compatibility boundaries.
  Removing `InvolutionRecord::legacy_element` follows after those callers are
  migrated.
- The compact final-only left-word prerequisite landed in `e1848cd`: focused
  job 3647624 passed Weyl 62/62, InvolutionTable 21/21, and KGB 11/11 in both
  profiles. Unlike repeated table `cross`, it permits non-table intermediate
  products and resolves only the final reflected involution. Use it for the
  arbitrary reflection words in `ExtParam`; use `cross` for simple twisted
  conjugations.

## Current Frontier: ExtParam and Cayley

- ExtParam is now ID-primary on ab21ff5 (fixture repair commits through
  28ec875): its production methods no longer own or mutate a WeylElement.
  Exact jobs 3650615 and 3650630 passed the focused debug/release gates, and
  full pipeline job 3650652 passed 360/360 with complete source-state checks.
- The current migration slice is InvolutionTable::cayley. Its compact
  implementation is isolated at ea2f23c; it preserves source-before-generator
  validation and partial-table None, while removing legacy_element from the
  Cayley hot path. Jobs 3651599/3651600 are the exact-commit focused and
  full-pipeline gates.
- The follow-up full-key BFS fallback is implemented at 798838e, with forced
  Full-key coverage in 38426d1. It computes the cross neighbor compactly and
  only materializes a legacy permutation on a compact-index miss. HPC target
  job 3651646 is the final focused test; no compatibility or performance claim
  is made until that result is collected.
- Compact identity resolution is staged in `97789af`/`2358259`: the table now
  exposes an internal `identity_id` lookup, and GlobalKgb, KgbGraph, and both
  RealFormSeed builders use it instead of constructing an identity
  `WeylElement`. HPC focused verification is still pending.
- The identity and full-key fallback follow-ups are now focused-green
  (`3652360` and `3652361`). `block_fiber_check` is also compact-ID
  primary at `4702f1e`: the target table resolves the source compact word
  through its own dual twist and returns a target `InvolutionId`. The first
  cross-table GREEN fixture incorrectly reused the source inner class;
  local equivalent `d99e5a0` (HPC snapshot `3822f6f`) repairs it with actual A2/B2 dual inner classes. Focused jobs
  `3652749`/`3652750` and targeted differential `3652816` pass.
  Full 361-fixture job `3652761` remains the pending whole-suite gate.
- After the full gate, the next low-risk language-boundary slice is the four
  `canonical_involution_expr(record.weyl_element())` printer consumers in
  `domain_builtins.rs`. They already have the table-level compact counterpart
  `weyl_canonical_involution_expr(InvolutionId)` and are exercised by the
  KGB/block printer fixtures. The separate `print_block` elected Weyl-word
  path and `print_blocku` support-set path still need a compact-word API or
  an explicitly documented materialization boundary.

## 7. Phase 4: Migrate Tits, KGB, and Remaining Real-Group Callers

Migrate in this order:

1. `tits_element.rs`: descents, simple multiplication, and twisted actions;
2. `kgb_graph.rs` and `global_kgb.rs`: frontier expansion, cross/Cayley edges,
   and record interning;
3. `real_weyl.rs`, `inner_class.rs`, and `ext_param.rs`: Weyl words,
   conjugation, and root images;
4. `block.rs`, `block_modifier.rs`, and `locator.rs`: Weyl parameters and
   reduced words;
5. `domain_builtins.rs`: convert to compatibility output only at the language
   boundary.

### Migration Rules

- compact operations accept `&CompactWeyl` plus `WeylElt` values;
- no `from_permutation` in hot loops;
- no per-edge or per-record inverse permutation construction;
- root image operations go through explicit materialization/cache paths;
- shared context may use `Arc`, but `WeylElt` remains a plain value;
- any retained full action must document why it is required.

Each module is migrated independently. For every module:

1. run focused unit/property tests;
2. run the relevant Rust/C++ differential fixtures;
3. record timing and RSS;
4. only then migrate the next module.

## 8. Phase 5: Remove the Old Primary Representation

Only after Phases 3 and 4 are differential-stable:

- remove forward/inverse ownership from the primary Weyl value;
- delete `build_data` and `PeelBuffers` if they serve only the old path;
- make full root permutation an explicit materialization result;
- evaluate contiguous arenas or caches for materialized permutations;
- evaluate `u8` transducer tables where the state/output bound permits it;
- benchmark `mult_by_piece`, local left multiplication, inverse, and canonical
  word independently.

The old permutation code should remain temporarily in a test-only or diagnostic
form until all conversion and differential tests are stable.

## 9. Parallelism and HPC

### Single-node parallelism

Parallelize only independent, pure work:

- compact element to root permutation;
- compact element to `WeylAction`;
- twisted-involution predicates;
- Cartan candidate/orbit calculations.

Do not thread a single compact transducer operation. The operation is too small,
and synchronization would cost more than it saves.

### Multi-node HPC

Use SLURM arrays for independent:

- fixtures;
- group types;
- real forms;
- parameters;
- old/new implementation comparisons.

Do not initially MPI-parallelize one KGB BFS. If KGB parallelism becomes
necessary, use:

```text
frontier level
  -> thread-local expansion
  -> deterministic merge/sort/dedup
  -> sequential canonical intern
```

This preserves Atlas-visible numbering and makes failures reproducible.

## 10. Acceptance Metrics

Every phase report must include:

- Rust/C++ wall-time ratio;
- Rust/C++ peak-RSS ratio;
- allocation count/bytes when instrumentation is available;
- `size_of::<WeylElt>()`;
- number of materialized permutations;
- output/event-stream hash;
- thread count and parallel efficiency;
- commit, toolchain, fixture, and HPC job ID.

The target is not a single speedup number. The required outcomes are:

1. compact microbenchmarks are no slower than the current permutation path;
2. E6 classification and enumeration use substantially less memory;
3. E7/E8 KGB/involution workloads show a sustained RSS reduction;
4. Weyl-heavy Rust/C++ ratios improve on repeated HPC runs;
5. all Atlas differential fixtures remain byte-identical.

## 11. Risks and Rollback

- **Generator-order mismatch:** compare internal/external maps, piece keys, and
  canonical words against C++ before debugging downstream behavior.
- **Compact/action mismatch:** keep the old matrix/permutation implementation as
  a conversion oracle until the migration is complete.
- **Observable numbering changes:** all parallel or hash-based discovery must
  use deterministic sort and commit.
- **Rank greater than 8:** retain an explicit fallback; do not truncate pieces.
- **Context identity errors:** materialization/cache keys must include complete
  datum identity.
- **Performance regression:** keep each phase isolated and accept changes only
  after HPC benchmark comparison.
- **Compatibility regression:** roll back the caller migration while retaining
  the verified compact API.

## 12. Deliverables

1. This plan in `docs/WEYL_ELEMENT_MIGRATION_PLAN.md`;
2. C++-aligned `CompactWeyl` API and tests;
3. explicit materialization boundaries and conversion tests;
4. staged InvolutionTable, Cartan, Tits, KGB, and real-group migration;
5. per-phase HPC differential and benchmark reports;
6. updates to `docs/BENCHMARKS.md`, `docs/HANDOFF.md`, and related design
   documents after each verified phase.

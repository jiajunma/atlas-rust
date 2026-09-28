# atlas-rust agent guide

Rust reimplementation of the [Atlas of Lie Groups](https://github.com/jeffreyadams/atlasofliegroups).
The compatibility target is the Atlas language and its observable behavior,
not a source-level C++ translation.

## Hard rules

1. **All testing and builds run on HPC compute nodes.** The user's latest
   mathematical-validation directive supersedes the older local-small-check
   convention. Use local execution for reading, editing, Git and hashing;
   use the login node for Git/source staging and dependency acquisition only.
   Submit Cargo builds/checks/tests, fixture/verifier execution, Atlas/CWEB
   differential jobs and benchmarks through SLURM. The original Atlas
   executable remains an HPC oracle. GitHub Actions is allowed only when its
   workflow is the requested verification environment.
2. **The original Atlas executable is the language oracle.** The upstream
   repository and its generated CWEB output define reference behavior. Do not
   infer undocumented semantics from what is convenient to implement.
3. **Differential tests precede implementation claims.** A feature is not
   supported until an HPC run compares it with the reference on accepted and
   rejected inputs and stores the result artifact.
4. **Generated files are disposable.** Do not hand-edit CWEB-generated C/C++
   or parser-generated files. Generate derived artifacts in HPC job folders.
5. **No unsafe Rust in the core.** FFI and platform-specific code must be
   isolated, reviewed, and justified.
6. **Preserve user changes.** Never reset unrelated work. Use `apply_patch` for
   hand edits and conventional commits (`feat:`, `test:`, `fix:`, `docs:`,
   `chore:`).
7. **Every Rust calculation error requires a regression test.** Whenever an
   incorrect Rust mathematical result is discovered, add a reproducing case
   to the test library (`tests/math` or the relevant `tests/fixtures` suite)
   before fixing it. Preserve the triggering input and an independently
   justified expected result; never use the faulty Rust output as the golden.
   Run the regression and original-Atlas comparison on HPC, retaining evidence
   that it fails before the fix and passes after it. If the fix is deferred,
   keep the case explicitly tracked as failing; do not remove or weaken it.

## Repository map

- `crates/atlas-core`: lexer, parser, AST, values, evaluator, domain traits,
  diagnostics, and compatible file primitives.
- `crates/atlas-cli`: batch and interactive command-line behavior.
- `tests/fixtures`: Atlas source programs, expected events, and negative cases.
- `tests/reference`: oracle metadata and checksums; large outputs stay on HPC.
- `hpc`: SLURM jobs and synchronization helpers.
- `docs`: compatibility contract, language matrix, design, migration gates,
  and HPC operations.

## Required workflow

1. Read `docs/COMPATIBILITY.md`, `docs/LANGUAGE.md`, and `docs/DESIGN.md`.
2. Add or update a fixture and reference expectation first.
3. Sync to HPC and run the smallest relevant differential job.
4. Implement the smallest module owning the behavior.
5. Run the stage's HPC test and inspect its report.
6. Commit source, fixtures, and report metadata; never commit unverified local
   output.

## Working conventions (user directives, 2026-08-04)

1. **Submit, do not wait.** Once an HPC job is submitted (`sbatch`), move on
   to the next task immediately. Never block the local loop on a pending job;
   results are collected later in batches (a periodic poll is fine, but the
   default is to keep producing work).
2. **Heavy fixtures run on HPC with a generous timeout.** E7 and similar
   Weyl-heavy work go to the `fat` partition with a large `--timeout`
   (e.g. `TIMEOUT=1200`); the `cpu` partition's per-task 8G limit OOMs on
   E7. `#SBATCH` lines do not expand env vars — override via sbatch CLI flags
   (`--partition=fat --mem=32G --export=ALL,TIMEOUT=1200`).
3. **Benchmark every differential comparison.** The drivers
   (`hpc/pipeline_swap_diff.py`, `hpc/reference_capture.py`) record wall
   time AND peak RSS per fixture for both the Rust CLI and the oracle: GNU
   `time -v` on Linux (exact), `getrusage` fallback on macOS (approximate,
   cumulative child peak). Fields: `seconds`, `maxrss_kb`,
   `maxrss_approximate`. Keep this benchmark data in every report.
4. **Keep iterating until the whole Atlas is ported to Rust.** Do not stop
   at one milestone; after a fixture/commit lands, immediately pick the next
   builtin or coverage extension (see `docs/REMAINING_BUILTINS.md`).
5. **Record conventions and puzzles.** Anything a future agent must know
   (blockers, root causes, disproven hypotheses, HPC quirks) goes into
   `docs/REMAINING_BUILTINS.md` and `docs/HANDOFF.md`; do not rely on
   session scratch files for project state.
6. **Prefer HPC-side Git synchronization.** The HPC login node can access
   GitHub. For published source, fetch there and materialize the exact pinned
   commit in a fresh job stage; never pull into the shared dirty development
   checkout. For an unpushed candidate, transfer a small checksummed patch
   against an already pinned HPC baseline and build its source archive on HPC.
   Verify resulting source hashes; do not upload whole source/target trees
   when the baseline is already available. Do not push unverified code merely
   as a transport shortcut. Builds and tests still belong on compute nodes.
7. **Evaluate parallel acceleration with controlled A/B tests.** The user's
   2026-09-28 extension requires investigating computational parallelism and
   selecting effective changes by HPC evidence. First require full output
   equality with the original and between serial/parallel candidates. Hold
   source, input, compiler, node and resource limits fixed; alternate arm order,
   repeat fresh processes and record wall time, CPU time, peak RSS and thread
   settings. Distinguish Rust serial-to-parallel scaling from Rust-versus-C++
   speedup. Failed or mathematically unequal runs yield no accepted speed ratio.
   Prefer60-600s workloads where possible; small tests are correctness controls.
   Existing math-suite observations explicitly force RAYON_NUM_THREADS=1;
   never present those timings as measurements of the multicore configuration.

## Verified repair guard

### Type imports must carry pending assignments and their scope

- Matching foundation3834447 passes38 type tests/3 coercion tests and CLI
  check. Structural matching must expose the requested components while
  preserving repeated-variable constraints and recursive nominal identity.
  Direct function calls apply argument substitutions to the result; a failed
  two-sided trial restores both expression bodies and both assignment scopes.
- Source-backed scope foundation3834389 passes27 type tests (nine new),
  three coercion tests and CLI check. Import both the type body AND shifted
  pending substitutions; adding fresh empty slots alone discards constraints.
- Bake pending substitutions before raising the rigid-variable threshold.
  Pack tuple components with each component's own threshold, then freshen
  their free ranges separately. Reuse the overload argument's variable shift
  when substituting its result, and clear/restore failed candidate trials.
- Preserve unused declared constructor slots while the scope is clean.
  These are internal contracts, not generic-language acceptance: the active
  parser/analyzer migration and global/local wrong-assignment repairs remain.

### Parallel scaling is not removal of excess mathematical work

- E7 A/B3834321 and independent review3834377 verify all four whole-output
  comparisons: Rayon4 is2.958576x faster than Rayon1 with18.54% more peak RSS.
  Yet both Rust arms still consume about69-70 CPU seconds; Rust4 remains
  about17.7x slower than original. Keep algorithmic work reduction separate
  from serial/parallel scaling, and profile before attributing the CPU gap.
- Recheck full streams across rounds, not just within each pair. Bind the
  scheduler job/node/CPU count, affinity, binaries, source, GNU time metrics
  and alternating schedule. Reject incomplete/failing trials even if their
  reported summary looks fast; test the reviewer against those failures.
- Scoped four-thread selection does not justify a global thread default,
  D8 limit increase or claims about unitarity/Hodge/FPP/AV/cycles. See the
  indexed parallel A/B slice for the exact retained experiment and review.

### Structural specialisation must expose the receiver's components

- Foundation3834296's same test fails with the earlier Applied implementation
  and passes after: a read-only can_specialise check returned true but left the
  receiver Applied instead of the Row the caller needed to inspect.
- Follow axis-types.w:942-979: expand/refine the receiver on successful
  structural specialisation. Preserve the distinct-recursive-name termination
  guard and use disjoint before/after build targets. This internal fix is not
  generic-language support; the active analyzer still needs its scope migration.

### Type abstraction is not conventional lexical shadowing

- Generic capture3834266 accepts nested abstractions with distinct T/S names,
  but rejects an inner `any_type T` while outer T is still a TYPE_VAR token:
  the declaration grammar expects IDENT. Duplicate constructor formals in one
  not-yet-installed list are a different case and remain accepted.
- The same capture shows that polymorphic LOCAL empty rows are implicitly
  constant too (`axis.w::thread_bindings`), not just global bindings. Preserve
  both failing regressions, concrete [int] assignment controls and mutation of
  fixed-T locals; neither a global-only nor an empty-value-only fix is sound.
- Discovery fixtures must reach the intended semantics. Adjacent `>=` is one
  operator, and named struct specifications require at least two fields. Keep
  the rejected discovery inputs and add valid companions; do not interpret
  these syntax failures as evidence about unused type-parameter substitution.

### Missing upward Cayley images have a zero polynomial, not a missing descent

- E6 probe3833624 and build3833716's unchanged before test execute the
  imaginary-II `second image` panic. A partial block can contain only one
  upward Cayley image; original KL_y(UndefBlock) resolves to its zero slot.
- Preserve required downward links. Only optional upward imaginary-II
  contributions are zeroed. The same integer/half-scale test passes after;
  independent review3833739 verifies complete E6 KL output and retained
  F4 coefficient/history cases (14 mathematical matches,1 rejection).
  This does not establish general associated-cycle or E7 correctness.

### Discover polymorphic contracts instead of assuming conventional rules

- Current-original capture3833740 rejects an int call ambiguous between a
  generic identity and a concrete int overload; do not assume the concrete
  overload wins. It also ACCEPTS duplicate type formal names. Preserve the
  discovery inputs and investigate substitution before imposing uniqueness.
- Constructor-arity rejection lacks the literal `Type error` heading.
  Bind rejection expectations to the actual diagnostic and failing command;
  a Rust syntax error before the declaration is not a matching type rejection.
- The one-CPU capture submission rejected8GiB and reported a4GiB allowance.
  The same small capture completed with a2GiB CLI override. This does not
  establish a partition-wide4GiB cap: the two-CPU repair job used8GiB.
  Check the actual requested CPU/memory combination and scheduler response.

### Preserve the polymorphic versus concrete assignment distinction

- Capture3833758 shows original rejecting `set x=[]; x:=[1]` as constant
  while Rust accepts it. Probe3833788 executes the unchanged-runtime unit
  failure (`constant=false rejected=false`), after permitting concrete `[int]`.
- R3 capture3833787 confirms the concrete `[int]` control matches completely.
  Original global.w:992 derives implicit constness from the binding's type
  scheme. Preserve that type-level rule, including fixed versus polymorphic
  variables; do not make every empty list or every type variable constant.
  The regression remains explicitly failing until a real after-pass and
  full original-backed comparison exist. See the indexed generic-language slice.

### Compare archive paths in extracted-tree coordinates

- Review3833635 failed before examining mathematics: tar stored ./crates/...
  but file_manifest used crates/.... Bytes and archive pins had not changed.
- Normalize safe relative tar member names before exact file/hash comparison;
  reject traversal, absolute names, unsupported links and duplicate canonical
  files. Keep regression tests for both spelling forms and rejected aliases.
  Re-review existing frozen results with separately pinned checker code;
  never alter their source archives, reports or numerical comparison rules.

### Endgame search failure is not an absent cross

- F4 probe3833589 computes the same wrong P(4,334) in cold and history
  contexts. In first_endgame_pair, upstream kl.cpp:318-340 distinguishes
  an absent cross (valid pair with no t needed) from a defined cross with
  no suitable t (continue searching s). Rust had both branches wrong.
- Build3833612's unchanged coefficient unit fails before and passes after
  correcting only those branches. Review3833712 also verifies complete F4
  history output, including every coefficient, while retaining the separate
  E6 failure. Preserve both checks; one coefficient alone cannot establish
  correctness of the full table or downstream associated cycles.

### Keep patch backup files outside frozen source archives

- E6 probe3833616 was rejected before compilation because an offset GNU
  patch application created domain_builtins.rs.orig, changing the file set.
- Generate patches against the exact pinned base and use
  --no-backup-if-mismatch when staging into a fresh disposable tree. Keep
  the baseline archive as the recovery copy and retain exact file-set checks;
  never weaken the source guard to ignore accidental backup files.

### Revalidate old language exclusions after an oracle upgrade

- Latest original7e1b958c declares ANY_TYPE in parser.y and uses generic
  constructors and any_type scopes in basic.at. The old Rust lexer unit
  `oracle_non_keyword_is_scanned_as_an_identifier` asserts the opposite
  for any_type; its historical expectation does not define the new oracle.
- Keep the pinned old evidence, but use current original-backed positive
  AND negative fixtures to migrate lexer, parser, type-variable scope and
  overload/type-constructor semantics together. Do not preprocess away
  generics or replace upstream scripts merely to pass high-level math tests.

### Equal KL index matrices do not prove equal polynomials

- F4 history review3833539 and diagnosis3833564 find identical parameters
  and336x336 index matrices, yet pool entry102 at(4,334) differs:
  original[0,0,2,3,3,2], Rust[0,0,4,6,5,3]. The other175 pool entries match.
- Always compare complete coefficient vectors as well as parameter/index
  data. Internal cold/warm equality can hold while both answers are wrong.
  Preserve the full-output fixture and test the affected polynomial in both
  cold-full and partial/full history contexts; do not bless matching shapes.

### Fundamental-fiber coordinates are not grading-shift positions

- E6 probe3833407 reproduces the external-form constructor failure without
  full Weyl enumeration. The fundamental adjoint fiber basis is e1/e3, but
  e3 flips several imaginary-subsystem simple roots; its FIRST flipped root
  has ambient-simple coordinates[0,0,1,1,1,0], not a datum-simple root.
- Never infer a fiber basis coordinate from the first flipped grading root.
  For a distinguished diagram involution, exchanged pairs vanish in the
  fiber quotient and fixed fundamental-coweight coordinates survive. Verify
  the actual basis representatives and their ascending coordinate order.
- Original FormNumberMap uses the PARTITION overload of specialGrading,
  not its Fiber overload. Preserve the maximal-popcount/highest-index
  election and require unchanged-unit after-pass plus full original-backed
  E6 KGB/KL comparison before accepting an external-numbering repair.

### Read template defaults for root/coroot operations

- `InnerClass::int_item` calls `additive_closure` without a template argument;
  upstream `rootdata.h` defaults `for_coroots=true`. The Rust locator wrongly
  ported root sums. In B/C/F/G, root and coroot closures need not coincide.
- Probe3832726 executes two failing regressions: B2 closure gives4 instead
  of8 elements; F4 half-scale locator fails integral-image positivity.
  Preserve these tests, use exact coroot evaluations as an independent check,
  and require after-pass plus original-backed full-output comparison.
- `combine_roots`'s boolean selects subtraction, NOT root versus coroot.
  Do not flip that flag or remove positivity checks as a shortcut.

### Undefined links in a partial KL block are descent-specific

- F4 case80/job3832508 panics at `cross of extremal`. The unchanged-runtime
  probe3832534 isolates `x=263, y=278, s=2`, with descent `RealTypeII`.
- A real-II cross can leave a downward-closed partial block. Original
  `blocks.cpp` explicitly permits `UndefBlock` there, and `KL_pol` evaluates
  its contribution as zero. A complex descent or inverse Cayley descent,
  in contrast, must remain inside the interval.
- Do not eagerly unwrap every extremal's cross, silently zero every missing
  link, or require cross-closure of a partial block. Preserve the descent
  distinction and compare full KL parameters/matrices/pools, including
  partial/full/partial cache histories. Keep the original-backed regression
  and before-panic/after-pass evidence; no acceptance from avoiding a panic
  alone. See `docs/slices/math_failures_2026-09-28.md`.

### Isolate Cargo outputs for before/after archive builds

- HPC job `3832260` compiled the before source and failed the intended D4
  assertion, but the after command finished in 0.7 seconds and executed the
  same old word/binary despite the verified source containing the reversal.
  Both archive trees preserved timestamps and shared `CARGO_TARGET_DIR`.
- Never share Cargo target directories between the before and after trees in
  a regression proof. Use separate fresh `target-before` and `target-after`
  directories, and record their paths with each command. An exit code alone
  cannot establish which source was executed.
- Keep failed build artifacts. The isolated replacement `3832301` gives
  before-assertion-fail/after-pass. Differential review `3832333` confirms
  the D4 repair and E6, but E7 still times out; do not equate the unit pass
  with all-FPP acceptance. See the R2 submission/review JSONs in
  `tests/reference/hpc` and the separate E7 long-run receipt.

### Bare-core mathematical fixtures must use bare-core operations

- R10 jobs `3832308`/review `3832312` showed that equality of `[Param]` and
  complete KL tuples requires script overloads, even in the original.
  Loading latest `basic.at` would mask the independent kernel regression
  behind Rust's unrelated generic-language gap.
- Compare Param/vec/int entries explicitly, retain the full printed records,
  and keep the failed fixture artifacts. R11 review `3832340` then confirms
  four original-pass/Rust-fail regressions without changing either engine.

### Owned `LatticeInvolution` builders

- Root cause: a migration from borrowed to owned involution input left a
  read-only helper call unborrowed, which failed only at Rust type checking.
- Diagnostic: use the frozen `real_group_preflight.sbatch` package compile;
  job `3463647` reported the concrete mismatch before tests ran.
- Prevention: audit every helper call before the final move into the fiber
  model, then require a passing HPC preflight such as job `3463683`.

## Rustcox reuse

`rustcox` is a related but separate project. Its Coxeter, root-system, Bruhat,
Laurent-polynomial, W-graph, and canonical JSON modules may be adapted after
their APIs and semantics are wrapped by Atlas domain traits. Do not make the
Atlas language layer depend directly on rustcox internals.

## License

GPL-3.0-or-later. Reused code must have a compatible license and retain notices.

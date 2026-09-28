# atlas-rust agent guide

Rust reimplementation of the [Atlas of Lie Groups](https://github.com/jeffreyadams/atlasofliegroups).
The compatibility target is the Atlas language and its observable behavior,
not a source-level C++ translation.

## Hard rules

1. **Use local execution for small checks; use HPC for heavy work.** Small,
   bounded local checks such as `cargo check -p <crate>`, focused unit tests,
   formatting, and static analysis are allowed. Do not run long builds,
   full-workspace or large test suites, Atlas/CWEB differential jobs,
   benchmarks, or other resource-heavy work locally; submit those to XMU HPC.
   The original Atlas executable remains an HPC oracle. GitHub Actions is
   allowed only when its workflow is the requested verification environment.
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

## Verified repair guard

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

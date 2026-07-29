# Atlas Rust session handoff - 2026-07-29

This is the current continuation record for `/Users/hoxide/mycodes/atlas-rust`.
The user goal is an idiomatic, safe-Rust implementation of the Atlas language;
the upstream Atlas executable and generated CWEB sources remain the behavioral
oracle. No `unsafe` Rust is permitted in the core.

## Current commits and tree

- `c6d5d6a feat: activate typed Atlas session pipeline`
- `14818ab test: harden typed pipeline provenance`
- branch: `main`; the handoff update itself is the only intended change after
  those commits until it is committed.
- `origin/main` was behind these commits at the last local check. Push only
  after the documentation and local checks below are complete.

The typed pipeline is user-visible now. `session.rs` and `session_frame.rs`
convert and evaluate commands through `typed.rs`; the old dynamic `eval.rs`
module is deleted and no code path imports it. `session_fixture_tests.rs`
exercises the same session boundary used by the CLI.

## Verified local state

The following checks passed with Rust 1.90 and warnings denied:

- `cargo test -p atlas-core`: 145 tests passed, 0 failed.
- `cargo check -p atlas-core --lib --tests` with `RUSTFLAGS=-D warnings`.
- `cargo check -p atlas-cli` with `RUSTFLAGS=-D warnings`.
- `cargo fmt --all -- --check`, `git diff --check`.
- `PYTHONDONTWRITEBYTECODE=1 python3 -B -m unittest hpc/test_pipeline_swap_diff.py hpc/test_reference_capture.py`: 17 passed.
- `bash -n` for both new batch jobs and `bash hpc/test_source_state.sh`.
- `rg '\bunsafe\b' crates` returns no matches.

These are bounded local checks only. No full-workspace build, corpus run,
reference executable run, benchmark, or differential job has been run locally.

## What the B2 swap covers

The typed registry and evaluator currently cover the scalar and linear-value
surface already present in the repository: exact integers/rationals, strings,
booleans, tuples/lists, typed assignment and context persistence, overload
resolution, short-circuit conditionals, subscriptions, one-dimensional slices,
matrix/vector/ratvec crossings, RootDatum constructors, Cartan matrices, and
the exposed KGB constructor adapter. Session formatting now owns report lines,
void-type suppression, diagnostics, and `Bye.` output.

Two semantic repairs are part of the swap:

1. Balance accumulates conflicts and prunes earlier narrower candidates as
   later branches broaden the common type. Nested list/conditional failures
   retain their owner span, so only the balance that owns the offending branch
   may salvage it into a void row.
2. Raw Cartan inference tests all exact candidate matrices before trying a
   simultaneous permutation. This preserves canonical B2/C2 orientation while
   accepting non-symmetric relabelings such as B3.

The domain surface is intentionally incomplete. InnerClass/RealForm/KGB
renderings and relation semantics, stable KGB external numbering, primitive
`involution` constructors, `real_form(InnerClass,mat,ratvec)`, and later
math-layer overloads are pending. Do not describe the current KGB values as
compatible. The HPC pending list is explicitly limited to the selected
RootDatum/InnerClass/RealForm/KGB typed-pipeline scope; it does not enumerate
all future Atlas overloads.

## Differential evidence status

Checked-in oracle metadata and event fixtures are available under
`tests/reference/`. The domain-equality fixture is runnable only through its
RootDatum prefix (source lines 1-2); InnerClass/RealForm/KGB setup, displays,
numbering, and relations (lines 3-14) are recorded as pending rather than
being represented by placeholder Rust output. The selected constructor and
linear-value fixtures are ready for the typed swap job.

The provenance harness in `hpc/` now:

- validates declared versus detected commit and dirty-tree state before and
  after snapshot creation;
- freezes clean versioned input with `git archive <detected-commit>`;
- labels dirty/unversioned live-tree snapshots explicitly;
- binds `source_state.sh` to the Git blob for clean runs, or hashes the copied
  helper for dirty runs;
- checks Slurm spool script bytes against the frozen snapshot and clean Git
  blob; and
- writes a checksummed failure report if setup aborts before a normal report.

The report's snapshot scope is annotated after execution: clean runs say
`full tracked Git archive`; dirty jobs identify their live-tree scope. A prior
review found the stale fixed scope string in `reference_capture.py`; it was
replaced with a neutral pre-annotation value and the batch annotator now writes
the exact scope. This repair was verified by the 17 harness tests and shell
checks above.

The next HPC action is to sync a committed tree to XMU, verify the detached
checkout and clean state, and submit `hpc/pipeline_swap_diff.sbatch`. The last
attempt in this continuation timed out on direct SSH, and
`127.0.0.1:7897` was not listening, so there is currently no Rust-vs-Atlas
differential report for `c6d5d6a`.

## Immediate next slice: B3a

Follow the upstream parser and `docs/AXIS_CORE_DESIGN.md` rather than adding
ad-hoc syntax. The recommended bounded slice is non-recursive functions:

1. Add a fixture and obtain its raw upstream capture on HPC before claiming
   semantics. Start with typed lambda parameters, a generic call, a captured
   local, `return`, and identifier selector desugaring (`x.f` = `f(x)`).
2. Add the smallest AST/parser vocabulary (`Lambda`, `Return`, simple named
   parameters, and selector postfix), then typed closure creation and generic
   calls.
3. Represent closures with safe `Rc`/`RefCell` frame capture and manual
   equality using identity; apply the existing context-swap guard. Reject
   `return` outside a function during conversion.
4. Keep recursive functions, destructuring/const/discard patterns, operator
   selectors, definitions, loops, and full `set_type` syntax for later slices.

The exploration notes identify the likely ownership files:
`crates/atlas-core/src/syntax.rs`, `grammar.lalrpop`, `typed.rs`, `value.rs`,
and the existing frame implementation. Parallel work is appropriate for an
oracle/fixture task, a parser/AST task, and an independent Rust review, but
each worker must own disjoint files and preserve this shared tree.

## Standing workflow

For each substantive slice: read the upstream grammar/axis implementation,
add or update a fixture and reference metadata, run the smallest HPC
differential job available, implement the owning module, run bounded local
checks, request a Rust/code review, update this handoff with evidence, then
commit and push. Never hand-edit generated CWEB or parser output. Keep heavy
work on HPC and never infer undocumented behavior from convenience.

Historical designs and stage gates remain in `docs/AXIS_CORE_DESIGN.md`,
`docs/AXIS_CORE_TRACE.md`, and `docs/AXIS_LANGUAGE_TRACE.md`; this file is the
current state and should be updated whenever a verified repair or blocker is
found.

# Atlas-Rust handoff - 2026-07-29 (B6 case/counted-for verified)

This is the continuation record for `/Users/hoxide/mycodes/atlas-rust`.
The goal is source-compatible Atlas language behavior, with the upstream Atlas
executable and CWEB sources as the behavior oracle. The core remains safe Rust.

## Current state

- Branch: `main`.
- B3a non-recursive functions, B3b recursive functions / definition sugar,
  B3c parameter patterns, B3d selectors, B4 loops, B5 `set_type`, and B6
  case / counted-for are implemented and differentially verified; B11
  precedence needed no change and is verified as well. The exact commit is
  shown by `git log -1 --oneline`.
- B7 misc commands have frozen reference events (capture `3499657`); the
  B7 implementation is in progress.
- B8 user overloads (`3499692`, `3499705`), B9 file-command redirection
  (`3499747`), B10 fromfile/quit (`3500378`), and B12 runtime errors
  (`3500488`) have frozen references ahead of their implementations. The
  B11/B12 probes are retired.
- No uncommitted repository changes should remain after the handoff commit.

The typed session pipeline is active: `session.rs` and `session_frame.rs`
convert/evaluate through `typed.rs`; the old dynamic `eval.rs` path is deleted.
The current typed surface includes scalar and linear values, subscriptions,
one-dimensional slices, matrix/vector/ratvec crossings, RootDatum/Cartan
constructors, the exposed KGB constructor adapter, and now non-recursive
functions: typed lambda literals `(int n): body`, parameterless `@: body`
closures with frame capture (including escaped captures), `return` intercepted
at the call boundary and rejected at analysis outside a function body,
identifier selector postfix `receiver.name` lowered to `name(receiver)`,
function-definition sugar `f(params): body` in `let`/`set` declarations,
`rec_fun` recursive functions in declaration and expression form with explicit
result types (the self binding lives in the argument frame), binding and
parameter patterns (tuple destructuring, discard `type .`, const `!x`,
whole-value `(a, b): t`) compiled to a shared `SlotShape` frame layout,
operator/unit selectors (`2.-`, `2.3`) with operator selectors resolving
through the standard overload table, and loops (`while`/`for` collecting each
iteration's body value into a row, `break` discarding the breaking iteration,
`for x@i` index binding, `;` sequencing). This is not a
claim of full Atlas compatibility: InnerClass/RealForm/KGB rendering and
numbering, relations, primitive `involution` constructors, user overloads,
file commands, and later math overloads remain pending differential
evidence.

## Verified stage: B6 case and counted for

- `tests/fixtures/eval/casefor_b6.atlas` (11 accepted events: integer case
  with 0-based in-range selection, remainder wrapping for out-of-range
  without else, else catching out-of-range, then catching negative,
  positional union case with function branches, counted `for i: n from m`,
  `downto`, anonymous `for : n`, and `e1 next e2` collecting e1) and
  `tests/fixtures/eval/casefor_b6_rejected.atlas` (2 rejected type errors:
  non-function union branch `found int while (int->*) was needed.`,
  disagreeing branch types `found string while int was needed.`).
  Implementation: `IntCase`/`UnionCase`/`CountedFor`/`Next` typed variants,
  `conform_types` wording aligned to the oracle `found {} while {} was
  needed.` format (commit `5f58160`).
- Oracle capture: `3499627` (commit `cfdd9cc`), PASS against the frozen
  oracle.
- Differential: `pipeline_swap_diff` job `3500495` at commit `6df6622`
  reports both fixtures PASS; all previously verified fixtures PASS
  (regression clean).
- Reference metadata: `tests/reference/eval/casefor_b6{,_rejected}.meta.json`
  carry `rust_status: verified_hpc` with `differential_job: 3500495`.

## Verified stage: B5 set_type

- `tests/fixtures/eval/settype_b5.atlas` (accepted: single-name `set_type`
  aliases with projector/injector overloads, bracketed `set_type [ ... ]`
  entering the tabled type map for case discrimination and recursion, union
  values displaying as `value.tag`, tabled types printing by name in
  `whattype`, `Defined type:`/`Type:` headers) and
  `tests/fixtures/eval/settype_b5_rejected.atlas` (rejected: `expr : type`
  ascription syntax error, case discrimination on a union named only by the
  single-name form, discrimination branches with disagreeing result types).
- Oracle capture: `3499601` (commit `559f363`), PASS against the frozen oracle.
- Differential: `pipeline_swap_diff` job `3500393` at commit `9bb95e3`
  reports both fixtures PASS (suite PARTIAL as long as
  `pipeline_swap_domain_equality` keeps its pending domain lines; B6-B12
  fixtures still FAIL until implemented).
- Reference metadata: `tests/reference/eval/settype_b5{,_rejected}.meta.json`
  carry `rust_status: verified_hpc` with `differential_job: 3500393`.
- Note: job `3500391` was invalidated by fixture-side file creation inside
  the frozen snapshot; commit `9bb95e3` moved fixture execution into an
  isolated per-run workspace directory.

## Verified stage: B4 loops

- `tests/fixtures/eval/loops_b4.atlas` (8 accepted lines: `while`/`for`
  collecting each iteration's body value into a row, `break` contributing
  nothing for the breaking iteration, condition-less `while do ... od`,
  `for x@i` index binding, `begin`-style `;` sequencing) and
  `tests/fixtures/eval/loops_b4_rejected.atlas` (4 rejected lines: top-level
  `break`, `break x` syntax error, iterating a non-row, non-boolean while
  condition). Implementation: `Sequence`/`While`/`For`/`Break` typed variants
  with analysis-time `loop_depth` legality and `Control::Break(usize)`
  evaluation (commit `5be00f9`).
- Oracle capture: `3498786` (commit `a5856a1`), PASS against the frozen oracle.
- Differential: `pipeline_swap_diff` job `3499732` at commit `152138ca`
  reports both fixtures PASS (suite PARTIAL as long as
  `pipeline_swap_domain_equality` keeps its pending domain lines).
- Reference metadata: `tests/reference/eval/loops_b4{,_rejected}.meta.json`
  carry `rust_status: verified_hpc` with `differential_job: 3499732`.

The bounded local checks for this stage:

- `cargo test -p atlas-core --lib`: 174 passed, 0 failed;
- `cargo clippy -p atlas-core --lib -- -D warnings`;
- `cargo fmt --all -- --check` and `python3 hpc/test_pipeline_swap_diff.py`.

## Verified stage: B3c parameter patterns and B3d selectors

- `tests/fixtures/eval/patterns_b3c.atlas` (5 accepted lines: tuple
  destructuring bindings, discard `type .` parameters, const `!x` bindings,
  whole-value `(a, b): t` patterns) and
  `tests/fixtures/eval/patterns_b3c_rejected.atlas` (3 rejected lines:
  const assignment, two pattern shape mismatches). Implementation: `Pattern`
  AST with `SlotShape` frame layout shared by let groups and call frames
  (commit `83debd3`).
- `tests/fixtures/eval/selectors_b3d.atlas` (3 accepted lines: unit selector
  `().f`, chained identifier selectors `2.f.g`, operator selector `2.-`) and
  `tests/fixtures/eval/selectors_b3d_rejected.atlas` (2 rejected lines:
  `2.+` without a unary-plus overload, `2.3` calling a non-function).
  Implementation: selector callee variants identifier/operator/unit-literal,
  operator selectors reusing `OperatorCall` overload resolution (commit
  `f6a5e5c`).
- Oracle captures: B3c `3498578`, B3d `3498619`, both PASS against the
  frozen oracle.
- Differential: `pipeline_swap_diff` job `3499673` at commit `a938573`
  reports all four fixtures PASS (the same run reports `loops_b4` FAIL,
  expected: its implementation was still in flight).
- Reference metadata: `tests/reference/eval/patterns_b3c{,_rejected}.meta.json`
  and `selectors_b3d{,_rejected}.meta.json` carry
  `rust_status: verified_hpc` with `differential_job: 3499673`.

The bounded local checks for these stages:

- `cargo test -p atlas-core --lib`: 169 passed, 0 failed;
- `cargo clippy -p atlas-core --lib -- -D warnings`;
- `cargo fmt --all -- --check` and `python3 hpc/test_pipeline_swap_diff.py`.

## Verified stage: B3a non-recursive functions

- `tests/fixtures/eval/functions_b3.atlas` (5 accepted lines) and
  `tests/fixtures/eval/functions_b3_rejected.atlas` (6 rejected lines: top-level
  return, argument type mismatch, wrong arity as void-vs-pattern, calling a
  non-function, missing-colon lambda syntax error, undefined selector target).
- Oracle capture: HPC jobs `3498312` (accepted) and `3498466` (rejected),
  both PASS against the frozen `/public/home/majj/atlasofliegroups-4d3e9449`
  checkout (revision `4d3e9449062a07c1c85f4e6df215eb6ccc0eeae9`, binary sha256
  `66f5d7d4...`, submitted with `ATLAS_BIN` and
  `EXPECTED_ATLAS_BINARY_SHA256=66f5d7d47d560e616363392b38205166d1579985dc7337cc95ba4cae50be65c9`).
- Differential: `pipeline_swap_diff` job `3498527` reports both fixtures PASS
  (stdout/exit/diagnostics exact; suite remains PARTIAL only for the known
  `pipeline_swap_domain_equality` pending cases).
- Reference metadata: `tests/reference/eval/functions_b3{,_rejected}.meta.json`
  carry the capture provenance and `rust_status: verified_hpc` with
  `differential_job: 3498527`.

The bounded local checks for this stage:

- `cargo test -p atlas-core --lib`: 154 passed, 0 failed;
- `cargo clippy -p atlas-core --lib -- -D warnings`;
- `cargo fmt --all -- --check`, JSON validation of the reference files, and
  `python3 hpc/test_pipeline_swap_diff.py`.

Only bounded local checks are appropriate here. The project policy puts full
workspace tests, Atlas/CWEB execution, differential jobs, and benchmarks on
XMU HPC.

## Verified stage: B3b recursive functions and definition sugar

- `tests/fixtures/eval/functions_b3b.atlas` (6 accepted lines: single- and
  multi-parameter definition sugar, `rec_fun` in declaration and expression
  form with explicit result types, parameterless sugar, recursive closures
  capturing their `let` scope) and
  `tests/fixtures/eval/functions_b3b_rejected.atlas` (3 rejected lines: body
  type error under sugar, recursive call with mismatched argument type,
  recursive declaration missing its result type).
- Oracle capture: HPC job `3498562`, PASS against the frozen oracle.
- Differential: `pipeline_swap_diff` job `3498653` at commit `f773695`
  reports both fixtures PASS.
- Reference metadata: `tests/reference/eval/functions_b3b{,_rejected}.meta.json`
  carry `rust_status: verified_hpc` with `differential_job: 3498653`.
- The bison expecting-list after `syntax error, unexpected IF` is not
  asserted; only the offending token is (see `docs/DESIGN.md` on diagnostic
  wording vs semantic equality).

The bounded local checks for this stage:

- `cargo test -p atlas-core --lib`: 160 passed, 0 failed;
- `cargo clippy -p atlas-core --lib -- -D warnings`;
- `cargo fmt --all -- --check`, JSON validation of the reference files, and
  `python3 hpc/test_pipeline_swap_diff.py`.

## HPC operations notes (verified this stage)

- The submit checkout must be clean at the declared commit. A previous job's
  root-level Slurm stdout (`atlas-*-<jobid>.out`) is untracked and makes the
  next submission dirty; move it away before resubmitting. The same applies to
  stale untracked sources (an old `eval.rs` leftover blocked one sync).
- The frozen oracle `/public/home/majj/atlasofliegroups-4d3e9449` is a git
  checkout at the pinned revision and must stay clean: job `3498017` failed
  because legacy `oracle-results/` and copied fixture files were left inside
  it (now in `/tmp/atlas-oracle-trash` on the login node). The unpinned
  `/public/home/majj/atlasofliegroups` tree is no longer a git repository and
  its binary differs from every pin; do not use it for captures.
- `reference_capture.sbatch` fails before the harness when declared and
  detected source state differ; the FAIL fallback report names the phase.
- After any commit that touches `crates/`, a subsequent rsync that excludes
  `crates/` (while a background agent holds uncommitted changes) leaves the
  remote checkout dirty against its HEAD, and a capture submitted in that
  window records `dirty_tree: true`. Repair with
  `git archive HEAD crates | ssh ... tar -x -C <remote>` before submitting,
  and re-capture anything taken in the dirty window (job `3499634` was
  re-taken as `3499638`).

## Next implementation slice (B7 misc commands in flight, then B8/B9/B10/B12)

In rough dependency order, each with its own fixture + HPC capture first:

1. B7 misc commands (capture `3499657`, commit `21ee423`): `forget` of
   unknown identifiers and of single overloads, `die` as a runtime
   diagnostic with batch continuation, coercion fallback after overload
   removal. The `whattype id_op ?` overload listing is deferred until the
   domain types appearing in builtin lists are ported.
2. B8 user overloads (captures `3499692`, `3499705`): `set f = <lambda>`
   accumulates overloads (`Defined f: T`, `Added definition [2] of f: T`,
   `Redefined f: T` for a repeated signature), `whattype f ?` lists user
   overloads in definition order, calls resolve by arity, and a variable can
   coexist with function definitions on one identifier; wrong-arity calls
   are analysis-time type errors.
3. B9 file commands (capture `3499747`; probe `3499729`, file evidence
   `3499737`): `> "f" expr` / `>> "f" expr` redirect only the
   `Value: ...` line (truncate/append), a failed open prints
   `Failed to open <name>` on stderr and continues, and `tofile` accepts
   only an expression (`set` there is a syntax error). The accepted lines
   already PASS as of job `3500393`; the rejected line needs parse failure
   before the output file is opened, and open failures must render through
   the `Io error:` diagnostic header.
4. B10 fromfile/quit (capture `3500378`): `< "f"` / `<< "f"` with a missing
   target print `failed to open input file '<name>'.` on stderr, batch
   continues, exit stays 0; `quit` mid-input terminates evaluation
   immediately, still prints `Bye.`, exit 0. Accepted-form inclusion
   semantics still need an HPC-absolute helper probe.
5. B12 runtime-error messages (capture `3500488`; differential `3500489`
   shows 2 of 5 already exact): row subscription out-of-range must append
   the space-free subscription source (`in subscription [1,2][5]`), tuple
   subscription with a non-constant index is a type error worded `Cannot
   subscript value of type (int,int) with index of type int`, and string
   subscription must exist as a runtime-checked operation.
6. Domain surface, smallest first: `pipeline_swap_domain_equality` lines
   3-14 (capture `3496440`). Gap analysis (2026-07-29, measured against the
   oracle): KGB `#0` numbering and all six equality/inequality events already
   match; the only blockers are two Display placeholders. (a) InnerClass
   print (`domain_builtins.rs:190-194`) needs: LieType reconstruction from
   the Cartan matrix (Dynkin classification + Bourbaki layout, no Rust
   module yet), inner-class type letters from the distinguished twist
   (`c`/`s`/`u`/`C`; `InnerClass::new` currently requires a distinguished
   involution and exposes no twist API), `numRealForms` (READY via
   `ExternalFormOrder::form_count`), and `numDualRealForms` (needs the dual
   root datum / dual weak-real-form partition — the largest sub-gap, no
   dual machinery in Rust yet). (b) RealForm print
   (`domain_builtins.rs:195-197`) needs: connected/compact/split/quasisplit
   flags (most-split Cartan involution export + dual component group —
   dual again) and the `printType` Lie-algebra naming module
   (`ExternalFormOrder` sorting is ported; per-form special gradings and
   the A/B/C/D/E/F/G/T naming branches are not). Upstream evidence:
   `atlas-types.w:3164-3172`, `3565-3575`; `output.cpp:751-782`. After
   those, the
   14 `tests/fixtures/domain/*.atlas` fixtures are blocked one level deeper:
   an Atlas-callable constructor/event adapter must exist before their
   oracle references can even be captured. Also uncovered: `showall`,
   `dont`, `quit` semantics, `whattype id_op ?` builtin listing, `fromfile`,
   KL/file formats, interactive input, and the primitive domain types
   (Split/Block/KType/KTypePol/Param/ParamPol).

Before continuing, run the smallest local parser/core check with the project
toolchain, then sync a clean committed tree to HPC and submit the relevant
SLURM job. Record the job id, reference revision, source commit, dirty state,
fixture manifest, exit code, and checksums in the reference metadata/report.

## Local environment

- `rustup` is installed through Homebrew.
- Stable toolchain: Rust 1.96.0; project `rust-toolchain.toml` selects stable
  and requires clippy/rustfmt.
- Rust 1.90.0 is also installed for the repository's earlier local gate.
- `rust-analyzer` is installed at `/opt/homebrew/bin/rust-analyzer`.
- `~/.cargo/bin` now precedes `/opt/local/bin` in `~/.zprofile`, so new shells
  use rustup's `rustc`, `cargo`, `clippy`, and `rustfmt` proxies. Restart the
  shell or source `~/.zprofile` before checking versions.

## Standing rules

- Read `docs/COMPATIBILITY.md`, `docs/LANGUAGE.md`, and `docs/DESIGN.md` before
  changing language behavior.
- Add/update fixture and reference metadata before implementation claims.
- Never hand-edit generated CWEB or parser output.
- Keep root-data and real-group invariants in their owned domain layer.
- Preserve unrelated user changes and do not commit unverified HPC output.

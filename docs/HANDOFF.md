# Atlas-Rust handoff - 2026-07-30 (handoff to next coding agent)

This is the continuation record for `/Users/hoxide/mycodes/atlas-rust`.
The goal is source-compatible Atlas language behavior, with the upstream Atlas
executable and CWEB sources as the behavior oracle. The core remains safe Rust.

## Start here (next agent)

HEAD at handoff: `5aa7cf6` (main). Working tree clean. All eval slices B3a-B12
and the InnerClass/RealForm display are `verified_hpc` (differential
`3501467`); `root_coroot` (`af6cd7b`) and `kgb_generation` (`d7cef57`) are
implemented and awaiting the HPC differential `3501555` (submitted at
`5aa7cf6`, report under
`results/5aa7cf6a8425f1fcd4285b86ae2ca2dfcc3397df/3501555/pipeline_swap/pipeline_swap_diff_report.json`
on the HPC side — check `squeue -u majj`, fetch, verify PASS, then upgrade the
four metas `tests/reference/domain/root_coroot{,_rejected}.meta.json` and
`kgb_generation{,_rejected}.meta.json` to `rust_status: verified_hpc` with
`differential_job: "3501555"` and commit).

One background subagent (agent-9) may still be implementing the `real_group`
slice (5 missing builtins: `nr_of_dual_real_forms`, `form_names`,
`dual_form_names`, `dual_real_form`, `dual_quasisplit_form` — everything else
in that fixture already works). If its changes are uncommitted in the tree
(`crates/atlas-core/src/domain_builtins.rs`, `typed.rs`,
`crates/atlas-real-group/`, `hpc/pipeline_swap_diff.py`), verify them
yourself per the loop below before committing; otherwise re-do the slice
from scratch — its contract is frozen and fully probed.

## The per-slice loop (follow exactly)

1. Pick the next contract from the queue below. Contracts are already
   frozen (events.json status `verified_hpc_reference`); do NOT redesign
   them unless an implementation proves the probe wrong — in that case
   re-probe the oracle, never guess.
2. Implement in the smallest owning module. Domain builtins register in
   `crates/atlas-core/src/typed.rs` `builtin_registry()` (pattern: the six
   `root_coroot` entries after `Cartan_matrix(RootDatum)`, commit
   `af6cd7b`) and evaluate in `crates/atlas-core/src/domain_builtins.rs`;
   crate-level math lives in `crates/atlas-real-group/` (safe Rust only).
   Add `FixturePlan(name="domain/<n>")` (and `_rejected`) to
   `hpc/pipeline_swap_diff.py`.
3. Local bounded checks, all must pass:
   `cargo test -p atlas-core --lib`, `cargo test -p atlas-real-group --lib`,
   `cargo clippy -p atlas-core -p atlas-real-group --lib --tests -- -D warnings`,
   `cargo fmt --all -- --check`, `cargo build -p atlas-cli`
   (use `export PATH="$HOME/.cargo/bin:$PATH"`).
4. Verbatim fixture check in a /tmp cwd: run
   `./target/debug/atlas-cli tests/fixtures/domain/<n>.atlas`, compare
   stdout/stderr/exit against events.json via
   `hpc/pipeline_swap_diff.py:expected_cli_observation`.
5. Full local regression (FAIL allowed only for
   `fromfile_accepted_b10`, which needs HPC paths):
   `cd /tmp && rm -rf R && mkdir -p R/workspace && cp -R <repo>/tests R/workspace/ && cd R && python3 <repo>/hpc/pipeline_swap_diff.py <repo>/target/debug/atlas-cli out --workspace-root workspace --fixture-root <repo>/tests/fixtures --reference-root <repo>/tests/reference --commit local --dirty-tree true --detected-commit local --detected-dirty-tree true --job-id local --source-snapshot-sha256 local`
   Delete `hpc/__pycache__` afterwards (it is gitignored but keep the tree
   tidy); delete any stray file `x` at repo root if a runner creates it.
   Also run `python3 hpc/test_pipeline_swap_diff.py` (10 tests).
6. Commit (conventional commits, no push without asking).
7. Sync and submit the HPC differential:
   `rsync -az --delete .git/ majj@10.26.14.64:/public/home/majj/atlas-rust/.git/ && git archive HEAD | ssh majj@10.26.14.64 'cd /public/home/majj/atlas-rust && tar -xf -'`
   then `ssh majj@10.26.14.64 "cd /public/home/majj/atlas-rust && ATLAS_COMMIT=$(git rev-parse HEAD) ATLAS_DIRTY_TREE=false sbatch hpc/pipeline_swap_diff.sbatch"`.
   This sync pattern is robust against dirty working trees (concurrent
   subagents); the remote checkout must equal HEAD exactly.
8. When the job finishes: fetch the report, confirm the target fixtures
   PASS and no regressions (suite PARTIAL is normal while pending
   overloads remain), then upgrade the fixture metas to
   `rust_status: verified_hpc` + `differential_job` and commit.

## Implementation queue (all contracts frozen, in suggested order)

Domain (contracts in `tests/fixtures/domain/`, events verified):
`real_group` (agent-9, see above) → `kgb_operations` → `grading` →
`weyl_element` → `cartan_aggregation` → `seed_x0` → `involution_table` →
`adjoint_fiber` → `real_form_labels` → `weak_real_form` →
`involution_decomposition` → `tits_operations` → `strong_real` →
`split_basic` (eval/) → `block_basic` → `ktype_basic` → `ktypepol_basic` →
`param_basic` → `parampol_basic` → `involution_primitive` →
`overloads_ops_b8c` + `whattype_ops_b8d` (operator-`set` form and the
builtin `whattype * ?` listing; b8d pins the current 23-row `*` table) are
implemented and verified in differential `3501643`.

Uncovered matrix items needing contract design first (probe the oracle,
then freeze): KL file formats and readline completion. `dont`, `showall`,
`quit`, and the basic interactive TTY banner/prompt are implemented; the
newly frozen language fixtures are covered by differential `3501643`. Deeper math
overloads (KL polynomials, `W_graph`, `deform`, extended blocks).

## dont/showall probe findings (2026-07-30)

- Bare top-level `let x = 3` is a SYNTAX ERROR in the oracle
  (`expecting IN or THEN or ','`); `let x = 3 in x` evaluates fine.
- `dont` is only valid where parser.y has `do_expr` (while bodies,
  do-if branches, case arms): `for` loop bodies are plain `expr` and
  reject it; `while true do dont od` also fails because after `DO` the
  `tertiary DO expr` rule wants `expr`. The do_expr `DONT` alternative
  (parser.y:442) makes `sequence(false, die)` — canonical usage is
  `if cond then dont else ... fi` inside `while` bodies (see
  atlas-scripts/test.at:43). A valid minimal probe was NOT yet found;
  try `while true; if false then dont fi od` shapes before writing the
  fixture.
- `showall` prints `Overloaded operators and functions:` then
  `name: (signature): {source}` per overload (huge); untested further.

## Environment facts

- Local: macOS, `export PATH="$HOME/.cargo/bin:$PATH"`; CLI at
  `./target/debug/atlas-cli`. Upstream C++ sources (read-only reference):
  `/Users/hoxide/mycodes/atlasofliegroups` (master `4d3e9449`).
- HPC: `ssh majj@10.26.14.64`, project `/public/home/majj/atlas-rust`,
  frozen oracle `/public/home/majj/atlasofliegroups-4d3e9449/atlas`
  (rev `4d3e9449062a07c1c85f4e6df215eb6ccc0eeae9`, binary sha256
  `66f5d7d47d560e616363392b38205166d1579985dc7337cc95ba4cae50be65c9`).
- Direct oracle probe (for designing new contracts; login node needs the
  gcc runtime):
  `ssh majj@10.26.14.64 'module load misc/gcc/12.1 >/dev/null 2>&1; gcc_lib="$(dirname "$(gcc -print-file-name=libstdc++.so.6)")"; export LD_LIBRARY_PATH="$gcc_lib:$LD_LIBRARY_PATH"; cd /public/home/majj/atlasofliegroups-4d3e9449/atlas-scripts && printf "<lines>\nquit\n" | /public/home/majj/atlasofliegroups-4d3e9449/atlas 2>&1'`
- Reference capture: `ATLAS_BIN=... EXPECTED_ATLAS_BINARY_SHA256=66f5d7d... sbatch hpc/reference_capture.sbatch tests/fixtures/<sub>/<name>.atlas ...`
  (FULL paths with extension). Reports land in
  `results/<commit>/<jobid>/reference_capture/reference_capture_report.json`;
  per-fixture stdout/stderr text is embedded — verify verbatim against
  events.json before writing provenance.
- Meta provenance fields (order): fixture/oracle("atlas")/stage/
  reference_status/reference_atlas_revision/reference_binary_sha256/
  reference_job/source_archive_sha256/fixture_sha256/oracle_exit_status/
  oracle_stdout_sha256/oracle_stderr_sha256/capture_artifacts_sha256/
  rust_status/upstream_evidence/notes(/differential_job). The artifacts
  hash: on HPC in the capture dir,
  `shasum -a 256 "$PWD/x.stdout" "$PWD/x.stderr" > artifacts_x.sha256`,
  then take that file's own sha256. events.json status goes
  `pending_hpc_reference` → `verified_hpc_reference`; rust_status goes
  `not_implemented` → `verified_hpc` (with `differential_job`).
- Harness dirty detection ignores `atlas-*.out` everywhere
  (`b1afa5e`, `cbf538f`); `__pycache__/` is gitignored (`4843b9f`).
- Value/event encodings used in events.json: integers/booleans/strings
  plain; `{"type":"vec","display":"[ 1, 0 ]"}` (padded); rows unpadded
  `[0,1,0]`; `{"type":"ratvec","display":"[ 1, 0 ]/2"}`;
  `{"type":"matrix","display":"\n| 1, 0 |\n| 0, 1 |\n"}`; domain values
  `{"type":"domain","domain":"RealForm","display":"..."}`; KTypePol/
  ParamPol terms have a leading-newline display; any value may carry
  `display` verbatim (harness `render_value` short-circuits on it).

## Current state

- Branch: `main`.
- B3a non-recursive functions, B3b recursive functions / definition sugar,
  B3c parameter patterns, B3d selectors, B4 loops, B5 `set_type`, B6
  case / counted-for, B7 forget/die, B8 user overloads + `set`, B9
  redirect-body parsing, B10 file inclusion (accepted and missing-file),
  B11 precedence, and B12 subscription/runtime diagnostics are implemented
  and differentially verified. The exact commit is shown by
  `git log -1 --oneline`.
- InnerClass/RealForm values now print exactly as the oracle renders them
  (compact/split/quasisplit/disconnected variants, dual-form
  singular/plural), verified by differential `3501467`; the
  `pipeline_swap_domain_equality` fixture runs fully in the swap plan.
- Domain contracts frozen against the oracle: `root_coroot` + `kgb_generation`
  (implemented `af6cd7b`/`d7cef57`, differential `3501555` in flight),
  `real_group` (`3501368`), `grading` + `involution_primitive` (`3501449`),
  `weyl_element` + `kgb_operations` (`3501466`), `cartan_aggregation` +
  `seed_x0` + `involution_table` + `adjoint_fiber` + `real_form_labels` +
  `weak_real_form` + `involution_decomposition` + `tits_operations` +
  `strong_real` (`3501500`), `split_basic` + `block_basic` (`3501519`),
  `ktype_basic` + `ktypepol_basic` + `param_basic` + `parampol_basic`
  (`3501537`) — all pending implementation except where noted.
- Eval contracts `overloads_ops_b8c{,_rejected}`, `whattype_ops_b8d`, and
  `dont_b13{,_rejected}` are implemented and verified by differential
  `3501643`.
- Harness: Slurm stdout files (`atlas-*.out`) no longer count as checkout
  dirt in either the bootstrap or the checked source-state helper
  (commits `b1afa5e`, `cbf538f`); `__pycache__/` is gitignored (`4843b9f`).
- No uncommitted repository changes should remain after the handoff commit.

The typed session pipeline is active: `session.rs` and `session_frame.rs`
convert/evaluate through `typed.rs`; the old dynamic `eval.rs` path is deleted.
The current typed surface includes scalar and linear values, subscriptions
(including string subscript with the oracle range wording), one-dimensional
slices, matrix/vector/ratvec crossings, RootDatum/Cartan constructors, the
exposed KGB constructor adapter, non-recursive functions: typed lambda
literals `(int n): body`, parameterless `@: body` closures with frame capture
(including escaped captures), `return` intercepted at the call boundary and
rejected at analysis outside a function body, identifier selector postfix
`receiver.name` lowered to `name(receiver)`, function-definition sugar
`f(params): body` in `let`/`set` declarations, `rec_fun` recursive functions
in declaration and expression form with explicit result types, binding and
parameter patterns (tuple destructuring, discard `type .`, const `!x`,
whole-value `(a, b): t`) compiled to a shared `SlotShape` frame layout,
operator/unit selectors (`2.-`, `2.3`) with operator selectors resolving
through the standard overload table, loops (`while`/`for` collecting each
iteration's body value into a row, `break` discarding the breaking iteration,
`for x@i` index binding, `;` sequencing), user overloads with merged
builtin/user dispatch (`Defined`/`Added definition [n]`/`Redefined` reports,
`whattype f ?` listings, shadow-on-exact-replace forget semantics), `set`
parallel bindings (all RHS analyzed, then evaluated, then bound), and
redirect bodies parsed as expressions before the sink opens. This is not a
claim of full Atlas compatibility: RootDatum/InnerClass/RealForm/KGB domain
queries beyond construction and display, relations, primitive `involution`
constructors, Cartan classes, Weyl elements, synthetic KGB seeds, and the
later math overloads remain pending differential evidence.

## Verified stage: B8/B9/B10/B12 + domain display (differential 3501467)

- `overloads_b8{,b,_rejected}`: user function overloads via `set f =
  (int x): x` — heterogeneous signatures accumulate (`Added definition
  [2] of f:`), same-signature replaces (`Redefined`), non-function values
  bind as variables coexisting with the overload table, `whattype f ?`
  lists instances, and merged dispatch inserts user variants among the
  builtins (commit `162216c`).
- `file_commands_b9{,_rejected}`: redirect bodies parse as expressions
  before the sink opens, so `> "x" set qfc = 10` fails with
  `syntax error, unexpected '='` and creates no file; the expression
  grammar accepts the parser.y:264 `set pattern := expr` form (analysis
  rejects it as not yet implemented) (commit `2a3eff6`).
- `fromfile_accepted_b10`: HPC-only include fixture, fixed by the B8
  `set` implementation.
- `runtime_errors_b12`: range errors carry the compact subscription
  source (`index N out of range (0<= . <L) in subscription EXPR`), tuple
  subscript is the axis.w:4101-4105 type error, and string subscript is
  legal with one-character results (commit `a3c2f8d`).
- `pipeline_swap_domain_equality` now runs fully: InnerClass/RealForm
  display matches the oracle (Dynkin classification, inner-class layout,
  dual counts, topology, real-form type naming, presentation bits;
  commit `b4c8dc6`).
- Differential: `pipeline_swap_diff` job `3501467` at commit `8feb364`
  reports all 31 fixtures PASS with zero failures (suite PARTIAL only for
  the three plan-level pending overloads: two `involution` constructors
  and the synthetic `real_form`). Metadata carries
  `rust_status: verified_hpc` with `differential_job: 3501467`.
- Harness fixes landed alongside: Slurm stdout ignored in dirty detection
  (`b1afa5e`, `cbf538f`), `__pycache__/` gitignored (`4843b9f`).

## Verified stage: B7 forget/die + B10 missing-file diagnostics

- `tests/fixtures/eval/commands_b7.atlas` (4 accepted events: `forget x`
  on an unknown name reports `Identifier 'x' not known`, `forget + @
  (int,int)` reports `Definition of '+@(int,int)' forgotten`, after which
  `1+2` resolves through int->rat coercion to `3/1`) and
  `tests/fixtures/eval/commands_b7_rejected.atlas` (`die` raises runtime
  `I die` and the batch continues; an undefined identifier is the name
  error `Undefined identifier 'x'`). Implementation: `Command::Forget` /
  `Command::ForgetOverload` / `Expr::Die`; overload removal is a
  per-context filter over the static builtin registry
  (`Analysis::forgotten`); the plain-identifier and assignment undefined
  wordings now match `axis.w:1431` (commit `f86fc68`).
- `tests/fixtures/eval/fromfile_b10.atlas` (2 io diagnostics for missing
  `<`/`<<` targets, batch continues, exit 0): span-less diagnostics render
  with the `<Kind> error:` header (commit `73c7d81`); the oracle prints
  the same lines bare, so the header is a harness-grammar surface, not an
  oracle wording change.
- Oracle captures: `3499657` (B7), `3500378` (B10).
- Differential: `pipeline_swap_diff` job `3500583` at commit `37e0f23`
  reports all three fixtures PASS; all previously verified fixtures PASS
  (regression clean). Metadata carries `rust_status: verified_hpc` with
  `differential_job: 3500583`.

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

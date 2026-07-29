# Atlas-Rust handoff - 2026-07-29 (B3a verified)

This is the continuation record for `/Users/hoxide/mycodes/atlas-rust`.
The goal is source-compatible Atlas language behavior, with the upstream Atlas
executable and CWEB sources as the behavior oracle. The core remains safe Rust.

## Current state

- Branch: `main`.
- The B3a non-recursive function stage is implemented and differentially
  verified. The exact commit is shown by `git log -1 --oneline`.
- No uncommitted repository changes should remain after the handoff commit.

The typed session pipeline is active: `session.rs` and `session_frame.rs`
convert/evaluate through `typed.rs`; the old dynamic `eval.rs` path is deleted.
The current typed surface includes scalar and linear values, subscriptions,
one-dimensional slices, matrix/vector/ratvec crossings, RootDatum/Cartan
constructors, the exposed KGB constructor adapter, and now non-recursive
functions: typed lambda literals `(int n): body`, parameterless `@: body`
closures with frame capture (including escaped captures), `return` intercepted
at the call boundary and rejected at analysis outside a function body, and
identifier selector postfix `receiver.name` lowered to `name(receiver)`. This
is not a claim of full Atlas compatibility: InnerClass/RealForm/KGB rendering
and numbering, relations, primitive `involution` constructors, recursive
functions, parameter patterns beyond a simple typed name, operator selectors,
loops, `set_type`, and later math overloads remain pending differential
evidence.

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

## Next implementation slice (B3b candidates)

In rough dependency order, each with its own fixture + HPC capture first:

1. Function-definition sugar `f(x): ...` / `set f = ...` forms and
   `REC_FUN IDENT '(' id_specs_opt ')' type ':' expr` recursive functions
   (parser.y:231, 251-255); recursion wraps closure+arguments (axis.w:3294-3297).
2. Parameter patterns: destructuring `'(' id_specs ')'`, discard `type .`,
   const patterns (parser.y:725-790 `id_spec`/`pattern` rules).
3. Operator selectors (`.+` etc., parser.y:321-337 `selector: operator`) and
   unit selector `().name`.
4. Loops (`while`/`for`) and `set_type` syntax.

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

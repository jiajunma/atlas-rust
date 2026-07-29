# Atlas-Rust handoff - 2026-07-29

This is the continuation record for `/Users/hoxide/mycodes/atlas-rust`.
The goal is source-compatible Atlas language behavior, with the upstream Atlas
executable and CWEB sources as the behavior oracle. The core remains safe Rust.

## Current state

- Branch: `main`.
- The latest committed work before this handoff is the typed session pipeline
  series ending at `6e4d0a7 docs: refresh typed pipeline handoff`.
- This handoff and the B3a parser fixture are committed together after local
  checks. The exact commit is shown by `git log -1 --oneline`.
- No uncommitted repository changes should remain after the handoff commit.

The typed session pipeline is active: `session.rs` and `session_frame.rs`
convert/evaluate through `typed.rs`; the old dynamic `eval.rs` path is deleted.
The current typed surface includes scalar and linear values, subscriptions,
one-dimensional slices, matrix/vector/ratvec crossings, RootDatum/Cartan
constructors, and the exposed KGB constructor adapter. This is not a claim of
full Atlas compatibility: InnerClass/RealForm/KGB rendering and numbering,
relations, primitive `involution` constructors, and later math overloads remain
pending differential evidence.

## This stage: B3a preparation

The committed B3a slice adds syntax data structures only:

- `Expr::Lambda` for a non-recursive function literal;
- `Expr::Return` for `return value`;
- `LambdaParam` for a simple typed named parameter;
- selector postfix metadata for `receiver.name`, intended to lower to
  `name(receiver)`;
- parser token variants for `return`, `@`, and `.`;
- `tests/fixtures/eval/functions_b3.atlas` with five representative cases;
- pending-HPC reference metadata and an empty events placeholder.

The lexer, LALRPOP grammar, typed conversion/evaluation, closure capture,
return unwinding, and selector lowering are not implemented yet. Do not call
B3 functions supported until the parser compiles, runtime behavior is wired,
and both accepted and rejected cases have an HPC differential report.

## Verification and evidence

The bounded local checks completed for this commit are:

- `cargo test -p atlas-core --lib`: 145 passed, 0 failed;
- `cargo clippy -p atlas-core --lib -- -D warnings`;
- `cargo fmt --all -- --check`, `git diff --check`, and JSON validation for the
  two B3 reference files.

Only bounded local checks are appropriate here. The project policy puts full
workspace tests, Atlas/CWEB execution, differential jobs, and benchmarks on
XMU HPC. The B3 reference files intentionally remain `pending_hpc_reference`.

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

## Next implementation slice

1. Read the upstream parser and axis references listed in
   `tests/reference/eval/functions_b3.meta.json`.
2. Run an HPC raw capture for every B3a fixture line and normalize only after
   capture; retain negative/rejected cases.
3. Complete lexer and grammar support for typed lambda parameters, `@`, `.`,
   `return`, and the `in` expression form.
4. Add safe closure values with frame capture and a call-boundary return signal.
   Keep recursive functions, destructuring/const/discard patterns, operator
   selectors, definitions, loops, and full `set_type` syntax for later slices.
5. Run focused local checks, then the HPC differential job, update this file,
   and commit the verified stage.

## Standing rules

- Read `docs/COMPATIBILITY.md`, `docs/LANGUAGE.md`, and `docs/DESIGN.md` before
  changing language behavior.
- Add/update fixture and reference metadata before implementation claims.
- Never hand-edit generated CWEB or parser output.
- Keep root-data and real-group invariants in their owned domain layer.
- Preserve unrelated user changes and do not commit unverified HPC output.

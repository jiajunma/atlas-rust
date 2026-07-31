# Brief L4: oracle-compatible unterminated-string recovery in the lexer

You are working in `/Users/hoxide/mycodes/atlas-rust` (Rust reimplementation of
the Atlas of Lie Groups language; upstream C++/CWEB read-only reference at
`/Users/hoxide/mycodes/atlasofliegroups`). The original Atlas lexer treats an
unterminated string as a recoverable lexical warning: it prints
`Closing string denotation.`, recovers the string content up to end-of-line,
and evaluation continues (exit status 0). The Rust CLI currently makes it a
fatal lexical error (exit 1). Your job: port the recovery behavior so the
frozen contract `negative/unterminated_string` compares VERBATIM.

## Scope discipline

- Only `crates/atlas-core` lexer (+ wherever diagnostic severity/exit-status
  is decided). Do NOT touch `tests/`, `hpc/`, `docs/`, events/meta, or the
  parser/evaluator message wordings (other slices own those).
- No git commits. Leave edits in the working tree.

## Frozen target (HPC capture job 3503334; events already frozen)

Fixture `tests/fixtures/negative/unterminated_string.atlas` (one line:
`"unterminated`):
- stdout: `Value: "unterminated"\nBye.\n` — the string is recovered as the
  literal `unterminated` and the expression evaluates.
- stderr diagnostic: category `lexical`, message `Closing string denotation.`
- exit status 0 (the diagnostic is a warning; it must NOT force exit 1).

Upstream anchor: grep `Closing string denotation` in the upstream sources
(lexer `.l`/`.w` files) to confirm the recovery semantics (string closes at
end of line, warning printed, parsing continues).

The harness parses the CLI stderr with a `Lexical error at <path>:l:c: <msg>`
header plus `  | ` source-echo lines — keep that rendering shape so the
diagnostic stays parseable (category `lexical`), but make the run exit 0 and
continue evaluating. Check how the CLI currently decides exit status: a
warning must not flip it to 1.

## Verification (must all pass)

1. `export PATH="$HOME/.cargo/bin:$PATH"`.
2. `cargo build -p atlas-cli`, then
   `python3 /tmp/check_fixture.py negative/unterminated_string` — VERBATIM.
3. Full local regression:
   ```
   cd /tmp && rm -rf R && mkdir -p R/workspace && cp -R /Users/hoxide/mycodes/atlas-rust/tests R/workspace/ && cd R && python3 /Users/hoxide/mycodes/atlas-rust/hpc/pipeline_swap_diff.py /Users/hoxide/mycodes/atlas-rust/target/debug/atlas-cli out --workspace-root workspace --fixture-root /Users/hoxide/mycodes/atlas-rust/tests/fixtures --reference-root /Users/hoxide/mycodes/atlas-rust/tests/reference --commit local --dirty-tree true --detected-commit local --detected-dirty-tree true --job-id local --source-snapshot-sha256 local 2>&1 | tail -3
   ```
   Only allowed FAIL: `eval/fromfile_accepted_b10`.
4. `cargo test -p atlas-core --lib`, `cargo clippy -p atlas-core --lib --tests -- -D warnings`, `cargo fmt --all -- --check` clean.

## Report

- Files changed; how warning-vs-error severity is represented; where recovery
  happens; check_fixture + regression + suite tails.

# Brief L2: reproduce bison-style syntax error messages in the Rust parser

You are working in `/Users/hoxide/mycodes/atlas-rust` (Rust reimplementation of
the Atlas of Lie Groups language; upstream C++/CWEB read-only reference at
`/Users/hoxide/mycodes/atlasofliegroups`, grammar at
`sources/interpreter/parser.y`). The original Atlas parser is bison-generated;
its syntax errors read `syntax error, unexpected X, expecting Y`. The Rust
hand-written parser currently reports a generic `unexpected token` /
`unexpected end of input`. Your job: make the parser emit bison-style messages
so the frozen contracts below compare VERBATIM, without breaking any of the
other wired fixtures.

## Scope discipline

- Only `crates/atlas-core` parser/lexer side (the parser is hand-written
  recursive descent; find the error-report sites).
- Do NOT touch `tests/`, `hpc/`, `docs/`, events/meta files, or evaluator
  diagnostic wordings (another slice owns those).
- No git commits. Leave edits in the working tree.

## Frozen targets (from HPC capture job 3503334; events already frozen)

1. `parse/negative_trailing_token` (`1 2`):
   `syntax error, unexpected INT, expecting '\n'`
2. `commands/invalid_token_continues` (`1 $ + 2` then `3`):
   `syntax error, unexpected '$', expecting '\n'`; recovery must still reach
   line 2 (`Value: 3` on stdout — already works).
3. `commands/mismatched_delimiter_continues` (`(1]` then `2`):
   `syntax error, unexpected ']', expecting ','`
4. `commands/nested_invalid_token_continues` (`(``` + line `2`):
   `syntax error, unexpected $undefined, expecting -> or '|'` — the two
   backticks lex as an unknown token (bison's `$undefined`), and the parser
   was inside a parenthesised/case context expecting `->` or `|`. Check the
   fixture `tests/fixtures/commands/nested_invalid_token_continues.atlas`
   and grammar to see which production is active; the CLI must ALSO recover
   and print `Value: 2` for line 2 (currently it aborts the whole input —
   oracle recovery discards the bad command and continues at the next line;
   the current CLI already recovers for the sibling fixtures, so align this
   case with them).
5. `commands/container_syntax_errors`:
   - `[1,]` -> `syntax error, unexpected ']'`
   - `[1 2]` -> `syntax error, unexpected INT, expecting ']'`
   - `(1,]` -> `syntax error, unexpected ']'`
   - The trailing dangling `[` (line 7) is EXCLUDED from the runnable plan
     (the oracle saw the harness-appended `quit` where the CLI sees EOF), so
     you only need the first three messages verbatim.

## Token naming (bison conventions, from parser.y)

- Named tokens print unquoted uppercase: `INT`, `QUIT`, ...
- Single-char tokens print quoted: `']'`, `'$'`, `'\n'`, `','`, `'='`
- The invalid/unknown token prints `$undefined`
- Multi-char operators print unquoted: `:=`
- Expected lists: `expecting X` or `expecting X or Y` (bison ordering follows
  the grammar state; pin the exact strings above from the frozen events —
  `tests/reference/<name>.events.json`).

## Verification (must all pass)

1. `export PATH="$HOME/.cargo/bin:$PATH"`.
2. `cargo build -p atlas-cli`, then
   `python3 /tmp/check_fixture.py parse/negative_trailing_token commands/invalid_token_continues commands/mismatched_delimiter_continues commands/nested_invalid_token_continues`
   — all VERBATIM. For `commands/container_syntax_errors` the CLI must emit
   the three messages above verbatim on stderr (the fixture as a whole is
   checked later through the harness plan, which excludes the dangling `[`).
3. Full local regression (guard against breaking verified fixtures):
   ```
   cd /tmp && rm -rf R && mkdir -p R/workspace && cp -R /Users/hoxide/mycodes/atlas-rust/tests R/workspace/ && cd R && python3 /Users/hoxide/mycodes/atlas-rust/hpc/pipeline_swap_diff.py /Users/hoxide/mycodes/atlas-rust/target/debug/atlas-cli out --workspace-root workspace --fixture-root /Users/hoxide/mycodes/atlas-rust/tests/fixtures --reference-root /Users/hoxide/mycodes/atlas-rust/tests/reference --commit local --dirty-tree true --detected-commit local --detected-dirty-tree true --job-id local --source-snapshot-sha256 local 2>&1 | tail -3
   ```
   The only allowed FAIL is `eval/fromfile_accepted_b10`. B-slice rejected
   fixtures with syntax errors must stay verbatim — if your message change
   alters their diagnostics, you must extend THOSE frozen events only by
   stopping and reporting (do not edit events).
4. `cargo test -p atlas-core --lib`, `cargo clippy -p atlas-core --lib --tests -- -D warnings`, `cargo fmt --all -- --check` clean.

## Design guidance (not prescriptive)

Carry bison-style token display names in the lexer token type, and let each
parser error site supply its expected-token description (recursive descent
makes the active production explicit). Keep it minimal: you need the sites
above plus not regressing existing `unexpected token` consumers — check which
verified fixtures assert syntax diagnostics and keep their messages intact.

## Report

- Files changed; the token-name table; how expected sets are computed per
  site; per-fixture before/after; check_fixture + regression + suite tails;
  any verified fixture whose syntax diagnostic text changed (should be none —
  if any, explain).

# Brief L3: `set quiet`/`set verbose` option commands + verbose analysis traces

You are working in `/Users/hoxide/mycodes/atlas-rust` (Rust reimplementation of
the Atlas of Lie Groups language; upstream C++/CWEB read-only reference at
`/Users/hoxide/mycodes/atlasofliegroups`). The original interpreter supports
the option commands `set quiet` / `set verbose` (a session verbosity flag) and,
in verbose mode, prints an analysis trace for every accepted expression
command. The Rust CLI has no `set`-option support at all. Your job: port it so
the frozen contract `lex/basic` compares VERBATIM.

## Prerequisite

This slice is sequenced AFTER the bison syntax-message slice (L2). The fixture
needs L2's `syntax error, unexpected :=, expecting '='` message for its line 2.
If that message is not yet in place when you run, note it and implement the
rest; do not implement L2's general syntax-message work yourself.

## Scope discipline

- Only `crates/atlas-core` (session/command loop, verbosity state, trace
  printing) and if needed `crates/atlas-cli`.
- Do NOT touch `tests/`, `hpc/`, `docs/`, events/meta files.
- No git commits. Leave edits in the working tree.

## Frozen target (HPC capture job 3503334; events already frozen)

Fixture `tests/fixtures/lex/basic.atlas`:
```
set verbose
let x := 42 in x + 1
"a""b" { comment }
```
Expected observation (`tests/reference/lex/basic.events.json`):
- stdout:
  ```
  Expression before type analysis: "a"b"
  Type found: string
  Converted expression: "a"b"
  Value: "a"b"
  Bye.
  ```
- one diagnostic: category `syntax`, message `syntax error, unexpected :=, expecting '='`
- exit status 1

So: line 1 sets verbosity (no output), line 2 fails analysis with the bison
message (L2 provides it; recovery already continues to line 3), line 3 is a
string-adjacency concatenation evaluated with the verbose trace.

## Upstream anchors (study these)

- `sources/interpreter/parser.y:171-178`: `SET IDENT '\n'` option command —
  `quiet` is option 0, `verbose` is option 1 (the first two identifiers in the
  main hash table); unknown option prints `'X' is not something one can set`
  on stderr; YYABORT (no expression output for the set command itself).
- `sources/interpreter/main.w:495-516`: the main loop — after yyparse, if
  verbosity==1 print `Expression before type analysis: <parsetree>` (the raw
  parse tree Display).
- `sources/interpreter/main.w:528-540`: after type analysis, if verbosity>0
  print `Type found: <type>` and `Converted expression: <converted expr>`,
  then `Value: <v>` (void results suppress the Value line as usual).
- The verbosity flag persists across commands until changed by another
  `set quiet`/`set verbose`.

The two trace lines need Display implementations for the pre-analysis parse
tree and the post-analysis converted expression. For the frozen contract only
the string literal case is gated (`"a"b"` in both places). Port the Display
for the common node shapes faithfully (literals, operators, calls) but keep it
minimal — do not build a full parsetree pretty-printer beyond what the trace
needs; record in your report which node shapes are covered.

## Verification (must all pass)

1. `export PATH="$HOME/.cargo/bin:$PATH"`.
2. `cargo build -p atlas-cli`, then `python3 /tmp/check_fixture.py lex/basic`
   — must print VERBATIM (if the syntax message is still L2-pending, verify
   everything else matches and report that single remaining diff).
3. `set verbose` must not alter behavior of other fixtures: full local
   regression —
   ```
   cd /tmp && rm -rf R && mkdir -p R/workspace && cp -R /Users/hoxide/mycodes/atlas-rust/tests R/workspace/ && cd R && python3 /Users/hoxide/mycodes/atlas-rust/hpc/pipeline_swap_diff.py /Users/hoxide/mycodes/atlas-rust/target/debug/atlas-cli out --workspace-root workspace --fixture-root /Users/hoxide/mycodes/atlas-rust/tests/fixtures --reference-root /Users/hoxide/mycodes/atlas-rust/tests/reference --commit local --dirty-tree true --detected-commit local --detected-dirty-tree true --job-id local --source-snapshot-sha256 local 2>&1 | tail -3
   ```
   Only allowed FAIL: `eval/fromfile_accepted_b10`.
4. `cargo test -p atlas-core --lib`, `cargo clippy -p atlas-core --lib --tests -- -D warnings`, `cargo fmt --all -- --check` clean.

## Report

- Files changed; where the verbosity flag lives; how `set <option>` is
  recognized (including the unknown-option message); which Display shapes the
  trace covers; check_fixture + regression + suite tails.

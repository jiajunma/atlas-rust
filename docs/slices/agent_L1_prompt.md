# Brief L1: align four legacy diagnostic wordings with the frozen oracle contracts

You are working in `/Users/hoxide/mycodes/atlas-rust` (Rust reimplementation of the
Atlas of Lie Groups language; upstream C++ read-only reference at
`/Users/hoxide/mycodes/atlasofliegroups`). Four legacy fixtures are frozen
verbatim from HPC capture job 3503334 but the Rust CLI's diagnostic MESSAGES
diverge. Your job: change the CLI message wording so all four fixtures compare
VERBATIM, without breaking any of the other 135 wired fixtures.

## Scope discipline

- Only `crates/atlas-core` (evaluator/typecheck/runtime diagnostic sites).
- Do NOT touch `tests/`, `hpc/`, `docs/`, events/meta files, or the parser/lexer
  (other slices own those).
- No git commits. Leave edits in the working tree.

## The four divergences (oracle message -> current CLI message)

1. `commands/assignment_errors` (tests/fixtures/commands/assignment_errors.atlas):
   - oracle: `Undefined identifier 'missing' in assignment missing:=2`
   - current: `Undefined identifier 'missing' in assignment`
   - The oracle appends the assignment's source text (`missing:=2`, no spaces).
   Upstream anchor: grep `in assignment` in the upstream sources
   (`sources/interpreter/atlas-types.w`) for the exact construction.

2. `commands/slice_errors`:
   - oracle `lower bound -1 out of range (should be >=0) in slice [1,2][-1:1]`
     vs current same prefix but ending `in slice` (no source text appended).
   - oracle `upper bound 3 out of range (should be <=2) in slice [1,2][0:3]`
     vs current `... (should be <= 2) in slice` (note: oracle has NO space in
     `<=2`, and appends the slice source text).
   - oracle `both bounds -1:3 out of range (should be >=0 respectively <=2) in slice [1,2][-1:3]`
     vs current `... respectively <= 2) in slice`.
   - oracle type error `Cannot slice value of type (int,int)`
     vs current `slice requires a row, found (int,int)`.
   Upstream anchors: grep `out of range` / `Cannot slice` in
   `sources/interpreter/atlas-types.w`.

3. `commands/subscription_errors`:
   - oracle type error `Cannot subscript value of type [int] with index of type bool`
     vs current `found bool while int was needed.` — for a bool index into a row
     value the oracle raises a dedicated cannot-subscript analysis error, not a
     coercion failure. Anchor: grep `Cannot subscript` upstream.

4. `eval/container_errors`:
   - oracle `No common type found between components of list expression: { [int], [string] }`
     (also `{ (int,int), (bool,string) }` and `{ int, string }`)
     vs current `branches have incompatible types [int] and [string]`.
   Anchor: grep `No common type found` upstream.

## Verification (must all pass)

1. `export PATH="$HOME/.cargo/bin:$PATH"`.
2. `cargo build -p atlas-cli` then
   `python3 /tmp/check_fixture.py commands/assignment_errors commands/slice_errors commands/subscription_errors eval/container_errors`
   — all four must print VERBATIM.
3. Full local regression (guard against breaking verified fixtures):
   ```
   cd /tmp && rm -rf R && mkdir -p R/workspace && cp -R /Users/hoxide/mycodes/atlas-rust/tests R/workspace/ && cd R && python3 /Users/hoxide/mycodes/atlas-rust/hpc/pipeline_swap_diff.py /Users/hoxide/mycodes/atlas-rust/target/debug/atlas-cli out --workspace-root workspace --fixture-root /Users/hoxide/mycodes/atlas-rust/tests/fixtures --reference-root /Users/hoxide/mycodes/atlas-rust/tests/reference --commit local --dirty-tree true --detected-commit local --detected-dirty-tree true --job-id local --source-snapshot-sha256 local 2>&1 | tail -3
   ```
   The only allowed FAIL is `eval/fromfile_accepted_b10` (a known HPC-path
   permission). Any other FAIL means your wording change broke a verified
   fixture — find and fix (do NOT edit events to match you).
4. `cargo test -p atlas-core --lib`, `cargo clippy -p atlas-core --lib --tests -- -D warnings`, `cargo fmt --all -- --check` clean.

If an existing unit test pins the old wording, update the test to the oracle
wording (the oracle is authoritative) and say so in your report. If a verified
fixture relies on the old wording, STOP and report the conflict instead of
forcing it.

## Report

- Files changed, per-message before/after, upstream anchors (file:line).
- check_fixture output, regression tail, three-piece suite tails.
- Any conflicts or judgment calls.

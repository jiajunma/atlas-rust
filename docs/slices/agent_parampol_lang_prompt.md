# Brief: parampol_basic language slice (ParamPol domain value + overloads)

You are wiring ParamPol (finite formal sums of standard parameters with Split
coefficients) into the Atlas language layer of
`/Users/hoxide/mycodes/atlas-rust`. The Param slice (domain value + crate
StandardRepr) has already landed; your job is the ParamPol surface so the
frozen contract `domain/parampol_basic` compares VERBATIM.

## Scope discipline

- `crates/atlas-core/src/typed.rs`, `crates/atlas-core/src/domain_builtins.rs`,
  and the domain value definitions (follow the Param/KTypePol patterns; the
  two pol containers should share structure).
- Do NOT touch `tests/`, `hpc/`, `docs/`.
- No git commits. Leave edits in the working tree.
- Only ONE language slice runs at a time on `typed.rs`/`domain_builtins.rs`;
  if the tree has uncommitted changes there when you start, STOP and report.

## Frozen contract (capture-verified; events are authoritative)

`tests/fixtures/domain/parampol_basic.atlas` (split A1, x = KGB(rf,2), p =
param(x,[0],[0]/1)):
- `V := null_module(rf)` displays `Empty sum of standard modules`; `#V` = 0
- `W := V+p` displays `\n1*parameter(x=2,lambda=[1]/1,nu=[0]/1) [0]` — note
  `1*parameter(...)` has NO space after `*` (contrast KTypePol's `1* K_type`
  whose print_K_type carries a leading space); ` [0]` is the height
- `#W` = 1
- `first_term(W)` = `((1+0s),final parameter(x=2,lambda=[1]/1,nu=[0]/1))` —
  Split in FULL `(e+fs)` form, Param WITH the adjective prefix
- `W-p` displays `Empty sum of standard modules` (like-term cancellation)

## Upstream anchors

- HANDOFF REP RECON (docs/HANDOFF.md): the ParamPol install list is
  `sources/interpreter/atlas-types.w:8542-8570` — null_module(RealForm->ParamPol)
  :8542, real_form :8544, # :8546 (TERM count), =/!= :8548-8552,
  +(ParamPol,Param) :8554 / -(ParamPol,Param) :8555 merging like terms,
  add-termlist forms :8556-8558, first_term(ParamPol->Split,Param) :8567.
  The gated subset here is: null_module, #, +, -, first_term. The chunk's
  =/!=/K_type_pol/scaling/last_term/truncate/scale-by-rat and
  deform/twisted_deform/block_deform await their own contracts — do NOT
  register them.
- `sources/io/basic_io.cpp:214` print_SR_poly — per-term: coefficient
  embellishment (same rule as KTypePol: full print_split only when both e and
  s occur across terms, else bare e / `{s}s`), then `*` + print_stdrep
  `parameter(x=N,lambda=[..]/d,nu=[..]/d)` (NO leading space) + ` [{height}]`,
  one `\n` per term; empty -> `Empty sum of standard modules`.

## Implementation notes

- ParamPol mirrors KTypePol over StandardRepr; merging is on StandardRepr
  equality.
- Display: term Param prints WITHOUT the adjective chain inside a pol
  (contrast first_term, WITH adjectives); empty sum text differs from
  KTypePol's.

## Verification (must all pass)

1. `export PATH="$HOME/.cargo/bin:$PATH"`.
2. `cargo build -p atlas-cli`, then
   `python3 /tmp/check_fixture.py domain/parampol_basic` — VERBATIM.
3. Full local regression (only allowed FAIL: `eval/fromfile_accepted_b10`):
   ```
   cd /tmp && rm -rf R && mkdir -p R/workspace && cp -R /Users/hoxide/mycodes/atlas-rust/tests R/workspace/ && cd R && python3 /Users/hoxide/mycodes/atlas-rust/hpc/pipeline_swap_diff.py /Users/hoxide/mycodes/atlas-rust/target/debug/atlas-cli out --workspace-root workspace --fixture-root /Users/hoxide/mycodes/atlas-rust/tests/fixtures --reference-root /Users/hoxide/mycodes/atlas-rust/tests/reference --commit local --dirty-tree true --detected-commit local --detected-dirty-tree true --job-id local --source-snapshot-sha256 local 2>&1 | tail -3
   ```
4. `cargo test -p atlas-core --lib` (+ `-p atlas-real-group --lib` if touched),
   `cargo clippy -p atlas-core -p atlas-real-group --lib --tests -- -D warnings`,
   `cargo fmt --all -- --check` clean.

## Report

- Files changed; registered overload table; display assembly; check_fixture +
  regression + suite tails; deferred entries.

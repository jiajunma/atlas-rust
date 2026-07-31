# Brief: param_basic language slice (Param domain value + overloads)

You are wiring the Rep_context crate milestone (already landed in
`crates/atlas-real-group`) into the Atlas language layer of
`/Users/hoxide/mycodes/atlas-rust`. The crate provides StandardRepr and the
Rep_context operations (sr_gamma/sr/gamma, KType conversion); your job is the
language surface so the frozen contracts `domain/param_basic` and
`domain/param_basic_rejected` compare VERBATIM.

## Scope discipline

- `crates/atlas-core/src/typed.rs` (overload registrations),
  `crates/atlas-core/src/domain_builtins.rs` (dispatch + validation), and the
  domain value definitions (follow the KType slice pattern; Param and KType
  share the crate Rep_context).
- Do NOT touch `crates/atlas-real-group`, `tests/`, `hpc/`, `docs/`.
- No git commits. Leave edits in the working tree.
- Only ONE language slice runs at a time on these files; if the tree has
  uncommitted changes on `typed.rs`/`domain_builtins.rs` when you start, STOP
  and report instead of merging.

## Frozen contracts (capture-verified; events are authoritative)

`tests/fixtures/domain/param_basic.atlas` (split A1, x = KGB(rf,2)):
- `p := param(x,[0],[0]/1)` displays `final parameter(x=2,lambda=[1]/1,nu=[0]/1)`
- `%p` = `(KGB element #2,[ 0 ],[ 0 ]/1)` — NOTE the third component is the
  info character gamma, NOT the input nu (atlas-types.w:6252; here gamma=[0]/1
  because lambda projects to 0 on the split Cartan)
- `height(p)` = 0
- `is_standard(p)`=true, `is_final(p)`=true, `is_zero(p)`=false
- `real_form(p)` = the split form
- `K_type(p)` = `final K-type K_type(x=2, lambda=[1]/1)` (sr_K restrict)
- `param(K_type(x,[0]))=p` true (param(KType) = sr with nu=0)

`tests/fixtures/domain/param_basic_rejected.atlas`:
- `param(x,[0,0],[0]/1)` -> runtime `Rank mismatch: (1,2,1)`
- `param(x,[0],[0,0]/1)` -> runtime `Rank mismatch: (1,1,2)`
- exit 1 after the first failing line? NO — check the events: BOTH
  diagnostics appear, so evaluation continues after a runtime error in batch
  mode for this fixture. Match the events exactly.

## Upstream anchors

- `sources/interpreter/atlas-types.w:6215` — param(KGBElt,vec,ratvec->Param) =
  Rep_context::sr(x,lam_rho,nu); rank check `Rank mismatch: ({rank},{lam_size},{nu_size})`.
- `atlas-types.w:6252` — %(Param->KGBElt,vec,ratvec) = (x, rc().lambda_rho(val),
  val.gamma()).
- `sources/representation/repr.h:242` — sr(x,lam,nu) = sr_gamma(x,lam,gamma(x,lam,nu)).
- `sources/io/basic_io.cpp` print_stdrep — `parameter(x=N,lambda=[..]/d,nu=[..]/d)`
  (no space after commas); the display adjective chain is the same 6-way chain
  as KType (`final ` here).
- SLICE BOUNDARY (atlas-types.w install chunk): register ONLY the
  fixture-gated set — param constructor, %, height, real_form, K_type(Param),
  param(KType), =/!=, is_standard/is_final/is_zero. The chunk's
  equivalent/is_dominant/is_semifinal/dominant/normal/cross/Cayley/twist etc.
  await their own contracts — do NOT register them.

## Implementation notes

- Add a `Param` domain value variant carrying the crate StandardRepr plus the
  owning RealForm context.
- The `param` wrapper has two arities (KGBElt,vec,ratvec) and (KType); the
  `K_type(Param)` wrapper restricts via the crate.
- Rank check before the crate call (upstream order), runtime diagnostic
  wording exactly `Rank mismatch: (r,l,n)`.

## Verification (must all pass)

1. `export PATH="$HOME/.cargo/bin:$PATH"`.
2. `cargo build -p atlas-cli`, then
   `python3 /tmp/check_fixture.py domain/param_basic domain/param_basic_rejected`
   — both VERBATIM.
3. Full local regression (only allowed FAIL: `eval/fromfile_accepted_b10`):
   ```
   cd /tmp && rm -rf R && mkdir -p R/workspace && cp -R /Users/hoxide/mycodes/atlas-rust/tests R/workspace/ && cd R && python3 /Users/hoxide/mycodes/atlas-rust/hpc/pipeline_swap_diff.py /Users/hoxide/mycodes/atlas-rust/target/debug/atlas-cli out --workspace-root workspace --fixture-root /Users/hoxide/mycodes/atlas-rust/tests/fixtures --reference-root /Users/hoxide/mycodes/atlas-rust/tests/reference --commit local --dirty-tree true --detected-commit local --detected-dirty-tree true --job-id local --source-snapshot-sha256 local 2>&1 | tail -3
   ```
4. `cargo test -p atlas-core --lib`, `cargo clippy -p atlas-core --lib --tests -- -D warnings`, `cargo fmt --all -- --check` clean; `cargo test -p atlas-real-group --lib` stays green.

## Report

- Files changed; registered overload table; how gamma flows into the %
  decompose; check_fixture + regression + suite tails.

# Brief: ktype_basic language slice (KType domain value + overloads)

You are wiring the Rep_context crate milestone (already landed in
`crates/atlas-real-group`) into the Atlas language layer of
`/Users/hoxide/mycodes/atlas-rust`. The crate provides the KType math
(StandardRepr/sr_K normalization, predicates); your job is the language
surface so the frozen contracts `domain/ktype_basic` and
`domain/ktype_basic_rejected` compare VERBATIM.

## Scope discipline

- `crates/atlas-core/src/typed.rs` (overload registrations),
  `crates/atlas-core/src/domain_builtins.rs` (dispatch + validation), and the
  domain value definitions (find where RootDatum/InnerClass/RealForm/KGBElt/
  Block values are declared — likely `value.rs`/`domain*.rs`; follow the Block
  slice 4167249 / involution_primitive 152f4b8 patterns).
- Do NOT touch `crates/atlas-real-group` (the crate milestone is frozen for
  this slice), `tests/`, `hpc/`, `docs/`.
- No git commits. Leave edits in the working tree.
- Only ONE language slice runs at a time on these files; if the tree has
  uncommitted changes on `typed.rs`/`domain_builtins.rs` when you start, STOP
  and report instead of merging.

## Frozen contracts (capture-verified; events are authoritative)

`tests/fixtures/domain/ktype_basic.atlas` (split A1, x = KGB(rf,2)):
- `K := K_type(x,[0])` displays `final K-type K_type(x=2, lambda=[1]/1)`
- `%K` = `(KGB element #2,[ 0 ])` (tuple: KGBElt, vec of the ELECTED lam_rho)
- `height(K)` = 0
- `is_standard(K)`=true, `is_dominant(K)`=true, `is_zero(K)`=false,
  `is_final(K)`=true, `is_semifinal(K)`=true
- `real_form(K)` = the split form
- `dominant(K)`=`normal(K)`=`theta_stable(K)`=K (same display)
- `K2 := K_type(x,[2])`; `K=K2` true; `equivalent(K,K2)` true

`tests/fixtures/domain/ktype_basic_rejected.atlas`:
- `K_type(x,[0,0])` -> diagnostic category `runtime`,
  message `Rank mismatch: (1,2)` (rank 1, supplied size 2), exit 1

## Upstream anchors (read for exact semantics)

- `sources/interpreter/atlas-types.w:6071-6088` — the 16-entry K_type install
  list; this slice registers exactly the fixture-gated subset:
  K_type(KGBElt,vec->KType), %(KType->KGBElt,vec), real_form(KType->RealForm),
  height(KType->int), =/!=(KType,KType), equivalent(KType,KType->bool),
  is_standard/is_dominant/is_zero/is_semifinal/is_final(KType->bool),
  dominant/normal/theta_stable(KType->KType).
  (to_canonical_fiber is in the install list; register it too if the crate
  exposes it cheaply, otherwise note it.)
- `atlas-types.w:5240` — the rank check producing `Rank mismatch: ({rank},{size})`.
- `sources/io/basic_io.cpp` print_K_type — `K_type(x=N, lambda=[..]/d)`; the
  adjective chain (atlas-types.w:5210-5224): the display prefix is the
  adjective sequence `non-standard ` / `non-dominant ` / `zero ` /
  `non-final ` / `non-normal ` / `final ` then `K-type ` + print_K_type.
  Match the oracle: `final K-type K_type(x=2, lambda=[1]/1)`.

## Implementation notes

- Add a `KType` domain value variant carrying the crate's KType object plus
  the owning RealForm context (follow how Block carries its graph + forms).
- The `K_type` wrapper: evaluate args, rank-check BEFORE calling the crate
  (upstream order), map crate errors to the runtime diagnostic wording.
- Display goes through the same domain-display path as other domain values;
  the adjective chain is computed from the crate predicates.
- `%` on KType returns the tuple (KGBElt, vec) — check how `%` is dispatched
  for other domain values in typed.rs/domain_builtins.rs and register the
  KType row.
- `=`/`!=` and `equivalent` need the overload registrations; equality on
  KType is on normalized forms (crate provides it).

## Verification (must all pass)

1. `export PATH="$HOME/.cargo/bin:$PATH"`.
2. `cargo build -p atlas-cli`, then
   `python3 /tmp/check_fixture.py domain/ktype_basic domain/ktype_basic_rejected`
   — both VERBATIM.
3. Full local regression (guard):
   ```
   cd /tmp && rm -rf R && mkdir -p R/workspace && cp -R /Users/hoxide/mycodes/atlas-rust/tests R/workspace/ && cd R && python3 /Users/hoxide/mycodes/atlas-rust/hpc/pipeline_swap_diff.py /Users/hoxide/mycodes/atlas-rust/target/debug/atlas-cli out --workspace-root workspace --fixture-root /Users/hoxide/mycodes/atlas-rust/tests/fixtures --reference-root /Users/hoxide/mycodes/atlas-rust/tests/reference --commit local --dirty-tree true --detected-commit local --detected-dirty-tree true --job-id local --source-snapshot-sha256 local 2>&1 | tail -3
   ```
   Only allowed FAIL: `eval/fromfile_accepted_b10`.
4. `cargo test -p atlas-core --lib`, `cargo clippy -p atlas-core --lib --tests -- -D warnings`, `cargo fmt --all -- --check` clean. Also
   `cargo test -p atlas-real-group --lib` must stay green (you did not touch
   that crate).

## Report

- Files changed; the registered overload table (name, signature, dispatch);
  how the adjective chain is computed; where the rank check lives;
  check_fixture + regression + suite tails; anything deferred (e.g.
  to_canonical_fiber).

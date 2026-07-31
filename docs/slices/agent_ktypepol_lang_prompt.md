# Brief: ktypepol_basic language slice (KTypePol domain value + overloads)

You are wiring KTypePol (finite formal sums of K-types with Split
coefficients) into the Atlas language layer of
`/Users/hoxide/mycodes/atlas-rust`. The KType slice (domain value + crate
Rep_context) has already landed; your job is the KTypePol surface so the
frozen contract `domain/ktypepol_basic` compares VERBATIM.

## Scope discipline

- `crates/atlas-core/src/typed.rs`, `crates/atlas-core/src/domain_builtins.rs`,
  and the domain value definitions (follow the KType slice pattern). The
  polynomial container itself (term merging, coefficient arithmetic) may live
  in `crates/atlas-real-group` next to KType if it is pure math, or in
  atlas-core if it is thin — judge by the existing KType placement and keep it
  minimal.
- Do NOT touch `tests/`, `hpc/`, `docs/`.
- No git commits. Leave edits in the working tree.
- Only ONE language slice runs at a time on `typed.rs`/`domain_builtins.rs`;
  if the tree has uncommitted changes there when you start, STOP and report.

## Frozen contract (capture-verified; events are authoritative)

`tests/fixtures/domain/ktypepol_basic.atlas` (split A1, x = KGB(rf,2), K =
K_type(x,[0])):
- `P := null_K_module(rf)` displays `Empty sum of K-types`; `#P` = 0
- `Q := P+K` displays `\n1* K_type(x=2, lambda=[1]/1) [0]` (leading newline,
  ONE `\n` per term; coefficient `1*` + ` K_type(...)` — note the SPACE after
  `*` because print_K_type has a leading space; ` [0]` is the height)
- `#Q` = 1 (TERM count, not coefficient sum)
- `R := Q+K` displays `\n2* K_type(x=2, lambda=[1]/1) [0]` (like terms merge,
  coefficient doubles); `#R` = 1
- `real_form(Q)` = the split form
- `first_term(Q)` = `last_term(Q)` = `((1+0s),final K-type K_type(x=2, lambda=[1]/1))`
  — a tuple whose Split prints in FULL `(e+fs)` form and whose KType prints
  WITH the adjective prefix
- `2*Q` displays `\n2* K_type(x=2, lambda=[1]/1) [0]`

## Upstream anchors

- `sources/interpreter/atlas-types.w:6091-6117` — the KTypePol install list;
  this slice registers the fixture-gated subset: null_K_module(RealForm),
  real_form, # (term count), +(KTypePol,KType), =(KTypePol,KTypePol) if
  needed by fixtures (check events), first_term/last_term(KTypePol->(Split,KType)),
  *(int,KTypePol). The +(KTypePol,(Split,KType)) / +(KTypePol,[(Split,KType)])
  / -(KTypePol,KType/Pol) / *(Split,KTypePol) / truncate_above_height forms
  are NOT gated by this fixture — register only what the fixture exercises
  (read the fixture source) and note the rest as deferred.
- `sources/io/basic_io.cpp:165` print_K_type_pol — per-term: coefficient
  embellishment (full print_split only when BOTH e and s components occur
  across terms, else bare e or `{s}s`), then `*` + ` K_type(x=N, lambda=rho+lam_rho)`
  (NO adjective prefix inside the pol) + ` [{height}]`, one `\n` per term;
  empty pol prints `Empty sum of K-types`.
- Coefficients are Split numbers (dual numbers e+fs over Z); the Split domain
  type already exists (eval split_basic, verified 3502718) — reuse its value
  representation.

## Implementation notes

- KTypePol = sorted/merged list of (Split coefficient, KType) terms over one
  RealForm. Merging is on KType equality (normalized forms).
- Display: term KType prints WITHOUT the adjective chain inside a pol
  (contrast first_term, where it prints WITH it); empty sum has its own text.
- `#` (length/count) on KTypePol returns the TERM count.
- `2*Q` is int * KTypePol scaling (Split coefficient promotion).

## Verification (must all pass)

1. `export PATH="$HOME/.cargo/bin:$PATH"`.
2. `cargo build -p atlas-cli`, then
   `python3 /tmp/check_fixture.py domain/ktypepol_basic` — VERBATIM.
3. Full local regression (only allowed FAIL: `eval/fromfile_accepted_b10`):
   ```
   cd /tmp && rm -rf R && mkdir -p R/workspace && cp -R /Users/hoxide/mycodes/atlas-rust/tests R/workspace/ && cd R && python3 /Users/hoxide/mycodes/atlas-rust/hpc/pipeline_swap_diff.py /Users/hoxide/mycodes/atlas-rust/target/debug/atlas-cli out --workspace-root workspace --fixture-root /Users/hoxide/mycodes/atlas-rust/tests/fixtures --reference-root /Users/hoxide/mycodes/atlas-rust/tests/reference --commit local --dirty-tree true --detected-commit local --detected-dirty-tree true --job-id local --source-snapshot-sha256 local 2>&1 | tail -3
   ```
4. `cargo test -p atlas-core --lib` (+ `-p atlas-real-group --lib` if you
   touched it), `cargo clippy -p atlas-core -p atlas-real-group --lib --tests -- -D warnings`,
   `cargo fmt --all -- --check` clean.

## Report

- Files changed; registered overload table; term-merging rule; display
  assembly (coefficient embellishment state); check_fixture + regression +
  suite tails; deferred install-list entries.

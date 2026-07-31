# Brief: Rep_context subset crate milestone (atlas-real-group)

You are implementing the **crate-level math core** for the KType/Param family in
atlas-rust, a Rust reimplementation of the Atlas of Lie Groups software. This is a
milestone inside a larger porting effort whose compatibility target is the original
C++ Atlas executable's observable behavior.

## Repository layout

- Working repo: `/Users/hoxide/mycodes/atlas-rust` (you work here; do NOT commit)
- Upstream read-only reference: `/Users/hoxide/mycodes/atlasofliegroups` (C++/CWEB
  sources, commit 4d3e9449). NEVER modify anything there.
- Crate to extend: `crates/atlas-real-group` (pure math crate, no unsafe allowed)
- The language layer (`crates/atlas-core/src/typed.rs`,
  `crates/atlas-core/src/domain_builtins.rs`) is **out of scope** — a later slice
  wires your API into the interpreter. Do not touch those files, the harness
  (`hpc/`), fixtures, events, or metadata.

## What to build

A `Rep_context` subset sufficient for the KType and StandardRepr surfaces, as new
module(s) in `crates/atlas-real-group/src/` (suggested: `rep_context.rs` and
`ktype.rs`, but follow existing module patterns). Everything must be grounded in
the upstream sources below, not invented.

### Upstream anchors (read these first)

- `sources/interpreter/atlas-types.w:6071-6088` — the K_type install list (16
  entries) defining the surface: K_type(KGBElt,vec->KType) = Rep_context::sr_K
  normalizing lambda-rho mod (1-theta_x)X*, rank check
  'Rank mismatch: ({rank},{size})' (atlas-types.w:5240); %(KType->KGBElt,vec)
  elected representative; real_form; height; =/!= on normalized forms;
  equivalent (SR-equivalence); is_standard ((1+theta)lambda imaginary-dominant)
  / is_dominant / is_zero (a singular compact simply-imaginary root exists) /
  is_semifinal (no real parity roots) / is_final; dominant / to_canonical_fiber
  / normal / theta_stable (KType->KType).
- `sources/interpreter/atlas-types.w:6215` — param(KGBElt,vec,ratvec->Param) =
  Rep_context::sr(x,lam_rho,nu), rank check
  'Rank mismatch: ({rank},{lam_size},{nu_size})'; :6252 — %(Param->KGBElt,vec,
  ratvec) = (x, rc().lambda_rho(val), val.gamma()) — NOTE the third component is
  the info character gamma, NOT the input nu.
- `sources/representation/repr.h:242` — sr(x,lam,nu) = sr_gamma(x,lam,gamma(x,lam,nu)).
- `sources/representation/repr.cpp:756` — sr_gamma(x,lam_rho,gamma) =
  StandardRepr(x, y_pack(i_x,lam_rho), gamma, height((1+theta)gamma)).
  Trace `gamma(...)`, `y_pack`, `height`, `lambda_rho`, `sr` in this file and
  repr.h for their exact math.
- `sources/representation/K_repr.cpp` (626 lines total) and `K_repr.h` — the
  K_type constructor with the mod-(1-theta)X* normalization, and the predicate
  set. Port the constructor + predicates faithfully; skip whatever belongs to
  later slices (branch/KL/deform are NOT in scope).
- `sources/io/basic_io.cpp` — print_K_type ('K_type(x=N, lambda=[..]/d)') and
  print_stdrep ('parameter(x=N,lambda=[..]/d,nu=[..]/d)'). The adjective chain
  (non-standard/non-dominant/zero/non-final/non-normal/final + ' K-type') is
  language-layer work, NOT yours; but your predicate set must support it.

### Existing pieces to reuse (verify before writing anything)

- `crates/atlas-real-group/src/involution_table.rs` — InvolutionTable (theta,
  cross/Cayley, torus bits) landed and verified (job 3502272).
- Tits coset reduction: seed_x0's `quotient_representative` (~ upstream y_pack);
  find it under `crates/atlas-real-group/src/` (grep for quotient_representative).
- KGB status/gradings: `kgb.rs` / status bits; `g_rho_check` exists (grep).
- `block.rs` — BlockGraph (4167249) if needed for context; probably not needed.
- Look at how `strong_real.rs` / `weak_real_form.rs` structure their public API
  and how `crates/atlas-core/src/domain_builtins.rs` consumes crate APIs — your
  surface will be consumed the same way later. Match those conventions
  (naming, error types, Display where present).

### Frozen acceptance anchors (from the HPC-verified oracle contracts)

Fixture `tests/fixtures/domain/ktype_basic.atlas` (split A1, x = KGB(rf,2)):
- `K := K_type(x,[0])` displays `final K-type K_type(x=2, lambda=[1]/1)`
  (stored lambda = lam_rho + rho; rho=[1] for A1).
- `%K` = `(KGB element #2,[ 0 ])`; `height(K)` = 0.
- is_standard=true, is_dominant=true, is_zero=false, is_final=true,
  is_semifinal=true.
- `real_form(K)` = the split SL(2,R) form; dominant(K)=normal(K)=theta_stable(K)=K.
- `K2 := K_type(x,[2])` equals K: K=K2 true, equivalent(K,K2) true
  (normalization mod (1-theta)X* = 2X* for this x).

Fixture `tests/fixtures/domain/param_basic.atlas` (same group):
- `p := param(x,[0],[0]/1)` displays `final parameter(x=2,lambda=[1]/1,nu=[0]/1)`.
- `%p` = `(KGB element #2,[ 0 ],[ 0 ]/1)` — third component gamma=[0]/1
  (lambda projects to 0 on the split Cartan).
- `height(p)` = 0; is_standard=true, is_final=true, is_zero=false.
- `real_form(p)` = split form; `K_type(p)` = the K above;
  `param(K_type(x,[0]))=p` true (param(KType) = sr with nu=0).

### Required work

1. Port the math: StandardRepr struct (x, y-component, gamma, height),
   Rep_context operations sr_gamma / sr / gamma / lambda_rho / height,
   KType with the sr_K normalization, and the predicate/operation set listed
   above. Keep it generic (any inner class/real form), driven by the existing
   InvolutionTable/Tits/KGB infrastructure — not an A1 special case, but you
   only need to be correct where the existing infrastructure reaches.
2. Unit tests (rust, in-crate): pin every anchor above, plus at least one
   compact-form case if the existing infrastructure makes it cheap (e.g. SU(2)
   compact where theta=1: check sr_K normalization is trivial and is_zero
   behavior). Follow the existing test style in the crate.
3. Doc comments citing the upstream file:line for each ported function,
   matching the density/style of neighboring modules (e.g. block.rs,
   involution_table.rs).
4. Keep the public API minimal and typed for later language-layer consumption:
   the language slice will need, per value: x index, lam_rho vector (elected),
   gamma (ratvec), height, predicates, real_form identity, KType<->Param
   conversions, equality and SR-equivalence.

## Hard rules

- `export PATH="$HOME/.cargo/bin:$PATH"` for cargo.
- No git commits. Leave the working tree with your edits only.
- No unsafe. Follow AGENTS.md hard rules; minimal, focused changes.
- Verify: `cargo test -p atlas-real-group --lib`,
  `cargo clippy -p atlas-real-group --lib --tests -- -D warnings`,
  `cargo fmt --all -- --check` must all be clean. Also run
  `cargo test -p atlas-core --lib` to prove you broke nothing (you should not
  have touched atlas-core at all).
- If an upstream passage is ambiguous, prefer the reading that makes the
  frozen anchors verbatim; record the decision in your report.

## Deliverable report (final message)

- Files changed with line counts.
- The public API surface (types + functions with one-line semantics).
- Where each upstream anchor lives in your code (file:line -> your file:line).
- Test list and results; the three-command verification output tail.
- Any ambiguities/decisions taken, and anything deliberately left for the
  language slice.

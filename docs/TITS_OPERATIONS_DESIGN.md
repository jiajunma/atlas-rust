# Tits operations design (KGB stage c)

## Approved scope

Stage (c) of the KGB map: torus parts and the Tits-group operations the
KGB generation loop performs per element — the mod-2 tables, the
element representation, the based cross action, the Cayley transform,
gradings, and the reduction normal form. The seed (`x0`, the elected
cocharacter and base grading) is stage (d); the generation loop itself
is stage (e). This stage consumes the stage-(b) `InvolutionTable`
(canonical records, O(1) cross links, the Cayley edge) and the
stage-(a) `WeylElement` substrate.

## Atlas construction (oracle trace)

Citations are to `~/mycodes/atlasofliegroups`, master `4d3e9449`.

- A `TitsElt` is `t . sigma_w` with the `TorusPart` (a rank-bit vector
  in `X_* / 2X_*`) stored on the LEFT of the canonical Weyl lift
  (tits.h:323-334). Side conversion `push_across`/`pull_across` applies
  the per-letter `reflect` along a reduced word (tits.cpp:434-451);
  upstream's own comment names the sophistication this port adopts:
  precomputing the conjugation matrices (tits.cpp:425-432).
- Tables (tits.h:447-467, tits.cpp:379-395): `dual_m_alpha(s)` = simple
  root mod 2 (X^* side), `m_alpha(s)` = simple coroot mod 2 (X_* side),
  `d_involution` = mod-2 of delta-transpose acting on `X_*`
  (`twisted(x)`), and a `dual_involution` this stage does not need.
- `reflect(x,s)`: `if dual_m_alpha(s).dot(x) { x += m_alpha(s) }` —
  conjugation of `x` by `sigma_s`, which IS the mod-2 simple reflection
  on `X_*` (tits.h:515-526). `left_add` is a plain add;
  `right_add(a,t)` = `d_t += pull_across(d_w, t)` (tits.h:569-572).
- The four sigma multiplications (tits.cpp:469-503): `sigma_mult(s,a)`
  reflects the torus part, left-multiplies the Weyl part, and adds
  `m_alpha(s)` on the LEFT when the length DECREASES;
  `sigma_inv_mult` mirrors on increase; `mult_sigma`/`mult_sigma_inv`
  mutate the Weyl part FIRST and then `right_add(m_alpha(s))` on
  decrease/increase respectively — the pull crosses the ALREADY-UPDATED
  Weyl part. `twistedConjugate(a,s)` = `sigma_mult(s,a)` then
  `mult_sigma_inv(a, twisted(s))` (tits.h:598-599).
- `TitsCoset` (tits.h:660-671): a `grading_offset` bit per simple root
  (the sign in `delta(X_alpha) = +- X_{twist(alpha)}`).
  `simple_grading(a,s)` (true = noncompact) =
  `offset[s] XOR dual_m_alpha(s).dot(left_torus_part(a))`
  (tits.h:785-789). `basedTwistedConjugate(a,s)` = `twistedConjugate`
  plus `left_add(m_alpha(s))` when `offset[s]` (tits.h:810-816; valid
  modulo torus conjugation). `Cayley_transform(a,s)` is a bare
  `sigma_mult(s,a)` (tits.h:717-719). `inverse_Cayley_transform`
  (tits.cpp:605-644): `sigma_inv_mult(i,a)`, then if `alpha_i` failed to
  become noncompact, scan the SOURCE involution's mod-space basis IN
  ORDER for the first vector pairing oddly with `dual_m_alpha(i)` and
  left-add it; it is NOT used by KGB generation (inverse Cayley links
  are installed by inverting forward links, kgb.cpp:669-682).
- Gradings beyond simple roots: computed by CONJUGATING the root to a
  simple one through based cross actions (tits.cpp:661-672; same
  pattern kgb.cpp:818-831), not by a sum formula; the sum-over-support
  formula exists only for simple-imaginary roots at a torus element
  (tits.cpp:674-717).
- Reduction: `i_tab.reduce(a)` replaces the torus bits by the canonical
  coset representative modulo the involution's `fiber_denom` subspace
  (zero bits at the echelon pivot positions, subquotient.cpp:104-124),
  called after EVERY cross action and Cayley transform before the hash
  probe (kgb.cpp:563-564, 597-599). Bitwise equality on the reduced
  pair (Weyl part, torus bits) is the dedup relation (tits.h:377-381,
  624-635).
- Handoff boundary: during construction upstream carries full
  `TitsElt`s; it PERSISTS per element only the reduced left torus part
  plus an involution index (kgb.h:288-308, kgb.cpp:661-663) — exactly
  the pair this port's elements are made of from the start.
- Status assignment (stage (e)'s consumption, kgb.cpp:577-611):
  complex/real from Weyl tests, imaginary compact/noncompact from
  `simple_grading` alone.

## Port decisions and divergences

1. ELEMENT = `(InvolutionId, torus bits)` FROM THE START. Upstream
   carries the Weyl word per element during construction only because
   its table lacks O(1) edges; the stage-(b) table has stored cross
   links and the Cayley edge, so the port's `TitsElt` is the persisted
   shape already — `involution: InvolutionId` plus a left-stored
   `ModTwoVector`. No per-element Weyl data exists at any phase; at
   split-E8 scale this is 320,206 x (8 bits + one index) instead of
   320,206 transducer words.
2. MATRIX TRANSPORT replaces word walks. `pull_across(w, t)` equals the
   mod-2 action of `w` on `X_*`, so the port applies the mod-2
   reduction of the record's `WeylAction::coweight_matrix()` instead of
   walking a reduced word — the sophistication upstream's own comment
   names (tits.cpp:425-432), justified by the same
   honest-W-representation argument the stage-(a) semantics review
   verified. The right-multiplication correction in the based cross
   action pulls across the TARGET involution's matrix (upstream
   right-adds after mutating the Weyl part).
3. The based cross action becomes one CLOSED-FORM per-element map. With
   `w = record(n).weyl_element()`, `n' = cross(s, n)`,
   `w' = record(n').weyl_element()`:
   - `t' = reflect(t, s)`;
   - if `s` is a left descent of `w`: `t' += m_alpha(s)`
     (the `sigma_mult` decrease branch, sign from the stored element's
     O(1) descent query);
   - if `l(w') > l(s w)` (both lengths cached: `l(s w) = l(w) -+ 1`
     from the same descent): `t' += mod2(w') . m_alpha(twist(s))`
     (the `mult_sigma_inv` increase branch);
   - if `offset[s]`: `t' += m_alpha(s)`;
   - reduce modulo `record(n').mod_space()`.
   Every step is O(rank^2) bit work; no intermediate non-involution
   Weyl part is ever materialized.
4. Cayley transform: `n' = table.cayley(s, n)` (the stage-(b) edge;
   `None` remains the not-yet-added signal), torus part
   `t' = reflect(t, s)` plus `m_alpha(s)` exactly when `s` is a left
   descent of `w` — for a noncompact-imaginary `s` the length GROWS, so
   the correction never fires there, but the formula stays the general
   `sigma_mult` one — then reduce modulo the TARGET's mod-space (the
   subspace has grown, kgb.cpp:599).
5. Inverse Cayley is ported for completeness of the layer (consumers
   are real-form recovery paths, not KGB generation): `sigma_inv_mult`
   shape plus the ordered first-match repair over the SOURCE
   involution's `basis_vectors()`. The repair CHOICE depends on the
   echelon basis, which this port's RREF convention need not reproduce
   bitwise — recorded as adapter-level, same reduced class either way
   (the stage-(b) semantics review's finding).
6. The general imaginary-root grading uses upstream's
   conjugate-to-simple loop over based cross actions, not a sum
   formula. Included because stage (d)'s seed verification and
   post-KGB queries consume it; stage (e)'s loop touches only
   `simple_grading`.
7. `grading_offset` is an INPUT (stage (d) elects it from the
   square-class cocharacter; tests use hand values and the adjoint
   convention `offset[s] = (twist(s) == s)`, tits.cpp:564-588). The
   port stores it as a `Vec<bool>` over simple generators, validated
   against the semisimple rank and — matching upstream's
   `TitsCoset::is_valid` shape — the delta-compatibility contract is
   the caller's.
8. NORMAL-FORM CONTRACT, stated once: lookups, hashing, and numbering
   all ride on the reduction map; the port's `quotient_representative`
   (RREF, pivot bits zeroed) plays upstream's role exactly, but its
   basis convention is the crate's own, so RAW TORUS BITS are
   adapter-deferred observables. OPEN QUESTION for review (1): upstream
   `torus_factor` (kgb.cpp:699-712) symmetrizes
   `g - lift(torus bits)` by `(1 + theta^T)`, which annihilates the
   `(-1)`-eigenlattice that spans the mod-space ambiguity — the design
   believes `torus_factor` is therefore normal-form-INDEPENDENT and
   stays directly differential-comparable (as the stage map already
   claims); the semantics review must confirm or refute this
   annihilation argument.

## Data layout and public boundary

```text
// tits_element.rs (name avoids clash with the C++ "TitsGroup")
TitsElt {                                  // derives Clone, Debug, Eq, PartialEq, (Hash?)
    involution: InvolutionId,
    torus: ModTwoVector,                   // left-stored, lattice-rank bits
}
    involution() / torus_bits()

TitsCoset::new(&InnerClass, grading_offset: Vec<bool>) -> Result<...>
    // owns: m_alpha / dual_m_alpha tables (simple coroots/roots mod 2),
    // the mod-2 delta-transpose (twisted()), the validated offset, and a
    // clone of the inner class for provenance; does NOT own the table
grading_offset() -> &[bool]
simple_grading(&InvolutionTable, &TitsElt, generator) -> Result<bool>
cross(&InvolutionTable, generator, &TitsElt) -> Result<TitsElt>
    // closed-form based cross action, reduced at the target
cayley(&InvolutionTable, generator, &TitsElt) -> Result<Option<TitsElt>>
    // None while the target Cartan is not added (stage-(b) contract)
inverse_cayley(&InvolutionTable, generator, &TitsElt) -> Result<Option<TitsElt>>
grading(&InvolutionTable, &TitsElt, root: RootId) -> Result<bool>
    // conjugate-to-simple loop; Err on non-imaginary roots' invariant
reduce(&InvolutionTable, &TitsElt) -> Result<TitsElt>   // idempotent normal form
```

Operations take `&InvolutionTable` per call — the substrate idiom the
`WeylElement`/`&RootSystem` pairing established — because the table is
the heavyweight shared value stage (e) also owns; the coset gates
provenance by datum equality against the table's inner class, and the
element's `InvolutionId` is validated by the table's bounds. The mod-2
transport matrices are derived per call from the record's stored
`WeylAction` (an O(rank^2) reduction); caching them per involution is a
recorded optimization the table can adopt if the HPC preflight shows
the reduction binding — NOT taken now, per the crate's
correctness-first discipline.

Errors: `TitsCosetInvariantViolation { invariant }` for the grading
repair, conjugation termination, and offset-shape gates;
`IndexOutOfRange` for generator and involution bounds. No resource
variant: every operation is O(roots) or better with no enumeration.

## Resource and arithmetic policy

Per-operation work is O(rank^2) bit operations plus O(1) table lookups
(the imaginary-grading loop is O(length x rank^2) worst case, bounded
by the positive-root count). `ModTwoVector`/`ModTwoLinearMap` supply
checked dimensions; no new allocation pattern beyond `try_capacity`.

## Tests and fixture gate

- SL(2,R) (A1 split, offset [true]): the fundamental involution's
  element with zero bits is noncompact (`simple_grading` true); the
  Cayley transform lands at the split involution with the reduced
  torus part; cross fixes both elements; the three KGB elements'
  worth of (involution, bits) states are pairwise distinct under
  reduce.
- Compact SU(2) (A1, offset [false]): `simple_grading` false, no
  noncompact roots, no Cayley edge taken.
- Sp(4,R) (B2 split): closed-form cross action against the DEFINITION:
  for every element state and generator, compare the port's `cross`
  against a straight word-level recomputation (build the twisted
  conjugate at the WeylElement layer, transport the torus part by
  upstream's exact four-step sigma recipe using reduced words) — the
  matrix-transport shortcut must agree bit-for-bit BEFORE reduction,
  and after reduction land on the table's normal form.
- Grading by conjugation: on B2, the grading of every imaginary root
  at hand-picked torus bits agrees with the simple-imaginary sum
  formula evaluated directly (tits.cpp:704-717 shape) where both
  apply.
- Inverse Cayley: on SL(2,R), inverse of the forward Cayley recovers a
  preimage whose forward Cayley returns the reduced original; the
  compact-grading repair path is exercised.
- Offset validation: wrong-length and out-of-range offsets rejected;
  foreign-table provenance rejected by the datum gate.

`tests/fixtures/domain/tits_operations.atlas` is reserved; language
observables arrive at stage (e) (KGB sizes and statuses), per the
stage map.

## Consequential updates

Landing this stage must update: `lib.rs` (module and exports);
`error.rs` (the invariant family); `KGB_STAGE_MAP.md` (stage (c)
landed); `REAL_GROUP_DESIGN.md`'s progression (next: stage (d), the
seed). The stage-(b) design's record-scale note gains the observation
that stage (c) derives its transport matrices from the stored
`WeylAction`, strengthening the case for keeping it in the record.

## Three independent design checks

Before implementation, this design must be reviewed in three fresh
subagent contexts: (1) Atlas source semantics — the closed-form based
cross action against the four sigma multiplications (operand order,
which side each correction lands on, the target-matrix pull), the
Cayley/inverse-Cayley formulas, and the torus_factor annihilation
question; (2) Rust internals — the mod-2 machinery fit
(`ModTwoVector`/`ModTwoLinearMap` APIs, matrix reduction cost, the
closed-form map's bit algebra), element derives, and the
per-call-table calling convention; (3) public API and consumer fit —
what stage (d)'s seed election and stage (e)'s loop actually call, the
Option-vs-error shapes, naming against the crate vocabulary, and the
test plan's executability. Findings and corrections will be recorded
here before source edits begin.

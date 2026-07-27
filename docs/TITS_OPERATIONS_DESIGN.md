# Tits operations design (KGB stage c)

## Approved scope

Stage (c) of the KGB map: torus parts and the Tits-group operations the
KGB generation loop performs per element — the mod-2 tables, the
element representation, the based cross action, the Cayley transform,
gradings, and the reduction normal form. The seed (`x0`, the elected
cocharacter and base grading) is stage (d); the generation loop itself
is stage (e). This stage consumes the stage-(b) `InvolutionTable`
(canonical records, O(1) cross links, the Cayley edge) and the
stage-(a) `WeylElement` substrate. The INVERSE Cayley transform is
deferred out of this stage per review: KGB generation installs inverse
links by inverting forward links (kgb.cpp:669-682), and its only
upstream consumers (realredgp.cpp:238-239, standardrepk.cpp:1013) are
real-form-recovery paths beyond stage (f); the deferral note below
records what that future stage needs.

## Atlas construction (oracle trace)

Citations are to `~/mycodes/atlasofliegroups`, master `4d3e9449`.

- A `TitsElt` is `t . sigma_w` with the `TorusPart` (a rank-bit vector
  in `X_* / 2X_*`) stored on the LEFT of the canonical Weyl lift
  (tits.h:323-334). Side conversion `push_across`/`pull_across` applies
  the per-letter `reflect` along a reduced word (tits.cpp:434-451);
  upstream's own comment names the sophistication this port adopts:
  precomputing the conjugation matrices (tits.cpp:425-432).
  `pull_across(w, .)`, letters applied last to first, IS the mod-2
  action of `w` itself on `X_*` (`push_across` is `w^{-1}`).
- Tables (tits.h:447-467, tits.cpp:379-395): `dual_m_alpha(s)` = simple
  root mod 2 (X^* side), `m_alpha(s)` = simple coroot mod 2 (X_* side),
  `d_involution` = mod-2 of delta-transpose acting on `X_*`
  (`twisted(x)`), and a `dual_involution` this stage does not need.
- `reflect(x,s)`: `if dual_m_alpha(s).dot(x) { x += m_alpha(s) }` —
  conjugation of `x` by `sigma_s`, the mod-2 simple reflection on `X_*`
  (tits.h:515-526). `left_add` is a plain add; `right_add(a,t)` =
  `d_t += pull_across(d_w, t)` (tits.h:569-572).
- The four sigma multiplications (tits.cpp:469-503): `sigma_mult(s,a)`
  reflects the torus part, left-multiplies the Weyl part, and adds
  `m_alpha(s)` on the LEFT when the length DECREASES;
  `sigma_inv_mult` mirrors on increase; `mult_sigma`/`mult_sigma_inv`
  mutate the Weyl part FIRST and then `right_add(m_alpha(s))` on
  decrease/increase respectively — the pull crosses the ALREADY-UPDATED
  Weyl part. `twistedConjugate(a,s)` = `sigma_mult(s,a)` then
  `mult_sigma_inv(a, twisted(s))` (tits.h:598-599).
- `TitsCoset` (tits.h:660-671): a `grading_offset` bit per simple root
  (the sign in `delta(X_alpha) = +- X_{twist(alpha)}`); a strong
  involution FORCES `offset[s] == offset[twist(s)]` (tits.h:664-667).
  `simple_grading(a,s)` (true = noncompact) =
  `offset[s] XOR dual_m_alpha(s).dot(left_torus_part(a))`
  (tits.h:785-789), meaningful only when `s` is imaginary for the
  element's involution — upstream evaluates it blindly after
  classification (kgb.cpp:588-591). `basedTwistedConjugate(a,s)` =
  `twistedConjugate` plus `left_add(m_alpha(s))` when `offset[s]`
  (tits.h:810-816; valid modulo torus conjugation).
  `Cayley_transform(a,s)` is a bare `sigma_mult(s,a)` (tits.h:717-719).
  `is_valid(a)` — `a . twisted(a) == e` — is an ELEMENT predicate (the
  strong-involution square condition, tits.cpp:728-733), not an offset
  gate; stage (d) may port it for seed verification.
- Gradings beyond simple roots: computed by CONJUGATING the root to a
  simple one through based cross actions (tits.cpp:661-672; same
  pattern kgb.cpp:818-831), not by a sum formula. The
  sum-over-support formula (tits.cpp:674-717) is valid ONLY for
  simple-imaginary roots at the FUNDAMENTAL involution (it takes a bare
  torus part); gradings are NOT additive in general, so extending it is
  ill-posed.
- Reduction: `i_tab.reduce(a)` replaces the torus bits by the canonical
  coset representative modulo the involution's `fiber_denom` subspace —
  the saturated `(-1)`-eigenlattice of theta-transpose mod 2, per the
  CODE at tits.cpp:834-838, not the header's "image of theta-1" — with
  zero bits at the echelon pivot positions (subquotient.cpp:104-124),
  called after EVERY cross action and Cayley transform before the hash
  probe (kgb.cpp:563-564, 597-599). Bitwise equality on the reduced
  pair is the dedup relation (tits.h:377-381, 624-635).
- Handoff boundary: upstream PERSISTS per element only the reduced left
  torus part plus an involution index (kgb.h:288-308) — exactly the
  pair this port's elements are made of from the start.
- Status assignment (stage (e)'s consumption, kgb.cpp:577-611):
  complex/real from Weyl tests, imaginary compact/noncompact from
  `simple_grading` alone.

## Port decisions and divergences

1. ELEMENT = `(InvolutionId, torus bits)` FROM THE START. Upstream
   carries the Weyl word per element during construction only because
   its table lacks O(1) edges; the stage-(b) table has stored cross
   links and the Cayley edge, so the port's `TitsElement` is the
   persisted shape already. No per-element Weyl data exists at any
   phase.
2. MATRIX TRANSPORT replaces word walks. `pull_across(w', t)` equals
   the mod-2 action of `w'` on `X_*`, so the port applies the record's
   `WeylAction::coweight_matrix()` reduced mod 2 — ONE matrix-vector
   apply per cross action (O(rank^2) parity checks; at split E8,
   8 x 320,206 cross calls x 64 parity tests ~ 1.6e8 bit ops,
   sub-second). Implementation hazard, pinned: the matrix is that of
   `w'` AS A GROUP ELEMENT — not of `w'^{-1}` and not of
   `theta' = w'.delta`; the record's `twisted_involution().weyl_action()`
   stores exactly the pure Weyl factor.
3. The based cross action is one CLOSED-FORM per-element map, verified
   step-for-step against the four sigma multiplications. With
   `w = record(n).weyl_element()`, `n' = cross(s, n)`,
   `w' = record(n').weyl_element()`:
   - `t' = reflect(t, s)` — applied ONCE, to the original bits, before
     any correction;
   - if `s` is a left descent of `w` (`l(sw) < l(w)`, the stored
     element's O(1) query): `t' += m_alpha(s)`;
   - if `l(w') > l(sw)` (both cached; `l(sw) = l(w) -+ 1` from the same
     descent): `t' += mod2(w') . m_alpha(twist(s))` — the pull crosses
     the TARGET Weyl part;
   - if `offset[s]`: `t' += m_alpha(s)`;
   - reduce modulo `record(n').mod_space()`.
   The second correction also fires on COMPLEX ASCENTS
   (`l(w') = l(w)+2 > l(sw) = l(w)+1`), and on REAL generators BOTH
   length corrections fire — the closed form handles every case with no
   per-kind branching.
4. Cayley transform: `n' = table.cayley(s, n)` (`None` remains the
   not-yet-added signal), torus part `t' = reflect(t, s)` plus
   `m_alpha(s)` exactly when `s` is a left descent of `w` — at a
   noncompact-imaginary `s` the length grows (asserted upstream,
   kgb.cpp:598) so the correction never fires there, but the formula
   stays the general `sigma_mult` one — then reduce modulo the TARGET's
   mod-space (the subspace has grown).
5. INVERSE CAYLEY DEFERRED (scope note above). Recorded for the future
   stage: `sigma_inv_mult` shape; repair when `alpha` fails to grade
   noncompact, by the FIRST odd-pairing vector of the SOURCE
   involution's ordered mod-space basis (`basis_vectors()` already
   exists `pub(crate)` at mod_two.rs:312, ascending-pivot order); the
   repair choice is PROVABLY representative-independent (any two
   odd-pairing vectors differ by an element of the target space, so the
   repaired elements coincide after target reduction) — an invariant,
   not merely adapter-level; port upstream's post-repair assert
   (tits.cpp:642) as the invariant violation.
6. The general imaginary-root grading uses upstream's
   conjugate-to-simple loop over based cross actions. It returns
   `Result<Option<bool>>` with `None` for a non-imaginary root
   (mirroring `simple_root_kind`'s Option shape); the invariant family
   is reserved for internal breaks (conjugation termination).
   `simple_grading` evaluates the formula blindly with the imaginary
   precondition DOCUMENTED (upstream parity; stage (e) guards via
   `simple_root_kind` first). Both return true = noncompact, matching
   the strong-real layer's `Grading::is_noncompact` vocabulary.
7. `grading_offset` is an INPUT (stage (d) elects it from the
   square-class cocharacter; tests use hand values and the adjoint
   convention `offset[s] = (twist(s) == s)`, tits.cpp:564-588). The
   constructor validates length (`RankMismatch`) AND twist-invariance
   `offset[s] == offset[twist(s)]` (tits.h:664-667, an O(rank) check at
   the boundary); which square class the offset realizes remains the
   caller's contract.
8. NORMAL-FORM AND OBSERVABLES, corrected by review. The naive
   annihilation argument is FALSE: two bit-lifts of the same coset
   differ by `xi + 2u`, and `(1+theta^T)/2` sends that to
   `(1+theta^T)u` — an integer coweight that need not vanish — so
   `torus_factor` (kgb.cpp:699-712) is representative-independent only
   MODULO the integral lattice `(1+theta^T)X_*`: its fractional class
   (the torus element) is invariant, the exact rational vector is not.
   Direct differential comparability nevertheless holds because both
   implementations use the SAME canonical form — upstream zeroes the
   pivot bits of a lowest-pivot RREF basis (bitvector.cpp:364, 542;
   subquotient.cpp:104-124), and the crate's `ModTwoSubspace`
   deliberately implements the identical low-pivot RREF over the same
   `X_*` coordinates (mod_two.rs:173-176, 215-235). This is CONVENTION
   COINCIDENCE, not convention independence: raw torus bits and exact
   `torus_factor` vectors are comparable GIVEN that coincidence, which
   the differential harness must pin with a fixture test; if either
   convention ever changes, comparison drops to the invariant class
   mod `(1+theta^T)X_*`. The stage map's observables paragraph is
   updated accordingly.

## Data layout and public boundary

```text
// tits_element.rs
TitsElement {                     // derives Clone, Debug, Eq, Hash, Ord,
    involution: InvolutionId,     //   PartialEq, PartialOrd — field order
    torus: ModTwoVector,          //   load-bearing: derived Ord groups by
}                                 //   involution; raw-bit relations are
                                  //   meaningful on REDUCED representatives
TitsElement::new(&InvolutionTable, InvolutionId, ModTwoVector) -> Result<...>
    // gated: involution bounds (IndexOutOfRange), bit dimension vs
    // lattice rank (RankMismatch); does NOT auto-reduce
involution() / torus_bits()

TitsCoset::new(&InnerClass, grading_offset: Vec<bool>) -> Result<...>
    // owns: a clone of the inner class, m_alpha / dual_m_alpha tables,
    // the mod-2 delta-transpose, the validated offset, its OWN derived
    // twist permutation and simple-reflection root permutations (a
    // named duplication of the table's private caches — the coset does
    // not own the table, and rebuilding per call is forbidden)
grading_offset() -> &[bool]
simple_grading(&InvolutionTable, &TitsElement, generator) -> Result<bool>
cross(&InvolutionTable, generator, &TitsElement) -> Result<TitsElement>
cayley(&InvolutionTable, generator, &TitsElement) -> Result<Option<TitsElement>>
grading(&InvolutionTable, &TitsElement, root: RootId) -> Result<Option<bool>>
reduce(&InvolutionTable, &TitsElement) -> Result<TitsElement>   // idempotent
```

Argument-order rule, stated once: EDGE operations put the generator
first (`cross`, `cayley` — mirroring the table's edges); QUERIES put
the element first (`simple_grading`, `grading` — mirroring
`simple_root_kind`). Operations take `&InvolutionTable` per call — the
substrate idiom — and the PROVENANCE GATE is full `InnerClass` equality
(`table.inner_class() == &self.inner_class`), not datum equality alone:
two inner classes over one datum with different deltas would silently
corrupt every transport under a datum-only gate. The equality compare
is O(root-system bytes); if the HPC preflight shows it binding, the
recorded fallback is the `WeylElement`-style cheap dimension gate plus
caller-contract wording. `reduce` stays public — stage (d)'s seed bits
and central-fiber candidates arrive raw and must be normalized before
comparison — and auto-reduction inside `cross`/`cayley` matches
upstream's reduce-before-probe discipline. The mod-2 matrix apply is a
plain in-module function (output bit `i` = parity of `matrix[i][j]`
over the set bits `j` of the input) — `ModTwoLinearMap` is test-only
by design (mod_two.rs:113-118) and stays so; no `mod_two.rs` changes
are needed (`basis_vectors` already exists for the deferred inverse
Cayley).

Errors: `RankMismatch` for offset and bit-dimension shapes,
`DatumMismatch` (via inner-class inequality) for provenance,
`IndexOutOfRange` for generator and involution bounds,
`TitsCosetInvariantViolation { invariant }` for internal breaks only
(conjugation termination). No resource variant: every operation is
O(roots) or better with no enumeration.

## Resource and arithmetic policy

Per-operation work is one O(rank^2) matrix-vector parity apply plus
O(rank) xors and O(1) table lookups (the imaginary-grading loop is
O(positive-roots x rank^2) worst case). `ModTwoVector` supplies checked
dimensions; allocations via `try_capacity`.

## Tests and fixture gate

All tests live in the in-crate `#[cfg(test)]` module (the bit-for-bit
recomputation uses `pub(crate)` surfaces).

- SL(2,R) (simply connected A1, offset [true]): the fundamental fiber's
  two elements `(e,0)`/`(e,1)` are SWAPPED by the cross action (type I
  — the `m_alpha` offset correction with `dual_m_alpha = alpha mod 2 =
  0`), the split element is fixed (its bits reduce to zero); both
  fundamental elements grade noncompact and Cayley to the split
  involution's reduced element; the three reduced states are pairwise
  distinct. The cross-FIXES expectation belongs to PGL(2,R) (type II,
  `m_alpha = 0` in the adjoint lattice) — tested there.
- Compact SU(2) (A1, offset [false]): `simple_grading` false, no
  noncompact roots, no Cayley taken.
- Sp(4,R) (B2 split): for EVERY (element state, generator), the
  closed-form `cross` agrees bit-for-bit BEFORE reduction with a
  word-level recomputation of upstream's exact four-step sigma recipe
  (reduced words + per-letter `reflect` via `ModTwoVector` ops), and
  after reduction lands on the table's normal form. This is the
  center-of-gravity test: it exercises complex ascents/descents and
  real generators (both corrections firing) across all 6 involutions.
- Imaginary grading by conjugation: on the SU(2,1) inner class (A2 with
  the diagram flip) the simple-imaginary root `alpha_1 + alpha_2` is
  NON-simple, so the conjugate-to-simple loop performs a genuine step;
  its result is checked against the word-level definitional
  recomputation (the sum formula is fundamental-involution-only and is
  checked there for exactly the simple roots, where it degenerates to
  the `simple_grading` formula).
- PGL(2,R) (adjoint A1): type-II cross action fixes the fundamental
  fiber elements; the same fixture anchors the deferred inverse-Cayley
  repair's future test.
- Offset and provenance: wrong-length offsets rejected
  (`RankMismatch`); twist-variant offsets rejected on an unequal-rank
  class; out-of-range generator/involution arguments rejected
  (`IndexOutOfRange`); a coset built for a SAME-DATUM inner class with
  a DIFFERENT delta is rejected by the inner-class equality gate.

`tests/fixtures/domain/tits_operations.atlas` is reserved; language
observables arrive at stage (e), per the stage map.

## Consequential updates

Landing this stage must update: `lib.rs` (module and exports);
`error.rs` (`TitsCosetInvariantViolation`); `KGB_STAGE_MAP.md` (stage
(c) landed; the observables paragraph gains the convention-coincidence
correction for torus_factor and raw bits); `REAL_GROUP_DESIGN.md`'s
progression (next: stage (d), the seed). No `mod_two.rs` or
`involution_table.rs` changes are required.

## Three independent design checks (returned; corrections folded)

1. Atlas semantics — VERIFIED the closed-form cross action
   step-for-step (reflect placement, both correction gates, the
   target-matrix pull, the pull_across direction), the Cayley formula,
   and the deferred inverse-Cayley ruling (upgraded to a provable
   invariant). CORRECTED: the torus_factor annihilation argument
   (refuted by counterexample; replaced with the
   convention-coincidence statement); the SL(2,R) cross-fixes test
   oracle (type I SWAPS — the design had encoded wrong semantics of
   its central operation); the SL(2,R) repair expectation (repair can
   never fire there; moved to type II / PGL(2,R)); the near-vacuous B2
   grading comparison (sum formula is fundamental-only,
   simple-imaginary-only; replaced with SU(2,1) and word-level
   validation). GAPS closed: offset twist-invariance validation;
   simple_grading's documented imaginary precondition; the is_valid
   miscitation (element predicate, noted for stage (d)).
2. Rust internals — CORRECTED: `ModTwoLinearMap` is test-only (plain
   in-module apply function instead); the per-call cost wording (one
   matrix-vector apply, not a full-matrix reduction); the offset gate
   (`RankMismatch`, not the invariant family); `DatumMismatch` added
   to the error list. GAP closed: the coset owns its own twist (the
   table's is private). VERIFIED: all dimensions line up at lattice
   rank; every assumed accessor exists at the claimed cost; no
   borrow-shape problem; derives resolved to the full set with
   involution-first field order; the test plan is executable in-crate.
3. API and consumer fit — BLOCKING closed: the gated
   `TitsElement::new` constructor (elements were otherwise
   uncreatable). CORRECTED: provenance by full inner-class equality;
   `TitsElt` renamed `TitsElement` (no C++ abbreviations in the
   crate); `grading`'s `Option` shape; inverse Cayley deferred with
   its consumer named; the argument-order rule stated. One reviewer
   claim REFUTED during folding: `ModTwoSubspace::basis_vectors`
   DOES exist (`pub(crate)`, mod_two.rs:312) — no mod_two change
   needed for the deferred stage.

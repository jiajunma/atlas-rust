# KGB seed design (stage d)

## Approved scope

Stage (d) of the KGB map: the seed `x0` — the square-class cocharacter
(`some_coch`), the elected base grading (the `TitsCoset` offset), the
binary grading-shift solve, and the central-fiber minimization —
handing stage (e) a per-form seed bundle (form id, offset, exact
cocharacter, reduced seed element). Substrates stay outside the bundle
per the crate idiom. DEFERRED with named consumers: `backtrack_seed`
(partial KGB only; sole caller kgb.cpp:543) and `minimal_torus_part`
(sole caller the synthetic-real-form interpreter path,
atlas-types.w:3866).

## Atlas construction (oracle trace, review-corrected)

Citations are to `~/mycodes/atlasofliegroups`, master `4d3e9449`.

- `some_coch(G, csc)` (innerclass.cpp:966-977): elect the square
  class's representative real form — the LOWEST `RealFormNbr` in the
  class (innerclass.cpp:886-888; partition.cpp:65-83, first value
  sticks) — take its compact-simple set, sum the RATIONAL fundamental
  coweights over it, and return `stable_log(exp_2pi(sum), xi^T)`
  (y_values.cpp:155-166). `stable_log` semantics, verbatim from
  review: `adapted_basis(xi+1)` returns a basis `B` of `X_*` whose
  first `d` columns Z-span EXACTLY the saturated fixed lattice
  `(X_*)^{xi^T}` (matreduc.cpp:244-262); the map takes the first-`d`
  `B`-coordinates of `log_2pi(t)` (itself per-coordinate nonneg mod-1
  reduced), reduces THOSE mod 1 — i.e. reduces the fixed part modulo
  the full fixed lattice, not modulo `X_*` — and drops the trailing
  coordinates. The output is EXACTLY `xi^T`-fixed (it lies in the
  +1-eigenspace), not merely mod 1: a port representative may only
  ever be shifted by `(X_*)^{xi^T}` elements, never per-coordinate
  mod-1 normalized, or twist-invariance and the parity observables
  break. `exp_2pi(some_coch) == exp_2pi(sum)` exactly, while
  `exp_pi(some_coch)` differs from `exp_pi(sum)` by an H(2) element —
  which is exactly why `x0_torus_part` re-measures its base from
  `exp_pi(some_coch)` itself before XORing (innerclass.cpp:1072-1078),
  and the final assert (1090-1092) proves the compensation complete.
  PRECONDITION for any general `stable_log` port: the trailing
  adapted-basis coordinates of the input's log must be integral
  (guaranteed structurally for `some_coch`; NOT guaranteed for
  `real_form_of`'s general squares) — documented and asserted.
- `g_rho_check()` IS the stored `some_coch` value: the store is
  realredgp.cpp:35-47 (the accessor realredgp.h:61, 91-92);
  `base_grading() = grading_of_simples(G, g_rho_check())`
  (realredgp.cpp:147-150) seeds the `TitsCoset` (kgb.cpp:525-526).
  The innerclass.cpp:959-964 comment is stale: it contradicts
  realredgp.cpp:40 and cites a nonexistent function; the square-root
  election note lives at innerclass.cpp:975.
- Parities (innerclass.cpp:1295-1303 vs 949-957): `grading_of_simples`
  bit = EVEN pairing (noncompact for imaginary simples, ALL simples
  marked); `compacts_for` bit = ODD pairing. Complementarity on
  imaginary simples is exact. INTEGRALITY PRECONDITION, recorded:
  `<some_coch, alpha_s>` must be an exact integer for every simple `s`
  (upstream asserts inside `RatWeight::dot`, ratvec.h:158-165; it
  holds because the output is congruent mod `X_*` to a Z-sum of
  fundamental coweights) — the port gates this as a named invariant.
  Upstream's `TitsCoset` never asserts offset twist-invariance; the
  port's stage-(c) constructor already supplies that check.
- `x0_torus_part(G, rf)` (innerclass.cpp:1070-1095): base = compacts
  of `exp_pi(some_coch)`; `rf_cpt` = the weak-orbit representative's
  bits on the imaginary-simple positions (891-909);
  `grading_shift_repr(base XOR rf_cpt)` (986-1007) slices to the
  DELTA-FIXED SIMPLE positions (`simple_roots_imaginary()` — NOT the
  imaginary-subsystem basis; for twisted A2 the delta-fixed simple set
  is EMPTY while the subsystem basis is not), builds the mod-2 matrix
  (fundamental fiber-group basis paired with those simple roots),
  solves by `section()` with the exactness assert, lifts to rank
  bits; then the CENTRAL-FIBER MINIMIZATION (1041-1089, 1020-1036):
  solve `toAdjoint(y) = wrf_rep(rf) - class_base(csc)`, filter the
  fiber-partition class of `y` to members with the SAME adjoint image,
  and minimize `bits + c` — the SHIFTED value, upstream's comment is
  explicit — under the numeric bit order (single-word unsigned, bit 0
  LSB; upstream multi-word order is high-word-dominant but moot at
  RANK_MAX 32). Any solution `y` in the same orbit yields the same
  candidate set (the differences form the grading-stabilizer
  subgroup), so the section election matters only up to orbit — pinned
  by fixture anyway.
- KGB consumption (kgb.cpp:489-560): ONE seed element at the identity
  Weyl part, reduced and hashed; the fundamental fiber is discovered
  by the closure loop. `torus_factor(x) = symmetrise(g_rho_check() -
  lift(bits), theta)` (kgb.cpp:699-712) — EXACT rational data. Exact
  consumers (recorded): the `torus_factor` and `base_grading_vector`
  builtins, the synthetic-real-form path, `KGBElt_from_data`'s
  integrality test, `KGB::twisted`, extended blocks, and
  `Rep_context::g_rho_check`. Parity-only consumers: the coset offset
  and the compacts/base gradings.

## The three numberings (review-adjudicated)

1. PORT ORDER vs INTERNAL `RealFormNbr`: the ordering audit's verdict
   is COINCIDE — no reindexing anywhere. Proof chain (recorded; full
   detail in the review archive): both number orbits by ascending
   fresh-seed walks (partition_def.h:69-96 vs `walk_mask_orbits`),
   both elect class representatives as the minimal member, both use
   the same element-to-integer bijection (same ambient simple-coweight
   coordinates, same low-pivot RREF subquotient bases, same
   complement election, same coordinate extraction), the same
   all-ones base grading anchors quasisplit = 0, and square classes
   are the same coset-coordinate integers with the same lowest-form
   class bases. The port's B2 data already realizes upstream's
   internal C2 order: form 0 = sp(4,R) (KGB 11), 1 = sp(1,1) (4),
   2 = sp(2) (1). CONSEQUENTIAL CRATE FIX: the doc comments in
   `weak_real_form.rs` and `strong_real.rs` claiming these numberings
   sit under the adapter deferral are WRONG and must be corrected
   with this stage; the `CartanId` disclaimer stays (genuinely
   non-Atlas). One caveat kept: square-class LABELS inherit the
   subquotient basis election; the audit proved the port's election
   coincides too, but any future basis change moves csc labels.
2. INTERNAL vs the interpreter's EXTERNAL `FormNumberMap` order
   (output.cpp:73-129): `.atlas` fixtures speak the external order —
   ascending depth with a grading tiebreak, so the COMPACT form is
   external 0 and the split form is LAST (for Sp(4,R): external 0 =
   sp(2), 1 = sp(1,1), 2 = sp(4,R) — the REVERSE of internal). Stage
   (d) works entirely in internal numbering; the LANGUAGE ADAPTER owns
   the internal-to-external permutation, recorded now as an adapter
   obligation with the B2 pin test labeling its ids INTERNAL and
   noting the external permutation alongside.

## Port decisions

1. New integral machinery: `integer_lattice.rs` gains an
   `adapted_basis` routine tracking the left transform and its inverse
   during reduction (the current `ReductionState` keeps only the right
   factor), as a FAITHFUL port of matreduc's pivot strategy
   (matreduc.cpp:228-336) — the basis election is observable-bearing
   (it fixes `g_rho_check`, hence every `torus_factor` rational), so
   a different pivot strategy would collapse the stage-(c)
   direct-comparability ruling via basis election. Threads the
   existing `IntegerLatticeBudget`, with rational intermediates
   bounded under the same coefficient-bits knob. The mod-1 reductions
   (coordinatewise nonneg before, first-`d` after) are replicated
   exactly.
2. Fundamental coweights: `varpi_i^vee = sum_j (C^{-1})_{ji}
   alpha_j^vee` in full lattice-rank coordinates (zero radical
   component; rootdata.cpp:850-853, 1015-1016) — solve `C c = e_i`
   rationally and combine simple coroots. A small shared rational-
   solve helper is factored rather than duplicating
   `real_form_labels.rs`'s specialized elimination a third time.
3. The grading-shift solve pairs the fundamental fiber group's basis
   (`ambient_fiber().basis_representatives()` mod 2) against the
   DELTA-FIXED SIMPLE roots, and reuses the augmented-elimination
   idiom through a NEW SHARED crate-private helper (this is the
   fourth copy — grading.rs, real_form_labels.rs, strong_real.rs own
   the other three; refactoring those onto the helper is recorded
   follow-up). The review proved the idiom elects the same particular
   solution as upstream `section()` (both greedy lowest-pivot in
   index order, solution supported on pivot columns) — still pinned
   by fixture.
4. Central fiber by IN-STAGE RE-WALK: the strong layer's per-class
   mask tables were deliberately transient ("a public partition
   arrives when KGB demonstrably needs one" — this stage is that
   need, and the smaller change is to NOT retain them): `build`
   re-runs the shared `walk_mask_orbits` for the single elected
   square class at the fundamental Cartan, bounded by a
   `max_fiber_elements` scalar in the existing style, re-solves `y`
   with the shared elimination helper, and takes the same-adjoint-
   image filter and shifted-value minimum. The strong-layer gate is
   therefore an EXPLICITLY RECORDED DOWNGRADE: count-consistency
   (form and square-class counts) plus caller contract — the strong
   classification contributes only the form-to-square-class map and
   `kgb_size`.
5. The seed's element: the fundamental involution is located by
   `table.lookup(&WeylElement::identity(...))` — NEVER assumed to be
   `InvolutionId(0)` — erroring if the fundamental Cartan is not yet
   added. The bundle's doc mirrors `TitsElement`'s "in one table's
   numbering" caveat; the contract is ONE SHARED table per inner
   class (the table's own doc already says so) under append-only
   growth. Seed verification: the ported x0-compacts assert
   (`invariant: "x0 compacts"`) plus an INTERNAL check that the seed
   element squares into the identity coset, restated in available
   terms at the identity Weyl part (a torus-bits statement against
   the fundamental record's mod space and delta transport — no new
   public `TitsCoset` method; nothing in stage (e) calls one).

## Data layout and public boundary

```text
// real_form_seed.rs  (fixture stays domain/seed_x0)
RealFormSeed {                    // derives Clone, Debug, Eq, PartialEq;
    form: WeakRealFormId,         //   private fields, accessors only —
    grading_offset: Vec<bool>,    //   public field construction would let
    cocharacter: RationalCoweight,//   callers assemble mismatched triples
    element: TitsElement,         //   (reduced, fundamental involution)
}
form() / grading_offset() / square_class_cocharacter() / element()
RealFormSeed::build(
    &InnerClass, &CartanClassification, &StrongRealClassification,
    &InvolutionTable, WeakRealFormId,
    &IntegerLatticeBudget, max_fiber_elements: usize,
) -> Result<RealFormSeed, StructureError>
// lattice.rs gains RationalCoweight: an opaque newtype (private
// malachite storage; pub(crate) exact access for torus_factor;
// public checked numerator/denominator views) — exposing raw
// malachite types is REFUSED (no third-party types in the API).
```

Construction invariant, recorded: `grading_offset ==
grading_of_simples(cocharacter)` (a named duplication, the stage-(c)
precedent). Forms sharing a square class share (offset, cocharacter);
stage (e) may share one coset per square class. Provenance gates:
(i) `table.inner_class() == inner_class` (the TitsCoset idiom);
(ii) InnerClass-to-classification through the fundamental class —
identity Weyl action, datum equality, stored involution equals delta
(the classification normalizes exactly so);
(iii) strong-to-classification: the recorded count-consistency
downgrade of decision 4; (iv) the form id ranged via `kgb_size`.
Errors: `SeedInvariantViolation { invariant }` ("x0 compacts",
"grading-shift exactness", "seed square", "integral simple pairing",
"stable-log integrality"), `SeedResourceLimit { resource, limit }`
for the re-walk, plus pass-through integral/fiber variants. Budgets
follow precedent exactly: no knob for the rank-bounded rational
solve, the existing `IntegerLatticeBudget` for the adapted basis, one
`max_fiber_elements` scalar for the walk — no new budget kinds.

## Tests and fixture gate

- SL(2,R) and PGL(2,R): quasisplit seeds; base grading all-noncompact
  at the quasisplit form; the x0-compacts invariant holds; zero-bit
  seed elements where the theory says so.
- Sp(4,R): seeds for all three weak real forms in INTERNAL order
  (ids disambiguated by `kgb_size` 11/4/1 — the only surface; the
  external FormNumberMap permutation 2/1/0 recorded in a comment);
  gradings at each seed match the `rf_cpt` complement; two builds
  agree (`PartialEq` is load-bearing).
- SU(2,1) (twisted A2): the delta-fixed simple set is EMPTY — the
  grading-shift system is 0-dimensional and the seed still validates;
  the compact form of the equal-rank classes grades all-compact.
- Central-fiber minimization: the B2 so(5) square class with three
  fiber orbits (already exercised in strong_real.rs tests) elects the
  minimal shifted value deterministically.
- Numbering: the coincidence pin — the port's internal form order
  against upstream's published internal C2 data — plus the crate
  doc-comment corrections landing with this stage.

`tests/fixtures/domain/seed_x0.atlas` is reserved.

## Consequential updates

`lib.rs` (module + exports); `error.rs` (the two Seed families);
`lattice.rs` (`RationalCoweight`); `integer_lattice.rs`
(`adapted_basis` with transform tracking); the shared augmented-
elimination helper (new home; existing three copies' refactor is
recorded follow-up); DOC CORRECTIONS in `weak_real_form.rs` and
`strong_real.rs` (the adapter-deferral claims on WeakRealFormId /
SquareClassId are wrong — numbering coincides with upstream);
`KGB_STAGE_MAP.md` (stage (d) landed; the external-order adapter
obligation); `REAL_GROUP_DESIGN.md` progression (next: stage (e)).

## Three independent design checks (returned; corrections folded)

1. Atlas semantics — VERIFIED the pipeline end to end (some_coch
   election, grading-shift matrix, central-fiber filter and the
   shifted-value minimization, deferral claims). CORRECTED: exact
   xi^T-fixedness replaces the mod-1 stability gloss; the stale-
   comment citation; the stable_log general-input hazard. GAPS
   closed: the THIRD numbering (FormNumberMap external order — an
   adapter obligation, with the B2 test relabeled internal); the
   parity integrality precondition as a named invariant; the exact-
   value consumer list for torus_factor.
2. Rust internals — THE ORDERING AUDIT: COINCIDE (outcome A), proof
   chain recorded above; the crate's own adapter-deferral doc claims
   corrected as consequential updates. CORRECTED: integer_lattice
   cannot produce the adapted basis today (new transform-tracking
   routine, faithful matreduc pivot strategy — basis election is
   observable-bearing); fundamental-coweight semantics (inverse-
   Cartan-weighted coroot combinations, zero radical part); the
   grading-shift root list (delta-fixed simples, NOT the imaginary-
   subsystem basis — twisted A2 would break); the Ord bound stated
   as lattice rank <= 64. GAPS closed: the strong layer's missing
   central-fiber ingredients (resolved by the re-walk decision); the
   nonexistent is_valid carrier (restated internally); the
   integrality gate.
3. API and consumer fit — BLOCKING closed: `&CartanClassification`
   added to the signature (the fiber data lives only there; the
   strong-real design had explicitly assigned this stage the pairing-
   gate decision). CORRECTED: associated constructor (free functions
   are not the crate idiom), private fields with the mismatched-
   triple argument, the opaque `RationalCoweight` boundary, budget
   adjudication per precedent, `real_form_seed.rs` naming, the
   form id added to the bundle, the shared-table contract with
   lookup-not-assume for the fundamental involution.

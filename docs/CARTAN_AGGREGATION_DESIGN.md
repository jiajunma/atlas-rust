# Cartan aggregation design

## Approved scope

This stage bundles the per-Cartan machinery into one owning value per
twisted-conjugacy class and assembles the inner-class-level facts that are
derivable from it: per-real-form Cartan sets, the most-split Cartan of each
real form, the total involution count, and the Cartan partial order from
single-root Cayley links. Out of scope: the strong-real layer (square
classes, per-square-class fiber partitions, strong representatives) and the
`KGB_size` numbers it unlocks — the next stage; the dual fiber and
everything dual-sided (dual labels, block sizes); `simpleComplex` and the
Lie-type orbit-size formula (the crate counts orbits by enumeration; a
future real-Weyl-group stage is the other `simpleComplex` consumer); and
Atlas's Cartan numbering, which is generation-order-dependent and stays
under the standing adapter deferral.

## Atlas construction

Citations are to `~/mycodes/atlasofliegroups`, master `4d3e9449`.

Atlas's `CartanClass` owns a fiber, a dual fiber, the `simpleComplex` list,
and the orbit size (cartanclass.h:478-523); everything except the dual
fiber and `simpleComplex` already has a validated Rust equivalent. The
orbit size — upstream a Lie-type formula — equals the twisted-conjugacy
cardinality the crate counts, because `w -> w delta` intertwines twisted
conjugacy with ordinary `W`-conjugacy of the involution and the formula's
denominator is that involution's centralizer (cartanclass.cpp:1046-1064,
cartanclass.h:487-522). The `InnerClass` Cartan table (`C_info`,
innerclass.h:209-223) adds: `real_forms` — exactly the image of the label
map (`Cartan_set`, innerclass.cpp:674-683); sample torus parts — transport
artifacts the grading-based correlation avoids; and `below` — one bit per
single-root Cayley link.

The Cayley link is class-level: the insertion (innerclass.cpp:265) sits
inside the loop over ALL positive imaginary roots, outside the grading
conditional, so a link is recorded regardless of compactness; only
real-form transport and `mostSplit` consult the grading. The successor of a
class with representative `w` at imaginary root `alpha` is the class of
`s_alpha after w` — left multiplication, representative-independent
(innerclass.cpp:252-253 after conjugating `alpha` simple; for imaginary
`alpha` the reflection commutes with `w delta`, so the product is again a
twisted involution). Upstream closes the relation incrementally
(`new_max`, poset.h:127-140), which is sound only because its generation
order is a graded BFS — every Cayley step drops `dim H^theta` by one, so
parents always precede targets. The crate's class order carries no such
guarantee, so the port records all link bits into a matrix and closes with
an order-independent Warshall pass, asserting irreflexivity.

The order, stated unambiguously: `is_below(a, b)` is true iff `a != b` and
`(H^{theta_b})_0` is `W`-conjugate into `H^{theta_a}` — `a` is the
more-compact end of a nonempty Cayley chain into `b`, matching Atlas
`Cartan_ordering().lesseq(a, b)` with equality excluded
(innerclass.h:156-164, 316-321). The fundamental class is below every
other; most-split classes are maximal.

`mostSplit(rf)` upstream is a generation-order artifact ("the last
assignment sticks", innerclass.cpp:287) — but the CLASS it denotes has an
order-free characterization the port uses instead: a Cartan is most split
for a weak real form iff the grading of that form's local representative is
trivial — every imaginary root compact (`CartanClass::isMostSplit`,
cartanclass.cpp:133-144). Existence and uniqueness within the form's
Cartan set are theorems (the maximally split Cartan of a real form is
unique up to conjugacy); both are hard construction-time checks here, not
debug asserts. `numInvolutions` is the sum of orbit sizes
(innerclass.cpp:696-717); the per-form variant falls out of `cartan_set`
plus orbit sizes and needs no separate accessor.

Order-independent aggregates this stage exposes: Cartan sets as sets,
most-split classes, counts, orbit sizes, the partial order as a relation.
Order-dependent Atlas observables (Cartan numbers, `below` bitmap layout,
the most-split INDEX) remain adapter territory.

## Data layout and public boundary

```text
// cartan_class.rs, beside TwistedConjugacyClass
CartanClass {
    class_info: TwistedConjugacyClass,
    decomposition: CayleyCrossDecomposition,
    grading: CartanGradingData,
    partition: WeakRealFormPartition,
    labels: RealFormLabels,
}
  representative() / twisted_involution_count() / decomposition()
  / grading() / partition() / labels()

TwistedConjugacyPartition   // also cartan_class.rs
  classes() -> &[TwistedConjugacyClass]
  class_of(&TwistedInvolution) -> Result<usize, StructureError>

// inner_class.rs
InnerClass::twisted_conjugacy_partition(weyl_budget)
    -> Result<TwistedConjugacyPartition, StructureError>

// cartan_classification.rs
CartanClassificationBudget::new(
    integer_lattice: IntegerLatticeBudget,     // owned; const fn
    adjoint_fiber: AdjointFiberBudget,
    weyl_budget: usize,
    max_fiber_elements: usize,
    max_peeling_steps: usize)

CartanClassification::build(&InnerClass, &CartanClassificationBudget)
    -> Result<CartanClassification, StructureError>

cartan_classes() -> &[CartanClass]
cartan_class(CartanId) -> Option<&CartanClass>
weak_real_form_count() -> usize
cartan_set(WeakRealFormId) -> Option<&[CartanId]>       // precomputed
most_split(WeakRealFormId) -> Option<CartanId>          // precomputed
twisted_involution_count() -> usize
is_below(CartanId, CartanId) -> Option<bool>            // strict, closed
```

The per-Cartan owner takes the Atlas name `CartanClass` — the crate
consistently adopts Atlas names for documented partial ports, and
`cartan_class.rs` was reserved for exactly this; it holds a
`TwistedConjugacyClass` rather than extending it, since class enumeration
must stay fiber-free. Accessors are `cartan_classes`/`cartan_class`, not
`classes`/`class`, to avoid colliding with the id-iterator shape of
`WeakRealFormPartition::classes`. `CartanId(pub(crate) usize)` mirrors
`WeakRealFormId` exactly, derives included.

Crate Cartan order: THE FUNDAMENTAL CLASS FIRST, then the remaining
classes in `twisted_conjugacy_partition` order. The raw enumeration order
is matrix-lexicographic and puts the identity class LAST, so `build`
locates the class whose representative's Weyl action is the identity and
moves it to index 0 (absence is
`CartanClassificationInvariantViolation { invariant: "fundamental class" }`);
site 0's labels are then the identity map by the label stage's fundamental
theorem, and `weak_real_form_count` and the quasisplit anchor delegate to
its partition.

`TwistedConjugacyPartition` exists so class membership is a lookup, not a
re-enumeration: the orbit sweep already keys candidates by their root-image
permutations, and the partition retains that map plus provenance anchors
(datum and distinguished involution data). `class_of` gates datum
(`DatumMismatch`) and delta-factorization
(`DistinguishedInvolutionMismatch`) before the map hit — the permutation
key does not encode the datum, so the gates are load-bearing — and a miss
after the gates is the `"enumerated class lookup"` invariant violation,
never an `Option`: every twisted involution of the inner class is in the
full-W enumeration by construction.
`InnerClass::twisted_conjugacy_classes` becomes a thin wrapper so one
orbit implementation remains.

Construction, per class: `CartanFiber::build` at the representative's
involution (borrowed, no clone), `AdjointCartanFiber::build`,
`CartanGradingData::build`, `WeakRealFormPartition::build` with the
CALLER'S `max_fiber_elements` (never a self-scaling per-site value),
`CayleyCrossDecomposition::build`, then `RealFormLabels::build` against
class 0's grading and partition. Everything is built at the SAME
representative, satisfying each layer's provenance gates by construction.
The budget bundle owns its sub-budgets — both are small `Clone + Eq`
values with `const fn new`, and `AdjointFiberBudget` already nests an
`IntegerLatticeBudget`; the standalone one feeds `CartanFiber::build`,
matching existing usage. Derives on `CartanClass` and
`CartanClassification` are `Clone, Debug` only (the fiber-bearing
components have no `Eq`).

The poset: for each class `i` and each positive imaginary root at its
representative (imaginary kind, positive simple coordinates), form
`s_alpha after w_i` via `root_reflection` and compose, wrap as a
`TwistedInvolution`, and look up its class. For genuinely imaginary roots
the composite is involutive, so a construction failure here is the
`"Cayley successor"` invariant violation — propagated, never filtered the
way the global enumeration filters non-involutions; a successor equal to
its source likewise. Links go directly into a `Vec<Vec<bool>>` matrix
(`below[target][source]`), closed by Warshall, then checked irreflexive
(`"strict Cartan order"`). Mask or `ModTwoVector` rows are rejected: masks
cap the class count, and the mod-two layer has XOR, not OR.

`cartan_set`, `most_split`, and the involution total are precomputed at
build (per-form scans of the label images; the trivial-grading test via
the labeled local representative; checked sum), so the uniqueness and
existence checks (`"most-split uniqueness"`) fire at construction and the
accessors are pure `Option` lookups afterward, per the crate's
out-of-range convention.

The strong-real stage's reach path needs no new accessors:
`grading().adjoint_fiber()` exposes `ambient_fiber()` and `fiber_map()`,
and the ambient `CartanFiber` exposes dimension, coordinates, basis
representatives, and element construction — enough to enumerate the fiber
and pull square classes through the map.

## Resource and arithmetic policy

Per class the work is the already-budgeted fiber chain plus correlation;
the poset adds one successor lookup per positive imaginary root with a map
hit, and a closure cubic in the class count — tens at the ranks this crate
verifies locally (the measured full-suite baseline is under half a second,
and the label sweep already builds more chains than this stage will). The
budget bundle threads each scalar to exactly one existing knob and adds no
new limit kinds. Allocations use `try_reserve_exact`; the lookup map's
node allocation is infallible `BTreeMap`, a pre-existing pattern. One new
error variant:
`CartanClassificationInvariantViolation { invariant: &'static str }` with
invariants `"fundamental class"`, `"enumerated class lookup"`,
`"Cayley successor"`, `"strict Cartan order"`, `"most-split uniqueness"`.

## Tests and fixture gate

- A1 identity: two classes, orbit sizes (1, 1); the split form has both
  Cartans, the compact form only the fundamental; most-split of the
  quasisplit form is class 1; involution count 2.
- A2 identity: two classes, sizes (1, 3) in the fundamental-first order;
  `su(2,1)` has both Cartans, `su(3)` only class 0; involution count 4;
  poset exactly `{0 < 1}`.
- B2 identity: four classes, sizes (1, 2, 2, 1) summing to 6; three weak
  real forms with consistent Cartan sets; the split form's most-split
  class has empty imaginary set; the poset is exactly the five pairs
  `{0<1, 0<2, 0<3, 1<3, 2<3}` with 1 and 2 incomparable.
- A1 x A1 identity: four singleton classes (the abelian Weyl group makes
  every involution its own class); the swap-twist inner class: one class
  of orbit size 2, empty poset.
- The fundamental-class anchor: class 0's representative is the identity
  and its labels are the identity map, in every case above — including
  that `build` performed the move-to-front from the raw matrix order.
- `TwistedConjugacyPartition::class_of` round-trips every enumerated
  twisted involution, rejects foreign datums and wrong-delta involutions
  with their named errors, and agrees with the class list.
- Budget and provenance rejections thread through the existing named
  errors of each layer unchanged.

`tests/fixtures/domain/cartan_aggregation.atlas` is reserved; Cartan
counts, per-form Cartan-set sizes, involution counts, and most-split
gradings are the first differential targets once the language adapter
exists — all order-independent.

## Consequential updates

Landing this stage must update: `lib.rs` (module `cartan_classification`
plus exports `CartanClass` and `TwistedConjugacyPartition` from
`cartan_class`, alphabetical); the `TwistedConjugacyClass` doc — its
"later additions to this type" sentence is superseded: the data live in
the sibling owner (`"Fiber groups, real-form attribution, and real Cartan
component data live in [crate::CartanClass], which owns a value of this
type"`), with the supersession noted in `WEAK_REAL_FORM_DESIGN.md`;
`inner_class.rs` (the partition accessor and doc);
`REAL_FORM_LABELS_DESIGN.md` (one-sentence confirmation that the predicted
labels-beside-decomposition layout landed); and `REAL_GROUP_DESIGN.md`'s
progression paragraph (aggregation implemented; next the strong-real
layer and `KGB_size`, then KGB). The full-W enumeration scalability limit
stays tracked as separate work.

## Three independent design checks

1. The Atlas semantics review pinned the order direction (`is_below(a, b)`
   iff `a`'s fixed torus contains a conjugate of `b`'s — `a` more
   compact), proved the link is class-level (recorded outside the grading
   conditional), verified the left-multiplication successor and the
   orbit-size/twisted-class bijection, re-derived all five fixture
   anchors — including A1 x A1's four singleton classes and B2's
   (1, 2, 2, 1) with the exact five-pair poset — and found upstream's
   incremental closure sound only under its graded BFS order, mandating
   the order-independent closure here.
2. The Rust internals review found, by an empirical probe against the
   built crate, that the raw class order puts the identity class LAST in
   every fixture — the fundamental-first reorder in `build` is therefore
   load-bearing, not defensive. It designed the partition-returning
   lookup with its provenance gates, specified the Warshall closure over
   `Vec<Vec<bool>>`, pinned the assembly sequence and the owned budget
   bundle, flagged the self-scaling fiber-elements trap in the existing
   test helper, and measured the added test cost as negligible.
3. The consumer review named the types (`CartanClass` in the reserved
   module; `CartanClassification`), fixed the accessor names and the
   precomputed-`Option` query convention, confirmed the strong-real reach
   path needs no new accessors, aligned the count accessors
   (`twisted_involution_count`, `weak_real_form_count`), and completed
   the consequential-update list including the two prior-design notes.

# Weak real form design

## Approved scope

This stage ports the weak-real-form partition: the orbits of the imaginary
Weyl group `W_im` on the adjoint fiber group, acting through the grading
layer. It exposes the class count, the class of an element, and one
deterministic representative element per class. It does not port central
square classes, strong real forms, real-form labels, `realFormPartition`,
Cartan classes, or KGB data, and it does not name real forms.

## Atlas construction

Citations are to `~/mycodes/atlasofliegroups`, master `4d3e9449`.

`Fiber::makeWeakReal` (cartanclass.cpp:514-519) builds a `FiberAction` from
the base grading, the grading shifts, and `adjointMAlphas`, then calls
`partition::orbits` over all `2^adjointFiberRank` elements
(`adjointFiberSize`, subquotient.h:214) with `imaginaryRank()` generators.

`FiberAction` (cartanclass.cpp:69-97) encodes an adjoint fiber element as the
integer whose binary digits are its coordinates on the canonical subquotient
basis. It transposes the per-generator grading shifts into per-imaginary-root
columns `d_alpha[s]` (bit `j` of `d_alpha[s]` is bit `s` of shift `j`,
cartanclass.cpp:84-86), evaluates
`grading(x, s) = base[s] XOR parity(x AND d_alpha[s])`
(cartanclass.cpp:90-91), and lets generator `s` translate `x` by the
coordinates of adjoint `m_alpha_s` exactly when that grading is noncompact
(cartanclass.cpp:92-96). Each generator is an involution because
`parity(adjoint_m_alpha_s AND d_alpha[s]) = <alpha_s, alpha_s_vee> mod 2 = 0`
for genuine grading tables — a property of the validated inputs, not of
arbitrary action triples; upstream assumes only that each generator is a
permutation (partition_def.h:47-49), and for this map shape permutation,
injectivity, and involutivity coincide with that parity being zero.
Construction debug-asserts the parity, so orbit closure needs no inverses.

`partition::orbits` (partition_def.h:69-102) scans elements in increasing
integer order; each still-unclassified element seeds a new class and its
orbit is closed depth-first. Class numbers are therefore assigned by
ascending minimal element, the stored class representative is that minimal
element (partition.cpp:96-101), and class 0 is the orbit of element 0. Atlas
relies on that last fact for the quasisplit normalization:
`quasisplit() == RealFormNbr(0)` (innerclass.h:343, asserted live at
innerclass.cpp:563). Weak-real-form numbers are language-observable, so this
numbering is compatibility-relevant: it is deterministic given the
subquotient basis and the simple-imaginary order, both of which differ from
Atlas in this crate's deterministic conventions and are covered by the
standing adapter deferral recorded in `ROOT_COROOT_DESIGN.md` and
`GRADING_DESIGN.md`.

The fiber-layer consumers of `weakReal()` are exactly the class count
(`numRealForms`, cartanclass.h:336), class membership (`adjoint_orbit`,
cartanclass.h:407-408), and class representatives lifted back to elements
(`wrf_rep`, cartanclass.h:404-406). A full sweep of `d_weakReal` consumers
confirms everything else — `makeRealFormPartition`,
`makeStrongRepresentatives`, `toWeakReal`, `isMostSplit`, `specialGrading`,
`toMostSplit`, `map_real_forms` — is square-class, strong-real-form,
real-form-label, or Cartan-construction layer, all strictly downstream and
all reachable from these three accessors. Output-layer orbit listing
(`printFiber`) is reconstructible from the class table.

## Data layout and public boundary

```text
WeakRealFormId(pub(crate) usize)   // Copy/Eq/Ord/Hash newtype, like RootId

WeakRealFormPartition::build(&CartanGradingData, max_elements: usize)
    -> Result<WeakRealFormPartition, StructureError>

class_count() -> usize
classes() -> impl ExactSizeIterator<Item = WeakRealFormId>
class_of(&AdjointFiberElement) -> Result<WeakRealFormId, StructureError>
class_representative(WeakRealFormId) -> Option<&AdjointFiberElement>
quasisplit_class() -> WeakRealFormId
adjoint_fiber() -> &AdjointCartanFiber
```

Weak-real-form numbers are language-observable identifiers that the
real-form-label stage will carry across type boundaries, so they get the
`RootId` treatment now — an externally opaque newtype — rather than a
breaking retrofit later. `classes()` is the id-typed enumeration of
`0..class_count()`. `class_of` returns `Result` with the existing
fiber-mismatch error through the element's provenance check;
`class_representative` returns `Option` for out-of-range ids per crate
precedent. `quasisplit_class()` is a method, not a constant: it returns the
class of the identity element, which is class 0 under this crate's
numbering, and stays renumberable by a future compatibility adapter. The
representative of a class is its minimal element in canonical-coordinate
integer order — a deterministic non-canonical choice, named
`class_representative` to keep it apart from the fibers'
`canonical_representative`.

The partition consumes the validated grading tables — the base grading, the
grading shifts, and the adjoint `m_alpha` elements are exactly the
`FiberAction` inputs — and `CartanGradingData` already owns the validated
adjoint fiber those tables belong to, so `build` takes no separate fiber or
involution input. The returned value owns exactly:

```text
adjoint: AdjointCartanFiber          (Arc-backed clone; provenance-preserving)
class_of_by_mask: Vec<u32>           (indexed by mask; sentinel u32::MAX)
representatives: Vec<AdjointFiberElement>
```

The Arc-backed clone keeps `Arc::ptr_eq` validation working for elements
minted by the original fiber while rejecting independently built identical
fibers. The grading tables themselves are consumed into fixed-width masks at
build time and are not retained.

Internally the orbit walk is pure fixed-width bit arithmetic, exactly
`FiberAction`: masks are the canonical coordinates of elements, and the
transposed shift columns, the adjoint `m_alpha` coordinate masks, and the
base-grading bits are precomputed once, so the enumeration never calls back
into fiber machinery. Converting between elements and masks needs the
element's canonical coordinates, so `CartanFiber` gains a `coordinates`
accessor and `AdjointCartanFiber` a delegating one — the inner fiber and the
element's tuple field are module-private, so the delegation is part of the
public design, mirroring the `canonical_representative` pair:

```text
CartanFiber::coordinates<'e>(&self, &'e CartanFiberElement)
    -> Result<&'e ModTwoVector, StructureError>
AdjointCartanFiber::coordinates<'e>(&self, &'e AdjointFiberElement)
    -> Result<&'e ModTwoVector, StructureError>
```

The explicit `'e` is load-bearing: elision would tie the borrow to the
fiber, and the vector lives in the element. The accessor validates
membership first, exactly like `canonical_representative`. Element opacity
means provenance, not secrecy: coordinate views are served only by the
owning fiber against its documented low-pivot basis, and bit `j` selects
`basis_representatives()[j]`, whose XOR is the canonical representative —
no new coordinate commitment is created. Class representatives are
materialized back into elements once per class after the walk, by one
ascending rescan of the class table: because classes are numbered by
ascending minimal mask, class `c`'s first occurrence in ascending order is
its minimal mask.

## Resource and arithmetic policy

The enumeration is exponential by nature: `2^dimension` elements. The
caller passes `max_elements`, and construction checks, in this order and
before allocating anything:

1. `dimension <= 63`, rejected as
   `WeakRealFormResourceLimit { resource: "mask bits", limit: 63 }`. Masks
   are `u64`, and this check must precede the power computation, whose shift
   would otherwise be out of range for high-rank data that permissive
   integer-lattice budgets can legally construct.
2. `1u128 << dimension <= max_elements`, compared in unsaturated `u128`
   against the widened caller limit, rejected as
   `WeakRealFormResourceLimit { resource: "fiber elements", limit }`. The
   comparison is deliberately not saturated: saturating would wrongly accept
   `dimension == 64` under `max_elements == usize::MAX`.

During the walk, seeding a class beyond `u32::MAX - 1` is rejected as
`WeakRealFormResourceLimit { resource: "classes", limit }`. This guard is
load-bearing, not defensive: a simply-connected product has zero adjoint
`m_alpha` vectors, so `W_im` acts trivially and every mask is a singleton
class; at dimension 33 with a 32 GiB class table — feasible on the HPC
nodes — the class count would overflow `u32`.

The walk performs at most `2^dimension * imaginary_rank` constant-cost
steps; that bound is implied by the element limit and needs no second knob.
The class table uses sentinel `u32::MAX` rather than an option type (no
niche; half the memory), and the to-do stack reserves `2^dimension` slots
up front, which is an honest bound because each element is pushed exactly
once, at its unclassified-to-classified transition. All allocations use
`try_reserve_exact` with `AllocationFailed`; representative materialization
reserves `class_count` once after the walk rather than reallocating per
class. A product of 33 A1 factors with the identity involution must be
rejected by an honest `max_elements` budget, preserving the dynamic-rank
principle: the limit is the caller's, never a rank cap. No lattice
arithmetic occurs, so no new overflow surface opens.

## Tests and fixture gate

- A2 with the identity involution partitions its four elements into two
  classes: `{0, e0, e1}` seeded by the identity (the quasisplit form) and a
  singleton, matching the two weak real forms of the compact inner class of
  `sl(3)`. This count is symmetric under the root swap, so it is
  order-independent.
- B2 with the pinned Cartan matrix `[[2,-2],[-1,2]]` (long root at simple
  index 0) yields three classes with representatives the identity, `e0`,
  and `e0+e1`, matching the three weak real forms of the compact inner
  class of `so(5)`. The middle representative depends on the pinned simple
  order; the count and the identity's class do not.
- The simply-connected A1 with identity involution yields two singleton
  classes: adjoint `m_alpha` is zero, so `W_im` acts trivially.
- The A2 diagram twist has a zero-dimensional adjoint fiber and exactly one
  class whose representative is the identity.
- Class 0 contains the identity element in every case above,
  `quasisplit_class()` equals `class_of(identity)`, and
  `class_of(class_representative(c))` is `c` for every class.
- A foreign adjoint-fiber element is rejected by `class_of` with the
  existing fiber-mismatch error.
- The 33-factor A1 product is rejected by an honest `max_elements` budget
  with the `"fiber elements"` resource error; an undersized budget on A2 is
  rejected before any allocation; a rank-64 A1 product under
  `max_elements == usize::MAX` is rejected with `"mask bits"`.
- The `u32` class guard is exercised through a private helper with a
  synthetic count, since reaching it publicly needs `2^32` classes; the
  helper is production code called by the walk, so nothing is test-only.

`tests/fixtures/domain/weak_real_form.atlas` is reserved for the later
differential corpus; it stays declared and unexecutable until the
language-level constructors and both oracle adapters exist. Real-form
COUNTS per inner class are a natural early differential target once the
adapter lands, since Atlas prints them directly.

## Consequential updates

Landing this stage must also update: `lib.rs` (module and alphabetical
export of `WeakRealFormId`/`WeakRealFormPartition`); the grading module doc
("`W_im` orbits live in `WeakRealFormPartition`; real-form labels and
strong real forms are later layers"); the Cartan-fiber module doc (add the
partition to the implemented list; strong real forms and KGB remain);
`REAL_GROUP_DESIGN.md` (drop "weak real forms" from the unimplemented list
and advance the progression paragraph to real-form labels and Cartan
classes); and the `TwistedConjugacyClass` doc, whose "later additions"
sentence should say "fiber groups, real-form attribution, and real Cartan
component data are later additions to this type" so it does not read as a
crate-wide claim. (Superseded by the aggregation stage: those data live in
the sibling owner `CartanClass`, which owns a `TwistedConjugacyClass`
value; see `CARTAN_AGGREGATION_DESIGN.md`.) `InnerClass` deliberately gains
no convenience constructor: the real-form-label stage will decide what
`InnerClass` owns.

## Three independent design checks

1. The Atlas semantics review confirmed the `FiberAction` formulas, the
   `makeWeakReal` inputs, the ascending-seed orbit numbering with
   seed-as-representative, the quasisplit-class-zero normalization, the
   scope boundary (a full `d_weakReal` consumer sweep found nothing this
   stage forgets), and the absence of any upstream sparse shortcut or
   budget beyond `RANK_MAX`. It corrected the involutivity claim to the
   input-conditional parity statement now in the construction section,
   repointed the membership citation to `Fiber::adjoint_orbit`, and flagged
   the B2 representative anchor as simple-order-dependent, now pinned.
2. The Rust internals review supplied the delegating `coordinates`
   accessor with its explicit element-bound lifetime, reordered the budget
   checks so the mask-bits gate precedes the power computation, replaced
   the saturated comparison with an unsaturated `u128` one, established
   that the `u32` class guard is reachable in principle and therefore
   load-bearing, and specified the sentinel table, the once-reserved to-do
   stack, and the post-walk representative rescan.
3. The consumer review minted `WeakRealFormId` now rather than as a
   breaking change at the label stage, added `classes()` and the partition's
   `adjoint_fiber()` accessor, confirmed the naming and Option/Result
   conventions and the plain `max_elements` parameter, rejected an
   `InnerClass` convenience constructor, and enumerated the consequential
   doc updates above.

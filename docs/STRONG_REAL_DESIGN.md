# Strong-real layer design

## Approved scope

This stage ports, per Cartan class: the central square-class partition of
the weak real forms, the per-square-class strong-real partitions of the
FIBER group, and the strong representatives; and, at the classification
level, `fiber_size`, `kgb_size`, and `global_kgb_size` — the first
differential numbers comparable against published Atlas values. Out of
scope: the dual side entirely (`dualFiberSize`, block sizes), strong real
form naming, inner-class-level square classes beyond what the sizes need,
and KGB itself.

## Atlas construction

Citations are to `~/mycodes/atlasofliegroups`, master `4d3e9449`.

`makeRealFormPartition` (cartanclass.cpp:550-579) forms the quotient of the
adjoint fiber group by the image of `toAdjoint` (a `SmallSubquotient` with
full-space numerator) and assigns each weak real form the coset coordinate
integer of its orbit representative, as an UNNORMALIZED partition: the
square-class number IS the coordinate integer in the quotient's canonical
echelon basis, every value in `[0, 2^m)` occurs — orbits refine cosets
because `toAdjoint` intertwines the `W_im` actions, and cosets cover the
group — and `classRep(c)` is the smallest weak-form index in the class.
Surjectivity is load-bearing upstream (the unnormalized constructor indexes
by raw value), so the port checks it. `class_base(c)`
(cartanclass.h:353-355) is that form's adjoint orbit representative — the
coset base point.

`makeStrongReal` (cartanclass.cpp:617-645) runs, per square class `c`, one
`W_im` orbit walk over the `2^fiberRank` FIBER group with `FiberAction`
inputs: base grading = `grading(class_base(c))`; fiber-side grading shifts
`gs[i]` = the XOR of adjoint grading shifts selected by column `i` of
`toAdjoint` (cartanclass.cpp:626). Exactly:
`grading(toAdjoint(b_i)) = ambient_base XOR gs[i]`, where `ambient_base`
is the fiber's all-noncompact base grading — the grading of the ZERO
adjoint fiber element — so `gs[i]` is computed ONCE, independent of the
square class; only the per-class base grading varies. Substituting
`grading(class_base(c))` into the shift computation would corrupt the
orbits (a parity-dependent term, not a translation), which is why this
sentence is explicit. Translations = the AMBIENT `m_alpha` fiber elements
(cartanclass.cpp:629, 409-419), not the adjoint ones.
`makeStrongRepresentatives` (654-688) solves, per weak form,
`toAdjoint(x) = (the form's adjoint orbit representative) XOR class_base(c)`
over `F_2` (a particular solution; upstream supports it on pivot bits with
free variables zero; solvability is guaranteed since both sides lie in one
`im(toAdjoint)` coset) and stores the fiber orbit of `x` with the square
class. Translation by `ker(toAdjoint)` commutes with the action — a kernel
element's pulled-back shift combination is the shift combination of zero —
so every solution's orbit has the same size; only the stored orbit NUMBER
is solver-convention-dependent.

`fiberSize(rf, cn)` (innerclass.cpp:602-618): convert the global form to
the local weak class through the label list, take its strong
representative, and return the class size of that orbit in its square
class's fiber partition — always the ordinary fiber, never the dual.
`KGB_size(rf)` = sum over the form's Cartan set of
`orbitSize(cn) * fiberSize(rf, cn)` (innerclass.cpp:850-860);
`global_KGB_size` = sum over all Cartans of
`orbitSize * numRealFormClasses * 2^fiberRank` (867-878).

Hand-verified anchors (simply connected, identity inner classes; per-Cartan
terms `orbitSize x fiberSize`): SL(2): `sl(2,R) = 1*2 + 1*1 = 3`,
`su(2) = 1`, global 5. A2: `su(2,1) = 1*3 + 3*1 = 6`, `su(3) = 1`,
global 7. B2: `so(3,2) = 1*4 + 2*1 + 2*2 + 1*1 = 11` (the known Sp(4,R)
value), `so(4,1) = 1*2 + 2*1 = 4`, `so(5) = 1`, global 17 (`so(5)`
carries two strong forms: its square class's partition is `{0,3},{1},{2}`
with two singletons over the one weak form). The two B2 reflection
Cartans are identified invariantly: the one carrying two weak forms
contributes `2*1` to the split form, the split-only one `2*2`. A1 x A1 is
fully multiplicative — Cartans, orbit sizes, fibers, and `W_im` all
factor — so `kgb_size(split x split) = 9` and the global is the product
`25`.

Numbering observables: square-class numbers are coset coordinate integers
in the quotient's echelon basis — basis-convention-dependent, so this
crate's numbers need not match Atlas's and stay under the adapter
deferral, while the partition STRUCTURE and all sizes are invariant;
weak/strong orbit numbers follow the ascending-minimal convention both
sides share; the strong representative's orbit number is
solver-convention-dependent, its class size is not.

## Data layout and public boundary

```text
// strong_real.rs
SquareClassId(pub(crate) usize)        // CartanId derive set; the number IS
                                       // the coset coordinate integer in the
                                       // crate's echelon basis
StrongRealFormRep { fiber_orbit: usize, square_class: SquareClassId }  // Copy;
                                       // fiber_orbit is solver-convention-
                                       // dependent, its class size is not

StrongRealData {                       // per Cartan class; Clone, Debug
    central_square_classes: Vec<SquareClassId>,   // by local weak class
    class_bases: Vec<AdjointFiberElement>,        // by square class
    orbit_sizes: Vec<Vec<usize>>,                 // by square class, by orbit
    strong_representatives: Vec<StrongRealFormRep>, // by local weak class
}
  square_class_count() -> usize
  central_square_class(local: WeakRealFormId) -> Option<SquareClassId>
  strong_real_form(local: WeakRealFormId) -> Option<StrongRealFormRep>
  fiber_orbit_count(square: SquareClassId) -> Option<usize>
  fiber_size(local: WeakRealFormId) -> Option<usize>

StrongRealClassification::build(&CartanClassification, max_fiber_elements)
    -> Result<StrongRealClassification, StructureError>

strong_real_data(cartan: CartanId) -> Option<&StrongRealData>
fiber_size(form: WeakRealFormId, cartan: CartanId) -> Option<usize>
kgb_size(form: WeakRealFormId) -> Option<usize>       // precomputed
global_kgb_size() -> usize
```

Names follow Atlas vocabulary per crate convention. `SquareClassId` gets
the newtype treatment because square-class numbers cross the API boundary;
the strong representative is a named public struct so its two components
cannot be swapped and the solver-convention caveat has a place to live.
No public partition type exists: the walks' mask tables are transient, and
only per-orbit sizes are retained — a public partition arrives when KGB
demonstrably needs one. `fiber_size(form, cartan)` returns `None` only for
an out-of-range id; a valid pair where the form does not live at that
Cartan returns `Some(0)` — the form has no fiber elements there, and KGB
sums over all Cartans stay correct. Per-form `kgb_size` values are
precomputed at build with checked arithmetic — overflow is a build-time
`ArithmeticOverflow` — so the accessor is a pure lookup like `most_split`.
`global_kgb_size` is likewise precomputed. `CartanClass` stays untouched:
the strong layer is a sibling consumer of the finished
`CartanClassification`, whose surface was verified sufficient; a future
KGB build must decide its own pairing gate between the two
classifications explicitly.

Mechanics on existing APIs: the image span of `toAdjoint` is built by
minting each ambient fiber basis element (`element_from_ambient` of each
`basis_representatives()` entry), applying the validated fiber map —
obtained ONCE per Cartan, since the accessor constructs it — and inserting
the images' adjoint COORDINATES (never lattice-rank representatives) into
a `ModTwoSubspace`; the square quotient is a full-space-numerator
`ModTwoSubquotient` over it, and `to_coordinates` bits give the class
integer. Fiber-side shift columns are built bitwise — bit `j` of column
`i` is `grading(fiber_map(b_j)).is_noncompact(i) !=
base_grading().is_noncompact(i)` — because `Grading` deliberately has no
XOR; per-square-class base bits come from `grading(class_base)`, and
translation masks from `coordinates()` of the ambient `m_alpha` elements.
The mask walk is the weak stage's, extracted into a crate-internal helper
(`walk_mask_orbits(dimension, max_elements, base, alpha_columns,
m_alpha_masks, limit_error)`), dimension-agnostic and parameterized over
the resource-limit constructor: the strong stage's walks emit
`StrongRealResourceLimit { resource, limit }` with the same
`"mask bits"`/`"fiber elements"`/`"classes"` resources, so error
attribution stays honest, and the weak stage keeps its own variant
unchanged. The walk dimension is the AMBIENT fiber dimension, which
exceeds the adjoint one whenever central directions exist — both the
63-bit gate and the element gate apply to it, so a budget that passed the
weak build can honestly fail here. One class table buffer is reused
across square classes. The strong-representative solve reuses the
augmented-elimination pattern with marker bits over the adjoint dimension
plus fiber dimension; the resulting fiber mask is classified against the
square class's transient table before it is dropped, and the stored pair's
square class is checked against `central_square_class`
(`"square class consistency"`).

Errors: `StrongRealResourceLimit { resource: &'static str, limit: usize }`
and `StrongRealInvariantViolation { invariant: &'static str }` with
`"square class count"`, `"strong representative"`,
`"square class consistency"`; pass-through includes each layer's own
errors and `AdjointFiberResourceLimit` from the fiber-map applications,
which consume projection budget.

## Tests and fixture gate

- SL(2) simply connected, identity: `kgb_size` 3 and 1, global 5; the
  fundamental Cartan has two square classes with fiber sizes 2 and 1.
- A2 identity: `kgb_size` 6 and 1, global 7; one square class at the
  fundamental Cartan (invertible `toAdjoint`).
- B2 identity: `kgb_size` 11, 4, 1 in fundamental-partition form order
  (split, so(4,1), so(5)); global 17; the so(5) square class has three
  orbits with two strong forms over the one weak form
  (`fiber_orbit_count` 3, two singleton sizes).
- A1 x A1 identity: `kgb_size(split x split) = 9` and global 25, the
  product-group cross-check.
- Budget rejection: an undersized `max_fiber_elements` rejects with the
  STRONG stage's `"fiber elements"` resource error; invariant violations
  are unreachable through public constructors and are asserted by
  construction.

`tests/fixtures/domain/strong_real.atlas` is reserved; the KGB sizes are
the flagship differential targets once the language adapter exists.

## Consequential updates

Landing this stage must update: `lib.rs` (module `strong_real` between
`root_system` and `twisted_involution`; exports `SquareClassId`,
`StrongRealClassification`, `StrongRealData`, `StrongRealFormRep`);
`cartan_class.rs` (the `CartanClass` doc's "later additions" sentence:
the dual fiber and `simpleComplex` remain later additions, while the
strong-real layer lives in the sibling `StrongRealClassification`);
`cartan_fiber.rs` module doc (strong real forms now exist; KGB remains);
the weak-real-form and grading module docs;
`CARTAN_AGGREGATION_DESIGN.md` (one-sentence confirmation that the
predicted no-new-accessors reach path held); and `REAL_GROUP_DESIGN.md`'s
fiber-boundary and progression paragraphs (next: KGB data). The B2 anchor
derivation used C2 coordinates; the test pins the crate's own B2 matrix
and identifies the reflection Cartans invariantly.

## Three independent design checks

1. The Atlas semantics review re-derived all five anchors — including the
   B2 square-class split, the three-orbit so(5) class, and both A1 x A1
   product claims — verified the quotient construction with its
   load-bearing surjectivity, the kernel-translation independence, and
   the size formulas, and issued one blocking phrasing correction now in
   the construction section: the shift columns are measured against the
   fiber's ambient all-noncompact base grading and are
   square-class-independent; substituting the per-class base would
   corrupt the orbits.
2. The Rust internals review confirmed implementability on existing APIs
   with no visibility changes, specified the extraction split for the
   shared mask walk and its dimension-agnosticism, flagged the
   coordinates-versus-representative dimension trap and the
   ambient-versus-adjoint gate difference, required the fiber map to be
   obtained once per Cartan, added the projection-budget pass-through,
   and recommended retaining per-orbit sizes rather than mask tables.
3. The consumer review aligned the surface with Atlas vocabulary, minted
   `SquareClassId` and the named `StrongRealFormRep`, made `kgb_size`
   precomputed, pinned the `Some(0)` not-at-this-Cartan rule, added the
   two demonstrably needed accessors, replaced the borrowed weak-stage
   error with the stage's own resource variant via the parameterized
   helper, and completed the consequential-update list.

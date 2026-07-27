# Real-form label design

## Approved scope

This stage ports the per-Cartan real-form correlation: for one Cartan class
of an inner class, the map sending each weak-real class of that Cartan's
adjoint fiber to the weak-real class of the FUNDAMENTAL fiber that is the
inner class's global real-form number. It follows the grading-based legacy
mechanism, whose last working version is upstream commit `858041bd`; the
modern sample-transport route through the based adjoint Tits coset is
deliberately not ported. Out of scope: dual real forms and dual labels,
central square classes, strong real forms, Cartan-set and most-split
derivations (both derivable from labels later), real-form names, and Atlas's
user-visible real-form numbering, which is an output-layer sort
(output.cpp:82-140) belonging to a later presentation adapter.

## Atlas construction

Citations are to `~/mycodes/atlasofliegroups`, master `4d3e9449`; the
correlation loop text sits in `#if 0` (innerclass.cpp:1442-1580) and is
bitrotted, so semantics follow the last working ancestor `858041bd`
(complexredgp.cpp:517-560) with two modernizations recorded below.

The global numbering is exactly the fundamental fiber's weak-real
partition: `d_fundamental` is built at the distinguished involution
(innerclass.cpp:137), `numRealForms` delegates to it (innerclass.h:332),
`quasisplit() == RealFormNbr(0)` (innerclass.h:343), and on the fundamental
Cartan the label list is the identity. This crate already implements that
partition; the stage's entire content is the correlation at the other
Cartans.

Per weak-real class `j` of the fiber at a Cartan representative `ti` with
decomposition `(so, u)` (`Cayley_and_cross_part`):

1. Take the grading `gr` of the class representative over the
   simple-imaginary roots `rl` at `ti` (innerclass.cpp:1479-1482).
2. Pull `gr` back through the Cayley set: for each position `i` and each
   `alpha` in `so`, flip bit `i` when `rl[i] + alpha` is a root
   (`gradings::transform_grading`, gradings.cpp:266-273) — the B2-parity
   rule; for orthogonal imaginary roots the sum and difference tests agree
   by root-string symmetry, so the stored sign normalization of `so` is
   harmless.
3. Extend `rl` by the roots of `so` and extend `gr` by all-noncompact bits
   (innerclass.cpp:1486-1488): the Cayley condition.
4. Cross-transform the ROOT LIST by the composite `u` — positions of `gr`
   travel unchanged — after which every root is imaginary for the
   distinguished involution (innerclass.cpp:1489-1494;
   `crossTransform` applies the word as the element it denotes, which is
   exactly the stored `cross_action`).
5. Solve for a fundamental adjoint-fiber element with grading `gr` at `rl`
   (`makeRepresentative`, innerclass.cpp:1380-1412): base column = the
   fundamental base grading extended to `rl` by mod-2 linearity; columns =
   the fundamental grading shifts restricted to `rl`; right-hand side =
   `gr XOR base`; first F2 solution taken. The working 2009 version ignored
   the solver's boolean — no solution was impossible-by-invariant; the
   modern dead text throws. This port errors honestly instead.
6. The label is the fundamental partition's class of that element
   (working semantics `class_of`; the dead text's `weakReal()(x)` no longer
   compiles — recorded bitrot, like the integer-element `grading` calls).

Upstream asserts `labels[0] == quasisplit()` (innerclass.cpp:1502 and the
live analogue 563); here that is a checked invariant. Label
well-definedness across different representatives of one Cartan class is a
mathematical invariance the differential corpus confirms; structurally the
crate requires only that the fiber chain and the decomposition use the SAME
representative.

## The base-grading extension

Step 5's base column needs the fundamental base grading at arbitrary
imaginary roots: noncompact iff the coefficient sum of the root over the
simple-imaginary basis is odd (`makeBaseGrading`, cartanclass.cpp:304-327).
This is the extension the grading stage explicitly deferred, and it now
arrives in its budgeted form: express the root in the simple-imaginary
basis by pairing with the subsystem coroots (`bracket`) and solving the
imaginary-subsystem Cartan system exactly over the rationals with the
Malachite `Rational` arithmetic the crate already uses for finite-type
validation; the solution is integral, and only its coefficient-sum parity
is consumed. The solve is `imaginary_rank`-sized dense elimination per
root, bounded by already-budgeted inputs (`imaginary_rank <= semisimple
rank`, list length `<= imaginary_rank + |so|`), so no new caller knob is
introduced; a non-imaginary input or a singular subsystem system is an
invariant error, never a silent zero. The shift columns need no extension
machinery: bit `k` at root `beta` is the mod-2 pairing of `beta`'s
full-simple coordinates with the `k`-th adjoint basis representative,
exactly as the grading stage computes it for simple-imaginary roots.

## Data layout and public boundary

```text
RealFormLabels::build(
    inner_class: &InnerClass,
    fundamental_grading: &CartanGradingData,
    fundamental_partition: &WeakRealFormPartition,
    cartan_grading: &CartanGradingData,
    cartan_partition: &WeakRealFormPartition,
    decomposition: &CayleyCrossDecomposition)
    -> Result<RealFormLabels, StructureError>

labels() -> &[WeakRealFormId]   // position k labels local class k, classes() order
label(WeakRealFormId) -> Option<WeakRealFormId>
```

Flat named references, no tuples: both sides have identical types, so
grouping cannot prevent a swap — the provenance gates do. A side swap is
rejected by the fundamental-involution check except on the fundamental
Cartan itself, where it is semantically harmless; an interleave is rejected
by the partition-fiber identity probes. `WeakRealFormId` serves both sides
deliberately: the global numbering IS the fundamental partition, and the
quasisplit anchor compares label values against
`fundamental_partition.quasisplit_class()` directly; a distinct global
newtype belongs to the later presentation adapter where Atlas's
user-visible renumbering actually lives. The residual local-vs-fundamental
confusion hazard matches the accepted `RootId`-across-systems hazard and is
mitigated by documented conventions on both types. There is no
`class_count` accessor (`labels().len()` is the local count) and no
quasisplit accessor (that is the partition's job). This stage also
discharges the weak-real-form deferral on `InnerClass` ownership:
`InnerClass` gains no fiber, partition, or label storage; per-Cartan state
remains caller-assembled until the Cartan-class stage defines its owner.
`RealFormLabels` deliberately stores no Cartan anchor either — the
Cartan-class stage will hold labels alongside the decomposition, which
already carries the anchor. (Confirmed: `CartanClass` in
`CARTAN_AGGREGATION_DESIGN.md` landed exactly that layout.)

The `InnerClass` anchors provenance: the fundamental grading data must be
built at its distinguished involution, the Cartan grading data at the
decomposition's twisted involution, each partition at its grading data's
fiber, and every datum equal. Concretely: the fundamental side's ambient
involution must equal `distinguished_involution().involution()`, the Cartan
side's must equal the decomposition's stored composed involution, and both
partitions' adjoint fibers must be the same values their grading data own
(the `Arc` provenance the fibers already enforce makes elements
transferable exactly when this holds). The exact gates, in order before any
computation: datum disagreements are `DatumMismatch`; the fundamental
grading's ambient involution not equal to the distinguished involution, and
the Cartan grading's not equal to the decomposition's composed involution,
are `CartanFiberInvolutionMismatch`; the decomposition's factorization is
re-checked against this inner class's distinguished involution
(`DistinguishedInvolutionMismatch`); and each partition is probed with its
grading data's fiber identity element through `class_of`, so a foreign
fiber fails as `CartanFiberMismatch` in constant time — the partition's
build clones its fiber `Arc`-backed, so elements transfer exactly within
one chain. The returned value stores only the label vector (fundamental
`WeakRealFormId` per local class); the heavy inputs stay with their
owners. Derives: `Clone, Debug, Eq, PartialEq`. On the fundamental Cartan
itself — the decomposition of the identity twisted involution — the labels
are the identity map, which is a test, not a special case: with empty
Cayley and cross parts the F2 system is exactly the fundamental
`element_from_grading` system, whose faithfulness gate makes the solution
unique. Away from the fundamental Cartan the restricted columns need not
be independent, so the solver may return one of several elements; label
single-valuedness across them is the representative-invariance the
differential corpus confirms, matching upstream's first-solution
semantics.

The invariant checks inside `correlate`: every cross-transformed root must
be imaginary for the distinguished involution
(`RealFormLabelInvariantViolation { invariant: "fundamental imaginary" }`),
the solver must find an element (`ImpossibleGrading` reused: the caller
asked for a grading no fundamental element has — reachable only through
machinery bugs or mismatched inputs that slipped provenance, and honest
either way), and the quasisplit anchor
(`{ invariant: "quasisplit anchor" }`): the label of the local class of the
identity element is the fundamental quasisplit class.

## Resource and arithmetic policy

Per local class the work is: `|so|` sum-is-root tests per grading bit, one
composite cross application per root, one rational solve of size
`imaginary_rank` per root for the base column, and one F2 elimination of
size `(imaginary_rank + |so|) x fundamental_dimension` — all bounded by
data whose construction was already budgeted, so `correlate` takes no
budget parameter, exactly like the grading stage. Root arithmetic uses the
existing checked coordinate paths; rational elimination is exact; F2 work
uses the existing `ModTwoSubspace` augmented-elimination pattern from
`element_from_grading`. Allocations use `try_reserve_exact`.

## Tests and fixture gate

- A2 with the identity involution: the reflection Cartan has one local
  class, whose label is the quasisplit class — hand-derived: `rl` is empty,
  the Cayley root contributes an all-noncompact bit matching the base
  parity of `alpha_0` (coefficient one over the simple-imaginary basis), so
  the solver returns the identity element. The compact form `su(3)` never
  appears at the split-er Cartan.
- Simply-connected A1: the split Cartan's unique local class labels to the
  quasisplit class; the fundamental Cartan's labels are the identity map on
  two classes.
- B2 with the identity involution: for every twisted-conjugacy class,
  correlation succeeds, `labels[0]` is quasisplit, every label is a valid
  fundamental class, and the fundamental Cartan's labels are the identity;
  the exact per-Cartan label values are pinned as computed the first time
  the test runs and cross-checked against the so(5)/so(4,1)/so(3,2)
  Cartan-attribution facts where hand-derivable.
- The A2 twist inner class: one fundamental class, every Cartan labels to
  it.
- Provenance rejections: a fundamental grading built at a non-distinguished
  involution, a Cartan grading built at a different representative than the
  decomposition, and partitions from foreign fibers each reject with their
  named mismatch errors.
- The base-grading extension is unit-tested directly: parities of
  non-simple imaginary roots in A2 and B2 identity classes match their
  hand-computed simple-imaginary coefficient sums, and a real (non-
  imaginary) root input is an invariant error.

`tests/fixtures/domain/real_form_labels.atlas` is reserved; real-form
counts and per-Cartan label multisets are the first natural differential
targets once the language adapter exists, while raw class numbers remain
crate-order observables under the standing adapter deferral.

## Consequential updates

Landing this stage must update: `lib.rs` (module and exports); the
`weak_real_form.rs` type docs (real-form labels now exist, and
`WeakRealFormId` gains the global-number convention sentence; square
classes and strong real forms remain later layers); `grading.rs`'s
`CartanGradingData` doc ("real-form labels live in `RealFormLabels`");
`cartan_class.rs` (labels correlate through `RealFormLabels` at the same
representative); `inner_class.rs` (anchors provenance for
`RealFormLabels`); `GRADING_DESIGN.md`'s deferral paragraph (the extension
now exists for the label solve); `REAL_GROUP_DESIGN.md`'s
fiber-boundary and progression paragraphs (labels implemented; next Cartan
classes, then KGB); and the `CayleyCrossDecomposition` doc (its first
consumer exists).

## Three independent design checks

1. The Atlas semantics review found no blocking corrections: the five-step
   loop, the sign-normalization safety (orthogonality makes the root-string
   symmetric and all solve columns are mod-2 sign-insensitive), the
   base-extension formula with its integrality argument, the bitrot
   corrections, and the A2 hand anchor were all confirmed by independent
   re-derivation. It settled the cross-transform direction: the stored
   composite `cross_action` IS the element the word denotes, and applying
   it per root equals upstream `crossTransform`. Precision notes folded in:
   `makeRepresentative` is compiled but unreachable upstream; the 2009
   working version ran the anchor unchecked (the assert is later); and a
   future most-split derivation needs the Cartan ordering in addition to
   labels.
2. The Rust internals review corrected the rational system's orientation —
   the elimination matrix is the TRANSPOSE of the bracket-indexed subsystem
   Cartan matrix (`row j`: brackets of every basis root against coroot
   `j`), load-bearing for non-symmetric B2 — and established that
   integrality is not an imaginarity test: the extension helper must gate
   on `RootKind::Imaginary` explicitly, and lives as a crate-internal free
   function over `(&RootSystem, &RootInvolutionData, RootId)`, since the
   grading tables store neither. It barred copying `element_from_grading`'s
   right-hand-side shortcut (the base is not all-ones here), barred
   representing the extended bit list as a `Grading` (its length matches
   neither side's imaginary rank), specified the partition-fiber identity
   probe, moved `combine_roots` into `root_system.rs` for shared use, and
   confirmed the no-budget claim with a full allocation inventory.
3. The consumer review flattened the signature to six named references,
   renamed the constructor to `build`, dropped the redundant local
   `class_count`, kept `WeakRealFormId` on both sides against a premature
   global newtype, rejected minting a per-Cartan bundle before the
   Cartan-class stage defines its owner, pinned the provenance error
   mapping, trimmed the derive set, and completed the consequential-update
   list.

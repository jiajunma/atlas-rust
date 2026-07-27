# Cartan grading design

## Approved scope

This stage ports the grading layer that sits on top of the Cartan and adjoint
fibers: `m_alpha`, adjoint `m_alpha`, the base grading, the grading shifts,
grading evaluation for adjoint-fiber elements, and the unique-element inverse
of a grading. It deliberately does not port weak or strong real forms, real
form labels, `W_im` orbits, Cartan classes, square classes, or KGB data, and
it does not evaluate gradings at non-simple imaginary roots.

The compatibility target remains the Atlas language and its observable
behavior. Nothing in this stage is a language-level claim; the fixture stays
reserved until the constructor syntax and both oracle adapters exist.

## Atlas construction

All citations are to the upstream tree at `~/mycodes/atlasofliegroups`,
master `4d3e9449`.

`Fiber::mAlphas` (cartanclass.cpp:409-419) reduces, for each simple-imaginary
root, its full-lattice coroot mod 2 and expresses the class in the fiber
group's canonical subquotient basis via `toBasis`. The mod-2 reduction of a
possibly negative coordinate must test `coordinate % 2 != 0`; the upstream
comment (bitvector.h:167) warns that testing `== 1` is wrong for negative
values. The membership precondition — the reduced coroot lies in the
numerator `ker_F2(I + theta_Y)` — holds because `theta` fixes the coroot of
an imaginary root.

`Fiber::adjointMAlphas` (cartanclass.cpp:437-456) builds, for each
simple-imaginary root `alpha_i`, the vector over full simple roots `j` of
`bracket(alpha_j, alpha_i) mod 2` — the mod-2 fundamental-coweight
coordinates of `coroot(alpha_i)` on the adjoint cocharacter lattice — and
expresses its class in the adjoint fiber group's canonical basis. On a rank
one factor this vector is the Cartan diagonal `2 ≡ 0`, so an A1 adjoint
`m_alpha` is the zero class in every datum; a test that expects a nonzero
class there pins a wrong implementation.

`Fiber::makeBaseGrading` (cartanclass.cpp:304-327) returns the all-ones
grading on simple-imaginary bit positions by definition, not by computation
(cartanclass.cpp:326, cartanclass.h:160-164): the base point (fiber element
zero) grades every simple-imaginary root noncompact. This normalization is
backed by an external theorem invoked in prose (innerclass.cpp:521-533) and
is applied to every fiber, not only the fundamental one. Atlas additionally
extends the base grading to all imaginary roots by mod-2 linearity — the
parity of the coefficient sum over the simple-imaginary basis — into a
root-number-indexed set. That extension, and the matching `RootNbrSet` form
of the shifts, is the part of the layer this stage excludes.

`Fiber::makeGradingShifts` (cartanclass.cpp:351-394) computes, for each
canonical basis vector of the ADJOINT fiber group with distinguished
representative `rep_j`, the grading over simple-imaginary positions whose bit
`i` is `< alpha_i mod 2, rep_j >`, with `alpha_i` expressed in the FULL
simple-root basis mod 2. The upstream comment (cartanclass.cpp:344-349)
explicitly warns that this coordinate system differs from
`makeBaseGrading`'s simple-imaginary coordinates despite reused names; that
confusion is the layer's primary hazard. Because this stage drops the
all-imaginary-roots extension, every pairing this stage computes lives in
full-simple-root coordinates mod 2 against adjoint fundamental-coweight
representatives.

Grading evaluation (cartanclass.cpp:722-730) is affine-linear:
`grading(x) = base XOR XOR_{j in x} shift_j`. Because the canonical
representative of `x` is the sum of the basis representatives selected by
`x`'s coordinates and the pairing is linear, the evaluation used here is
equivalent and simpler: bit `i` of the grading of `x` is
`1 XOR < alpha_i mod 2, representative(x) >`. Well-definedness modulo the
quotient denominator is the integral argument that a `-1`-eigenvector pairs
to zero with every imaginary root; it applies to any class representative,
canonical or not, because two representatives differ by a denominator
element. This argument requires that the adjoint fiber's denominator consist
of classes liftable to integral `-1`-eigenvectors, which holds because the
fiber stage builds it as the mod-2 image of the integral eigenlattice
(upstream tori.cpp:172,179-182); a fiber normalized against any other
denominator would forfeit the equivalence. `Fiber::gradingRep`
(cartanclass.cpp:740-755) inverts the affine map and throws on an impossible
grading; uniqueness of the inverse is exactly the faithfulness invariant
below. Grading value 1 means noncompact everywhere in this layer
(gradings.cpp:20-23); no flip exists.

`grading_kernel` (cartanclass.cpp:264-294) is the kernel of the linear map
whose columns are the grading shifts. The Fiber constructor asserts it is
zero (cartanclass.cpp:172); in a release build a nontrivial kernel would be
silently accepted and would poison `gradingRep` and downstream real-form
identification (innerclass.cpp:552, 585, 1353). The Rust model replaces the
assert with a construction-time rejection.

## Data layout and public boundary

The value type comes first: gradings get their own newtype rather than a raw
mod-two vector. In the mandated A2 identity test, the ambient fiber
representative, the adjoint fiber representative, and the grading all have
dimension two, so the crate's only runtime guard (`RankMismatch`) cannot
separate them; only a type can. This follows the crate's existing discipline
of `Weight`/`Coweight` and `AmbientCoweight`/`AdjointCoweight`.

```text
Grading(ModTwoVector)   // bit i = i-th imaginary simple root; 1 = noncompact
  Grading::from_noncompact(imaginary_rank, noncompact_indices)
  Grading::imaginary_rank()
  Grading::is_noncompact(index) -> Option<bool>
  Grading::noncompact_indices() -> impl Iterator<Item = usize>
```

`Grading` derives value semantics including `Hash`/`Ord`/`PartialOrd` so
later orbit code can key on it; `ModTwoVector` gains the same derives, which
is sound because its construction provably zeroes padding bits. The derived
order is documented as an arbitrary deterministic map-key order, not a
mathematical one, exactly like `RestrictedWeight`. No `Arc` provenance: a
grading is observable data about named roots.

The table owner is `CartanGradingData`, named after `RootInvolutionData`
(validated derived tables about one involution); `Model` is this crate's
private suffix for `Arc`-shared inner state and stays reserved.

```text
CartanGradingData::build(
    &RootSystem, &RootInvolutionData, &AdjointCartanFiber)
    -> Result<CartanGradingData, StructureError>

imaginary_rank() -> usize
imaginary_simple_roots() -> &[RootId]
imaginary_simple_root(imaginary_index) -> Option<RootId>
m_alpha(imaginary_index) -> Option<&CartanFiberElement>
adjoint_m_alpha(imaginary_index) -> Option<&AdjointFiberElement>
base_grading() -> &Grading                       // all-ones
grading_shift(adjoint_basis_index) -> Option<&Grading>
adjoint_fiber() -> &AdjointCartanFiber
grading(&AdjointFiberElement) -> Result<Grading, StructureError>
element_from_grading(&Grading) -> Result<AdjointFiberElement, StructureError>
```

The build signature is deliberately three inputs. `FiberToAdjoint` is
dropped: it exposes only `apply` and cannot mint adjoint elements, and no
method here maps ambient elements forward. The ambient `CartanFiber` is
dropped as a parameter because value equality cannot express fiber identity
(`CartanFiber` elements are tied to their `Arc`-shared model); instead
`AdjointCartanFiber` gains a `ambient_fiber()` accessor returning the exact
source fiber the adjoint descent was proved against, and `m_alpha` is built
against that fiber, closing the wrong-pairing hole by construction.
`RootSystem` and `RootInvolutionData` stay: neither is stored by the adjoint
fiber, and both are load-bearing (`coroot`, `simple_coordinates`, `bracket`;
`imaginary_simple_roots`). Cross-validation is literally the adjoint build's
own two checks: datum equality between the root system and the involution
data, and involution equality between the involution data and the adjoint
fiber's ambient source.

`CartanGradingData` owns its derived tables — the imaginary simple root
list, the `m_alpha` and adjoint `m_alpha` element lists, the shift gradings
— plus a clone of the `AdjointCartanFiber` (cheap: `Arc`-backed), exposed
via `adjoint_fiber()` so the forthcoming `W_im` orbit builder combines
elements against provably the same fiber. Element inputs to `grading` are
validated by the fiber's own provenance discipline. Two `usize` index spaces
coexist on this type; the parameter names `imaginary_index` and
`adjoint_basis_index` carry the distinction, and each accessor documents its
bound (`imaginary_rank()` versus `adjoint_fiber().dimension()`).

`m_alpha(i)` is the ambient-fiber class of `coroot(imaginary_simple_root(i))`
reduced mod 2, built with the existing `element_from_coweight_mod_two` on
`adjoint_fiber().ambient_fiber()` so subquotient membership is checked by
fiber code rather than assumed. `adjoint_m_alpha(i)` routes the coroot
through the existing `AdjointProjection` (`source_coweight` then
`map_coweight`) and then `element_from_coweight_mod_two` on the adjoint
fiber: the projection's `Pi(y)_j = <alpha_j, y>` IS the bracket vector, so
the pairing and the sign-safe parity rule keep their single existing
implementations.

The faithfulness gate: the `imaginary_rank x adjoint_dimension` shift matrix
must have full column rank over F2, computed with the existing
`ModTwoSubspace` insertion. A dependent column is
`StructureError::GradingShiftsNotFaithful`, rejected at construction. This
replaces Atlas's debug-only assert with an unconditional check and
underwrites the uniqueness contract of `element_from_grading`.

`grading` evaluates bits directly against the element's canonical ambient
representative in the adjoint fiber, using the mod-2 full-simple coordinates
of each simple-imaginary root from `RootSystem::simple_coordinates` with the
sign-safe parity rule. `element_from_grading` XORs the base grading into the
target and solves the shift columns by F2 elimination; an unsolvable target
is `StructureError::ImpossibleGrading`, and solvability plus the
faithfulness gate makes the solution unique. A `Grading` whose
`imaginary_rank()` differs from the model's is rejected with the existing
`RankMismatch { expected, actual }` before any elimination; no new
dimension-error variant is introduced, and the slight imprecision of that
variant's display text for F2 dimensions is a recorded, accepted choice.
Both new error variants are unit-like with no payload, following
`InvalidRootDatumAutomorphism` rather than the labeled resource style,
because neither is a caller-tunable limit. Accessors that merely index owned
tables return `Option` per `root`/`coroot` precedent; the two evaluators
return `Result` because they validate and allocate. The names `grading` (not
`grading_of`) and `element_from_grading` (not `grading_representative`)
follow the crate's bare-noun accessor and `element_from_*` constructor
conventions; "representative" stays reserved for non-canonical choices,
which this unique inverse is not.

## Resource and arithmetic policy

This stage introduces no new budget type. Every input the model consumes was
already constructed under a caller-owned budget (`RootSystem`,
`CartanFiber`, `AdjointCartanFiber`), and the model's own work is bounded by
`imaginary_rank * lattice_rank` bit operations plus one
`imaginary_rank x adjoint_dimension` F2 elimination — no enumeration, no
closure, no unbounded intermediate. Allocations of the derived tables use
`try_reserve_exact` with `AllocationFailed`, matching crate policy. Mod-2
reduction of `i32` coordinates uses the sign-safe `% 2 != 0` parity test;
no other arithmetic on lattice coordinates occurs, so no new overflow
surface opens.

The all-imaginary-roots extension was deferred here and has since arrived
in its predicted budgeted form: the real-form-label stage
(`REAL_FORM_LABELS_DESIGN.md`) implements the exact rational
subsystem-coordinate solve as `base_grading_extension`, gated on root kind
and consuming only its parity, exactly for the `makeRepresentative`-style
fundamental solve. `W_im` orbit generation never needed it: it consumes
only per-simple-imaginary data plus adjoint `m_alpha` translation
(`FiberAction`, cartanclass.cpp:69-97, fed by makeWeakReal at 514-519).

## Tests and fixture gate

The implementation must add tests before it is called structurally complete:

- Equal-rank A1, simply-connected datum, identity involution: fiber and
  adjoint fiber each have dimension one, `m_alpha` is the nonzero ambient
  class, adjoint `m_alpha` is the ZERO class (its bracket vector is the
  Cartan diagonal `2 ≡ 0 mod 2`), the shift matrix `[1]` is faithful,
  element zero grades the unique simple-imaginary root noncompact (the
  quasisplit normalization), and the other element grades it compact.
- A2 with the identity involution: two simple-imaginary roots, shift columns
  forming a permutation matrix in this crate's root order (the imaginary list
  sorts `alpha_2` first, so the Atlas-order identity matrix appears
  antidiagonally), adjoint `m_alpha` values the off-diagonal Cartan columns,
  all four gradings realized, and `element_from_grading` inverting `grading`
  on every element.
- The A2 diagram twist: one simple-imaginary root `alpha_1 + alpha_2`, a
  zero-dimensional adjoint fiber, an empty shift matrix that is vacuously
  faithful, base grading all-ones still evaluated, and the all-compact
  target rejected as `ImpossibleGrading`.
- The A1 x A1 swap involution: no imaginary roots, imaginary rank zero,
  every accessor total on the empty index set, and the empty grading
  round-tripping through `element_from_grading`.
- A central torus: the rank-two A1 datum with coroot `(2,1)` puts the mod-2
  coroot class in the ambient fiber with a nonzero central coordinate,
  distinguishing `m_alpha` from its adjoint image.
- A B2 case with a negative coroot coordinate pins the sign-safe parity rule
  in both `m_alpha` and the projected bracket vector of adjoint `m_alpha`.
- Provenance rejections: an involution-data value or adjoint fiber built
  from a different same-rank datum or involution is rejected before any
  table is computed, and a foreign element is rejected by `grading`.
- The faithfulness gate is exercised directly through the private shift-rank
  helper with an injected dependent column, since no public constructor is
  known to reach a nonfaithful kernel; the design records this check as
  defensive, exactly like the closure invariants of the root stage.
- A dynamic product of 33 A1 factors with the identity involution keeps the
  layer free of any inherited packed-rank boundary.

`tests/fixtures/domain/grading.atlas` is reserved now for the later
positive and negative differential corpus; it is declared, not executable,
until the language-level constructors and both oracle adapters exist.

## Consequential doc updates

Implementation must also amend the statements this stage makes stale:
`cartan_fiber.rs`'s module comment and `REAL_GROUP_DESIGN.md`'s fiber
paragraph now name the owners of adjoint maps and gradings;
`ADJOINT_FIBER_DESIGN.md`'s deferral list drops the four grading items,
marks the mandate discharged under the final type name, and keeps its
rejection of exposing gradings through `CartanFiber` verbatim as the
recorded reason for the separate type; `REAL_GROUP_DESIGN.md`'s compactness
invariant is amended to "compact/noncompact labels exist only as `Grading`
bits derived from a validated `CartanGradingData`, evaluated at an
adjoint-fiber element" with `RootKind` deliberately carrying no compactness
variants; and the crate-private legacy `compact_imaginary` prototype in
`lib.rs` gains a doc line stating it is an unvalidated caller assertion
superseded by `Grading`. `lib.rs` exports
`grading::{CartanGradingData, Grading}` in alphabetical position.

## Structural preflight evidence

XMU HPC job `3473257` completed on 2026-07-27 using Rust 1.96. It passed the
package format check, Clippy with warnings denied, and 102 structural unit
tests, including the simply-connected A1 quasisplit normalization with its
zero adjoint `m_alpha`, the A2 grading bijection, the A2-twist impossible
grading, the central-torus and sign-safe-parity cases, provenance and
injected-nonfaithfulness rejections, and the dynamic product of 33 A1
factors. The job ran from frozen source snapshot
`db34c980ed7bb00f9fa1669733985a757d1c6f46a67c3edae08404288e2dc348`; its JSON
report checksum is
`1e056987efc59c4b2975697c3d92d9009045ff3deb38fe97dbc771af4659e809`.

This is Rust structural evidence only. The report lists the grading fixture
as declared and unexecuted, and no reference Atlas constructor or event
adapter ran in this job.

## Three independent design checks

1. The Atlas semantics review confirmed every formula, citation, and the
   equivalence proof obligations of the representative-pairing evaluation
   (linearity plus integral `-1`-eigenvector lifting of the denominator,
   with the lifting precondition now recorded), confirmed 1 = noncompact
   with no flips, and confirmed three of four worked examples. It corrected
   the fourth: the A1 adjoint `m_alpha` is the zero class, and the test
   expectation was fixed before implementation. It also verified that
   dropping the all-imaginary extension is sound through weak real forms,
   square classes, and strong real forms, and named the real-form-labeling
   stage as the first true consumer.
2. The Rust internals review could not be completed as an independent
   fresh-context run: four attempts each died on the same subagent
   infrastructure DNS failure while reviews 1 and 3 succeeded. Its question
   list was instead verified in-session against the actual sources before
   implementation: `ModTwoSubspace::insert` returns whether rank increased
   (the faithfulness gate is `!insert`), `ModTwoVector` gained the `dot`
   pairing and the map-key derives (sound because constructors provably zero
   padding bits), element provenance rides the fibers' own
   `canonical_representative` checks, the inverse uses augmented elimination
   with marker bits over existing subspace APIs, and every allocation is
   `try_reserve_exact`-bounded by `imaginary_rank`, the semisimple rank, or
   the adjoint dimension. The 102-test suite, warnings-denied Clippy, and
   the HPC preflight stand as the executable check of the same claims; an
   independent re-review remains welcome when the infrastructure recovers.
3. The consumer review replaced the raw mod-two grading carrier with the
   `Grading` newtype (equal dimensions make `RankMismatch` unable to
   separate the three coordinate systems), renamed the owner to
   `CartanGradingData` and the evaluators to `grading` /
   `element_from_grading`, reduced the build signature to three inputs by
   adding `AdjointCartanFiber::ambient_fiber()` (fiber identity cannot be
   expressed by value equality, so the ambient fiber must come from the
   adjoint fiber itself), added the `adjoint_fiber()` accessor for the
   forthcoming orbit builder, kept `Option`/`Result` conventions, and
   listed the doc sites this stage makes stale.

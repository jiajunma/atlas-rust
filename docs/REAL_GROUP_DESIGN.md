# Real reductive group design

## Scope and migration order

`atlas-real-group` is an owned mathematical layer, not a C++ query wrapper.
The migration starts with root data and only then adds Weyl actions, Cartan
involutions, restricted roots, real forms, and KGB graphs:

```text
character/cocharacter lattices
  -> based root datum
  -> canonical Weyl actions
  -> lattice involution, root action, and restricted-root system
  -> partial inner-class state / real forms
  -> KGB graph and representation algorithms
```

Each stage needs its own small accepted and rejected Atlas corpus before it is
called compatible. The existing A1 prototype is deliberately not a claim that
the corresponding Atlas constructor syntax or KGB semantics are supported.
The current domain fixture is reserved rather than executable because Atlas
syntax has not yet exposed this Rust domain API; no real-group feature is
marked compatible before the dedicated HPC oracle job exists.

## Core invariants

- `Weight` and `Coweight` are different Rust types. They can be paired but
  cannot be passed to each other's operations accidentally.
- `RootSystem`, `LatticeInvolution`, and `WeylAction` each retain their
  `BasedRootDatum` provenance. Combining equal-rank data from different root
  data is rejected before any root or matrix calculation.
- A `BasedRootDatum` stores both `lattice_rank` and `semisimple_rank`; central
  tori and higher rank are not erased. Its Cartan matrix is checked for exact
  symmetrizability and positive definiteness, so affine and indefinite data
  cannot enter a real reductive group.
- The mathematical model is dynamically sized. The Atlas C++ fixed
  `RANK_MAX = 32` behavior is enforced only by specific compatibility APIs
  that need its packed representation.
- `LatticeInvolution` acts on both lattices and preserves their pairing. A
  `RootInvolutionData` is created only after that lattice action is checked to
  permute the generated ordinary roots and to transport each stored coroot to
  the coroot of its image root; pairing preservation alone admits actions that
  fix every root while moving central-torus coroot coordinates.
  `InnerClass` additionally requires its
  distinguished action to permute the original simple roots; actual Cartan
  involutions are `TwistedInvolution` values of the form `w after delta`.
  Compact/noncompact labels are never a bare flag on an arbitrary root
  action: they exist only as `Grading` bits derived from a validated
  `CartanGradingData`, evaluated at an element of the adjoint Cartan fiber
  and indexed by the simple-imaginary root list. `RootKind` deliberately
  carries no compactness variants.
- Ordinary roots retain their coordinates in the original simple-root basis.
  `RootInvolutionData` therefore derives the inherited simple bases of the
  imaginary and real root subsystems exactly; it does not infer positivity from
  an arbitrary ordering of ambient lattice vectors.
- Weyl-element equality is equality of canonical actions in a datum context,
  never equality of generator words. Enumeration is an explicitly requested,
  resource-bounded algorithm rather than a constructor side effect.
- `TwistedInvolution` is the checked root action `w after theta` used to seed
  Cartan-class enumeration. Its constructor rejects a Weyl translate whose
  square is not the identity; it is not itself a Cartan class or a KGB node.
- Weyl and twisted-involution enumeration are both explicit, caller-budgeted
  operations. `InnerClass::twisted_involutions` intentionally returns the
  unquotiented candidates. `InnerClass::twisted_conjugacy_classes` groups them
  by the Atlas twisted-conjugacy relation but exposes only a deterministic,
  non-Atlas-canonical representative; Atlas canonicalization, Cayley ordering,
  fibers, and real-form data remain later.
- Restricted roots are nonzero classes in the exact quotient
  `X^* / ker(1-theta)`. The implementation uses the image under `1-theta` as
  an opaque canonical quotient encoding, so `alpha-theta(alpha)` is never
  mistaken for an ambient restricted-root coordinate. Ordinary roots are
  grouped into deterministic fibers; multiplicity is derived from fiber size,
  and `lambda` versus `2lambda` is retained for nonreduced systems.
- The future KGB graph will use stable node IDs and explicit simple-root
  transitions, including cross, Cayley, inverse Cayley, and unavailable cases.

## Arithmetic and resource policy

Language integers and rationals use Malachite. Root-data coordinates remain
fixed-width signed integers, with `i128` checked intermediates and checked
narrowing. Domain code returns `StructureError::ArithmeticOverflow` instead
of wrapping or relying on C++ signed-overflow behavior. Explicit compatibility
operations may be modular only when the upstream operation is documented as
such.

Root-system and Weyl-group closure must receive a caller-visible resource
budget. A fixed global enumeration limit is not a mathematical invariant and
must not be used to decide datum validity.

The upcoming Cartan-fiber layer uses dynamic `F_2` vector spaces for finite
torus-component quotients. This is distinct from general integer arithmetic:
`ModTwoVector` and `ModTwoSubspace` have no packed rank cap, while integral and
rational coordinates continue to use checked fixed-width data or Malachite as
appropriate.

## Cartan-fiber boundary

Atlas's Cartan fiber is not merely the kernel of an involution modulo two.
For cocharacter lattice `Y` and Cartan involution `theta`, its abstract group
is `Y^theta / (I+theta_Y)Y`. Atlas does not use that quotient's naive ambient
coordinates: it starts from a character-lattice matrix and packs an isomorphic
group into `Y/2Y`. Once Rust is given the already-dual cocharacter action
`theta_Y`, the current structural implementation follows the C++ formula

```text
ker_F2(I + theta_Y) / red_2 ker_Z(I + theta_Y).
```

The denominator is the mod-two reduction of the saturated integral `-1`
eigenlattice, not the raw mod-two image of `I+theta_Y`. These differ for valid
integral involutions. For example, with
`theta_Y = [[1, 2], [0, -1]]`, the raw image is zero modulo two, while the
negative eigenlattice reduces to the diagonal line; using the raw image would
incorrectly produce a rank-two quotient instead of rank one. Nor may Rust
transpose `theta_Y` again: `LatticeInvolution` already stores the dual action.

`ModTwoSubspace` is therefore only the dynamic finite-field foundation. Its
deterministic row-reduced basis now supports an internal `ModTwoSubquotient`,
whose complement and coordinate extraction follow the observed low-pivot
convention of Atlas `SmallSubquotient`.
The public `CartanFiber` computes the exact denominator first under a
caller-owned `IntegerLatticeBudget`, then exposes opaque elements with
deterministic ambient representatives. It itself deliberately stops before
the C++ `Fiber` object's higher state; adjoint maps live in
`AdjointCartanFiber`/`FiberToAdjoint`, gradings in `CartanGradingData`, and
the weak-real-form partition in `WeakRealFormPartition`, while central-square
classes and strong real forms remain unimplemented.

The internal `integer_lattice` layer performs this exact prerequisite. It uses
a Smith-style diagonal reduction with tracked unimodular column operations,
rather than rational row reduction with cleared denominators. For an integer
matrix `A`, zero diagonal columns in the implicit `U * A * V = D` reduction
give a basis of the full saturated kernel from `V`; independently cleared
rational vectors need not do so. `IntegerLatticeBudget` is public only as the
resource policy for operations such as `CartanFiber::build`; integer matrices
and integral bases remain crate-private.

Its caller-owned budget limits rank, aggregate live working entries, elementary
operations, and coefficient bits. The live-entry bound includes the retained
source matrix used for postcondition verification, the mutable matrix, the `V`
factor, each temporary row or column replacement, and the output basis while it
is materialized. Before a linear combination is materialized, a conservative
product-and-sum bit bound is checked, so a rejected coefficient does not first
allocate an unbounded result. The implementation has exact tests for zero and
full-rank matrices, primitive relations `[2,4]` and `[2,3]`, alternating
row/column reductions, `+I`, `-I`, the swap involution, parity reduction, and
each resource boundary.

For the Cartan-fiber denominator, it computes `ker_Z(I + theta_Y)` directly
from `LatticeInvolution::coweight_matrix()`. That matrix already is the dual
cocharacter action; it must not be transposed again merely because the Atlas
C++ presentation starts with a character-lattice matrix.

XMU HPC job `3462432` ran the structural preflight against Rust 1.96 and
Malachite 0.10 on 2026-07-27. It passed the package format check, Clippy with
warnings denied, and 67 unit tests. Its input snapshot is
`a3e8d6472b20ab767ab5e41443ba29e56db0e9677e94fab2c4f8ff44d79e67f9`, and
its report checksum is
`fdbd319eaa994c86d049f632dcb3b953921d445a903d20d7a11761bb86282a91`.
The job froze the complete submit tree before Cargo and verified that the
snapshot was unchanged afterwards. This establishes only a Rust preflight;
Atlas syntax and oracle-event adapters remain absent, so there is still no
Atlas differential compatibility claim.

## Audit record

Three independent clean-context design reviews shaped this design:

1. The structural audit identified the required separation of character and
   cocharacter lattices, root data, real forms, restricted roots, and KGB.
2. The C++ overflow audit established that Atlas language values are custom
   arbitrary-precision numbers, while domain `int` arithmetic is mixed and
   often unchecked.
3. The rank audit established that 32 is a packed Atlas compatibility limit,
   not a global mathematical rank limit; total lattice rank can also exceed
   semisimple rank.

A subsequent clean-context implementation review checked the Atlas low-pivot
`F_2` convention, the non-symmetric cocharacter-action regression, and the
exact-lattice resource accounting. Two repaired defects are retained in the
tests: reduction now scans every increasing low pivot rather than stopping at a
gap, and `max_entries` accounts for the retained source, working copy, `V`,
temporary replacements, and the output basis.

The HPC evidence review also found that an exit-trap hash of a mutable submit
directory does not prove which source Cargo consumed. The preflight now freezes
the complete submit tree before execution, runs only from that copy, records
the frozen-tree hash, and rejects a changed snapshot. Its dirty-state check
includes untracked files, since these modules are synchronized and compiled.

The canonical-action Weyl module now uses `BasedRootDatum` and does not
enumerate group elements. The pairing-preserving dual-lattice involution and a
caller-budgeted ordinary root system are implemented; `RootInvolutionData`
validates root preservation and coroot transport against that full root system
before classification. Restricted roots now use exact quotient encodings and
retain their fibers. `InnerClass` now owns these three validated values as
partial shared state, but it deliberately does not claim that Cartan classes,
real forms, torus data, or KGB transitions have been implemented. It does
provide deterministic root-theoretic twisted-conjugacy orbits, which are not
Atlas canonical representatives or full `CartanClass` values. The adjoint
Cartan fiber and its descent-proved ambient map are implemented per
`ADJOINT_FIBER_DESIGN.md`, and the ordinary-root/coroot correspondence —
paired closure, `RootSystemBudget`, `bracket`, and dual coweight reflection —
is implemented per `ROOT_COROOT_DESIGN.md`. The grading layer — `m_alpha`,
adjoint `m_alpha`, the all-ones base grading, grading shifts with the
unconditional faithfulness gate, and the unique grading-to-element inverse —
is implemented as `CartanGradingData` with the `Grading` value type per
`GRADING_DESIGN.md`. The weak-real-form partition — `W_im` orbits of the
adjoint fiber with ascending-minimal numbering and the quasisplit class —
is implemented as `WeakRealFormPartition` per `WEAK_REAL_FORM_DESIGN.md`,
and the Cayley/cross factorization of twisted involutions relative to the
distinguished involution as `CayleyCrossDecomposition` per
`INVOLUTION_DECOMPOSITION_DESIGN.md`. The real-form-label correlation —
per-Cartan grading pullback through the decomposition, the budgeted
base-grading extension, and the fundamental-fiber solve with the quasisplit
anchor — is implemented as `RealFormLabels` per
`REAL_FORM_LABELS_DESIGN.md`, completing the structural chain from a based
root datum to the real forms of an inner class and their per-Cartan
attribution. The Cartan classes are now owning values — `CartanClass`
bundling the full per-Cartan machinery, aggregated by
`CartanClassification` with per-form Cartan sets, order-free most-split
classes, and the strict Cayley poset per
`CARTAN_AGGREGATION_DESIGN.md`. The strong-real layer — square classes,
per-square-class fiber partitions, strong representatives, and KGB sizes
matching published Atlas values (Sp(4,R) = 11) — is implemented as
`StrongRealClassification` per `STRONG_REAL_DESIGN.md`. The KGB
construction proceeds by the stage map in `KGB_STAGE_MAP.md`; stage (a) —
the word-level Weyl substrate on the root-permutation representation,
`WeylElement` per `WEYL_ELEMENT_DESIGN.md`, with the `RootSystem`
positivity slice and simple-root IDs it required — is implemented, and
stage (b) — the involution table with canonical-from-theta records,
per-Cartan orbit slices, stored cross links, and the Cayley edge,
`InvolutionTable` per `INVOLUTION_TABLE_DESIGN.md` — is implemented,
and stage (c) — `TitsElement`/`TitsCoset` with the closed-form based
cross action, Cayley, gradings, and the reduction normal form per
`TITS_OPERATIONS_DESIGN.md` — is implemented. Next: stage (d), the
seed `x0` (square-class cocharacter, base grading, binary section
solve, central-fiber minimization).

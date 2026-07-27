# Adjoint Cartan-fiber design

## Approved boundary

This stage ports the finite adjoint-fiber construction and the natural map from
an ambient Cartan fiber. It does not port Atlas `Fiber` wholesale. In
particular, it does not expose weak real forms, strong real forms, or KGB
data; `m_alpha`, adjoint `m_alpha`, base gradings, and grading shifts were
later added by the grading stage (`GRADING_DESIGN.md`).

The public Rust boundary is deliberately typed:

```text
AdjointBasedRootDatum
AdjointFiberBudget
AmbientCoweight
AdjointCoweight
AdjointProjection: Y -> P^vee
AdjointCartanFiber
AdjointFiberElement
FiberToAdjoint: CartanFiber -> AdjointCartanFiber
```

`AdjointProjection` first binds an ambient coweight `y` to its source datum,
then maps it to fundamental-coweight
coordinates

```text
Pi(y)_i = <alpha_i, y>.
```

It is a lattice morphism, not an assertion of an inverse or a canonical lift.
Central coweights can be in its kernel, and its induced fiber map need not be
injective or surjective.

## Atlas construction

Let `T` be the validated root-data involution on the root lattice, expressed in
the full simple-root basis, with column `s` equal to the simple-root coordinates
of `theta(alpha_s)`. Atlas constructs `q = -T^t` and calls
`tori::dualPi0(q)` in `cartanclass.cpp`. Equivalently, if `theta_ad_Y` is the
action on the adjoint coweight lattice `P^vee`, the packed adjoint fiber is

```text
F_ad = ker_F2(I + theta_ad_Y) / red_2 ker_Z(I + theta_ad_Y).
```

The dual action is mathematically inverse-transpose. Since `T` is first
validated to be an involution, `T^-1 = T`, so `theta_ad_Y = T^t` is valid only
after that validation. Rust must construct a pairing-preserving
`LatticeInvolution`; it must not accept a caller-supplied transpose convention.

`AdjointBasedRootDatum` wraps the standard datum on the root lattice, whose
character basis is the simple-root basis and whose cocharacter basis is the
fundamental-coweight basis. This avoids silently treating an adjoint lattice as
an arbitrary `BasedRootDatum`.

## Quotient descent

The source fiber and adjoint fiber are mod-two subquotients. The map is induced
by `Pi mod 2`, not by applying `Pi` only to a chosen normal-form
representative. Construction verifies both:

```text
Pi(source numerator)   is contained in target numerator
Pi(source denominator) is contained in target denominator.
```

Only after these checks may Rust apply the map to its deterministic source
representative. This preserves addition independently of the chosen ambient
representative. Source and target must also carry the same validated ambient
Cartan involution provenance; matching ranks alone is insufficient.

`AdjointCartanFiber::build` accepts the matching source `CartanFiber` rather
than a later same-rank fiber. It proves descent once, then `FiberToAdjoint`
projects on demand. It deliberately does not cache a dense ambient restriction
matrix or quotient-coordinate images.

The existing exact integral-kernel implementation remains the owner of the
denominators. `AdjointFiberBudget` contains its `IntegerLatticeBudget` and two
separate bounds. With adjoint rank `r` and source lattice rank `n`, construction
checks a conservative retained-coordinate bound of `16 r^2 + r n` before it
allocates the target datum or action matrices. It also bounds the worst-case
descent proof by `2 r n^2` direct pairing operations. These are caller-owned
computational limits, not a rank-32 cap or a mathematical restriction. The
finite-field map uses dynamic, fallibly allocated `ModTwoVector` values.

## Explicit deferrals

Atlas's grading layer needs more data than the current domain layer owns.
`m_alpha` and `adjointMAlphas` require arbitrary-root coroots and brackets, and
the grading bit positions require Atlas's explicit ordered imaginary basis.
`makeBaseGrading` uses coordinates in the simple-imaginary subsystem, whereas
`makeGradingShifts` uses full-simple-root coordinates. Rust must not substitute
one parity convention for the other.

This mandate is discharged: `CartanGradingData` (the final name; see
`GRADING_DESIGN.md`) owns that validated phase, rejecting a nonfaithful
grading-shift kernel and impossible gradings instead of encoding missing data
as zero or `Option` values.

## Three independent checks

1. The adjoint-fiber oracle traced `adjoint_involution`, `dualPi0`,
   `adjointMAlphas`, and `makeFiberMap` in Atlas C++. It established the
   root-basis action, the saturated denominator, and the fact that the
   published map is in canonical quotient coordinates rather than a raw
   restriction matrix.
2. The grading oracle traced `makeBaseGrading`, `makeGradingShifts`, and
   `grading_kernel`. It established the two different coordinate systems and
   rejected exposing any grading through the existing `CartanFiber`.
3. A type and quotient review required inverse-transpose duality, a distinct
   adjoint datum/projection type, exact provenance validation, and an explicit
   proof of descent through both quotient denominators. Its implementation
   review additionally rejected pre-budget dense allocation and raw same-rank
   coweight inputs; the current design uses pre-allocation budgets and bound
   `AmbientCoweight` values instead.

These checks select the smallest boundary that models the Atlas algebraic
construction without inventing unavailable real-form semantics. They do not
establish Atlas language compatibility; an adapter and HPC differential corpus
remain required.

## Structural preflight evidence

XMU HPC job `3463683` completed on 2026-07-27 using Rust 1.96. It passed the
package format check, Clippy with warnings denied, and 77 structural unit tests,
including the non-symmetric `A1 x A2` intertwining case, central-kernel case,
provenance rejection, resource rejections, and dynamic adjoint rank 33. The
job ran from frozen source snapshot
`e5fbb4e9841172e25383a75868e6d218eb18ee0376e4c3750b0964e1de61b1e8`; its
JSON report checksum is
`18a98a2016e36de688558c90978dd2622f2a05c568eb6a7a52f583ca85325f80`.

This is Rust structural evidence only. The report lists the adjoint fixture as
declared and unexecuted, and no reference Atlas constructor or event adapter
ran in this job.

## Planned differential cases

The reserved corpus needs accepted and rejected cases for:

- A1 with identity involution;
- an A2 Weyl-reflection Cartan involution, which catches a mistaken second
  transpose of the adjoint coweight action;
- an A1 datum with a central torus, where the fiber map has a nontrivial
  kernel; and
- source-fiber provenance mismatch, foreign projection-bound coordinates, and
  invalid adjoint-image membership; and
- a product of 33 A1 factors, which must remain a dynamic structural case
  rather than inheriting Atlas's legacy packed-rank boundary.

Atlas root numbering and the language constructor syntax are not yet adapted,
so these cases remain structural fixtures until an HPC oracle adapter exists.

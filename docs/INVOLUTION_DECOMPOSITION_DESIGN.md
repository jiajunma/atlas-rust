# Twisted-involution decomposition design

## Approved scope

This stage ports the decomposition of a twisted involution into a Cayley part
and a cross part relative to the distinguished involution: Atlas's
`involution_expr` peeling and `Cayley_and_cross_part` replay, including
`long_orthogonalize`. It is the first half of the real-form-label mechanism;
the grading transforms, the fundamental-fiber solve, and the correlation loop
are the follow-on stage (`REAL_FORM_LABELS_DESIGN.md`, forthcoming). This
stage adds no fiber, grading, or real-form semantics.

## Atlas construction

Citations are to `~/mycodes/atlasofliegroups`, master `4d3e9449`. The two
algorithms are live code consumed by `tits.cpp:756-782` today; the historical
correlation consumer sits in an `#if 0` block whose last working ancestor is
commit `858041bd`.

`weyl::involution_expr` (weyl.cpp:1338-1356) peels a twisted involution `tw`
by repeatedly taking the first left-descent generator in Atlas's INTERNAL
transducer numbering and recording either a Cayley letter (when `s` and `tw`
twisted-commute, `s w = w delta(s)`; peel by left multiplication, Weyl length
drops by one) or a cross letter (otherwise; peel by twisted conjugation,
length drops by two). The output is deterministic and lexicographically
first only for that internal renumbering, which reverses the generator order
of B, C, and D components (weyl.cpp:495-527); Atlas even keeps a separate
`canonical_involution_expr` (weyl.cpp:1358-1385) for the external order. A
matrix port peeling the lowest EXTERNAL descent therefore produces a
different, equally valid word: upstream states both parts of the
decomposition are non-unique (innerclass.cpp:1242-1244) and every consumer
uses only the replay invariant. Consequently the differential gate for this
stage and its consumers must compare replay-invariant and label-level
results, never raw involution words or raw Cayley/cross parts.

`Cayley_and_cross_part` (innerclass.cpp:1255-1289) replays the word in
reverse from the distinguished involution: a Cayley letter appends the
simple root to a growing set `so` and left-multiplies; a cross letter
appends to the cross word, twisted-conjugates, and reflects every root
already in `so` by that generator. The collected `so` is then made strongly
orthogonal by `long_orthogonalize` (rootdata.cpp:784-805): an orthogonal
pair whose sum is a root — necessarily two short roots spanning a B2 — is
replaced by the long sum and difference; one ascending sweep suffices
because every replacement output is long and long roots can never re-pair
(comment rootdata.cpp:778-782). The upstream code silently assumes the set
stays positive and pairwise orthogonal (its sign handling at
rootdata.cpp:791-796 would misbehave on a negative entry); the port asserts
orthogonality and normalizes signs explicitly instead of copying that
latent hazard.

The invariant, verified historically by `checkDecomposition`
(innerclass.cpp:1557-1578) and live as the pre-orthogonalization assert at
innerclass.cpp:1284: starting from the distinguished involution, applying
the cross letters in order by twisted conjugation and then left-multiplying
by the reflection of each Cayley root reproduces `tw`; each Cayley root is
imaginary at its replay step. The imaginarity statement applies to the
final long-orthogonalized roots taken in ascending order — equivalence with
the raw set is the `refl_prod` remark at rootdata.cpp:774-776.

No canonicity precondition exists anywhere in this pipeline: Atlas feeds the
canonical involution of a Cartan class only because that is its stored
per-class representative. The crate's deterministic twisted-conjugacy
representatives suffice, provided later consumers build their fiber data at
the same representative they decompose. Cross-representative invariance of
the eventual labels is confirmed by the differential corpus, not assumed.

## Rust mechanics on the classification layer

The peeling loop needs no matrix arithmetic at all. With the current
twisted involution re-wrapped as a `TwistedInvolution` each step (which
revalidates it and exposes its `RootInvolutionData`):

- Left descent: `s` is a left descent of `w` iff `w^{-1}(alpha_s)` is
  negative, and for a twisted involution `w^{-1} = delta w delta`, so the
  test is `delta(theta(alpha_s))` — two stored image-permutation lookups
  followed by a sign scan of the image root's `simple_coordinates`.
  Negativity must go through `simple_coordinates` of the image `RootId`,
  never ambient coordinate signs, which differ for `from_simple_data` data.
- Letter kind: twisted commutation of a descent `s` is
  `theta(alpha_s) = -alpha_s`, i.e. `kind(alpha_s) == Real` in the current
  involution's classification: a Real descent is a Cayley letter, a Complex
  descent a cross letter, and an Imaginary root can never be a descent
  (its test image is a positive simple root); hitting one is an invariant
  error. The matrix identities (`s w == w delta(s)`; `w^{-1} = delta w
  delta`) hold by the twisted-involution convention (weyl.cpp:1289) and are
  demoted to debug assertions.
- The provenance gate runs before peeling and is load-bearing for
  termination: recompose the candidate's Weyl action with this inner
  class's distinguished involution and require matrix equality with its
  stored composed involution, rejecting with
  `DistinguishedInvolutionMismatch` otherwise. Given that gate, the
  factorization `(w delta)^2 = 1` already validated by
  `TwistedInvolution::new` yields `w^{-1} = delta(w)`, the peeling measure
  is the Weyl length of `w` (strictly reduced by one or two per step), and
  `max_peeling_steps` is purely defensive.
- The twist permutation `delta(s)` is a simple-index permutation because
  every distinguished involution this crate accepts preserves the simple
  system (`InnerClass::new`); it is precomputed once through the
  distinguished classification's image lookups.

The replay direction requires reflections in arbitrary roots, so
`WeylAction` gains a `root_reflection(&BasedRootDatum, &RootSystem, RootId)`
constructor building both dual matrices from the stored root/coroot pair
with the existing checked arithmetic; sign is irrelevant to a reflection,
and the stored Cayley roots are normalized to positive roots and sorted
ascending. Normalization is semantically safe for the follow-on stage: for
orthogonal imaginary roots the Cayley grading flip tests `beta + alpha` and
`beta - alpha` equivalently, by root-string symmetry.

```text
CayleyCrossDecomposition::build(
    &InnerClass, &TwistedInvolution, max_peeling_steps: usize)
    -> Result<CayleyCrossDecomposition, StructureError>

cayley_roots() -> &[RootId]        // strongly orthogonal, positive, ascending
cross_word() -> &[usize]           // simple generators, application order
cross_action() -> &WeylAction      // the replayed composite, u
twisted_involution() -> &TwistedInvolution   // the decomposed input
```

The `InnerClass` parameter is not a convenience: the distinguished
involution is not recoverable from a `TwistedInvolution` (which stores only
its Weyl action and composed involution, and `WeylAction` has no inverse),
and the generator-index cross word is meaningful only under the
based-automorphism guarantee `InnerClass::new` enforces. The value stores
the replayed composite `cross_action` and a clone of the decomposed
`TwistedInvolution` so the label stage recomputes nothing and can assert
identity with its Cartan representative by derived equality. The
intermediate involution and the full root permutation of `u` are
deliberately not stored: each is one existing call away for any consumer.
Derives are `Clone, Debug, Eq, PartialEq`; accessors are total.

Construction verifies what it claims: after peeling it checks pairwise
orthogonality of the raw Cayley set
(`CayleyCrossInvariantViolation { invariant: "orthogonal Cayley set" }`),
long-orthogonalizes with the ascending one-pass sweep (termination measure:
the long-root count strictly increases), normalizes and sorts, then replays
from the distinguished involution — cross letters in order, then ascending
Cayley reflections with per-step imaginarity checks
(`"Cayley root imaginary"`) — and requires equality with the input
(`"replay equality"`). The peeling budget rejection is
`CayleyCrossResourceLimit { resource: "peeling steps", limit }`. All vector
allocations use `try_reserve_exact` with `AllocationFailed`; coordinate
sums and differences for the orthogonalization use checked arithmetic.

## Tests and fixture gate

- The distinguished involution itself decomposes to empty parts, for both
  the identity and a diagram twist.
- A1 x A1 with the identity involution: `tw = s_0` twisted-commutes with
  itself, giving one Cayley root and no cross part. The swap-twist inner
  class has no imaginary roots anywhere, so no Cayley letter can occur in
  it: its unique nontrivial twisted involution `s_0 s_1` decomposes to one
  cross letter and an empty Cayley set.
- A2 with the identity involution: each simple reflection is a single
  Cayley letter; the longest involution decomposes with one cross letter
  and the Cayley root `alpha_0 + alpha_1`, exercising both letter kinds.
- B2 pinned as `[[2,-1],[-2,2]]` (so `alpha_0` is short under this crate's
  `cartan[i][j] = <alpha_i, alpha_j_vee>` orientation): the longest
  involution's raw Cayley set collects the two orthogonal short roots
  `alpha_0` and `alpha_0 + alpha_1`, and `long_orthogonalize` replaces them
  with the long roots `2 alpha_0 + alpha_1` and `alpha_1`. Under the
  transposed pinning the raw set is already long and the pass is a no-op;
  both behaviors are asserted.
- Every enumerated twisted involution of the A2 and B2 identity classes
  and the A2 twist class decomposes, replays to itself, and has pairwise
  strongly orthogonal Cayley roots — the exhaustive invariant sweep.
- A one-step budget rejects with the named `"peeling steps"` resource
  error; a datum mismatch rejects with `DatumMismatch`; a twisted
  involution built against a different distinguished involution over the
  same datum rejects with `DistinguishedInvolutionMismatch`.

`tests/fixtures/domain/involution_decomposition.atlas` stays reserved,
declared and unexecutable until the language-level constructors and both
oracle adapters exist; when the adapter lands, its corpus must compare
labels and replay invariants, never raw words, per the numbering finding
above.

## Consequential updates

Landing this stage must also update: `lib.rs` (module `cayley_cross`
between `cartan_fiber` and `error`, alphabetical export of
`CayleyCrossDecomposition`); the `twisted_involution.rs` type doc (the
Cayley/cross decomposition now lives in `CayleyCrossDecomposition`;
Atlas canonicalization remains unimplemented); the `cartan_class.rs` doc
(representatives are decomposable; consumers must build fiber data at the
representative they decompose); the `inner_class.rs` type doc (it supplies
the distinguished-involution context for decomposition); and
`REAL_GROUP_DESIGN.md`'s progression paragraph, which is additionally
stale from the weak-real-form stage and must now record: weak-real-form
partition and Cayley/cross decomposition implemented, next the real-form
label correlation.

## Three independent design checks

1. The Atlas semantics review confirmed the peeling mechanics, letter
   encoding, reverse replay with reflection bookkeeping, the invariant and
   its ascending-order imaginarity reading, `long_orthogonalize`'s one-pass
   sufficiency, both matrix identities, and the no-canonicity claim. It
   corrected three anchors: the lexicographic-first claim holds only for
   Atlas's internal transducer numbering (so raw words are not
   differential-comparable), the A1 x A1 swap class has no Cayley letters
   at all (the one-Cayley anchor moved to the identity twist), and the B2
   short-pair anchor requires the `[[2,-1],[-2,2]]` orientation under this
   crate's Cartan convention. It also flagged upstream's silent
   positivity/orthogonality assumptions, now explicit checks.
2. The Rust internals review replaced the matrix-based descent and
   commutation tests with classification lookups (`delta(theta(alpha_s))`
   image chasing; Real descent = Cayley, Complex descent = cross),
   identified the missing `WeylAction::root_reflection` constructor, made
   the provenance gate a precondition of the termination argument, pinned
   the difference-sign convention, and supplied the error-variant family.
3. The consumer review changed the constructor input to `&InnerClass`
   (the distinguished involution is otherwise unreachable and the
   generator-index cross word otherwise unfounded), added the stored
   `cross_action` and decomposed `TwistedInvolution` so the label stage
   recomputes nothing, fixed naming to `CayleyCrossDecomposition::build`
   with `max_peeling_steps`, and enumerated the consequential doc updates
   including the stale progression paragraph.

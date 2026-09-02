# Ordinary-root and coroot design

## Approved scope

This stage adds the finite ordinary-root correspondence needed before Atlas
`m_alpha`, adjoint `m_alpha`, and grading construction can be represented in
Rust:

```text
RootId -> (ambient Weight alpha, ambient Coweight alpha_vee,
           original-simple-root coordinates of alpha)
```

It does not implement a grading, a real form, a Cartan class, or Atlas language
syntax. A completed structural implementation remains distinct from Atlas
compatibility until an HPC differential adapter executes the fixture against the
reference executable.

## Source boundary

Atlas keeps root and coroot information paired by ordinary-root number, then
materializes full-lattice roots and coroots in `RootDatum`. In particular,
non-simple coroots are constructed during root-system closure rather than
recovered from simple-root coordinates: `rootdata.cpp` builds positive roots
level by level and backfills each non-simple coroot by one dual reflection
along that root's first descent. Atlas does not re-check other descent paths;
the Rust closure propagates each pair along every generator and fails closed on
disagreement, which is semantically equivalent and strictly stronger.
`cartanclass.cpp` subsequently uses the ambient coroot of a subsystem-simple
imaginary root for `mAlphas`, and uses the pairing of full simple roots with
that coroot for `adjointMAlphas`.

The stored Cartan orientation is `C(i,j) = <alpha_i, alpha_j_vee>`: rows index
roots and columns index coroots, matching `PreRootDatum::Cartan_matrix` and
this crate's `from_simple_data` validation. The B2 fixture values below depend
on this orientation and are not transpose-invariant.

The important boundary is therefore an owned ordinary-root correspondence, not
a direct port of the C++ packed `RootSystem`, a grading shortcut, or a `rustcox`
dependency. `rustcox` has neither full coweight data nor Atlas's root-left,
coroot-right pairing convention.

This stage keeps the crate's deterministic lexicographic root order. Atlas
numbering is a function of height-ordered generation, mirrored negatives, and
the `prefer_co` flag, and grading bit positions later index enumeration-derived
lists. The deferral is sound for `m_alpha`, `adjointMAlphas`, and gradings only
if the future compatibility adapter permutes not just root IDs but every
enumeration-derived position: subsystem-simple-basis list order, grading bit
positions, and imaginary-root-set indices, with the `prefer_co` dependence
modeled explicitly.

## Data layout

`RootSystem` retains four parallel vectors, indexed by its existing stable
`RootId`:

```text
datum: BasedRootDatum
roots: Vec<Weight>
coroots: Vec<Coweight>
simple_coordinates: Vec<Vec<i32>>
```

The vectors are built together from one closure map and always have equal
length. Index alignment is a public invariant, not an internal convenience:
`id_of` binary-searches `roots`, so `roots` must stay ascending in
lexicographic coordinate order, and `RestrictedRootSystem::build` constructs
`RootId` values directly from the `roots()` enumeration index. The standard
library has no three-way `unzip`, so materialization consumes the closure's
hash map, sorts its `(root coordinates, record)` pairs lexicographically once,
then pushes into three `try_reserve_exact`-reserved vectors in lockstep; a
second pass would silently break alignment. Keeping
`roots` as a separate vector preserves the existing zero-copy `roots()` API
used by root involutions and restricted-root code. No `coroots()` slice
accessor is added; iteration goes through `entries`. The budget is never
stored in `RootSystem`: its fields remain exactly the four above, so derived
equality stays a statement about the mathematical value, not the resource
policy that produced it.

The public additions are:

```text
RootSystemBudget::new(max_lattice_rank, max_roots,
                      max_coordinate_entries, max_reflection_steps)
RootSystemBudget::complete_for(&BasedRootDatum, max_roots)
RootSystem::enumerate_with_budget(&BasedRootDatum, &RootSystemBudget)
RootSystem::coroot(RootId) -> Option<&Coweight>
RootSystem::entries() -> impl ExactSizeIterator<Item = (RootId, &Weight, &Coweight)>
RootSystem::bracket(root: RootId, coroot: RootId) -> Result<i32, StructureError>
BasedRootDatum::reflect_coweight(generator, &Coweight)
```

`bracket(root, coroot)` means `<root(root), coroot(coroot)>`, matching Atlas's
root-left, coroot-right argument order; the parameter names carry the
asymmetry that the ID types erase. The name `bracket` is kept deliberately
because it is the Atlas operation being modeled. Out-of-range IDs are
`IndexOutOfRange`, and the arithmetic is one call to the existing `pair`, so
overflow reporting has a single implementation. `coroot` mirrors `root`'s
`Option` return. `entries` is an `ExactSizeIterator` so consumers can size
fallible reservations; its first real consumer is `RootInvolutionData::new`.
`RootSystemBudget` lives in `root_system.rs` and is exported alongside
`RootId` and `RootSystem`. Its field names diverge from `IntegerLatticeBudget`
on purpose: `max_lattice_rank` keeps the crate's lattice/semisimple rank
distinction visible, and `max_coordinate_entries` counts coordinate values,
not allocator metadata. The `_with_budget` suffix is acceptable only because
the plain `enumerate` name is retained as the compatibility wrapper; it is not
a general crate convention.

The old `RootSystem::enumerate(&datum, max_roots)` remains a compatibility
wrapper with byte-identical observable behavior. It builds
`RootSystemBudget::complete_for(datum, max_roots)`:

```text
max_lattice_rank       = datum.lattice_rank()
max_roots              = max_roots
max_coordinate_entries = entry bound below, saturated to usize::MAX
max_reflection_steps   = max_roots * semisimple_rank, saturated
```

The three derived limits are non-binding by construction, so only root
cardinality can reject. The wrapper maps the budgeted cardinality error
`RootSystemResourceLimit { resource: "roots", limit }` back to the historic
`ResourceLimitExceeded { limit }` before returning; if a derived limit ever
fires anyway, the new named error surfaces, which is the desired fail-visible
behavior. A system with exactly `max_roots` roots still succeeds, and
`enumerate(&datum, usize::MAX)` still succeeds. `InnerClass::new` keeps its
`root_budget: usize` parameter and therefore keeps its rejection behavior.

## Closure and invariants

The closure stores a full root coordinate key and a record containing its
candidate coroot and simple-root coordinates. It seeds both signs of every
simple pair:

```text
(alpha_i, alpha_i_vee, e_i)
(-alpha_i, -alpha_i_vee, -e_i).
```

Seed negation applies checked negation to the root, the coroot, and the
simple coordinates; a datum built through `from_simple_data` may legally hold
`i32::MIN` coroot coordinates, so the coroot negation is a reachable overflow.
For every pending pair and simple generator `i`, it computes both dual
reflections with checked fixed-width arithmetic:

```text
s_i(alpha)     = alpha     - <alpha, alpha_i_vee> alpha_i
s_i(alpha_vee) = alpha_vee - <alpha_i, alpha_vee> alpha_i_vee.
```

The full root coordinate is the sole closure key. A duplicate key must have
exactly the same coroot and simple-root coordinates; otherwise enumeration
fails with `RootSystemInvariantViolation { invariant: "coroot agreement" }`
instead of silently accepting the first path. Every candidate record also
checks `<alpha, alpha_vee> = 2` and reports
`RootSystemInvariantViolation { invariant: "self pairing" }` on failure.
These checks make braid-path disagreement fail closed and rule out the invalid
shortcut of deriving a coroot from a root's simple coordinates. They are
defensive: `from_simple_data` validates every simple pairing against a finite
Cartan matrix, so no public constructor is known to reach either failure. The
test for the duplicate-key conflict therefore injects the conflicting record
directly through the private closure state, which production
`enumerate_with_budget` also uses; there is no test-only production code path.

`RootInvolutionData::new` additionally checks that the stored coweight action
maps the stored coroot of each root to the stored coroot of its root image,
failing with `InvalidRootDatumAutomorphism`. This is the missing root-datum
automorphism condition: `LatticeInvolution` proves only pairing preservation
and the existing root loop proves only that roots map to roots, so a datum
with a central torus admits actions that pass both while moving a coroot off
its central coordinates. The check runs inside the existing per-root loop,
immediately after the image ID is resolved, via the new `entries` iterator.
`RootSystem::action_permutation` deliberately does not gain this check: every
`WeylAction` is a word in simple reflections whose coweight generator is the
same dual-reflection formula the closure uses, so the closure's duplicate-key
check already makes transport a theorem for all constructible actions; the
regression guard is a unit test over the enumerated Weyl group, not a second
check in the one hot loop. One recorded behavior change: the inner-class test
that shifts a simple coroot into the central torus now fails earlier, inside
`RootInvolutionData::new`, with `InvalidRootDatumAutomorphism` instead of
`InvalidBasedAutomorphism`.

## Resource and overflow policy

Root-system closure receives a caller-owned `RootSystemBudget` with four
limits:

```text
max_lattice_rank
max_roots
max_coordinate_entries
max_reflection_steps
```

These are limits for one computation, not a mathematical rank cap. In
particular, a product of 33 A1 factors must work when supplied a budget that
covers its actual storage and work.

Let `n` be the full lattice rank, `r` the semisimple rank, and `R` the
requested root limit, with `R_eff = 0` when `r = 0` (a pure torus allocates no
records) and `R_eff = R` otherwise. Live coordinate entries are bounded,
following the `integer_lattice` convention of counting the caller's retained
input, by

```text
borrowed caller datum:           r^2 + 2 r n
owned datum snapshot:            r^2 + 2 r n
map plus pending queue:          R_eff (3 n + r)
scratch buffers:                 3 n + 2 r
one in-flight candidate record:  3 n + r
total:                           2 (r^2 + 2 r n) + (R_eff + 1)(3 n + r) + (3 n + 2 r).
```

A map entry retains a root key, coroot, and simple coordinates (`2n + r`); a
pending queue entry retains only its root key (`n`). The scratch term covers
the reusable coroot/simple-coordinate copies plus reflected root, coroot, and
simple-coordinate buffers. The in-flight term covers the candidate key,
record, and pending root copied while a new reflection is inserted. At
materialization the pending queue is empty; sorting moves the map's coordinate
buffers into the three output vectors, with only tuple metadata retained by
the temporary sorted vector. The entry bound intentionally counts coordinate
values rather than allocator metadata; `max_roots` separately bounds hash-map
slots and vector-object counts.

One reflection step is one `(pending record, simple generator)` visit, that
is, one dual reflection pair; the seed phase performs `2r` insertions and zero
steps. The worst case is exactly `R_eff * r` steps.

Construction checks the budget in a fixed order before any allocation:
lattice rank, then roots (for `r >= 1`, the `2r` seeds must fit `max_roots`),
then coordinate entries, then reflection steps. The entry and step bounds are
evaluated in `u128`, saturated to `usize::MAX`, and compared against their
limits, so an unrepresentable worst case is rejected by every finite limit and
accepted only by an explicit `usize::MAX` limit; no size calculation can
itself report `ArithmeticOverflow`. Once these consistency checks pass, live
entries and executed steps are monotone functions of the accepted root count,
so the closure needs exactly one runtime check: the insert-time cardinality
check, `RootSystemResourceLimit { resource: "roots", limit }`. The other
labels are `"lattice rank"`, `"coordinate entries"`, and
`"reflection steps"`. The two new `StructureError` variants are

```text
RootSystemResourceLimit { resource: &'static str, limit: usize }
RootSystemInvariantViolation { invariant: &'static str }
```

with `usize` limits (the `u64` of `IntegerLatticeResourceLimit` forces a
fallible narrowing that this layer does not need), plus
`InvalidRootDatumAutomorphism` for the coroot-transport failure, whose message
must not claim a pairing violation, since pairing preservation is exactly what
`LatticeInvolution` already guarantees.

Every coordinate vector — the datum snapshot, each reflected root and coroot,
each simple-coordinate vector, the map key copy, and the three output vectors
— is built with `Vec::try_reserve_exact` and maps failure to
`AllocationFailed`. The pending queue calls `VecDeque::try_reserve(1)` before
each `push_back`; its entries contain only root coordinates. Hash-map bucket
allocation is fallible through the map's reserve path and remains bounded a
priori by `max_roots`; hash metadata is outside the coordinate-entry budget.
The snapshot
uses an internal `BasedRootDatum::try_clone` that copies the already-validated
fields directly instead of re-running `from_simple_data` validation.
`reflect_coweight` mirrors `reflect_weight`'s signature and errors, and both
use the fallible-reservation shape so the dual pair is symmetric.

Arithmetic uses `i128` products and sums, checked subtraction, checked
negation, and checked narrowing to `i32`. The `i128` intermediates cannot
overflow at these dimensions; the reachable failures are the final `i32`
narrowing and `checked_neg` on `i32::MIN`, and tests target those. Thus an
impossible pairing, reflection coordinate, negation, or budget shape returns
`StructureError::ArithmeticOverflow` or a named root-system resource error; it
cannot wrap as the C++ fixed-width paths can. One public behavior change is
accepted and recorded: a datum whose coroot reflections overflow `i32` while
its root reflections do not previously enumerated and now reports
`ArithmeticOverflow`.

## Tests and fixture gate

The implementation must add tests before it is called structurally complete:

- A2 preserves the current deterministic root order and gives paired coroots.
- B2 with Cartan matrix `[[2,-2],[-1,2]]` proves non-simply-laced behavior:
  the roots `(1,1)` and `(1,2)` have coroots `(2,0)` and `(0,1)`.
- An A1 root in a rank-two lattice with coroot `(2,1)` preserves the nonzero
  central coweight coordinate for both signs; a pure torus has no pairs.
- Every stored pair has self-bracket two; the enumerated A2 Weyl actions
  transport roots and coroots together through `action_permutation` and
  `act_on_coweight`; a duplicate conflict injected through the private closure
  state is rejected with the `"coroot agreement"` invariant error.
- Independent lattice-rank, root-count, coordinate-entry, and reflection-step
  budgets reject with their named resource errors before closure work starts;
  the root-count limit also rejects at runtime when discovery exceeds it;
  checked coweight reflection rejects an overflowing narrowed coordinate, and
  a seed coroot holding `i32::MIN` rejects at negation.
- The wrapper stays byte-identical: `enumerate(&datum, usize::MAX)` succeeds,
  an undersized wrapper budget still returns
  `ResourceLimitExceeded { limit }`, and `enumerate_with_budget` under
  `complete_for` returns the same system as `enumerate`.
- A caller-budgeted product of 33 A1 factors succeeds, demonstrating that no
  Atlas `RANK_MAX = 32` boundary entered this layer.

`tests/fixtures/domain/root_coroot.atlas` is reserved now for the later
positive and negative Atlas/Rust differential corpus. It is declared but not
executable because the language-level constructor and event adapters do not
yet exist.

## Three independent design checks

The mandated fresh-context reviews ran before implementation and their
corrections are folded into the sections above.

1. The Atlas semantics review confirmed all four source claims against
   `rootdata.cpp`, `cartanclass.cpp`, `involutions.cpp`, and
   `prerootdata.cpp`: coroots are born in closure via dual reflections along a
   first-descent path, `bracket` is root-left/coroot-right, `mAlphas` uses the
   subsystem-simple imaginary coroot while `adjointMAlphas` pairs full simple
   roots against it, and storage is paired by root number with full-lattice
   materialization in `RootDatum`. It added the explicit Cartan orientation
   statement and the enumeration-order conditions the future adapter must
   absorb.
2. The Rust resource review corrected the entry bound (borrowed caller datum,
   the root-only pending queue, reusable scratch buffers, and a pure torus
   need explicit accounting), fixed the check order and `u128` saturation rule, established that the
   upfront consistency checks make runtime entry/step meters unreachable,
   named the two new error variants with `usize` limits, replaced the vague
   fallible-allocation clause with the exact per-container statement including
   the hash-map metadata carve-out, required `try_clone` instead of re-validation,
   and rejected storing the budget inside `RootSystem` because derived
   equality would become resource-policy-dependent.
3. The consumer review inventoried every call site (one production caller of
   `enumerate`, in `InnerClass::new`; all others are module tests), required
   the byte-identical wrapper contract with saturated derived limits and the
   cardinality error mapping, and moved the coroot-transport check out of
   `action_permutation` (redundant for every constructible `WeylAction`, and
   in the sole hot loop) into `RootInvolutionData::new`, where a
   central-torus counterexample shows it is load-bearing. It pinned the
   public signatures, the export list, and the doc comments that must change,
   and recorded the one existing test whose expected error changes.

## Structural preflight evidence

XMU HPC job `3464542` completed on 2026-07-27 using Rust 1.96. It passed the
package format check, Clippy with warnings denied, and 93 structural unit
tests, including the paired A2/B2 closures, the central-torus and pure-torus
cases, the injected invariant conflicts, each named budget rejection, the
byte-identical wrapper cases, and the dynamic product of 33 A1 factors. The
job ran from frozen source snapshot
`f951905d998b0d500b7963e653de15e7887f06fe558e29b615cd717bffb2e218`; its JSON
report checksum is
`5894bb2c50ec53b6f504f6b45d3cde5fe675c82a2361fdcbccadf9e6efe795fc`.

This is Rust structural evidence only. The report lists the root-coroot
fixture as declared and unexecuted, and no reference Atlas constructor or
event adapter ran in this job.

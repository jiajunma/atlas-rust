# Involution table design (KGB stage b)

## Approved scope

Stage (b) of the KGB map: the table of twisted involutions the KGB
generation walks — per-involution theta, root classification, mod-2
dedup subspace, involution and Weyl lengths, and cross-action
propagation, generated per Cartan class as contiguous orbit slices. It
marries the stage-(a) word-level `WeylElement` with the crate's existing
matrix-level `TwistedInvolution` under one numbering. Stages (c)-(f)
(torus parts, seed, generation, descents) consume it.

## Atlas construction (oracle trace)

Citations are to `~/mycodes/atlasofliegroups`, master `4d3e9449`.

`InvolutionTable` (involutions.h:89-254) stores one `record` per twisted
involution (involutions.h:100-121): `InvolutionData id` (root
permutation, imaginary/real/complex bitmaps over ALL roots, simple bases
of the imaginary and real subsystems — involutions.h:46-85), the theta
matrix on X^*, `M_real`/`lift_mat` (an image basis of `1-theta` and the
coordinate map onto it), `th1_rho = (1+theta)rho`, the
`SmallSubspace mod_space`, and two lengths. The precise `M_real` recipe,
recorded for the future on-demand derivation: `lift_mat` = the echelon
image basis `A` of `1-theta`, `M_real` = the first `r` rows of
`col^{-1}` (involutions.cpp:206-207), with invariants
`lift_mat.M_real = 1-theta` and — forced by the columns lying in
`ker(theta+1)` — `M_real.lift_mat = 2.I_r`, a sharp free test oracle.
Exactly two mutators:

- `add_involution` (involutions.cpp:186-225) seeds an orbit: theta is
  built by walking `involution_expr` (Cayley letters as root
  reflections, cross letters conjugating), `column_echelon(1-theta)`
  yields `lift_mat`/`M_real`, `mod_space = tits::fiber_denom(theta)`,
  and `W_length = l(tw)`, `length = (W_length + #Cayley)/2`
  (involutions.cpp:214-215).
- `add_cross(s,n)` (involutions.cpp:228-258) propagates: the record is
  COPIED from the neighbor and edited — classification renumbered by
  the simple-root permutation, `theta <- s.theta.s`, `M_real`/
  `lift_mat` transported one reflection, `th1_rho` updated through the
  new root involution, `mod_space.apply` (through the constructor-built
  `torus_simple_reflection` mod-2 matrices, involutions.h:124,
  involutions.cpp:174-183) then re-normalized, and the length
  bookkeeping `length += d/2; W_length += d` where `d` is
  `twistedConjugate`'s change in `{0, +-2}` (involutions.cpp:251-252;
  the set is exact — one left and one right +-1 multiplication,
  weyl.h:538-548).

`Cartan_orbit` (involutions.cpp:362-379) BFSes one Cartan class from
`InnerClass::canonicalize`'s canonical involution, iterating generators
in EXTERNAL order, asserting the orbit fills exactly `orbitSize()` (the
a-priori Weyl-order quotient `|W| / (|W_im|.|W_re|.|W_cx|)`,
cartanclass.cpp:1046-1064); each orbit is a contiguous pool slice,
located by binary search on starts (involutions.cpp:401-417).
`Cartan_orbits::add` is idempotent (involutions.cpp:388-398), and the
table is SHARED across real forms of one inner class — KGB generates
only the form's Cartan set on demand (kgb.cpp:509-512). Hash keys are
transducer piece arrays (weyl.cpp:1542-1548); the sort comparer is
(length, W_length, raw piece compare) (involutions.cpp:419-428). Dedup
of KGB elements reduces torus bits to the canonical coset
representative modulo `mod_space` (involutions.cpp:430-434,
subquotient.cpp:103-137).

Upstream nuances the port records:

- `fiber_denom` (tits.cpp:833-838) computes the mod-2 reduction of the
  SATURATED integral `-1`-eigenlattice of theta-transpose (the X_*
  action); the tits.h:390 comment says "image of theta-1" — a generally
  strictly smaller subspace (the gap is the 2-torsion of the quotient).
  The port follows the CODE, which is the right equivalence both
  mathematically (T-conjugacy moves torus parts by the order-2 elements
  of `(1-theta)T`) and operationally (every dedup site uses it and
  sizes are validated against `kgb_size`).
- The Tits-based KGB main loop (kgb.cpp:553-614) decides statuses
  purely by Weyl-group tests and the grading; the KGB Cayley machinery
  itself also never consults the classification — simple Cayley is a
  bare `sigma_mult` (tits.h:717-719) and inverse Cayley uses the
  grading plus MOD-SPACE BASIS VECTORS (tits.cpp:627-644 scans
  `mod_space.basis(j)` for a vector pairing to 1 and left-adds it).
  The classification data serves the deferred `global_KGB`, in-loop
  asserts, and the post-KGB parameter/block layers. Two consequences:
  stage (e) must not over-depend on the classification, and the
  `ModTwoSubspace` this table exposes MUST support ordered basis
  enumeration for stage (c)'s inverse-Cayley repair.
- Upstream's `mod_space` is NOT path-dependent despite the transport:
  `Subspace::apply` renormalizes, and the reflected eigenlattice IS the
  new involution's eigenlattice, so transported-and-normalized equals
  fresh `fiber_denom` exactly. `M_real`/`lift_mat` by contrast ARE
  path-dependent (echelon only at the seed, a reflected non-echelon
  transport elsewhere); only the two product invariants are stable.

## Port decisions and divergences

1. ONE canonical entry path. This port derives EVERY record field
   canonically from theta at entry: classification, mod-space, and
   `th1_rho` are computed fresh, so no field is path-dependent at all.
   Torus-part COORDINATES are adapter-deferred observables in both
   implementations (upstream's are path-transported-basis-dependent,
   echelon only at the seed), so canonicalizing is observable-neutral.
   The transported-record optimization (whose full upstream ingredient
   list includes the `torus_simple_reflection` cache) is a documented
   deferral, to be taken up only if the HPC preflight shows the fresh
   path binding.
2. Theta itself is canonical given the involution, so `theta' =
   s.theta.s` along a BFS edge is an evaluation strategy, not a
   transport: the port uses it, maintaining the matrix-level
   `WeylAction` by composing `action(s) . action(w) . action(twist(s))`
   on each ACCEPTED entry (two `rank^3` products plus a datum clone per
   `compose`; misses cost only the word-level probe).
3. Reuse over reimplementation: the per-involution matrix bundle is the
   existing `TwistedInvolution` (theta as validated action plus
   `RootInvolutionData` classification; its constructor takes the
   composed `WeylAction` by value); the mod-space reuses
   `negative_coweight_eigenspace` + `reduce_basis_mod_two` into a
   `ModTwoSubspace` — exactly the composition `cartan_fiber.rs` already
   trusts as the fiber denominator — with coset reduction by
   `quotient_representative`. Stage (b) calls the two `pub(crate)`
   functions directly rather than building a `CartanFiber` per
   involution: the fiber also computes a numerator this stage does not
   need, and dedup must accept arbitrary torus bit patterns, which
   `quotient_representative` does while the subquotient's
   `canonical_representative` gates membership — stage (e) must NOT
   reach for the subquotient.
4. `M_real`/`lift_mat` are DROPPED from stage (b) entirely. The full
   upstream consumer trace shows every external consumer lives in the
   parameter/block layer (`repr.cpp`/`K_repr.cpp` y-packing and
   lambda/gamma-lambda normalization; K-type equality is comparison of
   those normal forms); NO KGB stage (c)-(f) machinery consumes them —
   `x_pack`/`x_equiv` recompute their own saturation fresh, inverse
   Cayley uses mod-space basis vectors, `x0` seeding uses gradings and
   the central fiber. The deferral is revisited when the PARAMETER
   layer is designed, not at stage (c)/(d), and the derivation-from-
   theta recipe plus its two product invariants are recorded above.
   `complex_is_descent` (involutions.h:185) joins the same deferral.
5. Numbering: Cartan classes in the caller's add order, each orbit a
   contiguous slice, BFS with generators in external order 0..rank.
   `add_cartan` is IDEMPOTENT (re-adding returns the existing slice,
   upstream parity), and the documented determinism discipline is that
   callers add Cartan classes in ascending `CartanId` order — stage (e)
   adds the form's `cartan_set(form)`, which the classification already
   returns ascending — making numbering reproducible regardless of
   which real form is generated first. The seed divergence (the
   crate's `TwistedConjugacyClass` representative instead of
   `canonicalize`'s dominance-normalized one) affects insertion ORDER
   only: closure under all simple twisted conjugations from any class
   member generates the whole class, and every record field is
   canonical-from-theta. The sort comparer for later stages is
   (involution length, Weyl length, `WeylElement` derived `Ord`), as
   stage (a) recorded.
6. Length bookkeeping stays the cheap BFS recurrence (`W_length` from
   the element's cached length — free at the word level — and
   `length += d/2` with `d` from cached-length subtraction). The seed
   uses `(W_length + #Cayley)/2` with the Cayley count from
   `CayleyCrossDecomposition` — once per Cartan class and in the
   small-rank tests ONLY: its build constructs a fresh
   `TwistedInvolution` per peeled letter, so it is forbidden as a
   per-entry tool at scale. The recurrence is intrinsic from any seed
   because `#Cayley = 2.length - W_length` is invariant along cross
   edges within a class.
7. The BFS edge is implemented DIRECTLY in the table as
   `s_elem.multiply(w).multiply(twist_s_elem)` over `rank`
   simple-reflection `WeylElement`s precomputed once at construction —
   NOT via `WeylElement::twisted_conjugate`, which builds both
   reflections fresh per call through `from_action` (an O(roots x
   rank^2) hidden term that would dominate the whole BFS by an order
   of magnitude at E8). The twist array is validated once at
   construction, not per edge.

## Data layout and public boundary

```text
// involution_table.rs
InvolutionId(pub(crate) usize)   // full ID derive set incl. Ord (stage (e) sorts these)
InvolutionTableBudget {           // const new
    max_involutions: usize,
    integer_lattice: IntegerLatticeBudget,   // threads to the eigenlattice reduction
}
InvolutionRecord {                // derives Clone, Debug, Eq, PartialEq
    weyl_element()       -> &WeylElement
    twisted_involution() -> &TwistedInvolution
    theta()              -> &LatticeInvolution   // shallow convenience
    mod_space()          -> &ModTwoSubspace      // ordered basis_vectors for stage (c)
    involution_length()  -> usize
    weyl_length()        -> usize
    theta_plus_one_rho() -> &Weight
}
InvolutionTable::new(&InnerClass, InvolutionTableBudget) -> Result<...>
    // stores a clone (the inner class owns datum, root system, and
    // distinguished involution together, so no cross-gate is needed);
    // derives once: the validated simple twist, the rank
    // simple-reflection WeylElements, and 2rho from the positivity slice
add_cartan(&mut self, &CartanClassification, CartanId)
    -> Result<(InvolutionId, usize), ...>        // idempotent; typed start
lookup(&WeylElement) -> Option<InvolutionId>
record(InvolutionId) -> Option<&InvolutionRecord>
cross(generator, InvolutionId) -> Result<InvolutionId, ...>   // O(1) stored link
cayley(generator, InvolutionId) -> Result<Option<InvolutionId>, ...>
    // left-multiply by the reflection, look up; None = target Cartan
    // not yet added (the stage-(e) contract adds the form's
    // upward-closed Cartan set first, after which None is the
    // caller's invariant violation)
simple_root_kind(InvolutionId, generator) -> Option<RootKind>
    // one accessor covering upstream's is_{complex,imaginary,real}_simple
root_system() -> &RootSystem / inner_class() -> &InnerClass
cartan_of(InvolutionId) -> Option<CartanId>      // binary search on starts
involution_count() / orbit_slice(CartanId) -> Option<(InvolutionId, &[InvolutionRecord])>
```

Constructor shape follows the crate: no public borrows, so `new` stores
CLONES of the inner class and root system (the `TwistedConjugacyPartition`
precedent), gating that the system was enumerated from the inner class's
datum. `new` rather than `build` because the table starts empty and is
filled by `add_cartan` — the `build` verb in this crate means "consume
other layers and return finished". `add_cartan` takes the classification
by reference PER CALL (not stored), fetching `cartan_class(id)` itself so
the seed and the expected count can never be a mismatched pair; the seed
representative is matrix-level, converted once per class via
`WeylElement::from_action`. Provenance gates mirror
`TwistedConjugacyPartition::class_of` (datum equality plus the
distinguished factorization).

Storage shape (memory-honest): records own the `WeylElement`; the dedup
map is `BTreeMap<Vec<RootId>, InvolutionId>` keyed by a clone of the
FORWARD PERMUTATION only — sound because stage (a) pinned that derived
equality agrees with permutation-only equality, halving key memory and
avoiding hashing the redundant inverse; `lookup` stays zero-copy via the
slice borrow. `BTreeMap` (not the crate's first-ever `HashMap`) follows
the `TwistedConjugacyPartition` permutation-key precedent and keeps
iteration deterministic, though the map is lookup-only; determinism
rides on the records Vec. During the BFS the table also fills a
rank-by-count CROSS-LINK table, making `cross` an O(1) infallible read
after build and moving the orbit-closure check into `add_cartan`, where
a failure surfaces at build time
(`InvolutionTableInvariantViolation { invariant: "orbit closure" }`).

Honest scale figures (split E8, ~2.0e5 involutions, 240 roots, rank 8):
element ~3.9 KB, `TwistedInvolution` bundle ~11 KB (it carries two
`BasedRootDatum` clones — a compaction target if E8 memory ever binds),
permutation key ~1.9 KB, cross links ~13 MB: roughly 13-15 KB per
involution, ~2.6-3 GB total — NOT comparable to upstream's ~3 KB
records; acceptable on HPC nodes, conditional (below) in any case.
Entry-path wall clock at E8 is on the order of MINUTES single-threaded,
dominated not by the classification loop but by
`subsystem_simple_roots` inside `RootInvolutionData` (O(P^2 x rank x
log P), P the larger positive-root count of the imaginary/real
subsystem, with heavy allocation churn) — the specific quantity the HPC
preflight must watch before taking up the transported-record deferral.

E7/E8 DEPENDENCY, stated out loud: seeds, per-class expected counts,
and the `max_involutions` derivation all flow from
`CartanClassification`, whose twisted-conjugacy partition today
enumerates the FULL Weyl group — infeasible at |W(E8)| ~ 7e8. At small
and medium rank everything above stands as written; at E7/E8 stage (b)
is blocked behind task #9 (on-demand class generation plus a ported
order-quotient `orbitSize`), and the scale analysis above is
conditional on that dependency.

## Resource and arithmetic policy

Per-entry work is O(roots x rank^2 + P^2 x rank x log P) — the
simple-basis extraction, not the classification loop, binds at E8.
Total work and storage are bounded by `max_involutions`; reservations
use the shared `try_capacity` with the a-priori orbit size. Length
arithmetic is checked; `d/2` uses exact integer division after checking
`d` is in `{0, +-2}` (`invariant: "twisted length step"` — unreachable
on a correct substrate, kept as the crate's standard defensive gate);
the seed formula checks `W_length + #Cayley` is even
(`invariant: "length parity"`). `th1_rho` is constructed as: `2rho` =
the sum of positive roots read off the `RootSystem` positivity slice,
apply theta, add, halve with an exactness check — all under checked
arithmetic (the crate has no rho helper; this is its first
construction). Error families: `InvolutionTableResourceLimit
{ resource, limit }` and `InvolutionTableInvariantViolation
{ invariant }` with the standard display shapes.

## Tests and fixture gate

- A1 split: two Cartan classes, orbits of size one each; the table
  agrees with the classification, `cross` fixes both involutions, and
  `add_cartan` re-added returns the same slice (idempotence).
- A2 with the swap twist and B2 split: per-Cartan orbit sizes match the
  classification's twisted-involution counts; total equals
  `twisted_involution_count`; ascending-add numbering is reproducible.
- Record canonicality: for every entry at small rank, the
  BFS-propagated theta, lengths, and mod-space agree with a fresh
  `TwistedInvolution`/`CayleyCrossDecomposition` computation from the
  element alone — the recurrence-versus-formula cross-check
  (`length == (W_length + #cayley_roots)/2`), plus `theta_plus_one_rho`
  against a direct `(1+theta)rho` evaluation.
- Mod-space: for each involution of SL(2,R) and Sp(4,R), the reduction
  of a nonzero torus bit pattern matches the hand-computed coset
  representative; quotient dimensions match the fiber layer where both
  exist; `basis_vectors` enumeration is exercised (the stage-(c)
  consumer shape).
- Cayley edge: on B2, `cayley` from the fundamental involution reaches
  the next Cartan when added and returns `None` before it is added.
- Budget and invariants: an under-budgeted build is the named resource
  rejection; a foreign-system element is rejected by the expressible
  provenance gate; `simple_root_kind` agrees with the record's
  classification.

`tests/fixtures/domain/involution_table.atlas` is reserved; the
differential observables (per-Cartan orbit sizes, length profiles)
reach the language layer through KGB, per the stage map.

## Consequential updates

Landing this stage must update: `lib.rs` (module and exports);
`error.rs` (the two variant families); `KGB_STAGE_MAP.md` (stage (b)
landed); `REAL_GROUP_DESIGN.md`'s progression (next: stage (c), Tits
operations); and task #9's note, SOFTENED per review: the BFS machinery
here is REUSABLE for task #9, but the replacement of the full-W
enumeration needs its own design round (seed discovery without full-W,
an independent order-quotient orbit-size formula, and the
classification's remaining `class_of` dependencies).

## Three independent design checks (returned; corrections folded)

1. Atlas semantics — no BLOCKING/HIGH. VERIFIED: record inventory, both
   entry paths, length recurrence exactness (d in {0,+-2} exact),
   fiber_denom code-over-comment, seed-divergence-is-order-only, all
   citations. RULED: dropping `M_real`/`lift_mat` is safe through stage
   (f) — the consumer set is exactly the parameter layer. CORRECTED:
   the deferral's revisit point (parameter layer, not stage (c)/(d));
   the classification-consumer sentence (KGB Cayley machinery uses only
   gradings + mod-space); ADDED: the mod-space ordered-basis
   requirement for inverse Cayley; the `M_real.lift_mat = 2.I_r`
   invariant; the `torus_simple_reflection` ingredient; upstream
   mod_space path-independence.
2. Rust internals — CORRECTED: per-entry complexity (the
   `subsystem_simple_roots` term binds, not the classification loop);
   the BFS edge must use precomputed reflection elements (the
   `twisted_conjugate` API rebuilds them per call); the honest memory
   figure (~13-15 KB per involution, not 4 KB) and the
   forward-permutation-only map key; the budget must nest
   `IntegerLatticeBudget`; the E7/E8 task-#9 dependency stated in the
   scale paragraph; the seed's `from_action` conversion named;
   `CayleyCrossDecomposition` forbidden per entry at scale; the parity
   invariant and the `th1_rho` construction specified (no rho helper
   exists in the crate).
3. API and consumer fit — CORRECTED: concrete `new(&InnerClass,
   &RootSystem, budget)` owning-clone signature;
   `add_cartan(&CartanClassification, CartanId)` self-consistent
   seeding; idempotence and ascending-add determinism; the missing
   `cayley` edge with its None-before-added semantics;
   `simple_root_kind` covering the three upstream simple tests;
   `InvolutionNbr` renamed `InvolutionId`; record accessors renamed
   `weyl_element()`/`twisted_involution()` (the bare `involution()`
   collides with crate vocabulary); typed `(InvolutionId, usize)`
   returns; `BTreeMap` choice recorded; the task-#9 note softened to
   "reusable machinery, separate design round".

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
coordinate map onto it, invariant `lift_mat*M_real == 1-theta`),
`th1_rho = (1+theta)rho`, the `SmallSubspace mod_space`, and two
lengths. Exactly two mutators:

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
  new root involution, `mod_space.apply` then re-normalized, and the
  length bookkeeping `length += d/2; W_length += d` where `d` is
  `twistedConjugate`'s change in `{0, +-2}` (involutions.cpp:251-252).

`Cartan_orbit` (involutions.cpp:362-379) BFSes one Cartan class from
`InnerClass::canonicalize`'s canonical involution, iterating generators
in EXTERNAL order, asserting the orbit fills exactly `orbitSize()` (the
a-priori Weyl-order quotient, cartanclass.cpp:1046-1064); each orbit is
a contiguous pool slice, located by binary search on starts
(involutions.cpp:401-417). Hash keys are transducer piece arrays
(weyl.cpp:1542-1548); the sort comparer is (length, W_length, raw piece
compare) (involutions.cpp:419-428). Dedup of KGB elements reduces torus
bits to the canonical coset representative modulo `mod_space`
(involutions.cpp:430-434, subquotient.cpp:103-137).

Two upstream nuances the port records:

- `fiber_denom` (tits.cpp:833-838) computes the mod-2 reduction of the
  INTEGRAL `-1`-eigenlattice of theta-transpose (the X_* action); the
  tits.h:390 comment says "image of theta-1" instead — the port follows
  the CODE.
- The Tits-based KGB main loop (kgb.cpp:553-614) does NOT consult the
  root classification for statuses — complex/real/imaginary are decided
  by Weyl-group tests and the grading; the classification data serves
  `global_KGB`, the Cayley machinery, and cross-checks. Stage (e) must
  not over-depend on it.

## Port decisions and divergences

1. ONE canonical entry path. Upstream's `add_cross` transports record
   fields for 1990s-era economy, leaving `M_real`/`lift_mat`
   PATH-DEPENDENT (only `lift_mat*M_real == 1-theta` is invariant; the
   stored basis depends on which BFS edge won). This port derives EVERY
   record field canonically from theta: classification, mod-space,
   `th1_rho`, and the `1-theta` image data are computed fresh at each
   entry, so no field is path-dependent at all. Torus-part COORDINATES
   are adapter-deferred observables in both implementations
   (echelon-basis-dependent upstream too), so canonicalizing is
   observable-neutral. The transported-record optimization is a
   documented deferral, to be taken up only if the HPC preflight shows
   the fresh path binding at E7/E8 scale.
2. Theta itself is canonical given the involution, so `theta' =
   s.theta.s` along a BFS edge is an evaluation strategy, not a
   transport: the port uses it, and maintains the matrix-level
   `WeylAction` by composing `action(s) . action(w) . action(twist(s))`
   along the edge — public API, O(rank^3) per edge.
3. Reuse over reimplementation: the per-involution matrix bundle is the
   existing `TwistedInvolution` (theta as validated action plus
   `RootInvolutionData` classification); the mod-space reuses
   `negative_coweight_eigenspace` + `reduce_basis_mod_two` into a
   `ModTwoSubspace` (the same machinery the Cartan fiber layer already
   trusts), with coset reduction by the existing
   `quotient_representative`.
4. Numbering: Cartan classes in the caller's add order, each orbit a
   contiguous slice, BFS with generators in external order 0..rank —
   fully documented crate order. It diverges from upstream only through
   the seed representative (the crate's `TwistedConjugacyClass`
   representative, not `canonicalize`'s dominance-normalized one) and
   the absence of the internal-order seed walk; since every record
   field is canonical-from-theta, the divergence affects insertion
   ORDER only, which the adapter deferral already covers. The sort
   comparer for later stages is (involution length, Weyl length,
   `WeylElement` derived `Ord`), as stage (a) recorded.
5. Length bookkeeping stays the cheap BFS recurrence (`W_length` from
   the element's cached length — free at the word level — and
   `length += d/2` with `d` from cached-length subtraction, exactly the
   signal stage (a) pinned); the seed uses the `(W_length + #Cayley)/2`
   formula with the Cayley count from `CayleyCrossDecomposition`, and
   the tests cross-check the recurrence against that decomposition on
   every table entry at small rank.

## Data layout and public boundary

```text
// involution_table.rs
InvolutionNbr(pub(crate) usize)          // newtype, crate ID idiom
InvolutionTableBudget { max_involutions } // const new
InvolutionRecord {                        // accessors, no public fields
    element()         -> &WeylElement     // the twisted involution, word level
    involution()      -> &TwistedInvolution  // theta action + classification
    mod_space()       -> &ModTwoSubspace  // X_* mod 2 dedup subspace
    involution_length() -> usize
    weyl_length()     -> usize
    theta_plus_one_rho() -> &Weight
}
InvolutionTable::new(<inner-class context>, budget) -> Result<...>
add_cartan(CartanId, <seed context>) -> Result<(start, size)>
lookup(&WeylElement) -> Option<InvolutionNbr>
record(InvolutionNbr) -> Option<&InvolutionRecord>
cross(generator, InvolutionNbr) -> Result<InvolutionNbr>
cartan_of(InvolutionNbr) -> Option<CartanId>   // binary search on starts
involution_count() / orbit_slice(CartanId)
```

The table owns its records and a `HashMap<WeylElement, InvolutionNbr>`
(the stage-(a) derives exist for exactly this). Provenance follows the
crate idiom: construction takes the inner-class context once, and
per-operation gates are what the substrate can express (the stage-(a)
root-count gate plus datum equality where a datum is comparable). The
`1-theta` image data (`M_real`/`lift_mat` equivalents) is NOT stored in
stage (b): its only consumer is the dual-side packing of later stages,
it is derivable on demand from theta, and deferring it keeps this
record free of basis-choice content entirely; the deferral is recorded
here and revisited when stage (c)/(d) name their need.

Memory at scale is bounded by the involution count as the stage-(a)
review established: split E8 is about 2.0e5 involutions at roughly 4 KB
each (element + inverse dominate) — comparable to upstream's ~3 KB
records — with the element's `inverse` droppable later if compaction is
ever needed.

`cross` is the KGB-facing edge: `twisted_conjugate` the element, look
up the result, and return the neighbor's number; it must ALWAYS hit
after an orbit is complete, checked as
`InvolutionTableInvariantViolation { invariant: "orbit closure" }`.
`add_cartan` asserts the generated orbit size equals the
classification's per-class count — a hard invariant, mirroring
upstream's assert — and errors `InvolutionTableResourceLimit { resource:
"involutions", limit }` when the budget binds. Reservations use the
shared `try_capacity` with the a-priori orbit size.

## Resource and arithmetic policy

Per-entry work is O(roots x rank^2) (fresh classification dominates);
total work and storage are bounded by `max_involutions`, which the
caller derives from the classification's counts. Length arithmetic is
checked; `d/2` uses exact integer division after checking `d` is in
`{0, +-2}` (`invariant: "twisted length step"`). Error families:
`InvolutionTableResourceLimit { resource, limit }` and
`InvolutionTableInvariantViolation { invariant }` with the standard
display shapes.

## Tests and fixture gate

- A1 split: two Cartan classes, orbits of size one each; the table
  agrees with the classification and `cross` fixes both involutions.
- A2 with the swap twist and B2 split: per-Cartan orbit sizes match the
  classification's twisted-involution counts; total equals
  `twisted_involution_count`.
- Record canonicality: for every entry at small rank, the
  BFS-propagated theta, lengths, and mod-space agree with a fresh
  `TwistedInvolution`/`CayleyCrossDecomposition` computation from the
  element alone — the recurrence-versus-formula cross-check
  (`length == (W_length + #cayley_roots)/2`).
- Mod-space: for each involution of SL(2,R) and Sp(4,R), the reduction
  of a nonzero torus bit pattern matches the hand-computed coset
  representative; quotient dimensions match the fiber layer where both
  exist.
- Budget and invariants: an under-budgeted build is the named resource
  rejection; a foreign-system element is rejected by the expressible
  provenance gate.

`tests/fixtures/domain/involution_table.atlas` is reserved; the
differential observables (per-Cartan orbit sizes, length profiles)
reach the language layer through KGB, per the stage map.

## Consequential updates

Landing this stage must update: `lib.rs` (module and exports);
`error.rs` (the two variant families); `KGB_STAGE_MAP.md` (stage (b)
landed); `REAL_GROUP_DESIGN.md`'s progression (next: stage (c), Tits
operations); and task #9's scope note — the per-Cartan BFS here IS the
on-demand orbit generation that should eventually replace
`twisted_conjugacy_classes`' full-W enumeration, recorded there.

## Three independent design checks

Before implementation, this design must be reviewed in three fresh
subagent contexts: (1) Atlas source semantics — the record fields
against involutions.h/cpp, the length recurrence, the fiber_denom
code-versus-comment call, and whether dropping `M_real`/`lift_mat` from
stage (b) loses something stages (c)/(d) need; (2) Rust internals — the
all-fresh entry path's cost honesty at E8 scale, the HashMap keying on
`WeylElement`, reuse of `TwistedInvolution`/`negative_coweight_eigenspace`,
and the `WeylAction` composition along BFS edges; (3) public API and
consumer fit — the constructor/ownership shape against the crate's
provenance idiom, what stage (c) and the KGB loop actually call, and
the `add_cartan`/`cross`/`lookup` surface. Findings and corrections
will be recorded here before source edits begin.

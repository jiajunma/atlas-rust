# KGB generation design (stage e)

## Approved scope

Stage (e) of the KGB map: the per-real-form KGB graph — BFS from the
stage-(d) seed over cross actions and Cayley transforms, statuses and
descents, the sorted numbering, inverse-Cayley installation, and the
hard size assert against the strong layer. Consumes every landed
substrate: `WeylElement`, `InvolutionTable`, `TitsElement`/`TitsCoset`,
`RealFormSeed`. Bruhat/Hasse data (upstream `makeHasse`) is stage (f).

## Atlas construction (oracle trace)

Citations are to `~/mycodes/atlasofliegroups`, master `4d3e9449`.

- Pool and dedup: `TE_Entry` hash of reduced `(Weyl part, torus bits)`
  pairs; "new" is `match` returning the append index (kgb.cpp:514-515,
  570-572). Reserves are taken from the a-priori size
  `KGB_size(rf) = sum over Cartans of orbitSize * fiberSize`
  (kgb.cpp:517-520; innerclass.cpp:849-859), asserted EXACTLY at the
  end of the BFS, before sorting (kgb.cpp:616). The involution table
  is force-completed for the form's (upward-closed, asserted) Cartan
  set BEFORE generation (kgb.cpp:501-512).
- Per (generator, element): `cross_image`, `Cayley_image`, and an
  `inverse_Cayley_image` PAIR that stays undefined through the whole
  BFS (kgb.h:72-82) — inverse Cayley is a pure post-pass in KGB
  (kgb.cpp:669-682; global_KGB differs, do not copy it). Per element:
  a 2-bit-per-generator status plus a descent bitset — and NOTHING
  else: length is INVOLUTION-TABLE data reached through the element's
  involution, never per-element state (kgb.h:140-141).
- BFS body (kgb.cpp:553-614), per x ascending, per s ascending:
  cross first — `lc = 0` if the twisted involution is fixed, else the
  Weyl length change; `lc != 0` means COMPLEX, descent iff `lc < 0`;
  `lc == 0` with a Weyl descent means REAL, always a descent, and the
  cross image is asserted to be x itself (kgb.cpp:584 — a canonicality
  check on the reduction); otherwise IMAGINARY, graded by
  `simple_grading`, never a descent. Status slots are OR-set, so each
  `(s, x)` is written exactly once, at x's visit. If noncompact
  imaginary: Cayley transform, asserted length-increasing, reduced at
  the grown target subspace, hashed, forward link only
  (kgb.cpp:594-607).
- Renumbering (kgb.cpp:618-667), end to end: `inv_nrs` = the form's
  involution numbers per Cartan; STABLE-sorted by (involution length,
  Weyl length, involution number); `inv_loc` inverse with a sentinel
  for absent involutions; `invs[x]` = sorted bucket of x's involution;
  `standardization(invs, buckets, &first_of_tau)` — a COUNTING SORT
  whose cumulative counts ARE `first_of_tau` (size = buckets + 1) and
  whose output `pi[i]` = new position of old i, stable within equal
  buckets (permutations.cpp:249-284); records are placed by the
  INVERSE (`new slot i <- old a[i]`) while link TARGETS renumber
  through the forward map — the classic two-map split; the per-element
  torus parts are extracted in sorted order directly from the pool.
  Per-element length and Cartan class stay derived:
  element -> `first_of_tau` bucket -> `inv_nrs` -> table record.
- Inverse-Cayley installation (kgb.cpp:669-682): ascending over the
  SORTED numbering, each forward Cayley link fills its target's pair —
  `.first` if unset (hence the lower source), else `.second`; type II
  leaves `.second` undefined.
- `torus_factor(x)` = `symmetrise(g_rho_check - lift(bits), theta)` =
  `(v + v.theta)/2` normalized — exact rational, theta-transpose-fixed
  (kgb.cpp:705-712); `titsElt` reassembles from the persisted
  (involution index, torus bits) pair (kgb.cpp:694-695).

## Port decisions

1. The KGB value is `KgbGraph` (one weak real form), built by
   `KgbGraph::build(&InnerClass, &CartanClassification,
   &StrongRealClassification, &mut InvolutionTable, &RealFormSeed,
   ...)`: the builder ADDS the form's `cartan_set` to the shared table
   itself (ascending — the stage-(b) determinism discipline;
   idempotent re-adds make cross-form sharing safe), mirroring
   upstream's forced orbit generation, then builds the `TitsCoset`
   from the seed's offset. Provenance: the gates already landed in the
   substrates (table/inner-class equality, seed's form ranged through
   the strong layer) plus seed-element bounds against the table.
2. Elements are the stage-(c) `TitsElement` throughout — the persisted
   shape IS the working shape. Dedup by `BTreeMap<TitsElement, KgbId>`
   (the crate's map precedent; derived `Ord` on reduced elements is
   the normal-form contract). `KgbId(pub(crate) usize)` with the full
   ID derive set.
3. Statuses: a public 4-variant `KgbStatus` enum (Complex,
   ImaginaryCompact, Real, ImaginaryNoncompact) stored one per
   generator per element (a Vec, not bit-packing — dynamic rank, no
   RANK_MAX). Descents as `Vec<bool>` per element. The complex length
   change reads the two records' cached Weyl lengths; the real test is
   the record element's O(1) descent query; imaginary grading is the
   coset's `simple_grading`. The real-cross self-loop and the
   Cayley-length-grows asserts port as invariants ("real cross fixed",
   "Cayley length step").
4. Links: `cross: Vec<Vec<KgbId>>`, `cayley: Vec<Vec<Option<KgbId>>>`
   (generator-major like upstream is NOT kept — element-major
   `links[x][s]` matches the crate's records-then-fields layout),
   inverse Cayley as `Vec<Vec<(Option<KgbId>, Option<KgbId>)>>`
   installed by the ascending post-pass. `None` plays `UndefKGB`.
5. Numbering: `inv_nrs` sorted by (involution length, Weyl length,
   `WeylElement` derived `Ord`) — the stage-(a) documented tie-break
   in place of upstream's transducer compare (adapter-deferred
   numbering divergence within equal-length groups, recorded since
   stage (a)); the counting-sort standardization ported EXACTLY
   (stable within buckets — within a tau packet the BFS discovery
   order is preserved, which the two implementations SHARE given the
   seed and reduction coincidences); `first_of_tau` from the
   cumulative counts; records placed by the inverse map, link targets
   renumbered by the forward map.
6. Size discipline: `kgb_size(form)` is the a-priori bound — reserves
   come from it, exceeding it during the BFS is
   `KgbInvariantViolation { invariant: "kgb size" }` (hard-fail early,
   upstream asserts late; both are checked errors here), and the
   post-BFS exact-equality check is the same invariant. Per-Cartan
   packet sizes are cross-checked against `orbitSize x fiberSize`
   (`fiber_size(form, cartan)`) as a test, not a runtime gate. No new
   budget kind: the only knob is what the substrates already take.
7. `torus_factor(x)`: exact rational from the seed's cocharacter minus
   the bit lift, symmetrized by the element's theta and normalized —
   returning `RationalCoweight` (its pub(crate) coordinates finally
   consumed in production). Also exposed: `length(x)` (via the table),
   `cartan_of(x)`, `involution_of(x) -> InvolutionId`, `element(x) ->
   &TitsElement`, `tau_packet(id range per involution)`, sizes.
8. Length is NEVER per-element state (upstream parity): derived
   through `first_of_tau` bucketing. Per-element memory: one
   `TitsElement` + rank statuses/links — matching the upstream
   persisted shape plus links.

## Data layout and public boundary

```text
// kgb_graph.rs
KgbId(pub(crate) usize)             // full ID derive set
KgbStatus { Complex, ImaginaryCompact, Real, ImaginaryNoncompact }
KgbGraph::build(
    &InnerClass, &CartanClassification, &StrongRealClassification,
    &mut InvolutionTable, &RealFormSeed,
) -> Result<KgbGraph, StructureError>
size() / form() -> WeakRealFormId
element(KgbId) -> Option<&TitsElement>
involution_of(KgbId) -> Option<InvolutionId>
length(KgbId, &InvolutionTable) -> Option<usize>       // table-derived
cartan_of(KgbId, &InvolutionTable) -> Option<CartanId>
status(KgbId, generator) -> Option<KgbStatus>
is_descent(KgbId, generator) -> Option<bool>
cross(KgbId, generator) -> Option<KgbId>
cayley(KgbId, generator) -> Option<Option<KgbId>>
inverse_cayley(KgbId, generator) -> Option<(Option<KgbId>, Option<KgbId>)>
tau_packet(involution position) / first_of_tau() accessors
torus_factor(KgbId, &RealFormSeed?) -> ...             // exact rational
```

Whether the graph stores the seed's cocharacter (making
`torus_factor(KgbId)` self-contained) or takes the seed per call, and
whether table-derived accessors take `&InvolutionTable` per call or
the graph stores what it needs (lengths per involution position are
tiny — copying them in at build time removes the per-call table
dependency), are REVIEW QUESTIONS — the port leans toward copying the
per-involution (length, cartan) pairs and the cocharacter into the
graph at build, so the finished value answers every query without
substrate references, at O(#involutions + rank) extra memory.

Errors: `KgbResourceLimit { resource, limit }` (reserve failures ride
`AllocationFailed`; the variant exists for the a-priori bound if a
review prefers it over the invariant) and
`KgbInvariantViolation { invariant }` ("kgb size", "real cross fixed",
"Cayley length step", "cayley target missing", "status write-once").

## Tests and fixture gate

- Published sizes, the gate: SL(2,R) = 3, PGL(2,R) = 2, Sp(4,R) = 11,
  SU(2,1) = 6, compact forms = 1 — via the full pipeline (seed +
  shared table), with the strong layer's `kgb_size` agreeing by
  construction (the invariant) and the global sum over forms matching
  `global_kgb_size` where cheap.
- Structure: cross is involutive and status-consistent (real cross
  fixes, complex cross moves); every noncompact-imaginary generator
  has a Cayley link whose target's inverse pair points back; type
  I/II pair shapes on SL(2,R) (double) vs PGL(2,R) (single);
  tau-packet sizes equal `orbitSize x fiberSize` per Cartan; lengths
  ascend along Cayley links by one.
- Numbering: element 0 is the seed; two builds agree; per-length
  counts match Atlas published data for Sp(4,R) (1,2,2,...) as
  recorded in the fixture notes.
- torus_factor: theta-fixedness of every value; the seed element's
  factor equals the symmetrized cocharacter.
- The compact form: one element, all generators ImaginaryCompact, no
  links beyond self-crosses.

`tests/fixtures/domain/kgb_generation.atlas` is reserved; KGB sizes,
per-length counts, and status multisets are the first structurally
complete language-level observables of the whole chain.

## Consequential updates

`lib.rs` (module + exports); `error.rs` (the Kgb families);
`KGB_STAGE_MAP.md` (stage (e) landed); `REAL_GROUP_DESIGN.md`
progression (next: stage (f) descents/Bruhat, then the language
bridge); task #8's language-bridge design gains its first executable
observable set.

## Three independent design checks

Before implementation, three fresh reviews: (1) Atlas semantics — the
BFS status/descent assignment against kgb.cpp:553-614, the
renumbering pipeline's two-map split and standardization stability,
the inverse-Cayley pair shape, and the torus_factor formula; (2) Rust
internals — the substrate fit (coset cross/cayley shapes, table
mutation during build, BTreeMap dedup cost at split-E8 scale, the
copied-in per-involution data), and the counting-sort port; (3) API
and consumer fit — the build signature's five inputs and gates, the
self-contained-graph question, accessor shapes against the crate
idiom, and what stage (f) and the language bridge consume. Findings
and corrections will be recorded here before source edits begin.

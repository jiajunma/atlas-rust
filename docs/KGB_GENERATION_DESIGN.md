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
length(KgbId) -> Option<usize>            // copied per-involution data
cartan_of(KgbId) -> Option<CartanId>
status(KgbId, generator) -> Option<KgbStatus>
is_descent(KgbId, generator) -> Option<bool>
cross(KgbId, generator) -> Option<KgbId>
cayley(KgbId, generator) -> Result<Option<KgbId>, ...>
inverse_cayley(KgbId, generator)
    -> Result<Option<(KgbId, Option<KgbId>)>, ...>
    // outer None = generator not real; (first, None) = type II;
    // (first, Some(second)) = type I with first < second
tau_packet(position: usize) -> Option<(KgbId, usize)> / packet_count()
torus_factor(KgbId, &InvolutionTable) -> Result<RationalCoweight, ...>
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
  construction. `global_kgb_size` counts STRONG forms and is NOT the
  sum of per-weak-form KGB sizes (SL(2): 3 + 1 = 4 vs 5) — no
  global-sum test exists.
- Structure: cross is involutive and status-consistent (real cross
  fixes, complex cross moves); every noncompact-imaginary generator
  has a Cayley link whose target's inverse pair points back; type
  I/II pair shapes on SL(2,R) (double) vs PGL(2,R) (single);
  each tau packet has size `fiberSize(form, cartan)` and the sum over
  a Cartan's `orbitSize` involutions is `orbitSize x fiberSize`;
  lengths ascend along Cayley links by one.
- Numbering: element 0 is the seed (the identity involution is the
  unique length-0 bucket minimum, immune to the tie-break divergence);
  two builds agree; per-length ELEMENT counts for Sp(4,R) are
  (4,3,3,1) and per-length involution counts (1,2,2,1) — oracle-run
  data, requiring the simply connected C2 datum for size 11.
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

## Three independent design checks (returned; corrections folded)

The full findings live in the review archive; the deltas adopted:

1. Atlas semantics (verified against the RUNNING oracle binary):
   Sp(4,R) per-length data corrected above; the involution sort's
   third key is the TwistedInvolution VALUE compare (the upstream
   "internal number" comments are STALE — the port's WeylElement-Ord
   tie-break is the right analogue), and `stable_sort`'s stability is
   vacuous there (a strict total order) — the load-bearing stability
   is exclusively the counting sort's. Shared within-packet discovery
   order requires ALL of: same generator numbering, the single seed,
   canonical reduction, and identical simple_grading decisions; and
   Sp(4,R) itself EXERCISES the tie-break divergence (ties at
   (1,1) and (2,2)), so exact numbering is not comparable even for
   the primary fixture — counts, packet contents, and structure are.
   Write-once holds by loop discipline (upstream's OR-set would
   silently corrupt; the port's checked invariant is a strengthening;
   descents are assignments, not ORs). torus_factor sharpenings: the
   lift is an INTEGER 0/1 vector; the bits are the LEFT torus part;
   symmetrise is v += v.theta (right product), halve, then normalize
   (gcd out, positive denominator). The lc and real tests are LEFT
   descents. New checked invariants: "inverse Cayley pair" (third
   preimage; upstream silently overwrites) and "involution bucket"
   (no sentinel may reach the counting sort). The persisted-shape
   claim is corrected: the port's per-element TitsElement is a
   SUPERSET of upstream's torus-bits-only persisted state. The
   upward-closure precondition is vacuous in full-KGB scope; the
   early size check is strict-greater after append plus final exact
   equality.
2. Rust internals: the self-contained-graph claim breaks on theta —
   DECIDED HYBRID: the graph copies per-involution-position
   (InvolutionId, involution length, CartanId) plus a per-element
   position index and the cocharacter, making every accessor
   substrate-free EXCEPT `torus_factor(KgbId, &InvolutionTable)`.
   The per-call deep InnerClass equality gate would cost minutes at
   E8 across millions of coset calls — the builder gates ONCE and
   uses pub(crate) pre-gated coset entry points (a small
   tits_element.rs addition landing with this stage). The seed-table
   binding gate: after the add_cartan loop,
   `table.lookup(identity) == Some(seed.element().involution())`
   plus reduced-fixity ("seed element" invariant) — an in-bounds id
   from a DIFFERENT same-inner-class table is otherwise a silently
   wrong graph. Cayley edges are memoized per (generator, involution)
   in the builder (the table edge re-multiplies per call). Storage is
   FLAT strided (`x * rank + s`) — the nested-Vec layout wastes
   ~60-90 MB and ~1.9M allocations at E8; the dedup BTreeMap is
   build-local and dropped. Statuses classify from
   `simple_root_kind` as primary with the length-change
   reconstruction demoted to an invariant cross-check; write-once
   slots are free via 1-byte `Option<KgbStatus>` niches. The
   counting-sort port snapshots `first_of_tau` BEFORE the placement
   loop (the in-place post-increment trap); `inv_loc` is
   `Vec<Option<usize>>` at full table size; `sort_unstable_by` is
   legal (total order). Borrow discipline: nothing borrows the table
   across the `&mut` add_cartan phase; each BFS visit clones its
   element out of the pool first. The torus_factor helper family
   (bit lift to rationals, integer-matrix-times-rational-vector,
   symmetrize) is NEW code; `fractional_part` moves to the shared
   home.
3. API and consumer fit: the false global-sum identity deleted
   (finding confirmed by the crate's own SL(2) test: 4 vs 5);
   `cayley` returns `Result<Option<KgbId>>` (the two existing Cayley
   surfaces' exact idiom); the inverse pair is
   `Option<(KgbId, Option<KgbId>)>` so the illegal `(None, Some)`
   state is unrepresentable; `KgbStatus` stays a public 4-variant
   enum but the parallel descent Vec is DROPPED — `is_descent` is
   computed O(1) from copied lengths across the stored cross link;
   `KgbResourceLimit` is dropped (never-constructed variant; the
   invariant covers both checks); `KgbGraph` derives
   `Clone, Debug, Eq, PartialEq` (the two-builds test needs it);
   `&InnerClass` stays in the signature WITH its explicit equality
   gate stated; a public per-coordinate exact view on
   `RationalCoweight` joins the consequential updates (the language
   bridge cannot read pub(crate) coordinates); the meta scenarios
   gain the compact-form shape, the seed-factor check, and status
   multisets; and `KGB_STAGE_MAP.md`'s stage (f) is rewritten to
   descents-consumption/Bruhat-Hasse only (inverse Cayley and tau
   packets land HERE, faithful to upstream's constructor).

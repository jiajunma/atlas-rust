# On-demand twisted-conjugacy partition design (task #9)

Reviewed 2026-07-28 by three independent fresh-context checks (Atlas
semantics / Rust internals / API-consumer fit); every correction is
folded below. Sections marked [R] changed under review.

## Approved scope

Replace the full-Weyl-group enumeration inside
`InnerClass::twisted_conjugacy_partition` with the upstream Cartan-class
discovery loop, removing the port's only |W|-sized computation.
Quantified motivation (the HPC differential, job 3488002): B5/C5 at
145 ms and E6 at 2.1 s against ~30 ms upstream — entirely this
enumeration — and the language bridge's `WEYL_BUDGET = 200_000` ceiling
that shuts out E7/E8 (|W(E7)| = 2.9e6, |W(E8)| = 7e8).

## The bottleneck today

`InnerClass::enumerated_twisted_involutions` builds ALL |W| canonical
`WeylAction` matrices, filters the involutive translates, and
`twisted_conjugacy_partition` conjugates every candidate by every
enumerated action — O(|W| x #involutions x roots) work and O(|W| x
rank^2) live memory. Consumers: `CartanClassification::build` (class
list with per-class involution counts, `class_of` for the fundamental
lookup and the Cayley `below` links) and the public
`twisted_involutions`/`twisted_conjugacy_classes` wrappers.

## The upstream mechanism (traced, innerclass.cpp:197-294)

Upstream NEVER enumerates twisted involutions to find Cartan classes.
`InnerClass::construct()` runs a worklist BFS over the CLASSES:

- Seed: `Cartan[0]` = the identity twisted involution (canonical as
  is). The vector grows while the loop `for i < Cartan.size()` runs.
- For each known class `i`: take the positive imaginary roots of its
  canonical representative; for EACH such root alpha, Cayley-transform
  and `canonicalize` the result.
- Dedup = EQUALITY of canonical representatives against the stored
  `Cartan[ii].tw` (linear scan over the few known classes). Unseen
  canonical value => `push_back` — the only growth point.
- Numbering = discovery order; every parent has a smaller index (each
  single-root Cayley raises the involution length by one, so the BFS
  is graded upward).
- `below` covers are recorded child <- parent at match time.
- Per-class involution ORBITS (`C_orb`) stay lazy; KGB fills them per
  real form — exactly our stage-(b) `InvolutionTable::add_cartan` BFS.

Work: O(#classes x #posImRoots x canonicalize), memory O(#classes).
No global involution set exists at any point.

[R] Upstream conjugates alpha simple first (a descent loop) because it
must transport torus parts along the conjugating word. This port needs
no word (see "canonicalize"), and with `c` the conjugator making alpha
simple with reflection `s`: `s.(c.w.twist(c)^-1)` is twisted-conjugate
to `s_alpha.w`, so canonicalize sends both to the same canonical form.
The port therefore takes the candidate DIRECTLY as `s_alpha . w`
(`WeylAction::root_reflection` composed with the representative — the
exact pattern already tested at cartan_classification.rs:189-202) and
skips the descent loop entirely.

## canonicalize (innerclass.cpp:739-834) — the new port

`canonicalize(sigma)` rewrites a twisted involution in place to its
unique class-canonical form by twisted conjugations
`sigma -> s . sigma . twist(s)`:

1. Make the REAL-posroot sum `rrs = 2rho(real)` dominant (any simple
   root with negative pairing gets a correcting twisted conjugation);
   ties broken by making the IMAGINARY-posroot sum `irs` dominant on
   simple roots orthogonal to `rrs` (the interleaved criterion is
   `c < 0 or (c == 0 and <irs, alpha_i_vee> < 0)`). Every corrected
   root is complex.
2. Restrict the generator set to FULL-SYSTEM simple roots orthogonal
   to BOTH sums. [R] Note this is NOT the same object as the simple
   basis of the orthogonal-complex subsystem used by the orbit-size
   computation below; the two coincide only once phase-1 dominance
   holds. Keep the helpers separate.
3. Within that generator set, twisted-conjugate until theta maps each
   of its members to a POSITIVE root.

The result is deterministic per class, so dedup is plain equality.

[R] REPRESENTATION: canonicalize runs on the CHEAP representation —
`WeylElement` (root permutation; `twisted_conjugate` landed at
weyl_element.rs:229) — with `rrs`/`irs` maintained INCREMENTALLY by
simple reflections of the two weight vectors, exactly as upstream
(`simple_reflect(s, rrs)` per step). No `TwistedInvolution` and no
`RootInvolutionData` is constructed per step: one `RootInvolutionData`
per candidate at entry (to seed the real/imaginary sums), one
`TwistedInvolution` per NEW class for the stored representative
(replay the action via `root_reflection` composition along the
element's reduced word). Building `TwistedInvolution` per step would
re-validate the whole root system each time and defeat the sub-10 ms
target.

[R] TERMINATION (checked, not assumed): phase 1 strictly decreases the
lexicographic potential (#{positive coroots negative on rrs},
#{positive coroots negative on irs}) — a `c<0` step drops the first
coordinate by exactly one, a tie step fixes rrs and drops the second
by one — bound O(#posroots^2) steps. Phase 3 steps are complex
descents, each dropping the involution length by 2 — bound #posroots.
The port carries intrinsic iteration caps derived from these bounds
and raises `CartanClassificationInvariantViolation` on exceed (the
`max_peeling_steps` convention); NO new caller-facing budget knob.

[R] The conjugating word upstream returns is used only for torus-part
transport of per-Cartan real-form reps; this port computes those via
`RealFormLabels`/gradings instead, and every port consumer at class
representatives needs only the representative itself (verified across
`CartanClassification::build`, `CayleyCrossDecomposition::build`,
`InvolutionTable::add_cartan`, KGB generation). The word is dropped.

## Replacement architecture

1. DISCOVERY (`InnerClass`): worklist over classes (the index-cursor
   pattern proven at involution_table.rs:243-281). For each class:
   positive imaginary roots of the canonical representative; candidate
   `s_alpha . w` per root; canonicalize; dedup by equality of the
   Weyl part ONLY (image permutation — the involution_table.rs:99
   map-key precedent), never full `TwistedInvolution` equality.
   [R] At every dedup match, CHECK `matched_index > current_index`
   (raise the invariant violation otherwise) — this subsumes the
   self-cover check and is precisely what makes graded numbering
   trustworthy rather than assumed. [R] The existing
   `InvalidInvolution -> CartanClassificationInvariantViolation`
   remap carries into candidate construction, including candidates
   that match existing classes.
2. CLASS ORDER: discovery order, fundamental = 0 by construction (the
   identity IS canonical; no post-hoc reshuffle). [R] This order is
   crate-deterministic but PRESENTATION-DEPENDENT (our `RootId` order
   is ambient-coordinate lexicographic) and is NOT expected to equal
   upstream `CartanNbr`; only `CartanId(0)` = fundamental is pinned.
   The standing compatibility-adapter deferral stays at full width,
   and the differential is NOT cited as validating numbering (it
   compares per-form KGB sizes and sorted length multisets, which are
   numbering-invariant).
3. REPRESENTATIVES: the canonical involutions themselves; the
   fundamental representative is the identity, as label gates require.
4. `class_of(twisted)`: [R] provenance gates FIRST (`DatumMismatch`,
   `DistinguishedInvolutionMismatch` — the pinned behaviour of
   cartan_class.rs:77-103), then canonicalize + linear scan. [R] The
   `TwistedConjugacyPartition` type cannot run canonicalize today (it
   lacks the `RootSystem` and the simple-generator twist permutation,
   currently private to `InvolutionTable::new`): the partition GAINS a
   `RootSystem` field (it already clones the datum), and the
   twist-of-generators computation is extracted to a shared
   `InnerClass` helper. This is a named structural change, not "API
   kept as-is".
5. PER-CLASS INVOLUTION COUNT: the order-quotient
   `orbitSize = |W| / (|W_im| x |W_re| x |W_cx|)`
   (cartanclass.cpp:1046-1064) with `weyl_size::weyl_order_of_cartan`
   (landed; order-only recognition — subsystem simple bases are
   pairwise obtuse and crystallographic, so entries stay in
   {0,-1,-2,-3} and the branch analysis is total for genuine inputs):
   - |W|: the datum's full Cartan matrix.
   - |W_im|, |W_re|: `RootSystem::bracket` matrices over
     `RootInvolutionData::{imaginary,real}_simple_roots` (landed).
   - |W_cx|: the `makeSimpleComplex` port (cartanclass.cpp:1002-1043).
     [R] VERBATIM pair-keeping rule: collect roots orthogonal to both
     `2rho(imaginary)` and `2rho(real)`; take the simple basis of that
     set (a GENERALIZED `subsystem_simple_roots` over an arbitrary
     closed positive subset — extract from root_involution.rs:124-181,
     which is currently kind-keyed and private); split into Dynkin
     components (extract the component splitter from weyl_size.rs as a
     data-returning helper); then for each surviving component compute
     `img = theta(first basis root)` and erase the FIRST FOLLOWING
     component containing a basis root NON-ORTHOGONAL to `img`. Theta
     does NOT map basis onto basis (`img` is generally not a basis
     root — upstream's own NOTE at cartanclass.cpp:986-1000 records
     this bug class); orthogonality search is the only correct pairing
     test. [R] Invariants upstream lacks, all checked here: every
     surviving component erases exactly one distinct partner; kept
     components cover exactly HALF the orthogonal-set basis; pairing
     failure is an invariant violation. (A silent pairing failure
     yields an EXACT but WRONG quotient — exactness alone does not
     guard this; the BFS cross-check test does.)
   - [R] Exactness: `DivisibleBy` on the full product
     |W_im| x |W_re| x |W_cx| (the crate's real_form_labels.rs:279
     pattern), then exact-divide; usize conversion via `TryFrom`, with
     failure mapped to the INVARIANT violation (a huge-but-exact
     quotient is not `ArithmeticOverflow`).
   `twisted_involution_count` = checked sum of orbit sizes.
6. BUDGET: `weyl_budget` is REINTERPRETED as the twisted-involution-
   count budget: `sum > budget` (STRICTLY greater — equality passes;
   A1 with budget 2 = 2 involutions and A1xA1 with 4 = 4 are
   load-bearing edges) raises `ResourceLimitExceeded { limit }` (the
   variant is REUSED, no new kind); the discovery worklist is
   additionally capped by the same parameter as a strict class-count
   bound. Every current SUCCESS call site stays valid (#involutions <=
   |W|). [R] FAILURE-path tests must be re-derived, not just
   CartanId-hardcoding ones: inner_class.rs:309-313 asserts A2 fails
   at budget 5 (|W| = 6) — under the new meaning 4 <= 5 succeeds, so
   the test rewrites to a budget BELOW the involution count (e.g. 3).
   No signature change; docs updated
   (`CartanClassificationBudget::weyl_budget` included).
7. `below`: covers come from discovery. [R] The transitive closure
   KEEPS the existing order-independent Warshall + irreflexivity check
   (cartan_classification.rs:212-230) — O(#classes^3) on a tiny n —
   instead of porting upstream's incremental `new_max`, whose
   precondition (all covers into i recorded before row i closes) is
   exactly what a canonicalize bug would silently violate. Together
   with the `matched_index > current_index` check this KEEPS all three
   protections of today's phase-4 loop (successor validity, no
   self-cover, closed irreflexive order); the loop itself retires only
   as duplicated WORK, not as lost checking.
8. `twisted_involutions(budget)` (the public unquotiented list):
   generated per class by a closure-BFS helper, bounded by the same
   involution-count budget. [R] Extract ONE free-standing orbit-
   closure helper (over root system + reflections + twist) shared by
   `InvolutionTable::add_cartan` (currently the only copy, and
   circularly unreachable from `InnerClass`), this wrapper, and the
   cross-check test — no third hand-rolled BFS. [R] Ordering contract
   documented: classes in discovery order, BFS order within a class
   (the docstring's "stable list" stays true with the new order
   stated).

## Consequences

- `enumerate_actions` remains for small-rank tests and the Weyl layer;
  nothing in the classification path calls it.
- The language bridge's `WEYL_BUDGET` constant retires in favour of an
  involution-count constant (E6: 892; E7 ~ 1e4; E8 ~ 2e5). E7 unlocks
  immediately; E8's KGB remains gated by fiber sizes, not this stage.
- Expected timing: E6 classification from ~2.1 s to the class-count
  scale; B5/C5/D5 similar collapses.
- Tests hardcoding `CartanId` values are revalidated; budget-failure
  tests re-derived (point 6).

## Tests and gate

- Transition test old-vs-new on A1, A2 (twisted + untwisted), B2,
  A1xA1, [R] PLUS A3-twisted, B3, D4-identity (the cheap old path
  covers them free): compare class member SETS under the class
  bijection AND the full `is_below` matrix AND `cartan_set` /
  `most_split` per form — not member sets alone. Rank <= 2 exercises
  only trivial complex pairing; A3/D4 reach paired components of
  rank >= 2 where `img` is non-simple.
- Orbit-size cross-check: per class, closure-BFS size == quotient
  formula, on every group in the local battery (BFS and quotient are
  unrelated computations — this is the guard for silent pairing
  failures).
- `canonicalize` unit tests: idempotent; constant on hand-conjugated
  pairs; identity fixed; A2-twisted and B2 classes land on distinct
  canonical forms.
- The full local suite green; clippy 1.90 AND 1.96 clean; fmt.
- The HPC differential battery (17 groups + E7 added) all-MATCH with
  timings recorded; corpus scoreboard unchanged or improved.

## Review disposition

Semantics: 4 findings — folded (pair-keeping verbatim + invariants,
checked graded covers, numbering claim withdrawn, two-bases
clarification). Internals: 10 findings — folded (WeylElement-based
canonicalize with incremental sums, partition structural change named,
descent loop dropped, termination caps specified, helper extractions,
budget-test inversions, Warshall kept, permutation-only dedup,
DivisibleBy/TryFrom arithmetic, shared orbit-closure helper).
API: 6 findings — folded (budget failure-path rewrite + strict
inequality + variant reuse, all three phase-4 protections retained,
transition test widened to poset observables, numbering claim
corrected, provenance gates first, ordering contract documented).
No unresolved blocking items.

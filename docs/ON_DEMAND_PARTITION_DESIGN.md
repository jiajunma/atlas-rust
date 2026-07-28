# On-demand twisted-conjugacy partition design (task #9)

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
  canonical representative; for EACH such root alpha, conjugate alpha
  simple (descent loop), Cayley-transform (`sigma = s . conjugated
  tw`), then `canonicalize(sigma)`.
- Dedup = EQUALITY of canonical representatives against the stored
  `Cartan[ii].tw` (linear scan over the few known classes). Unseen
  canonical value => `push_back` — the only growth point.
- Numbering = discovery order: fundamental first, most-split last
  (asserted upstream); every parent has a smaller index, so the order
  is graded upward.
- `below` covers are recorded child <- parent at match time; the poset
  is the incremental transitive closure (`new_max`, poset.h:127-140).
- Per-class involution ORBITS (`C_orb`) stay lazy; KGB fills them per
  real form — exactly our stage-(b) `InvolutionTable::add_cartan` BFS.

Work: O(#classes x #posImRoots x canonicalize), memory O(#classes).
No global involution set exists at any point.

## canonicalize (innerclass.cpp:739-834) — the new port

`canonicalize(sigma)` rewrites a twisted involution in place to its
unique class-canonical form by twisted conjugations
`sigma -> s . sigma . twist(s)`:

1. Make the REAL-posroot sum `rrs = 2rho(real)` dominant (any simple
   root with negative pairing gets a correcting twisted conjugation);
   ties broken by making the IMAGINARY-posroot sum `irs` dominant on
   simple roots orthogonal to `rrs`. Every corrected root is complex.
2. Restrict the generator set to simple roots orthogonal to BOTH sums
   (the complex subsystem).
3. Within that subsystem, twisted-conjugate until theta maps each of
   its simple roots to a POSITIVE root.

The result is deterministic per class, so dedup is plain equality.
Port shape: operates on our `TwistedInvolution` (root-permutation +
`WeylAction`); needs 2rho sums of the real/imaginary root sets
(available from `RootInvolutionData`), simple-coroot pairings, and the
existing twisted-conjugation step. The returned conjugating word is
NOT needed by this task (no torus-part transport here — the KGB layer
already transports via its own machinery).

## Replacement architecture

1. DISCOVERY (`InnerClass`): port the construct() task-1 loop verbatim
   — worklist over classes, Cayley candidates from every positive
   imaginary root of the canonical representative (conjugate-to-simple
   descent loop included), canonicalize, dedup by equality, record
   `below` covers. Produces: ordered class list (canonical
   representatives), cover relation.
2. CLASS ORDER: discovery order — the SAME rule as upstream CartanNbr.
   Fundamental = 0 by construction (no post-hoc normalization needed:
   the identity IS the canonical representative). The current
   matrix-lex order and its fundamental-first reshuffle in
   `CartanClassification::build` both retire. Candidate roots iterate
   ascending by our `RootId`; upstream iterates ascending RootNbr — if
   our root numbering matches upstream's the numbering coincides
   exactly; either way it is deterministic and the differential
   validates every observable.
3. REPRESENTATIVES: the upstream-canonical involutions themselves —
   strictly better than any port-invented minimum. The fundamental
   representative is the identity, as the label gates require.
4. `class_of(twisted)`: canonicalize the argument, compare with the
   stored representatives (linear scan over #classes). No global
   permutation map. The `TwistedConjugacyPartition` public type keeps
   its API; its membership map is replaced by this computation.
5. PER-CLASS INVOLUTION COUNT: the order-quotient
   `orbitSize = |W| / (|W_im| x |W_re| x |W_cx|)`
   (cartanclass.cpp:1046-1064) with `weyl_size::weyl_order_of_cartan`
   (landed, tested: A/B/C/D/E/F/G/torus recognition over exact
   `Integer`s — only ORDERS are needed, so B-vs-C is irrelevant):
   - |W|: the datum's full Cartan matrix.
   - |W_im|, |W_re|: Cartan matrices of the SIMPLE BASES of the
     imaginary/real root sets (`simpleBasis` port over our
     `RootSystem` if not already present).
   - |W_cx|: `makeSimpleComplex` (cartanclass.cpp:1002-1043) — roots
     orthogonal to both `2rho(imaginary)` and `2rho(real)`, simple
     basis of that set, split into Dynkin components, keep ONE
     component per theta-swapped pair (theta maps RC_0 onto RC_1).
   - The division must be exact — CHECKED in the port (upstream's
     exponent arithmetic is unchecked); non-exactness is an invariant
     violation, as is a quotient that overflows usize.
   `twisted_involution_count` = sum of orbit sizes (checked add). This
   count is what the reinterpreted budget bounds (below).
6. BUDGET: `weyl_budget` is REINTERPRETED as the twisted-involution-
   count budget: after sizes are computed, `sum > budget` is a budget
   error; the discovery worklist itself is additionally bounded by the
   same parameter as a class-count cap (classes <= involutions always,
   so this cannot tighten). Every current caller passes |W| >=
   #involutions, so existing budgets only get looser. No signature
   change; docs updated (`CartanClassificationBudget::weyl_budget`
   included).
7. `below`: the classification's Cayley-link recomputation (its
   phase-4 loop duplicates discovery) retires — covers come from
   discovery, closed transitively with the upstream incremental
   scheme, which is now correct because the order is graded (parents
   strictly below in index). The irreflexivity invariant stays.
8. `twisted_involutions(budget)` (the public unquotiented list): now
   generated per class by the stage-(b) closure BFS (union of orbits),
   bounded by the same involution-count budget. Test-facing; not on
   the classification's hot path.

## Consequences

- `enumerate_actions` remains for small-rank tests and the Weyl layer;
  nothing in the classification path calls it.
- `CartanId` now follows the upstream discovery-order rule — the
  standing "non-Atlas numbering" caveat narrows to root-numbering
  agreement, and the adapter deferral covers the remainder.
- The language bridge's `WEYL_BUDGET` constant retires in favour of an
  involution-count constant (E6: 892; E7 ~ 1e4; E8 ~ 2e5). E7 unlocks
  immediately; E8's KGB remains gated by fiber sizes, not this stage.
- Expected timing: E6 classification from ~2.1 s to the class-count
  scale (sub-10 ms); B5/C5/D5 similar collapses.
- Tests hardcoding `CartanId` values are revalidated: A1 keeps 0/1;
  multi-Cartan groups (Sp(4): 1/4/11 gate) recheck against the oracle
  through the differential battery.

## Tests and gate

- Transition test: old-vs-new class SETS equal (as sets of orbit
  members via the closure BFS) on A1, A2 (twisted + untwisted), B2,
  A1xA1; then the old path retires from the partition entry point.
- Orbit-size cross-check: per class, closure-BFS size == quotient
  formula, on every group in the local battery (this is the
  independent check — BFS and quotient are unrelated computations).
- `canonicalize` unit tests: idempotent; constant on hand-conjugated
  pairs; identity fixed; A2-twisted and B2 classes land on distinct
  canonical forms.
- The full local suite green; clippy 1.90 AND 1.96 clean; fmt.
- The HPC differential battery (17 groups + E7 added) all-MATCH with
  timings recorded; corpus scoreboard unchanged or improved.

## Three independent design checks

(1) Atlas semantics — discovery/canonicalize/order vs upstream,
quotient-formula inputs (esp. makeSimpleComplex pair-keeping);
(2) Rust internals — canonicalize termination, descent-loop bounds,
Integer arithmetic, borrow structure of the worklist;
(3) API and consumer fit — budget reinterpretation, CartanId order
change blast radius, partition-type API stability, transition test
adequacy. Corrections fold here before implementation.

# KGB stage map

Oracle trace of the upstream KGB construction (master `4d3e9449`:
kgb.cpp, tits.cpp/h, innerclass.cpp, involutions.cpp), recorded as the
plan of stages. Each stage follows the standard cadence: design, three
reviews, implementation, tests, preflight.

## Element representation and operations (upstream facts)

A KGB element is a `TitsElt`: torus part `t` (an `X_* / 2X_*` vector at
FULL rank, stored left) times the canonical braid-lift `sigma_w`
(tits.h:317-334). The Tits group tables hold `alpha_s mod 2`,
`m_alpha_s = alpha_s_vee mod 2`, and the mod-2 twist matrices
(tits.h:441-468). Core primitive `reflect(x, s)`: add `m_alpha_s` when
`<alpha_s mod 2, x> = 1` (tits.h:523-526); `push_across`/`pull_across`
apply it along a reduced word left-to-right / right-to-left
(tits.cpp:434-451). They depend only on the group element — the defining
equation is in `w` itself and `reflect` is conjugation by `sigma_s` in an
honest mod-2 W-representation (tits.cpp:425-432, tits.h:516-526) — so
ANY word for `w` serves, reduced or not.
Sigma multiplications correct by `m_alpha` on length change:
`sigma_mult` on decrease, `sigma_inv_mult` on increase, `mult_sigma` /
`mult_sigma_inv` mirrored on the right (tits.cpp:469-503). The based
group (`TitsCoset`) adds a `grading_offset` over simple roots;
noncompactness test `offset[s] XOR <alpha_s mod 2, t>` (tits.h:785-789);
KGB cross action = twisted conjugation plus `m_alpha` when the offset bit
is set (tits.h:810-816); Cayley transform at noncompact imaginary `s` is
plain `sigma_mult` (tits.h:717-719); inverse Cayley adds a mod-space
repair vector when the grading is compact (tits.cpp:605-644).

Seed `x0`: the square-class cocharacter (`some_coch`,
innerclass.cpp:966-977) elects a base grading; the torus part solves the
binary grading-shift system against the strong-real representative's
compact bits and is standardized by minimizing over the central fiber
(`x0_torus_part`, innerclass.cpp:1070-1095, 986-1055).

Generation (kgb.cpp:489-683): BFS from `x0` over cross actions and
Cayley transforms with torus parts REDUCED modulo the involution's
mod-space (the mod-2 image of the `-1`-eigenlattice, tits.cpp:834-838);
dedup by (involution, reduced torus part); statuses per generator
(Complex / ImaginaryCompact / Real / ImaginaryNoncompact); final stable
sort by (involution length, Weyl length, transducer tie-break) with
packet boundaries; the size must equal the strong layer's
`kgb_size(form)` (assert kgb.cpp:616 — a checked error here).

## Stages

- (a) **Weyl element substrate** (`WEYL_ELEMENT_DESIGN.md`): word-level
  elements on the root-permutation representation — length, descents,
  multiplication with length change, reduced words by descent peeling,
  twisted conjugation. Pure Coxeter; drops the transducer tie-break into
  the adapter deferral.
- (b) **Involution table** (`INVOLUTION_TABLE_DESIGN.md`, LANDED as
  `involution_table.rs`): twisted involutions with theta, root
  classification, mod-space, involution length `(W_length + #Cayley)/2`,
  and cross propagation (involutions.cpp:186-258, 362-379) — one
  canonical entry path (no transported fields; `M_real`/`lift_mat`
  deferred to the parameter layer), per-Cartan contiguous slices, O(1)
  stored cross links, and the Cayley edge. Cross-checked: per-Cartan
  orbit sizes against the classification.
- (c) **Torus parts and Tits operations** (`TITS_OPERATIONS_DESIGN.md`,
  LANDED as `tits_element.rs`): `TitsElement` as `(InvolutionId, mod-2
  bits)` from the start; the based cross action as one closed-form map
  (verified bit-for-bit against the four sigma multiplications), Cayley
  with target-side reduction, blind `simple_grading`,
  conjugate-to-simple general gradings, and the public `reduce` normal
  form. Word walks replaced by mod-2 matrix transport from the
  stage-(b) records' `WeylAction`s. Inverse Cayley deferred beyond
  stage (f) with its repair invariance recorded.
- (d) **Seed x0** (`SEED_X0_DESIGN.md`, LANDED as `real_form_seed.rs`
  plus the `adapted_basis` reduction in `integer_lattice.rs`):
  `RealFormSeed::build` — some_coch via the elected `stable_log` (exact
  +1-eigenspace output, integrality gates), the parity offset, the
  grading-shift solve over the DELTA-FIXED simples, and the
  central-fiber re-walk minimizing the shifted value. The ordering
  audit proved the crate's weak-real-form/square-class numbering EQUALS
  upstream's internal one; the interpreter's external FormNumberMap
  order is a recorded language-adapter obligation.
- (e) **KGB generation** (`KGB_GENERATION_DESIGN.md`, LANDED as
  `kgb_graph.rs`): the per-form BFS with reduce-and-dedup, statuses
  classified from the table, the counting-sort standardization with
  the crate tie-break, inverse-Cayley pairs by the ascending
  post-pass, tau packets, and exact `torus_factor` rationals. Sizes
  verified against `kgb_size` and the oracle binary: SL(2,R)=3
  (type-I pair), PGL(2,R)=2 (type II), Sp(4,R)=11 with per-length
  (4,3,3,1), SU(2,1)=6 (equal-rank A2), compact forms=1.
- (f) **Bruhat data**: descent-set consumption and the
  Richardson-Springer Hasse construction when blocks need it (inverse
  Cayley and tau packets landed with stage (e), faithful to
  upstream's constructor).

Deferred indefinitely: global KGB fingerprints, `EnrichedTitsGroup`
backtrack seeding (partial KGB only), the external distinguished twist.

## Observables

Directly differential-comparable: KGB size, per-length counts, status
multisets, and per-Cartan packet sizes AS MULTISETS keyed by involution
(the tie-break reorders packets within equal-length groups, so the raw
packet sequence is not comparable). `torus_factor` (kgb.cpp:705-712)
and raw torus-part bit patterns are comparable by CONVENTION
COINCIDENCE, not convention independence (stage-(c) review): both
implementations canonicalize by the same low-pivot RREF over the same
X_* coordinates, and the harness must pin that coincidence with a
fixture test; the convention-independent content is only the class
modulo the integral lattice `(1+theta^T)X_*` (the torus element), so
if either convention ever changes, comparison drops to that class.
Adapter-dependent: element numbering (on the full-KGB path the sort
tie-break at involutions.cpp:427 is the only transducer leak; the
deferred partial/global seeding paths additionally leak `leftDescent`'s
internal order through `involution_expr`), cross/Cayley tables as index
arrays, packet boundaries.

## Scale

Element counts are known exactly beforehand (`kgb_size`); split E8 is
320,206. Per-element work is rank times short word operations; KGB
elements store an involution INDEX plus a torus part, so full
root-permutation elements live only in the involution table, bounded by
the involution count, not the element count. Budget knobs: reserve from
`kgb_size` and hard-fail on mismatch; involution-table cap from
`numInvolutions`; optional length-truncated generation for tests.

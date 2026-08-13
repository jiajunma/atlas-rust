# Remaining builtin coverage (post-language-gate)

## Signature-level reconciliation (2026-08-13)

### P0 status and Rep_table blocker (2026-08-13)

P0 now has the exact `from_dominant`, `Cartan_info`, `KL_block`, and
`KL_column` types plus the `(int,Param)` `cross`/`Cayley` overloads.  The
parameter transforms must use `IntegralSubsystem`/`CommonContext`: counting
integral ambient simple roots is wrong when the integral subsystem has a
non-simple parent root (the B2 `[3,1]/2` case is the regression anchor).
`KL_block` validates standardness before its no-value gate; `KL_column`
validates standardness then finality; `from_dominant` validates only rank in a
no-value context; `Cartan_info` skips its computation there.

The accepted P0 fixture still differs at exactly one sequence-sensitive
observation: after `KL_block(p)`, upstream `KL_column(p)` reports raw block row
`1`, while a fresh Rust partial block reports row `0`.  Do not restore the
rejected exact-`StandardRepr` seed cache.  Upstream `Rep_table` keys a block
family by `Reduced_param = (transformed x, integral-system id,
codec(gamma_lambda) mod Smith diagonal)`, stores a locator/block modifier,
swallows related partial blocks when a full block is installed, and lets every
full-block materializer affect later `lookup`.  The fix therefore belongs in a
shared per-`RealFormContext` block pool with faithful reduced keys and relative
locators, not in individual builtin callers.  Required sequence fixtures are
`KL_block -> KL_column`, `print_common_block -> KL_column`, a related parameter
in the same family, and a changed-gamma/block-modifier case.

The first faithful implementation slice may support the full-integral,
identity-locator domain, but it must already use an explicit
`ReducedParamKey { x, integral_system: Full, residue }`. Compute `residue`
from the transported `RealProjection::lift_mat` and the Smith-diagonal codec,
using Euclidean remainders and upstream's wrapping `u32` mixed-radix packing.
Register every representative of a materialized block through `co_reduce`
(reverse insertion so the smallest row wins); registering only the queried
seed is still the rejected seed cache in disguise.

The complete pool belongs to the shared real-form value and needs stable block
IDs, reduced-key `Place`s, locators/modifiers, and partial/full promotion. Keep
the mutex structural: reduce/probe under a short lock, materialize block/KL
data outside it, then re-probe and atomically commit. Full materializers
include `print_common_block`, `block(Param)`, `block_Hasse`, `KL_block`,
`dual_KL_block`, Param `W_graph`/`W_cells`, and `KL_sum_at_s_to_height`;
partial consumers include `length`, partial block/KL operations, `KL_column`,
`KL_sum_at_s`, and deformation recursion. `print_block(Param)`,
`print_partial_block`, and standalone external-delta extended computations do
not install entries. Tests must also prove same-context default-form sharing,
custom-form isolation, and isolation between separate `TypedContext`s.

### P2 Block W-graph status (2026-08-13)

`W_graph(Block)` and `W_cells(Block)` are implemented and locally match the
HPC-captured A1 values, rejected calls, and upstream's observable no-value
assignment bug (the graph is still built before its value is dropped).
`block(Param)` remains deliberately unregistered.  A tempting implementation
through `common_block_srms`/the classic Block graph is not a compatible
subset: the legal A1 parameter `param(KGB(rf,2),[1],[1]/2)` triggers Rust's
height-parity invariant although upstream returns a one-element common block,
and a non-standard parameter must fail the upstream `test_standard` gate even
when the result is discarded.  Exact completion is part of the shared
RepTable/ReducedParam work above, not a builtin-local approximation.

The P2 pipeline plans therefore run the Block graph/cell events and declare
both the accepted `block(Param)` event and the affected `block(RealForm)`
overload-rejection event pending.  Removing one overload changes Atlas's
candidate-set diagnostic, so rejected fixtures must be rechecked whenever a
signature is temporarily withheld; preserving only the runtime call lines is
not sufficient evidence of language compatibility.

The name-level closure below was insufficient: upstream `atlas-types.w`
registers **305 distinct `(name, argument type, result type)` signatures**
across 187 names. A fresh comparison against the Rust registry found:

- 277 exact signature matches;
- 28 exact missing/mismatched signatures (23 missing argument signatures,
  5 result-type mismatches);
- 23 signatures grouped into 16 small wrapper/registration tasks;
- 12 registered signatures with an explicit reachable unsupported branch;
- 6 signatures whose completion depends on larger deformation/common-block
  algorithms.

The simple queue is, in order: `from_dominant` (two vec overloads),
`Cartan_info` result type, `KL_block` result order, `KL_column` row result,
`cross`/`Cayley` on a simple-root index and `Param`, Weyl `#`/`##`,
`Cartan_class(KGBElt)`, unary/list polynomial operations for KTypePol and
ParamPol, `block(Param)`, Block `W_graph`/`W_cells`, and Param `twist`
overloads. Larger work includes arbitrary-root Param transforms,
proper-integral common blocks, non-integral W-graphs/ext-KL, alcove shrinking,
and timed deformation/cancellation semantics.

There is no KL-file or GNU-readline builtin in these 305 interpreter
registrations. `filekl` is a stand-alone/interface concern; readline is CLI
infrastructure (the separate `global.w` helper `readline_completions` is not
part of `atlas-types.w`). Completion claims must therefore use the 305
signature ledger, not unique builtin-name counts.

## Batch status (2026-08-12 late — full reconciliation)

Reconciliation vs upstream atlas-types.w (187 unique install_function
names — count via a multiline-tolerant scan; an earlier strict regex
undercounted at 152): **170+ live in typed.rs, 10 never registered.
A first-pass "3 skip-only + 12 partial-skip arms" finding was retracted
after empirical probing — those typed.rs skip registrations are dead
code shadowed by live arms** (see below; operator names like `!= # ##
% * + - / =` are live via the operator layer;
`classify_involution`/`element`/`index` are live via
`domain_builtin_validate`, which name-based scans must treat as a real
registration).

Never registered:
- E2: scale_extended, K_type_pol_extended, finalize_extended
- E3: twisted_deform, twisted_full_deform, twisted_KL_sum_at_s,
  block_deform
- shift_flip — **LANDED 2026-08-12 (`46963fd`)**, differential 3541888
  in flight
- print_partial_block, print_partial_common_block (upstream installs
  them; fixture + reference captured, language layer pending — brief
  /tmp/slice_ppb_brief.md)

**NDEBUG assert parity lesson (2026-08-12, `f668589`)**: upstream
`assert`s (e.g. ext_block.h:356 `(1+theta_x)*shift==0`,
ext_block.cpp:938 `same_standard_reps` in same_sign) are compiled out
in the oracle (-DNDEBUG). The shift_flip wrapper reaches both with
violating inputs; as Rust debug_asserts they panicked where the oracle
returns `false`. When porting, omit any upstream assert that the
wrapper layer can reach with violating inputs, with a comment citing
NDEBUG parity.

Skip-only / partial-signature skips — **retracted (2026-08-12
empirical)**: probing every upstream signature on the committed tree
(dafdc03) shows the "skip" arms in typed.rs are dead registrations
shadowed by live ones. dual(RootDatum), inner_class(RealForm),
involution(KGBElt/CartanClass), twist(KGBElt), twist(KGBElt,mat),
K_type(Param), param(KType), real_form(Param/KType), dual(Block),
`#`(Block), KL_block(Param), dual_KL(Block — the only upstream
signature, atlas-types.w:9102), KL_sum_at_s_to_height(Param,int) ALL
evaluate correctly. There is no skip-arm tail. (These conversion arms
are live but several lack dedicated fixtures — a coverage gap, not an
implementation gap.)

So the entire remaining builtin surface is the 10 never-registered
names above.

## Batch status (2026-08-12)

Landed and verified_hpc since 08-11: ext three-builtin registration
(extended_block/raw_ext_KL/partial_extended_KL_block, differential
3537192), slice A (coroot_queries sweep + root numbering family,
differential 3537366). Slice B (orbit/ladder + poly surface) committed
`53872bb`, differential 3538136 in flight. Crate additions:
RootSystem min_roots_for/min_coroots_for + bourbaki_permutation
(`57049ca`), global_KGB + print_X layout (`64048ac`),
BlockDescent::dual + BlockGraph::dual (`1e7fcc4`).

New known gap (slice B, agent-43): W_graph/W_cells on a parameter with
non-integral gamma now builds the common block on the gamma-integral
root system (lookup_full_block semantics), but imaginary
compact/noncompact grading for that case is NOT ported — the arm
raises a loud "not yet supported" error instead of silently producing
wrong values. No current fixture covers non-integral gamma with
imaginary roots in the integral system; add coverage when the grading
port lands.

Slice E recon (agent-45, /tmp/slice_e_brief.md) revised the
ext_param+star estimate upward: the whole ext_param layer including
star (ext_block.cpp:990-2280, ~1300 lines C++) is missing from the
crate — estimated 1400-1800 lines of Rust. Split plan: E1 crate
ext_param+star core → E2 finalise three-piece set (needs new fixtures
+ HPC probe capture first) → E3 twisted family + block_deform
(fixtures already verified_hpc_reference). Correction: dual_KL is NOT
unlocked by BlockGraph::dual — upstream raw_dual_KL_wrapper uses a
block with swapped real forms + dual_map.

## Batch status (2026-08-11)

Differential `3533446` PASS (199 fixtures: 198 PASS, 1 known PARTIAL, 0
FAIL); the five Weyl/B2 metas upgraded to verified_hpc (`7a5eba5`) — all
harness fixtures now verified_hpc. ext_kl crate slice landed (`602fce6`):
DescentTable + ExtKlTable (KL_table) + condense + ext_kl_matrix with
A2-trivial/A2-flip/Sp4 oracle anchors. Crate side of the ext family is
now complete (ext_block `28e6109` + ext_kl); language registration of
extended_block/raw_ext_KL/partial_extended_KL_block is the next slice
once agent-30's alcove/FPP slice frees atlas-core. Per-slice recon for
ALL 50 missing builtins is now complete in
docs/slices/post_weyl_lang_queue.md (§3 ext registration, §4 print
family + shift_flip dependency correction, §5.1-5.5 ladder/orbit/
small-sweep/deform anchors): 8 near-flips (skip arms already shared
with live siblings + semisimple_rank + reducibility_points), the rest
mapped to concrete crate/language gaps.

Reconciliation vs upstream atlas-types.w (178 install_function names):
128 live in typed.rs, 50 missing (28 never registered + 22
skip-placeholder only; several skip names have main overloads live and
lack only partial signatures).

Remaining: alcove_center/alcove_root_vertex/FPP_numers/FPP_w_shifts (in
flight, agent-30), extended_block/raw_ext_KL/partial_extended_KL_block
(crate ready, wrappers atlas-types.w:7366-7431/8682-8728/7445-7468),
shift_flip (NOT cheap: needs per-parameter shifted_default_extension —
belongs to the ext_param+star slice, post_weyl_lang_queue.md §4),
ext_param+star (largest single block ~1000-1200 lines), finalise three
(finalize_extended/K_type_pol_extended/scale_extended), affine_orbit_ws/
basic_orbit_ws, root_ladder_bottoms/coroot_ladder_bottoms,
root_expression/root_index/coroot_expression/coroot_index/
root_permutation/root_involution (oracle root numbering blocked),
twisted_deform/twisted_full_deform/twisted_KL_sum_at_s/block_deform/
dual_KL_block (+KL_block/dual_KL/KL_sum_at_s_to_height/
truncate_above_height partial signatures, common-block srm pool),
print_gradings/print_real_Weyl/print_blockstabilizer (RealWeyl crate
**已移植** `51b9d83`：real_weyl.rs 1858 行含 10 个字节级锚点测试；坑：
对偶侧必须用精确 `-θ` fiber 链——取 primal 代表元 canonical word 在
对偶 datum 重放后右乘对偶最长元，不能用对偶 classification 的
canonical 代表元，cartanclass.cpp:121；尚缺语言层 wrapper 注册), print_X (GlobalTitsGroup 600+), print_common_block/
print_block(Param)/print_common_block (srm pool, last; print_partial_* 钉住版未安装),
reducibility_points, KType/Rep skips (K_type_pol/first_term/last_term/
null_module/W_cells), small items (semisimple_rank/two_rho_check/
simple_coroots/poscoroots/coroot_radical/mod_central_torus_info/adjoint).

## Batch status (2026-08-09)

Weyl layer landed (`9111b7d`, agent-30): walls/walls_attitude
(alcoves.cpp:112-236), Weyl_orbit/Weyl_orbit_ws both argument orders
(rootdata.cpp:1690-1876), from_dominant corrected (lattice_rank torus
pass-through, true simple-root pairings). RootNumbering keys on coroot
level/coordinates when the datum prefers coroots (rootdata.cpp:164-167).
Fixtures weyl_orbit(+_rejected)/walls(+_rejected) frozen from the local
pinned oracle; HPC differential pending.
**B2 block_sizes root cause fixed**: fiberSize is the STRONG-real fiber
orbit class size (innerclass.cpp:603-614), not the adjoint weak partition;
`fiber_size` switched to `StrongRealClassification::fiber_size`, B2 rows
restored in the block_sizes fixture (oracle 4/5/12 now reproduced).
Known gap: Weyl_orbit/Weyl_orbit_ws oversize-vector semantics (wrapper
does no size check; v.size()!=rank output diverges from the oracle,
details in docs/slices/post_weyl_lang_queue.md §1.5).

Remaining (unchanged): alcove_center/alcove_root_vertex,
FPP_numers/FPP_w_shifts, root_expression/root_index/root_permutation
(oracle root numbering), root_ladder_bottoms/coroot_ladder_bottoms, the
ext_block builtins (extended_block/raw_ext_KL/partial_extended_KL_block —
crate side landed 28e6109 + ext_kl in flight; shift_flip;
finalize_extended/K_type_pol_extended/scale_extended; dual_KL_block),
block_deform series (block_deform/twisted_deform/twisted_full_deform/
KL_block/twisted_KL_sum_at_s), and the print family (print_X/
print_gradings/print_real_Weyl/print_blockstabilizer/print_common_block).

## Batch status (2026-08-06, updated 02:50)

Second sweep round: 56 more skip-registrations live-ized (arms already
implemented) — Cartan_* family, integrality family, simple_roots/
simple_factors/simply_connected/adjoint/derived_info/fundamental_*/
is_Cartan_matrix, Smith_Cartan, posroots/nr_of_*/prefers_coroots,
occurrence_matrix, partial_block, raw_KL, two_rho, strong_components,
normal/theta_stable/to_canonical_fiber/dominant, torus_*, dual_real_form(s).
All 82 non-rejected domain fixtures diff clean; the 42 rejected fixtures
differ only in the known L1 Runtime-error line format. Reverted (kept
skip) where arms were partial: first_term, K_type_pol, truncate_above_height,
KL_block (common-block/PolP gaps). `integrality_datum` now keeps the full
lattice (A1.T1 at half-integral) with SC/Other isogeny. HPC differential
`3520179` running (cargo offline + synced cache/index).

## Batch status (2026-08-06)

Overnight sweep (00:40-01:05 local) landed ~25 more builtins, all
VERBATIM against the oracle on A2/B2/G2/A3/A1A1 probes:

- `cofolded` (InnerClass->RootDatum): fold_orbits + cofold via
  `RootInvolutionData::image_permutation`; B2 identity, A2/G2/A3 split
  (A1.T1), and the orthogonal A1A1 two-type pair all byte-identical.
- KType predicates: `height`, `is_standard`, `is_dominant`, `is_zero`,
  `is_final`, `is_semifinal`, `dominant`, `to_canonical_fiber` (live
  registrations; the dominant/normal/theta_stable/to_canonical_fiber
  transform arm already existed).
- Param predicates: `height`, `is_standard`, `is_dominant`, `is_zero`,
  `is_final`, `is_semifinal` (StandardRepr methods 2500-2603).
- `dual_datum` (InnerClass->RootDatum, G->dual_datum),
  `quasisplit_form`/`dual_quasisplit_form` (InnerClass->RealForm via
  build_real_form + quasisplit_external).
- `dual` overloads (RootDatum->RootDatum rd->dual(), InnerClass->InnerClass
  G->dual(), Block->Block) — the RootDatum arm uses `dual::dual_datum`
  (now `pub`).
- `form_names`/`dual_form_names` (InnerClass->[string] via
  RealFormPresentation::name), `form_number`, `distinguished_involution`.
- `root_datum` InnerClass coercion (G->datum), `central_fiber`
  (strong_real::central_fiber -> [vec]), `KGB_size`.
- `cross` (int, Param -> Param): repr.cpp:891-910 port (made_dominant +
  gamma_lambda - pos_neg real-root correction + simple reflection +
  sr_gamma). `Cayley` (int, Param -> Param): repr.cpp:943-1002 port
  (ImaginaryNoncompact raise with parity/rho_r corrections, or real
  inverse-Cayley with parity gate; Cayley_error passes the input
  parameter back unchanged).
- Live registrations for `rank` (RootDatum/LieType), `length`
  (KGBElt), `orientation_nr` (Param) — arms already existed.

Remaining (unchanged): walls/walls_attitude, Weyl_orbit family,
alcove_center/alcove_root_vertex, FPP_numers/FPP_w_shifts,
root_expression/root_index/root_permutation (oracle root numbering),
root_ladder_bottoms/coroot_ladder_bottoms (root_perm/link), then the
ext_block layer (extended_block/finalize_extended/partial_extended_KL_block/
dual_KL_block/K_type_pol_extended/scale_extended/raw_ext_KL/shift_flip),
block_deform series (block_deform/twisted_deform/twisted_full_deform/
KL_block/twisted_KL_sum_at_s), and the print family (print_X/
print_gradings/print_real_Weyl/print_blockstabilizer/print_common_block).


The language gate is complete (166/166 frozen fixtures verified_hpc).
The upstream interpreter registers 132 distinct builtin names; the Rust
typed layer registers 102. This ledger tracks the 50 missing names in
implementation batches. Each batch follows the per-slice loop: probe
the oracle (local `/Users/hoxide/mycodes/atlasofliegroups/atlas` works),
freeze a fixture, implement, gate, HPC differential, meta upgrade.

## Batch status (2026-08-05)

| Batch | scope | status |
|---|---|---|
| 1 | root-datum surface | DONE (simple_roots/simple_coroots/is_Cartan_matrix/dual_datum, two_rho, fundamental_weight/coweight, simple_factors, Cartan_matrix_type) |
| 3 | root/radical data | DONE except root_ladder_bottoms/coroot_ladder_bottoms (need root_perm/link); integrality_rank/integrality_datum/is_integrally_dominant DONE `174ae58` (fixture `domain/integrality` VERBATIM; integrality_points implemented but its RatVec-list display differs from the oracle RatNum list — recorded in meta) |
| 4 | print family | PARTIAL (RealWeyl crate ported `51b9d83`; still needs global KGB for print_X, srm pools for print_common_block, language-layer wrappers for the rest) |
| 5 | W-cells/KL | DONE except twisted_KL_sum_at_s (needs ext_block) |
| 6 | extended blocks | PARTIAL (default_extended/extend/partial_block/partial_KL_block done; rest need ext_block layer) |
| 7 | deform variants | PARTIAL (full_deform done; rest need block_deformation_to_height / common-block srm pool) |
| 8 | misc | DONE except shift_flip (needs ext_block); Cartan_matrix_type done |

Remaining (recorded): walls/walls_attitude (weyl::wall_set), from_dominant (WeylElt decompose),
derived_info / mod_central_torus_info (PreRootDatum projector), cofolded (construct_cofolded),
Weyl_orbit family, alcove_center/alcove_root_vertex, FPP_numers/FPP_w_shifts, root_expression/
root_index/root_permutation (oracle root numbering), then the ext_block / print / block_deform
layers. Performance work (2026-08-04/05) is in docs/BENCHMARKS.md: E6 13.7s->0.45s warm, E7 10.3s/4.1GB
->8.4s/2.2GB via rho-descent longest, compact [u8;8] WeylElt, u8 root permutations, full-content
classification cache, rayon parallelization (7 sites).

## Batch status (2026-08-01)

| Batch | scope | status |
|---|---|---|
| 1 | root-datum surface: simple_roots, simple_coroots, is_Cartan_matrix, dual_datum(InnerClass) | DONE `4857d2a`, fixture `domain/simple_roots` VERBATIM |
| 2 | KGB Bruhat printers: print_KGB_order, print_KGB_graph (KgbGraph::bruhat_hasse, n_bruhat_comparable) | DONE `0894ccf`, fixture `domain/kgb_bruhat` VERBATIM |
| 3 | root/radical data | DONE: root_coradical, coroot_radical (`domain/radical`), components_rank, strong_components (`domain/components_rank`), two_rho/two_rho_check (`domain/two_rho`, HPC `3507991`) all VERBATIM + HPC. Only root_ladder_bottoms / coroot_ladder_bottoms remain (they need the root_perm/link permutations of rootdata.cpp:243-313 that RootTable does not store). |
| 4 | print family: print_X, print_gradings, print_real_Weyl, print_blockstabilizer, print_common_block | NOT STARTED — print_X (KGB global), print_gradings (Cartan grading bits + Bourbaki numbering of the imaginary subsystem), print_real_Weyl (real Weyl group), print_blockstabilizer / print_common_block (common-block stabilizer) all need deeper layers (global KGB, realweyl, srm pools); print_gradings additionally needs the oracle's root numbering for the simple-root listing. |
| 5 | W-cells and KL access: W_cells, W_graph, KL_column, raw_KL, raw_ext_KL, dual_KL, KL_sum_at_s, KL_sum_at_s_to_height, twisted_KL_sum_at_s | DONE EXCEPT twisted_KL_sum_at_s: W_cells/W_graph(Param) (`domain/w_graph_param`), raw_KL/dual_KL (`domain/raw_kl`), KL_sum_at_s/_to_height (`domain/kl_sum_at_s`), KL_column (`domain/kl_column`, HPC `3508248`) all VERBATIM + HPC. The KL-table Cayley argument-order fix (`24ba188`) unlocked B2/G2 KL (HPC `3508004`); the multi-bit grading-shift fix (`fbed749`) unlocked A3+ dual real forms (raw_kl covers A2/B2/G2/A3/D4, HPC `3508109`; w_graph_param/kl_sum_at_s cover A3, HPC `3508132`) — all 0 FAIL. twisted_KL_sum_at_s needs ext_block. Known: KL_sum_at_s uses the input parameter's lambda-rho for every block element (height-parity mismatch for mid-block parameters; fixtures use the block's lowest element). |
| 6 | extended blocks: default_extended, extend, extended_block, finalize_extended, partial_block, partial_KL_block, partial_extended_KL_block, dual_KL_block, K_type_pol_extended, scale_extended | PARTIAL: **default_extended** COMPLETE (`fab1593`+`6855ca2`) — the 4-tuple (lambda, tau, l, t) via the srm gamma-lambda unique mod X* (real_unique) + ell, with the generic twist solved by matreduc::find_solution (exact rational Gaussian elimination); A2 identity + A3 non-identity byte-identical; **extend** (`9b0abbb`); **partial_block** (`domain/partial_block`, HPC `3511402`); partial_KL_block (HPC `3511377`); the rest need the ext_block layer. |
| 7 | deform variants: twisted_deform, twisted_full_deform, block_deform, full_deform, KL_block | PARTIAL: **full_deform** (`domain/full_deform`, HPC verified) — finals_for + reducibility-point recursion; the rest need block_deformation_to_height (repr.cpp:2027-2124, the partial-block deform recursion) and/or the common-block srm pool (KL_block needs lookup_full_block + survivors condensation). |
| 8 | misc: Cartan_info, KGB_Hasse, block_Hasse, orientation_nr, shift_flip | DONE except shift_flip: Cartan_info (`domain/cartan_info`), KGB_Hasse (`domain/kgb_hasse`), block_Hasse (`domain/block_hasse`), orientation_nr (`domain/orientation_nr`) all VERBATIM + HPC. shift_flip needs the ext_block layer (Batch 6). |

## Oracle-probed shapes (A2)

- `simple_roots(simply_connected A2)` → `| 2, -1 | / | -1, 2 |`; `simple_coroots` → identity
- `is_Cartan_matrix([[2,-1],[-1,2]])` → true; identity → false
- `dual_datum(ic)` → `adjoint root datum of Lie type 'A2'`
- `print_KGB_order(rf)` → kgbsize + Hasse rows + comparable-pair count
- `print_KGB_graph(rf)` → Graphviz digraph with black/blue/green/gray edges
- `root_coradical(simply_connected A2)` → Cartan rows (coradical empty); `coroot_radical` → identity
- `root_coradical(adjoint A2)` → identity; `coroot_radical` → Cartan rows
- `root_ladder_bottoms(ra, 0)` → `[-3,-1,0,1]` on A2
- `Cartan_info(CartanClass)` → `((2,0,0),[ ],(1,4),(A2,empty,empty))` on A2 — the first triple is
  `classify_involution` (already ported, identity A2 → (2,0,0) verified)

## E6/D5 column-echelon — RESOLVED (2026-08-04)

The `RealProjection::build` port is fixed. Root cause: the incremental
column-echelon port is not equivalent to C++'s one-shot `column_apply`.
The fix (commit 248aeb9) combines:
1. one-shot ops-matrix sweeps with `ops(mindex,mindex)=-1` recorded
   (matreduc.h:70-122 + column_apply);
2. Euclidean row-reduction inverse of the unimodular `col` matrix
   (no scaling division);
3. truncating division in `lambda_unique` (match `arithmetic::divide`:
   `divide(-1,2)==0`, which is what the A2 su(2,1) anchors require —
   the earlier A2/E6 "contradiction" was a div_euclid-vs-trunc artifact).
E6 involution 187 and the D5 so*(10) real form now factor
`lift_mat * M_real == 1-theta`; E6/D5 KL_column, deform, raw_KL,
KL_sum_at_s all byte-identical vs the oracle. E7 kgb_hasse verified on
HPC fat (swap 3515688: 506s, 12.4G peak RSS).

## E6 column-echelon debugging notes (2026-08-03, resolved upstream)

The `RealProjection::build` port of `matreduc::column_echelon` fails its
`lift_mat * M_real == 1-theta` check for E6's involution 187. The
investigation produced these verified facts (all in Python reproductions
and Rust experiments):

1. The original incremental port (`column_operation` mutating `a`
   directly) is NOT equivalent to C++'s `column_apply(M, ops)` one-shot
   semantics — the E6 factorization only holds with the one-shot ops.
2. With one-shot ops, E6 involution 187 needs BOTH the local-pivot
   flip (`row[mindex] = -row[mindex]`) AND `ops(mindex,mindex) = -1`
   recorded: `flip+record` -> zero_columns=4 check=True;
   `flip+no-record` -> zero_columns=2 check=False.
3. `col` is unimodular but the plain Gauss-Jordan inverse with scaling
   division breaks on non-±1 pivots; the Euclidean row-reduction inverse
   (row swaps + subtractions only) is the working variant.
4. CONTRADICTION: the A2 su(2,1) anchor `K_type(x4,[1,0])` gives
   lambda_rho [1,0] in the oracle (== the no-record variant), while E6
   needs the record variant. The same C++ code cannot produce both under
   the current simulation — the A2 single-active-column flip+swap case
   must cancel the recorded -1 differently in C++ (matrix.h
   swapColumns/columnApply interplay), which the simulation misses.
   Root-cause understanding of that cancellation is the open task.

Suggested next step: instrument the real C++ `involutions.cpp` build for
A2 x4 (or read `matrix.h`'s PID_Matrix swapColumns/columnApply once more
for hidden sign flips), then reconcile the A2/E6 split.

## D5 column-echelon limit (2026-08-04)

The E6 involution-187 `RealProjection` failure also hits D5: the so*(10)
real form's `KL_sum_at_s` panics on "image basis factorization". The
same root cause (incremental column-echelon port vs C++ one-shot
`column_apply`, see the E6 notes below). `raw_KL` on the D5 block passes,
so the block graph itself is fine — only packet involutions of certain
real forms trip the projection. Verified fixtures must avoid D5/D6+ real
forms until the column-echelon port is reconciled.

## Root-index builtins limit (2026-08-04, detail) — STALE, unblocked 2026-08-11

**2026-08-11 update: this limit is stale.** RootNumbering
(domain_builtins.rs:2809-2880) now ports the oracle order and is
differential-verified on B2 (fixtures 3516408 posroots order
`[1,0],[0,1],[2,1],[1,1]`, 3533446 walls). See
docs/slices/post_weyl_lang_queue.md §5.6. The notes below are kept for
history.

The oracle's B2 positive-root order is [1,0],[0,1],[1,2],[1,1] (probe):
root_expression(rb,2) = [1,2] = alpha_1 + 2 alpha_2, so the oracle's B2
uses the Bourbaki numbering (alpha_1 SHORT), while Rust's standard B2
Cartan [[2,-2],[-1,2]] has alpha_1 LONG. The oracle `ri` order is the
roots_at_level generation order (rootdata.cpp:144-219), which depends on
this numbering; mapping oracle RootNbrs to Rust roots therefore needs the
Bourbaki simple-root renumbering first. That renumbering would touch the
whole RootDatum surface (simple_roots, Cartan_matrix, KGB block orders),
so the root-index family stays unimplemented and fixtures avoid it.

## Root-index builtins limit (2026-08-04) — STALE, unblocked 2026-08-11

See the 2026-08-11 note on the detail section above; kept for history.

`root_expression`/`coroot_expression`/`root_permutation`/`root_involution`
take an oracle RootNbr (internal_root_index: N + numPosRoots, positive
roots only). The Rust `RootSystem::roots()` orders positive roots by
ambient-coordinate lexicographic order, which differs from the oracle's
`ri` (roots_at_level) order — and the oracle's B2 order ([1,0],[0,1],[1,2],
[1,1]) is not the naive height/level order either, so a simple re-sort
does not match. Porting the oracle's level-generation order (rootdata.cpp
:144-219) is the open task; until then the root-index family stays
unimplemented (fixtures avoid them).

## Known structural limit: E6-and-larger Rep_context

The `RealProjection::build` column-echelon port (matreduc.h:129-161,
the `1-theta` image basis) fails its `lift_mat * M_real == 1-theta`
check for E6's involution 187 (packet 74): product 7 vs expected -1 at
entry (0,5). Every smaller rank (A1..A4, B2..B4, C3/C4, G2, F4, D4)
passes; the E6 class-1 real form's KL/deform surface is therefore
unavailable (KGB_Hasse still works — it does not build a Rep_context).
The failure is in the column-echelon port (or its divisor semantics),
not in the KL machinery. Fixing it unlocks E6 KL_sum_at_s / deform /
W_cells and is a 1-2 hour debugging task against upstream matreduc.

## Polynomial term owner identity and integer conversion (2026-08-13)

- The upstream KTypePol/ParamPol term wrappers compare the owning real-form
  `shared_ptr`, not structural real-form equality.  Rust therefore keeps
  canonical/default real forms in one logical owner class, while every
  genuinely custom real-form construction receives a distinct owner token.
  In contrast, `equivalent(KType,KType)` and `equivalent(Param,Param)` compare
  structural real-form values and must accept identical custom constructions.
  Keep this distinction when moving ownership into the future session
  `Rep_table`; `Arc::ptr_eq` alone is also wrong because Rust currently
  rebuilds canonical real-form values.
- `big_int::int_val()` accepts exactly the signed 32-bit range.  Oracle probes
  confirm that positive `2147483648` reports `Integer value to big for
  conversion`; it does not wrap to `-2147483648`.  Preserve the checked `i32`
  conversion for Weyl generator builtins and their no-value validation paths.
- Bulk polynomial term-list addition is a volume-oriented API upstream.  The
  Rust implementation appends expansions, sorts once, and linearly coalesces
  equal terms; do not regress it to repeated linear `Vec::position` merging.

## Parameter twist sentinel contract (2026-08-13)

- The explicit outer twist of an otherwise valid KGB element or parameter can
  produce upstream `UndefKGB`, whose language-visible number is `~0u`, printed
  as `4294967295`.  It is a real observable value, not the same outcome as the
  `Inexistent KGB element` diagnostic.
- Rust models that value explicitly but never treats it as a `KgbGraph` index.
  A parameter sentinel also retains the already transported lambda/nu needed
  for the upstream display.  Ordinary follow-up operations reject the
  sentinel through stable checked paths rather than indexing or panicking.
- Unary `twist(Param)` first calls `make_dominant`; `twist(Param,mat)` validates
  compatibility and twists the parameter exactly as supplied.  Do not merge
  these paths even when the matrix is the distinguished involution.
- Oracle jobs `3543783`, `3543792`, `3543798`, and `3543906` pin the
  nonstandard case, both sentinel constructors, and the safe field-only
  surface. Strict equality, `%`, `height(Param)`, and `real_form(Param)` remain
  valid on the sentinel; only operations that need a graph element reject it.
  The P3 differential must include all six twist fixtures before closure.

## Full-deform outer term merge contract (2026-08-13)

- `full_deform(Param)` accumulates its outer result by KType key. Distinct
  KTypes with equal Split coefficients must both survive; equal KTypes combine
  coefficients and zero sums disappear. The previous coefficient-only merge
  silently discarded valid terms.
- Reference capture `3543807` pins a minimal A2 two-term result and its rank
  rejection. This is only a narrow polynomial-accumulation repair; it does not
  complete proper-subsystem deformation, high-denominator alcove shrinking,
  recursion, cancellation/deadline support, or `full_deform(Param,int)`.

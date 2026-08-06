# Remaining builtin coverage (post-language-gate)

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
| 4 | print family | NOT STARTED (needs global KGB, realweyl, srm pools) |
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

## Root-index builtins limit (2026-08-04, detail)

The oracle's B2 positive-root order is [1,0],[0,1],[1,2],[1,1] (probe):
root_expression(rb,2) = [1,2] = alpha_1 + 2 alpha_2, so the oracle's B2
uses the Bourbaki numbering (alpha_1 SHORT), while Rust's standard B2
Cartan [[2,-2],[-1,2]] has alpha_1 LONG. The oracle `ri` order is the
roots_at_level generation order (rootdata.cpp:144-219), which depends on
this numbering; mapping oracle RootNbrs to Rust roots therefore needs the
Bourbaki simple-root renumbering first. That renumbering would touch the
whole RootDatum surface (simple_roots, Cartan_matrix, KGB block orders),
so the root-index family stays unimplemented and fixtures avoid it.

## Root-index builtins limit (2026-08-04)

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

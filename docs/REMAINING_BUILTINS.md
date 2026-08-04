# Remaining builtin coverage (post-language-gate)

The language gate is complete (166/166 frozen fixtures verified_hpc).
The upstream interpreter registers 132 distinct builtin names; the Rust
typed layer registers 102. This ledger tracks the 50 missing names in
implementation batches. Each batch follows the per-slice loop: probe
the oracle (local `/Users/hoxide/mycodes/atlasofliegroups/atlas` works),
freeze a fixture, implement, gate, HPC differential, meta upgrade.

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

## E6 column-echelon debugging notes (2026-08-03, unresolved)

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

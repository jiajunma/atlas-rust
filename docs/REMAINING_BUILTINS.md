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
| 4 | print family: print_X, print_gradings, print_real_Weyl, print_block, print_blockd, print_blocku, print_blockstabilizer, print_common_block, print_KL_basis, print_prim_KL, print_KL_list, print_W_cells, print_W_graph | PARTIAL: print_block/blockd/blocku (`domain/block_print`), print_KL_basis/prim_KL/list, print_W_graph, print_W_cells (`domain/kl_print` — now covering the A2, B2 split 12-element and G2 blocks after the KL fix, `dfd62ef` cell ordering and `f7bda08` KL_list ordering, HPC `3508049`) — all verbatim. Known: print_block prints the greedy reduced Weyl word (the oracle prints the WeylGroup transducer word, so longer words differ; the A2/B2 fixture rows use words where they coincide). Remaining: print_blockstabilizer/print_common_block need the common-block + realweyl layer; print_X needs the global KGB; print_gradings needs the Cartan fiber; print_real_Weyl needs realweyl |
| 5 | W-cells and KL access: W_cells, W_graph, KL_column, raw_KL, raw_ext_KL, dual_KL, KL_sum_at_s, KL_sum_at_s_to_height, twisted_KL_sum_at_s | DONE EXCEPT twisted_KL_sum_at_s: W_cells/W_graph(Param) (`domain/w_graph_param`), raw_KL/dual_KL (`domain/raw_kl`), KL_sum_at_s/_to_height (`domain/kl_sum_at_s`), KL_column (`domain/kl_column`, HPC `3508248`) all VERBATIM + HPC. The KL-table Cayley argument-order fix (`24ba188`) unlocked B2/G2 KL (HPC `3508004`); the multi-bit grading-shift fix (`fbed749`) unlocked A3+ dual real forms (raw_kl covers A2/B2/G2/A3/D4, HPC `3508109`; w_graph_param/kl_sum_at_s cover A3, HPC `3508132`) — all 0 FAIL. twisted_KL_sum_at_s needs ext_block. Known: KL_sum_at_s uses the input parameter's lambda-rho for every block element (height-parity mismatch for mid-block parameters; fixtures use the block's lowest element). |
| 6 | extended blocks: default_extended, extend, extended_block, finalize_extended, partial_block, partial_KL_block, partial_extended_KL_block, dual_KL_block, K_type_pol_extended, scale_extended | PARTIAL: **partial_block** (`domain/partial_block`, HPC `3511402` submitted) — the partial-block parameter list (KL descent closure + singular survivors); partial_KL_block (HPC `3511377`); the rest need the ext_block layer. |
| 7 | deform variants: twisted_deform, twisted_full_deform, block_deform, full_deform, KL_block | PARTIAL: **full_deform** (`7a5c2a3`, HPC `3511044`) and **KL_block** (`32398d5`, HPC `3511352` submitted) land — full_deform via finals_for + reducibility_points + scale/deform_readjust/deformation_terms (A1/A2/B2/G2/A3 byte-identical); KL_block via the **common-block closure matched by gamma-lambda mod the cocharacter lattice** (`523e647`, also fixes block_Hasse A2 x=3) + singular survives (coroot·numer==0, repr.cpp:526-534) + finals_for condensation (A2 x=0, A1 x=2 byte-identical; singular parameters needing common-block srm descent statuses are a known limit). block_deform needs block_deformation_to_height; the twisted variants need ext_block. |
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

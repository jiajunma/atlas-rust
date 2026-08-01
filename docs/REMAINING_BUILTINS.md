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
| 3 | root/radical data | PARTIAL `843e24a`: root_coradical, coroot_radical done (`domain/radical` VERBATIM, integer_lattice::saturated_kernel). root_ladder_bottoms / coroot_ladder_bottoms need the full root-system machinery (root_perm + link permutations, rootdata.cpp:243-313) which RootTable (atlas-core) does not store; components_rank / strong_components need the real-form component layer. |
| 4 | print family: print_X, print_gradings, print_real_Weyl, print_block, print_blockd, print_blocku, print_blockstabilizer, print_common_block, print_KL_basis, print_prim_KL, print_KL_list, print_W_cells, print_W_graph | TODO — block/real-Weyl/strong-real printers sit on the block + KL data; print_X needs the global KGB |
| 5 | W-cells and KL access: W_cells, W_graph, KL_column, raw_KL, raw_ext_KL, dual_KL, KL_sum_at_s, KL_sum_at_s_to_height, twisted_KL_sum_at_s | TODO — on the KL_table from the deform slice |
| 6 | extended blocks: default_extended, extend, extended_block, finalize_extended, partial_block, partial_KL_block, partial_extended_KL_block, dual_KL_block, K_type_pol_extended, scale_extended | TODO — needs the ext_block layer |
| 7 | deform variants: twisted_deform, twisted_full_deform, block_deform, full_deform, KL_block | TODO — the twisted KLV variant of the deform slice |
| 8 | misc: Cartan_info, KGB_Hasse, block_Hasse, orientation_nr, shift_flip | TODO — Cartan_info needs the WeylWord + orbit/fiber + subsystem classification of a CartanClass |

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

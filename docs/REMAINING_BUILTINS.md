# Remaining builtin coverage (post-language-gate)

The language gate is complete (166/166 frozen fixtures verified_hpc).
The upstream interpreter registers 132 distinct builtin names; the Rust
typed layer registers 96. This ledger tracks the 56 missing names in
implementation batches. Each batch follows the per-slice loop: probe
the oracle (local `/Users/hoxide/mycodes/atlasofliegroups/atlas` works),
freeze a fixture, implement, gate, HPC differential, meta upgrade.

## Batch 1 — root-datum surface (simple)

| builtin | signature | oracle shape (A2) |
|---|---|---|
| `simple_roots` | `(RootDatum->mat)` | rows of simple roots; A2 → `\| 2, -1 \| / \| -1, 2 \|` |
| `simple_coroots` | `(RootDatum->mat)` | rows of simple coroots; A2 → identity |
| `is_Cartan_matrix` | `(mat->bool)` | Cartan-matrix predicate; A2 cartan → `true` |
| `dual_datum` | `(InnerClass->RootDatum)` | the dual root datum; A2 → `adjoint root datum of Lie type 'A2'` |
| `Cartan_info` | `(CartanClass->...)` | `((2,0,0),[ ],(1,4),(Lie type 'A2',...))` on A2 |

## Batch 2 — print/debug family (verbatim printers)

`print_KGB`, `print_KGB_graph`, `print_KGB_order`, `print_X`,
`print_gradings`, `print_real_Weyl`, `print_strong_real`, `print_block`,
`print_blockd`, `print_blocku`, `print_blockstabilizer`,
`print_common_block`, `print_KL_basis`, `print_KL_list`,
`print_prim_KL`, `print_W_cells`, `print_W_graph`.

## Batch 3 — root/radical data

`root_ladder_bottoms`, `coroot_ladder_bottoms`, `root_coradical`,
`coroot_radical`, `components_rank`, `strong_components`.

## Batch 4 — parameter transforms

`cross`/`Cayley` on Param (need the integral SubSystem),
`orientation_nr`.

## Batch 5 — W-cells and KL access

`W_cells`, `W_graph`, `KL_column`, `raw_KL`, `raw_ext_KL`,
`dual_KL`, `KL_sum_at_s`, `KL_sum_at_s_to_height`,
`twisted_KL_sum_at_s`.

## Batch 6 — extended blocks

`default_extended`, `extend`, `extended_block`, `finalize_extended`,
`partial_block`, `partial_KL_block`, `partial_extended_KL_block`,
`dual_KL_block`, `K_type_pol_extended`, `block_Hasse`, `KGB_Hasse`,
`shift_flip`.

## Batch 7 — deform variants

`twisted_deform`, `twisted_full_deform`, `block_deform`,
`full_deform`, `KL_block`.

## Batch 8 — misc

`print_common_block` (also listed above), `root_radical` etc.

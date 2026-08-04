# Benchmark ledger (swap 3516408, HPC fat partition)

Frozen fixture differential, 2026-08-04. Rust = atlas-cli; oracle = frozen
atlas executable on XMU HPC (fat nodes, `--mem=32G`). Wall seconds + peak RSS.

Total fixtures: 188; slow fixtures (>=1s on either side): 14

| fixture | rust s | rust RSS | oracle s | oracle RSS | rust/oracle |
|---|---:|---:|---:|---:|---:|
| w_graph_param | 2795.00 | 178M | 0.113 | 10M | 24734.5x |
| raw_kl | 2110.00 | 181M | 0.077 | 7M | 27402.6x |
| partial_block | 2032.00 | 178M | 0.043 | 6M | 47255.8x |
| kl_column | 2015.00 | 177M | 0.040 | 7M | 50375.0x |
| kl_print | 1527.00 | 179M | 0.038 | 6M | 40184.2x |
| kgb_hasse | 1412.00 | 12089M | 0.000 | 0M | n/a |
| kl_sum_at_s | 676.00 | 74M | 0.044 | 6M | 15363.6x |
| partial_kl_block | 650.00 | 73M | 0.030 | 6M | 21666.7x |
| full_deform | 624.00 | 175M | 0.029 | 6M | 21517.2x |
| cartan_info | 610.00 | 174M | 0.038 | 4M | 16052.6x |
| orientation_nr | 597.00 | 174M | 0.022 | 5M | 27136.4x |
| block_hasse | 41.00 | 10M | 0.034 | 5M | 1205.9x |
| dual_datum | 1.00 | 4M | 0.000 | 0M | n/a |
| involution_decomposition_b2_c2_preference_probe | 1.00 | 4M | 0.000 | 0M | n/a |

Notes: kgb_hasse runs the E7 KGB + Hasse (12G RSS, ~24 min). The KL-family
fixtures now carry E6/D6 rows; the Rust fibred-block KL fill dominates the wall
time there (a performance optimization target, not a correctness issue: every
fixture is byte-identical to the oracle).
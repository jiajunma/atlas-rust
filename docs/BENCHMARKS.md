# Benchmark — Rust vs the real Atlas C++ (fair, same machine)

Method (2026-08-04): identical `.atlas` scripts, one machine (macOS,
Apple Silicon), `target/release/atlas-cli` (cargo release) vs the locally
built Atlas C++ (`-Wall -O3 -DNDEBUG`, `/Users/hoxide/mycodes/atlasofliegroups/atlas`).
Wall time of the whole script, best-of-3.

## Small groups — Rust ≈ C++

| script | Rust | C++ | ratio |
|---|---|---|---|
| A1+A2+B2 W_graph | 0.01s | 0.01s | ~1x |
| A1–A4 W_graph | 0.023s | 0.012s | ~2x |

## Large groups — Rust slower (Weyl-group enumeration)

| script | Rust | C++ | ratio |
|---|---|---|---|
| G2+D6 W_graph | 0.34s | 0.024s | ~14x |
| E6 W_graph | 0.33s (warm) | 0.021s | ~16x |

## Where the time goes (E6, profiled)

- `CartanClassification::build` (twisted-involution classification) runs
  ~4× per fixture (primal, dual, dual-of-dual, block side) and dominates:
  it enumerates the whole Weyl group (51840 for E6) as 6×6 integer
  matrices, deduped in a HashMap, composing matrix products per BFS step.
- The oracle's Weyl layer uses a transducer (parabolic decomposition)
  where `longest()` is O(rank) (weyl.cpp:765) and elements are compact
  piece-words — no full matrix enumeration for classification.
- KL-table fill is NOT a hotspot (E6: ~2ms; it was mis-attributed in the
  earlier HPC ledger, which also measured whole fixtures on shared fat
  nodes, inflating wall times).

## Optimizations landed (this session, byte-identical output)

1. `longest_action` now walks `2rho → -2rho` by positive coroot-pairing
   reflections (O(length) ≈ 36 for E6) instead of enumerating |W|.
2. `WeylAction` carries `Arc<BasedRootDatum>` — compose is a refcount
   bump, not a full datum clone.
3. `compose_matrices` accumulates in i64 without per-entry overflow
   checks (entries are Cartan-bounded).
4. `enumerate_actions` dedups in a HashMap with flattened matrix keys and
   a `compose_fast` hot loop (no shape checks).

E6 W_graph probe: 13.7s → 0.33s warm (-98%). Timeline: rho-descent longest;
Arc datum; i64 compose; compact transducer Weyl layer (weyl_transducer.rs,
group orders + inverse + matrix-equivalence tests); parallel piece-matrix
materialization (rayon); parallel no-alloc action permutations with a
root-coordinate HashMap index; compact twisted-involution enumeration
wired into CartanClassification (orbit sweep needs the FULL enumeration,
not just the candidates — the discovery bug); the orbit sweep's root
permutations compose from the compact enumeration (simple-reflection
perms → per-piece perms → parallel element perms), so only the ~892
twisted involutions materialize matrices; and the orbit conjugation
itself is parallelized per Weyl element; and the dual Cartan
correspondence reuses the classification's stored twisted-conjugacy
partition instead of rebuilding it (a second ~125ms per side); and the
twisted-involution scan over all 51840 compact elements is parallelized
(pure per-element test + candidate matrix build); the KGB BFS runs
two-phase (parallel status/cross/cayley computation, sequential intern);
the involution-key construction (piece words) is parallelized; and the
Cartan classification is cached by FULL datum content (lattice rank +
Cartan + simple roots + coroots — the first attempt keyed only Cartan,
conflating simply-connected/adjoint data) + theta matrices + budget
(0.45s cold -> 0.33s warm on repeated fixtures).
Remaining levers recorded: the KGB post-processing (involution sort +
piece keys + graph fill, ~160ms for the E6 dual) and the structural
memory gap (Vec/Arc nesting vs native arrays).
Memory (max RSS): Rust 69MB vs C++ 7.2MB (E6 W_graph) — 9.6x, down from
44x; the remaining gap is Vec/Arc nested allocation vs native arrays.
Next levers: classification caching (dual builds repeat), KGB build
parallelization.

## Fixture-level HPC ledger (swap 3516408, fat nodes)

The full differential suite is 189 fixtures, 0 FAIL (1 known PARTIAL),
all byte-identical. HPC wall times there include shared-node contention
and whole-fixture multi-row scripts, so they overstate the per-parameter
cost measured here; RSS is meaningful: kgb_hasse (E7) peaks at ~12GB.

## Script-corpus ledger (HPC cpu partition, 240 atlas-scripts, both binaries per script)

Every corpus run records wall time and peak RSS per script for the Rust CLI
and the oracle (`script_corpus_report.json` under results/<commit>/<job>/).
"over_5x" counts scripts where rust_seconds/cpp_seconds > 5.

| job | commit | comparable | over_5x | notes |
|---|---|---|---|---|
| 3615211 | eeee72a~ | 104 | 95 | every script pays ~4-5s constant: eager E8 inner-class orbit construction |
| 3616252 | 6dff4ab | 104 | 97 | orbit-construction perf landed: 174/238 scripts improved >0.5s, median 4.26s -> 3.67s; ~2.8s/script hot spot still unidentified |
| 3617082 | 29651e4 | 229 | 222 | discrimination unblock (132 scripts now run deep enough to compare) |
| 3617285 | c13b06a | 234 | 227 | deeper coverage; echo regression fixed next commit |
| 3617878 | 0ab4baa | 236 | 229 | MATCH 45->93 (echo regression + Levi fixes landed); gdb sampling (GKfast.at, generic_degrees.at) pinned the ~2.8s hot spot: `coercions::same`/`is_close` under `typed::merged_variants` in overload resolution |
| 3617910 | 952b2c7 | 237 | 230 | MATCH 93, OUTPUT_DIFF 144, EVAL_FAIL 1 (gl4H.at ext_block panic, fixed in 7b6bb90); all 144 OUTPUT_DIFFs first-diverge on the SAME line: bracketed set_type echo printed tuple/union arrow sides naked (`(int,int->int)` vs oracle `((int,int)->int)`), fixed in 9a33da9; median rust/cpp ratio 29.5, slowest E8_small_block_cell_parameter_numbers.at 76.8x |
| 3617953 | 9a33da9 | 238 | 192 | MATCH 93->236 (set_type echo fix flipped all 144); remaining OUTPUT_DIFF: example.at (nested closures printed bare head, fixed b28664c), exceptionalData.at ('redefined as', fixed b28664c); fat job 3617912: both ~3MB E8 cell scripts MATCH; overload-variant cache (659df32+f1c5fc5): median 29.5x -> 12.97x, over_5x 230->192, slowest 75.3x |
| 3622339 | fc85095 | 238 | 185 | FULL CORPUS GREEN: MATCH 238/238 (2 SKIPPED_LARGE = the ~3MB E8 cell scripts, fat-verified MATCH in 3617912). Single-HEAD confirmation after all echo/printing fixes. O(1) hidden-builtin name check (24a0d1d): median 12.97x -> 10.25x, over_5x 192->185, within_2x 5, rust_faster 3; slowest E8_small_block_cell_parameter_numbers.at 76.7x, then unipotent_representations_exceptional.at 13.4x (cpp 5.7s / rust 77.2s — the heavy compute outlier) |
| 3622779 | 67ba94d | 3 targeted | — | fat LTO + codegen-units=1 (release profile): unipotent_representations_exceptional.at 77.2s -> 68.0s (-12%), groups.at 0.70 -> 0.66, elliptic.at 0.90 -> 0.86; all MATCH. Modest but free; the malachite inner-loop and per-call KGB rebuild remain the real hot spots (sampling job 3622755) |

Orbit-construction detail (agent-106, jobs 3616233/3616234/3616245):
groups.at load 4.39s -> 0.83s wall (oracle 0.05s); E8 involution partition
1331ms -> 261ms, classification 1758ms -> 270ms (199,952 involutions, 10
Cartan classes unchanged); E7 142ms -> 17.8ms; F4 9ms -> 1.7ms. Method:
permutation-level Cayley BFS + canonicalize (decision-identical to the
matrix path), u128-packed simple-root-image keys (injective: involutions
are linear, simple roots a Z-basis), classification reuses the partition's
BFS order.

Remaining per-script gap is NOT orbit construction. gdb sampling
(jobs 3617887/3617888, GKfast.at + generic_degrees.at) pinned it to
overload resolution: `coercions::same`/`is_close` called from
`typed::merged_variants` under `convert_overload_application`, plus heavy
malloc/free churn (deep type clones). Fix in progress (agent-111).

## Dedicated workload benchmarks (hpc/workloads/*.atlas)

Four self-contained loop workloads (deform_A3, deform_B2, partial_KL_A2,
partial_KL_B3) run through the corpus driver, which records wall time and
peak RSS for both binaries:

    sbatch --export=ALL,TIMEOUT=600 hpc/script_corpus.sbatch \
      '/public/home/majj/atlas-rust/hpc/workloads/workload_*.atlas'

| job | commit | workload | cpp s | rust s | ratio | notes |
|---|---|---|---|---|---|---|
| 3622312 | 24a0d1d | deform_a3 | 0.016 | 0.016 | 1.0 | MATCH; all 4 workloads MATCH |
| 3622312 | 24a0d1d | deform_b2 | 0.015 | 0.024 | 1.6 | MATCH |
| 3622312 | 24a0d1d | partial_kl_a2 | 0.012 | 0.023 | 1.9 | MATCH |
| 3622312 | 24a0d1d | partial_kl_b3 | 0.012 | 0.018 | 1.5 | MATCH |

Heavier iteration counts (~100-150x, oracle 0.5-0.9s per workload),
job 3622323 @ 3ef1a6f, all 4 MATCH:

| job | commit | workload | cpp s | rust s | ratio | cpp MB | rust MB |
|---|---|---|---|---|---|---|---|
| 3622323 | 3ef1a6f | deform_a3 (30k calls) | 0.73 | 1.22 | 1.68 | 4 | 8 |
| 3622323 | 3ef1a6f | deform_b2 (60k calls) | 0.85 | 1.67 | 1.97 | 5 | 9 |
| 3622323 | 3ef1a6f | partial_kl_a2 (40k calls) | 0.54 | 1.64 | 3.05 | 5 | 9 |
| 3622323 | 3ef1a6f | partial_kl_b3 (18k calls) | 0.70 | 1.31 | 1.88 | 4 | 8 |

Real compute ratios are 1.7x-3.0x slower than the oracle on these
small-group loops (vs the 12.97x corpus median, which is dominated by
startup + overload resolution on heavier scripts).

WARNING (3622312): all four workloads finish in 10-25 ms on BOTH
interpreters — the loops are too light to measure anything but process
startup, so these ratios are noise. The workloads need heavier iteration
counts (or larger groups) before their numbers mean anything; use the
script-corpus ledger above for real per-script timing until then.
| 3622804 | 7c325cf | 1 targeted | — | i64 fast path in bounded_linear_combination (460370e): unipotent 68.0s -> 65.7s (-3%). The same job's ATLAS_PROBE_KGB output shows 26 builds for 26 groups: the per-real-form KGB cache ALREADY works, no rebuild per call (see HANDOFF 2026-08-24d) |
| 3622901 | 9dc4b37 | 1 targeted | — | KGB-build micro-opts (f1a1c18 flat class-orbit buffer + 9dc4b37 hash-map intern index/inline mod-two + 6b5df6a two-thread Cartan classification): unipotent 65.7s -> 60.6s (-8%, -21% vs the 77.2s pre-LTO baseline; ratio 13.4x -> 10.19x). MATCH |
| 3622856/3622857 | 9faf2e4 | 1 targeted x2 | — | RAYON_NUM_THREADS=4 unchanged (66.2s), threads=1 WORSE (81.0s): the rayon BFS helps; latch-park samples are normal main-thread waiting, not starvation |
| 3622952 | 6b5df6a | 238 | 34 | FULL CORPUS GREEN again (MATCH 238/238 + 2 SKIPPED_LARGE). KGB micro-opt chain: median rust/cpp 10.25x -> 4.42x, mean 4.38x, over_5x 185 -> 34, within_2x 5 -> 41, rust_faster 3. The mid-tier KGB cluster collapsed (gl4H 12.5x->4.9x, example/speh/test_K/all 11.8x->3.8x, rust 4.4s->1.44s). Remaining outliers: E8_small_block_cell_parameter_numbers.at 76.6x (one big E8 build, rust 3.9s vs cpp 0.05s), unipotent 10.4x (59.7s), then a 5-7x tail of W-class/class-table scripts |
| 3623635 | 907dcd4 | 5 targeted | — | allocation-free cross-edge probes in InvolutionTable BFS (907dcd4): unipotent 59.7s -> 52.7s (-12%, ratio 10.4x -> 9.12x); example/test_K ~1.45s (3.8x), parameters 0.37s (4.7x); all 5 MATCH. E8_small_block_cell_parameter_numbers.at still the outlier at 72.4x (rust 3.91s vs cpp 0.05s, single E8 build) |
| 3623676 | 06b85d7 | 6 targeted | — | typed-side gate-indexed coercion table (4ed8fe0) + SourceText::span line-index share (9e81bc3) + BFS result-row inlining (1cca878): all 6 MATCH; all.at 1.44->1.20s, test_braid 0.81->0.74s, GKfast 0.82->0.75s, generic_degrees 0.80->0.74s, class_tables 0.67->0.62s, groups 0.33->0.30s |
| 3623687 | 9b6f20f | 5 targeted | — | unipotent 52.7s -> 41.5s (ratio 9.12x -> 7.19x; scratch-buffer root classification 06b85d7 + pointer-eq WeylAction gate 9b6f20f); example/test_K 1.45->1.20s. ALL 5 MATCH. REGRESSION FLAG: E8_small_block_cell_parameter_numbers.at 3.91s -> 15.97s (72x -> 296x) — one of 1cca878/06b85d7/9b6f20f quadrupled the single-E8-build path; needs bisect (suspect 1cca878 inline result rows or the pointer-eq gate backfiring on E8's dense cross actions) |
| 3623704 | 06b85d7 | 238 | 26 | FULL CORPUS GREEN with the typed-side changes in (MATCH 238/238 + 2 SKIPPED_LARGE): median rust/cpp 4.42x -> 4.09x, over_5x 34 -> 26, within_2x 41 -> 49, rust_faster 3 -> 4, unipotent 59.7s -> 41.5s (10.4x -> 7.1x). Mean rose 4.38x -> 5.01x ONLY because of the E8_small_block regression (301x, see 3623687 flag — real-group lane, not the typed-side commits). class_tables 0.66 -> 0.64s, test.at 0.32s (5.6x) |
| 3623965 | 9e74504 | 5 targeted | — | permutation-level record classification (9e74504: RootInvolutionData::from_images via w_perm[delta_perm[r]] + precomputed RootSystem negation table, skipping per-root matrix applies in push_record): unipotent 41.5s -> 22.8s (ratio 7.19x -> 3.88x; 59.7s at session start, -62% total); example 1.23s (3.15x), test_K 1.21s (3.17x), parameters 0.35s (4.3x); all 5 MATCH. E8_small_block still 15.98s — confirmed NOT contention; 2026-08-24i correction shows the script is a giant single-line int literal (no KGB build), so the regression window is typed-lane {4ed8fe0, 9e81bc3}, bisect jobs 3623976/3623991 in flight |

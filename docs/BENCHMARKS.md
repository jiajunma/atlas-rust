# Benchmark — Rust vs the real Atlas C++ (fair, same machine)

## Positive-root index (2026-09-03)

Commit `5207c154` passed HPC quick-check `3674970`, focused
Weyl/InvolutionTable/KGB gate `3674971`, and the full 240-script corpus
`3674972` (**240/240 MATCH**). The corpus summary was 57 scripts within 2x,
0 over 5x, and 5 Rust-faster. The integrated local commit is `c3cfedc`, with
the lane-D KGB optimizations preserved; the full local real-group suite is
568/568. The real E7 unitarity workload `3674973` is still running, so wall
time and RSS for this optimization remain uncredited until the Rust/C++
comparison completes. Use an interleaved same-node A/B before attributing a
small timing difference to the index.

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

## ID-primary ExtParam and compact Cayley gates (2026-08-30)

| job | commit | workload | result |
|---|---|---|---|
| 3650615 | 503f81f | atlas-real-group ext_param debug/release | 11/11 passed in each profile |
| 3650630 | 503f81f | focused Weyl/InvolutionTable/KGB debug/release | 62/62, 21/21, 11/11 in each profile |
| 3650652 | 503f81f | full registered differential pipeline | 360/360 PASS, zero pending |
| 3651598 | ea2f23c | fat single-script unipotent corpus benchmark | pending |
| 3651599 | ea2f23c | focused Weyl/InvolutionTable/KGB debug/release | running |
| 3651600 | ea2f23c | full registered differential pipeline | running |

Jobs 3650651 and 3650652 both ran the full registered pipeline; the attempted
positional fixture argument is ignored by pipeline_swap_diff.sbatch, so 3650651
is not a valid unipotent benchmark. No Rust/C++ speedup claim is made for the
compact Cayley slice until job 3651598 reports both wall time and peak RSS.

## Compact involution printer gate (2026-08-31)

| job | commit | workload | result |
|---|---|---|---|
| 3660426 | 4f579f4 | atlas-core compact printer API | 1/1 passed |
| 3660427 | 4f579f4 | focused Weyl/InvolutionTable/KGB debug/release | 62/62, 25/25, 11/11 in each profile |
| 3660449 | 4f579f4 | full registered differential pipeline | 361/361 PASS, zero pending, complete wall/RSS fields |

The language-boundary migration removes legacy record reads from four
canonical-involution printers but retains the per-record legacy permutation,
so no standalone speedup or RSS reduction is claimed. In job `3660449`,
`block_print` measured `0.004s / 6684KB`, `print_block_words`
`0.008s / 7432KB`, and both full-KGB B2/C2 probes measured
`0.005s / 6660KB`. Report SHA-256:
`be3557da568738c6d0c168471445fe39628357c25609c2af31e7e9cc10cab083`.

## Shared root classifications (dirty integrated snapshot, 2026-09-01)

| job | snapshot | workload | result |
|---|---|---|---|
| 3660844 | dirty-root-negatives-final | focused Weyl/InvolutionTable/KGB | 65/65, 29/29, 14/14 in debug/release |
| 3660848 | dirty-root-negatives-final2 | `unipotent_representations_exceptional.at` | MATCH; Rust 7.387s / 1,936,520KB; C++ 4.734s / 881,300KB; 1.560x wall / 2.197x RSS |

The change removes `RootInvolutionData::kind_by_root` and shares
`RootSystem::negatives` through `Arc<[RootId]>` while preserving the public
classification APIs. Compared with integrated baseline job 3660836, peak RSS
fell by about 69MB; wall time is within run variance, so no standalone speed
claim is made. Report SHA-256:
`fa45354d624c186683d07cf9f3a1f3da045c624ecbd1705323d54934e8b57e1a`.

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
| 3624025 | 7231e4c | 238 | 27 | FULL CORPUS GREEN (MATCH 238/238 + 2 SKIPPED_LARGE): unipotent 41.5s -> 22.99s (7.1x -> 3.8x) via permutation-level record classification (9e74504); median 4.10x flat, within_2x 48. E8_small_block still 296x — attribution CORRECTED by bisect 3623976/3623991 (see HANDOFF 2026-08-24i): culprit is 9e81bc3 SourceText::span (byte-filter column scan not auto-vectorized on the single-line 430KB literal; old chars().count() was), real-group trio exonerated; lexer cursor fix dispatched to agent-116 |
| 3624066 + 3624062 | c0d021b | 1 + 6 targeted | — | lexer LineCursor (c0d021b) fixes the 9e81bc3 regression: E8_small_block 15.97s -> 0.11s (3 runs, timer job 3624066, dedicated worktree on cpu; ratio vs cpp 0.053s = ~2.1x, was 296x; also far below the pre-regression 3.91s baseline — the O(n^2) per-token line-prefix scans are gone, spans are amortized O(1) per token). Corpus subset 3624062: 6/6 MATCH, timings flat vs 3623676 (GKfast 0.74s, test_braid 0.73s, class_tables 0.62s, example/all ~1.18s, generic_degrees 0.73s). quick_check 3624061 green (863 unit tests incl. the new cursor sweep test) |
| 3624108 | 83dd11a | 238 | 23 | FULL CORPUS GREEN after the lexer LineCursor fix (MATCH 238/238 + 2 SKIPPED_LARGE): median rust/cpp 4.09x -> 4.01x, over_5x 27 -> 23, within_2x 48 -> 50. E8_small_block_cell_parameter_numbers.at 15.99s -> 0.118s (296x -> 2.19x — regression fixed and 33x under the pre-regression 3.91s). unipotent 22.34s (3.81x; sparse reflection compose 83dd11a roughly offset by measurement noise). New worst ratio: ellipticExceptional.at 13.8x (rust 0.152s vs cpp 0.011s — small-work builtin overhead, see HANDOFF 2026-08-24i startup analysis) |
| 3624220 | b456e1c | 5 targeted | — | mod-space cross-edge transport (b456e1c: `transport_mod_space` carries the record's mod-2 dedup subspace across the BFS cross edge via `b \|-> b XOR <b,alpha_s>*beta_s` — upstream involutions.cpp:256 — instead of fresh `negative_coweight_eigenspace` + `reduce_basis_mod_two` per record; exact because the -1 eigenspace is reflection-equivariant and ModTwoSubspace RREF is canonical; unit test pins transported==fresh on the full B2 table): unipotent 22.8s -> 15.27s (ratio 3.88x -> 2.47x vs cpp 6.18s; session chain 59.7 -> 15.3s, -74%); example 1.21s (2.67x), test_K 1.19s (3.10x), parameters 0.35s (4.44x), E8_small_block 0.12s (1.95x); ALL 5 MATCH. quick_check 3624219 green (498 real-group tests incl. the new pin) |
| 3624259 | 32097c1 | 2 targeted (the 2 SKIPPED_LARGE) | — | CORPUS COVERAGE NOW COMPLETE: E8_big_block_cell_parameter_numbers.at + cells.E8.repsonly.at (3MB single-line literals, previously over the 2MiB size cap) run with SIZE_CAP=4MiB on fat: BOTH MATCH, each rust 0.56s/490MB vs cpp 0.26s/113MB = 2.18x/2.20x — the LineCursor fix scales linearly to 7x-larger literals. Corpus total: 240/240 MATCH, zero skips |
| 3624257 | 32097c1 | 238 | 6 | FULL CORPUS GREEN (238/238 MATCH + the same 2 large scripts verified separately in 3624259): RootSystem simple-reflection permutation cache (32097c1) lifts the whole fleet — median rust/cpp 4.01x -> 3.29x, over_5x 23 -> 6, within_2x 50 -> 51. unipotent 15.27s -> 14.64s (2.40x). ellipticExceptional 13.8x -> 2.46x (0.152s -> 0.03s). Remaining over_5x all small fixed-cost scripts: groups 5.62x, test 5.58x, combinatorics 5.32x, conjugacy_class_partial_order 5.11x, elliptic 5.10x, partitions 5.00x (agent-116's lane). Memory baseline unchanged: median maxrss ratio 12.15x, flat ~130-147MB rust vs 6-14MB cpp |
| 3624255/3624256 | 32097c1 | 8 targeted | — | RootSystem simple-reflection permutation cache + O(1) descent length in left/right_multiply_simple (32097c1; attribution from perf-loop profiles 3624174 — tail is real-group Weyl plumbing, NOT typed eval): 8/8 MATCH. ellipticExceptional 0.155s/14x -> 0.027s/2.46x, class_tables 0.615 -> 0.509s (5.21x -> 4.31x), GKfast 0.74 -> 0.63s (3.02x), test_braid 0.73 -> 0.62s (3.32x), generic_degrees 0.73 -> 0.61s (3.33x), example 1.18 -> 1.07s (2.74x), all 1.18 -> 1.05s (2.55x), test.at flat 0.32s/5.55x (49% CartanClassification::build, other lane). Includes b456e1c. quick_check 3624255 green |
| 3624438/3624439 | 754f66b | 7 targeted | — | WeylElement tiered inline root permutations (754f66b): all 7 MATCH but REGRESSION — sizeof(WeylElement) 3.8KB (enum layout pays the largest tier in every stored element): GKfast 0.62 -> 1.14s (3.0x -> 6.2x), class_tables 0.52 -> 0.83s (7.06x), test_braid 0.64 -> 1.09s (5.94x), example 1.07 -> 2.34s (6.08x), maxrss ~135 -> 194-323MB (example 680MB); only unipotent improved 14.64 -> 10.58s (2.23x). Lesson recorded: fixed-size permutation representations pay their worst case per element, unlike upstream's 8-byte transducer pieces; superseded by 57da89a |
| 3624893/3624894 | 57da89a | 7 targeted | 0 | WeylElement flat exact-size Box<[RootId]> (57da89a: one allocation per element, forward+inverse in one buffer, 24-byte struct): ALL regressions from 754f66b fixed AND unipotent improved further — GKfast 1.14 -> 0.657s (3.10x), class_tables 0.83 -> 0.518s (4.35x), W_reps 0.577 -> 0.456s (4.22x), test_braid 1.09 -> 0.643s (3.48x), generic_degrees 1.07 -> 0.637s (3.43x), example 2.34 -> 1.111s (2.59x), unipotent 10.58 -> 9.99s (2.10x vs cpp 4.76s; 59.7s at session start, -83%); maxrss back to the ~142-147MB flat baseline (unipotent 3.72GB — per-record memory still open). All 7 MATCH |
| 3645698 | dirty-phase3 snapshot | 1 targeted (fat) | 0 | Compact KGB lookup boundary and fixed compact sort key: `unipotent_representations_exceptional.at` MATCH, Rust 7.934s / C++ 5.914s (1.342x), Rust 3,697,052KB / C++ 881,292KB (4.195x). Compared with same snapshot before the lookup-only delta (3645670: 7.925s / 5.864s), no measurable end-to-end change; correctness gate only. |
| 3645946 | 1c34563 | 1 targeted (fat) | 0 | Clean exact-commit confirmation: `unipotent_representations_exceptional.at` MATCH, Rust 9.196s / C++ 5.326s (1.727x), Rust 3,702,888KB / C++ 881,300KB (4.202x). Together with dirty runs 3645670/3645698, this shows run variance and no measurable compact-lookup speedup; the ~4.2x memory gap remains. Full exact-commit pipeline job 3645947 passed 360/360 fixtures with zero pending cases. |
| 3646032 | bf7bb57 | 1 targeted (fat) | 0 | Compact-primary `InvolutionRecord` with release compact/legacy coherence gate: `unipotent_representations_exceptional.at` MATCH, Rust 8.440s / C++ 4.818s (1.752x), Rust 3,702,644KB / C++ 881,296KB (4.201x). Against clean 3645946 this is timing variance, not a measurable speed change; RSS confirms the legacy per-record permutation is still the memory target. Full exact-commit pipeline 3646033 passed 360/360 with zero pending cases. |
| 3646938 | 0f4bc42 | 1 targeted (fat) | 0 | GlobalKGB involution generation uses compact twisted commutation: `unipotent_representations_exceptional.at` MATCH, Rust 8.383s / C++ 4.797s (1.748x), Rust 3,705,124KB / C++ 881,296KB (4.204x). This matches the preceding run band, so no standalone speedup is claimed; it removes one legacy hot-loop consumer while the stored compatibility permutations still determine RSS. Full exact-commit pipeline 3647072 passed 360/360 with zero pending cases. |
| 3647186 | 186bee52 | 1 targeted (fat) | 0 | Compact GlobalKGB canonical involution words: `unipotent_representations_exceptional.at` MATCH, Rust 8.461s / C++ 4.738s (1.786x), Rust 3,708,008KB / C++ 881,300KB (4.207x). This is the same timing/RSS band as 0f4bc42, so no standalone speedup is claimed; the production GlobalKGB word path no longer materializes a legacy Weyl element. Focused 3647184 and GlobalKGB debug/release 3647185 passed; full exact pipeline 3647187 passed 360/360 with benchmark fields complete. |
| 3647609 | 8e7d94d4 | 1 targeted (fat) | 0 | Compact BlockGraph dual-packet pairing: `unipotent_representations_exceptional.at` MATCH, Rust 8.414s / C++ 4.746s (1.773x), Rust 3,706,016KB / C++ 881,080KB (4.206x). The result remains in the preceding run band, so no standalone speedup is claimed; BlockGraph no longer clones/hashes or retains full Weyl permutations for packet pairing. Focused 3647607 and block debug/release 3647608 passed; full exact pipeline 3647610 passed 360/360 with clean source state and complete benchmark fields. |
| 3661806 | 8649686 (phase-5 final, compact migration) | 240 | 7 | FULL CORPUS GREEN after the compact-involution-ID migration (240/240 MATCH, zero skips): median rust/cpp 3.32x, within_2x 55, rust_faster 5. unipotent 7.32s/1.54x with maxrss 1.93GB vs 881MB (2.2x — halved from the pre-migration 3.72GB by compact-primary records + packed KGB/GlobalKGB edges + shared datum/negation storage). over_5x all small fixed-cost: groups 5.88x, test 5.59x, partitions 5.51x, elliptic 5.29x, combinatorics 5.22x. Fixed baseline unchanged: ~141-145MB flat vs cpp 6-12MB (median maxrss ratio 12.22x) |
| 3661865 | adb4051 | 240 | 8 | FULL CORPUS GREEN at the handover tip (240/240 MATCH; parent re-baseline after aligning the HPC main checkout): median rust/cpp 3.44x, within_2x 47, rust_faster 2. unipotent 10.36s/1.62x on cpu node (cpp also slower there: 6.41s — node variance), maxrss 2.94GB vs 881MB. groups.at 0.349s/6.2x/133MB, test.at 5.9x. Fixed ~133-145MB baseline unchanged. Open lanes: CartanClassification baseline (agent-122), unipotent residual RSS (agent-123 plan) |
| 3662251 | 443af7a | 240 | 6 | FULL CORPUS GREEN at the merged tip (240/240 MATCH; merge of agent-cartan-baseline chunked-orbit streaming + agent-legacy-element legacy WeylElement removal): median rust/cpp 3.31x (was 3.44x @ adb4051), median maxrss ratio 12.2x -> **4.17x** — the fixed ~140MB baseline is gone (groups.at 0.327s/35MB vs cpp 0.054s/6MB = 6.06x/5.51x, was 21x rss; test.at 5.47x/5.44x). within_2x 52, rust_faster 4. over_5x all small fixed-cost scripts (groups/test/elliptic/simple_factors/partitions/combinatorics ~5.1-6.1x — the CartanClassification::build CPU time tail, next lever). unipotent (separate fat run 3662094 @ fcc3aee): 7.16s/1.93GB vs cpp 4.88s/881MB = 1.47x/2.20x (was 2.94GB @ adb4051; -34% from the legacy WeylElement removal). quick_check 3662194 green at the merge commit |
| 3662464 | ef1e337 | 240 | 6 | FULL CORPUS GREEN with the COW evaluator merged (240/240 MATCH; cherry-picks 9c6df60/9269054/8d76aa0 = agent-cow-eval minus mimalloc, see HANDOFF 2026-09-01d): median rust/cpp 3.31x -> 3.24x, median maxrss ratio FLAT 4.17x (SharedValue is RSS-neutral — the mimalloc RSS doubling was NOT taken). within_2x 52 -> 56. GKfast 0.616 -> 0.573s, class_tables 0.496 -> 0.464s, example 1.076 -> 0.975s; groups/test flat (fixed Cartan-build cost dominates — agent-128's lane). Matrix-write probe (n=1000 diagonal): 0.42s -> 0.01s, ~40x, oracle scale. quick_check 3662463 green |
| 3662657 | 900f295 | 240 | 0 | FULL CORPUS GREEN with agent-cartan-cpu merged (240/240 MATCH; quick_check 3662656 green at the merge): **over_5x = 0** (was 6), median rust/cpp 3.24x -> **2.55x**, within_2x 53, rust_faster 4. The ~0.25s small-script fixed cost (CartanClassification phase-two cross closure) cut by padded probe buffers + PackedKeySet u128 open-addressing + combinatorial reflection perms (kills the bigint apply_matrix path) + two-thread closure for E7/E8: groups.at 0.327 -> 0.193s (6.06x -> 3.45x), test.at 0.312 -> 0.210s, GKfast 0.573 -> 0.481s (2.52x), class_tables 0.464 -> 0.376s. COST: median maxrss ratio 4.17x -> 5.21x (+11MB fixed on small scripts from worker frontiers/thread stacks — agent-131 follow-up dispatched) |
| 3663798 | 8180166 | 240 | 0 | FULL CORPUS GREEN with agent-unip-rss record slimming merged (240/240 MATCH; quick_check 3663797 green): median rust/cpp **2.45x**, median maxrss 5.21x -> **4.79x** (agent-128's threading RSS regression mostly recovered: groups.at 46.2 -> 40.1MB, test.at 46.4 -> 41.8MB), within_2x 55, rust_faster 4. groups.at 0.193 -> 0.180s (3.27x), class_tables 0.376 -> 0.334s (2.81x), example 0.953 -> 0.855s (2.18x). unipotent (fat, 3663665 @ a4cf71c): 7.10s/1.91GB -> **6.13s/1.54GB** = wall 1.29x / RSS 1.74x vs oracle 4.76s/881MB — massif: per-record WeylAction 246MB -> 0, theta-via-compose 232MB -> 0 (theta transport across BFS; RealProjection i32; u64 PackedKeySet rank<=8). Next biggest block: image_by_root permutations 516MB (agent-132 dispatched) |
| 3664315 | 47f06fe | 240 | 0 | FULL CORPUS GREEN with agent-imgbyroot merged (240/240 MATCH; quick_check 3664314 green): median wall flat 2.45x, median maxrss 4.82x, within_2x 55, over_5x 0. The win is on the heavy end — unipotent (fat, same-node 3664028→3664118): 6.14s/1.54GB -> **5.85s/1.15GB** = wall 1.24x / **RSS 1.31x** vs oracle (image_by_root retained as u16 — 4x slimmer than the oracle's own 8-byte RootNbr storage; cross_links flattened, ~10MB). massif: the 516MB add_cartan block -> 126.5MB. Small scripts: groups.at 0.179s/38.1MB (3.25x/5.86x). Next: theta matrices 246MB deferred — upstream keeps one int_Matrix (coweight = transpose), but ~40 hot coweight_matrix() call sites make transpose-on-demand a wall regression; needs a dedicated lane |
| 3664793 | 2f63a80 | 240 | 0 | FULL CORPUS GREEN with agent-theta-mat merged (240/240 MATCH; quick_check 3664792 green): median wall **2.452x**, median maxrss **4.75x**, within_2x 54, rust_faster 4, over_5x 0 (was 8 at adb4051 re-baseline). unipotent in-corpus 7.44s/983.6MB = 1.15x/1.12x; dedicated fat same-node run 3664724: 5.35s/985.6MB = **1.11x/1.12x** vs oracle (one theta int_Matrix per record + transposed coweight view — the 246MB duplicate-matrix block eliminated; massif 3664725 peak 988.3MB, -158MB). Slowest remaining are all sub-0.25s fixed-cost scripts: elliptic 3.36x, groups 3.20x, test 3.10x. Next massif blocks (unipotent peak 988MB vs oracle 881MB): add_cartan records backing array ~151MB, RealProjection ~112MB (flat-matrix candidate), hashbrown dedup ~104MB, transport_mod_space ~89MB |
| 3665875 | bb107cd | 240 | 0 | FULL CORPUS GREEN at the agent-134+135 merge (240/240 MATCH; quick_check 3665874 green): median wall 2.452x -> 2.474x (flat/noise), median maxrss **4.75x -> 4.33x** (agent-135 InnerClass orbit-build slimming: shrink_to_fit + exact-reserve concat + 1.5x push_slim + stride-adaptive chunks), within_2x 54, rust_faster 4. groups.at 0.191s/34.8MB (3.54x/5.23x; rss 38.1 -> 34.8MB). unipotent per agent-134's same-node fat run 3665722 @ac67bd3: **5.114s/826MB vs cpp 5.238s/881MB = 0.976x/0.937x — rust now FASTER and SMALLER than the oracle on the heaviest workload** (RealProjection flat row-major + record body 304B->240B; massif peak -144.5MB/-14.6%). Slowest wall ratios all sub-0.25s fixed-cost scripts (groups/conjugacy_class_partial_order/test/elliptic ~3.1-3.5x). Residual unipotent heap blocks: image_by_root u16 126.5MB, theta 123MB, records 110.8MB, transport_mod_space 88.7MB, RealProjection 66.3MB (i32-irreducible), dedup ~55MB |
| 3666109 | 65a0e10 | probe (arena) | — | mallopt threshold pinning (atlas-cli main): groups.at median maxrss 34.6MB -> **31.2MB** (-3.4MB), test.at 34.9 -> 31.2MB (7-rep medians, cu052; A/B vs 3666095 default column; env thresholds on top add nothing once pinned). quick_check 3666108 green. Wall unaffected (probe measured RSS only; corpus gate at next merge). Small-script rss ratio now ~4.7x (groups.at 31.2MB vs cpp 6.7MB) |
| 3666702 | 95d87ac | 240 | 0 | MERGE-TIP FULL CORPUS GREEN: 240/240 MATCH, median wall **2.571x**, median RSS **3.762x**, 54 within 2x, 4 rust-faster, 0 over 5x; exact GNU-time fields and clean source state. Compared with 3665875, wall is within run/node variance while small-script RSS improves (groups 31,212KB vs 34,816KB). Report SHA-256: `1a254edefdf322d47ceb014f035ccf3b033af89fec06221f6ba42cfec5d2fa5c`. |
| 3666725 | c641acc (orbit candidate) | 240 | 0 | FULL CORPUS GREEN for the InnerClass orbit-closure CPU pass: 240/240 MATCH, median wall **2.491x**, median RSS **3.770x**, 54 within 2x, 4 rust-faster, 0 over 5x. The median change versus merge-tip run 3666702 is within benchmark noise; no standalone wall/RSS speedup claim. Report SHA-256: `c7b56c2690e416f50cfa0607490551106d2018bc82c8895fec48babab2dc0f21`. |
| 3666726 | c641acc (orbit candidate) | 1 targeted | 0 | `unipotent_representations_exceptional.at` MATCH: Rust **5.129s / 807,780KB**, C++ **4.701s / 881,300KB** = **1.091x wall / 0.917x RSS**. Compared with merge-tip 3666722 (1.076x / 0.915x), wall is run variance; memory remains below the oracle. Report SHA-256: `60a58925da8cac810e457e8edbe8bf2bb6d9888e6b77a1bcfcb85a0021409610`. |
| 3666902 | 1f49e2f | 240 | 0 | FINAL ORBIT-CPU MERGE-TIP CORPUS: 240/240 MATCH, 54 within 2x, 4 rust-faster, 0 over 5x; representative short-script ratios groups 3.439x, test 3.390x, elliptic 3.347x. Exact report SHA-256: `50487164fd5946c77c03bce7a34dd25ff42301f302e37b51861b1690a89d36e3`. |
| 3666900 | 1f49e2f | 1 targeted | 0 | Final merge-tip `unipotent_representations_exceptional.at` MATCH: Rust **5.141s / 808,592KB**, C++ **4.751s / 881,084KB** = **1.082x wall / 0.918x RSS**. Exact report SHA-256: `1a420bdfbf20eddba38d08c585b207af9bd282c2dc333f305778f6f006913e7e`. |
| 3666957 | 86917e5 | 240 | 0 | RootSystem closure-storage candidate: **240/240 MATCH**, median wall **2.591x**, median RSS **3.768x**, 54 within 2x, 4 rust-faster, 0 over 5x. The HashMap + final sort and root-only pending queue preserve the full corpus; report SHA-256: `75cc91d56fd4393e28bf799ee0fe5b461fcd73f08dd92fe48f91a6f8ee404282`. |
| 3666956 | 86917e5 | 1 targeted | 0 | RootSystem closure-storage candidate `unipotent_representations_exceptional.at`: Rust **5.097s / 809,244KB**, C++ **4.695s / 881,300KB** = **1.086x wall / 0.918x RSS**, exact MATCH. Report SHA-256: `0b863872f933bbf08473e99e11bd26a582f1dc33cef32371dd9f6e794ba8749e`. |
| 3666958 | 86917e5 | KGB differential | 0 | **12/12 groups MATCH** (A1-A4, B2-B4, C3-C4, D4, F4, G2); report SHA-256: `c601debc86e1df0a99d24f65ee02bba3ee1b33f79dd69bc4a16d6d506982a90b`. |
| 3666959 | 86917e5 | Weyl/InvolutionTable/KGB focused | 0 | Focused gate passed: Weyl **64/64**, InvolutionTable **30/30**, KGB **14/14**, debug and release; report SHA-256: `d61bcaf008f186542bea67527215ab99c911560d587821bd104999d9eb797546`. |
| 3667072 | 6a7f6f3 | quick-check | 0 | `TEST_DONE status=0`; largest test group **553 passed**. Stdout artifact SHA-256: `70b8cf297322c74b03b73174f7ffe3eb1a34d0e360c5a69861bdc4985934b342`. |
| 3667073 | 6a7f6f3 | Weyl/InvolutionTable/KGB focused | 0 | Focused gate passed: Weyl **64/64**, InvolutionTable **30/30**, KGB **14/14**, debug and release. Stdout artifact SHA-256: `6538d5d21d987712956d6dff0030c8664c9099a5b3b4686f0e4128a2de86fa24`. |
| 3667074 | 6a7f6f3 | KGB differential | 0 | **12/12 groups MATCH** after explicitly binding the GCC 12 oracle runtime; report SHA-256: `3b4e28e6aa1b572f607d413604af8c5ceb740f1d502d9b69267b05b9142ddf85`. |
| 3667075 | 6a7f6f3 | 240 | 0 | FULL CORPUS GREEN: **240/240 MATCH**, median wall **2.3275x**, median RSS **3.757x**, four rust-faster, zero over 5x. The median remains dominated by fixed setup work in short scripts; the change from earlier runs is within node/run variance, so no standalone micro-optimization speedup is claimed. Report SHA-256: `3afb44f71f5e694cfd1a004c874591a698eab383b9dc3e1eb51fe980c857867d`. |
| 3667076 | 6a7f6f3 | 1 targeted (fat) | 0 | `unipotent_representations_exceptional.at` MATCH: Rust **5.087s / 813,608KB**, C++ **4.783s / 881,300KB** = **1.064x wall / 0.923x RSS**. Wall remains in the established run band; heavy-workload RSS is reliably about 8% below the oracle. Report SHA-256: `02dd2c7ec42ef4ef4b519d5e1041537aef8ae02ac9c4c959f6a9b7809b356504`. |
| 3667222 | 3322f54 | Weyl/InvolutionTable/KGB focused | 0 | Focused gate passed: Weyl **64/64**, InvolutionTable **30/30**, KGB **14/14**, debug and release. Report SHA-256: `cf3704c4cad0a8fbd9777765b29a3e6acca84c78c710b7dd646822a8e7e09537`. |
| 3667223 | 3322f54 | KGB differential | 0 | **12/12 groups MATCH**. Rust is about 5–8ms/group versus C++ 25–30ms/group in this probe. Report SHA-256: `c86f2582dd93f8895279470ef0107077d87398b19adba2372e305a5474d97a86`. |
| 3667224 | 3322f54 | 1 targeted (fat) | 0 | `unipotent_representations_exceptional.at` MATCH: Rust **4.938s / 813,044KB**, C++ **4.729s / 881,300KB** = **1.044x wall / 0.923x RSS**. This remains within the prior run band; no standalone speed claim for direct projection buffers. Report SHA-256: `d93d5fff65671c31ea29c78a9c0755da099e485dd332ea7a208ae19b949c2830`. |
| 3667241 | 3448a3a | Weyl/InvolutionTable/KGB focused | 0 | Focused gate passed: Weyl **64/64**, InvolutionTable **31/31**, KGB **14/14**, debug and release. Report SHA-256: `18ba134b5dac9ec6d2d0f8a0c656e2fcad77783f9909696d1e6eb02760e31363`. |
| 3667242 | 3448a3a | 1 targeted (fat) | 0 | `unipotent_representations_exceptional.at` MATCH: Rust **4.889s / 811,464KB**, C++ **4.747s / 881,288KB** = **1.030x wall / 0.921x RSS**. The SmallVec cross-link row and direct projection buffer changes preserve the established heavy-workload memory advantage; timing remains run-band noise. Report SHA-256: `a180d8d464a38107d9333a67cb6e12a0f4eb1cc812a055c2a38b3b676d89c1c2`. |
| 3667272 | 08c3af7 | Weyl/InvolutionTable/KGB focused | 0 | Focused gate passed in debug and release; report SHA-256: `869ad7ba2914a52a5b2b788e6153e6e42f821e6cc33513d14adb294010e713ff`. |
| 3667273 | 08c3af7 | 1 targeted (fat) | 0 | `unipotent_representations_exceptional.at` MATCH: Rust **4.983s / 813,548KB**, C++ **4.753s / 881,088KB** = **1.048x wall / 0.923x RSS**. This remains within the established heavy-workload wall-time band while retaining the ~7.7% RSS advantage. Report SHA-256: `31facfe927109d2ad5a3e5e14a2803912bbca32adad0d9b998a0310705f64108`. |

The `08c3af7` theta-plus-rho fusion is output-identical and passed the focused
gate. Its single-target fat run does not isolate a wall-time gain: 4.983s is
slower than the immediately preceding 3448a3a sample, but still inside the
same-node run band. The stable result is memory: Rust used 813,548KB versus
881,088KB for the oracle (0.923x RSS).

The next candidate, `1763362`, changes only `RootSet::iter` to enumerate set
bits with trailing-zero extraction instead of scanning all 64 positions in
each bitmap block. HPC jobs 3667444-3667447 are submitted; no performance
claim is made until their reports are collected.

The `f835987`/`5352503`/`6a7f6f3` root-system micro-optimizations reduce
search or storage work, but these end-to-end results do not isolate their
individual wall-time effects. Treat the newer same-node run at `3448a3a`,
**1.030x wall / 0.921x RSS**, as the current heavy KGB/Weyl comparison, while
retaining **2.3275x wall / 3.757x RSS** as the current
full-corpus median; the latter is primarily a short-script fixed-cost metric.
| 3666702 | 95d87ac | 240 | 0 | FULL CORPUS GREEN with agent-136 RootSystem enumeration merged (240/240 MATCH; quick_check 3666701 green): median wall 2.571x (node noise; interleaved A/B showed parity), median maxrss **4.33x -> 3.76x** (mallopt threshold pinning landed), over_5x 0, within_2x 54. Decisive metric is retired instructions (noise-free): groups.atx100 32.61G -> **29.55G, -9.4%** (job 3666649, interleaved n=5, cu052). Attribution: eliminated weyl::apply_matrix/compose_matrices helper calls + allocator traffic in enumerate BFS + direct simple-reflection table build. Tooling note: perf srcline/annotate useless under lto=fat (cgu-0:0); use call-graph + instruction counting |

## KLV/unitarity heavy-lane baseline (2026-09-02, jobs 3667606/3668692/3668708 @ probes)

Sizing probes on fat partition (TIMEOUT 600-1800s), same-node A/B per script, both sides MATCH:

| job | probe | rust | cpp | wall | RSS |
|---|---|---|---|---|---|
| 3667606 | probe_klv_e8.atlas (one `deform`, E8 split-ish) | 4.463s / 594,704KB | 4.849s / 804,804KB | **0.920x** | **0.739x** |
| 3668692 | probe_partial_kl_e8.atlas (`partial_KL_block`, E8 n-1 form) | 3.535s / 593,596KB | 3.901s / 804,812KB | **0.906x** | **0.738x** |
| 3668692 | probe_unitary_e6.atlas (`is_unitary` via `<deform.at`, E6 n-1 form) | 0.316s / 43,264KB | 0.159s / 13,004KB | 1.987x | 3.327x |
| 3668707 | probe_unitary_e7.atlas (same, E7 n-1 form) | 0.449s / 67,652KB | 0.279s / 44,528KB | 1.609x | 1.519x |
| 3668708 | probe_unitary_e8.atlas (same, E8 real_form 2) | 3.991s / 619,856KB | 4.051s / 810,972KB | **0.985x** | **0.764x** |
| 3668680 | probe_partial_kl_e6/e7.atlas | 0.028s / 0.164s | 0.024s / 0.192s | ~1x | — |

Reading: at heavy-KL scale (E8 block, ~4s) the Rust KLV/deform/unitarity path is
already at parity or faster than the oracle, with a stable ~25% RSS advantage.
The unitarity ratios >1x at E6/E7 are fixed-cost noise on sub-second scripts
(interpreter startup + inner-class build), not algorithmic. `is_unitary` is
NOT a builtin — it lives in `deform.at`; probes must `<deform.at` first
(both sides reject it identically otherwise — good differential signal).
Optimization lanes in flight target the remaining KL-fill allocator traffic
(agent-138 KlPol in-place) and per-call dual KL table rebuild
(agent-139 dual-block KL cache + accumulator hash).

## KLV lane merges (2026-09-02, merge point cb4e950)

- Merge point `d48c4b0` (agent-139 dual-block KL cache + deform accumulator
  hash): full corpus job **3669375** 240/240 MATCH.
- Merge point `cb4e950` (+ agent-138 in-place KlPol ops): full corpus job
  **3670016** 240/240 MATCH.
- KlPol A/B (job 3669311, cu084, E8 single deform): base 4.50s/594,176KB vs
  branch 4.54s/593,924KB — wall-neutral (within noise, +0.9%), output
  IDENTICAL both vs base and vs cpp oracle (4.88s/804,800KB). The in-place
  ops cut allocator traffic without changing single-shot wall time; the win
  is expected to compound with the dual-KL cache on multi-deform sessions.
- agent-139 dual-KL cache A/B (repeated deform on the same E6 1881-element
  block, same node cu023): bound −1 ×3 base 59.45–59.68s vs branch
  58.63–60.98s (run variance — at bound −1 the O(survivors³)
  `inverse_upper_triangular` dominates at ~19.5s/call vs ~0.06s KL fill);
  bound 0 ×10 base 1.14s vs branch **0.56s (2.0×)**; all stdout
  byte-identical. Jobs 3670244/3670254/3670278/3670322.

## block_deform probe-design correction (2026-09-02, from agent-139)

`param(KGB(rf,0),…)` normalizes to nu=0, so `block_deform`'s nu≠0 gate
silently no-ops — this includes `probe_klv_e8.atlas`, whose 0.920x baseline
above therefore measures block construction, NOT `block_deformation_to_height`.
Real block_deform workloads now on mainline: `probe_bd_e6_repeat.atlas`
(x=1790, bound −1 ×3), `probe_bd_e6_x10_h0.atlas` (bound 0 ×10),
`probe_bd_e7_single.atlas` (x=20925, bound −1; reproduces the pre-merge
`cross of extremal` panic fixed in a8b2fd8). The x3-E8 A/B (job 3670321:
base 4.42s/593,280KB vs a8b2fd8 4.96s/593,688KB, IDENTICAL) is recorded as
a no-op control, not a deform measurement.

Same trap hits the `probe_unitary_e{6,7,8}.atlas` series (2026-09-02):
`param(KGB(rf,0),…)` normalises to nu=0 and `is_unitary` deforms a single
trivial term (E7 probe finishes in 0.61s — "Fully deforming 1 terms"), so
those rows above measure startup + inner-class build, not the KLV
recursion. Real unitarity workload: `probe_unitary_e7_heavy.atlas`
(x=20925, nu=[1,…,1]/1); A/B + oracle triple-compare in job 3671349.
The perf-sample job 3671090 profiled the no-op probe and is void.
- agent-139 dual-KL cache A/B (job 3669387, repeated deform on same block):
  pending at ledger time.

## agent-deformopt deform coefficient pass (2026-09-02, c9c74a3, jobs 3671496/3671531)

Profile first (perf record, prof build with frame pointers): on
`probe_klv_e8.atlas` and `probe_unitary_e8.atlas` the deform/KLV/unitarity
path proper (deform.rs, matreduc.rs, kl fill) is **<0.5% of cycles** — the
probes are dominated by inner-class/KGB/involution-table construction
(tits_element::apply_matrix_mod_two ~10-11%, root_involution::
subsystem_simple_roots ~10%, HashMap insert ~10%, weyl_transducer::
inner_left_mult ~8%, involution_table::push_record ~6%). Phase split
(setup-only probes, job 3671095): `deform(p)` = 4.52s - 4.50s ≈ 0.02s;
`is_unitary(p)` = 5.01s - 4.84s ≈ 0.17s.

Change (wall-neutral at this scale, targets large-n blocks): deform.rs
parity/orientation coefficient pass now precomputes per-column nonzero
opposite-parity q_mat entries and computes coef_j directly (no per-position
O(n) scratch vec, empty columns and zero-sum rows skipped, bit-identical
reassociation of wrapping arithmetic); matreduc::inverse_upper_triangular
uses contiguous row slices and skips zero entries.

A/B on one node, byte-identical stdout vs base AND vs C++ oracle on both
probes (job 3671496); alternating-order 6-rep A/B (job 3671531, cu026):
- probe_klv_e8: base median 4.56s / ~594MB vs branch 4.57s / ~594MB (parity)
- probe_unitary_e8: base median 5.02s / ~622MB vs branch 5.03s / ~621MB
  (parity); oracle 5.13-5.23s / ~811MB

Gates: quick_check 3671394 (CHECK_DONE/TEST_DONE status=0, 565 real-group
tests incl. new wrapping-exactness test); corpus job below.

**Parent correction (2026-09-02):** the deform.rs half of this change was
NOT merged. Its byte-identical A/Bs used the x=0 E8 probes whose deform
path is a no-op (see the phase split above), so they never exercised the
changed coefficient pass. The same-parity skip in the update loop diverges
from upstream repr.cpp:2109-2120 (which applies coef[j] for ALL j<pos):
coef = Q^{-1} v mixes parities under back substitution, so same-parity
coef entries are generically nonzero (hand counterexample at n=3,
randomized check finds them immediately). See the "Lane C review" entry in
docs/HANDOFF.md. Only the matreduc.rs half landed (8e340b6, corpus 3671533
MATCH 240). Decisive A/B with-real-deform-workload: job 3671577.

## Decisive deform A/B + matreduc merge A/B (2026-09-02, jobs 3671577/3671543)

- 3671577 (cu-partition, c9c74a3 WITH the deform.rs parity skip vs 8e340b6
  mainline without it): `probe_bd_e6_repeat` (x=1790, bound -1 x3) **DIFF**,
  10,575 differing lines, coefficients wrong from the first deform print
  (e.g. `(16-16s)*parameter(x=1765,...)` vs correct `(4-4s)*...`); the skip
  branch ran 19.73s vs 60.88s because it drops real terms.
  `probe_bd_e6_x10_h0` (bound 0) IDENTICAL 0.54s/0.56s (bound 0 exits
  before the coefficient pass matters). The mathematical rejection of the
  parity skip (see "Lane C review" in docs/HANDOFF.md) is now empirically
  confirmed; corpus 240/240 did NOT catch this — the corpus simply never
  prints a block_deformation_to_height result with a nonzero same-parity
  coef term. Coverage gap recorded.
- 3671543 (matreduc-only merge 8e340b6 vs 7dacfe2, probe_bd_e6_repeat):
  stdout IDENTICAL; wall 59.34s -> 61.18s (+3.1%, single non-alternating
  pair — within the phantom-regression range documented by lane C;
  alternating-rep confirmation pending).

## Alternating-rep matreduc A/B + oracle reference (2026-09-02, jobs 3671613/3671638)

- 3671613 (4 alternating reps, probe_bd_e6_repeat, 8e340b6 vs 7dacfe2):
  all IDENTICAL; old walls 59.00/62.75/59.16/58.70 (median ~59.1s), new
  61.20/61.06/61.05/60.50 (median ~61.0s) — a CONSISTENT +3.3%, so the
  lane C row-slice+zero-skip inversion rewrite is a small real regression
  on dense matrices (the zero-skip branch never fires). Superseded by the
  loop interchange in 8a77b38.
- 3671638 (CPP oracle, probe_bd_e6_repeat): 15.41s/15.12s, 38.6MB —
  **the oracle is ~4x faster than our 60s** on real bound -1 E6 deform.
  Output content matches ours (line-order permutations only). With
  inverse_upper_triangular at ~19.5s/call x3 dominating our side, the
  inversion's column-strided cache behavior is the main suspect; the
  loop interchange in 8a77b38 targets exactly that.

## E7 block_deform large-scale reference (2026-09-02, job 3670294)

CPP oracle on `probe_bd_e7_single.atlas` (E7 x=20925, nu=[1,..,1], bound
-1, real deform workload): **1:39:27 wall, 5.18GB RSS**. The fixed Rust
build (a8b2fd8) ran past its pre-fix panic point and is still going at
1h46m+ (job 3670256, 2h limit; probe-only 6h rerun staged as
`probe_bd_e7_rust.sbatch` in atlas-rust-klundef if it times out).
Heavy E7 unitary A/B (3671409, probe_unitary_e7_heavy): rust 7dacfe2
20:03.5 vs a8b2fd8 20:25.0 (~+1.8% from the ext_kl in-place change, single
run each), RSS ~449MB both; CPP leg still running.

## Loop-interchanged inversion A/B (2026-09-02, job 3671682 @ 8a77b38)

probe_bd_e6_repeat (bound -1 x3, E6), old=8e340b6 vs new=8a77b38, 2 reps,
both IDENTICAL: old 60.77/61.36s -> new **53.04/53.33s (-13%)**, RSS
unchanged (~46MB). Oracle reference remains 15.4s (3671638), so a ~3.4x
gap is still open; perf record job 3671836 (frame-pointer build) targets
the residual. Gates at 8a77b38: quick_check 3671672 TEST_DONE status=0,
corpus 3671673 MATCH: 240.

## Orientation-order cache + hoist A/B (2026-09-02, job 3672106 @ f9202e7)

perf record 3671836 (frame-pointer build, E6 probe) attributed the residual
gap: `block_deformation_to_height` 83.6% children, of which
`RepContext::orientation_number` 73.7% -> `driftsort_main`/`sort_by_key`
64.6% with an alloc/free storm (`try_allocate_in` 12.8% flat, `_int_free`
14.1% flat) — every call re-sorted the positive roots by coroot coordinates
with an allocating key, and the coefficient pass called it once per
(position, j) survivor pair.

Fix f9202e7: the coroot-coordinate order of positive roots (+ inverse
position table) is precomputed once in `RepContextDerived`; the coefficient
pass computes each survivor's orientation number once (O(n) calls, was
O(n^2)). Gates: quick_check 3672055 TEST_DONE status=0, corpus 3672056
MATCH: 240.

A/B 3672106 (probe_bd_e6_repeat, old=8a77b38, interleaved reps, both
IDENTICAL): old 57.30/52.34s -> new **4.43/4.06s (~-92%, 12.9x)**, RSS
unchanged (~46MB). **Versus the oracle's 15.4s (3671638) the Rust build is
now ~3.7x FASTER on this E6 workload** — the block_deform E6 gap is closed
and inverted. E7 large-scale probe on this binary submitted as 3672161
(fat, 6h) to check the big-group scaling against the oracle's 1:39:27.

Post-fix profile (job 3672168, frame-pointer build @ f9202e7, same probe):
`block_deformation_with_dual_kl` is now 86% SELF time (inlined
SplitInteger/matrix arithmetic in the coefficient pass — the algorithm's
real work), with inverse_upper_triangular down to 3.6% and
`KlSupport::prim_index`'s BTreeMap lookups at 1.7%. No remaining
redundancy of the allocation/sort kind; further E6 gains would need
algorithmic changes, and the probe already beats the oracle 3.7x.
Gotcha: sbatch `#SBATCH --output=` lands in the SUBMISSION cwd — cd to the
worktree before sbatch or the .out goes to $HOME (3672168 did).

## Mask-hasher prim_index A/B (2026-09-02, job 3672384 @ 0fe6955)

`KlSupport::prim_index` BTreeMap<u32,_> -> HashMap with a single-multiply
mask hasher (the table is consulted on every `kl_pol`). Gates: quick_check
3672382 TEST_DONE status=0, corpus 3672383 MATCH: 240. A/B vs the f9202e7
binary (probe_bd_e6_repeat, 4 interleaved reps, all IDENTICAL): old
4.38/3.94/4.46/4.52 vs new 3.92/3.88/4.41/4.48 — new faster in every pair,
median 4.42 -> 4.16s (~-6% at this scale; expect a larger share on
KL-denser large-group runs).

## E7 block_deform large-scale probe harvest (2026-09-03, jobs 3671969/3672161/3672980)

probe_bd_e7_single (split E7, x=20925; deform(p) + block_deform(p,d,-1)),
fat partition. Oracle reference: **1:39:27 / 5.18GB** (job 3670294).

| build | wall | max RSS | job |
|---|---|---|---|
| a8b2fd8 (pre-orientation-cache) | 3:15:54 | 5.33GB | 3671969 |
| f9202e7 (orientation cache) | **1:07:04** | 5.33GB | 3672161 |
| 0fe6955 (+ maskhash) | 1:07:04 | 5.33GB | 3672980 |

The orientation-order cache scales to E7: 3:15:54 -> 1:07:04 (-66%),
**1.48x faster than the oracle**. maskhash is wall-neutral on this
workload (its E6 -6% came from KL-dense prim_index lookups; the E7
block_deform profile is dominated by coefficient arithmetic). All three
rust outputs are byte-identical.

**CORRECTNESS CAVEAT (under investigation, top priority):** the sorted
diff against the oracle output is NOT clean at E7 scale — ~43k differing
lines out of ~46k. Same (x, nu, height) terms carry different Split
coefficients (e.g. x=12492 nu=[133,-95,-38,0,304,-209,-57]/42 [271]:
oracle (196-196s) vs rust (-10000+10000s); x=4010 nu=[18,21,...]/8 [370]:
oracle (1000-1000s) vs rust (63832-63832s)); 626 x-values appear only in
the rust output, none only in the oracle. The divergence predates
f9202e7/0fe6955 (a8b2fd8 output identical). Corpus MATCH: 240 passes and
the E6 probe matched the oracle, so this manifests between E6 and E7
scale. Triage: ladder probe 3674359 (B4/D5/E6 rust-vs-oracle), E7
deform-only probe 3674360. Until resolved, the E7 wall comparison above
must be read as "same script, possibly different arithmetic".

## Lane D tip increments v9 A/B (2026-09-03, jobs 3672941/3672942 @ 83fb43b)

Fast leg (3672941, interleaved reps, IDENTICAL): unitary_e8 4.06 -> 3.55s
(-12.6%), klv_e8 3.69 -> 3.08s (-16.6%), kgb_e7_allforms parity. Heavy leg
(3672942, probe_unitary_e7_heavy, stdout IDENTICAL 190937B, all legs
EXITED 1 = probe's own exit quirk): base 19:45.7/19:56.8 vs branch
20:02.2/19:35.8 — parity within noise; RSS ~450MB. Combined with the v7
numbers (3672407) the whole lane-D chain is validated; merged as 12a39a1.

## Heavy unitary A/B final state (job 3671409, atlas-rust-extkl)

Rust legs done before the outage: 7dacfe2 20:03.5, a8b2fd8 20:24.9,
outputs identical (190937B). The oracle leg never completed: job
cancelled 2026-09-03T06:10:37 (SLURM time limit) while the C++ binary was
still running. Bound: oracle >> rust ~20min on probe_unitary_e7_heavy
(exact oracle number unobtainable within a 6h fat job; rerun with a longer
limit only if the number is needed).

## deform KL record-cache A/B + perf attribution (2026-09-04, jobs 3679415/3679419/3679388)

83a78bf A/B (3679415, probe_deform_e7_only, alternating 3 reps, E7 x=20925
deform(p) print): base b7ed9e5 median 19.06s/383MB vs tip median
18.78s/394MB — wall parity (+1.5%, within noise), RSS +12MB for the
retained per-block KL tables (matches upstream's one-table-per-block
residency). The cache is neutral for a single deform; its value is
multi-final workloads that revisit one block.

Frame-pointer perf (CARGO_PROFILE_RELEASE_LTO=off, -C force-frame-pointers,
hpc/perf_unitary_profile.sbatch, data files perf-unitary-*.data in the
merged worktree):

- deform-only E7 (3679419, 17s run): kl_support::prim_index 14.7%,
  kl_table fill_kl_column 11.5%, LocatedBlock::with_kl_table 9.5%
  (wrapper! investigate), allocation/free/Vec-clone ~20%, SipHash
  (DefaultHasher) ~6-7%, root_system::id_of_slice 3.3%.
- heavy unitary E7 (3679388, 25min sampling): locator::additive_closure
  61.6% inclusive -> combine_roots 53.9% -> id_of_slice 32.2% flat
  (23.8%) + slice lexicographic compare 17.2%; combine_roots allocates
  two fresh vectors per call. Allocation ~12%, BTreeMap ~3.5%.

Associate-variety A2 correctness baseline (3679427,
probe_associated_variety_a2.atlas): SORTED_MATCH 3606 lines; rust
0.69s/55.6MB vs oracle 0.26s/14.1MB (A2 is startup-dominated; the large
AV workload still needs profiling before any optimization there).

## avopt lane baselines @0835205 (2026-09-04, probe-diff, fat/cpu as noted)

All SORTED_MATCH. Jobs 3679584-87:
- probe_associated_variety_a2: rust 0.80s/56.1MB vs oracle 0.23s/14.1MB
- probe_unitary_e6: rust 0.66s/43.5MB vs oracle 0.18s/13.0MB
- probe_unitary_e7 (light, x=0): rust 0.60s/71.4MB vs oracle 0.28s/44.5MB
- probe_deform_e7_only: rust 18.80s/394MB vs oracle 10.01s/204MB (1.88x wall)

AV oracle-freeze probes (jobs 3679972-80, all SORTED_MATCH, cpu nodes):
gkfast_{a3,b3,g2,d4}, av_ann_{a3,b3,g2,d4}, av_ann_b2_nonintegral —
all startup-dominated (rust 0.71-0.98s vs oracle 0.23-0.46s); reference
outputs frozen as probe_*.{cpp,rust}.out in the HPC lane. E6-scale freezes
(probe_gkfast_e6 / probe_av_ann_e6, jobs 3679983/3679984, fat 6h) pending.

## Root-sum table (85acd90): verified interleaved A/B vs 0835205

Lazy n×n u16 root-sum table on RootSystem (O(1) combine_roots, subtract via
negatives table), negatives-table negate_root, work-queue additive_closure.
Gates: quick_check 3679927 PASS (573 real-group tests); probe-diff
3679957-59 SORTED_MATCH (unitary_e6, deform_e7_only 46118 lines, av_a2).

- Heavy unitary E7 (probe_unitary_e7_heavy, fat, A/B 3679961, 3 interleaved
  reps): base 78.69/78.97/78.67s -> tip 37.05/36.85/36.81s = **2.12x**,
  RSS neutral (~427MB both).
- deform-only E7 (probe_deform_e7_only, cpu, A/B 3679960): base median
  20.55s vs tip median 20.42s — neutral as predicted (additive_closure is
  not hot there); RSS identical.

## Prim-slot hoist (368707a): verified interleaved A/B vs aa148b4

KL primitive-index records moved behind a mask->slot map; hot column loops
resolve the slot once (Copy) instead of a HashMap probe per access.
Gates: quick_check 3680023 PASS; probe-diff 3680036-38 SORTED_MATCH.
Attribution is pure: the benchmarked binary predates the 1caa620 lane merge.

- deform-only E7 (cpu, A/B 3680039): base median 20.62s -> tip median
  18.95s = **1.088x** (base 20.53/20.66/20.62, tip 18.95/18.89/19.04).
- heavy unitary E7 (fat, A/B 3680040): base median 37.03s -> tip median
  35.53s = **1.042x** (base 37.03/36.98/37.08, tip 35.58/35.48/35.53).

Cumulative avopt-lane heavy unitary E7: 78.7s -> 35.5s = **2.21x**.
deform-only E7 remains ~1.9x slower than the oracle (18.95s vs 10.01s);
next recorded levers: allocation ~20% spread, id_of_slice in KL paths (3.1%).

## Profile 3680072 (deform-only E7 @368707a, post-prim-slot, flat self%)

fill_kl_column 10.1, with_kl_table 10.0 (inlined callback body, not wrapper),
kl_pol 5.2, prim_slot 5.2 (residual per-(x,y)-query hash probes across
columns), KlPol::sub_shifted_assign 5.2, allocation cluster ~20% (try_allocate
4.2 + free 4.1 + malloc 3.9 + Vec::clone 3.4 + _int_malloc 2.4 + memmove 1.8 +
...), id_of_slice 3.4 + SliceOrd compare 2.3 (KL-path binary searches),
match_pol 2.0, SipHash ~2.7 total. Next lever: allocation reduction in the
KL column fill (per-column Vec rebuilds, working[] KlPol clones,
mu_pairs/down_set rebuilds); then cross-column prim_slot caching (Cell would
break Sync — needs a different design).

## agent-klfill (da5ccf2, KL column-fill allocation reduction, merged 2a54201)

- deform-only E7 (cpu, interleaved A/B 3680164, REPS=3): base median
  19.03s -> tip median 15.40s = **1.235x** (base 19.09/19.03/18.93,
  tip 15.41/15.38/15.40; RSS neutral ~394MB vs ~398MB).
- Gates: quick_check 3680159 green; probe_deform_e7_only SORTED_MATCH
  46118 lines (3680161); probe_partial_kl_e7 SORTED_MATCH (3680163).
- deform-only E7 vs oracle now 15.40s vs 11.11s = 1.39x slower (was 1.9x).

## Profile 3680172 (deform-only E7 @da5ccf2, post-klfill, flat self% + callers)

fill_kl_column 19.95, with_kl_table 10.35, kl_pol_at_slot 6.10 (5.2 self from
common_deformation_terms per-(x,z) queries), id_of_slice 4.82 + SliceOrd 2.83
— callers: PartialBlock::build via pos_to_neg (~1.4%) and bruhat_below /
BruhatGenerator::block_below recursion (~2.8%), i.e. BLOCK CONSTRUCTION, not
the KL column loop; match_pol 3.41 (fill_kl_column 2.63) + SipHash/write 2.31
+ hash_one 1.37; allocation cluster down to ~13% (was ~20%); misc:
reflect_coordinates 1.86, Arc<BlockTopology>::length 1.78, merge_pol_term
1.67, StandardRepr::eq 1.60, BTreeMap insert ~2.2 (pos_to_neg/bruhat_below),
lattice::pair 1.09. Next levers: fast hasher for KlHashTable (codebase has a
MixingHasher precedent in involution_table), id_of_slice -> hash map in
RootSystem, BTreeSet->sorted-Vec in block build.

## probe_unitary_e7_heavy: oracle is the slow one

- Rust leg (da5ccf2, cpu): 47.78s wall, 430MB RSS — but exits 1 on the
  pre-existing finals-invariant bug (see HANDOFF 2026-09-04g).
- Oracle leg (cpu, 3680162): CANCELLED at the 1h limit — interpreted
  is_unitary at E7 x=20925 takes the oracle >50min. Once the finals fix
  lands, rust is >60x faster than the oracle on this workload; correctness
  gating needs the 8h fat reference job (3680251) or the fast term-level
  repro probe (probe_finals_e7_term20138, oracle leg ~1min).

## agent-fasthash (55806e6, merged 77313d4)

- deform-only E7 (cpu, interleaved A/B 3680269, REPS=3): base median
  14.81s -> tip median 13.22s = **1.120x** (base 14.79/14.87/14.76,
  tip 13.33/13.18/13.14; RSS neutral ~398MB).
- Gates: quick_check 3680263 green; probes 3680265-67 SORTED_MATCH
  (deform_e7 46118 lines, partial_kl_e7, finals_term20138); filekl diff
  4/4 blocks PASS (3680268).
- Cumulative deform-only E7 this session: 19.03s -> 13.22s = 1.44x;
  vs oracle ~10.0s the gap is now ~1.32x (was 1.9x).

## agent-blockbuild (merged 6e749a8)

- deform-only E7 (cpu, interleaved A/B 3680308, REPS=3): base median
  13.91s -> tip median 13.39s = **1.039x** (pos_to_neg/bruhat_below:
  BTree sets/maps -> sorted vecs, clone elision).

## agent-ratfast + agent-fastdiv (i128/u64 rational fast paths; merge pending)

- ratfast (machine-word fast paths for rational scalar ops, root-table
  length arithmetic, interpreter rat ops) REQUIRED two correctness fixes
  caught by its own unit tests, invisible to the E6/E7 probes: malachite
  stores sign separately from a non-negative numerator (small_parts dropped
  it), and |i64::MIN| = 2^63 overflows a signed conversion (fixed c00faf5,
  232b85c; tests 120a18d now treat None-vs-out-of-window correctly).
- probe_gkfast_e6 interleaved A/B vs 6e749a8 (REPS=3):
  ratfast-only (3680372): 2.607s -> 2.577s = 1.012x (noise-level);
  ratfast+fastdiv (3680418): base 2.56/2.52/2.54 -> tip 2.37/2.33/2.34,
  median 2.540s -> 2.347s = **1.082x** cumulative.
- Profile 3680340 (E6 gkfast, post-ratfast): allocator cluster ~24%,
  128-bit division libcalls ~10% (__divti3 5.27 + u128_div_rem 3.52 +
  __umodti3 0.99), FastSum::add_term 5.95, orbit_cross_closure 5.42,
  squared_length 3.40, Value::clone 2.33, RootTable::build 2.23.
  fastdiv targets the libcall block via guarded u64 hardware division in
  gcd_u128 / from_i128_parts / FastSum::add_term.

## mimalloc early reads (00edcff, gate pending; cross-job, indicative only)

- Offline HPC build works: mimalloc 0.1.52 + libmimalloc-sys 0.1.49 +
  cc 1.4.4 + shlex 2.0.1 + find-msvc-tools 0.1.11 pinned to the HPC cargo
  cache (compute nodes have no crates.io access); MM build 3680403.
- All probes SORTED_MATCH: gkfast_e6 3680419, deform_e7 3680421,
  finals_term20138 3680422.
- Timing (mm worktree, single runs): gkfast_e6 rust 2.16s (vs ~2.5-2.6s
  non-mimalloc); **deform_e7 rust 9.56s vs oracle 9.54s — parity**;
  finals rust 9.31s vs oracle 8.32s.
- RSS roughly doubles under mimalloc: gkfast_e6 184MB (was ~90MB),
  deform_e7 436MB, finals 467MB (oracle ~204-209MB). Fine on fat/cpu
  limits; watch E8-scale headroom.

## agent-rtcache (merged ac440c6) + agent-mimalloc (merged 489eb15)

- rtcache probe_gkfast_e6 interleaved A/B vs fastdiv tip (3680445, REPS=3):
  base 2.35/2.34/2.41 -> tip 1.38/1.36/1.49, median 2.35s -> 1.38s =
  **1.70x** (RootTable memoized per (BasedRootDatum, prefers_coroots) via a
  session-wide OnceLock<Mutex<Vec>> mirroring weyl_datum_shared; the 2.2%
  flat profile share understated its true cost — rebuilds also drove
  allocator churn). Cumulative ratfast+fastdiv+rtcache on gkfast_e6:
  2.54s -> 1.38s = 1.84x. Gates: quick_check 3680438 green (test fix
  120a18d confirmed), probes 3680440-444 all SORTED_MATCH.
- mimalloc gates: quick_check 3680452 green, probes 3680454-458 all
  SORTED_MATCH; interleaved A/B vs ac440c6 (REPS=3):
  deform_e7 (3680459) base 13.40/13.71/13.30 -> tip 11.41/11.38/11.07,
  median 13.40s -> 11.38s = **1.18x**; gkfast_e6 (3680460) base
  1.46/1.38/1.33 -> tip 1.03/1.03/1.03, median 1.38s -> 1.03s = **1.34x**.
  RSS ~2x under mimalloc (E6 172MB vs 90MB; deform-E7 437-449MB vs ~399MB).
- Session totals: gkfast_e6 2.63s -> 1.03s = **2.55x** (oracle 0.59s; gap
  4.4x -> 1.75x). deform-E7 19.03s -> 11.38s = 1.67x (oracle ~10s; gap
  ~1.14x).

## agent-orbitopt (merged 83d264d) / agent-rscache (merged 282146d) / agent-callargs (merged 723585b)

- orbitopt AB 3680678 (gkfast_e6 vs b1319c1): 1.02s -> 1.03s, neutral
  (kept: helps many-small-build workloads; the E6 probe is a FEW-BIG-BUILDS
  regime — classification probe 3680679: only 28 builds, all distinct, one
  E8-scale rank-8/240-roots build dominates orbit_cross_closure's 12%).
- rscache AB 3680691 (gkfast_e6 vs orbitopt tip): 1.03s -> 1.03s, neutral
  on this probe; removes per-InnerClass-construction RootSystem
  re-enumeration (matters for repeated-datum sessions).
- callargs (a0a596e: for-loop iterable borrow, closure args SharedValue,
  denotation Rc) gates: quick_check 3680701 green, 5 probes SORTED_MATCH;
  AB 3680708 gkfast_e6: base 1.05/1.08/1.05 -> tip 0.97/0.94/0.96, median
  1.05s -> 0.96s = **1.094x**. E7 interpreted reads (3680709/710):
  gkfast_e7 19.85 -> 17.72s, av_ann_e7 21.55 -> 19.53s (oracle 14.05/14.24).
- Session totals now: gkfast_e6 2.63 -> 0.96s = **2.74x** (oracle 0.59s,
  gap 1.63x); gkfast_e7 17.72s vs oracle 14.05s (gap 1.26x).

## Upstream involution-orbit machinery (explore report, for the next slice)

- Upstream never enumerates involution orbits in the InnerClass constructor
  (laziness: closure runs only on KGB/global-KGB build, kgb.cpp:197/247/512);
  we build twisted_conjugacy_partition twice (primal+dual) eagerly per
  inner-class value (domain_builtins.rs:2342-2368). gkfast_e6 DOES build
  KGB, so laziness only defers; the representation is the lever.
- Upstream element = weyl::WeylElt = 32-byte transducer array (vs our
  240-byte root permutation for E8); one global open-addressing hash+pool
  per inner class serves dedup+numbering+membership (Cartan_orbit /
  InvolutionTable::add_cross, structure/involutions.cpp:222-251,362-379);
  per edge = two transducer multiplies (~a few table lookups on 32 bytes).
- Our weyl_transducer.rs already has CompactWeyl/WeylElt=[u8;8],
  inner_mult/inner_left_mult/apply_twist/materialize_root_permutation —
  Option B = port Cartan_orbit/add_cross onto it inside
  orbit_cross_closure, keeping phase-one class order/representatives and
  discovery-order edge enumeration for byte-identical output.

## agent-builtinargs (merged here; 14b7e77)

- Builtin::run takes Vec<SharedValue>; ~150 scalar-op arms + relation/
  printer families borrow through &[SharedValue] (helpers return refs);
  domain table materializes via own_all (unchanged cost); consuming scalar
  arms own explicitly. domain_builtins.rs untouched.
- Gates: quick_check 3680723 green; probes 3680725-729 SORTED_MATCH.
- AB 3680730 gkfast_e6 vs 723585b: base 0.97/0.96/0.98 -> tip
  0.90/0.93/0.90, median 0.97s -> 0.90s = **1.078x**.
- E7 interpreted (3680731/732): gkfast_e7 17.72 -> 16.79s, av_ann_e7
  19.53 -> 18.38s (oracle 14.05/14.24; gaps 1.20x / 1.29x).
- Session totals: gkfast_e6 2.63 -> 0.90s = **2.92x** (oracle 0.59s; gap
  1.53x, was 4.4x).

## agent-transduce (9870711) — REJECTED at the gate, kept unmerged locally

- Transducer-based orbit closure (8-byte WeylElt keys + u64 pool) is
  CORRECT (quick_check 3680741 green incl. new S6 pin; 5 probes
  SORTED_MATCH) but SLOWER on the target workload: AB 3680748 gkfast_e6
  0.90s -> 1.07s = 0.84x; E7 reads also slower (17.54/19.00 vs
  16.79/18.38). Reverted per the >2% gate.
- Root cause of the regression: the BFS dedup got cheap, but the
  membership `entries` index still needs theta's simple-root images per
  member, and synthesizing those from the compact element (word replay
  through reflection tables) costs MORE than gathering bytes from the
  already-materialized 240-byte permutation the old path carried as its
  BFS state. Upstream avoids this because its membership IS the pool
  index (Cartan_orbits::locate over orbit offsets) — a membership-index
  redesign (pool-offset keyed) would be required for the transducer path
  to pay off, not just the inner loop swap. Recorded so it is not
  retried in this shape; branch agent-transduce stays local.

## agent-rrcnt (merged here; 73ac5e9+12f115e)

- Container Value variants (Row/Tuple/...) hold Rc<[Value]> elements:
  clones become refcount bumps; per-element Rc allocations from phase 1
  removed in phase 2 by switching the element store itself to Rc slices.
- Gates: quick_check 3680767 green; probes 3680769-773 SORTED_MATCH.
- AB 3680774 gkfast_e6 vs 1626174: base 0.93/0.92/0.92 -> tip
  0.82/0.82/0.80, median 0.92s -> 0.82s = **1.122x**.
- E7 interpreted (3680776/777): gkfast_e7 14.39s, av_ann_e7 15.74s
  (oracle 14.05/14.24; gaps 1.02x / 1.11x — gkfast E7 essentially at
  oracle parity; E7 RSS also dropped to ~780MB from ~1.17GB).
- Session totals: gkfast_e6 2.63 -> 0.82s = **3.2x** (oracle 0.59s;
  gap 1.39x, was 4.4x).

## agent-rrcnt follow-up: union payloads through Rc (merged; 28fc1aa)

- `Value::Union.value` Box -> Rc; case/union-case branch binding is now an
  Rc bump (typed.rs Case/UnionCase), UnionInject shares the forced payload.
- Gates: quick_check 3680785 green; probes 3680787-791 SORTED_MATCH; E7
  3680793/794: gkfast_e7 14.32s (oracle 14.09), av_ann_e7 15.68s (oracle
  14.19), RSS 780MB (oracle 387MB — 2x RSS remains an open item).
- AB 3680792 gkfast_e6 vs 12f115e: base 0.80/0.87/0.80 tip 0.84/0.80/0.78,
  medians equal 0.80s — NEUTRAL on speed (Union is cold here), RSS tip
  ~6% lower. Accepted as a structural clone-cost elimination.

## probe_setup_e6 (3680795): the whole E6 gap is fixed setup

- Setup-only probe (3 script includes + E6 inner class + param, NO GKfast
  query): rust 0.65s vs oracle 0.40s => fixed gap ~0.25s.
- gkfast_e6 totals 0.82s vs 0.59s => the GKfast query leg is 0.17s rust vs
  0.19s oracle — the query itself is AT PARITY (slightly ahead).
- Conclusion: closing E6 means cutting startup: script parse/interpretation
  of the prelude and the E6 inner-class build (orbit_cross_closure 13% of
  the profile is that build). Post-rrcnt profile 3680781: Value clone is
  gone from the map; remaining flat items are orbit_cross_closure 13.1%,
  memmove 8.1% (mostly parser), typed::evaluate 6.0%, allocator ~7%,
  parser+lexer ~4%.
- Membership redesign assessed (explore agent-24): ClassMembership is
  queried only O(classes x imaginary roots) times per build, never per KGB
  element; swapping sorted-vec->hash buys nothing without the WeylElt-keyed
  BFS-dedup redesign (Variant B = transducer + membership as one change) —
  shelved, matches the agent-transduce postmortem prediction.

## agent-rootkey (merged; 467be26+48a63cf)

- id_of_slice: rank <= 8 root coordinates pack into one u64 (8-bit lanes,
  +128 bias) — the block-construction probes hash/compare one integer
  instead of a boxed slice; rank guard keeps the packed map exact (the
  -128 bias aliases shorter keys by construction, so len != rank misses
  the packed map and goes to the boxed-slice fallback; rank > 8 uses the
  fallback outright).
- Gates: probes 3680802-806/808 SORTED_MATCH (on 467be26); quick_check
  3680813 green on 48a63cf (delta = derive(Clone,Debug) only).
  FIRST gate after quick_check.sbatch learned to propagate failures —
  pre-fix, job 3680800's test-compile error did NOT stop the chain.
- AB 3680807 gkfast_e6 vs 12f115e: base 0.81/0.81/0.80 tip 0.79/0.78/0.80,
  median 0.81s -> 0.79s = **1.025x**. E7 3680808: gkfast_e7 14.18s
  (oracle 13.91).

## agent-klpol (ee96d32) — REJECTED at the gate, kept unmerged locally

- KlPol on SmallVec<[i32;8]> (inline coefficients): all 5 probes
  SORTED_MATCH, quick_check 3680839 green, but AB 3680846 on
  probe_finals_e7_term20138 vs 0546294: base 9.27/9.23/9.17 tip
  10.00/9.89/9.88 — median 9.23s -> 9.89s = **0.93x REGRESSION**.
- Root cause (disproven hypothesis): "KL polynomials are short, so inline
  storage kills allocation cost". In this workload the interned pool is NOT
  short-dominated — E7 μ-correction intermediates are long — so SmallVec
  spills to the heap anyway, while the fatter 40-byte KlPol (vs 24-byte
  Vec) makes every pool/index clone a bigger memcpy and thins cache
  density. klfill's scratch-buffer reuse had already removed the
  allocation pressure SmallVec was meant to save. Branch agent-klpol
  stays local; do not retry inline storage for KlPol in this shape.

## agent-klhash (merged; 0153e07+25c6d19)

- KlHashTable: std HashMap<KlPol, usize> -> open-addressing slot table of
  pool indices (slot = index+1, 0 = empty); the pool is the single owner
  of polynomial storage — a miss clones ONCE (was: pool copy + key copy),
  a hit is one hash + one pool compare. Interning order (= filekl index
  layout) unchanged; growth rehashes from the pool; new unit test pins
  first-seen order across growth rounds.
- Gates: quick_check 3680865 green (first chain exercised the new
  failure propagation: 3680854 correctly blocked on a missing Hasher
  import); probes 3680867-871 SORTED_MATCH; E7 3680873/874:
  av_ann_e7 15.62s, gkfast_e7 14.29s, RSS 690MB (was 780MB).
- AB 3680872 finals_e7_term20138 vs 0546294: base 9.12/9.16/9.24 tip
  8.99/8.95/9.00 — median 9.16s -> 8.99s = **1.019x**, every tip rep
  below every base rep; RSS -12% on the probe.

## agent-mixhash (merged; 62605e1)

- MixingHasher: fmix64 avalanche moved from EVERY write_u64 (3 multiplies
  per 8 bytes) to finish() (once per key); per-word round is the fx-style
  rotate-xor-multiply. The lane-D E8 blowup (job 3672155) came from having
  NO low-bit avalanche; finishing with fmix64 keeps that property.
- Gates: quick_check 3680892 green; 7 probes SORTED_MATCH incl. the
  kgb_e7_allforms involution-table canary (3680900, 0.31s, no blowup).
- AB 3680899 finals_e7_term20138 vs f17d907: base 9.23/9.13/9.06 tip
  8.99/9.22/9.05 — median 9.13s -> 9.05s = 1.009x, WITHIN NOISE; merged
  as neutral-but-principled (strictly less per-word work). Cross-job
  drift note: the same code measured 8.99 as klhash-tip in 3680872 and
  9.13 as base here — ±1.5% cross-job band; only within-job interleaving
  is trustworthy.

## agent-seqpar (merged; fc833aa)

- involution_orbits: the >=100-roots parallel driver now also requires
  available_parallelism > 1. SLURM probe slots run cpus-per-task=1, where
  two workers timeshare one core and duplicate the per-worker membership
  tables for nothing.
- Gates: quick_check 3680913 green; probes 3680915-919 SORTED_MATCH.
- AB 3680921 probe_setup_e6 at 1 cpu, REPS=5, vs d278e664: base median
  0.61s tip median 0.60s (wall ~neutral) BUT RSS 150MB -> 106MB (-30%)
  on the setup probe. Merged for the memory win; wall unchanged means the
  parallel driver was NOT the E6 setup gap.
- Setup-gap decomposition (probes 3680906/3680909/3680911): ic-only E6 =
  0.04s BOTH engines; + real_form/KGB/param = 0.05s rust vs 0.06s oracle
  (parity); the whole 0.23s gap is in loading the GK/AV script stack
  (nilpotent_orbits.at + associated_variety_annihilator.at + GKfast.at),
  whose load triggers ~28 classification builds incl. one E8-scale one.

## probe_unitary_e7_heavy: the 60x oracle claim was FALSE (Sep 4 artifacts)

- The rust leg (47.78s) had CRASHED at term 780 (exit 1, the since-fixed
  finals-invariant bug); the oracle leg was cancelled at the 1h limit past
  term 1540, also incomplete. It was half-work vs more-than-full-work —
  never a like-for-like comparison. Prefix through term 780 is
  byte-identical, so the completed prefix was CORRECT; the speed claim
  was the artifact. Partial captures saved as
  probe_unitary_e7_heavy.{rust.out.sep04-crash,cpp.out.sep04-partial}.
  The true oracle number awaits fat job 3680270; the post-deformfix rust
  leg is being re-measured (3680947).

## 2026-09-05b: dedup fix + closure/wallset slices; alcove cache slice rejected

All times HPC cpu partition (probe_diff, GNU time -v), oracle = C++ atlas.

Dedup fix 4cddc28 gate (3681101-07), all SORTED_MATCH:
- deform_e7_only 46118 lines; gkfast_e6/e7 3604; unitary_e7 2930;
  finals_e7_term20138 2935; klsum_e7_terms 44777; av_ann_e7 3606.

Probe wall times by frontier (rust seconds, oracle ~stable):
| probe | dedup 4cddc28 | alcove 2f0bf8d | closure 1477116 | wallset 1cd724c | av2 d7057b9 | oracle |
|---|---|---|---|---|---|---|
| deform_e7_only | 11.1 (earlier) | 10.93 | 10.45 | 10.39 | 10.93 | 10.4-10.9 |
| gkfast_e6 | — | — | 1.07 | 0.91 | 0.96 | 0.59-0.77 |
| gkfast_e7 | ~14.0 | — | 13.78 | 13.67 | 14.28 | 13.2-13.9 |
| unitary_e7 | 0.68 | — | 0.69 | 0.62 | 0.67 | 0.37-0.50 |
| finals_e7_term20138 | — | — | 9.29 | 9.35 | 9.60 | 8.40-8.61 |
| klsum_e7_terms | — | — | 10.09 | 10.14 | 10.45 | 8.65-8.82 |
| av_ann_e7 | 15.45 | 22.50 | 23.98 | 24.31/24.48 | 15.76/15.74 | 14.0-14.3 |

- av_ann_e7 +54% regression pinned to 2f0bf8d by same-job interleaved A/B
  (3681489: dedup 15.55-15.59 vs alcove 23.80-24.02 x3 reps, outputs
  byte-identical after sort; oracle 14.23). Release-flat perf
  (3681511 vs 3681730): mimalloc arena/bitmap/page cluster 7% -> 24.5%,
  vdso_clock_gettime 0 -> 2.2%; minor faults 14001 -> 19870 (3681789).
  No alcove symbol hot in either profile -> indirect mechanism (RootSystem
  OnceLock cache fields perturb allocator/codegen). Slice REJECTED; av2
  frontier = dedup + closure + wallset-without-caches restores 15.7s.
- heavy unitarity: cpu 2:30 legs 3681098/3681132 both TIMEOUT with zero
  output (batch stdout is whole-script buffered — main.rs:113). Fat 8h
  legs running: 3681407 (closure+alcove), 3681418 (wallset+alcove),
  3681939 (av2); oracle reference 3680270 (8h).

## 2026-09-05c: swar + fastgj + elimops gates (all 7/7 SORTED_MATCH)

Probe wall times (rust seconds) and rust/oracle ratio per gate run
(oracle times vary run to run; ratios are the comparable column):

| probe | swar a869688 | fastgj 2ce2d15 | elimops 328bfbd | oracle range |
|---|---|---|---|---|
| deform_e7_only | 10.89 (0.99x) | 9.97 (1.015x) | 10.99 (0.995x) | 9.8-11.1 |
| gkfast_e6 | 0.95 | 1.14 (1.93x) | 1.03 (1.34x) | 0.59-0.77 |
| gkfast_e7 | 14.10 (1.013x) | 13.17 (1.066x) | 14.12 (1.014x) | 12.4-13.9 |
| unitary_e7 | 0.63 | 0.73 | 0.64 | 0.36-0.50 |
| finals_e7 | 9.37 (1.10x) | 9.31 (1.11x) | 9.20 (1.098x) | 8.4-8.5 |
| klsum_e7 | 10.23 (1.16x) | 10.08 (1.17x) | 10.01 (1.16x) | 8.6-8.8 |
| av_ann_e7 | 15.44 (1.09x) | 16.29 (1.16x) | 16.50 (1.14x) | 14.1-14.5 |

- fastgj (SmallRat GJ in domain_builtins) clearly helps the
  rational-heavy probes: deform 10.9 -> 9.97, gkfast_e7 14.1 -> 13.17.
- elimops (in-place prevalidated integer_lattice row/col ops, no per-call
  replacement Vec, no pivot-pair clones): ratios stable to slightly
  better (gkfast_e6 1.93x -> 1.34x, finals/klsum/av_ann a touch down);
  absolute times within run-to-run noise of the fastgj gate.
- alcrat (ec80bd5, rebased 66f9a54): SmallRat sweeps for alcove.rs
  labels_for_component + solve_rational_system (profile 3682286:
  Rational::sub_assign 4.28% self, 2.24% under labels_for_component;
  mul 1.24% same path). Gate pending heavy-elimops (3682963) start.

## rowmask gate (3683288-96, 2026-09-05 evening, machine under heavy fat-leg load)

| probe | rowmask 509543a | oracle same-job | ratio |
|---|---|---|---|
| deform_e7_only | 12.18 | 14.23 | 0.856x |
| gkfast_e6 | 1.68 | 0.60 | (small, noisy) |
| gkfast_e7 | 15.34 | 13.87 | 1.106x |
| unitary_e7 | 1.15 | 0.38 | (small fixed cost) |
| finals_e7 | 9.88 | 8.39 | 1.178x |
| klsum_e7 | 10.27 | 8.77 | 1.171x |
| av_ann_e7 | 16.93 | 14.38 | 1.177x |

- rowmask (509543a, merged c1a4142): bitset candidate generation in
  additive_closure. Justified by closure histogram 3683272: avg 125
  members/call, 99.9% of calls >32 members, 9.4e9 probed pairs per 8min
  of the heavy E7 unitary probe. deform improves 15.81 -> 12.18s
  (-23% vs the sumtab8 round; first probe clearly faster than oracle).
- intcache (b2dd03d): memoize IntegralDatumTable::int_item by gamma —
  int_item is pure in gamma and RepTable::reduce calls it per query,
  while a unitarity run shares one infinitesimal character across all
  finals (profile 3682964: additive_closure 5.9% + wall_set ~5% cluster
  all under int_item). Gate 3683307-14.
- Upstream check: integral_datum_item::data(inv) (subsystem.cpp:220)
  constructs a fresh codec per call, so our per-call IntegralCodec::new
  in canonical_key matches upstream structure — no gap to close there.

## streamout + gj64 gates (2026-09-05 night; streamout 3683413-21, gj64 3683435-42; both 7/7 SORTED_MATCH)

| probe | streamout c879b94 | ratio | gj64 5a83105 | ratio |
|---|---|---|---|---|
| deform_e7_only | 11.21 / 13.42 | 0.835x | 15.95 / 15.55 | 1.026x |
| gkfast_e6 | 0.95 / 0.61 | (small) | 0.98 / 0.76 | (small) |
| gkfast_e7 | 14.33 / 13.95 | 1.027x | 14.28 / 14.08 | 1.014x |
| unitary_e7 | 0.65 / 0.42 | (small) | 0.73 / 0.50 | (small) |
| finals_e7 | 9.23 / 8.46 | 1.091x | 9.50 / 8.51 | 1.116x |
| klsum_e7 | 10.13 / 8.67 | 1.168x | 10.08 / 8.62 | 1.169x |
| av_ann_e7 | 16.41 / 14.20 | 1.156x | 16.45 / 14.20 | 1.158x |

- streamout (c879b94, merged d251678): ATLAS_STREAM_OUTPUT=1 streams
  printer output live so time-boxed heavy legs keep partial results;
  default behavior byte-identical. Perf-neutral on probes; deform leg of
  this round ran on an unloaded node (0.835x) vs gj64 round on a loaded
  one — cross-round absolute times not comparable.
- gj64 (5a83105, merged 27b6507): u64 fast path in SmallRat
  reduce_i128 (profile 3683299: gj_normalize_and_clear 9.04% self incl.
  inlined u128 gcd). Probe-level neutral (target is the heavy E7
  unitarity load); heavy leg 3683443 measures the increment there.
- frontier after both merges = 27b6507 (rowmask + intcache + streamout
  + gj64). Heavy streaming pair 3683422 (rust frontier1=c879b94) vs
  3683423 (oracle), 7h timeout each, decides the real E7 multiplier.

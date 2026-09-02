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

<<<<<<< HEAD
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
>>>>>>> 762b472 (docs: E7 oracle block_deform reference (1:39:27/5.2GB) + heavy unitary interim numbers)

# Parallel acceleration: controlled screening, not assumed speedups

The user extended the mathematical-validation goal to include computational
parallelism selected by HPC A/B tests. Correctness remains the first gate;
memory is a capacity/regression guardrail rather than the primary objective.

## First candidate: existing Rayon paths

Current Rust already uses Rayon for Weyl action permutations, twisted-orbit
conjugation and KGB window computation (`weyl.rs`, `weyl_transducer.rs`,
`inner_class.rs`, `kgb_graph.rs`). KGB computes windows of64 in parallel and
defers deterministic interning until after the pure computation. CLI worker
pool initialization honors the Rayon environment; it sets worker stack size
but does not hardcode worker count.

The existing mathematical observer explicitly sets RAYON_NUM_THREADS=1,
OMP_NUM_THREADS=1 and OPENBLAS_NUM_THREADS=1. Thus the independently reviewed
E7 KGB baseline (Rust median69.416s versus original1.35766s, main05625 build)
is a SERIAL benchmark. It is not evidence about current multicore scaling.
The first A/B uses the later mathematically repaired build3833716, not the old
PGO worktree, so new timings must not silently replace the historical baseline.

## Frozen experiment policy

- Same accepted source/binary, input, node and allocation for every arm.
- One original arm, Rust with1 requested Rayon thread (A), and the SAME Rust
  binary with4 requested Rayon threads (B); other thread libraries remain1.
- Four fresh-process paired rounds; order alternates original/A/B and
  B/A/original. No simultaneous arms, no padded workloads, no changed inputs.
- Initial workload is E7_kgb (catalog64), a known correct bare-core operation
  whose serial Rust execution previously took about one minute. Record actual
  60-600s membership for each arm; a faster parallel run can fall below a minute.
- 4 CPUs,8GiB allocation; each child retains the existing6GiB address-space
  cap and300s timeout. Record host/CPU model/affinity, exact input/source/binary
  hashes, requested thread count, wall seconds, GNU-time user/system CPU and
  peak RSS. Do not call the requested count measured active worker utilization.
- Every Rust arm must match the original's complete stdout/stderr and pass
  existing mathematical checks. Original output must stay byte-identical across
  rounds. Failure, timeout or output drift invalidates the speedup.

`hpc/math_parallel.py` produces separately labelled ratios: Rust1/Rust4 for
scaling, and original/Rust for comparison with C++. A fast parallel result can
still be slower than C++. The preliminary screening rule needs all four paired
speedups at least1.05, nonoverlapping elapsed-time ranges, and no more than2x
serial peak RSS. It is not a statistical significance test, and does not justify
a global default or blanket algorithm-performance claim. The filesystem cache
is not flushed; in-process state is fresh. Independent result review is still
required before declaring a selected optimization.

## Accepted scoped result: E7 KGB, build3833716

Experiment3834321 COMPLETE6:23; independent review3834377 COMPLETE1:01.
The reviewer runs54 tests (including14 new negative/positive A/B checks),
rehashes606 original/1212 Rust source files,264 scripts, both executables and
all36 raw output/metrics files. It independently checks four alternating
orders, successful exits, whole stdout/stderr within AND between rounds,
CPU affinity, GNU-time CPU/RSS and elapsed time, and recomputes every ratio.
An external sacct snapshot binds completion,4 CPUs and nodecu025. The trial
uses Xeon Gold6338, affinity54-57; requested threads are not measured worker
counts. No interpreter rerun was needed for review.

| Arm | Median wall seconds | Observed wall range | Maximum peak RSS (KiB) |
|---|---:|---:|---:|
| Current original |1.327984|1.312909-1.336919|41,656|
| Rust, Rayon1 |69.595196|69.224900-70.444554|1,471,876|
| Same Rust, Rayon4 |23.523671|23.379801-23.654732|1,744,768|

Paired serial/parallel speedup median2.958576 (range2.926471-3.013052),
four-thread efficiency73.96%; maximum-peak-RSS ratio1.185404 (+18.54%).
All four pairs pass the predeclared screening rule. Decision:
**SELECTED_FOR_THIS_WORKLOAD**. This selects the existing four-thread path for
this workload, not a newly implemented algorithm or a global default change.
The experiment's retained summary still says PROMISING_PENDING_REVIEW; the
separate review's `decision` records final scoped acceptance.

Crucially, Rust4 is still about17.7 times slower than original (ratio of
median wall times), and its maximum RSS is about41.9 times original. Rust1
is about52.4 times slower. CPU totals remain about69-70s for BOTH Rust arms:
parallelism distributes existing work; it has not removed that excess work.
Only Rust1 lies in the60-600s band here. Neither this case nor its speedup
establishes minute-scale original performance, D8 feasibility, general KGB,
FPP, unitarity, Hodge, AV-ann or associated-cycle acceptance.

Evidence:
- Experiment report SHA28edde93bc5092005001b84cbe17143a75896d7ae8c211778d95947b88beb7a0.
- Review report SHA537d2a1a986e2dcfc9ac341994eef4b90577e152d0ddcf3007620de55bb008ea,
  `tests/reference/hpc/math_parallel_review_2026_09_28.json`.
- Review stage `/public/home/majj/atlas-parallel-review-20260928.fL023YvY`,
  pin353ac2152f403ceeb7de3b9434992e14d9cc93e2d1a0ab8d0a5f26e34e2e1194.

## Accepted recalibration: direct-generation build3841502

Independent reviews3841670(E7) and3841690(D8) verify both four-round
experiments below. Each rehashes36raw artifacts,606original/1423Rust source
files,264scripts, binaries, metrics and scheduler provenance. Complete KGB
rows and all simple-root operations match original within AND between rounds.
Both decisions are SELECTED_FOR_THIS_WORKLOAD; no global default changed.

| Case | Original median s | Rust1 median s | Rust4 median s | Paired Rust1/Rust4 median | Max RSS original/Rust1/Rust4 KiB |
|---|---:|---:|---:|---:|---|
| E7 KGB |1.332328|12.795832|5.855221|2.183940|41664 / 242100 / 242060|
| D8 KGB |3.915221|32.004259|13.885711|2.303566|84484 / 510804 / 513368|

Rust4 remains approximately4.39x(E7)/3.55x(D8) slower than original.
All arms are BELOW60s: these are accepted bounded screening cases, not the
requested minute-scale performance coverage. The older69s E7 source/node
is different; do not present its ratio to this run as a controlled algorithm
A/B effect. These results validate eager517, NOT the pending lazy523 source.

E7 review SHA e049496c5559d67da40ab758b8ba58c2787934a99c4a0b3b214b008a863b1a1c;
D8 review SHA eaef5768366ad7df363975fff569687f93d6239017acf1c0f10ea82b5ac2d776.
Reports and stage/pin bindings are in math_direct_parallel_submission_2026_09_29.json
and math_direct_{e7,d8}_parallel_review_2026_09_29.json.

The reviewer's scope label now uses the actual case id, not hardcoded E7;
an additional regression covers both E7/D8 labels. Numerical rules unchanged.

## Submission history and remaining follow-ups

NEW2026-09-29 exact517release3841502 has two frozen recalibration/A-B jobs:
E7_kgb3841647 at atlas-direct-e7-parallel-20260929.TwcjbExQ and
D8_kgb3841648 at atlas-direct-d8-parallel-20260929.36ynzazc. These use the
unchanged four-round original/Rust1/Rust4 driver, full KGB inputs,300s per
arm,4CPUs/8GiB and6GiB child cap.59inputs are pinned per stage. They test
the wired direct-generation source, NOT lazy523, and require complete output
equality before ratios plus independent review. Recalibrate actual durations;
do not inherit old69s timings or pad a faster input to manufacture minute-scale
membership. The independent reviews above now accept these scoped results;
the submitted experiment reports retain their pre-review status.

UPDATE2026-09-29: direct generation is now wired in integration3841489,
and209capture3841497 confirms complete D8/E7/E8 Cartan inventories with
98whole positive streams,3gains/no losses. This is a NEW serial source and
does not inherit the old E7 scaling result above. Recalibrate workloads and
rerun original/Rust1/Rust4 with complete equality after source-bound KGB
revalidation; do not combine old69s timings with the new inventory timings.
See direct_twisted_generation_2026-09-29.md for exact scope and pending116gate.

The current [twisted-orbit source audit](twisted_orbit_search_2026-09-29.md)
identifies a separate all-Weyl conjugacy sweep and the original's generator
queue alternative. It also disproves assuming stable raw partition IDs:
enumerate returns HashSet iteration; external Cartan canonicalization is a
separate numbering layer. No optimization or speed result is accepted there.

1. Four-thread E7 scaling is now verified, but the CPU-work gap remains. Profile
   Weyl enumeration/orbit setup and allocation before adding more threads or
   selecting a new algorithm. Source inspection alone does not attribute the
   measured69-70 CPU seconds. Do not merely increase D8's enumeration cap.
2. Preserve deterministic external KGB numbering: parallelize read-only
   transition computation, then merge/intern in a fixed order. Test cross,
   Cayley, lengths, involutions and all output records, not just counts.
3. KL columns have dependency order. Parallelize only independent ready work
   after proving the dependency boundary; mutable polynomial interning and
   representation caches need owned/per-worker scratch plus deterministic
   merging, not an unsound shared mutable cache or unsafe Rust.
4. Distinguish running independent fixtures concurrently (suite throughput)
   from accelerating one mathematical calculation. Keep unitarity/Hodge/FPP,
   AV-ann and associated-cycle validation requirements unchanged.

Accepted parallel screening now covers the historical E7 case and the new
direct-generation E7/D8 cases above, not other mathematical operations.
Other stages and open work are indexed in HANDOFF and reference receipts.

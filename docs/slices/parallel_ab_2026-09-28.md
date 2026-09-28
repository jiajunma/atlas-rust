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

## Follow-up candidates, not implemented or accepted

1. Measure where existing1/4 scaling helps before adding more threads. If gains
   saturate, investigate sequential Weyl enumeration/orbit setup and allocation
   overhead; do not merely increase D8's enumeration cap.
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

No parallel speedup is accepted yet. Active stages/jobs are indexed in HANDOFF
and `tests/reference/hpc/math_parallel_*` receipts.

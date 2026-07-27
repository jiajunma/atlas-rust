# Upstream Atlas parallelism study

Question studied: does upstream Atlas have a multithreaded KGB enumeration on
some branch, and what should the Rust port learn from whatever multithreading
does exist? Study date 2026-07-27, upstream clone `~/mycodes/atlasofliegroups`
at master `4d3e9449`.

## Exhaustive branch scan result

All 60 `jeffreyadams/atlasofliegroups` branches and the 9 fork-only branches
of `jiajunma/atlasofliegroups` were scanned for `std::thread`, `<thread>`,
`std::atomic`, `std::mutex`, `pthread_create`, `#pragma omp`, `tbb::`,
`concurrent_`, and `hardware_concurrency`, plus a commit-message search for
thread/parallel.

There is no multithreaded KGB enumeration on any branch. `kgb.cpp`,
`kgb_base`, and `involutions.*` are untouched by threading everywhere; KGB
generation is sequential in every branch. Exactly two genuinely multithreaded
implementations exist, both filling the Kazhdan-Lusztig table, not KGB:

- `origin/crack-E8` (2006-07, pthreads): the historical E8 computation
  lineage; `kl.{h,cpp}-thread` was made the default on the branch
  (`7d520efa`, `5bf8d851`).
- `origin/parallel-KL` (modern, `std::thread`): threads
  `KL_table::silent_fill`, in staged commits `07fc3949` (reorganize),
  `233ee0a9` (group columns by length), `fd4c9595` (first threading),
  `6c526e15`/`b50a73b7` (thresholds), `bdab3066` (stratify shared reads),
  `abc40470` (centralized ticket distribution, branch tip).

Process-level parallelism also exists (Python `ProcessPoolExecutor` driving
whole atlas processes: `K_char.py`, `facets.py`, `Kpols.py`, branch
`FPP_jeff`), which parallelizes independent language-level computations, not
any single enumeration.

A memorable "multithreaded enumeration in some branch" is therefore
`parallel-KL`'s enumeration of KL columns; KGB itself has never been
threaded upstream.

## parallel-KL architecture (`kl.cpp` on `origin/parallel-KL`)

`KL_table::silent_fill` (kl.cpp:843-1011):

- Length-level barrier. The outer loop walks column lengths; the unfilled
  columns `ys` of one length are mutually independent because computing
  column `y` reads only columns of strictly smaller length, all committed
  before the level starts. Levels are separated by a full join.
- Sequential fallback. Fewer than four columns in a level are filled inline
  (kl.cpp:861); thread count is
  `min(ys.size(), hardware_concurrency())` (kl.cpp:964-966).
- Work distribution. A mutex-protected `distributor` hands out remaining
  column numbers as tickets (kl.cpp:894-907). Each worker starts with one
  assigned column and pulls a ticket each time it finishes one
  (self-scheduling; kl.cpp:948-953).
- Thread-local everything. Each worker owns a full-block working vector and
  accumulates its output as per-column `(x, KLPol)` lists, moving polynomials
  out of the working vector (kl.cpp:911-946). Workers never touch the shared
  polynomial hash table.
- Sequential commit. After joining, the main thread reaps workers in fixed
  thread order, interning polynomials into the shared hash and writing
  `d_KL[y]` (kl.cpp:990-1004). Deduplication is therefore serialized and the
  parallel phase is lock-free on shared data.
- Residual nondeterminism. Which worker processed which column depends on
  timing, so polynomial-store insertion order (hence `KLIndex` numbering)
  varies run to run; the set of stored polynomials and every `P_{x,y}` value
  do not. Atlas does not expose `KLIndex` at the language level, so observable
  behavior stays deterministic.

## crack-E8 architecture (`kl.cpp` on `origin/crack-E8`)

`Helper::fill` (kl.cpp:1562-1660) and `ThreadStartup` (kl.cpp:2516-2538):

- Same length-level barrier, with `NThreads` extra pthreads spawned per level
  and the main thread joining the workforce; mu-rows are then filled
  single-threaded per level.
- Work unit: under `YThread.mutex`, a worker takes the current `y` and
  advances the shared cursor past the whole same-orbit cluster of rows, then
  fills that cluster outside the lock (kl.cpp:2520-2533).
- `NThreads` defaults to 0 (fully sequential) and is set per command in
  test.cpp; threading was opt-in.
- Synchronization inventory: exactly two mutexes, the work cursor and a
  statistics guard (`stat_guard`). The shared polynomial hash table is NOT
  protected: concurrent `d_hashtable.match(...)` calls from `writeRow`
  (kl.cpp:2013, 2025) and the Thicket path (kl.cpp:2206, 2237) can insert and
  rehash concurrently. By modern standards this is a data race whenever
  `NThreads > 0`; the branch predates the C++11 memory model and was frozen
  as history, not merged.

## Implications for the Rust port

The upstream pattern that survived into the modern branch is the one worth
porting: level-synchronous parallelism with thread-local outputs and a
serialized, deterministic commit — not concurrent shared mutation. For this
repository:

- KL fill (future stage): `parallel-KL` maps directly onto a per-length
  `rayon` scope with per-worker output buffers and a sequential interning
  commit. Making the commit iterate columns in `y` order (rather than
  worker order) removes even the store-order nondeterminism upstream accepts,
  which keeps byte-identical reports cheap for differential testing.
- KGB enumeration (future stage): upstream offers no prior art, so a parallel
  design must come from the same pattern. KGB generation is length-graded
  (cross actions preserve or step length by one, Cayley transforms ascend),
  so a level-synchronous closure applies: expand the current length frontier
  in parallel into thread-local candidate lists, then dedup, canonically
  sort, and number sequentially at the barrier. Atlas KGB element numbering
  is observable language behavior, so parallel discovery must never leak
  discovery order; numbering must be a deterministic function of the level's
  canonical sort, with the Atlas numbering itself reproduced (or explicitly
  adapted) by the compatibility layer exactly as this repository already
  treats root order.
- Scale check before complexity: KGB sizes are modest (split E8 is about
  3.2e5 elements); blocks and KL tables carry the real cost. Parallel KGB is
  an optimization to justify with HPC measurements, not a prerequisite. The
  crack-E8 unlocked-hash race is the cautionary example of what skipping the
  commit-phase discipline buys.

These are design notes, not implemented behavior. Any parallel enumeration in
`atlas-real-group` or a future KGB crate still owes the standard stage
artifacts: a design document, three independent reviews, structural tests,
and an HPC preflight before any claim.

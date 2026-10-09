# Benchmark — Rust vs the real Atlas C++ (fair, same machine)

## Authoritative accepted HPC baseline — 2026-09-30

Job3868782 on one `cu315` node is the current controlled end-to-end baseline:
four fresh serial repetitions per arm, alternating order, identical inputs,
full result preservation,46 checker tests,13 focused tests, all629 core tests,
26 complete original histories,72 retained streams and final source/artifact
integrity. Times include startup and library loading.

| Complete input | Rust median | C++ median | Rust/C++ | Rust median RSS | C++ median RSS |
|---|---:|---:|---:|---:|---:|
| Rank1 unitarity parameters | 2.538780s | 0.277168s | 9.15971x | 44,118KiB | 12,192KiB |
| Finite AV-ann/cycle anchor | 3.208666s | 0.425888s | 7.53406x | 53,912KiB | 14,338KiB |

The accepted optimization improved the prior Rust medians by21.67% and25.52%
respectively, while RSS changed only+0.41%/+0.51%. Rust remains substantially
slower and uses about3.62x/3.76x the oracle RSS on these two inputs. These are
small-rank, loading-inclusive serial measurements—not kernel, multicore or
60--600s workload evidence. Ratios from older machines/nodes must not be
multiplied with them. Exact report:
`tests/reference/hpc/math_overload_command_after_r2_2026_09_30.json`, SHA
`fbc615d23c5d111aeea83ece1fecb6c67f7b37d6c0435c6af21e062a86d787fc`.

The next accepted measurement must wait for the parent seal and the
original-backed root-ladder BEFORE/repair/AFTER sequence, then reprofile the
same rank1 source. Current attribution points to repeated Weyl-context
construction during exceptional library initialization; this is a measured
lead, not yet an accepted optimization or parallel speedup.
UPDATE 2026-10-10: the named preconditions have since been met — the parent
seal (job3872554) and the root-ladder AFTER-v3 (job3875239) are both FINAL
and independently accepted — so the rank1 reprofile is unblocked by that
sequence.  It is still queued behind the current Weyl-semantic arc (G2-v1
capture) and the HPC tunnel; no new measurement exists yet.

## Diagnostic minute-scale KLV observation — not an accepted A/B

The complete complex-A6 regular-KLV fixture in job3851324 is a useful future
60--600s workload, but currently has only one loading/output-inclusive
observation per implementation:

| Input | Rust | C++ | Rust/C++ | Rust RSS | C++ RSS | RSS ratio |
|---|---:|---:|---:|---:|---:|---:|
| complex A6 complete raw/dual KLV | 69.806s | 26.983s | 2.587x | 1,403,840KiB | 413,876KiB | 3.392x |

The fixture also emits about341MB of output. Source review shows that Rust's
`raw_KL` matrix packing can hold three i32 `n*n` payloads concurrently; at
`n=5040` their payload alone is290.70MiB. This is a hypothesis boundary, not
an attribution: table storage, rendering and output buffering overlap, and the
run was neither repeated nor arm-alternated. Before optimizing, collect
per-phase timing/RSS plus table pool/index/clone/rebuild counts with probe-on/
off output equality. See `KL_CHAIN_TRACE.md` for the required counters and
candidate order.

## Historical local baseline — 2026-08-04

The remainder of this file is preserved as historical evidence. It predates
the current HPC source, workload and acceptance protocol and is not the current
Rust-versus-C++ ratio.

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

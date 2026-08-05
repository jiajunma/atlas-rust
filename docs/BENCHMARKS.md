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
| G2+D6 W_graph | 3.05s | 0.024s | ~127x |
| E6 W_graph | 1.5s | 0.028s | ~53x |

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

E6 W_graph probe: 13.7s → 1.5s (-89%). Timeline: rho-descent longest;
Arc datum; i64 compose; compact transducer Weyl layer (weyl_transducer.rs,
group orders + inverse + matrix-equivalence tests); parallel piece-matrix
materialization (rayon); parallel no-alloc action permutations with a
root-coordinate HashMap index; compact twisted-involution enumeration
wired into CartanClassification (orbit sweep needs the FULL enumeration,
not just the candidates — the discovery bug); and the final lever: the
orbit sweep's root permutations now compose from the compact enumeration
(simple-reflection perms → per-piece perms → parallel element perms), so
only the ~892 twisted involutions materialize matrices.
Memory (max RSS): Rust 69MB vs C++ 7.2MB (E6 W_graph) — 9.6x, down from
44x; the remaining gap is Vec/Arc nested allocation vs native arrays.
Next levers: classification caching (dual builds repeat), KGB build
parallelization.

## Fixture-level HPC ledger (swap 3516408, fat nodes)

The full differential suite is 189 fixtures, 0 FAIL (1 known PARTIAL),
all byte-identical. HPC wall times there include shared-node contention
and whole-fixture multi-row scripts, so they overstate the per-parameter
cost measured here; RSS is meaningful: kgb_hasse (E7) peaks at ~12GB.

# Brief A: port the KL_table (KLV polynomial engine) into a new crate module

You are working in `/Users/hoxide/mycodes/atlas-rust` (Rust reimplementation
of the Atlas of Lie Groups language). The language gate is complete (166
verified_hpc). The only remaining frozen contract is `domain/deform`, which
needs the Kazhdan-Lusztig-Vogan polynomial table (`kl::KL_table`).

## Scope discipline

- Only `crates/atlas-real-group/src/` — create new `kl_table.rs` and
  `kl_polynomial.rs` modules. Do NOT touch `tests/`, `hpc/`, `docs/`,
  events/meta files, or the language layer.
- No git commits. Leave edits in the working tree.
- No unsafe. Minimal, focused changes. Match existing module style
  (doc comments citing upstream file:line, the public API conventions
  of `block.rs` and `rep_context.rs`).

## What to build

A `kl_table` module that can store and compute KLV polynomials for a
single `BlockGraph`. The frozen fixture's A2 block has ≤6 elements, so
the polynomial table is tiny — you do NOT need to port the full
recursion engine (direct_recursion, nice_and_real, endgame_pair) if a
simple brute-force fill works for block size ≤12.

### Upstream anchors (read first)

- `sources/gkmod/kl.h` — the full `KL_table` class definition (181
  lines). Read it for the data-member layout and the public API.
- `sources/gkmod/kl.cpp` — implementation. Key entry points:
  - constructor (`KL_table::KL_table`, lines ~60-120)
  - `silent_fill` (lines ~150-300)
  - `fill_KL_column` (lines ~350-600) — the core recursion
  - `KL_pol(x,y)` and `mu(x,y)` — simple accessors
  - `complete_primitives` (~lines 700-850)
- `sources/gkmod/klsupport.h` — the `KLSupport` base class:
  block reference, length/status/descent queries. You can inline the
  few needed methods directly into `KL_table` (the base is thin).

### Required API surface

```rust
pub struct KLTable {
    block: BlockGraph,           // (owned clone for simplicity)
    columns: Vec<KLColumn>,      // one per block element
    mu_columns: Vec<MuColumn>,   // μ-coefficients
    pool: KLStore,               // polynomial storage (Vec<Vec<i32>>)
    holes: BitSet,               // columns yet to compute
    hash: KLHashTable,           // dedup map → pool index
}

pub type KlIndex = usize;       // index into the pool
pub type MuCoeff = i32;         // μ-coefficient (small integers)

impl KLTable {
    /// kl.cpp:60-120. Allocate columns = block.size(), holes = all.
    pub fn new(block: BlockGraph) -> Result<Self, StructureError>;

    /// Fill columns up to |limit| (inclusive). kl.cpp silents_fill.
    pub fn fill(&mut self, limit: BlockElt);

    /// kl.h:114. The KLV polynomial P_{x,y} as a pool index.
    pub fn kl_pol(&self, x: BlockElt, y: BlockElt) -> Option<KlIndex>;

    /// kl.h:119. The μ-coefficient μ(x,y).
    pub fn mu(&self, x: BlockElt, y: BlockElt) -> Option<MuCoeff>;

    /// kl.h:109. Bitmap of primitive elements for column y.
    pub fn primitives(&self, y: BlockElt) -> BitSet;
}
```

### BlockGraph API you can depend on

The crate's `BlockGraph` already exposes:
- `size() -> usize` — number of block elements
- `length(y) -> usize` — length of element y
- `status(y, generator) -> Option<KgbStatus>` — status per simple root
- `cross(y, generator) -> Option<BlockElt>` — cross action
- `cayley(y, generator) -> (BlockElt, Option<BlockElt>)` — Cayley pair
- `inverse_cayley(y, generator) -> (BlockElt, Option<BlockElt>)` — inverse
- `down_set(y) -> BitSet` — all x ≤ y in the Bruhat order
- `has_double_image(y, generator) -> bool` — for the recursion formula
- `dual_kl_index(y) -> BlockElt` — the dual element in the dual block

### Polynomial arithmetic

You need a minimal polynomial module (`kl_polynomial.rs`):
- `struct KlPol(Vec<i32>)` — a polynomial over ℤ (KLV polynomials have
  small integer coefficients, no overflow concern for A2).
- `KlPol::zero()`, `KlPol::one()`, `KlPol::constant(c)`
- `KlPol::add(&self, &KlPol) -> KlPol`
- `KlPol::sub(&self, &KlPol) -> KlPol`
- `KlPol::shift(&self) -> KlPol` — multiply by (1 + q): the recursion
  formula needs `p.shift() = p + q*p` (implement as element-wise
  shift + add). In the upstream `SafePoly` encoding this is a right-
  shift plus carry propagation, but for an in-memory Vec<i32> just do
  the operation directly.
- `KlPol::evaluate_at(q: i32) -> i32` — evaluate the polynomial at q.
  For q=-1: alternate sum of coefficients.

`KLHashTable`:
- `struct KLHashTable { pool: Vec<Vec<i32>>, map: HashMap<Vec<i32>, usize> }`
- `insert(pol: Vec<i32>) -> usize` — dedup and return index

### Fill algorithm (simplified for A2)

The three-phase recursion (first_direct_recursion, first_nice_and_real,
first_endgame_pair) is complex and NOT needed for the A2 block. For
A2 with block size ≤6, a direct algorithm is acceptable:

1. For each y from 0 to limit:
   - If y has no primitive elements: KL_pol(y,y) = 1, done.
   - Otherwise, compute primitive elements (descendants x < y where
     P(x,y) ≠ 0). For each primitive x:
     * Collect all x' where P(x',y) must be computed (the recursion
       column).
     * Apply the recursion formula: start from P(x, y-s) for a
       suitable simple root s, then propagate via cross/Cayley.

The upstream recursion_column (kl.cpp~400-550) is the entry point.
You can port it directly — it's ~150 lines and handles the three
cases (complex descent, real type II, imaginary type I/II).

### Test anchors

Write tests in a `#[cfg(test)] mod tests` at the bottom of `kl_table.rs`:

1. Construct the A2 compact block (same InnerClass/RealForm as the
   deform fixture) via the existing test helpers in `block.rs`
   (`pipeline`, `graph_with_size` with size 1 for the compact SU(3)
   form — actually A2 compact has KGB size 1, so the block is
   trivial...)。更实际的方式是使用现有的 A1 split 块进行测试：
   A1 compact 的 Quasisplit 块的 `dual_kl_index`。

等等，A2 compact 的 quasisplit 块的 KGB 大小是 6，而双块的 KGB 大小是 1（compact 只有 1 个元素）。所以 Block 大小 = 6 * 1 = 6。让我调整测试计划。

实际上：直接测试 fill(limit) 不 panic，kl_pol(y,y)==1 对所有 y，mu(y,y)==0。

### Verification

- `cargo test -p atlas-real-group --lib`
- `cargo clippy -p atlas-real-group --lib --tests -- -D warnings`
- `cargo fmt --all -- --check`
- These must all be clean. `cargo test -p atlas-core --lib` to prove no
  breakage.

## Report

- Files changed with line counts.
- The public API surface.
- Which upstream anchors map to which Rust code.
- Fill-then-read test for a real A2 block.
- Verification output tails.

# Brief B: port deformation_terms + deform_readjust into the crate

You are working in `/Users/hoxide/mycodes/atlas-rust`. Brief A (agent_deform_kl_core) ported `KL_table` as a new `kl_table.rs` module. Your
job is the second sub-slice: the deformation algorithm and the
`deform_readjust` helper, both into `rep_context.rs` or a new
`deformation.rs` module (follow existing module patterns).

## Scope discipline

- Only `crates/atlas-real-group/src/`. Extend `rep_context.rs` or add a
  new `deformation.rs` (export through `lib.rs`). Do NOT touch `tests/`,
  `hpc/`, `docs/`, events/meta files, or the language layer.
- No git commits. Leave edits in the working tree.
- No unsafe. Match the doc-comment style of `block.rs`/

## What to build

### 1. deform_readjust (repr.cpp:622-654)

A method on `RepContext` (or a free function) that adjusts a
`StandardRepr` for the `deform` algorithm: make `gamma` dominant while
exhausting singular complex descents.

```rust
impl RepContext<'_> {
    /// repr.cpp:622-654. Make gamma dominant and exhaust singular
    /// complex descents, in-place on `z`.
    pub fn deform_readjust(&self, z: &mut StandardRepr) -> Result<(), StructureError>;
}
```

Algorithm:
```
let datum = self.inner_class.datum();
let mut lr = self.lambda_rho(z)?;
let mut numer = z.gamma.numerator().to_vec(); // Ratvec_Numer_t
loop {
    let mut changed = false;
    for s in 0..datum.semisimple_rank() {
        if self.kgb_status(z.x(), s)? != KgbStatus::Complex { continue; }
        let eval = simple_coroot_numerator_pairing(s, &numer)?; // <gamma, alpha_s^v>
        if eval < 0 {
            self.simple_reflect_numerator(s, &mut numer)?;
            self.simple_reflect(s, &mut lr, 1)?;
            z.x = self.cross_at(z.x, s)?;
            changed = true; break;
        } else if eval == 0 && self.is_complex_descent(z.x, s)? {
            self.simple_reflect(s, &mut lr, 1)?;
            z.x = self.cross_at(z.x, s)?;
            changed = true; break;
        }
    }
    if !changed { break; }
}
z.gamma = RationalWeight::new(numer, z.gamma.denominator())?;
z.y_bits = self.y_pack(self.involution_of(z.x)?, &lr)?;
```

This is written in terms of existing RepContext API (`simple_coroot_numerator_pairing`,
`simple_reflect_numerator`, `simple_reflect`, `cross_at`, `y_pack`, `involution_of`).

### 2. deformation_terms (repr.cpp:1933-2025)

A method on a new `RepTable` struct or as a free function. The frozen
contract exercises the simplest case: identity block_modifier, empty
singular generators. You can implement ONLY that case.

```rust
/// repr.cpp:1933-2025 (simplified: bm=identity, singulars empty).
/// Returns (StandardRepr, SplitValue) terms.
pub fn deformation_terms(
    block: &BlockGraph,
    y: BlockElt,
    gamma: &RationalWeight,
    kl_table: &KLTable,
) -> Result<Vec<(StandardRepr, SplitValue)>, StructureError>;
```

Simplified algorithm (no block_modifier, no contributions/singulars):
1. If block.length(y) == 0 → return empty
2. List final elements: for each z in block, if no singular complex
   descents exist for z → final. Accumulate in reverse order.
3. Fill kl_table ||to y.
4. Initialize acc = remainder = vec![0; finals.len()]; remainder[0] = 1.
5. For each final z (from y down to 0):
   - If remainder[pos] == 0 → continue
   - For each x ≤ z (descending):
     - eval = kl_pol_evaluate_at_minus_one(kl_table.kl_pol(x,z))
     - If eval == 0 → continue
     - If (block.length(z)-block.length(x)) % 2 != 0 → eval = -eval
     - remainder[j] += c_cur * eval (triangular)
     - acc[j] += c_cur * eval (contribute if length parity differs)
6. Convert acc to (StandardRepr, SplitValue) pairs: for each final z
   with c != 0, compute `block.sr(z, gamma)` (needs a way to build
   StandardRepr from BlockElt + gamma — use the block's parameter
   construction, see block.rs).

### Orientation number

The orientation number is `orientation_number(sr)` — defined as 0 in
the simplest case (no compact Cartan, no real roots). For the A2
quasisplit block, it IS 0. For the MVP, return constant 0.

### Dependencies

- `BlockGraph::sr(element_index, gamma)` or a way to reconstruct
  `StandardRepr` from a block element.
- Check if `BlockGraph` already has a method to produce `StandardRepr`.
  If not, add `BlockGraph::sr(y: BlockElt, gamma: &RationalWeight) ->
  Option<StandardRepr>`.
- `KLTable` from brief A.

## Test anchors

1. Construct the A2 quasisplit block (same as the deform fixture).
2. Call inspect the final elements: verify the elements at indices
   0-5 include some finals.
3. Call `deform_readjust` on a non-dominant `StandardRepr` and verify
   it becomes dominant.
4. Call `deformation_terms` on each final element and verify the
   result is non-empty for non-zero-length elements.

## Verification

Same three-piece gate + prove atlas-core tests unaffected.

## Report

- Files changed, new API signatures, upstream anchor mapping.
- Test results for the A2 block.

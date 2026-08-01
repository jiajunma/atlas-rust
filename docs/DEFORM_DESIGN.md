# Deform slice design — the KL/deformation layer

## Overview

The language gate is complete (166 verified_hpc). The only remaining
frozen contract is `domain/deform` (reference captured by job `3506415`,
A2 compact inner class, 3 `deform(Param)` calls → 9 events). Deform is
the Kazhdan-Lusztig-Vogan deformation: `deform(p)` produces an `SR_poly`
(ParamPol) whose terms are the deformation formula of `p`.

The implementation needs KL polynomial calculation (kl.cpp) plus the
deformation machinery (repr.cpp). Three sub-slices, in order:

1. **KL_table** — the KLV polynomial table (kl.cpp/h)
2. **deformation_terms + readjust** — the deformation algorithm (repr.cpp)
3. **Language wiring** — build the domain value and register the builtin

## Repo map

- **Crate to extend**: `crates/atlas-real-group` (new modules:
  `kl_table.rs`, `kl_support.rs`; extend `rep_context.rs`)
- **Language layer**: `crates/atlas-core/src/domain_builtins.rs` +
  `typed.rs`
- **Upstream reference**: `sources/gkmod/kl.{h,cpp}` (KL_table),
  `sources/gkmod/repr.{h,cpp}` (deformation_terms, deform_readjust,
  deformation_unit, Rep_table), `sources/interpreter/atlas-types.w`
  (deform_wrapper at line 8084)
- **Fixture**: `tests/fixtures/domain/deform.atlas` (3 A2 deformations)
- **Existing pieces**: Pol term math (`finals_for`, `K_type_formula`
  wrappers) is in the crate; SR_pol/K_type_pol containers are in
  `domain_builtins.rs`. `BlockGraph` (block.rs) has the status/cross/
  Cayley/block_length/descent surface. `RepContext` has sr/sr_gamma/
  sr_K/lambda_rho/gamma/height/predicates.

## Sub-slice A: KL_table core (agent_deform_kl_core_prompt.md)

**Goal**: port `kl::KL_table` (kl.h:68-173, kl.cpp:1-1061) into a new
crate module `kl_table.rs`.

**Required surface** (the minimum the deformation algorithm needs):
- Constructor `KL_table::new(&BlockGraph)` — allocate columns,
  initialise storage_pool with 0/1
- `fill(BlockElt limit)` — compute KL polynomials for columns up to
  `limit` (the public face of silent_fill/verbose_fill)
- `KL_pol(x, y) -> KLPol` — the KLV polynomial P_{x,y} as an index into
  the storage pool
- `mu(x, y) -> MuCoeff` — the μ-coefficient μ(x,y)
- `primitives(y) -> BitMap` — primitive elements for column y (those x
  with nonzero KL_pol(x,y) and no earlier y' leading to same term)

**Dependencies** (existing or new):
- `BlockGraph` (block.rs) — size, length, status, cross, Cayley,
  inverse_Cayley, down_set, has_double_image, dual_KL_index
- `Pol` (new, minimal polynomial over ℤ) — storage for
  `SafePoly<KLCoeff>`, `KLPol`, `KLIndex`, `MuCoeff`, plus polynomial
  arithmetic (add, subtract, multiply by 1+q, shift). Use the existing
  `malachite::Integer` and keep coefficients 32-bit ints (the KLV
  algorithm produces small coefficients; the upstream uses `KLCoeff` =
  unsigned short in some builds, u32 in others).
- `KL_hash_Table` (new, minimal) — a deduping store that maps
  polynomial content to a `KLIndex`. A simple HashMap<Vec<i32>, KLIndex>
  wrapping the `KLStore` pool vector is sufficient for a single block.

**Test anchors**: the A2 block sizes (3 KGB elements for su(2,1), 1 for
compact) are tiny; KLV polynomials for A2 are trivial (most are 1). The
in-crate tests can verify:
- `KL_pol(y, y) == 1` for every y
- `mu(y, y) == 0` (by definition)
- Fill up to some limit without panicking

**Exclusions** (for a later slice): `first_direct_recursion`,
`first_nice_and_real`, `first_endgame_pair`, `wGraph` — these are
optimisation/support functions not needed for the tiny A2 block.

## Sub-slice B: deformation_terms + readjust (agent_deform_terms_prompt.md)

**Goal**: implement `deformation_terms` (repr.cpp:1933-2025) and
`deform_readjust` (repr.cpp:622-654) in the crate.

**deformation_terms** — given a BlockGraph, a final element `y`, a
block_modifier `bm`, and a dominant `gamma`, returns `Vec<(StandardRepr,
SplitValue)>` of deformation terms. The algorithm:
1. If block.length(y) == 0 → empty result.
2. Compute block contributions (via `contributions`, repr.cpp~1890)
   and list of final elements.
3. Build KL_table, fill to `y`.
4. Accumulate remainder/acc over finals, evaluating KL polynomials at
   q=-1 (alternating sign for odd-length differences).
5. Scale by orientation_number differences (Split_integer coefficients).

The block_modifier and contributions/block_singular can be simplified
for the frozen contract: the three A2 parameters deform with block size
2 or 3, the modifier is identity, and the singular generators are empty
(su(2,1) has no compact simple root). A simplified implementation that
handles the `bm=identity, singulars=empty` case first is acceptable.

**deform_readjust** (repr.cpp:622-654) — a variant of `made_dominant` that
also exhausts singular complex descents. The crate already has
`StandardRepr::made_dominant` (repr.cpp:1507-1561) and
`RepContext::complex_descent_w`/`complex_crosses`. The readjust loop is:

```
for s in 0..rank:
  if status(s) == Complex:
    eval = <gamma, alpha_s^v>
    if eval < 0: reflect gamma, lr; cross s; break
    elif eval == 0 and isDescent(s): reflect lr; cross s; break
```

**Exclusions**: block_modifier (for non-identity cases), full
`contributions` (for non-trivial singular generators), twisted_deform,
block_deform_to_height.

## Sub-slice C: Language wiring (agent_deform_lang_prompt.md)

**Goal**: register `deform(Param) -> ParamPol` in the language layer.

**Work items**:
1. In `domain_builtins.rs`: add a `DomainValue::ParamPol(SRPolyValue)`
   variant (if not already present) or reuse the existing
   ParamPol/KTypePol containers. The frozen contract's output is three
   `SRPolyValue` values (empty for the trivial case, two-term for the
   non-trivial ones).
2. In `typed.rs`: register `deform` as `domain_builtin("deform",
   Prim::Param, Prim::ParamPol, ...)` in `builtin_registry()`.
3. In `domain_builtins.rs`: implement the `deform` evaluator, which
   calls `finals_for`, `Rep_table` lookup (or a simplified path), and
   `deformation_terms`, then wraps into the language ParamPol container.

**Note**: the frozen contract exercises only the A2 compact inner class
(`ic := inner_class(rd,[[1,0],[0,1]])`) with the quasisplit su(2,1)
real form (`rf := real_form(ic,1)`). The KGB has 6 elements; the three
`deform` calls use KGB elements 3, 4, and 5.

## Workflow (same as all slices)

1. Read the per-slice brief in `docs/slices/`
2. Implement → local gate → check_fixture → full pipeline replay
3. HPC differential (sync HEAD, submit `pipeline_swap_diff.sbatch`)
4. Meta upgrade → commit

## Per-slice delivery loop (unchanged)

`cargo test -p atlas-core --lib`, `cargo test -p atlas-real-group --lib`,
`cargo clippy ...` , `cargo fmt --all -- --check`, `cargo build -p atlas-cli`,
`python3 /tmp/check_fixture.py domain/deform`, full local pipeline replay,
`python3 hpc/test_pipeline_swap_diff.py` → wire into pipeline → sync HPC →
submit differential → bump meta → record in HANDOFF → commit.

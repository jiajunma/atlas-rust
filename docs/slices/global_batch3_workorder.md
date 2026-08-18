# global.w batch 3 work order — linear algebra builtins (recon 2026-08-19)

Dispatch AFTER batch 2 lands (both touch typed.rs). Recon: agent-67.

## Upstream inventory (global.w rev 4d3e9449; helpers in
## sources/utilities/matreduc.{h,cpp}, matrix.cpp, structure/lattice.cpp)

| Builtin | Signature (install) | Wrapper | Returns |
|---|---|---|---|
| `gcd` | `(vec->int)` :5200 | :4820-4828 | d ≥ 0; `gcd([])`=`gcd([0,0])`=0. **Claimed by batch 2** (`ScalarOp::VectorGcd` in its dirty diff) — only add if batch 2 drops it |
| `Bezout` | `(vec->int,mat)` :5201 | :4830-4841 | `(d, C)`, C unimodular, `v*C=[d,0,…]`; det(C) may be −1 |
| `echelon` | `(mat->mat,mat,[int],int)` :5202 | :4848-4865 | `(E, C, pivots, flip)`; E has zero columns REMOVED (rank columns), pivots ascending, kernel columns rotated right in C, flip = sign det(C) |
| `linear_solve` | `(mat,vec->|vec,int,mat)` :5203 | :4891-4923 | union `empty_set.()` / `affine_subspace.(sol,factor,ker)`; needs the union type declarable — check basic.at first |
| `diagonalize` | `(mat->vec,mat,mat)` :5204 | :4934-4947 | `(diagonal, row, column)` — diagonal FIRST; entries positive except possibly first; row/col det +1 |
| `adapted_basis` | `(mat->mat,vec)` :5205 | :4949-4959 | `(B, diagonal)`; image(M) = span{dᵢ·B.col(i)}; diagonal NOT divisibility-ordered |
| `kernel` | `(mat->mat)` :5206 | :4975-4979 | m×(m−rank), columns span ker over ℤ; basis order oracle-defined (echelon recorder block) |
| `eigen_lattice` | `(mat,int->mat)` :5207 | :4981-4987 | `kernel(M−λI)`; NO square check; diagonal touch up to min(rows,cols) |
| `row_saturate` | `(mat->mat)` :5208, hunger 3 | :4989-4993 | `adapted_basis(Mᵀ)` rows |
| `Smith` | `(mat->mat,vec)` :5209 | :5000-5010 | `(B, inv_factors)`; factors positive, dᵢ\|dᵢ₊₁; zero matrix → `(id, [])`; B non-unique — capture oracle bytes verbatim |
| `invert` | `(mat->mat,int)` :5210 | :5017-5032 | `(N, d)`, N/d = M⁻¹, d = bigint lcm > 0; **singular square returns zero matrix + d=0, NO error**; non-square throws BEFORE no-value gate: `"Cannot invert a 2x3 matrix"` |
| `mod2_section` | `(mat->mat)` :5211 | :5043-5053 | GF(2) BinaryMap::section — DEFER to batch 4 |
| `subspace_normal` | `(mat->mat,mat,mat,[int])` :5212 | :5062-5102 | GF(2) inline echelon; validates `D>64`/`N>64` before gate — DEFER to batch 4 |

Validation before the no-value gate ONLY for `invert` (non-square),
`linear_solve` (size mismatch `"Linear system size mismatch R:S"`),
`subspace_normal`. Everything else: any mat/vec accepted including 0×0.

## Reuse map

- `atlas-real-group/src/matreduc.rs` (private): faithful wrapping-i32 ports
  of `gcd`, `diagonalise` (exact sign bookkeeping), `has_solution`,
  `find_solution`, `inverse_upper_triangular`. NO echelon/adapted_basis/Smith.
- `atlas-real-group/src/integer_lattice.rs`: `adapted_basis` over
  arbitrary-precision Integer with budget plumbing (pivot-order faithful).
- NOTHING has `column_echelon`/`echelon_solve`/`Smith_basis`/`lattice::kernel`.
- Do NOT reuse `domain_builtins::invert_integer_matrix` (BigRational
  Gauss-Jordan; denominator contract differs from upstream).
- Cleanest: new `crates/atlas-core/src/matreduc.rs` on
  `linear_values::Matrix` (column-major i32, wrapping), op-for-op from
  upstream, cross-checked against real-group matreduc.rs.

## Plumbing patterns (from batch 1)

`scalar_builtin(name, arg, result, hunger, ScalarOp::X)`; multi-return via
`Value::Tuple` + `at_builtin_level`; validation-before-gate via
`runtime(msg, span)` before the gate (copy `ScalarOp::NullMatrix`);
bigint results via `Value::Integer(BigInt)`; tuple destructuring in
assignment already unpacks (`set (E,C,p,f):=echelon(M)` just works).

## Slice order (fixture first, HPC capture, implement, differential)

1. `echelon` + `kernel` + `eigen_lattice` (one engine: `column_echelon`).
   Fixtures: `echelon(mat: [[1,2],[3,4]])`, `echelon(mat: [[2,4],[4,8]])`,
   `echelon(null(0,0))`, `kernel(mat: [[1,2],[2,4]])`,
   `kernel(mat: [[1,0],[0,1]])`, `eigen_lattice(mat: [[2,1],[1,2]], 1)`.
   No rejected cases exist upstream for these — do not invent any.
2. `invert` (needs `echelon_solve`). Fixtures: `invert(mat: [[1,2],[3,4]])`,
   `invert(id_mat(3))`, `invert(mat: [[1,2],[2,4]])` (PIN: zero matrix + 0,
   no error), `invert(null(0,0))`; rejected `invert(null(2,3))` +
   discarded-value variant to pin validation-before-gate.
3. `Smith` + `adapted_basis` + `diagonalize` (gcd-with-recorder +
   adapted_basis + Smith correction loop matreduc.cpp:369-381).
   Fixtures: `Smith(mat: [[2,0],[0,3]])` → factors `[1,6]`,
   `Smith(mat: [[4,6],[3,9]])`, `Smith(null(2,3))`, `Smith(mat: [[0]])`,
   `adapted_basis(mat: [[2,4],[4,8]])`, `diagonalize(mat: [[2,0],[0,3]])`,
   `diagonalize(mat: [[-2]])` (first-diagonal sign case — oracle decides).
4. `Bezout` (same gcd recorder). Fixtures: `Bezout([6,10,15])`,
   `Bezout([])`, `Bezout([-6,9])`.
5. `linear_solve` only if the `|vec,int,mat` union with `empty_set`/
   `affine_subspace` injectors is declarable; otherwise defer with a note in
   docs/REMAINING_BUILTINS.md.
6. Batch 4 (deferred): `mod2_section`, `subspace_normal` (GF(2)).

Universal traps: wrapping-i32 arithmetic to match upstream's overflow
regime; zero-size matrices legal except where checked; all expected outputs
from HPC oracle capture, never hand-written matrices.

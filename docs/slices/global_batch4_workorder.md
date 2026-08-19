# global.w batch 4 work order — matrix slicer + GF(2) builtins (recon 2026-08-18)

Dispatch AFTER batch 3 (landed; fat differential 3575810, 295 PASS + 1 declared PARTIAL; `crates/atlas-core/src/matreduc.rs` is now committed). Recon: subagent, read-only. All oracle outputs below are verbatim captures from the pinned local oracle (rev 4d3e9449, NDEBUG build).

## 0. Sweep verdict (item 5 of the brief)

global.w makes **164 registry insertions**: 160 `install_function` + 4 `install_special_function` (`-`(int->int) :2966, `+`/`-` (int,int) :2971/:2978, `/`(int,int->rat) :3231). After batch 3, a name+signature diff of global.w against the Rust registry (`typed.rs:5697-6766`, 159 `scalar_builtin`/`hidden_scalar_builtin` entries) leaves **exactly 6 unported signatures**, each a distinct name:

| # | Name | Install site | Batch-4 disposition |
|---|---|---|---|
| 1 | `swiss_matrix_knife` `(int,mat,int,int,int,int->mat)` | global.w:5195-5196 | **port** |
| 2 | `mod2_section` `(mat->mat)` | global.w:5211 | **port** |
| 3 | `subspace_normal` `(mat->mat,mat,mat,[int])` | global.w:5212-5213 | **port** |
| 4 | `"matrix slicer"` (hidden, trailing-space twin of #1) | global.w:5197-5198 | skip — see §2.4 |
| 5 | `"transpose "` (hidden, trailing-space) `(mat->mat)` | global.w:5188 | skip — see §4 |
| 6 | `readline_completions` `(string->[string])` | global.w:4390-4391 | documented exclusion — see §5 |

No other unported global.w signature exists. (The historical "89" was the 2026-08-18 gap count; batches 1-3 eroded it to these 6. The sweep is name-level; overload-level coverage of shared names was differential-verified by batches 1-3.) `basic.at` calls `swiss_matrix_knife` directly (basic.at:445, :492, :994 — flag arithmetic comments there corroborate the bit map below), so #1 blocks source-level basic.at compatibility; #2/#3 are not used by basic.at.

## 1. Reserved

## 2. `swiss_matrix_knife` + hidden `"matrix slicer"`

### 2.1 Anchors and semantics

- Wrapper: global.w:4714-4741; worker `transform_copy<transpose,negate>` global.w:4675-4702; bounds-check chunk :4745-4772; dimension chunk :4777-4785; template dispatch :4793-4809.
- Call shape: `swiss_matrix_knife(flags, M, i, k, j, l)` — rows are `i:k`, columns are `j:l` (wrapper pops `l,j,k,i,M,flags`, global.w:4715-4721). Half-open ranges, 0-based.
- Flag **bitfield** (BitSet<8>, global.w:4704-4710):

| bit | mask | meaning |
|---|---|---|
| 0 | 0x01 | reverse row order in output |
| 1 | 0x02 | row lower bound from end: `lwb_r = m - i` |
| 2 | 0x04 | row upper bound from end: `upb_r = m - k` |
| 3 | 0x08 | reverse column order in output |
| 4 | 0x10 | column lower bound from end: `lwb_c = n - j` |
| 5 | 0x20 | column upper bound from end: `upb_c = n - l` |
| 6 | 0x40 | transpose (result dims swapped BEFORE copy) |
| 7 | 0x80 | negate every entry |

- After bound resolution: `lwb>upb` clamps to empty (`upb=lwb`, :4778-4781); result is `(upb_r-lwb_r)×(upb_c-lwb_c)`, swapped if bit 6. Copy loop reverses rows/columns per bits 0/3 (passed as `rev_flags = flags[0]*0x1 ^ flags[3]*0x2` — `^` on disjoint bits, equivalent to `|`) and applies `-1 *` per entry if bit 7 (C++ `int` arithmetic → use **wrapping i32 negate** in Rust; `M = [[-2147483648]]` with bit 7 is the overflow corner).
- `flags` is read via `int_val()` into `BitSet<8>`: **no range check, no negativity check** — value is reduced to its low 8 bits; `-1` sets all bits (probed: `swiss_matrix_knife(0-1, M, 0,2,0,3)` on a 2×3 gives `The 0x0 matrix`, since bits 1+2 send `lwb_r=m`, `upb_r=m-2` and the clamp fires). `256` ≡ `0` (probed, identity slice).
- The four bounds are read via `ulong_val()` on the big-int: negative bound throws `"Negative integer where unsigned is required"`; bound > u64::MAX throws `"Integer value to big for conversion"` (upstream typo "to big", verbatim, probed).

### 2.2 Error texts (verbatim, probed) and validation order

Bounds check (global.w:4747-4771) runs **after all six arguments are evaluated, BEFORE the no-value gate** (:4727). Probed discarded-context: `begin swiss_matrix_knife(0, mat: [[1,2],[3,4]], 0, 5, 0, 2); 7 end` → errors, nothing else evaluated. Texts (on 2×2 / 3×2 matrices; note **no space after "are"**, space after the comma):

- `Range exceeds bounds: upper row bound 3 out of range, actual limits are2, 2`
- `Range exceeds bounds: lower row bound 5 out of range, actual limits are2, 2`
- `Range exceeds bounds: upper column bound 3 out of range, actual limits are3, 2`
- `Range exceeds bounds: lower column bound 5 out of range, actual limits are2, 2`
- `Range exceeds bounds: both row bounds 5,9 and upper column bound 7 out of range, actual limits are2, 2`
- `Range exceeds bounds: upper row bound 9 and upper column bound 8 out of range, actual limits are2, 2`
- `Range exceeds bounds: both row bounds 5,9 and both column bounds 3,7 out of range, actual limits are2, 2`

Selection logic: `r = m<max(i,k)`, `c = n<max(j,l)`; within each dimension, if the upper bound is out then "both ... I,K" when the lower is also out else "upper ...", else "lower ...". Trigger condition uses the **raw (pre-negation) bounds** — the from-end bits do not relax the check.

### 2.3 Probes (oracle outputs verbatim)

`M = mat: [[1,2],[3,4],[5,6]]` prints as the 2×3 matrix `| 1, 3, 5 | / | 2, 4, 6 |` (mat literals are COLUMNS).

- `swiss_matrix_knife(0, M, 0, 2, 0, 3)` → `| 1, 3, 5 | / | 2, 4, 6 |` (identity)
- `swiss_matrix_knife(64, M, 0, 2, 0, 3)` → `| 1, 2 | / | 3, 4 | / | 5, 6 |` (transpose)
- `swiss_matrix_knife(128, M, 0, 2, 0, 3)` → `| -1, -3, -5 | / | -2, -4, -6 |` (negate)
- `swiss_matrix_knife(129, M, 0, 2, 0, 3)` → `| -2, -4, -6 | / | -1, -3, -5 |` (rev-rows + negate)
- `swiss_matrix_knife(1, M, 0, 2, 0, 3)` → `| 2, 4, 6 | / | 1, 3, 5 |` (rev-rows)
- `swiss_matrix_knife(8, M, 0, 2, 0, 3)` → `| 5, 3, 1 | / | 6, 4, 2 |` (rev-cols)
- `swiss_matrix_knife(2, M, 1, 2, 0, 3)` → `| 2, 4, 6 |` (row lower from end: `lwb_r = 2-1`)
- `swiss_matrix_knife(4, M, 0, 1, 0, 3)` → `| 1, 3, 5 |` (row upper from end: `upb_r = 2-1`)
- `swiss_matrix_knife(16, M, 0, 2, 1, 3)` → `| 5 | / | 6 |` (col lower from end: `lwb_c = 3-1`)
- `swiss_matrix_knife(32, M, 0, 2, 0, 1)` → `| 1, 3 | / | 2, 4 |` (col upper from end: `upb_c = 3-1`)
- `swiss_matrix_knife(192, M, 0, 2, 1, 3)` → `| -3, -4 | / | -5, -6 |` (transpose+negate, cols 1..3)
- `swiss_matrix_knife(0, M, 2, 0, 0, 1)` → `The 0x1 matrix` (clamped empty, shape kept)
- `swiss_matrix_knife(64, M, 2, 0, 0, 1)` → `The 1x0 matrix` (transpose of empty)
- `swiss_matrix_knife(0, null(0,0), 0, 0, 0, 0)` → `The 0x0 matrix`
- `swiss_matrix_knife(255, null(2,3), 0, 0, 0, 0)` → `The 0x0 matrix`

### 2.4 The hidden `"matrix slicer"` copy

Reachable from the upstream parser, not nameable: parser.y:660-682 and :683-705 lower the **two-dimensional slice syntax** `M[i:k, j:l]` / `expr[i:k, j:l]` (each bound optional, optional trailing `~` per bound) to a `"matrix slicer"` call. The parser only ever sets bits 0x2/0x4/0x10/0x20 (from-end bits, `r_l_rev^r_u_rev^c_l_rev^c_u_rev`); an absent upper bound is compiled as literal `0` **with** the from-end bit (⇒ `upb = dim`), an absent lower as `0` without the bit — exactly the normalization Rust's 1-D `slice_suffix` already performs (syntax.rs:1852-1853). Probed upstream: `M[0:2, 0:3]` → whole matrix; `M[:,:]` → whole matrix; `M[1~:0~, 0:1~]` → `| 2, 4 |`.

Rust status: the parser has only the 1-D row slice (`PostfixSuffix::Slice` holds a single lower/upper pair, syntax.rs:548-553); `M[0:2, 0:3]` is a **parse error** today — a parser-level gap, independent of the registry. Conclusion: skipping the hidden `"matrix slicer"` registration is behavior-preserving; the 2-D slice syntax is a separate follow-up (if ever ported, it should call the same engine as `swiss_matrix_knife`, not a second registration). Record the gap in `docs/REMAINING_BUILTINS.md`.

## 3. GF(2) builtins

### 3.1 `mod2_section` `(mat->mat)` — global.w:5043-5053, install :5211

Semantics: converts the input to `BinaryMap = BitMatrix<64>` (entries `(x&1)!=0` — negative odd entries are 1; bitvector.cpp:145-154), computes `BitMatrix<64>::section()` (bitvector.cpp:346-405), returns it as a 0/1 int matrix of **transpose shape** (n_cols × n_rows). `section()` finds B with `ABA=A` and `BAB=B` over GF(2): copy A's columns and a c×c identity `basis`; per column k, pivot = `firstBit` (lowest set row), clear that pivot row out of earlier pivot columns above it and out of all later columns, tracking identical ops on `basis`; finally column r of B = `basis[pivot_col[r]]` for pivot rows r, zero elsewhere. **No validation whatsoever and no no-value gate before computation** (compute always runs; only the push is gated, :5051).

>64 rows or columns: upstream has only `assert`s (bitvector.h:385,393; bitvector.cpp:150) — compiled out under NDEBUG. Probed on the oracle: `mod2_section(null(65,1))` and `mod2_section(null(1,65))` both return zero matrices **without error** (exact shapes depend on out-of-range bit behavior — UB). Recommendation: exclude >64 from fixtures; in Rust, mask row bits ≥64 on input (reproduces the observed silent-drop on this oracle build) and note the UB caveat in REMAINING_BUILTINS.md.

Probes (verbatim):

- `mod2_section(mat: [[1,0],[0,1]])` → `| 1, 0 | / | 0, 1 |`
- `mod2_section(mat: [[1,1],[0,1],[1,0]])` (2×3, full row rank) → `| 1, 0 | / | 1, 1 | / | 0, 0 |` (3×2; a right inverse)
- `mod2_section(mat: [[2,4],[6,8]])` (all even) → `| 0, 0 | / | 0, 0 |`
- `mod2_section(mat: [[0-1]])` → `| 1 |`
- `mod2_section(mat: [[1,0,1],[0,1,1]])` (3×2) → `| 1, 0, 0 | / | 0, 1, 0 |`
- `mod2_section(null(0,0))` → `The 0x0 matrix`; `mod2_section(null(2,3))` → 3×2 zero matrix

No rejected cases exist upstream — do not invent any.

### 3.2 `subspace_normal` `(mat->mat,mat,mat,[int])` — global.w:5062-5102, install :5212-5213

Semantics: reduced column-echelon normal form over GF(2) for a generator set that need not be independent. `dim` = rows, `n_gens` = columns; each column is reduced mod 2 into a `BitVector<64>` (negative odd → 1, probed). Loop (global.w:5112-5134): reduce each generator by the current basis (pivot = `firstBit`, lowest set row), tracking for every generator a `combination` (its own basis of expressions, init identity); if the reduced vector is nonzero, clear its new pivot out of all existing basis vectors (updating their combinations) and append. Output assembly (:5146-5174) reorders basis columns by **ascending pivot** via `permutations::standardization(pivot, dim)` (permutations.cpp:257-282: `pi[l]` = #{values < pivot[l]} + #{earlier equal values}), and emits relations for the excluded generators. Returns, as a 4-tuple (`wrap_tuple<4>` at single_value, :5100-5101):

1. `basis_m`: dim × rank — the normalized basis, columns in ascending pivot order
2. `combin_m`: n_gens × rank — expression of each basis vector in the original generators
3. `relations_m`: n_gens × (n_gens − rank) — for each dependent generator j, the combination proving it dependent (column order = generator order minus pivoters)
4. `pivot_r`: `[int]` of length rank — pivot row indices, ascending

Validation (BEFORE the no-value gate, in this order — probed `null(65,65)` → dim error; discarded-context `begin subspace_normal(null(65,1)); 7 end` → errors):

- `dim > 64`: `"Dimension too large: 65>64"` (no spaces around `>`, verbatim)
- `n_gens > 64`: `"Too many generators: 65>64"`

Probes (verbatim):

- `subspace_normal(mat: [[1,0],[0,1]])` destructured → B = `| 1, 0 | / | 0, 1 |`, C = same, R = `The 2x0 matrix`, p = `[0,1]`
- `subspace_normal(mat: [[1,1],[1,0],[0,1]])` → B = `| 1, 0 | / | 0, 1 |`, C = `| 0, 1 | / | 1, 1 | / | 0, 0 |`, R = `| 1 | / | 1 | / | 1 |`, p = `[0,1]`
- `subspace_normal(mat: [[1,1,2],[2,0,2]])` (dim 3, 2 gens, second ≡ 0 mod 2) → `( | 1 | / | 1 | / | 0 | , | 1 | / | 0 | , | 0 | / | 1 | ,[0])`
- `subspace_normal(mat: [[1,0],[1,0],[0,0]])` (duplicate + zero column) → B = `| 1 | / | 0 |`, C = `| 1 | / | 0 | / | 0 |`, R = `| 1, 0 | / | 1, 0 | / | 0, 1 |`, p = `[0]`
- `subspace_normal(mat: [[0,1],[0,1]])` → B = `| 0 | / | 1 |`, C = `| 1 | / | 0 |`, R = `| 1 | / | 1 |`, p = `[1]`
- `subspace_normal(mat: [[0-1, 0],[0, 0-3]])` → identity basis, `The 2x0 matrix` relations, `[0,1]` (negative-entry parity)
- `subspace_normal(mat: [[3,5],[7,9]])` (all odd) → B = `| 1 | / | 1 |`, C = `| 1 | / | 0 |`, R = `| 1 | / | 1 |`, p = `[0]`
- `subspace_normal(null(0,0))` → `(The 0x0 matrix,The 0x0 matrix,The 0x0 matrix,[])`
- `subspace_normal(null(3,0))` → `(The 3x0 matrix,The 0x0 matrix,The 0x0 matrix,[])`
- `subspace_normal(null(0,2))` → `(The 0x0 matrix,The 2x0 matrix, | 1, 0 | / | 0, 1 | ,[])`

## 4. Hidden `"transpose "` (global.w:5188) — reachability verdict

**Not** unreachable from the upstream parser: the commabarlist row-display `[a,b | c,d]` (separator is `|`, NOT `;` — parser.y:370-376 with commabarlist :402-410) lowers to `transpose (mat: [[a,b],[c,d]])` via `lookup_identifier("transpose ")`. Probed: `[1,2| 3,4]` → `| 1, 2 | / | 3, 4 |` (note `[1,2; 3,4]` instead sequences: `;` is statement sequencing inside the display → `[1,3,4]`). It is unreachable only as a name: `transpose(mat: [[1,2],[3,4]])` → `Undefined identifier 'transpose'` (probed).

Rust status: the `Bar` token is lexed (syntax.rs:1051) but no expression production consumes a commabarlist — `[1,2|3,4]` is a parse error today. So skipping the `"transpose "` registration is behavior-preserving for name resolution; the row-display syntax is a **parser-level gap** to record alongside the 2-D slice gap. If row displays are ever ported, build the transposed matrix directly in the parser/typer — do not register the hidden name.

## 5. `readline_completions` (global.w:4390, wrapper :3546-3557, `completions` buffer.w:1175-1190)

Correction to the batch-2 note ("no batch semantics"): it **is** callable and observable in batch mode. Probed:

- `readline_completions("pred")` → `["pred"]`
- `readline_completions("xyzzy")` → `[]`
- `#readline_completions("")` → `297` (every keyword/builtin/loaded-library identifier at startup)

The wrapper pops the string, returns early only under `no_value` (:3548-3549), else enumerates `main_hash_table` in **insertion order**, filtered to keywords-or-bound identifiers — so output depends on session history (user-defined identifiers join the list) and on exact identifier registration order. Keep it a **documented exclusion**, but for the honest reason: its output is session- and hash-order-dependent (a stable differential target would require replicating upstream's identifier insertion sequence byte-for-byte), and its purpose is readline completion for front-ends — not because it is batch-inert. No batch-3 dependency.

## 6. Reuse map (batch-3 pieces available)

- GF(2) work shares **nothing** with matreduc.rs (i32 PID reduction) — both builtins are hard-limited to 64 and are clean `u64`-column code. Recommended: a small `gf2` section inside matreduc.rs or a sibling module, op-for-op from bitvector.cpp:346-405 (`section`) and global.w:5112-5134 (`subspace_normal`), using `PidMatrix::from_matrix`/`to_matrix` (matreduc.rs:47,57) for the value boundary.
- `atlas-real-group` exports `ModTwoVector`/`ModTwoSubspace` (lib.rs:125, lowest-pivot convention, reduced-at-every-pivot invariant — the same invariant as subspace_normal's inner loop), and `real_form_seed::solve_mod_two` (:693) elects the same particular solution as `BinaryMap::section`; but atlas-core already depends on atlas-real-group, and both candidates would still need combination/relation tracking bolted on — direct ports are smaller and byte-safer. Do NOT route through them; cite them only as cross-checks.
- `permutations::standardization` (permutations.cpp:257-282) is 10 lines: count values `< v`, prefix-cumulate, `result[i] = count[a[i]]++`. Port inline.
- Plumbing: copy batch 3's patterns — `scalar_builtin(name, arg, result, 0, ScalarOp::X)`; multi-return via `Value::Tuple` (same 4-tuple shape as `ScalarOp::Echelon`); validation-before-gate via `runtime(msg, span)` raised before the no-value check (copy `invert`'s non-square gate); wrapping-i32 negate for the slicer's bit 7; matrix entry access via `linear_values::Matrix`.

## 7. Proposed fixtures

`tests/fixtures/eval/global_batch4.atlas` (accepted; conventions per batch 3 — `mat:` column literals, `0-1` not `-1`):

```
set M = mat: [[1,2],[3,4],[5,6]]
swiss_matrix_knife(0, M, 0, 2, 0, 3)
swiss_matrix_knife(64, M, 0, 2, 0, 3)
swiss_matrix_knife(128, M, 0, 2, 0, 3)
swiss_matrix_knife(129, M, 0, 2, 0, 3)
swiss_matrix_knife(1, M, 0, 2, 0, 3)
swiss_matrix_knife(8, M, 0, 2, 0, 3)
swiss_matrix_knife(2, M, 1, 2, 0, 3)
swiss_matrix_knife(4, M, 0, 1, 0, 3)
swiss_matrix_knife(16, M, 0, 2, 1, 3)
swiss_matrix_knife(32, M, 0, 2, 0, 1)
swiss_matrix_knife(192, M, 0, 2, 1, 3)
swiss_matrix_knife(256, M, 0, 2, 0, 3)
swiss_matrix_knife(0-1, M, 0, 2, 0, 3)
swiss_matrix_knife(0, M, 2, 0, 0, 1)
swiss_matrix_knife(64, M, 2, 0, 0, 1)
swiss_matrix_knife(0, null(0,0), 0, 0, 0, 0)
swiss_matrix_knife(255, null(2,3), 0, 0, 0, 0)
mod2_section(mat: [[1,0],[0,1]])
mod2_section(mat: [[1,1],[0,1],[1,0]])
mod2_section(mat: [[2,4],[6,8]])
mod2_section(mat: [[0-1]])
mod2_section(mat: [[1,0,1],[0,1,1]])
mod2_section(null(0,0))
mod2_section(null(2,3))
subspace_normal(mat: [[1,0],[0,1]])
subspace_normal(mat: [[1,1],[1,0],[0,1]])
subspace_normal(mat: [[1,1,2],[2,0,2]])
subspace_normal(mat: [[1,0],[1,0],[0,0]])
subspace_normal(mat: [[0,1],[0,1]])
subspace_normal(mat: [[0-1, 0],[0, 0-3]])
subspace_normal(mat: [[3,5],[7,9]])
subspace_normal(null(0,0))
subspace_normal(null(3,0))
subspace_normal(null(0,2))
```

`tests/fixtures/eval/global_batch4_rejected.atlas` (rejected; `for i:2 do X od` pins validation-before-gate, batch-3 convention):

```
swiss_matrix_knife(0, mat: [[1,2],[3,4]], 0, 3, 0, 2)
swiss_matrix_knife(0, mat: [[1,2],[3,4]], 5, 1, 0, 1)
swiss_matrix_knife(0, mat: [[1,2],[3,4]], 0, 1, 0, 5)
swiss_matrix_knife(0, mat: [[1,2],[3,4]], 0, 1, 5, 1)
swiss_matrix_knife(0, mat: [[1,2],[3,4]], 5, 9, 3, 7)
swiss_matrix_knife(0, mat: [[1,2],[3,4]], 5, 9, 0, 7)
swiss_matrix_knife(0, mat: [[1,2],[3,4]], 0, 9, 1, 8)
swiss_matrix_knife(0, mat: [[1,2],[3,4]], 0-1, 2, 0, 2)
swiss_matrix_knife(0, mat: [[1]], 0, 99999999999999999999999, 0, 1)
for i:2 do swiss_matrix_knife(0, mat: [[1,2],[3,4]], 0, 5, 0, 2) od
subspace_normal(null(65,1))
subspace_normal(null(1,65))
subspace_normal(null(65,65))
for i:2 do subspace_normal(null(65,1)) od
```

Deliberately excluded: `mod2_section` >64 inputs (UB upstream, no diagnostic — document in REMAINING_BUILTINS.md instead); the `i32::MIN`-negation slicer corner until the oracle capture confirms the wrapping regime on HPC.

## 8. Traps carried forward

- "actual limits are2, 2" — no space after `are`, space after the comma; subspace_normal's `65>64` has no spaces around `>`. Copy texts from this report or the HPC capture, never from prose.
- Bounds check uses raw bounds before from-end resolution; flags byte truncates mod 256 and never throws; bounds throw on negative/oversize via `ulong_val`.
- Slicer negate must be wrapping i32; upstream `(negate ? -1 : 1) * src(k,l)` overflows identically to C++.
- `subspace_normal` pivot reordering is NOT the loop order — output columns are sorted by ascending pivot via `standardization`; relations columns follow original generator order minus pivoters (`d = j - l`).
- Universal: all expected outputs from HPC oracle capture, never hand-written; zero-size matrices legal except the two >64 checks.
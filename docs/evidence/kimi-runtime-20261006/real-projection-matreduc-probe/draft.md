---
title: 实投影与整数矩阵约化（real_projection.rs / matreduc.rs）知识来源包草案
source: atlas-rust/real-projection-matreduc
ingestedAt: 2026-10-06T00:00:00Z
---

# 实投影与整数矩阵约化：知识来源包（草案，待逐条核对）

## 0. 范围、依据与标注约定

- 本包仅依据以下两个文件的完整字节整理：
  - `crates/atlas-real-group/src/real_projection.rs`
  - `crates/atlas-real-group/src/matreduc.rs`
- 标注约定：
  - 【已实现】= 直接可在字节中读出的代码事实（含注释/文档字符串中的声明，会注明「注释声明」）。
  - 【推断】= 阅读代码得出的推断，未经任何执行或数学验证。
- 本包不做任何数学验收、性能或正确性声明；所有「与上游一致」的说法均仅为代码注释声明，未经核对。

---

## 1. `real_projection.rs` 裸签名清单

```rust
// ---- 模块文档（注释声明）----
// 每个对合的 `(1-theta)X^*` 像基对 (lift_mat, M_real)，对应上游
// `InvolutionTable::record`（involutions.h:104-105）；满足
// `lift_mat * m_real == 1 - theta`（involutions.h:105）。
// 基并非由 theta 唯一确定：上游在 Cartan 轨道典范对合处用
// `matreduc::column_echelon`（matreduc.h:129）播种，再沿 cross-action BFS
// 运输（involutions.cpp:196-208、242-243）。

// ---- 依赖导入 ----
use crate::{LatticeInvolution, StructureError};

// ---- 结构体 ----
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RealProjection {
    pub(crate) lift_mat: Vec<Vec<i64>>,
    pub(crate) m_real: Vec<Vec<i64>>,
}

// ---- impl 块 ----
impl RealProjection {
    pub(crate) fn build(theta: &LatticeInvolution) -> Result<Self, StructureError>;
    pub(crate) fn transported(&self, reflection: &[Vec<i32>]) -> Result<Self, StructureError>;
    pub(crate) fn check_against(&self, theta: &LatticeInvolution) -> Result<(), StructureError>;
    pub(crate) fn image_rank(&self) -> usize;
    pub(crate) fn coordinates(&self, weight: &crate::Weight) -> Result<Vec<i64>, StructureError>;
    pub(crate) fn lift(&self, coordinates: &[i64]) -> Result<Vec<i64>, StructureError>;
}

// ---- 私有自由函数（非 pub，列出以便核对错误分支）----
fn identity_matrix(rank: usize) -> Result<Vec<Vec<i64>>, StructureError>;
fn gcd_sweep(
    a: &mut [Vec<i64>],
    col: &mut [Vec<i64>],
    row: usize,
    limit: usize,
) -> Result<i64, StructureError>;
fn apply_column_ops(
    matrix: &mut [Vec<i64>],
    ops: &[Vec<i64>],
    limit: usize,
) -> Result<(), StructureError>;
fn invert_integer_matrix(matrix: &[Vec<i64>]) -> Result<Vec<Vec<i64>>, StructureError>;

// ---- 测试模块 ----
#[cfg(test)]
mod tests {
    // use super::*; use crate::{BasedRootDatum, Coweight, Weight};
    #[test] fn signed_gcd_sweep_matches_original_column_operations();
    #[test] fn skew_torus_projection_matches_original_image_basis();
    #[test] fn skew_product_projection_matches_original_image_basis();
    #[test] fn projection_zero_and_full_image_boundaries();
}
```

---

## 2. `real_projection.rs`：职责与核心算法

### 2.1 职责【已实现】

- 为单个对合 `theta` 构造并持有 `(1-theta)X^*` 的像基对：列阶梯形像基 `lift_mat`（`n x r`）与坐标矩阵 `m_real`（`r x n`）。
- 提供构造（`build`）、沿单条 cross 边运输（`transported`）、自校验（`check_against`）、坐标正/反变换（`coordinates` / `lift`）与秩查询（`image_rank`）。
- 字段 `lift_mat`、`m_real` 均为 `pub(crate)`，测试中直接按字面量比较。

### 2.2 `build`：列阶梯约化 + 列操作矩阵求逆【已实现】

1. 由 `theta.weight_matrix()` 构造整数矩阵 `a = 1 - theta`：`diagonal = i64::from(row_index == column_index)`，逐项 `checked_sub(i64::from(entry))`，溢出报 `ArithmeticOverflow`。
2. `col = identity_matrix(rank)`（`i64` 单位阵）。
3. 行扫描自底向上（`for row in (0..rank).rev()`）：对每行调用 `gcd_sweep(&mut a, &mut col, row, limit)`；返回主元值 `pivot == 0` 时 `continue`（该行部分行已全零、无主元），否则 `limit -= 1`，主元落在列 `limit`（注释引用 matreduc.h:136-148）。
4. 零列擦除（`zero_columns = limit`，注释引用 matreduc.h:150-158）：逐次对每行 `a_row.remove(limit)`；对 `col` 收集 `cc[k] = col[k][limit]`，将 `[limit, m_columns)` 区间左移一格（`m_columns = rank - erased - 1`），再把 `cc[k]` 停放到 `col_row[m_columns]`——核列一列一列向右端轮转，「已停放在右侧的列不再被触碰」【注释声明】。结尾 `debug_assert_eq!(erased, zero_columns)`。
5. `image_rank = rank - zero_columns`；`col_inverse = invert_integer_matrix(&col)?`；`m_real = col_inverse[..image_rank].to_vec()`（注释：`M_real = col.inverse().block(0,0,image_rank,rank)`，involutions.cpp:203）。
6. 收尾调用 `projection.check_against(theta)?` 做分解自校验后返回 `Ok`。

### 2.3 `gcd_sweep`：带符号记录的局部行 gcd【已实现】

- 入口 `dest = limit.checked_sub(1)`，`limit == 0` 时报 `RepInvariantViolation { invariant: "echelon pivot column" }`（防御性分支；按 `build` 的循环结构 `limit >= 1`【推断】）。
- 复制 `a[row][..limit]` 为 `local_row`；扫描非零项得 `active` 列表、最小绝对值 `min` 与其列 `mindex`；`active` 为空返回 `Ok(0)`。
- 若 `local_row[mindex] < 0`：取负并记录 `ops[mindex][mindex] = -1`（`ops` 初为 `limit x limit` 单位阵）。注释声明：该记录符号是 E6 involution-187 分解成立的必要条件。
- 主循环（`active.len() > 1`）：以 `pivot = local_row[mindex]`（已保证为正）对其它活跃列做 `quotient = local_row[j].div_euclid(pivot)`（欧几里得除法，注释声明：负余数会选出负主元、反转典范像基定向）；同步更新 `ops[r][j] -= quotient * ops[r][current]`（全部 `checked_*`）与 `local_row[j] -= pivot * quotient`；余数非零者进入 `survivors`，更小余数更新 `min`/`mindex`。`survivors` 用 `try_reserve_exact(active.len())` 分配。
- 若 `mindex != dest`，对每行 `ops[r].swap(dest, mindex)`（列交换）。
- 最后 `apply_column_ops(a, &ops, limit)?` 与 `apply_column_ops(col, &ops, limit)?` 同时施加（注释：对应上游 `column_apply` / `column_echelon` 把 ops 同时作用于 M 与 col，matreduc.h:143-144）。返回 `Ok(min)`。

### 2.4 `apply_column_ops`【已实现】

- 计算 `M' = M * ops` 的前 `limit` 列（注释引用 matrix.h:496-510）：`fresh[r][c] = Σ_k ops[k][c] * matrix[r][k]`，跳过 `weight == 0`，全部 `checked_mul`/`checked_add`；写回前 `limit` 列。
- 注意：`fresh = vec![vec![0_i64; limit]; rows]` 为非受检分配（不走 `try_reserve_exact`）。

### 2.5 `invert_integer_matrix`：幺模矩阵整数求逆【已实现】

- 非方阵（任一行长度 ≠ `rank`）报 `InvalidIntegerMatrixShape`。
- 构造增广 `[matrix | I]`（`augmented`、`extended` 用 `try_reserve_exact`，扩展单位阵部分用 `i64::from(row_index == column_index)`）。
- 对每个 `pivot`：在 `(pivot..rank)` 中选该列非零且绝对值最小者（`min_by_key(|&r| augmented[r][pivot].abs())`），全零则报 `RepInvariantViolation { invariant: "non-singular column operations matrix" }`；交换到主元行后进入 `while changed` 欧几里得消元：其它行若 `|entry| >= |pivot|` 则以截断商 `quotient = augmented[r][pivot] / augmented[pivot][pivot]` 做行减（`quotient != 0` 时；`if augmented[pivot][pivot].abs() > 1` 且绝对值不足则交换两行）。注意：此处行减用普通 `-`/`*` 运算符（非 `checked_*`）【已实现，与文件其它部分的受检风格不对称】。
- 消元后逐主元检查对角线：`== -1` 则整行取负；`!= 1` 报 `RepInvariantViolation { invariant: "unimodular column operations matrix" }`。
- 取右半为逆矩阵，最后逐项验证 `matrix * inverse == I`（`checked_*`），不符报 `RepInvariantViolation { invariant: "integer matrix inverse" }`。

### 2.6 `transported`：跨一条 cross 边运输【已实现】

- 注释声明对应上游 `InvolutionTable::add_cross`（involutions.cpp:242-243）：`m_real := m_real * s`、`lift_mat := s * lift_mat`，`s` 为反射的权格矩阵；像基路径相关（与新做的阶梯约化差列号/列序），分解不变量 `(s*L)*(M*s) == s*(1-theta)*s == 1-theta'` 保持。
- 校验：`reflection` 必须方阵（每行长度 == `reflection.len()`），否则 `InvalidIntegerMatrixShape`。**不**校验 `reflection.len()` 与 `self` 的 `n`（`lift_mat` 行数）一致【已实现；不一致时行为见 §2.8】。
- `lift_mat'[row] = Σ_k reflection[row][k] * lift_mat[k]`（`weight == 0` 跳过；`checked_mul`/`checked_add`），结果行数 = `reflection.len()`、列数 = `self.image_rank()`。
- `m_real'[row][j] = Σ_k m_real[row][k] * reflection[k][j]`（`entry == 0` 跳过），结果行数 = `image_rank`、列数 = `reflection.len()`。
- **不**调用 `check_against`【已实现】。

### 2.7 `check_against` / `image_rank` / `coordinates` / `lift`【已实现】

- `check_against`：三重循环逐项验证 `(lift_mat * m_real)[i][j] == i64::from(i==j) - i64::from(entry)`，不符报 `RepInvariantViolation { invariant: "image basis factorization" }`；乘加全部 `checked_*`。
- `image_rank()` = `self.m_real.len()`。
- `coordinates(weight)`：返回 `m_real * v`（长度 `r`）；内层用 `row.iter().zip(weight.as_slice())`——**zip 截断**，`weight` 短于 `n` 时缺失坐标按零贡献、不报错【已实现】。
- `lift(coordinates)`：返回 `lift_mat * coordinates`（长度 `n = lift_mat.len()`）；按 `coordinates` 长度遍历并直接索引 `lift_mat[row][basis_index]`——`coordinates` 长于 `r` 会索引越界 panic，短于 `r` 相当于零填充【已实现】。

### 2.8 错误分支与预算汇总

| 位置 | 错误/行为 | 触发条件 |
|---|---|---|
| `build` / `identity_matrix` / `gcd_sweep`(`survivors`) / `invert_integer_matrix` / `transported` / `coordinates` | `StructureError::AllocationFailed { requested }` | `try_reserve_exact` 失败；`requested` 分别为 `rank`、`active.len()`、`lift_mat.len()`、`image_rank()`、`m_real.len()` 等 |
| 上述各函数乘加减处 | `StructureError::ArithmeticOverflow` | `checked_mul/add/sub` 失败 |
| `transported`、`invert_integer_matrix` | `StructureError::InvalidIntegerMatrixShape` | 反射矩阵非方阵 / 输入非方阵 |
| `gcd_sweep` | `RepInvariantViolation { "echelon pivot column" }` | `limit == 0`（防御性） |
| `check_against` | `RepInvariantViolation { "image basis factorization" }` | 分解逐条目不符 |
| `invert_integer_matrix` | `RepInvariantViolation { "non-singular column operations matrix" }` / `{ "unimodular column operations matrix" }` / `{ "integer matrix inverse" }` | 列无主元 / 对角线非 ±1 / 乘积校验失败 |
| `lift` | 索引越界 panic（无 Err） | `coordinates.len() > image_rank()` |
| `transported` | 索引越界 panic 或静默变形（无 Err） | `reflection.len() != n`（仅方阵性被校验） |
| `debug_assert` | `build` 末 `erased == zero_columns` | 仅 debug |

- 预算：无显式迭代上限或内存预算；所有循环界由 `rank`/`limit`/`active.len()` 隐式给出。`gcd_sweep` 的终止依赖欧几里得下降【推断】。分配受检性不对称：`try_reserve_exact` 仅覆盖部分外分配，`vec![...]` / `to_vec()` / `collect()` 不受检【已实现】。
- 顺序保证【已实现】：行扫描自底向上；主元依次落列 `limit-1`；零列擦除逐列右旋核列且不动已停放列。

### 2.9 测试锚点【已实现】

| 测试 | 关键断言（字面量） |
|---|---|
| `signed_gcd_sweep_matches_original_column_operations` | 对 `[[8,-12],[4,-6]]` 第 1 行、`limit=2` 调 `gcd_sweep`：`(pivot, image, columns) == (2, [[0,4],[0,2]], [[-3,2],[-2,1]])`；注释：仅查分解无法区分错误符号（基例 original3840186） |
| `skew_torus_projection_matches_original_image_basis` | 秩 2 空 datum；theta=`[[-7,12],[-4,7]]`；`lift_mat == [[4],[2]]`，`m_real == [[2,-3]]`；`coordinates([0,1]) == [-3]`；`lift([-3]) == [-12,-6]`；`check_against` 通过 |
| `skew_product_projection_matches_original_image_basis` | 秩 3 datum（一根一权一余权）；theta 为 `[1] ⊕ [[-7,12],[-4,7]]` 块；`lift_mat == [[0],[4],[2]]`，`m_real == [[0,2,-3]]` |
| `projection_zero_and_full_image_boundaries` | `rank ∈ {0, 2}`：恒等 theta ⇒ `image_rank()==0`、`lift_mat == vec![vec![]; rank]`、`lift(&[]) == vec![0; rank]`；theta=`-I` ⇒ `image_rank()==rank`、`m_real == I`、`lift_mat == 2I` |

---

## 3. `matreduc.rs` 裸签名清单

```rust
// ---- 模块文档（注释声明）----
// 精确整数矩阵约化，逐操作移植自上游 utilities/matreduc.cpp
// （diagonalise、gcd、has_solution、find_solution）。
// 欠定方程组 A*x==b 的选定解在下游可观测（tau/t 坐标奇偶性进入
// ext_block::same_sign），故复现 C++ 的幺模操作序列与行列式符号簿记；
// 算术为 wrapping i32，镜像上游 C++ int（含其溢出域，溢出在选定解中可观测）。

use crate::StructureError;

// ---- 结构体（字段私有）----
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct IntMatrix {
    rows: usize,
    columns: usize,
    data: Vec<i32>, // 行优先
}

impl IntMatrix {
    pub(crate) fn new(rows: usize, columns: usize) -> Self;
    pub(crate) fn identity(n: usize) -> Self;
    pub(crate) fn from_entries(rows: usize, columns: usize, data: Vec<i32>) -> Self; // assert_eq!(rows*columns, data.len())
    pub(crate) fn n_rows(&self) -> usize;
    pub(crate) fn n_columns(&self) -> usize;
    pub(crate) fn get(&self, i: usize, j: usize) -> i32;
    pub(crate) fn set(&mut self, i: usize, j: usize, value: i32);
    pub(crate) fn apply_to(&self, v: &mut [i32]);                 // assert_eq!(v.len(), self.columns)
    pub(crate) fn right_prod(&self, w: &[i32]) -> Vec<i32>;       // assert_eq!(w.len(), self.rows)
    fn transpose(&mut self);                                      // 私有；assert 方阵
    fn row_operation(&mut self, i: usize, k: usize, c: i32);      // 私有；c==0 跳过
    fn column_operation(&mut self, j: usize, k: usize, c: i32);   // 私有；c==0 跳过
    fn swap_rows(&mut self, i: usize, j: usize);                  // 私有
    fn swap_columns(&mut self, i: usize, j: usize);               // 私有
    fn row_multiply(&mut self, i: usize, c: i32);                 // 私有
    fn column_multiply(&mut self, j: usize, c: i32);              // 私有
}

// ---- 私有自由函数 ----
fn divide(a: i32, b: i32) -> i32;                       // debug_assert!(b > 0)；a<0 时 -1 - ((-1 - a) / b)
fn row_apply(a: &mut IntMatrix, ops: &IntMatrix, i: usize);    // assert ops 方阵、i+r <= a.rows
fn column_apply(a: &mut IntMatrix, ops: &IntMatrix, j: usize); // assert ops 方阵、j+r <= a.columns
fn gcd(mut row: Vec<i32>, flip: &mut bool, dest: usize) -> (i32, IntMatrix);
fn pull_back_columns(m: &IntMatrix, pi: &[usize]) -> IntMatrix;
fn permutation_is_negative(pi: &[usize]) -> bool;

// ---- pub(crate) 自由函数 ----
pub(crate) fn diagonalise(m: &IntMatrix) -> (IntMatrix, IntMatrix, Vec<i32>);
pub(crate) fn has_solution(a: &IntMatrix, b: &[i32]) -> bool;
pub(crate) fn find_solution(a: &IntMatrix, b: &[i32]) -> Option<Vec<i32>>;
pub(crate) fn in_left_image(beta: &[i32], a: &IntMatrix) -> bool;
pub(crate) fn in_right_image(a: &IntMatrix, b: &[i32]) -> bool;
pub(crate) fn inverse_upper_triangular(m: &IntMatrix) -> Result<IntMatrix, StructureError>;
pub(crate) fn exp_i(n: i32) -> i32;                     // debug_assert n 为偶数

// ---- 测试模块 ----
#[cfg(test)]
mod tests {
    fn matrix(rows: usize, columns: usize, entries: &[i32]) -> IntMatrix;  // 辅助
    fn mat_mul(a: &IntMatrix, b: &IntMatrix) -> IntMatrix;                 // 辅助（assert 维度）
    fn det(matrix: &IntMatrix) -> i64;                                     // 辅助（Leibniz 全排列）
    #[test] fn diagonalise_reconstructs();
    #[test] fn find_solution_solves();
    #[test] fn image_membership();
    #[test] fn oracle_reference_cases();
    #[test] fn inverse_upper_triangular_recovers_the_unit_inverse();
    #[test] fn exp_i_is_the_fourth_root_of_unity();
}
```

---

## 4. `matreduc.rs`：职责与核心算法

### 4.1 职责【已实现】

- 提供行优先存储的 `i32` 矩形矩阵 `IntMatrix` 及其初等行/列操作（全部 wrapping）。
- 移植上游：`diagonalise`（幺模 `row`、`col` 使 `row * m * col` 对角化并返回 `(row, col, diagonal)`）、`has_solution`、`find_solution`、`in_left_image`（ext_block.cpp `in_L_image`）、`in_right_image`（`in_R_image`）；另移植 `inverse_upper_triangular`（matrix.cpp:420-440）与 `exp_i`（arithmetic.h:49-51）。

### 4.2 `divide`【已实现】

- 正除数的下取整除法（注释声明即 `arithmetic::divide`，调用方保证 `b > 0`；`debug_assert!(b > 0)`）：`a >= 0` 时 `a / b`；否则 `-1 - ((-1 - a) / b)`。该写法避免对 `i32::MIN` 取负溢出【推断】。

### 4.3 `gcd`【已实现】

- 输入一份行的拷贝、`flip: &mut bool`（累积记录操作的行列式符号）与目标列 `dest`；返回 `(正 gcd, 记录列操作的 IntMatrix)`；施加 `ops` 后 gcd 位于 `dest`。
- 流程：扫描非零项得 `active_entries`、`min`（`entry.wrapping_abs()`）、`mindex`；全零返回 `(0, identity)`。`row[mindex] < 0` 则取负、`*flip = !*flip`、`col.set(mindex, mindex, -1)`。主循环以当前主元 `d`（正）对其余活跃项做 `q = divide(row[j], d)`、`col.column_operation(j, cur_col, -q)`、`row[j] = row[j].wrapping_sub(d.wrapping_mul(q))`；余数为零者从 `active_entries` 原地删除，更小余数更新 `min`/`mindex`。循环内有 `debug_assert!(active_entries.len() == 1 || mindex != cur_col)`。末尾若 `mindex != dest`：`col.swap_columns(dest, mindex)` 并翻转 `flip`。返回 `(min, col)`。

### 4.4 `diagonalise`【已实现】

- 空矩阵（`n_rows == 0 || n_columns == 0`）直接返回 `(identity(n_rows), identity(n_columns), vec![])`。
- 簿记变量：`row_minus`、`col_minus`（分别表示 `det(row) == -1`、`det(col) == -1`）；`pivot_columns: Vec<bool>`。
- 对每列 `l`（`k` 为当前主元行）：
  1. 取部分列 `(k..n_rows)` 调 `gcd(partial, &mut flip, 0)`；`d == 0` 则 `continue`（**不**递增 `k`）。
  2. `pivot_columns[l] = true`；**`row_minus = flip`（覆盖赋值，非累积）**；`ops.transpose()` 后对 `m` 与 `row` 施加 `row_apply(..., k)`；`debug_assert_eq!(m.get(k, l), d)`。
  3. 内层交替循环：行 gcd（`col_minus ^= flip`，对 `m`、`col` 施加 `column_apply(..., l)`；`d == old_d` 则 `break`）与列 gcd（`row_minus ^= flip`，`row_apply`；`d >= old_d` 则 `break`）。每步后 `debug_assert_eq!(m.get(k, l), d)`。
  4. 循环退出后 **`row_minus ^= flip`（使用循环内最后一次 `flip`）**；`diagonal.push(d)`（注释：记录最终剩下的正 gcd）；`k += 1`。
- 收尾：若前 `n_piv` 列中存在非主元列（`below < n_piv`，即主元列未左对齐），构造 `pi`（主元列按序在前、非主元列按序在后，稳定），`col = pull_back_columns(&col, &pi)` 且 `col_minus ^= permutation_is_negative(&pi)`（逆序对奇偶）。
- 符号归一：`diagonal` 非空且 `row_minus != col_minus` 时 `diagonal[0] = -diagonal[0]`；`row_minus` 时 `row.row_multiply(0, -1)`（注释：ensure determinant of |row| is 1）；`col_minus` 时 `col.column_multiply(0, -1)`。
- 文档字符串声明：对角项除首项可能为负外均为正。

### 4.5 求解与像判定【已实现】

- `has_solution(a, b)`：`diagonalise(a)` 后 `row.apply_to(&mut b)`；**逆序**遍历 `i ∈ (0..b.len()).rev()`：`i < diagonal.len()` 时要求 `b[i].wrapping_rem(diagonal[i]) == 0`，否则要求 `b[i] == 0`；任一不满足返回 `false`。
- `find_solution(a, b)`：同样左乘 `row`；对每个对角项要求整除（否则 `None`）并 `wrapping_div`；`b[diagonal.len()..]` 有非零则 `None`；`b.resize(col.n_rows(), 0)` 后 `col.apply_to(&mut b)` 返回 `Some(b)`。注释声明：上游在无解时抛异常，此处返回 `None`，且「调用方应先跑 `has_solution`」。
- `in_left_image(beta, a)` / `in_right_image(a, b)`：分别用左因子 `left.apply_to` / 右因子 `right.right_prod` 变换后逐项做同样的整除/归零判定。

### 4.6 `inverse_upper_triangular` 与 `exp_i`【已实现】

- `inverse_upper_triangular`：非方阵报 `RepInvariantViolation { invariant: "invert triangular: matrix is not square" }`；任一对角项 `!= 1` 报 `RepInvariantViolation { invariant: "invert triangular: not unitriangular" }`；按列 `j`、行 `i` 自底向上回代（`k ∈ ((i+1)..=j).rev()`），全部 wrapping 算术。注释声明：在上游抛异常处报错。
- `exp_i(n)`：`debug_assert_eq!(n % 2, 0, "exp_i: odd exponent")`（注释声明：偶数前提是上游 assert）；`n % 4 == 0` 返回 `1`，否则 `-1`（负数偶数经测试：`-2 → -1`）。release 下奇数输入走 `-1` 分支【已实现，无防护】。

### 4.7 错误分支、panic 面与预算汇总

| 类别 | 位置与条件 |
|---|---|
| `Err(StructureError::RepInvariantViolation { .. })` | 仅 `inverse_upper_triangular` 两处（非方阵 / 非单位三角） |
| `Option::None` | `find_solution`：任一对角行不整除，或 `b[diagonal.len()..]` 有非零 |
| `assert_eq!`/`assert!` panic | `from_entries` 形状不符；`apply_to`（`v.len() != columns`）；`right_prod`（`w.len() != rows`）；`transpose`（非方阵）；`row_apply`/`column_apply`（ops 非方阵或越界）。由此 `has_solution`/`find_solution`/`in_*_image` 对长度不匹配的 `b`/`beta` 是 panic 而非 Err【已实现】 |
| `debug_assert!` | `divide`（`b > 0`）；`exp_i`（偶数）；`gcd` 循环不变量；`diagonalise` 三处 `m.get(k, l) == d` |
| wrapping 域 | 全文件 `wrapping_add/sub/mul/div/rem/neg/abs`；`i32::MIN` 相关的 `wrapping_abs`/`wrapping_div` 边界无直接测试【已实现 + 推断】 |
| 预算 | 无显式迭代上限；`diagonalise` 内层循环以「无改进」（`d == old_d` / `d >= old_d`）退出，下降性由 gcd 语义保证【推断】；`IntMatrix::new` 的 `rows * columns` 未做溢出检查【已实现】 |
| 顺序保证 | `has_solution` 逆序检查；`pull_back_columns` 稳定序（主元列相对序保持）；回代按列序、列内自底向上【已实现】 |

### 4.8 测试锚点【已实现】

| 测试 | 关键断言 |
|---|---|
| `diagonalise_reconstructs` | 11 个用例（含 `[[0]]`、`[[-7]]`、秩亏、矩形 3x2/2x3、含负数 3x3）；断言 `\|det(row)\| == 1`、`det(col) == 1`、`row*case*col` 的前 `diagonal.len()` 个对角项等于 `diagonal`、其余为零、`i > 0` 的对角项为正。注释声明：上游仅强制 `det(col) == 1`，`det(row)` 可为 -1（举例 `[[0,2],[2,0]]`） |
| `find_solution_solves` | `[[2,0],[0,4]]`：`[6,4]` 可解得 `[3,1]`、`[6,3]` 无解；秩亏 `[[1,2],[2,4]]`：`[3,6]` 可解、`[3,5]` 无解；矩形 `[[1,2,3],[4,5,6]]` 解长度 3 且逐行还原 |
| `image_membership` | 1 维 `[[2]]`：`in_left_image`/`in_right_image` 对 `[4]` 真、`[3]` 假 |
| `oracle_reference_cases`（注释声明：取自 C++ oracle，bit-exact） | ① `[[0,5],[0,0]]`：`row == I`、`col == [[0,1],[-1,0]]`、`diagonal == [-5]`；`has_solution` 对 `[-2,-6]`/`[-3,-4]`/`[1,4]` 假、对 `[10,0]` 真；`find_solution([10,0]) == Some([0,2])`。② `[[-4]]`：`diagonal == [-4]`；`[6]`/`[5]`/`[-5]` 无解、`[12]` 可解得 `[-3]`。③ 6x6 秩亏（主元列未左对齐）：完整逐字断言 `row`、`col`、`diagonal == [1,1,1,1,3]`；`b = [-13,12,9,6,-15,-25]` 的解为 `[6,14,5,-37,-421,345]`；另三个 `b` 无解 |
| `inverse_upper_triangular_recovers_the_unit_inverse` | `[[1,2,3],[0,1,4],[0,0,1]] → [[1,-2,5],[0,1,-4],[0,0,1]]`；`[[1,-7],[0,1]] → [[1,7],[0,1]]`；非方阵与对角非 1 输入均 `is_err()` |
| `exp_i_is_the_fourth_root_of_unity` | `0→1`、`2→-1`、`4→1`、`-2→-1`、`6→-1` |

---

## 5. 两文件的接口关系

### 5.1 依赖事实【已实现】

- 由 `use` 语句可见：`real_projection.rs` 仅导入 `crate::{LatticeInvolution, StructureError}`；`matreduc.rs` 仅导入 `crate::StructureError`。**两文件之间无任何直接调用或类型依赖**，唯一共享的是错误类型 `crate::StructureError`。
- 二者同 crate（路径均为 `crates/atlas-real-group/src/`），可见性均为 `pub(crate)`。

### 5.2 同源而不同分支的移植【已实现（注释声明）+ 推断】

- 两文件都声明移植自上游 `utilities/matreduc`  lineage，但入口不同：
  - `real_projection.rs` 复刻 `matreduc::column_echelon`（matreduc.h:129-161）及其 `gcd`（matreduc.h:70-122），服务于 `InvolutionTable::record` 的 `(1-theta)` 像基播种与 cross 运输。
  - `matreduc.rs` 复刻 `matreduc.cpp` 的 `diagonalise`/`gcd`/`has_solution`/`find_solution`，外加 `matrix::inverse_upper_triangular` 与 `arithmetic::exp_i`，服务于 `A*x==b` 求解与 `ext_block` 的像判定。
- 两处的 gcd 扫描共享同一骨架【已实现，逐点可读】：非零项扫描、最小绝对值主元选举、负主元取正并记录符号、正主元下取整除法消元、末尾列交换到 `dest`。
- 关键分歧【已实现】：
  | 维度 | `real_projection.rs::gcd_sweep` | `matreduc.rs::gcd` |
  |---|---|---|
  | 整数类型与溢出 | `i64`，全 `checked_*`（溢出 → `ArithmeticOverflow`） | `i32`，全 `wrapping_*`（镜像 C++ `int` 溢出域） |
  | 除法 | `div_euclid` | 自定义 `divide`（下取整，公式 `-1 - ((-1-a)/b)`） |
  | 符号簿记 | `ops[mindex][mindex] = -1` 记入 ops 矩阵 | `flip: &mut bool` 累积 + `col.set(mindex, mindex, -1)` |
  | ops 应用时机 | 就地 `apply_column_ops` 同时作用于 `a` 与 `col` | 返回 ops，由调用方（`diagonalise`）经 `transpose` + `row_apply`/`column_apply` 施加 |
  | 行列式簿记 | 无（`col` 隐式携带） | `row_minus`/`col_minus` 显式簿记，含首对角项取负与 det=1 归一 |
- 模块文档给出的各自动机【注释声明】：`matreduc.rs` 需要复现溢出域因为选定解在 `ext_block::same_sign` 可观测；`real_projection.rs` 需要逐操作复刻因为 `lambda-rho` 代表元与 `y_lift` 符号依赖精确像基。**`real_projection` 为何不复用 `matreduc.rs` 的 `gcd`/`IntMatrix`，字节中无说明**【未覆盖，见 §7】。

### 5.3 错误语义差异【已实现】

- `real_projection.rs`：丰富 `Err` 分支（`AllocationFailed` / `ArithmeticOverflow` / `InvalidIntegerMatrixShape` / 四种 `RepInvariantViolation`），少量 panic 面（索引越界）。
- `matreduc.rs`：`Err` 仅出现在 `inverse_upper_triangular`；无解用 `Option::None`；维度契约主要靠 `assert_*` panic 与 `debug_assert`。

---

## 6. 限制与未覆盖面

### 6.1 `real_projection.rs`

- `transported` 在本文件测试中**完全没有被调用**；其维度契约（`reflection.len() == n`）未被校验，错误路径（`InvalidIntegerMatrixShape`、溢出、分配失败）均无测试【已实现】。
- `check_against` 只经由 `build` 间接测试；`coordinates`/`lift` 仅在秩 2 斜环面用例中各出现一次。
- `AllocationFailed`、`ArithmeticOverflow`、`RepInvariantViolation` 各分支无直接测试。
- 注释提到的 E6 involution-187 符号敏感场景在本文件无对应测试。
- 边界秩仅测 `{0, 2}`；`limit == 0` 防御分支按循环结构不可达【推断】、亦无测试。
- 分配受检不对称（`try_reserve_exact` vs `vec!`/`to_vec`/`collect`）【已实现】；`invert_integer_matrix` 行消元用非受检 `-`/`*`，与文件其余部分的 `checked_*` 风格不一致【已实现】。

### 6.2 `matreduc.rs`

- wrapping 溢出域（模块文档声明可观测）无任何直接测试【已实现】。
- `in_left_image`/`in_right_image` 仅测 1 维偶数例子；矩形/秩亏场景未覆盖这两个函数。
- `divide` 对负被除数的下取整路径仅经由含负数矩阵的 oracle 用例间接覆盖【推断】。
- `exp_i` 奇数输入在 release 下的行为（落 `-1` 分支）无测试、仅 `debug_assert` 防护【已实现】。
- 空形状（`0 x n`、`n x 0`）的 `diagonalise` 提前返回路径无测试；`from_entries`/`apply_to` 的 panic 路径无测试。
- `has_solution` 中负对角项的 `wrapping_rem` 语义仅由 case 2（`diagonal == [-4]`）间接触及。

### 6.3 跨文件

- 无任何测试锁定两文件 gcd 骨架在相同输入上的一致性；双份实现是否刻意长期并存，字节中无说明【未覆盖】。

---

## 7. 待维护者逐条核对清单

1. 【核对·上游一致性】`diagonalise` 中 `row_minus = flip;`（每个新主元列**覆盖**赋值）与循环退出后再次 `row_minus ^= flip` 的组合：在第一个 `break`（行 gcd 后）退出时，该 `flip` 已计入 `col_minus`，退出后又计入 `row_minus`；在第二个 `break`（列 gcd 后）退出时，该 `flip` 在循环内已 `^=` 进 `row_minus`，退出后再次 `^=`（净效果抵消）。以上为代码事实+阅读推断，需对照上游 `matreduc.cpp` 原文确认逐行一致。oracle 6x6 用例（`col`/`row` bit-exact）间接覆盖此簿记。
2. 【核对·注释一致性】`diagonalise` 末尾 `if row_minus { row.row_multiply(0, -1) }` 注释称「ensure determinant of |row| is 1」，而测试注释称「上游只强制 det(col)==1；det(row) 可为 -1（如 `[[0,2],[2,0]]`）」。由于 `row_minus` 被覆盖赋值，`row` 行列式确实可能以 -1 输出【推断，与 `[[0,2],[2,0]]` 手工追踪一致】；两条注释的措辞建议统一。
3. 【核对·设计意图】`real_projection.rs` 不复用 `matreduc.rs` 的 `gcd` 是否刻意（`i64 checked` vs `i32 wrapping` 两种溢出制度的隔离）；若刻意，建议在模块文档中互引。
4. 【核对·契约】`transported` 是否应校验 `reflection.len() == lift_mat.len()`；`lift` 是否应校验 `coordinates.len() == image_rank()`；`coordinates` 对短 `weight` 的 zip 截断是否依赖 `Weight` 类型的构造期不变量（本包未见 `Weight` 定义，无法确认）。
5. 【核对·一致性】`invert_integer_matrix` 行消元的非受检算术是否有意为之。
6. 【核对·测试缺口】是否补充：`transported` 的 oracle 用例、matreduc 溢出域用例、`in_*_image` 的矩形用例、E6 involution-187 回归用例。
7. 【核对·文档声明】两文件中所有上游指针（involutions.h:104-105/211、involutions.cpp:196-208/242-243/346-356、matreduc.h:70-122/129-161/136-148/150-158/143-144、matrix.h:496-510、matrix.cpp:420-440、arithmetic.h:49-51、ext_block.cpp `in_L_image`/`in_R_image`、`ext_block::same_sign`）均为代码注释声明，本包未对其真实性做任何验证，收录前请抽样核对。

— 草案完 —
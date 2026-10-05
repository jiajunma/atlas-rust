```markdown
---
title: Atlas Rust：对偶分量群平凡性判定（topology.rs）与实形式李代数命名（form_name.rs）
source: atlas-rust/topology-form-name
ingestedAt: 2026-10-06T00:00:00Z
---

# 知识来源包草案：topology.rs 与 form_name.rs

> 核对说明：本草案仅依据所给两个源文件字节写成。文中以 **【已实现】** 标记可直接从字节读出的行为，以 **【推断】** 标记阅读推断（未验证、不作正确性/性能声明）。所有上游文件行号引用（如 topology.cpp:165-192）均来自代码注释，本草案未核对其上游原文。

- 文件一：`crates/atlas-real-group/src/topology.rs`
- 文件二：`crates/atlas-real-group/src/form_name.rs`

---

## 1. 文件一 `topology.rs`：对偶分量群平凡性判定

### 1.1 模块级文档【已实现（注释内容）】

- 主题：实形式的连通性——其最分裂 Cartan 的对偶分量群（注释引用 `topology::dual_component_group_basis`, topology.cpp:165-192）。
- 设定：格基 `B` 的列 = 单余根 + 余权格根部（radical）的一组基；`i_sw = B^t theta B^{-t}` 为 Cartan 对合转运到单连通覆盖权格上的矩阵（注释称对应上游 `theta.transposed().on_basis(basis).transposed()`）。
- 对偶分量群 = 限制映射 `dualPi0(theta) -> dualPi0(i_sw)` 的核，该映射由 `B_z^t mod 2` 诱导，其中 `B_z` 是把 `B` 的 radical 列清零后的矩阵；实形式连通当且仅当核消失（诱导映射单射）。

### 1.2 裸签名清单（含私有项与无注释项）

```rust
// 导入
use malachite::base::num::arithmetic::traits::Floor;
use malachite::base::num::basic::traits::Zero;
use malachite::Rational;
use crate::integer_lattice::{
    reduce_basis_mod_two, saturated_kernel, IntegerLatticeBudget, IntegerMatrix, IntegralBasis,
};
use crate::mod_two::{ModTwoAmbientMap, ModTwoSubquotient, ModTwoSubspace, ModTwoVector};
use crate::real_form_seed::invert_rational;
use crate::{BasedRootDatum, StructureError};

// 私有函数
fn dual_pi0(theta: &[Vec<i32>], budget: &IntegerLatticeBudget)
    -> Result<ModTwoSubquotient, StructureError>;

// 私有结构体（字段含注释）
struct CorootRestriction {
    /// `B` with the radical columns zeroed; columns index the map output.
    zeroed: Vec<Vec<i32>>,
}

// 可见 impl 块
impl ModTwoAmbientMap for CorootRestriction {
    fn source_dimension(&self) -> usize;
    fn target_dimension(&self) -> usize;
    fn apply(&self, source: &ModTwoVector) -> Result<ModTwoVector, StructureError>;
}

// 私有函数（无文档注释）
fn rational_matrix(matrix: &[Vec<i32>]) -> Vec<Vec<Rational>>;
fn rational_product(
    left: &[Vec<Rational>],
    right: &[Vec<Rational>],
) -> Result<Vec<Vec<Rational>>, StructureError>;
fn integral_entry(value: &Rational) -> Result<i32, StructureError>;

// 公有接口
pub(crate) fn dual_component_group_trivial(
    theta: &[Vec<i32>],
    datum: &BasedRootDatum,
    budget: &IntegerLatticeBudget,
) -> Result<bool, StructureError>;

pub fn dual_component_group_rank(
    theta: &[Vec<i32>],
    datum: &BasedRootDatum,
    budget: &IntegerLatticeBudget,
) -> Result<usize, StructureError>;

// 测试模块
#[cfg(test)] mod tests { /* 见 §1.6 */ }
```

### 1.3 辅助项逐项说明

#### `dual_pi0`（私有）【已实现】
注释：上游 `tori::dualPi0` 子商（tori.cpp:162-176, `eigen_lattice(theta, 1)`）。步骤：
1. `rank = theta.len()`；对每行检查 `row.len() != rank` → `Err(StructureError::InvalidIntegerMatrixShape)`。
2. 分子：把 `theta + I`（mod 2）的各行（按 `value + diagonal` 的奇偶收集 1 位）插入 `ModTwoSubspace::new(rank)`，取 `right_kernel()`。
3. 分母：构造 `theta - I`（i32），经 `IntegerMatrix::from_i32_rows(..., budget)?` 与 `saturated_kernel(..., budget)?` 得饱和 `+1` 特征格，再 `reduce_basis_mod_two`。
4. 返回 `ModTwoSubquotient::new(numerator, denominator)`。

#### `CorootRestriction`（私有）+ `impl ModTwoAmbientMap`【已实现】
- 语义：环境 mod-2 空间上的限制映射 `x -> B_z^t x mod 2`；`zeroed` 的列索引映射输出。
- `source_dimension()` 与 `target_dimension()` 均返回 `zeroed.len()`（方阵）。
- `apply`：若 `source.dimension() != zeroed.len()` → `Err(StructureError::RankMismatch { expected, actual })`；否则对每个输出列做奇偶累加（`zeroed[row][column] % 2 != 0 && source.bit(row) == Some(true)` 时翻转 `parity`），以 `ModTwoVector::from_ones(rank, ones)` 返回。

#### `rational_matrix` / `rational_product` / `integral_entry`（私有）【已实现】
- `rational_matrix`：i32 矩阵逐项转 `Rational`；不失败。
- `rational_product`：以 `left.len()` 为阶的方阵三重循环乘法；返回类型为 `Result`。**【推断】** 函数体实际永远返回 `Ok`，无显式错误分支；`rank` 取自 `left.len()`，索引 `left[row][middle]`、`right[middle][column]` 依赖调用方传入方阵，本文件内调用点均构造方阵。
- `integral_entry`：先 `floor()`；若 `*value != floored` → `Err(StructureError::LayoutInvariantViolation { invariant: "simply-connected involution integrality" })`；再 `i32::try_from(&floored)`，失败映射为 `Err(StructureError::ArithmeticOverflow)`。

### 1.4 `dual_component_group_trivial`（pub(crate)）【已实现】

注释：判定 Cartan 对合的对偶分量群是否平凡（realredgp.cpp:73-75 的 `IsConnected` 状态位）。`theta` 为最分裂 Cartan 的 Cartan 对合的权格矩阵；`budget` 限制整数格约化。

流程（与 `dual_component_group_rank` 完全同构，仅末行不同）：
1. 形状检查：`theta.len() != rank || 任一行 row.len() != rank` → `InvalidIntegerMatrixShape`（`rank = datum.lattice_rank()`；另有 `semisimple_rank = datum.semisimple_rank()`）。
2. 余权格根部：`root_rows = datum.simple_roots()`；若 `root_rows.is_empty()`（注释：无根 datum，T_k 情形）→ `IntegralBasis::full_space(rank)`；否则 `saturated_kernel(IntegerMatrix::from_i32_rows(&root_rows, budget)?, budget)?`。
   - 校验：`radical.rank() != rank - semisimple_rank` → `LayoutInvariantViolation { invariant: "radical rank" }`。
3. 组基 `B`（rank×rank，列序保证：先 `datum.simple_coroots()` 各列，再从列下标 `semisimple_rank` 起填 `radical.columns()`；radical 分量经 `i32::try_from`，失败 → `ArithmeticOverflow`）。
4. 有理计算 `i_sw = B^t · theta · B^{-t}`：转置 `basis` → `rational_matrix` → `invert_rational(&transposed_rational)?`（来自 `real_form_seed`）→ 再转置得逆；两次 `rational_product`；逐项 `integral_entry` 做整性检查（注释：上游 `on_basis` 整除精确）。
5. `source = dual_pi0(theta, budget)?`；`target = dual_pi0(&transported, budget)?`。
6. 构造 `zeroed = basis` 并将每行下标 `semisimple_rank..rank` 的列清零（代码为 `iter_mut().take(rank).skip(semisimple_rank)`）。
7. `source.validate_induced_map_to(&target, &map)?`（诱导映射须可下降）。
8. 对 `source.basis_representatives()` 中每个代表元：`map.apply(...)?` → `target.to_coordinates(...)?` → 插入 `image: ModTwoSubspace`。
9. 返回 `Ok(image.rank() == source.dimension())`（单射 ⟺ 平凡）。

### 1.5 `dual_component_group_rank`（pub）【已实现】

注释：对偶分量群的秩 = 诱导限制映射核的维数，即 `dualComponentReps` 基的大小（topology.cpp:165-192 `m2.kernel()`）；`dual_component_group_trivial` 恰为 `rank == 0`。

- 函数体与 §1.4 逐步相同；唯一差异：末行返回 `Ok(source.dimension() - image.rank())`。
- **【推断】** 两函数为整段复制管线，仅收尾表达式不同，存在同步维护风险。

### 1.6 错误分支汇总（本文件字节内可见）【已实现】

| 位置 | 条件 | 错误 |
|---|---|---|
| `dual_pi0` | 行长 ≠ `theta.len()` | `StructureError::InvalidIntegerMatrixShape` |
| 两个公有函数 | `theta` 非 rank×rank | `StructureError::InvalidIntegerMatrixShape` |
| 两个公有函数 | `radical.rank() != rank - semisimple_rank` | `LayoutInvariantViolation { "radical rank" }` |
| 两个公有函数 | radical 列分量 `i32::try_from` 失败 | `ArithmeticOverflow` |
| `integral_entry` | 有理值非整数 | `LayoutInvariantViolation { "simply-connected involution integrality" }` |
| `integral_entry` | 整数转 i32 失败 | `ArithmeticOverflow` |
| `CorootRestriction::apply` | 维数不符 | `RankMismatch { expected, actual }` |
| 各 `?` 传播 | `ModTwoSubspace::new` / `ModTwoVector::from_ones` / `insert` / `right_kernel` / `from_i32_rows` / `saturated_kernel` / `reduce_basis_mod_two` / `ModTwoSubquotient::new` / `invert_rational` / `validate_induced_map_to` / `to_coordinates` | **【推断】** 具体错误变体不在本文件字节内，无法枚举 |

- panic/断言：非测试代码无显式 `panic!`/`assert!`。**【推断】** 数组下标（如 `basis[row][column]`、`left[row][middle]`）依赖上游不变量，形状违约会 panic，但调用点均按 rank 构造。`budget: &IntegerLatticeBudget` 传入 `saturated_kernel` 与 `IntegerMatrix::from_i32_rows`；预算语义（超限行为）不在本文件字节内。

### 1.7 测试锚点（`mod tests`）【已实现】

测试预算辅助：`IntegerLatticeBudget::new(64, 100_000, 100_000, 128)`。

| 测试名 | 构造 | 断言 |
|---|---|---|
| `split_simply_connected_a1_is_connected` | `from_simple_data(1, [[2]], [Weight([2])], [Coweight([1])])`，`theta = [[-1]]` | `dual_component_group_trivial(...) == true`（注释：B=[[1]]，限制为 mod 2 恒等，单射） |
| `split_adjoint_a1_is_disconnected` | `BasedRootDatum::standard([[2]])`，`theta = [[-1]]` | `== false`（注释：B=[[2]] mod 2 为零，PGL(2,R) 两个分量） |
| `compact_simply_connected_a1_is_connected` | 同 sc A1 datum，`theta = [[1]]` | `== true`（注释：分子 ker_F2(theta+1) 为零，子商平凡） |
| `compact_adjoint_b2_is_connected` | `standard([[2,-1],[-2,2]])`，`theta = I₂` | `== true` |
| `split_gl2_is_disconnected` | `from_simple_data(2, [[2]], [Weight([2,0])], [Coweight([1,0])])`，`theta = -I₂` | `== false`（注释：radical/中心环面坐标被清零，秩一对偶分量群） |

**未覆盖**【已实现（从字节可见）】：本文件测试只调 `dual_component_group_trivial`；`dual_component_group_rank` 无直接测试；无根 datum（`root_rows.is_empty()` 分支）无测试。

---

## 2. 文件二 `form_name.rs`：实形式李代数命名表

### 2.1 模块级文档【已实现（注释内容）】

- 上游 `output::printType` 及其辅助 `print_real_form_name`、`printComplexType`、`split` 的移植（io/output.cpp:556-782）。
- 输入 grading 为 `specialGrading` 划分重载：datum 单根上的 bitset，1 = 非紧虚根。
- 按拉回 `pulled[k] = grading[perm[k]]`（permutations.cpp:66-76）适配到 Bourbaki 序，并像上游 `gr >>= rank` 循环一样逐单因子消费。

### 2.2 裸签名清单（含私有项与无注释项）

```rust
use crate::error::StructureError;
use crate::layout::InnerClassLayout;

// 私有函数
fn split(name: &str, n: usize, m: usize) -> String;
fn complex_name(letter: char, rank: usize) -> Result<String, StructureError>;
fn factor_name(bits: u32, letter: char, rank: usize, ic: char) -> Result<String, StructureError>;

// 公有接口
pub fn form_type_name(layout: &InnerClassLayout, grading: u128) -> Result<String, StructureError>;

// 测试模块
#[cfg(test)] mod tests { /* 见 §2.6 */ }
```

### 2.3 `split` 与 `complex_name`【已实现】

- `split(name, n, m)`：`m == 0` → `"{name}({n})"`；否则 `minority = if 2*m > n { n - m } else { m }`，输出 `"{name}({n-minority},{minority})"`（条目弱递减）。
- `complex_name(letter, rank)`（用于 `'C'` inner-class 条目，注释：覆盖两个同构因子，传入第一个）：

| letter | 输出 |
|---|---|
| `'A'` | `sl(rank+1,C)` |
| `'B'` | `so(2*rank+1,C)` |
| `'C'` | `sp(2*rank,C)` |
| `'D'` | `so(2*rank,C)` |
| `'E'` | `e{rank}(C)` |
| `'F'` | `f4(C)` |
| `'G'` | `g2(C)` |
| `'T'` | `gl(1,C)` |
| 其他 | `Err(LayoutInvariantViolation { invariant: "complex factor letter" })` |

### 2.4 `factor_name` 命名规则表【已实现】

公共量：`trivial = (bits == 0)`；`m = if trivial { 0 } else { bits.trailing_zeros() as usize + 1 }`（注释：上游 `m = gr.firstBit() + 1`，平凡 grading 时为 0）。文档注释明确：折叠小写字母 `'f'`/`'g'` 仅出现在某些对偶实形式的对偶侧命名中，本移植只覆盖 crate 布局可产生的非对偶字母。

| letter | 条件 | 输出 |
|---|---|---|
| `'A'` | `rank == 1` 且 trivial | `su(2)` |
| `'A'` | `rank == 1` 且非 trivial | `sl(2,R)` |
| `'A'` | `rank > 1`，`ic == 'c'` | `split("su", rank+1, m)` |
| `'A'` | `rank > 1`，`ic != 'c'`，`rank` 为奇 且 trivial | `sl((rank+1)/2, H)`（注释：unequal-rank A，上游两条件缺一不可） |
| `'A'` | 其余 | `sl(rank+1, R)` |
| `'B'` | — | `split("so", 2*rank+1, 2*m)` |
| `'C'` | `m == rank` | `sp(2*rank, R)` |
| `'C'` | 否则 | `split("sp", rank, m)` |
| `'D'`（n=2·rank） | `ic=='c'` 或（`rank` 偶 且 `ic=='s'`），且 `m < rank-1` | `split("so", n, 2*m)` |
| `'D'` | 同上前缀，`m >= rank-1` 且 `rank` 奇 | `so*(n)` |
| `'D'` | 同上前缀，`rank` 偶，且 `(rank % 4 == 0) == (m == rank)` | `so*(n)[1,0]` |
| `'D'` | 同上前缀，否则 | `so*(n)[0,1]` |
| `'D'` | 不等秩分支（`ic=='s'` 且 `rank` 奇，或其他未入上前缀者）且 `rank > 4` | `split("so", n, 2*m+1)` |
| `'D'` | 不等秩且 `rank <= 4` 且 trivial | `so(7,1)` |
| `'D'` | 不等秩且 `rank <= 4` 且非 trivial | `so(5,3)`（注释：unequal-rank D4 任何非零 grading 都是 so(5,3)） |
| `'E'` | trivial 且（`ic=='c'` 或 `rank > 6`） | `e{rank}`（提前返回，无括号） |
| `'E'` | rank 6，`ic=='c'`，`m ∈ {1,6}` | `e6(so(10).u(1))` |
| `'E'` | rank 6，`ic=='c'`，其他 m | `e6(su(6).su(2))` |
| `'E'` | rank 6，`ic!='c'`，trivial | `e6(f4)` |
| `'E'` | rank 6，`ic!='c'`，非 trivial | `e6(R)` |
| `'E'` | rank 7，`m == 7` | `e7(e6.u(1))` |
| `'E'` | rank 7，`m ∈ {2,5}` | `e7(R)` |
| `'E'` | rank 7，其他 | `e7(so(12).su(2))` |
| `'E'` | 其他 rank（else 分支），`bits & 0xCC != 0` | `e{rank}(e7.su(2))`（注释：E8 的 e8(e7.su(2)) 非紧位 {2,3,6,7} = 0xCC） |
| `'E'` | 其他 rank，`bits & 0xCC == 0` | `e{rank}(R)` |
| `'F'` | trivial | `f4` |
| `'F'` | `m >= 3` | `f4(so(9))` |
| `'F'` | 否则 | `f4(R)` |
| `'G'` | trivial | `g2` |
| `'G'` | 非 trivial | `g2(R)` |
| `'T'` | `ic == 'c'` | `u(1)` |
| `'T'` | 否则 | `gl(1,R)` |
| 其他 letter | — | `Err(LayoutInvariantViolation { invariant: "real form factor letter" })` |

### 2.5 `form_type_name`（pub）主循环【已实现】

注释：上游 `output::printType`，返回点分（`.` 分隔）的李代数名；`grading` 的位按 datum 单根索引。

1. 取 `layout.factors()` / `layout.letters()` / `layout.perm()`。
2. 宽度检查：任一 `perm` 位置 `>= 128` → `LayoutInvariantViolation { invariant: "grading key width" }`。
3. 遍历 `letters`（带条目下标 `entry` 与 inner-class 字符 `ic`）：
   - `entry > 0` 时先追加 `'.'`。
   - `factors.get(factor).ok_or(LayoutInvariantViolation { "letter/factor alignment" })?` 取 `(letter, rank)`。
   - `slice_rank = if letter == 'T' { 0 } else { rank }`（注释：环面因子半单秩为零，不消费 grading 位；上游 `gr >>= lt[i].semisimple_rank()`）。
   - `ic == 'C'`：追加 `complex_name(letter, rank)?`；`shift += slice_rank; factor += 1`；再以同样错误变体取伙伴因子 `(partner_letter, partner_rank)`；`shift += if partner_letter == 'T' { 0 } else { partner_rank }; factor += 1`（注释：Complex 条目消费两个因子与两段 rank 切片）。
   - 否则：拉回切片——`bits: u32`，对 `t in 0..slice_rank`，`position = perm.get(shift + t).ok_or(LayoutInvariantViolation { "permutation width" })?`，若 `(grading >> position) & 1 != 0` 则 `bits |= 1_u32 << t`；追加 `factor_name(bits, letter, rank, ic)?`；`shift += slice_rank; factor += 1`。
4. 返回 `Ok(name)`。

**【推断】边界观察**：
- `bits` 为 `u32` 且 `1_u32 << t`：本文件未对 `slice_rank > 32` 设防；`t >= 32` 时移位在调试构建会溢出 panic（未验证、亦未见相应检查）。
- `'D'` 分支的 `m < rank - 1`：`rank == 0` 会下溢；本文件假定 D 因子 `rank >= 4`（由布局保证与否不在字节内）。
- `'E'` 的 else 分支对一切非 6/7 的 rank 生效（含 8 以外者），均按 `0xCC` 掩码判定。
- 非测试代码无显式 `panic!`；集合访问均经 `.get(...).ok_or(...)`。

### 2.6 测试锚点（`mod tests`）【已实现】

辅助：`budget() = IntegerLatticeBudget::new(64, 100_000, 100_000, 128)`；`layout_of` = `InnerClass::new(datum.clone(), involution, 256).unwrap()` → `InnerClassLayout::build(&inner_class, &budget()).unwrap()`。

| 测试名 | 关键点 | 断言（`form_type_name`） |
|---|---|---|
| `a1_compact_and_split_names` | standard A1，恒等对合 | grading 0 → `"su(2)"`；1 → `"sl(2,R)"` |
| `a2_names_follow_the_inner_class_letter` | standard A2；紧布局与扭转 `[[0,1],[1,0]]` 布局 | 紧：0 → `"su(3)"`，1 → `"su(2,1)"`，2 → `"su(2,1)"`（弱递减：su(1,2) 打印为 su(2,1)）；扭转（不等秩）：0 → `"sl(3,R)"`（与 grading 无关） |
| `a3_unequal_rank_names_sl2h_and_sl4r` | standard A3，反序扭转 | 0 → `"sl(2,H)"`；1 → `"sl(4,R)"` |
| `complex_pair_name_consumes_two_factors` | standard A1×A1，交换对合 | 0 → `"sl(2,C)"` |
| `b2_names_cover_sp11_and_sp4r` | standard B2（`[[2,-2],[-1,2]]`），恒等 | 0 → `"so(5)"`；1 → `"so(3,2)"`（注释：bit 0 = datum 序中的长根在前）；2 → `"so(4,1)"` |
| `d4_unequal_rank_names` | standard D4，交换节点 2、3 的扭转 | 0 → `"so(7,1)"`；4 → `"so(5,3)"` |
| `torus_letters_name_u1_and_gl1r` | `from_simple_data(2, [[2]], Weight [2,0], Coweight [1,0])`，对合 `diag(1,-1)` | 0 → `"su(2).gl(1,R)"`；1 → `"sl(2,R).gl(1,R)"`（注释：A1 上 'c'、T1 上 's'） |

---

## 3. 两文件的接口关系（共同服务 presentation 层）

- **【已实现】** 两文件在所给字节内互不导入：无直接依赖边。
- **【已实现】** 共享 `StructureError`（topology.rs 经 `crate::StructureError`，form_name.rs 经 `crate::error::StructureError`），并都建立在根数据/内类管线的下游产物之上：topology.rs 的入参是 Cartan 对合矩阵 `theta: &[Vec<i32>]` + `&BasedRootDatum` + `&IntegerLatticeBudget`；form_name.rs 的入参是 `&InnerClassLayout` + `grading: u128`。
- **【已实现（注释）】** 输出侧都对应上游的呈现/输出关注点：topology.rs 注释指向 realredgp.cpp:73-75 的 `IsConnected` 状态位与 `dualComponentReps` 基大小；form_name.rs 注释指向 io/output.cpp 的 `printType` 打印。
- **【推断】** 二者分别产出"实形式的连通性/分量群秩"与"实形式的点分型名"，属于 presentation 层的两个互补侧面；对外可见面为 topology.rs 的 `pub dual_component_group_rank`（`dual_component_group_trivial` 仅 `pub(crate)`）与 form_name.rs 的 `pub form_type_name`。**【推断】** 调用方（未在字节内）可把分量群平凡性与型名拼成完整的形式描述。

---

## 4. 限制与未覆盖面

1. **正确性声明缺失**：本草案不做任何数学验收；上游行号引用均为注释原样转录，未核对上游源码。
2. **topology.rs**
   - `dual_component_group_rank` 在本文件无单元测试（仅 `dual_component_group_trivial` 被测）。
   - 无根 datum 分支（`IntegralBasis::full_space(rank)`，T_k 情形）无测试。
   - 两公有函数整段管线重复（仅末行不同），**【推断】** 存在漂移风险。
   - 未检查 `theta` 是否真为对合（如 `theta² = I`）；**【推断】** 输入合法性由调用方保证。
   - `invert_rational`、`saturated_kernel`、`ModTwoSubquotient` 等下游项的预算/错误语义不在本文件字节内，无法枚举其失败条件。
3. **form_name.rs**
   - 文档注释明确不覆盖折叠小写字母 `'f'`/`'g'`（对偶侧命名）；若出现将落入 `_ => Err(... "real form factor letter")`。
   - `bits: u32` 限制单因子切片宽度；`slice_rank > 32` 无防护（**【推断】** 溢出风险，未验证）。
   - 未测试分支（本文件字节内）：B/C 型的 `m == rank`（`sp(2n,R)`）、D 等秩 `so*` 系列（`[1,0]`/`[0,1]`）、E/F/G 全部分支、`'T'` 且 `ic=='c'` 的 `u(1)`、`'C'` 条目携 `'T'` 伙伴因子、`grading key width` / `letter/factor alignment` / `permutation width` 三个错误分支。
   - `InnerClass::new` 的第三参 `256` 与 `InnerClassLayout::build` 的预算语义不在本文件字节内。
4. **共同**：测试仅断言示例输出，未覆盖 `StructureError` 各错误路径；无属性测试或随机用例；`#[cfg(test)]` 外的 panic 可能性（索引、移位、下溢）仅以【推断】标注，未做验证。
```
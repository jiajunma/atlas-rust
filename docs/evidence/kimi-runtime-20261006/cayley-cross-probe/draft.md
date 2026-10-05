---
title: Cayley/Cross 分解与整对合分类：cayley_cross.rs 与 involution_classification.rs 来源包（草案）
source: atlas-rust/cayley-cross
ingestedAt: 2026-10-05T18:00:00Z
---

> 状态：**草案**，供维护者逐条核对后收录 `kb/sources/`。
> 证据范围：仅 `crates/atlas-real-group/src/cayley_cross.rs` 与 `crates/atlas-real-group/src/involution_classification.rs` 两个文件的完整字节。未阅读 crate 内其他模块（`InnerClass`、`TwistedInvolution`、`WeylAction`、`IntegerMatrix` 等的定义均不在证据范围内），不做任何数学验收、性能或正确性声明。

**标注约定**
- 【实现】：直接见于所给字节的代码行为（签名、控制流、错误分支）。
- 【注释】：代码注释/doc 注释中的声明，本包不独立验证。
- 【推断】：由所给控制流/算术直接推出的结论，未经运行验证。

---

## 1. 文件一裸签名清单：`crates/atlas-real-group/src/cayley_cross.rs`

### 1.1 导入（接口证据）

```rust
use crate::grading::try_capacity;
use crate::root_system::combine_roots;
use crate::twisted_involution::compose_matrices;
use crate::{
    InnerClass, RootId, RootKind, RootSystem, StructureError, TwistedInvolution, Weight, WeylAction,
};
```

### 1.2 pub 项与 impl 块

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CayleyCrossDecomposition {
    cayley_roots: Vec<RootId>,          // 私有字段
    cross_word: Vec<usize>,             // 私有字段
    cross_action: WeylAction,           // 私有字段
    twisted_involution: TwistedInvolution, // 私有字段
}

impl CayleyCrossDecomposition {
    pub fn build(
        inner_class: &InnerClass,
        twisted: &TwistedInvolution,
        max_peeling_steps: usize,
    ) -> Result<Self, StructureError>;

    pub fn cayley_roots(&self) -> &[RootId];        // doc: "Strongly orthogonal, positive, ascending."
    pub fn cross_word(&self) -> &[usize];           // doc: 应用顺序，首元素最先施加
    pub fn cross_action(&self) -> &WeylAction;      // doc: cross word 的重放复合
    pub fn twisted_involution(&self) -> &TwistedInvolution; // doc: 供消费方零成本同一性检查
}
```

说明【实现】：字段全部私有，除 `build` 外无其他构造入口；无 `pub` 常量、无 trait 定义、无其他 impl 块。

### 1.3 私有项（非 pub，列出以供完整核对）

```rust
enum Letter { Cayley(usize), Cross(usize) }        // 无 derive，文件私有

fn ensure_pairwise_orthogonal(
    root_system: &RootSystem,
    roots: &[RootId],
) -> Result<(), StructureError>;

fn long_orthogonalize(
    root_system: &RootSystem,
    mut roots: Vec<RootId>,
) -> Result<Vec<RootId>, StructureError>;

fn positive_form(root_system: &RootSystem, root: RootId) -> Result<RootId, StructureError>;

#[cfg(test)]
mod tests {
    fn inner(datum: &BasedRootDatum, involution: LatticeInvolution, budget: usize) -> InnerClass;
    fn tw(inner_class: &InnerClass, action: WeylAction) -> TwistedInvolution;
    // 6 个 #[test]，见 §6.1
}
```

---

## 2. 文件二裸签名清单：`crates/atlas-real-group/src/involution_classification.rs`

### 2.1 导入（接口证据）

```rust
use crate::grading::try_capacity;
use crate::integer_lattice::{reduce_basis_mod_two, saturated_kernel, IntegerMatrix};
use crate::{IntegerLatticeBudget, ModTwoSubspace, ModTwoVector, StructureError};
```

### 2.2 pub / pub(crate) 项与 impl 块

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvolutionClassification {
    compact: usize,   // 私有字段
    complex: usize,   // 私有字段
    split: usize,     // 私有字段
}

impl InvolutionClassification {
    pub const fn compact(&self) -> usize;                 // 无 doc 注释
    pub const fn complex(&self) -> usize;                 // 无 doc 注释
    pub const fn split(&self) -> usize;                   // 无 doc 注释
    pub const fn as_tuple(self) -> (usize, usize, usize); // 无 doc 注释
}

pub fn fiber_rank(
    weight_matrix: &[Vec<i32>],
    budget: &IntegerLatticeBudget,
) -> Result<usize, StructureError>;

pub fn classify_involution(
    matrix: &[Vec<i32>],
    budget: &IntegerLatticeBudget,
) -> Result<InvolutionClassification, StructureError>;

pub(crate) fn classify_plus_identity(
    plus_identity: &[Vec<i32>],
    budget: &IntegerLatticeBudget,
) -> Result<InvolutionClassification, StructureError>;
```

说明【实现】：`InvolutionClassification` 无 pub 构造器，字段私有，crate 外只能经 `classify_involution` 获得值。

### 2.3 私有项

```rust
fn is_involution(matrix: &[Vec<i32>]) -> Result<bool, StructureError>;

#[cfg(test)]
mod tests {
    fn budget() -> IntegerLatticeBudget; // IntegerLatticeBudget::new(16, 1_024, 10_000, 128)
    // 6 个 #[test]，见 §6.2
}
```

---

## 3. `CayleyCrossDecomposition`：表示、构造与调用契约

### 3.1 表示

【实现】存储四部分：`cayley_roots: Vec<RootId>`、`cross_word: Vec<usize>`（生成器下标，即 `datum.simple_roots()` 枚举位置，不是 `RootId`）、`cross_action: WeylAction`、以及输入 `twisted` 的克隆 `twisted_involution`。私有枚举 `Letter::{Cayley(usize), Cross(usize)}` 仅在构造期记录 peeling 字母，**不**存入结果。

【注释】struct doc 声明的构造期不变量：从 distinguished involution 出发，先按序对 cross word 做 twisted 共轭、再依次左乘每个 Cayley 根的反射，可复现输入；且每个 Cayley 根在其重放步上为 imaginary。doc 同时声明：两部分在 ports 间不唯一（"Atlas peels in its internal transducer order"），跨实现 diff 比较必须使用 replay-invariant 或 label 级结果，**不得**使用这些原始部分。

### 3.2 `build` 的分阶段流程（顺序即代码顺序）

**阶段 A：前置校验**
1. 【实现】`twisted.weyl_action().datum() != datum` → `Err(StructureError::DatumMismatch)`。
2. 【实现】provenance 门：`compose_matrices(twisted.weyl_action().matrix(), delta.weight_matrix())? != stored.weight_matrix()` **或** coweight 侧同样比较不成立 → `Err(StructureError::DistinguishedInvolutionMismatch)`；`compose_matrices` 的错误经 `?` 传播。`stored = twisted.root_involution().involution()`，`delta = inner_class.distinguished_involution().involution()`。【注释】该门保证存储的对合恰为 `w after delta`（对本 delta），并迫使 `w^{-1} = delta w delta`，从而支撑终止性论证。

**阶段 B：生成器表与 twist 置换**
3. 【实现】`simple_ids`：对 `datum.simple_roots()` 逐个 `root_system.id_of(root)`，缺失 → `InvalidRootAutomorphism`；容量经 `try_capacity(semisimple_rank)?`。
4. 【实现】`twist: Vec<usize>`：`delta_data.image(simple_id)` 为 `None` → `InvalidBasedAutomorphism`；其像不在 `simple_ids` 中（`position` 失败）→ `InvalidBasedAutomorphism`。即把 based 自同构落成生成器下标置换。
5. 【实现】`identity = WeylAction::identity(datum)?`。

**阶段 C：peeling 循环（顺序与预算）**
6. 【实现】每轮按生成器下标升序扫描 `simple_ids`，取**第一个** descent（注释："lowest external descent first"）。descent 判定：`theta_image = current.root_involution().image(simple_id)`（`None` → `InvalidRootAutomorphism`），`image = delta_data.image(theta_image)`（`None` → `InvalidRootAutomorphism`），`coordinates = root_system.simple_coordinates(image)`（`None` → `IndexOutOfRange { index: image.0, upper_bound: root_system.roots().len() }`），条件为 `coordinates.iter().all(|&c| c <= 0)`。
7. 【实现】无 descent 时：若 `current.weyl_action() != &identity` → `CayleyCrossInvariantViolation { invariant: "peeling termination" }`；否则跳出循环。
8. 【实现】预算检查在**找到 descent 之后、施加步进之前**：`if steps == max_peeling_steps` → `CayleyCrossResourceLimit { resource: "peeling steps", limit: max_peeling_steps }`；随后 `steps = steps.checked_add(1).ok_or(StructureError::ArithmeticOverflow)?`。
9. 【实现】字母类别由 `current.root_involution().kind(simple_id)`（`None` → `InvalidRootAutomorphism`）决定：
   - `RootKind::Real` → 记 `Letter::Cayley(generator)`；`new_action = s_g.compose(current)?`。
   - `RootKind::Complex` → 记 `Letter::Cross(generator)`；`new_action = s_g.compose(current)?.compose(&s_{twist[g]})?`。
   - `RootKind::Imaginary` → `CayleyCrossInvariantViolation { invariant: "descent kind" }`（注释：imaginary 根不可能是 descent）。
   - 其中 `s_g = WeylAction::simple_reflection(datum, generator)?`；每步末 `current = TwistedInvolution::new(datum, root_system, delta, new_action)?`。
   - 【实现】预算按字母计：Complex 步复合两次反射但 `steps` 只加 1。【注释】doc 称在 provenance 门下每步严格使 Weyl 长度减 1 或 2，`max_peeling_steps` 为防御性预算。
   - 【推断】预算为 0 且输入无需任何 peeling 步（首轮即无 descent）时不会触及 `CayleyCrossResourceLimit`；所给测试只固定了"s0 + limit 0"的失败路径。

**阶段 D：逆序重放收集**
10. 【实现】按 `letters.iter().rev()` 重放：`Cayley(g)` → `cayley_roots.push(simple_ids[g])`；`Cross(g)` → `cross_word.push(g)`，并对**已收集的每个** Cayley 根做生成器 g 的反射：`root_system.root(*entry)`（`None` → `IndexOutOfRange`）→ `datum.reflect_weight(generator, weight)?` → `root_system.id_of(&reflected)`（`None` → `InvalidRootAutomorphism`）原地替换。
    - 【推断】`cross_word` 由此处于"逆 peel 序"，与访问器 doc 所称"应用顺序（首元素最先施加）"一致。

**阶段 E：Cayley 集合后处理（顺序固定）**
11. 【实现】`ensure_pairwise_orthogonal`：对所有无序对，`bracket(l, r)? != 0 || bracket(r, l)? != 0` → `CayleyCrossInvariantViolation { invariant: "orthogonal Cayley set" }`。
12. 【实现】`long_orthogonalize`：先 `sort_unstable`；外层 `'restart` 循环升序扫描所有对，若 `combine_roots(rs, a, b, false)?` 为 `Some(sum)`（注释：B2 短根对、其和为根），则要求 `combine_roots(rs, a, b, true)?` 存在（`None` → `CayleyCrossInvariantViolation { invariant: "B2 pair" }`），以 `sum` 与 `positive_form(difference)?` 替换该对，重排序后 `continue 'restart`。【注释】每次替换产出长根、长根不再配对，长根数严格递增，故单次升序扫描足够且循环终止。
13. 【实现】逐根 `positive_form`：`simple_coordinates` 全 `>= 0` 则保留；否则对权重逐坐标 `checked_neg()`（溢出 → `ArithmeticOverflow`）构造 `Weight::new(negated)` 再 `id_of`（`None` → `InvalidRootAutomorphism`）；坐标缺失 → `IndexOutOfRange`。最后 `cayley_roots.sort_unstable()`。

**阶段 F：cross_action 与重放验证**
14. 【实现】`cross_action`：从 `identity.clone()` 起，按 `cross_word` 顺序右复合 `s_g`。
15. 【实现】重放验证：`replay = TwistedInvolution::new(datum, root_system, delta, identity)?`；按序施加 cross 字母（`s_g ∘ w ∘ s_{twist[g]}`，同阶段 C 的 Complex 模式）；再按（已升序排序的）`cayley_roots` 逐根：若 `replay.root_involution().kind(root) != Some(RootKind::Imaginary)` → `CayleyCrossInvariantViolation { invariant: "Cayley root imaginary" }`；否则左乘 `WeylAction::root_reflection(datum, root_system, root)?`。终态 `replay != *twisted` → `CayleyCrossInvariantViolation { invariant: "replay equality" }`。

### 3.3 调用契约小结

- 【实现】`twisted` 必须与 `inner_class` 同 datum（否则 `DatumMismatch`），且其存储对合必须恰为 `w ∘ delta`（weight 与 coweight 两侧矩阵均比较，否则 `DistinguishedInvolutionMismatch`）。【注释】distinguished involution 无法从 twisted involution 自身恢复，故需 `InnerClass` 参数及其 based-automorphism 保证。
- 【实现】所有失败路径均经 `Result`；非测试代码中无 `unwrap`/`panic!`/显式断言。
- 【实现】输出保证（访问器 doc）：`cayley_roots` 强正交、正根、升序；`cross_word` 为应用顺序；`cross_action` 为 cross word 的重放复合；`twisted_involution` 为输入克隆。
- 【注释】结果两部分不具备跨 port 唯一性（见 §3.1）。

### 3.4 `build` 错误分支一览（按首次可能出现的阶段）

| 错误 | 触发点（阶段） |
|---|---|
| `DatumMismatch` | A1 |
| `DistinguishedInvolutionMismatch` | A2 |
| `InvalidRootAutomorphism` | B3、C6、C9、D10、E13 |
| `InvalidBasedAutomorphism` | B4 |
| `IndexOutOfRange { index, upper_bound }` | C6、D10、E13 |
| `CayleyCrossResourceLimit { resource: "peeling steps", limit }` | C8 |
| `ArithmeticOverflow` | C8（`steps`）、E13（`checked_neg`） |
| `CayleyCrossInvariantViolation`，invariant ∈ `"peeling termination"` / `"descent kind"` / `"orthogonal Cayley set"` / `"B2 pair"` / `"Cayley root imaginary"` / `"replay equality"` | C7、C9、E11、E12、F15 |
| 经 `?` 传播：`try_capacity`、`compose_matrices`、`WeylAction::{identity, simple_reflection, root_reflection, compose}`、`TwistedInvolution::new`、`reflect_weight`、`bracket`、`combine_roots` 的错误 | 各阶段 |

---

## 4. `involution_classification`：分类规则

### 4.1 语义

【注释】`InvolutionClassification { compact, complex, split }` 是整对合在"identity / exchanged-pair / negated"整分解中三类因子的**个数（rank）**，分解本身被刻意不选定、不存储。

### 4.2 `classify_involution(matrix, budget)`（顺序即代码顺序）

1. 【实现】形状检查：`rank = matrix.len()`；任一行 `row.len() != rank` → `Err(InvalidIntegerMatrixShape)`（先于一切其他检查）。
2. 【实现】预算门：`drop(IntegerMatrix::from_i32_rows(matrix, budget)?)`——在立方复杂度的平方检查之前强制执行 rank/存储/系数预算；临时矩阵在分配 `theta + I` 前即drop，注释说明其不计入后续 live-entry 记账。
3. 【实现】`is_involution`：以 `i128` 全程 `checked_mul`/`checked_add`（溢出 → `ArithmeticOverflow`）计算 `M²` 并与单位阵逐元比较；任一元不符 → `Err(InvalidInvolution)`。
4. 【实现】构造 `theta + I`：对角元 `checked_add(1)`（溢出 → `ArithmeticOverflow`），容量经 `try_capacity(rank)?`。
5. 【实现】委托 `classify_plus_identity`。

### 4.3 `classify_plus_identity(plus_identity, budget)`（pub(crate)）

【注释】crate 内可见，供 central-torus 商计算在 Smith 基坐标下构造 `theta + I` 后复用同一 rank 内核；**对合前提由调用方负责**（该调用方不在所给字节内）。

【实现】规则（设 `rank = n`）：
1. 形状检查（同上 → `InvalidIntegerMatrixShape`）。
2. `plus_rank = rank − rank(saturated_kernel(IntegerMatrix::from_i32_rows(M, budget)?)?)`；`checked_sub` 失败 → `IntegerLatticeInvariantViolation`。
3. `complex`：把 `M` 每行的奇数坐标（`entry % 2 != 0`，负奇数亦计入）插入 `ModTwoSubspace::new(rank)?`，取其 `rank()`——即 `M mod 2` 行空间的 F₂ 秩。
4. `compact = plus_rank − complex`（`checked_sub` → `IntegerLatticeInvariantViolation`）。
5. `split = rank − (plus_rank + complex)`（`checked_add` → `ArithmeticOverflow`；`checked_sub` → `IntegerLatticeInvariantViolation`）。

【推断】由上述公式纯算术可得 `compact + 2·complex + split == rank`（在这些 `Result` 未提前失败的路径上）；这与 doc 的"exchanged-pair 因子"措辞相容，但本包不对其数学含义做验收。

### 4.4 `fiber_rank(weight_matrix, budget)`

【注释】`dualPi0(-theta^T)` 的 F₂ 维数，公式 `dim ker((q+I) mod 2) − dim span(plusBasis(q)) mod 2`（`q = -theta^T`），系 `Cartan_info` 打印的 fiber size 指数；doc 引用 `tori.cpp:162-173`、`subquotient.h:79`（**外部出处，无法从所给字节核验**）。

【实现】步骤：
1. 形状检查（方阵，否则 `InvalidIntegerMatrixShape`）。
2. `q_plus_i[row][col] = (-weight_matrix[col][row] + I).rem_euclid(2)`（转置取负加单位阵，mod 2）；行向量插入 `ModTwoSubspace`，`kernel_dim = rank − image.rank()`。
3. `q_minus_i[row][col] = -weight_matrix[col][row] − I`（**不**取 mod 2，保持精确 i32）；`IntegerMatrix::from_i32_rows(&q_minus_i, budget)?` → `saturated_kernel(&matrix, budget)?` → `reduce_basis_mod_two(&basis)?`。
4. 返回 `kernel_dim.saturating_sub(reduced.rank())`。
   - 【实现】注意此处为 `saturating_sub`（**不**报错），与 `classify_plus_identity` 中的 `checked_sub` 风格相反。

### 4.5 错误分支与优先级一览

| 错误 | 触发点 |
|---|---|
| `InvalidIntegerMatrixShape` | 三个函数入口的方阵检查（最先） |
| 预算错误（如测试固定的 `IntegerLatticeResourceLimit { resource: "rank", limit: 1 }`） | `from_i32_rows`；在 `classify_involution` 中先于 `is_involution` |
| `InvalidInvolution` | 仅 `classify_involution` 的 `is_involution` 检查 |
| `ArithmeticOverflow` | `is_involution`（i128 checked）、`theta+I` 对角 `checked_add`、`plus_rank + complex` |
| `IntegerLatticeInvariantViolation` | `classify_plus_identity` 中两处 `checked_sub` |
| 经 `?` 传播：`ModTwoSubspace::new/insert`、`ModTwoVector::from_ones`、`saturated_kernel`、`reduce_basis_mod_two`、`try_capacity` | 各处 |

【实现】`fiber_rank` 没有任何对合前提检查；`classify_plus_identity` 按 doc 约定不自查对合性。

---

## 5. 两文件的接口关系

- 【实现】两文件同属 crate `atlas-real-group`（路径推断自文件位置），但在所给字节内**互不引用**：两者的 `use` 列表均不包含对方定义的符号。
- 【实现】共享基础设施：均使用 `crate::grading::try_capacity` 与统一的 `StructureError` 错误枚举；均遵循"预算先行、全程 `Result`、非测试代码无 panic"的风格。
- 【实现】抽象层级不同：`cayley_cross.rs` 在 `InnerClass` / `TwistedInvolution` / `WeylAction` / `RootSystem` 层面工作；`involution_classification.rs` 在裸整数矩阵 `&[Vec<i32>]` + `IntegerLatticeBudget` + 整数格/模二子空间工具（`IntegerMatrix`、`saturated_kernel`、`reduce_basis_mod_two`、`ModTwoSubspace`、`ModTwoVector`）层面工作。
- 【注释】`classify_plus_identity` 的 doc 提到一个 crate 内消费方（central-torus 商计算），该消费方不在所给字节内。
- 【推断】两者分别服务"twisted involution 的 Cayley/cross 因子化"与"整格对合的因子计数/分量群秩"两条线；是否存在调用链（例如 `InnerClass` 内部使用 `classify_involution`）**无法**从所给字节确认，留作未覆盖面。

---

## 6. 测试锚点

### 6.1 `cayley_cross.rs`（辅助：`inner(...)` 用 `InnerClass::new(...).unwrap()`；`tw(...)` 用 `TwistedInvolution::new(...).unwrap()`）

| 测试 | 固定的行为 |
|---|---|
| `distinguished_involutions_decompose_to_empty_parts` | A2 数据 `[[2,-1],[-1,2]]`；identity 与 swap 两种 distinguished involution 下，identity `WeylAction` 分解结果 `cayley_roots` 与 `cross_word` 均为空（InnerClass 预算 6，peeling 预算 8）。 |
| `a1_x_a1_identity_gives_one_cayley_and_swap_gives_one_cross` | 数据 `[[2,0],[0,2]]`；identity 类 + `s0` → `cayley_roots == [id_of(Weight[1,0])]`、`cross_word` 空；swap 类 + `s0∘s1` → `cayley_roots` 空、`cross_word.len() == 1`（**未**固定 cross_word 具体内容）。 |
| `a2_identity_exercises_both_letter_kinds` | A2 identity 类：`s0` → `cayley == [id_of(Weight[1,0])]`；最长元 `s0s1s0` → `cayley == [id_of(Weight[1,1])]`、`cross_word == [0]`、`cross_action == simple_reflection(0)`。 |
| `b2_short_pinning_long_orthogonalizes_and_long_pinning_does_not` | B2 且 α₀ 短（`[[2,-1],[-2,2]]`）：最长元 `s0s1s0s1` → `cayley == [id([0,1]), id([2,1])]`、`cross_word == [1]`（长根化生效）；α₀ 长（`[[2,-2],[-1,2]]`）：`cayley == [id([1,0]), id([1,2])]`、`cross_word == [1]`（长根化为 no-op）。peeling 预算 16。 |
| `every_enumerated_twisted_involution_decomposes_and_replays` | 枚举 A2（identity+swap 两种 distinguished，预算 6）与 B2-长（仅 identity，预算 8）的 `inner_class.twisted_involutions(budget)`；peeling 预算 64 全部 `build(...).unwrap()` 成功；断言 Cayley 根两两 `bracket == 0`（单向）且 `decomposition.twisted_involution() == &twisted`。 |
| `rejects_budgets_and_foreign_provenance` | 三条负路径精确匹配：`s0` + 预算 0 → `CayleyCrossResourceLimit { resource: "peeling steps", limit: 0 }`；异 datum（B2 的 twisted 传入 A2 类）→ `DatumMismatch`；以 swap 为 backing 构造的 twisted 传入 identity 类 → `DistinguishedInvolutionMismatch`。 |

### 6.2 `involution_classification.rs`（辅助：`budget() = IntegerLatticeBudget::new(16, 1_024, 10_000, 128)`）

| 测试 | 固定的行为 |
|---|---|
| `identity_is_fully_compact` | `I₂` → `(2, 0, 0)`；并固定访问器与 `as_tuple` 一致。 |
| `a2_opposition_is_one_complex_factor` | `[[0,-1],[-1,0]]` → `(0, 1, 0)`。 |
| `parity_distinguishes_complex_from_compact_and_split_factors` | `[[1,1],[0,-1]]` → `(0,1,0)`；`[[1,2],[0,-1]]` → `(1,0,1)`（奇偶性区分 complex 与 compact+split）。 |
| `rejects_a_square_matrix_that_is_not_an_involution` | `2·I₂` → `Err(InvalidInvolution)`（`assert_eq` 比较 `Result`，依赖 `PartialEq`）。 |
| `rejects_a_ragged_matrix_before_reduction` | `[[1,0],[0]]` → `Err(InvalidIntegerMatrixShape)`（形状错误先于化简）。 |
| `enforces_the_exact_lattice_budget_before_classification` | 预算 `IntegerLatticeBudget::new(1, 16, 100, 128)` 下 `I₂` → `Err(IntegerLatticeResourceLimit { resource: "rank", limit: 1 })`（预算门先于分类）。 |

---

## 7. 限制与未覆盖面

1. `fiber_rank` 在所给字节内**没有任何测试**；其 doc 引用的外部出处（`tori.cpp:162-173`、`subquotient.h:79`、`Cartan_info`）无法从本证据核验。
2. `classify_plus_identity` 无直接测试（仅经 `classify_involution` 覆盖）；其 doc 所称的 crate 内复用方（central-torus 商）不可见。
3. `cayley_cross` 的失败分支覆盖不全：六种 `CayleyCrossInvariantViolation` 不变量串（`"peeling termination"`、`"descent kind"`、`"orthogonal Cayley set"`、`"B2 pair"`、`"Cayley root imaginary"`、`"replay equality"`）、`ArithmeticOverflow`、`IndexOutOfRange`、`InvalidRootAutomorphism`、`InvalidBasedAutomorphism` 均无专门负测试（枚举测试只走成功路径）。
4. 预算边界语义仅固定了"需要步进但预算为 0 → 报错"一条；【推断】 peeling 需求为 0 的输入在预算 0 下应成功（预算检查位于 descent 发现之后），但无测试固定。
5. A1×A1 swap 用例只固定 `cross_word.len() == 1`，未固定其内容；除 §6.1 所列外，`cross_action` 仅在 A2 用例中被断言。
6. 【注释】两个文件的 doc/注释中的声明（peeling 每步长度减 1 或 2、long-root 计数严格递增故 `long_orthogonalize` 终止、"imaginary 根不可能是 descent"、跨 port 不唯一等）本包仅如实转述，未做验证。
7. `Letter` 枚举无 `Debug`/`Clone` 等 derive 且为文件私有，构造期的字母序列对外界不可观测——外界只能看到后处理（正交化、长根化、正化、排序）之后的部分。
8. 本包不做任何数学验收、性能或正确性声明；所有"公式与 Atlas C++ 语义一致"类表述均超出本证据范围。
9. `classify_involution` 中 `is_involution` 为逐元三次方复杂度检查（i128）；本包仅记录其结构，不评估其成本。

---

## 8. 维护者核对清单（建议逐条打勾）

- [ ] §1.2/§2.2 裸签名清单与当前源文件一致（含可见性、derive、字段集）。
- [ ] §3.4 / §4.5 错误分支表无遗漏、错误 payload 字段名精确。
- [ ] §3.2 阶段 C 的预算检查位置（descent 发现之后、步进之前）与 `steps == max_peeling_steps` 语义。
- [ ] §4.4 `fiber_rank` 末尾为 `saturating_sub`（非 `checked_sub`）。
- [ ] §5 "两文件在所给字节内互不引用"的结论是否需要在更广上下文中修正。
- [ ] §6 测试锚点与现行测试逐一对应。
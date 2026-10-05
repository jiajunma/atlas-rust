```markdown
---
title: Grading 与 ModTwo：Cartan 紧性分级表与动态 F₂ 线性代数层（草案）
source: atlas-rust/grading-mod-two
ingestedAt: 2026-10-06T00:00:00Z
---

# 知识来源包草案：`grading.rs` × `mod_two.rs`

> **核对须知**：本草案仅依据以下两个源文件的完整字节撰写：
> - `crates/atlas-real-group/src/grading.rs`
> - `crates/atlas-real-group/src/mod_two.rs`
>
> 标注约定：
> - 【已实现】：可直接从字节读出的行为。
> - 【推断】：阅读推断（控制流/不变量推导），需维护者确认。
> - 【注释】：代码注释中的溯源/设计声明；其指向的外部文件（`gradings.cpp`、`cartanclass.cpp`、`bitvector.cpp`、`real_weyl.rs` 等）**不在本包字节内**，无法核验。
> - 【未覆盖】：两文件内 `#[cfg(test)]` 中没有对应测试。
>
> 本包不做任何数学验收、性能或正确性声明。两文件均未定义 enum；`StructureError` 变体系从 `crate` 根导入，其定义不在本包字节内。

---

## 第一部分：裸签名清单

### 1.1 `grading.rs`

**导入**（均来自 `crate` 根，定义不在本包内）：
`AdjointCartanFiber`, `AdjointFiberElement`, `CartanFiberElement`, `ModTwoSubspace`, `ModTwoVector`, `RootId`, `RootInvolutionData`, `RootSystem`, `StructureError`。

```rust
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Grading(ModTwoVector);              // 元组字段私有

impl Grading {
    /// 有 doc
    pub fn from_noncompact(
        imaginary_rank: usize,
        noncompact: impl IntoIterator<Item = usize>,
    ) -> Result<Self, StructureError>;

    pub fn imaginary_rank(&self) -> usize;      // 无 doc

    /// 有 doc
    pub fn is_noncompact(&self, imaginary_index: usize) -> Option<bool>;

    pub fn noncompact_indices(&self) -> impl Iterator<Item = usize> + '_; // 无 doc
}
```

```rust
#[derive(Clone, Debug)]
pub struct CartanGradingData {
    imaginary_simple_roots: Vec<RootId>,        // 以下字段全部私有
    simple_mod_two: Vec<ModTwoVector>,
    m_alphas: Vec<CartanFiberElement>,
    adjoint_m_alphas: Vec<AdjointFiberElement>,
    base_grading: Grading,
    grading_shifts: Vec<Grading>,
    adjoint: AdjointCartanFiber,
}

impl CartanGradingData {
    /// 有 doc
    pub fn build(
        root_system: &RootSystem,
        root_involution: &RootInvolutionData,
        adjoint: &AdjointCartanFiber,
    ) -> Result<Self, StructureError>;

    pub fn imaginary_rank(&self) -> usize;                       // 无 doc
    pub fn imaginary_simple_roots(&self) -> &[RootId];           // 无 doc

    /// 有 doc（"Bounded by `imaginary_rank()`."）
    pub fn imaginary_simple_root(&self, imaginary_index: usize) -> Option<RootId>;

    /// 有 doc
    pub fn m_alpha(&self, imaginary_index: usize) -> Option<&CartanFiberElement>;

    /// 有 doc
    pub fn adjoint_m_alpha(&self, imaginary_index: usize) -> Option<&AdjointFiberElement>;

    /// 有 doc
    pub fn base_grading(&self) -> &Grading;

    /// 有 doc（"Bounded by `adjoint_fiber().dimension()`."）
    pub fn grading_shift(&self, adjoint_basis_index: usize) -> Option<&Grading>;

    /// 有 doc
    pub fn adjoint_fiber(&self) -> &AdjointCartanFiber;

    /// 有 doc
    pub fn grading(&self, element: &AdjointFiberElement) -> Result<Grading, StructureError>;

    /// 有 doc
    pub fn element_from_grading(
        &self,
        target: &Grading,
    ) -> Result<AdjointFiberElement, StructureError>;
}
```

```rust
/// 模块私有（非 pub），有 doc
fn ensure_faithful_shifts(imaginary_rank: usize, shifts: &[Grading]) -> Result<(), StructureError>;

pub(crate) fn try_capacity<T>(capacity: usize) -> Result<Vec<T>, StructureError>;
```

**`#[cfg(test)] mod tests`**（模块私有辅助函数 + 9 个测试）：

```rust
fn integer_budget() -> IntegerLatticeBudget;   // IntegerLatticeBudget::new(64, 100_000, 100_000, 128)
fn adjoint_budget() -> AdjointFiberBudget;     // AdjointFiberBudget::new(integer_budget(), 50_000, 100_000)
fn build_chain(
    datum: &BasedRootDatum,
    involution: LatticeInvolution,
    max_roots: usize,
) -> (RootSystem, RootInvolutionData, AdjointCartanFiber);
fn ambient(rank: usize, indices: impl IntoIterator<Item = usize>) -> ModTwoVector;

#[test] fn simply_connected_a1_realizes_the_quasisplit_normalization();
#[test] fn a2_identity_gradings_are_a_bijection();
#[test] fn a2_twist_rejects_the_all_compact_grading();
#[test] fn a1_x_a1_swap_has_imaginary_rank_zero();
#[test] fn a_central_coweight_coordinate_distinguishes_m_alpha_from_its_adjoint_image();
#[test] fn b2_reduces_negative_coroot_coordinates_with_sign_safe_parity();
#[test] fn rejects_foreign_inputs_before_computing_tables();
#[test] fn an_injected_dependent_shift_column_is_rejected();
#[test] fn thirty_three_a1_factors_stay_dynamic();
```

测试额外导入：`crate::integer_lattice::IntegerLatticeBudget` 与 `crate::{AdjointFiberBudget, BasedRootDatum, CartanFiber, Coweight, LatticeInvolution, Weight}`。

### 1.2 `mod_two.rs`

**导入**：`crate::StructureError`。

```rust
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ModTwoVector {
    dimension: usize,       // 私有
    words: Vec<u64>,        // 私有
}

impl ModTwoVector {
    pub fn zero(dimension: usize) -> Result<Self, StructureError>;                  // 无 doc
    pub fn from_ones(
        dimension: usize,
        indices: impl IntoIterator<Item = usize>,
    ) -> Result<Self, StructureError>;                                              // 无 doc
    pub fn dimension(&self) -> usize;                                               // 无 doc
    pub fn bit(&self, index: usize) -> Option<bool>;                                // 无 doc
    pub fn xor_assign(&mut self, right: &Self) -> Result<(), StructureError>;       // 无 doc
    pub fn is_zero(&self) -> bool;                                                  // 无 doc
    /// 有 doc（"The `F_2` pairing"）
    pub(crate) fn dot(&self, right: &Self) -> Result<bool, StructureError>;
    fn toggle(&mut self, index: usize) -> Result<(), StructureError>;               // 私有，无 doc
}
```

```rust
/// 有 doc（BinaryMap::section 选举说明）；无 derive
pub(crate) struct CanonicalModTwoSection {
    pivots: Vec<Option<(ModTwoVector, u64)>>,   // 私有
}

impl CanonicalModTwoSection {
    pub(crate) fn new(dimension: usize, columns: &[ModTwoVector]) -> Result<Self, StructureError>; // 无 doc
    pub(crate) fn solve(&self, target: &ModTwoVector) -> Result<Option<u64>, StructureError>;      // 无 doc
}
```

```rust
/// 有 doc
pub(crate) trait ModTwoAmbientMap {
    fn source_dimension(&self) -> usize;                                            // 无 doc
    fn target_dimension(&self) -> usize;                                            // 无 doc
    fn apply(&self, source: &ModTwoVector) -> Result<ModTwoVector, StructureError>; // 无 doc
}
```

```rust
#[cfg(test)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ModTwoLinearMap {
    target_dimension: usize,    // 私有
    columns: Vec<ModTwoVector>, // 私有
}

#[cfg(test)]
impl ModTwoLinearMap {
    pub(crate) fn new(
        target_dimension: usize,
        columns: Vec<ModTwoVector>,
    ) -> Result<Self, StructureError>;
}

#[cfg(test)]
impl ModTwoAmbientMap for ModTwoLinearMap {
    fn source_dimension(&self) -> usize;
    fn target_dimension(&self) -> usize;
    fn apply(&self, source: &ModTwoVector) -> Result<ModTwoVector, StructureError>;
}
```

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModTwoSubspace {
    dimension: usize,                     // 私有
    pivots: Vec<Option<ModTwoVector>>,    // 私有
    rank: usize,                          // 私有
}

impl ModTwoSubspace {
    pub fn new(dimension: usize) -> Result<Self, StructureError>;                      // 无 doc
    pub fn dimension(&self) -> usize;                                                  // 无 doc
    pub fn rank(&self) -> usize;                                                       // 无 doc
    pub fn quotient_dimension(&self) -> usize;                                         // 无 doc
    /// 有 doc（一行）
    pub fn insert(&mut self, vector: ModTwoVector) -> Result<bool, StructureError>;
    pub fn contains(&self, vector: ModTwoVector) -> Result<bool, StructureError>;      // 无 doc
    /// 有 doc（一行）
    pub fn quotient_representative(&self, vector: ModTwoVector)
        -> Result<ModTwoVector, StructureError>;
    pub fn same_coset(
        &self,
        left: ModTwoVector,
        right: ModTwoVector,
    ) -> Result<bool, StructureError>;                                                 // 无 doc
    fn reduce(&self, mut vector: ModTwoVector) -> Result<ModTwoVector, StructureError>; // 私有
    pub(crate) fn right_kernel(&self) -> Result<Self, StructureError>;                 // 无 doc
    /// 有 doc（含 bitvector.cpp:673-697、real_weyl.rs 溯源）
    pub(crate) fn pivot_rows(&self) -> impl Iterator<Item = (usize, &ModTwoVector)>;
    pub(crate) fn basis_vectors(&self) -> impl Iterator<Item = &ModTwoVector>;         // 无 doc
}
```

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ModTwoSubquotient {
    numerator: ModTwoSubspace,          // 私有
    denominator: ModTwoSubspace,        // 私有
    complement_basis: Vec<ModTwoVector>,// 私有
}

impl ModTwoSubquotient {
    pub(crate) fn new(
        numerator: ModTwoSubspace,
        denominator: ModTwoSubspace,
    ) -> Result<Self, StructureError>;                                                 // 无 doc
    pub(crate) fn ambient_dimension(&self) -> usize;                                   // 无 doc
    pub(crate) fn dimension(&self) -> usize;                                           // 无 doc
    pub(crate) fn canonical_representative(
        &self,
        vector: ModTwoVector,
    ) -> Result<ModTwoVector, StructureError>;                                         // 无 doc
    pub(crate) fn to_coordinates(
        &self,
        vector: ModTwoVector,
    ) -> Result<ModTwoVector, StructureError>;                                         // 无 doc
    pub(crate) fn ambient_representative(
        &self,
        coordinates: &ModTwoVector,
    ) -> Result<ModTwoVector, StructureError>;                                         // 无 doc
    pub(crate) fn basis_representatives(&self) -> &[ModTwoVector];                     // 无 doc
    /// 有 doc
    pub(crate) fn validate_induced_map_to(
        &self,
        target: &Self,
        map: &impl ModTwoAmbientMap,
    ) -> Result<(), StructureError>;
}
```

```rust
fn word_count(dimension: usize) -> Result<usize, StructureError>;   // 私有，无 doc
fn lowest_set_bit(vector: &ModTwoVector) -> Option<usize>;          // 私有，无 doc
```

**`#[cfg(test)] mod tests`**（12 个测试，无辅助函数）：

```rust
#[test] fn dynamically_tracks_a_subspace_above_the_atlas_packed_rank_limit();
#[test] fn reduction_detects_dependent_vectors_and_dimension_mismatches();
#[test] fn exposes_deterministic_quotient_representatives_and_cosets();
#[test] fn handles_zero_dimension_and_word_boundaries_with_finite_field_parity();
#[test] fn eliminates_dependent_vectors_across_multiple_words();
#[test] fn insertion_keeps_the_basis_fully_reduced();
#[test] fn computes_right_kernels_from_a_canonical_row_space();
#[test] fn subquotient_uses_low_pivot_deterministic_coordinates();
#[test] fn rejects_oversized_dimensions_without_attempting_an_infallible_allocation();
#[test] fn canonical_sections_match_exhaustive_three_by_four_maps();
#[test] fn canonical_section_source_masks_are_checked_at_sixty_four_columns();
#[test] fn canonical_section_retains_dynamic_target_coordinates();
```

---

## 第二部分：职责与核心算法

### 2.1 `grading.rs`

**职责**【已实现】：为一个 Cartan 类（一次 Cartan 对合）构建并服务"简单虚根的紧性分级"表：`m_alpha`、伴随 `m_alpha`、基点分级与分级位移，并提供分级 ↔ 伴随纤维元素的双向换算。

**`Grading` 语义**：
- 【已实现】`Grading` 是 `ModTwoVector` 的新类型；置位 = NONCOMPACT。位 `i` 对应属主模型简单虚根列表的第 `i` 项（该列表是 `RootInvolutionData::imaginary_simple_roots` 在本 crate 确定性根序下的拷贝）。【注释】置位=非紧对应 `gradings.cpp:20-23`。
- 【注释】类型被刻意与 `crate::CartanFiber` / `AdjointCartanFiber` 的 ambient coweight mod-two 坐标区分：三者在 A2+恒等对合等常见情形维度相同，维度检查无法区分，必须由类型区分。
- 【已实现】`from_noncompact` 委托 `ModTwoVector::from_ones`；越界索引 → `StructureError::IndexOutOfRange`（经 `toggle`）。
- 【已实现】`is_noncompact(i)`：`i >= imaginary_rank()` 时返回 `None`，否则 `Some(bool)`；`noncompact_indices()` 按下标升序产出置位。

**`CartanGradingData::build` 流程**【已实现，按代码顺序】：
1. `root_system.datum() != root_involution.involution().datum()` → `Err(StructureError::DatumMismatch)`。
2. `adjoint.ambient_fiber().involution() != root_involution.involution()` → `Err(StructureError::CartanFiberInvolutionMismatch)`。
   - 【注释】ambient 纤维刻意不作为参数：值相等无法表达纤维身份，`m_alpha` 必须针对 `AdjointCartanFiber::ambient_fiber` 构建。
3. 取 `imaginary = root_involution.imaginary_simple_roots()`、`imaginary_rank`、`semisimple_rank = root_system.datum().semisimple_rank()`；用 `try_capacity` 预分配 4 个 Vec。
4. 对每个 `root_id`：
   - `root_system.coroot(root_id)` 为 `None` → `IndexOutOfRange { index: root_id.0, upper_bound: root_system.roots().len() }`；
   - `m_alpha` = `adjoint.ambient_fiber().element_from_coweight_mod_two(coroot)?`（错误传播）；
   - 伴随像：`adjoint.projection().source_coweight(coroot.clone())?` → `map_coweight(&bound)?` → `adjoint.element_from_coweight_mod_two(&image)?`。【注释】`Pi(y)_j = <alpha_j, y>` 正是伴随 `m_alpha` 的 bracket 向量，因此配对只保留在投影内的单一实现。
   - `simple_coordinates(root_id)` 为 `None` → 同上形状的 `IndexOutOfRange`；坐标按 `*coordinate % 2 != 0` 判奇（**含负奇数**，测试 6 钉住），奇坐标下标收集后经 `ModTwoVector::from_ones(semisimple_rank, odd)?` 存入 `simple_mod_two`。
5. `base_grading = Grading::from_noncompact(imaginary_rank, 0..imaginary_rank)?` —— 全 1。【注释】基点（伴随纤维零元）按 quasisplit 规范化把一切简单虚根判为非紧；其余元素的评分 = 对其典范 ambient 代表做仿射线性求值。
6. `grading_shifts`：对 `adjoint.basis_representatives()` 的每个代表，第 `i` 位置位 ⇔ `simple_mod_two[i].dot(representative)?` 为真；容量预分配为 `adjoint.dimension()`。
   - 【推断】`dot` 要求两侧 `dimension` 相等（否则 `RankMismatch`），故 `build` 隐含"`basis_representatives()` 元素维度 == `semisimple_rank`"这一运行时前提；该前提由被调类型保证，保证机制不在本包字节内。
7. `ensure_faithful_shifts(imaginary_rank, &grading_shifts)?`：把每个 `shift.0` 克隆插入 `ModTwoSubspace::new(imaginary_rank)?`；`insert` 返回 `false`（线性相关）→ `Err(StructureError::GradingShiftsNotFaithful)`。
   - 【注释】Atlas 在 `cartanclass.cpp:172` 用断言保证该性质；此处改为**无条件拒绝**。注释并称"没有已知公共构造路径能达到相关列，此检查为防御性"。
8. 存储 `adjoint: adjoint.clone()`。

**`grading(&self, element)`**【已实现】：
- `self.adjoint.canonical_representative(element)?`（错误传播；测试钉住外来元素 → `StructureError::CartanFiberMismatch`）；
- 第 `i` 位置位 ⇔ `!simple_mod_two[i].dot(&representative)?`（即基点全 1 与配对值的 XOR——【推断】自"基点全 1 + 置位条件取反"的代码形态）；
- 经 `Grading::from_noncompact` 返回。

**`element_from_grading(&self, target)`**【已实现，增广消元】：
1. `target.imaginary_rank() != self.imaginary_rank()` → `RankMismatch { expected, actual }`。
2. `augmented = imaginary_rank.checked_add(self.grading_shifts.len())` → 溢出 `ArithmeticOverflow`。
3. 向 `ModTwoSubspace::new(augmented)?` 插入每个 shift 列：低 `imaginary_rank` 位为分级位，第 `imaginary_rank + adjoint_basis_index` 位为标记位（记录这是哪一列）。【注释】约化右端时同步累积求解组合。
4. 右端 = `target XOR base`；因 base 全 1，实现为"收集 `target.is_noncompact(i) == Some(false)` 的紧位"。
5. `span.quotient_representative(...)?` 后，若余数在低 `imaginary_rank` 位有任何置位 → `Err(StructureError::ImpossibleGrading)`。
6. 否则按标记位把 `self.adjoint.basis_representatives()` 中对应代表 `xor_assign` 进 `ModTwoVector::zero(self.adjoint.datum().rank())?`，最后 `self.adjoint.element_from_ambient(representative)`（错误传播）。
- 【注释】唯一性 = 构造期检查的忠实性不变量；仿射像之外的分级 → `ImpossibleGrading`。

**`try_capacity<T>`**【已实现】：`Vec::new()` + `try_reserve_exact(capacity)`，失败映射 `AllocationFailed { requested: capacity }`。供本文件所有预分配使用（`pub(crate)`，其他调用方不在本包字节内）。

**其他**【注释】：`W_im` 轨道在 `crate::WeakRealFormPartition`，实形标签在 `crate::RealFormLabels`，强实形为后续层（这些类型不在本包字节内）。

### 2.2 `mod_two.rs`

**职责**【已实现/注释】：动态尺寸的 `F_2` 线性代数层：位打包向量、典范截面、行既约子空间、子商及其诱导映射校验。注释声明：与 Malachite 支撑的整数层刻意分离（编码有限向量空间元素，非无界整数）；该层为 Cartan 纤维结构层服务。

**`ModTwoVector`**【已实现】：
- 存储：`Vec<u64>` 位打包；`word_count(d) = (d + 63) / 64`，加法经 `checked_add`（`usize::MAX` → `ArithmeticOverflow`）。
- `zero`：`try_reserve_exact` → `resize(word_count, 0)`；分配失败 → `AllocationFailed { requested: word_count }`。
- `from_ones`：先 `zero` 再对每个下标 `toggle`。**重复下标会因两次翻转而抵消**（测试 `handles_zero_dimension_and_word_boundaries_with_finite_field_parity` 钉住：`[0,63,64,127,128,63]` 使位 63 为 `false`）。
- `bit(i)`：`i >= dimension` → `None`；否则按 `words[i/64] & (1 << (i%64))` 返回 `Some(bool)`。
- `xor_assign` / `dot`：维度不等 → `RankMismatch { expected: self.dimension, actual: right.dimension }`。`dot` 为坐标积的奇偶性：逐字 `(l & r).count_ones() & 1` 折叠 XOR，结果为 `parity == 1`。
- 【注释】派生的 `Ord`/`Hash` 是"任意但确定性的 map-key 全序，非数学序"；派生正确性依赖"所有构造器把 `dimension` 以上的 padding 位清零并保持为零"。

**`CanonicalModTwoSection`**【已实现】（【注释】原 `BinaryMap::section` 选举限制到其像）：
- `new(dimension, columns)`：
  - `columns.len() > 64`（`u64::BITS`）→ `ResourceLimitExceeded { limit: 64 }`（源坐标打包进 `u64` 掩码；目标向量仍动态）。
  - 每列维度 ≠ `dimension` → `RankMismatch`。
  - 对第 `index` 列：`image = column.clone()`、`preimage = 1 << index`；按行号升序对已存主元 `(basis, lift)` 消元（`image.bit(row)` 置位则 `image ^= basis; preimage ^= lift`）。
  - `lowest_set_bit(&image)` 为 `None` → **丢弃该列**（`continue`）。【注释】核关系不是额外主元，正如 `bitvector.cpp:section` 遗忘零化列；"依赖列被丢弃，即使其源标记在增广空间中是独立的；这一区分固定了可观测的实形种子代表"。
  - 否则取最低置位 `pivot`，把该主元位从**所有旧基行**中反向消去（同步 XOR `lift`），然后 `pivots[pivot] = Some((image, preimage))`。
- `solve(target)`：维度 ≠ `pivots.len()` → `RankMismatch { expected: pivots.len(), actual }`；按行升序消元并累积 `solution: u64`；`Ok(remainder.is_zero().then_some(solution))` —— 目标落在保留列的张成内时 `Some(掩码)`，否则 `None`。
- 【注释】一次分解服务同一映射的所有目标。

**`ModTwoSubspace`**【已实现】：
- 低主元约定（【注释】遵循 Atlas `BitVector::firstBit()`）；每次 `insert` 后基在**每个主元处既约**（RREF），与插入顺序无关，为本结构子商层提供确定性坐标。
- `insert(vector)`：`reduce` 后 `lowest_set_bit` 为 `None` → `Ok(false)`；否则把新主元位从旧行消去、存入 `pivots[pivot]`、`rank.checked_add(1)`（溢出 → `ArithmeticOverflow`），返回 `Ok(true)`。
- `reduce`（私有）：维度不等 → `RankMismatch`；按主元**升序**扫描，`vector.bit(pivot) == Some(true)` 且该行存在则 XOR。【注释】缺失的早期主元不意味着后期主元缺失；典范基在每个主元处已既约，升序扫描恰好清每个系数一次，如 Atlas `normalSpanAdd`。
- `contains` / `quotient_representative`：均为 `reduce` 的包装（前者判余为零）。`same_coset`：先查 `left.dimension != right.dimension` → `RankMismatch { expected: left.dimension, actual: right.dimension }`，再 XOR 后 `contains`。
- `right_kernel()`（`pub(crate)`）：对每个自由坐标（无主元），生成 `ones = [free] + {主元 p : 行 p 在 free 处置位}`，插入新子空间；容量 `rank+1` 经 `checked_add`（→ `ArithmeticOverflow`）与 `try_reserve_exact`（→ `AllocationFailed`）。
- `pivot_rows()`：按主元**升序**产出 `(pivot, row)`。【注释】这正是上游 `Gauss_Jordan` 提交的排序典范基（`bitvector.cpp:673-697`）；`real_weyl.rs` 的 R-group 核构造从这些行读生成元位，而 `right_kernel` 会把它们重新约化为新子空间。`basis_vectors()`：仅行（【推断】同一 Vec 迭代，亦为主元升序）。

**`ModTwoSubquotient`**【已实现】（`pub(crate)`；【注释】刻意 crate 私有：拥有低主元打包坐标，公开 API 是数学包装 `CartanFiber` 而非序列化格式）：
- `new` 不变量检查（任一失败 → 对应错误）：
  1. 维度不等 → `RankMismatch { expected: numerator.dimension, actual: denominator.dimension }`；
  2. `denominator.rank > numerator.rank` → `ModTwoSubquotientInvariantViolation`；
  3. 分母某基向量不被分子 `contains` → `ModTwoSubquotientInvariantViolation`；
  4. `complement_count = numerator.rank.checked_sub(denominator.rank)` 下溢 → `ModTwoSubquotientInvariantViolation`（【推断】防御性，前置检查 2 使其不可达）；
  5. 收集"主元位未被分母占用"的分子主元行作为补基；最终 `len != complement_count` → `ModTwoSubquotientInvariantViolation`（【推断】防御性）。
- `canonical_representative(vector)`：维度检查 → `RankMismatch`；`!numerator.contains(...)` → `NotInModTwoSubspace`；否则 `denominator.quotient_representative(vector)`。
- `to_coordinates(vector)`：典范代表在各补基向量的最低置位（`lowest_set_bit` 为 `None` → `ModTwoSubquotientInvariantViolation`，【推断】不可达，因补基均非零）处读位，产出 `dimension()` 维坐标。
- `ambient_representative(coordinates)`：维度检查 → `RankMismatch`；按坐标位 XOR 补基向量，得 `ambient_dimension()` 维代表。
- `validate_induced_map_to(target, map)`：`map.source_dimension() != self.ambient_dimension()` / `map.target_dimension() != target.ambient_dimension()` → `RankMismatch`；分子每个基向量的像须在 `target.numerator` 中，否则 `CartanFiberMapDoesNotDescend { relation: "numerator" }`；分母每个基向量的像须在 `target.denominator` 中，否则 `relation: "denominator"`。【注释】只查商基代表会让正规形选择在源分母向量的目标类非零时"看似"定义了映射。

**`ModTwoAmbientMap` / `ModTwoLinearMap`**：
- 【注释】trait 供域层在稠密矩阵属多余分配时直接实现；商层只需维度与"对基向量施加映射"。
- `ModTwoLinearMap` 为 `#[cfg(test)]` 稠密测试辅助：`new` 校验每列维度（`RankMismatch`）；`apply` 校验源维度后按置位列 XOR 累积。

---

## 第三部分：两文件接口关系

**`grading.rs` 对 `mod_two.rs` 的实际调用点**【已实现】：

| 调用点 | 位置 |
|---|---|
| `ModTwoVector`（类型；`Grading` 的私有元组字段） | `pub struct Grading(ModTwoVector)` |
| `ModTwoVector::from_ones` | `Grading::from_noncompact`；`build`（`simple_mod_two`、`grading_shifts`）；`element_from_grading`（增广列、右端差） |
| `ModTwoVector::dimension` / `bit` | `Grading::imaginary_rank` / `is_noncompact` / `noncompact_indices`；`element_from_grading` 余数与标记位读取 |
| `ModTwoVector::dot`（`pub(crate)`） | `build`（shift 列）；`grading`（逐根配对） |
| `ModTwoVector::zero` / `xor_assign` | `element_from_grading` 汇总代表 |
| `ModTwoSubspace::new` / `insert` / `quotient_representative` | `ensure_faithful_shifts`；`element_from_grading` 增广消元 |

**`mod_two.rs` 中未被 `grading.rs` 字节触及的项**【已实现（清单）+ 注释（用途声明）】：
`CanonicalModTwoSection`（含 64 列上限）、`ModTwoSubquotient` 全部方法、`ModTwoAmbientMap`、`ModTwoLinearMap`、`right_kernel`、`pivot_rows`、`basis_vectors`、`same_coset`、`contains`、`is_zero`、`quotient_dimension`。
- 【注释】这些项的用途指向 Cartan 纤维结构层与 `real_weyl.rs`（R-group 核构造读 `pivot_rows`），相关文件未提供，无法核验调用关系。
- 【注释】`ModTwoSubquotient` doc 称公开数学 API 为 `CartanFiber` 包装。

**其他观察**：
- 【已实现】`try_capacity` 定义在 `grading.rs`（`pub(crate)`）；`mod_two.rs` **未使用它**，而是在各处内联 `Vec::new() + try_reserve_exact + AllocationFailed` 的同一模式。
- 【已实现】两文件共享同一错误类型 `StructureError`（均从 `crate` 根导入），错误变体在两文件间经由 `?` 直接传播（如 `grading.rs` 中 `dot`/`insert` 的 `RankMismatch`）。

---

## 第四部分：错误分支与预算

### 4.1 `StructureError` 变体总表

| 变体 | 触发条件 | 抛出位置（文件/函数） | 测试锚点 |
|---|---|---|---|
| `DatumMismatch` | `root_system.datum() != root_involution.involution().datum()` | grading.rs `build` 首查 | `rejects_foreign_inputs_before_computing_tables` |
| `CartanFiberInvolutionMismatch` | `adjoint.ambient_fiber().involution() != root_involution.involution()` | grading.rs `build` 次查 | 同上 |
| `IndexOutOfRange { index: root_id.0, upper_bound: roots().len() }` | `coroot` / `simple_coordinates` 返回 `None` | grading.rs `build` | 【未覆盖】（测试数据索引均有效） |
| `RankMismatch { expected, actual }` | 见下分行 | 多处 | 多处 |
| └ `expected: imaginary_rank, actual: target.imaginary_rank()` | `element_from_grading` 入口分级维度不符 | grading.rs | 【未覆盖】 |
| └ 向量维度不符 | `xor_assign`、`dot`、`same_coset`、`reduce`（经 `insert`/`contains`/`quotient_representative`）、`CanonicalModTwoSection::new`（列）、`solve`（目标）、`ModTwoLinearMap::new`/`apply`、`ModTwoSubquotient::new`/`canonical_representative`/`ambient_representative`/`validate_induced_map_to` | mod_two.rs | `reduction_detects_...`（`{3,2}`）；`canonical_section_retains_...`（`{130,129}`） |
| `ArithmeticOverflow` | `word_count` 的 `checked_add`；`insert` 的 `rank.checked_add(1)`；`right_kernel` 容量 `checked_add`；`element_from_grading` 的 `imaginary_rank.checked_add(dimension)` | 两文件 | `rejects_oversized_dimensions_...`（`ModTwoVector::zero(usize::MAX)`）；其余分支【未覆盖】 |
| `AllocationFailed { requested }` | 所有 `try_reserve_exact` 失败（`try_capacity`、`ModTwoVector::zero`、`ModTwoSubspace::new`、`CanonicalModTwoSection::new`、`right_kernel`、`ModTwoSubquotient::new`、`to_coordinates`） | 两文件 | `rejects_oversized_dimensions_...`（`ModTwoSubspace::new(usize::MAX)` 经 `matches!` 断言）；其余分支【未覆盖】 |
| `ImpossibleGrading` | 增广消元余数在低 `imaginary_rank` 位有置位（分级不在仿射像中） | grading.rs `element_from_grading` | `a2_twist_rejects_the_all_compact_grading` |
| `GradingShiftsNotFaithful` | shift 列线性相关（`insert` 返回 `false`） | grading.rs `ensure_faithful_shifts` | `an_injected_dependent_shift_column_is_rejected`（重复列、零列） |
| `CartanFiberMismatch` | `grading()` 传入其他纤维实例的元素；**抛出点不可见**（经 `self.adjoint.canonical_representative(element)?` 传播，【推断】定义于 `AdjointCartanFiber` 所在文件） | grading.rs 传播 | `rejects_foreign_inputs_before_computing_tables` |
| `ResourceLimitExceeded { limit: 64 }` | `columns.len() > u64::BITS` | mod_two.rs `CanonicalModTwoSection::new` | `canonical_section_source_masks_are_checked_at_sixty_four_columns`（64 列通过、65 列拒绝） |
| `ModTwoSubquotientInvariantViolation` | `new` 的 4 处检查 + `to_coordinates` 的 `lowest_set_bit` 兜底 | mod_two.rs | 【未覆盖】（测试构造均满足不变量） |
| `NotInModTwoSubspace` | `canonical_representative` 的向量不在分子中 | mod_two.rs | 【未覆盖】 |
| `CartanFiberMapDoesNotDescend { relation }` | `relation: "numerator"` / `"denominator"` | mod_two.rs `validate_induced_map_to` | 【未覆盖】（且该函数在本两文件内无调用方；`relation` 字段类型在本包字节中不可见，两个字面量为 `"numerator"`/`"denominator"`） |

另：两文件大量经 `?` 传播来自其他类型的错误（如 `element_from_coweight_mod_two`、`projection().source_coweight`、`element_from_ambient`、`element_from_ambient` 等），具体变体取决于被调类型，不在本包字节内。

### 4.2 预算与资源限制

- 【已实现】唯一硬编码上限：`CanonicalModTwoSection` 源列 ≤ **64**（`u64` 掩码打包）；目标维度动态。
- 【已实现】所有分配走 `try_reserve_exact`（失败→`AllocationFailed`）；测试名声明 "without attempting an infallible allocation"。
- 【已实现】溢出防护点：`word_count`（`checked_add(+63)`）、`insert`（`rank.checked_add(1)`）、`right_kernel`（`rank.checked_add(1)` 容量）、`element_from_grading`（`imaginary_rank.checked_add(dimension)`）、`ModTwoSubquotient::new`（`checked_sub`，映射为 `ModTwoSubquotientInvariantViolation`）。
- 【已实现】测试预算常量（类型定义不在本包内，语义不可核验）：`IntegerLatticeBudget::new(64, 100_000, 100_000, 128)`；`AdjointFiberBudget::new(_, 50_000, 100_000)`；`build_chain` 的 `max_roots ∈ {2, 4, 6, 8, 66（=2*33）}`。
- 【已实现】动态维度实测点：`ModTwoVector`/`ModTwoSubspace` 在 129、130 维及 rank 33 用例下工作；无维度硬上限（仅溢出/分配失败）。

---

## 第五部分：顺序保证与确定性

- 【注释】`Grading` 与 `ModTwoVector` 派生的 `Ord` 为"任意但确定性的 map-key 序，**非数学序**"；`ModTwoVector` doc 称派生正确性依赖构造器清零 padding 位并保持为零。
- 【已实现】`ModTwoSubspace` 低主元（`lowest_set_bit` = 首个非零字的 `trailing_zeros`）；每次 `insert` 后基为全主元既约（RREF），【注释】"与插入顺序无关，提供确定性坐标"；`reduce` 按主元升序扫描；`pivot_rows` 按主元升序产出。
- 【已实现】`CanonicalModTwoSection` 保留**首批独立输入列**，依赖列丢弃；测试 oracle 钉住其确定性：对每个目标，解 = 数值最小的源掩码（`canonical_sections_match_exhaustive_three_by_four_maps`，穷举全部 `2^12` 个 3×4 映射 × 8 个目标）。
- 【已实现】虚根下标顺序 = `RootInvolutionData::imaginary_simple_roots` 的确定性根序拷贝；A2 测试钉住 index 0 = alpha_2（coroot `(-1, 2)`）、index 1 = alpha_1；shift 列在该根序下构成置换矩阵。
- 【已实现】`noncompact_indices()`、`basis_vectors()`、`pivot_rows()` 均按下标/主元升序迭代。
- 【已实现】`Grading::from_noncompact` 与 `ModTwoVector::from_ones` 对重复下标按翻转语义处理（偶次出现抵消）。

## 第六部分：panic / 断言面

- 【已实现】两文件非测试代码中**无** `panic!` / `assert!` / `unwrap` / `expect`。
- 【推断】直接索引点均依赖前置检查：`ModTwoVector::bit`/`toggle` 的 `words[i / 64]`（由 `index < dimension` 保护；`dimension > 0 ⇒ word_count ≥ 1`）；`ModTwoSubquotient::new` 中 `denominator.pivots[pivot]`（由前置维度相等检查保护，两子空间 `pivots` 长度等于各自维度）。
- 【注释】Atlas 在 `cartanclass.cpp:172` 以**断言**保证 shift 忠实性；本实现改为无条件返回 `Err`。
- 【已实现】测试代码使用 `assert!` / `assert_eq!` / `matches!` / `unwrap`（属测试锚点，非生产路径）。

---

## 第七部分：测试锚点

### 7.1 `grading.rs`（9 个）

| 测试 | 钉住的行为（关键断言） |
|---|---|
| `simply_connected_a1_realizes_the_quasisplit_normalization` | A1+恒等对合（`max_roots=2`）：`imaginary_rank=1`、`adjoint.dimension()=1`；`m_alpha(0)` 非单位元、`adjoint_m_alpha(0)` 为单位元；`grading_shift(0) = Grading[0]`；基点分级非紧、另一元素紧；`element_from_grading` 双向往返相等 |
| `a2_identity_gradings_are_a_bijection` | A2+恒等（`max_roots=6`）：`imaginary_rank=2`；根序 = `[id_of(Weight[0,1]), id_of(Weight[1,0])]`（index 0 = alpha_2）；`adjoint_m_alpha(0/1)` 的典范代表 = `ambient(2,[0])`/`ambient(2,[1])`；shift 为置换矩阵（`[1]`、`[0]`）；4 个元素的分级两两不同且全部往返 |
| `a2_twist_rejects_the_all_compact_grading` | A2+扭转对合（矩阵 `[[0,1],[1,0]]`，`max_roots=6`）：`imaginary_rank=1`、根 = `id_of(Weight[1,1])`（`Option` 比较）；`adjoint.dimension()=0`、`grading_shift(0).is_none()`、`m_alpha(0)` 单位元；基点分级 == `*base_grading()`；全紧分级 → `Err(ImpossibleGrading)` |
| `a1_x_a1_swap_has_imaginary_rank_zero` | A1×A1+交换对合（`max_roots=4`）：`imaginary_rank=0`；`imaginary_simple_root(0)`/`m_alpha(0)` 为 `None`；空分级与单位元往返 |
| `a_central_coweight_coordinate_distinguishes_m_alpha_from_its_adjoint_image` | rank-2 datum（Cartan `[[2]]`、Weight `(1,0)`、Coweight `(2,1)`，恒等，`max_roots=2`）：`m_alpha(0)` 非单位元、ambient 典范代表 = `ambient(2,[1])`；`adjoint_m_alpha(0)` 单位元 |
| `b2_reduces_negative_coroot_coordinates_with_sign_safe_parity` | B2+恒等（`max_roots=8`）：`imaginary_simple_root(1) = id_of(Weight[1,0])`；coroot `(2,-1)` 归约为 `ambient(2,[1])`（负奇坐标算奇），`m_alpha(1)` 与 `adjoint_m_alpha(1)` 的典范代表均为 `[1]` |
| `rejects_foreign_inputs_before_computing_tables` | B2 根系 + A2 数据 → `DatumMismatch`；扭转数据 + 恒等伴随纤维 → `CartanFiberInvolutionMismatch`；外来伴随纤维单位元传给 `grading()` → `Err(CartanFiberMismatch)` |
| `an_injected_dependent_shift_column_is_rejected` | 直接调用私有 `ensure_faithful_shifts`：重复列（`[Grading[0], Grading[0]]`，rank 2）与零列（`Grading[]`，rank 1）均 → `GradingShiftsNotFaithful` |
| `thirty_three_a1_factors_stay_dynamic` | 33 个 A1 因子（`max_roots=66`）：`imaginary_rank=adjoint.dimension()=33`；基点分级 33 个非紧位（`noncompact_indices().count()`）；基点与翻转位 7 的元素均往返 |

### 7.2 `mod_two.rs`（12 个）

| 测试 | 钉住的行为 |
|---|---|
| `dynamically_tracks_a_subspace_above_the_atlas_packed_rank_limit` | 130 维子空间插入位 0/32/64/129 后 `rank()=4`；`contains([0,64,129])` 真、`contains([1])` 假 |
| `reduction_detects_dependent_vectors_and_dimension_mismatches` | 重复插入返回 `false`；异维插入 → `Err(RankMismatch { expected: 3, actual: 2 })` |
| `exposes_deterministic_quotient_representatives_and_cosets` | 4 维中插入 `[1,3]`：`quotient_dimension()=3`；`[0,1,3]` 的商代表 = `[0]`；`same_coset([0,1,3],[0])` 真 |
| `handles_zero_dimension_and_word_boundaries_with_finite_field_parity` | 129 维：重复下标 63 抵消；跨字位 63/64/127/128 读数正确；`bit(129)=None`；0 维子空间 `insert` 假、`contains` 真 |
| `eliminates_dependent_vectors_across_multiple_words` | 129 维：`[0,64]`、`[64,128]` 独立（各返回真），`[0,128]` 相关（假） |
| `insertion_keeps_the_basis_fully_reduced` | 先插 `[1]` 再插 `[0,1]` 后：`pivots[0]=Some([0])`、`pivots[1]=Some([1])`（RREF 保持，直接读私有字段断言） |
| `computes_right_kernels_from_a_canonical_row_space` | 方程 `[1,2]`、`[0,1]` 的右核 rank 1，`contains([0,1,2])` 真、`contains([1,2])` 假 |
| `subquotient_uses_low_pivot_deterministic_coordinates` | 分子 = 全 `F_2^2`、分母 = span`[0,1]`：`dimension()=1`；`(1,0)` 的典范代表 = `(0,1)`；`to_coordinates((0,1)) = (1)`（1 维）；`ambient_representative((1)) = (0,1)` |
| `rejects_oversized_dimensions_without_attempting_an_infallible_allocation` | `ModTwoVector::zero(usize::MAX)` → `ArithmeticOverflow`；`ModTwoSubspace::new(usize::MAX)` → `AllocationFailed { .. }`（`matches!`） |
| `canonical_sections_match_exhaustive_three_by_four_maps` | 穷举 `0..2^12` 个 3 位 × 4 列映射与全部 8 个目标：`solve` 结果 == 暴力枚举的数值最小掩码（独立 oracle，断言消息 `map={encoding}, target={target}`） |
| `canonical_section_source_masks_are_checked_at_sixty_four_columns` | 64 列（第 63 列为 `[0]`）可构造且 `solve = Some(1<<63)`；第 65 列 → `ResourceLimitExceeded { limit: 64 }` |
| `canonical_section_retains_dynamic_target_coordinates` | 130 维列 `[0,129]`、`[64]`、`[0,64,129]`（第三列 = 前两列之和）：`solve(sum) = Some(3)`（选前两列、弃依赖列）；`solve([0]) = None`；异维目标 → `RankMismatch { expected: 130, actual: 129 }` |

---

## 第八部分：限制与未覆盖面

1. **证据边界**：`StructureError`、`RootSystem`、`RootInvolutionData`、`RootId`、`AdjointCartanFiber`、`AdjointFiberElement`、`CartanFiberElement`、`CartanFiber`、`LatticeInvolution`、`BasedRootDatum`、`Weight`、`Coweight`、`IntegerLatticeBudget`、`AdjointFiberBudget`、`WeakRealFormPartition`、`RealFormLabels` 等定义均不在本包字节内；其语义仅以调用点、测试断言与注释为据。
2. **64 列上限**：`CanonicalModTwoSection` 源坐标打包进 `u64`，>64 列拒绝；`grading.rs` 的 `element_from_grading` **未使用**该类型（改用动态 `ModTwoSubspace` + 标记位），测试钉住 rank 33 动态工作。
3. **仅测试使用的项**：`ModTwoLinearMap` 整体为 `#[cfg(test)]`；其 `ModTwoAmbientMap` 实现在本包中亦无生产调用方（`validate_induced_map_to` 的调用方不在字节内）。
4. **未覆盖的错误分支**【未覆盖】：`IndexOutOfRange`（build 内两处 `ok_or`）、`element_from_grading` 入口 `RankMismatch`、所有 `ArithmeticOverflow` 中除 `word_count` 外的三处、`AllocationFailed` 中除 `ModTwoSubspace::new(usize::MAX)` 外的各分配点、`ModTwoSubquotientInvariantViolation`（5 处）、`NotInModTwoSubspace`、`CartanFiberMapDoesNotDescend`（2 个 `relation` 值）。
5. **防御性检查**：`ensure_faithful_shifts` 仅经注入测试覆盖；【注释】"没有已知公共构造路径能达到相关列"。`ModTwoSubquotient::new` 的 `checked_sub` 与长度复核【推断】不可达。
6. **未测试的派生行为**：`Grading`/`ModTwoVector` 的 `Ord`/`Hash` 未在两文件测试中用作 map key；`dot` 在 `mod_two.rs` 内无直接测试（仅经 `grading.rs` 测试间接覆盖）；`Grading::from_noncompact` 的 `IndexOutOfRange` 路径无直接测试（doc 有声明）。
7. **维度前提**：`build`/`grading` 中 `dot` 要求 `simple_mod_two`（维 `semisimple_rank`）与伴随纤维代表维度一致【推断】；不一致时运行期 `RankMismatch`，无静态保证可见于本包。
8. **溯源注释未核验**：`gradings.cpp:20-23`、`cartanclass.cpp:172`、`bitvector.cpp` 的 `section` 与 `673-697`（`Gauss_Jordan`）、`BitVector::firstBit()`、`normalSpanAdd`、`BinaryMap::section`、`real_weyl.rs` 均仅见于注释。
9. **不做验收**：本包对"quasisplit 规范化"、"忠实性 ⇒ 唯一性"、"低主元典范形"等仅记录代码与注释的声明，不作数学正确性、性能或完备性结论。
```

以上为完整 Markdown 草案，可直接交由维护者逐条核对后收录进 `kb/sources/`。
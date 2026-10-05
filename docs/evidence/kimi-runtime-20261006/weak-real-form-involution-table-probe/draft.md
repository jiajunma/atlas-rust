```markdown
---
title: 弱实型划分与扭对合表来源包草案（weak_real_form.rs / involution_table.rs）
source: atlas-rust/weak-real-form-involution-table
ingestedAt: 2026-10-06T00:00:00Z
---

# 来源包草案：弱实型划分与扭对合表

## 0. 阅读约定与范围声明

- 覆盖文件（仅此两个文件的字节，未读取任何其他模块、文档或历史）：
  - `crates/atlas-real-group/src/weak_real_form.rs`
  - `crates/atlas-real-group/src/involution_table.rs`
- 标记约定：
  - 【实现】：可直接从字节读出的行为（签名、分支、常量、控制流）。
  - 【文档】：源码文档注释中的声明（含对上游 atlas 的对应关系、审计结论、成本界）。本包仅转述，无法从字节核实，不作背书。
  - 【推断】：阅读代码结构得出的结论，未经运行验证。
- 本包不做任何数学验收、性能或正确性声明；所有"已证明 / PROVED / O(1) / 成本上界"类表述均为【文档】转述。
- 两文件引用的外部符号（`StructureError` 各变体、`try_capacity`、`RealProjection`、`CartanGradingData`、`CayleyCrossDecomposition` 等）的语义以其自身模块字节为准，本包只记录在本两文件中的用法。

---

## 1. 文件一 `weak_real_form.rs`：裸签名清单

### 1.1 常量

```rust
pub(crate) const MAX_MASK_BITS: usize = 63;
const CLASS_SENTINEL: u32 = u32::MAX; // 私有
```

### 1.2 公开类型与 impl 块

```rust
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct WeakRealFormId(pub(crate) usize);

#[derive(Clone, Debug)]
pub struct WeakRealFormPartition {
    // 以下字段均私有：
    adjoint: AdjointCartanFiber,
    class_of_by_mask: Vec<u32>,
    representatives: Vec<AdjointFiberElement>,
}

impl WeakRealFormPartition {
    pub fn build(grading: &CartanGradingData, max_elements: usize) -> Result<Self, StructureError>;
    pub fn class_count(&self) -> usize;
    pub fn classes(&self) -> impl ExactSizeIterator<Item = WeakRealFormId>;
    pub fn class_of(&self, element: &AdjointFiberElement) -> Result<WeakRealFormId, StructureError>;
    pub fn class_of_mask(&self, mask: u64) -> Result<WeakRealFormId, StructureError>;
    pub fn class_representative(&self, class: WeakRealFormId) -> Option<&AdjointFiberElement>;
    pub fn quasisplit_class(&self) -> WeakRealFormId;
    pub fn adjoint_fiber(&self) -> &AdjointCartanFiber;
}
```

### 1.3 自由函数（pub / pub(crate)）

```rust
pub fn weak_real_form_at_representative(
    inner_class: &InnerClass,
    classification: &CartanClassification,
    twisted: &TwistedInvolution,
    raw_torus_factor: &[Rational],
) -> Result<WeakRealFormId, StructureError>;

pub(crate) struct MaskOrbits {
    pub(crate) class_of_by_mask: Vec<u32>,
    pub(crate) representative_masks: Vec<u64>,
}

pub(crate) fn walk_mask_orbits(
    dimension: usize,
    max_elements: usize,
    base: &[bool],
    alpha_columns: &[u64],
    m_alpha_masks: &[u64],
    limit_error: fn(&'static str, usize) -> StructureError,
) -> Result<MaskOrbits, StructureError>;
```

### 1.4 私有辅助项（补充列出，供完整核对）

```rust
fn project_torus_factor(theta: &LatticeInvolution, raw_torus_factor: &[Rational])
    -> Result<Vec<Rational>, StructureError>;
fn integral_pairing(torus_factor: &[Rational], root: &[i32]) -> Result<Integer, StructureError>;
fn weak_limit_error(resource: &'static str, limit: usize) -> StructureError;
fn seeded_class(
    class_count: usize,
    limit_error: fn(&'static str, usize) -> StructureError,
) -> Result<u32, StructureError>;
fn mask_of(fiber: &AdjointCartanFiber, element: &AdjointFiberElement) -> Result<u64, StructureError>;
fn element_of(fiber: &AdjointCartanFiber, mask: u64) -> Result<AdjointFiberElement, StructureError>;
fn mask_from_index(index: usize) -> Result<u64, StructureError>;
fn index_from_mask(mask: u64) -> Result<usize, StructureError>;
fn widened(value: usize) -> u128;
```

### 1.5 测试模块（`#[cfg(test)] mod tests`）

- 辅助：`integer_budget`、`adjoint_budget`、`classification_budget`、`simply_connected_a1`、`grading_chain`、`ambient`。
- 测试（11 个）：
  1. `synthetic_a1_representatives_match_the_frozen_weak_form_anchors`
  2. `synthetic_a1_gates_rank_and_noncentral_raw_factors`
  3. `synthetic_a1_rejects_a_foreign_same_rank_classification_first`
  4. `synthetic_a2_reflection_rejects_a_raw_basis_with_half_integral_projection`
  5. `a2_identity_has_two_weak_real_forms`
  6. `b2_identity_has_three_weak_real_forms_in_the_pinned_order`
  7. `simply_connected_a1_has_a_trivial_action_and_singleton_classes`
  8. `the_a2_diagram_twist_has_one_weak_real_form`
  9. `rejects_foreign_elements_and_undersized_budgets`
  10. `large_products_are_rejected_by_honest_budgets_not_rank_caps`
  11. `the_class_number_guard_rejects_the_sentinel_ordinal`

---

## 2. 文件二 `involution_table.rs`：裸签名清单

模块文档定位：扭对合表（KGB stage b）；按 Cartan 的连续轨道切片；编号为调用方加入 Cartan 的顺序（文档纪律为升序 `CartanId`），轨道内部为外序 BFS【文档】。

### 2.1 公开类型与 impl 块

```rust
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct InvolutionId(pub(crate) usize);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvolutionTableBudget {
    // 私有字段：
    max_involutions: usize,
    integer_lattice: IntegerLatticeBudget,
}

impl InvolutionTableBudget {
    pub const fn new(max_involutions: usize, integer_lattice: IntegerLatticeBudget) -> Self;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvolutionRecord {
    // 私有字段：
    element: WeylElement,
    involution: TwistedInvolution,
    mod_space: ModTwoSubspace,
    theta_plus_one_rho: Weight,
    involution_length: usize,
    weyl_length: usize,
    projection: RealProjection,
}

impl InvolutionRecord {
    pub fn weyl_element(&self) -> &WeylElement;
    pub fn twisted_involution(&self) -> &TwistedInvolution;
    pub fn theta(&self) -> &LatticeInvolution;
    pub fn mod_space(&self) -> &ModTwoSubspace;
    pub fn involution_length(&self) -> usize;
    pub fn weyl_length(&self) -> usize;
    pub fn theta_plus_one_rho(&self) -> &Weight;
    pub(crate) fn projection(&self) -> &RealProjection;
}

#[derive(Clone, Debug)]
pub struct InvolutionTable {
    // 私有字段：
    inner_class: Arc<InnerClass>,
    budget: InvolutionTableBudget,
    twist: Vec<usize>,
    reflections: Vec<WeylElement>,
    reflection_actions: Vec<WeylAction>,
    two_rho: Weight,
    records: Vec<InvolutionRecord>,
    index_by_permutation: BTreeMap<Vec<RootId>, InvolutionId>,
    cross_links: Vec<Vec<InvolutionId>>,
    orbits: Vec<(CartanId, usize, usize)>,
}

impl InvolutionTable {
    pub fn new(inner_class: &InnerClass, budget: InvolutionTableBudget) -> Result<Self, StructureError>;
    pub fn add_cartan(&mut self, classification: &CartanClassification, cartan: CartanId)
        -> Result<(InvolutionId, usize), StructureError>;
    pub fn lookup(&self, element: &WeylElement) -> Option<InvolutionId>;
    pub fn record(&self, id: InvolutionId) -> Option<&InvolutionRecord>;
    pub fn cross(&self, generator: usize, id: InvolutionId) -> Result<InvolutionId, StructureError>;
    pub fn cayley(&self, generator: usize, id: InvolutionId)
        -> Result<Option<InvolutionId>, StructureError>;
    pub fn simple_root_kind(&self, id: InvolutionId, generator: usize) -> Option<RootKind>;
    pub fn cartan_of(&self, id: InvolutionId) -> Option<CartanId>;
    pub fn involution_count(&self) -> usize;
    pub fn orbit_slice(&self, cartan: CartanId) -> Option<(InvolutionId, &[InvolutionRecord])>;
    pub fn root_system(&self) -> &RootSystem;
    pub fn inner_class(&self) -> &InnerClass;
    pub(crate) fn inner_class_shared(&self) -> &Arc<InnerClass>;
    fn gate_twisted(&self, twisted: &TwistedInvolution) -> Result<(), StructureError>; // 私有
}
```

### 2.2 私有辅助函数

```rust
fn stepped_length(
    current_length: usize,
    current_w_length: usize,
    neighbor_w_length: usize,
) -> Result<usize, StructureError>;

#[allow(clippy::too_many_arguments)]
fn push_record(
    inner_class: &InnerClass,
    budget: &InvolutionTableBudget,
    two_rho: &Weight,
    records: &mut Vec<InvolutionRecord>,
    index_by_permutation: &mut BTreeMap<Vec<RootId>, InvolutionId>,
    element: WeylElement,
    action: WeylAction,
    involution_length: usize,
    transported_projection: Option<RealProjection>,
) -> Result<InvolutionId, StructureError>;
```

本文件无 `pub(crate)` 自由函数，无独立常量项。

### 2.3 测试模块（`#[cfg(test)] mod tests`）

- 辅助：`table_budget`、`class_budget`、`context`、`filled_table`。
- 测试（7 个）：
  1. `a1_split_has_two_singleton_orbits_and_is_idempotent`
  2. `b2_split_orbits_match_the_classification_and_are_reproducible`
  3. `b2_records_are_canonical_from_theta`
  4. `b2_cayley_edge_appears_only_once_its_cartan_is_added`
  5. `twisted_a2_orbits_match_the_classification`
  6. `b2_projection_is_transported_not_recomputed`
  7. `budget_and_out_of_range_inputs_are_guarded`

---

## 3. 文件一职责与核心算法

### 3.1 职责概述

- `WeakRealFormPartition`：把一个伴随 Cartan 纤维（`AdjointCartanFiber`）划分为 `W_im` 轨道；轨道即该 Cartan 对合下内类的弱实型。分区持有类别表（`Vec<u32>`，按掩码下标）与每类一个确定性代表元【实现】。
- `WeakRealFormId`：弱实型的稳定编号。编号规则：按各类最小元的典范坐标整数序升序赋号；类 0 为恒等元所在轨道（quasisplit 规范化）【文档+实现：编号由 `walk_mask_orbits` 的升序播种实现，见 3.3】。文档另声明：stage-(d) 审计（`SEED_X0_DESIGN.md`）PROVED 该编号与上游内部 `RealFormNbr` 一致、不在 adapter 暂缓范围内，仅解释器的外部 `FormNumberMap` 顺序需要 adapter 置换；基本 Cartan 分区铸出的 id 兼任 crate 全局实型编号，由 `crate::RealFormLabels` 携带【文档，外部引用均无法从字节核实】。
- `weak_real_form_at_representative`：把存储在某一 Cartan 代表元处的原始 Atlas 语言环面因子归属到全局弱实型；文档称其为上游 `real_form_of`（`atlas-types.w:3878-3894`、`innerclass.cpp:1305-1355`）的代表元级内核【文档】。

### 3.2 `WeakRealFormPartition::build` 流程【实现】

输入：`grading: &CartanGradingData`、`max_elements: usize`（调用方对枚举纤维大小 `2^dimension` 的上界；文档称游走成本以该计数乘虚秩为界，故无第二个旋钮【文档】）。

1. `adjoint = grading.adjoint_fiber()`；`dimension = adjoint.dimension()`。
2. 闸门一：若 `dimension > MAX_MASK_BITS`（即 > 63）→ `Err(StructureError::WeakRealFormResourceLimit { resource: "mask bits", limit: MAX_MASK_BITS })`。
3. 以 `try_capacity` 预分配三个长度 `imaginary_rank = grading.imaginary_rank()` 的向量：`base: Vec<bool>`、`alpha_columns: Vec<u64>`、`m_alpha_masks: Vec<u64>`。
4. 对每个虚根下标 `imaginary_index in 0..imaginary_rank`：
   - `base[i] = grading.base_grading().is_noncompact(i)`，为 `None` → `IndexOutOfRange { index: i, upper_bound: imaginary_rank }`。
   - 构造列掩码 `column`：对 `adjoint_basis_index in 0..dimension`，取 `grading.grading_shift(adjoint_basis_index)`（`None` → `IndexOutOfRange { index, upper_bound: dimension }`），若 `shift.is_noncompact(i) == Some(true)` 则 `column |= 1_u64 << adjoint_basis_index`。
   - `element = grading.adjoint_m_alpha(i)`（`None` → `IndexOutOfRange { index: i, upper_bound: imaginary_rank }`），`translation = mask_of(adjoint, element)?`。
5. `walk_mask_orbits(dimension, max_elements, &base, &alpha_columns, &m_alpha_masks, weak_limit_error)?`（错误构造器固定为本阶段的 `weak_limit_error`）。
6. 对每个代表掩码 `element_of(adjoint, mask)?` 重建代表元；返回 `Self { adjoint: adjoint.clone(), class_of_by_mask, representatives }`。

### 3.3 `walk_mask_orbits`：掩码轨道游走与编号【实现】

- 前置闸门：
  - `dimension > MAX_MASK_BITS` → `limit_error("mask bits", MAX_MASK_BITS)`（与 `build` 中的闸门重复存在）。
  - `required = 1_u128 << dimension`；若 `required > widened(max_elements)` → `limit_error("fiber elements", max_elements)`。注释说明：`required` 故意以 `u128` 计算且比较不做饱和，使 `dimension == 64` 不能在 64 位宿主上借 `usize::MAX` 上限漏过【文档】。`widened` 为 `u128::try_from(value).unwrap_or(u128::MAX)`，无 panic 路径【实现】。
  - `element_count = usize::try_from(required)`，失败 → `ArithmeticOverflow`。
- 调试期断言：对每个 `(column, translation)` 对，`debug_assert_eq!((translation & column).count_ones() % 2, 0)`。注释：各生成元的对合性即 `<alpha_s, alpha_s_vee> = 2` 经已验证的 grading 表模 2 读取；上游只假设每个生成元置换该群【文档】。
- 主循环：`class_of_by_mask` 初始化为全 `CLASS_SENTINEL`；`pending` 为栈（`push`/`pop`，LIFO）。按 `seed_index` 升序扫描，未分类者以 `seeded_class(class_count, limit_error)?` 取得新类号并入栈；弹出每个掩码后，对每个虚下标 `i`：
  - `paired = ((mask & alpha_columns[i]).count_ones() & 1) == 1`；
  - 若 `base[i] != paired`（即文档所述转移规则 `FiberAction`：grading 位 `base[i] XOR parity(mask AND alpha_columns[i])` 为非紧时），则 `image = mask ^ m_alpha_masks[i]`；`image` 未分类则赋当前类并入栈。
  - `index_from_mask(image)?` 失败 → `ArithmeticOverflow`。
  - 每类收尾 `class_count.checked_add(1)`，失败 → `ArithmeticOverflow`。
- 代表元提取：按掩码升序扫描 `class_of_by_mask`，当 `class == representative_masks.len()` 时推入 `mask_from_index(index)?`。因播种按升序、类号递增分配，`representative_masks[c]` 即类 `c` 的最小掩码【实现，且文档如此声明】；末尾 `debug_assert_eq!(representative_masks.len(), class_count)`。
- `seeded_class`：`u32::try_from(class_count)` 成功且不等于 `CLASS_SENTINEL` → `Ok(class)`；否则 `limit_error("classes", CLASS_SENTINEL as usize)`。文档注释：该守卫"原则上可达"——单连通积没有伴随 `m_alpha` 向量，每个掩码自成一类【文档】。

### 3.4 掩码 ↔ 元素互转【实现】

- `mask_of`：`debug_assert!(fiber.dimension() <= MAX_MASK_BITS)`；`fiber.coordinates(element)?`（错误直接传播；测试锚定外来元素经此表面化为 `CartanFiberMismatch`，见 §7）；逐位 `coordinates.bit(index) == Some(true)` → `mask |= 1_u64 << index`。
- `element_of`：`ModTwoVector::zero(fiber.datum().rank())?`，对 `fiber.basis_representatives()` 中每个置位 `index` 执行 `representative.xor_assign(basis)?`，最后 `fiber.element_from_ambient(representative)`。
- `mask_from_index` / `index_from_mask`：`u64::try_from` / `usize::try_from`，失败 → `ArithmeticOverflow`。

### 3.5 `weak_real_form_at_representative`：闸门序列与归属流程【实现】

依次执行，任一失败即返回对应错误：

1. **秩闸门**：`raw_torus_factor.len() != inner_class.datum().lattice_rank()` → `RankMismatch { expected: rank, actual: raw_torus_factor.len() }`。
2. **分类出处闸门**：以 `inner_class` 的 datum、root system、distinguished involution 与 `WeylAction::identity(inner_class.datum())?` 重建基本扭对合 `fundamental`（`TwistedInvolution::new(...) ?`）；若 `classification.cartan_classes().first().map(|class| class.representative()) != Some(&fundamental)` → `DatumMismatch`。注释说明：`CartanClassification` 不持有 `InnerClass` 句柄，故显式重建以做出处闸门；其工作量以自有 datum 与根系为界，未来可用保留出处令牌降低开销【文档】。
3. **datum 闸门**：`twisted.weyl_action().datum() != inner_class.datum()` → `DatumMismatch`。
4. **分解闸门**（`w · delta == theta`）：令 `delta = inner_class.distinguished_involution().involution()`、`theta = twisted.root_involution().involution()`；若 `compose_matrices(twisted.weyl_action().matrix(), delta.weight_matrix())? != theta.weight_matrix()` 或 coweight 侧同式不成立 → `DistinguishedInvolutionMismatch`（`compose_matrices` 自身错误经 `?` 传播）。
5. **代表元查找**：在 `classification.cartan_classes()` 中 `find(|class| class.representative() == twisted)`，无 → `CartanClassificationInvariantViolation { invariant: "synthetic real-form Cartan representative" }`。
6. **投影**：`project_torus_factor(theta, raw_torus_factor)?`（内部再做一次长度闸门，见 3.6）。
7. **整性闸门（先于虚 grading 提取）**：对 `inner_class.datum().simple_roots()` 中每个单根执行 `integral_pairing(&torus_factor, root.as_slice())?`。注释说明：等价于上游对加倍投影的偶配对检查；故意放在提取虚 grading 之前，使实 Cartan（虚基为空）无法绕过该闸门【文档】。
8. **非紧标记**：对 `cartan.grading().imaginary_simple_roots()` 的每个 `(imaginary_index, root_id)`：`inner_class.root_system().root(root_id)` 为 `None` → `CartanClassificationInvariantViolation { invariant: "synthetic real-form imaginary root" }`；若配对整数 `divisible_by(&Integer::from(2))` 则 `noncompact.push(imaginary_index)`（即偶配对标记非紧单虚根【实现】）。
9. **局部 → 全局**：`Grading::from_noncompact(cartan.grading().imaginary_rank(), noncompact)?` → `cartan.grading().element_from_grading(&grading)?` → `cartan.partition().class_of(&element)?` 得局部类 → `cartan.labels().label(local)`，为 `None` → `CartanClassificationInvariantViolation { invariant: "synthetic real-form label" }`；否则返回该全局 `WeakRealFormId`。

使用约束【文档】：`twisted` 必须恰好是 `classification` 存储的代表元；把一般扭对合移到代表元需要经表驱动的 Tits cross 作用一并搬运其环面因子，本函数不静默构建或扩展该表；后续 `minimal_torus_part` 下降还需要仍独立的 inverse-Cayley 操作。

### 3.6 `project_torus_factor` 与 `integral_pairing`【实现】

- `project_torus_factor`：长度闸门（`!= theta.lattice_rank()` → `RankMismatch`）；按行向量/右乘约定计算 `v -> (v + v·theta)/2`：对每个 `column`，`transported = Σ_row raw[row] * Rational::from(theta.weight_matrix()[row][column])`，`projected[column] = (raw[column] + transported) / Rational::from(2)`。
- `integral_pairing`：长度不等 → `RankMismatch { expected: root.len(), actual: torus_factor.len() }`（注意 expected/actual 的填充顺序）；配对为逐项 `value * Rational::from(coordinate)` 的 `fold` 求和；`Integer::try_from(pairing)` 失败 → `InvalidStrongTorusFactor`。
- `weak_limit_error`：构造 `StructureError::WeakRealFormResourceLimit { resource, limit }`。

---

## 4. 文件二职责与核心算法

### 4.1 职责概述

扭对合表（KGB stage b）：按 Cartan 连续切片的扭对合轨道，跨一个内类的所有实型共享【文档】。每条记录含：词级元素 `WeylElement`、矩阵级 `TwistedInvolution`（theta 加根分类）、模 2 去重子空间 `ModTwoSubspace`、`(1+theta)rho`（存为 `(2rho + theta·2rho)/2`，见 4.4）、两种长度，以及 `(1-theta)X^*` 像基对（`RealProjection`，上游记录字段 `lift_mat`/`M_real`）。关键设计：除像基对外所有字段在入表时从 theta 重新典范推导；像基对在轨道典范对合处由 `1-theta` 的阶梯化播种、随后沿 cross 作用 BFS 搬运——因为该基是路径依赖的，`y_lift` 的符号依赖它（上游 `involutions.h:104-105`、`involutions.cpp:196-208`、`242-243`）【文档】。

### 4.2 `InvolutionTable::new`：一次性预计算【实现】

- `twist: Vec<usize>`：对每个单根 id，`delta_data.image(simple_id)` 为 `None` → `InvalidBasedAutomorphism`；其像在 `simple_root_ids` 中的位置查不到 → `InvalidBasedAutomorphism`。
- `reflections` / `reflection_actions`：对每个生成元 `WeylAction::simple_reflection(datum, generator)?` 与 `WeylElement::from_action(root_system, &action)?`。文档注释：BFS 边不得每次调用重建反射【文档】。
- `two_rho`：遍历 `root_system.entries()`，对 `positivity()[id.0]` 为真的根逐坐标 `checked_add` 累加（溢出 → `ArithmeticOverflow`），包成 `Weight::new(...)`。
- 注释说明：内类同时持有 datum、根系与 distinguished involution，故 `new` 不需要跨输入闸门【文档】。

### 4.3 `add_cartan`：种子 + 外序 BFS【实现】

1. **幂等**：若 `orbits` 已有该 `cartan`，直接返回既有 `(InvolutionId(start), size)`。
2. `classification.cartan_class(cartan)` 为 `None` → `IndexOutOfRange { index: cartan.0, upper_bound: classification.cartan_classes().len() }`。
3. `gate_twisted(representative)?`（见 4.6）；`expected = class.twisted_involution_count()`。
4. 以 `try_reserve(expected)` 预分配 `records` 与 `cross_links`，失败 → `AllocationFailed { requested: expected }`（两处同形）。
5. **种子**：`seed_element = WeylElement::from_action(root_system, representative.weyl_action())?`；`CayleyCrossDecomposition::build(&self.inner_class, representative, seed_w_length.checked_add(1).ok_or(ArithmeticOverflow)?)?`；`length_sum = seed_w_length + cayley_count`（`checked_add`）；`length_sum % 2 != 0` → `InvolutionTableInvariantViolation { invariant: "length parity" }`；以 `involution_length = length_sum / 2`、`transported_projection = None` 调 `push_record`（即种子处从 theta 新鲜构建投影）。注释：`CayleyCrossDecomposition` 只是每类一次的工具，绝不在条目级使用【文档】。
6. **BFS**：游标 `cursor` 从 `start` 递增遍历 `self.records`。对每个节点克隆其 element/action/length/projection；对每个生成元 `g`：
   - 邻居：`reflections[g] * current * reflections[twist[g]]`（两次 `multiply`，错误传播）。
   - 若 `index_by_permutation` 已含 `neighbor.image_permutation()` → 链接既有 id，`continue`（去重命中）。
   - 新长度：`stepped_length(current_length, current_element.length(), neighbor.length())?`。
   - 新作用：`reflection_actions[g].compose(&current_action)?.compose(&reflection_actions[twist[g]])?`。
   - 投影搬运：`current_projection.transported(self.reflection_actions[g].matrix())?`；注释强调用**普通生成元 s 而非 twist(s)**——delta 已并入 theta（上游 `involutions.cpp:242-243`）【文档】。
   - `push_record(..., Some(transported))?`，链接新 id。
   - 每访问一个节点推入一条 `links` 到 `cross_links`；`cursor.checked_add(1)` 溢出 → `ArithmeticOverflow`。
7. 收尾不变量：`size = records.len() - start`，`size != expected` → `InvolutionTableInvariantViolation { invariant: "orbit size" }`；否则 `orbits.push((cartan, start, size))`，返回 `(InvolutionId(start), size)`。

【推断】由于游标恰好覆盖本次新增的每条记录且每节点推入一条 links，`cross_links` 与 `records` 下标对齐；`cross()` 以 `cross_links[id.0]` 取值与此一致。

### 4.4 `push_record`：单一入表路径【实现】

1. 条目上限：`records.len() == budget.max_involutions` → `InvolutionTableResourceLimit { resource: "involutions", limit: budget.max_involutions }`（等于即拒绝）。
2. 由 `TwistedInvolution::new(datum, root_system, delta, action)?` 重建扭对合；`theta = involution.root_involution().involution()`。
3. `eigenlattice = negative_coweight_eigenspace(theta.coweight_matrix(), &budget.integer_lattice)?`（嵌套的 `IntegerLatticeBudget` 在此穿到逐条目的特征格约化）；`mod_space = reduce_basis_mod_two(&eigenlattice)?`。文档：`mod_space` 是 `X_*` 模 2 去重子空间，其有序基服务 Tits 阶段的 inverse-Cayley 修复【文档】。
4. `(1+theta)rho` 以 `two_rho` 计算：`theta_two_rho = theta.act_on_weight(two_rho)?`；逐坐标 `sum = plain.checked_add(reflected)`（溢出 → `ArithmeticOverflow`），`sum % 2 != 0` → `InvolutionTableInvariantViolation { invariant: "theta rho parity" }`，否则推入 `sum / 2`。
5. 投影：`Some(projection)` → `projection.check_against(theta)?`（注释：搬运在代数上保持 `lift_mat*m_real == 1-theta`，此处对新鲜推导的 theta 验证边数学【文档】），通过后采用搬运值；`None` → `RealProjection::build(theta)?`。
6. `weyl_length = element.length()`；`id = InvolutionId(records.len())`；键为 `element.image_permutation()` 拷入的 `Vec<RootId>`，`index_by_permutation.insert(key, id)`；推入记录。

【实现】对 seed 插入没有碰撞检查：`BTreeMap::insert` 语义下同键会静默覆盖旧值；字节中未见防护，依赖不同 Cartan 轨道键不重叠的调用纪律【推断部分】。

### 4.5 `stepped_length`【实现】

- `usize → isize` 转换失败、`checked_sub` 失败均 → `ArithmeticOverflow`。
- Weyl 长度差 `change == 2` → `current_length.checked_add(1)`；`change == -2` → `current_length.checked_sub(1)`（溢出均 → `ArithmeticOverflow`）；其余（含 0）→ `InvolutionTableInvariantViolation { invariant: "twisted length step" }`。注释：差 0 意味着该边固定此对合，去重命中已先行消费该情形【文档】。

### 4.6 `gate_twisted` 与查询面【实现】

- `gate_twisted`（私有，函数内局部 `use crate::twisted_involution::compose_matrices;`）：datum 不等 → `DatumMismatch`；weight 或 coweight 矩阵上 `compose_matrices(w, delta) != theta` → `DistinguishedInvolutionMismatch`（`compose_matrices` 错误传播）。注释：分解出处——代表元必须是**本内类** delta 下的 `w after delta`【文档】。
- `lookup`：以 `element.image_permutation()` 查 `BTreeMap`，返回 `Option<InvolutionId>`。文档：键为前向根置换，stage (a) 已钉其为完整相等键；同基数外来系统仍是调用方契约【文档】。
- `record`：`records.get(id.0)`（`Option`）。
- `cross`：`cross_links.get(id.0)` 无 → `IndexOutOfRange { index: id.0, upper_bound: cross_links.len() }`；`links.get(generator)` 无 → `IndexOutOfRange { index: generator, upper_bound: links.len() }`。文档：建表后 O(1)【文档】。
- `cayley`：`records.get(id.0)` / `reflections.get(generator)` 缺失 → 同形 `IndexOutOfRange`；计算 `reflection * record.element` 后查索引，未命中返回 `Ok(None)`（目标 Cartan 尚未加入）；文档：stage-(e) 契约要求先加入该型的上闭 Cartan 集，此后 `None` 即调用方不变量违例【文档】。
- `simple_root_kind`：一条访问器覆盖上游三个 `is_*_simple` 测试【文档】；记录或生成元缺失返回 `None`，否则 `record.involution.root_involution().kind(simple_id)`。
- `cartan_of`：扫描 `orbits`，`start <= id.0 < start + size` 者即所属。
- `orbit_slice`：返回 `(InvolutionId(start), &records[start..start + size])`。
- `involution_count` / `root_system` / `inner_class` / `inner_class_shared`（`pub(crate)`，返回 `&Arc<InnerClass>`）。

---

## 5. 两文件的接口关系

### 5.1 字节层面的事实【实现】

- 两文件**互不导入**：`weak_real_form.rs` 的 `use` 列表不含 `involution_table`，`involution_table.rs` 的 `use` 列表不含 `weak_real_form`。
- 共享导入：
  - `crate::grading::try_capacity`（两文件均用其做容量受检分配）。
  - `crate::twisted_involution::compose_matrices`：文件一顶部导入；文件二在 `gate_twisted` 内局部导入。
  - crate 根类型交集：`CartanClassification`、`InnerClass`、`LatticeInvolution`、`StructureError`、`TwistedInvolution`、`WeylAction`（另各自使用 `Grading`/`ModTwoVector` 与 `ModTwoSubspace`/`Weight`/`WeylElement` 等）。
- **同一闸门逻辑在两文件各出现一次**：datum 一致性 → `DatumMismatch`；weight 与 coweight 双矩阵上的 `compose_matrices(w, delta) == theta` 分解检查 → `DistinguishedInvolutionMismatch`。文件一的 `weak_real_form_at_representative` 额外多两道前置闸门：原始因子长度（`RankMismatch`）与"分类首代表 == 当场重建的基本扭对合"的出处闸门（`DatumMismatch`）。
- 对 `CartanClassification` 的使用面：文件一用 `cartan_classes()` 及类的 `representative()`、`grading()`（`imaginary_rank()`、`imaginary_simple_roots()`、`element_from_grading`）、`partition()`（`class_of`）、`labels()`（`label`，把局部类映到全局 `WeakRealFormId`）；文件二用 `cartan_class(cartan)`、`representative()`、`twisted_involution_count()` 与分类级 `twisted_involution_count()`（测试）。

### 5.2 文档层面的互相指向【文档 + 推断】

- 文件一 `weak_real_form_at_representative` 的注释称：把一般扭对合移到存储代表元需"table-backed Tits cross actions"搬运环面因子，本函数不建表。【推断】此处"表"即文件二的 `InvolutionTable`（其 `cross`/`cayley` 提供 cross/Cayley 链接），但字节中未出现把两文件连起来的调用方。
- 文件二 `mod_space()` 注释称其有序基服务"Tits 阶段的 inverse-Cayley 修复"；文件一注释称 `minimal_torus_part` 下降"还需要仍独立的 inverse-Cayley 操作"。两处都指向不在本字节范围内的后续 Tits/inverse-Cayley 层。
- 文件一注释提到标签层 `crate::RealFormLabels` 与平方类/强实层 `crate::StrongRealClassification`；文件二注释提到 `y_lift`、stage (a) 的相等键钉定与 stage (e) 契约。均为外部引用，无法从本字节核实。
- 测试层面的交叉一致性【实现】：两文件测试使用相同的标准 A2（`[[2,-1],[-1,2]]`）、B2（`[[2,-2],[-1,2]]`）Cartan 矩阵与相同的 A2 图扭转矩阵（`[[0,1],[1,0]]`）；预算辅助取值同形（`IntegerLatticeBudget::new(64, 100_000, 100_000, 128)`、`AdjointFiberBudget::new(.., 50_000, 100_000)`）。两文件测试锚定的是不同阶段（文件一：纤维掩码轨道与弱实型归属；文件二：扭对合轨道与投影搬运），字节中无跨文件共享的数值断言。

---

## 6. 错误分支、预算与顺序保证

### 6.1 文件一错误分支汇总【实现】

| 位置 | 触发条件 | 错误 |
|---|---|---|
| `build` | `dimension > MAX_MASK_BITS` (>63) | `WeakRealFormResourceLimit { resource: "mask bits", limit: MAX_MASK_BITS }` |
| `build` | `base_grading().is_noncompact(i)` 为 `None` | `IndexOutOfRange { index: i, upper_bound: imaginary_rank }` |
| `build` | `grading_shift(j)` 为 `None` | `IndexOutOfRange { index: j, upper_bound: dimension }` |
| `build` | `adjoint_m_alpha(i)` 为 `None` | `IndexOutOfRange { index: i, upper_bound: imaginary_rank }` |
| `walk_mask_orbits` | `dimension > MAX_MASK_BITS` | `limit_error("mask bits", MAX_MASK_BITS)` |
| `walk_mask_orbits` | `1_u128 << dimension > widened(max_elements)` | `limit_error("fiber elements", max_elements)` |
| `walk_mask_orbits` | `usize::try_from(required)` / 掩码↔下标转换 / `checked_add` 失败 | `ArithmeticOverflow` |
| `seeded_class` | 类序号不能放入 `u32` 或等于 `CLASS_SENTINEL` | `limit_error("classes", CLASS_SENTINEL as usize)` |
| `class_of_mask` | 掩码超出表长 | `IndexOutOfRange { index, upper_bound: class_of_by_mask.len() }` |
| `class_of_mask` | `usize::try_from(class)` 失败 | `ArithmeticOverflow` |
| `weak_real_form_at_representative` | 原始因子长度 ≠ 格秩 | `RankMismatch { expected: rank, actual: len }` |
| 同上 | 分类首代表 ≠ 重建的基本扭对合 | `DatumMismatch` |
| 同上 | `twisted` 的 datum ≠ 内类 datum | `DatumMismatch` |
| 同上 | `w·delta != theta`（weight 或 coweight） | `DistinguishedInvolutionMismatch` |
| 同上 | `twisted` 非任何类的代表元 | `CartanClassificationInvariantViolation { invariant: "synthetic real-form Cartan representative" }` |
| 同上 | 投影/配对非整（含实 Cartan 情形） | `InvalidStrongTorusFactor`（其 Display 被测试钉为 `"Torus factor does not define a valid strong involution"`） |
| 同上 | 虚单根 id 查根失败 | `CartanClassificationInvariantViolation { invariant: "synthetic real-form imaginary root" }` |
| 同上 | 标签缺失 | `CartanClassificationInvariantViolation { invariant: "synthetic real-form label" }` |
| `project_torus_factor` / `integral_pairing` | 长度不等 | `RankMismatch`（后者 `expected` 填 `root.len()`） |
| 多处 | `try_capacity` 失败 | 传播（变体定义在字节外） |
| `class_of`（经 `mask_of` → `coordinates`） | 外来纤维元素 | 测试锚定表面化为 `CartanFiberMismatch`（源头在字节外） |

### 6.2 文件二错误分支汇总【实现】

| 位置 | 触发条件 | 错误 |
|---|---|---|
| `new` | delta 把单根映出单根集（像缺失或位置缺失） | `InvalidBasedAutomorphism` |
| `new` | `two_rho` 逐坐标累加溢出 | `ArithmeticOverflow` |
| `add_cartan` | `cartan` 越界 | `IndexOutOfRange { index: cartan.0, upper_bound: classes.len() }` |
| `add_cartan` | `try_reserve(expected)` 失败（records / cross_links） | `AllocationFailed { requested: expected }` |
| `add_cartan` | 种子长度 `seed_w_length + 1 + cayley_count` 为奇 | `InvolutionTableInvariantViolation { invariant: "length parity" }` |
| `add_cartan` | 闭轨后 `size != expected` | `InvolutionTableInvariantViolation { invariant: "orbit size" }` |
| `gate_twisted` | datum 不一致 / 分解不成立 | `DatumMismatch` / `DistinguishedInvolutionMismatch` |
| `stepped_length` | Weyl 长度差非 ±2 | `InvolutionTableInvariantViolation { invariant: "twisted length step" }` |
| `push_record` | `records.len() == max_involutions` | `InvolutionTableResourceLimit { resource: "involutions", limit: max_involutions }` |
| `push_record` | `(2rho + theta·2rho)` 某坐标为奇 | `InvolutionTableInvariantViolation { invariant: "theta rho parity" }` |
| `push_record` | 搬运投影与本记录 theta 不符 | 经 `check_against(theta)?` 传播（变体在字节外） |
| `cross` / `cayley` | id 或 generator 越界 | `IndexOutOfRange`（upper_bound 分别为 `cross_links.len()` / `links.len()` / `records.len()` / `reflections.len()`） |
| 多处 | `checked_add/sub`、`isize/usize` 转换失败 | `ArithmeticOverflow` |
| 多处 | `multiply`/`compose`/`act_on_weight`/特征格与模 2 约化等外部调用失败 | 经 `?` 传播（变体在字节外） |

### 6.3 预算对象

- 文件一：单一旋钮 `max_elements: usize`（对 `2^dimension` 的上界）；硬常量 `MAX_MASK_BITS = 63`（掩码为 `u64`，63 保证所有 `1 << dimension` 移位在范围内【文档】）；类别表为 `Vec<u32>` 并保留 `CLASS_SENTINEL = u32::MAX`，故类数实际上限为 `u32::MAX - 1`【实现+文档】。
- 文件二：`InvolutionTableBudget { max_involutions, integer_lattice: IntegerLatticeBudget }`（`pub const fn new`）；条目数上限 + 嵌套整数格预算穿到逐条目的 `negative_coweight_eigenspace`；另有 `try_reserve` 的 `AllocationFailed` 防线。测试预算示例：`max_involutions` 取 1/4/8；`IntegerLatticeBudget::new(64, 100_000, 100_000, 128)`。

### 6.4 顺序保证

- 文件一【实现】：类号按升序最小掩码赋号（升序播种 + 递增类号 + 首次出现即代表）；`representative_masks[c]` 为类 `c` 的最小掩码；`quasisplit_class()` 恒返回 `WeakRealFormId(0)`，与掩码 0（恒等元）总被最先播种一致；`classes()` 返回 `impl ExactSizeIterator<Item = WeakRealFormId>`（`0..class_count` 的映射）。
- 文件二【实现+文档】：记录按调用方 `add_cartan` 顺序连续切片（文档纪律为升序 `CartanId`）；轨道内为外序 BFS；重复加入返回既有切片（幂等）；去重键为前向根置换；`cross` 链接建表后为存储直查（文档称 O(1)）。

### 6.5 panic / 断言面

- 文件一：仅 `debug_assert`（release 不生效）：生成元对合性奇偶断言、`representative_masks.len() == class_count`、`mask_of` 的维度上界。非测试代码无 `unwrap`/`expect`；`widened` 用 `unwrap_or(u128::MAX)`。
- 文件二：非测试代码无断言、无 `unwrap`/`expect`；全部算术经 `checked_*`/`try_from`。
- 两文件测试代码均大量使用 `unwrap`/`expect`（测试语义，不在库路径上）。

---

## 7. 测试锚点

### 7.1 文件一（11 个测试）

- `synthetic_a1_representatives_match_the_frozen_weak_form_anchors`：单连通 A1（`from_simple_data`，`Weight[2]`/`Coweight[1]`）；紧 Cartan（类 0 代表）配 `0` → `WeakRealFormId(0)`；分裂 Cartan（类 1 代表）配 `0` → `0`；紧 Cartan 配 `1/2` → `WeakRealFormId(1)`。
- `synthetic_a1_gates_rank_and_noncentral_raw_factors`：空切片 → `RankMismatch { expected: 1, actual: 0 }`；`1/4` → `InvalidStrongTorusFactor`。
- `synthetic_a1_rejects_a_foreign_same_rank_classification_first`：同秩外来 `BasedRootDatum::standard([[2]])` 的分类 → `DatumMismatch`（出处闸门先于一切类查找）。
- `synthetic_a2_reflection_rejects_a_raw_basis_with_half_integral_projection`：标准 A2 中恰有 2 个实根的反射 Cartan；构造投影为半整数的基向量原始因子；断言 `InvalidStrongTorusFactor` 且 Display 精确为 `"Torus factor does not define a valid strong involution"`。
- `a2_identity_has_two_weak_real_forms`：标准 A2 恒等，`max_elements = 4` → 2 类；恒等元与两个单基方向均属 quasisplit 类；其和（紧元）→ 类 1；每类代表元回查等于自身类号。
- `b2_identity_has_three_weak_real_forms_in_the_pinned_order`：标准 B2，上限 4 → 3 类；代表元（经 `canonical_representative`）精确钉为 `[ambient(2,[]), ambient(2,[0]), ambient(2,[0,1])]`；恒等元 → quasisplit。
- `simply_connected_a1_has_a_trivial_action_and_singleton_classes`：单连通 A1（作用平凡），上限 2 → 2 个单元类；`ambient(1,[0])` → 类 1。
- `the_a2_diagram_twist_has_one_weak_real_form`：A2 图扭转，上限 1 → 1 类；quasisplit 代表 == 恒等元；`class_representative(WeakRealFormId(1))` 为 `None`。
- `rejects_foreign_elements_and_undersized_budgets`：外来伴随纤维的恒等元 → `CartanFiberMismatch`；上限 3（需 4）→ `WeakRealFormResourceLimit { resource: "fiber elements", limit: 3 }`。
- `large_products_are_rejected_by_honest_budgets_not_rank_caps`：秩 33 对角 Cartan 积、上限 1_000_000 → `"fiber elements"` 上限拒绝；秩 64、上游纤维预算放宽（`AdjointFiberBudget::new(.., 200_000, 1_000_000)`）后由 `"mask bits"`（`limit: MAX_MASK_BITS`）拒绝——锚定拒绝来自诚实的预算门而非秩上限。
- `the_class_number_guard_rejects_the_sentinel_ordinal`：`seeded_class(5, _) == Ok(5)`；`seeded_class(u32::MAX as usize, _)` → `WeakRealFormResourceLimit { resource: "classes", limit: u32::MAX as usize }`。

### 7.2 文件二（7 个测试）

- `a1_split_has_two_singleton_orbits_and_is_idempotent`：`[[2]]`；`CartanId(0) → (InvolutionId(0), 1)`、`CartanId(1) → (InvolutionId(1), 1)`；重加幂等；`cross(0, id) == id`；基本记录：恒等元素、`involution_length = 0`、`theta_plus_one_rho = Weight[1]`、`mod_space` 秩 0；分裂记录：`weyl_length = 1`、`involution_length = 1`、`theta_plus_one_rho = Weight[0]`、`mod_space` 秩 1 且 `quotient_representative([1]) == zero`；`cartan_of(1) == CartanId(1)`；`simple_root_kind`：基本 `Imaginary`、分裂 `Real`。
- `b2_split_orbits_match_the_classification_and_are_reproducible`：总条数 == `classification.twisted_involution_count()`；各切片长度 == 各类 `twisted_involution_count()`；两次独立建表逐切片相等（可复现性）。
- `b2_records_are_canonical_from_theta`：每条记录 `weyl_length == element.length()`；`root_involution().image(root) == weyl_element.image(delta.image(root))`；`involution_length == (weyl_length + 新鲜分解的 Cayley 数)/2`；以 `two_rho = Weight[3,4]` 验证 `theta_plus_one_rho == (2rho + theta·2rho)/2`；`simple_root_kind` 与 `kind(simple_id)` 一致。
- `b2_cayley_edge_appears_only_once_its_cartan_is_added`：仅加 `CartanId(0)` 时 `cayley(0, fundamental) == None`；全部加入后得 `Some(target)`（`weyl_length = 1`、`involution_length = 1`）；两个生成元的 `cross` 均固定 fundamental。
- `twisted_a2_orbits_match_the_classification`：带图扭转的 A2；排序后轨道大小精确为 `[1, 3]`。
- `b2_projection_is_transported_not_recomputed`：arm64 oracle 钉定锚点【文档】；单连通 B2（基本权格基）；`theta = [[-1,0],[2,1]]` 的记录其 `projection().lift_mat == [[2],[-2]]`、`m_real == [[1,0]]`；而现场 `RealProjection::build(theta)` 得 `lift_mat == [[-2],[2]]`，且两者 `assert_ne`——锚定"搬运而非重算"。
- `budget_and_out_of_range_inputs_are_guarded`：`max_involutions = 1` 时加入第二个 Cartan → `InvolutionTableResourceLimit { resource: "involutions", limit: 1 }`；`CartanId(9)` → `IndexOutOfRange { index: 9, upper_bound: 2 }`；`record(InvolutionId(99)) == None`；`cross(5, InvolutionId(0))` → `IndexOutOfRange { index: 5, upper_bound: 1 }`；外来 A2 恒等元素 `lookup` → `None`。

---

## 8. 限制与未覆盖面

### 8.1 文件一

- 维度硬上限：`dimension > 63` 一律拒绝；边界 `dimension == 63` 允许但无测试覆盖（测试覆盖 64 拒绝、33 由预算拒绝）。【实现】
- `max_elements` 是唯一预算旋钮；成本界表述（元素数 × 虚秩）为【文档】，本包不验证。
- 类数以 `u32` 存储且保留哨兵值；`"classes"` 上限分支仅经 `seeded_class` 单元测试直接触发，未在完整分区流程中触发。【实现】
- `weak_real_form_at_representative` 只接受恰好存储的代表元；不搬运环面因子；出处闸门每次调用重建基本扭对合（注释称未来可用保留令牌优化）。【实现+文档】
- 生成元对合性仅为 `debug_assert`（release 不检查）；该性质由"已验证的 grading 表"保证的声称为【文档】。
- 编号与上游 `RealFormNbr` 一致的声明依赖外部审计文件 `SEED_X0_DESIGN.md`；上游行号引用（`atlas-types.w:3878-3894`、`innerclass.cpp:1305-1355`）均无法从字节核实。【文档】
- 本字节范围内未见测试触发的分支：`class_of_mask` 的越界/转换错误、各 `ArithmeticOverflow` 转换分支、`adjoint_fiber()` 访问器、`classes()` 之外的精确大小语义、边界 `max_elements == 2^dimension` 的显式断言（仅隐式通过）。【实现】

### 8.2 文件二

- 条目上限为包含式（`len == max_involutions` 即拒绝下一条）。【实现】
- `lookup` 仅以前向根置换为键；同基数外来系统的区分留给调用方契约【文档】；测试只钉了外来系统的 `None`。
- 种子插入 `index_by_permutation` 无碰撞检查（`BTreeMap::insert` 可静默覆盖）；字节中未见对"不同 Cartan 轨道键不重叠"的显式防护。【实现+推断】
- `classification` 与本表内类的一致性没有单一整体闸门；防护由 `gate_twisted`（对代表元）与收尾的 `"orbit size"` 不变量组合承担。【推断】
- `cayley` 在目标 Cartan 未加入时返回 `None`；stage-(e) 上闭集合契约为【文档】。
- 本字节范围内未见测试触发的分支：`"length parity"`、`"theta rho parity"`、`"orbit size"`、`"twisted length step"` 四个不变量错误；`AllocationFailed`；`cayley` 的 generator 越界；`lookup` 的 `Some` 命中（未被直接断言）；`cartan_of` / `orbit_slice` / `simple_root_kind` 的 `None` 分支。【实现】
- 对上游行号（`involutions.h:104-105`、`involutions.cpp:196-208`、`242-243`）与 arm64 oracle 运行锚点的引用无法从字节核实。【文档】

---

## 9. 待维护者逐条核对事项

1. §1、§2 裸签名清单与源码逐项一致（含 derive、可见性、字段、`pub const fn`）。
2. §6 错误分支表与源码逐条一致（变体名、字段值、字符串 `"mask bits"` / `"fiber elements"` / `"classes"` / `"involutions"` 及各 `invariant` 文案）。
3. 所有【文档】项（上游对应、审计声明、成本界、O(1)、stage 编号）是否收录为"注释声明"而非"已验证事实"。
4. 所有【推断】项（cross_links 下标对齐、quasisplit 与播种顺序的一致性、无碰撞检查的影响面）是否需要在正文或 kb 中降级/升级表述。
5. 外部符号（`try_capacity`、`RealProjection::{build, transported, check_against}`、`negative_coweight_eigenspace`、`reduce_basis_mod_two`、`CayleyCrossDecomposition`、`CartanGradingData` 系列方法、`StructureError` 变体定义、`RealFormLabels`）应分别以各自模块的来源包为准交叉链接。
```
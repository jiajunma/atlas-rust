```markdown
---
title: Cartan Fiber 与伴随 Cartan Fiber：表示、构造门控与预算语义（来源包草案）
source: atlas-rust/cartan-fibers
ingestedAt: 2026-10-05T18:00:00Z
---

# 0. 范围、依据与标注约定

- 本草案仅依据以下两个文件的完整字节：
  - `crates/atlas-real-group/src/cartan_fiber.rs`
  - `crates/atlas-real-group/src/adjoint_fiber.rs`
- 凡引用其它符号（如 `StructureError`、`ModTwoSubquotient`、`IntegerLatticeBudget`、`RootSystem`、`LatticeInvolution` 等）时，仅依据这两个文件中可见的调用点、签名与测试断言；这些符号的定义字节不在本包依据范围内。
- 标注约定：
  - 【已实现】：直接见于所给字节的行为（代码路径、错误分支、顺序）。
  - 【注释声明】：仅由文档注释/行内注释陈述，代码本身不独立可验证（尤其涉及数学断言处；本包不做任何数学验收、性能或正确性声明）。
  - 【推断】：阅读推断，需维护者核对。

---

# 1. 文件一裸签名清单：`cartan_fiber.rs`

## 1.1 导入（可见依赖面）

- `std::sync::Arc`
- `crate::mod_two::{ModTwoAmbientMap, ModTwoSubquotient}`
- `crate::integer_lattice::{negative_coweight_eigenspace, reduce_basis_mod_two, IntegerLatticeBudget}`
- `crate::{Coweight, LatticeInvolution, ModTwoSubspace, ModTwoVector, StructureError}`

## 1.2 类型与 impl 块

- `#[derive(Clone, Debug)] pub struct CartanFiber { model: Arc<CartanFiberModel> }`
- `#[derive(Debug)] struct CartanFiberModel { involution: LatticeInvolution, quotient: ModTwoSubquotient }`（私有）
- `#[derive(Clone, Debug)] pub struct CartanFiberElement { model: Arc<CartanFiberModel>, coordinates: ModTwoVector }`
- `impl PartialEq for CartanFiberElement`：`eq` 为 `Arc::ptr_eq(&self.model, &other.model) && self.coordinates == other.coordinates`
- `impl Eq for CartanFiberElement {}`

## 1.3 `impl CartanFiber`（含无注释项）

- `pub fn build(involution: &LatticeInvolution, budget: &IntegerLatticeBudget) -> Result<Self, StructureError>`
- `pub(crate) fn build_owned(involution: LatticeInvolution, budget: &IntegerLatticeBudget) -> Result<Self, StructureError>`
- `pub fn lattice_rank(&self) -> usize`（无注释项）
- `pub fn dimension(&self) -> usize`
- `pub fn involution(&self) -> &LatticeInvolution`（无注释项）
- `pub fn identity(&self) -> Result<CartanFiberElement, StructureError>`（无注释项）
- `pub fn element_from_ambient(&self, representative: ModTwoVector) -> Result<CartanFiberElement, StructureError>`
- `pub fn element_from_coweight_mod_two(&self, coweight: &Coweight) -> Result<CartanFiberElement, StructureError>`
- `pub fn canonical_representative(&self, element: &CartanFiberElement) -> Result<ModTwoVector, StructureError>`
- `pub fn coordinates<'e>(&self, element: &'e CartanFiberElement) -> Result<&'e ModTwoVector, StructureError>`
- `pub fn add(&self, left: &CartanFiberElement, right: &CartanFiberElement) -> Result<CartanFiberElement, StructureError>`
- `pub fn same_class(&self, left: &CartanFiberElement, right: &CartanFiberElement) -> Result<bool, StructureError>`（无注释项）
- `pub fn basis_representatives(&self) -> &[ModTwoVector]`
- `pub(crate) fn validate_induced_map(&self, target: &Self, ambient_map: &impl ModTwoAmbientMap) -> Result<(), StructureError>`
- `fn ensure_member(&self, element: &CartanFiberElement) -> Result<(), StructureError>`（私有）

## 1.4 `impl CartanFiberElement`

- `pub fn is_identity(&self) -> bool`（无注释项）

## 1.5 私有自由函数

- `fn mod_two_i_plus_coweight_kernel(involution: &LatticeInvolution) -> Result<ModTwoSubspace, StructureError>`

## 1.6 测试模块 `#[cfg(test)] mod tests`

- 辅助：`fn budget() -> IntegerLatticeBudget`（`IntegerLatticeBudget::new(8, 128, 1_000, 128)`）；`fn central_torus(rank: usize) -> BasedRootDatum`；`fn ambient(rank, indices) -> ModTwoVector`
- 测试（9 个）：
  1. `identity_involution_keeps_the_full_mod_two_torus`
  2. `negative_identity_has_a_trivial_fiber`
  3. `swap_involution_has_no_component_group`
  4. `uses_the_stored_coweight_action_and_negative_eigenlattice`
  5. `uses_coweight_rows_for_the_mod_two_numerator_without_transposing`
  6. `rejects_elements_from_a_different_fiber`
  7. `rejects_ambient_maps_that_do_not_descend_through_both_relations`
  8. `validates_coweight_rank_before_allocating_its_mod_two_image`
  9. `respects_the_caller_owned_exact_lattice_budget`

---

# 2. 文件二裸签名清单：`adjoint_fiber.rs`

## 2.1 导入与模块常量

- `std::sync::Arc`
- `crate::mod_two::ModTwoAmbientMap`
- `crate::{pair, BasedRootDatum, CartanFiber, CartanFiberElement, Coweight, IntegerLatticeBudget, LatticeInvolution, ModTwoVector, RootInvolutionData, RootSystem, StructureError, Weight}`
- `const ADJOINT_PERSISTENT_SQUARES: usize = 16;`（私有）

## 2.2 类型与 impl 块

- `#[derive(Clone, Debug, Eq, PartialEq)] pub struct AdjointFiberBudget { integer_lattice: IntegerLatticeBudget, max_persistent_entries: usize, max_projection_operations: usize }`
- `#[derive(Clone, Debug, Eq, PartialEq)] pub struct AdjointBasedRootDatum { datum: BasedRootDatum }`
- `#[derive(Debug)] struct AdjointProjectionModel { source_lattice_rank: usize, source_simple_roots: Vec<Weight>, target_datum: AdjointBasedRootDatum, max_projection_operations: usize }`（私有）
- `#[derive(Clone, Debug)] pub struct AmbientCoweight { model: Arc<AdjointProjectionModel>, coordinates: Coweight }`
  - `impl PartialEq for AmbientCoweight`（`Arc::ptr_eq && coordinates` 相等）；`impl Eq`
- `#[derive(Clone, Debug)] pub struct AdjointCoweight { model: Arc<AdjointProjectionModel>, coordinates: Coweight }`
  - `impl PartialEq for AdjointCoweight`（同上模式）；`impl Eq`
- `#[derive(Clone, Debug)] pub struct AdjointProjection { model: Arc<AdjointProjectionModel> }`
  - `impl ModTwoAmbientMap for AdjointProjection`
- `#[derive(Clone, Debug)] pub struct AdjointCartanFiber { source: CartanFiber, projection: AdjointProjection, fiber: CartanFiber }`
- `#[derive(Clone, Debug, Eq, PartialEq)] pub struct AdjointFiberElement(CartanFiberElement);`（元组字段无 `pub`，即私有）
- `#[derive(Clone, Debug)] pub struct FiberToAdjoint { source: CartanFiber, target: CartanFiber, projection: AdjointProjection }`（全字段私有）

## 2.3 `impl AdjointFiberBudget`

- `pub const fn new(integer_lattice: IntegerLatticeBudget, max_persistent_entries: usize, max_projection_operations: usize) -> Self`
- `pub fn integer_lattice_budget(&self) -> &IntegerLatticeBudget`
- 注：`max_persistent_entries` 与 `max_projection_operations` 无公开 getter【已实现】；模块内 `validate_adjoint_build_budget` 直接读字段。

## 2.4 `impl AdjointBasedRootDatum`

- `fn from_source(source: &BasedRootDatum) -> Result<Self, StructureError>`（私有）
- `pub fn as_based_root_datum(&self) -> &BasedRootDatum`（无注释项）
- `pub fn rank(&self) -> usize`（无注释项）

## 2.5 `impl AmbientCoweight` / `impl AdjointCoweight`

- `AmbientCoweight::pub fn as_coweight(&self) -> &Coweight`（无注释项）
- `AdjointCoweight::pub fn datum(&self) -> &AdjointBasedRootDatum`（无注释项）
- `AdjointCoweight::pub fn as_coweight(&self) -> &Coweight`（无注释项）

## 2.6 `impl AdjointProjection`（含无注释项）

- `fn from_source(source: &BasedRootDatum, max_projection_operations: usize) -> Result<Self, StructureError>`（私有）
- `pub fn source_lattice_rank(&self) -> usize`（无注释项）
- `pub fn target_datum(&self) -> &AdjointBasedRootDatum`（无注释项）
- `pub fn source_coweight(&self, coordinates: Coweight) -> Result<AmbientCoweight, StructureError>`
- `pub fn map_coweight(&self, source: &AmbientCoweight) -> Result<AdjointCoweight, StructureError>`
- `fn ensure_source(&self, source: &AmbientCoweight) -> Result<(), StructureError>`（私有）
- `fn apply_mod_two(&self, source: &ModTwoVector) -> Result<ModTwoVector, StructureError>`（私有）
- `fn check_projection_work(&self, vector_count: usize) -> Result<(), StructureError>`（私有）
- `impl ModTwoAmbientMap for AdjointProjection`：
  - `fn source_dimension(&self) -> usize`
  - `fn target_dimension(&self) -> usize`
  - `fn apply(&self, source: &ModTwoVector) -> Result<ModTwoVector, StructureError>`（委托 `apply_mod_two`）

## 2.7 `impl AdjointCartanFiber`（含无注释项）

- `pub fn build(root_system: &RootSystem, root_involution: &RootInvolutionData, source: &CartanFiber, budget: &AdjointFiberBudget) -> Result<Self, StructureError>`
- `pub fn datum(&self) -> &AdjointBasedRootDatum`（无注释项）
- `pub fn ambient_fiber(&self) -> &CartanFiber`
- `pub fn projection(&self) -> &AdjointProjection`（无注释项）
- `pub fn dimension(&self) -> usize`
- `pub fn identity(&self) -> Result<AdjointFiberElement, StructureError>`（无注释项）
- `pub fn element_from_ambient(&self, representative: ModTwoVector) -> Result<AdjointFiberElement, StructureError>`
- `pub fn element_from_coweight_mod_two(&self, coweight: &AdjointCoweight) -> Result<AdjointFiberElement, StructureError>`（无注释项）
- `pub fn coordinates<'e>(&self, element: &'e AdjointFiberElement) -> Result<&'e ModTwoVector, StructureError>`
- `pub fn canonical_representative(&self, element: &AdjointFiberElement) -> Result<ModTwoVector, StructureError>`（无注释项）
- `pub fn add(&self, left: &AdjointFiberElement, right: &AdjointFiberElement) -> Result<AdjointFiberElement, StructureError>`（无注释项）
- `pub fn same_class(&self, left: &AdjointFiberElement, right: &AdjointFiberElement) -> Result<bool, StructureError>`（无注释项）
- `pub fn basis_representatives(&self) -> &[ModTwoVector]`（无注释项）
- `pub fn fiber_map(&self) -> FiberToAdjoint`

## 2.8 `impl AdjointFiberElement` / `impl FiberToAdjoint`

- `AdjointFiberElement::pub fn is_identity(&self) -> bool`（无注释项）
- `FiberToAdjoint::pub fn apply(&self, source: &CartanFiberElement) -> Result<AdjointFiberElement, StructureError>`（无注释项）

## 2.9 私有自由函数（8 个）

- `fn validate_adjoint_build_budget(source: &BasedRootDatum, budget: &AdjointFiberBudget) -> Result<(), StructureError>`
- `fn root_basis_action(root_system: &RootSystem, root_involution: &RootInvolutionData) -> Result<Vec<Vec<i32>>, StructureError>`
- `fn clone_i32_matrix(matrix: &[Vec<i32>]) -> Result<Vec<Vec<i32>>, StructureError>`
- `fn clone_weights(weights: &[Weight]) -> Result<Vec<Weight>, StructureError>`
- `fn zero_square_matrix(rank: usize) -> Result<Vec<Vec<i32>>, StructureError>`
- `fn transpose_square(matrix: &[Vec<i32>]) -> Result<Vec<Vec<i32>>, StructureError>`
- `fn checked_product(left: usize, right: usize) -> Result<usize, StructureError>`
- `fn checked_sum(left: usize, right: usize) -> Result<usize, StructureError>`

## 2.10 测试模块 `#[cfg(test)] mod tests`

- 辅助：`integer_budget()`（`IntegerLatticeBudget::new(64, 100_000, 100_000, 128)`）；`adjoint_budget()`（`AdjointFiberBudget::new(integer_budget(), 50_000, 100_000)`）；`ambient(rank, indices)`
- 测试（9 个）：
  1. `identity_a1_projects_the_fiber_generator`
  2. `central_coweights_map_to_the_adjoint_fiber_kernel`
  3. `derives_the_adjoint_coweight_action_instead_of_reusing_root_rows`
  4. `requires_the_source_fiber_for_the_actual_involution_at_construction`
  5. `rejects_source_coordinates_bound_to_another_projection`
  6. `projects_a_nonsymmetric_actual_action_and_intertwines_the_dual_actions`
  7. `supports_dynamic_adjoint_rank_above_the_legacy_packed_limit`
  8. `rejects_adjoint_storage_before_allocating_target_matrices`
  9. `rejects_insufficient_projection_work_before_building_the_target`

---

# 3. `CartanFiber`：表示与约定

- 【注释声明】该类型是“附着于一个 Cartan 对合的 primary finite component group”；Atlas C++ 源码分析将其描述为 `Y / 2Y`（`Y` 为余特征格）的子商，本结构层采用的当前约定公式为：
  `ker_F2(I + theta_Y) / red_2 ker_Z(I + theta_Y)`。
- 【注释声明】抽象商 `Y^theta / (I + theta_Y)Y` 与之同构，但“自然坐标不同”；实现选择的是子商坐标约定。本包不对该同构断言做数学验收。
- 【注释声明】层定位：伴随映射在 `crate::AdjointCartanFiber` / `crate::FiberToAdjoint`，grading 在 `crate::CartanGradingData`，弱实形划分在 `crate::WeakRealFormPartition`，强实层在 `crate::StrongRealClassification`；KGB 数据“仍属于后续层”。
- 【已实现】运行时表示：`CartanFiber` 仅含 `Arc<CartanFiberModel>`；model 内含 `involution: LatticeInvolution` 与 `quotient: ModTwoSubquotient`。克隆 fiber 即克隆 `Arc`（共享 model）。
- 【已实现】`CartanFiberElement` 含同款 `Arc<CartanFiberModel>` 与 `coordinates: ModTwoVector`；`PartialEq` 要求 `Arc::ptr_eq` 且坐标相等——即来自不同 `build` 调用的元素即使坐标相同也不相等（测试 6 锚定）。【注释声明】“opaque 意味着 provenance 而非保密”，坐标视图仅由所属 fiber 按其约定基提供。

# 4. `CartanFiber`：构造门控、预算语义与顺序保证

- 【已实现】`build` 仅克隆 involution 后委托 `build_owned`。
- 【已实现】`build_owned` 的计算顺序固定：
  1. 先算整系数分母：`negative_coweight_eigenspace(involution.coweight_matrix(), budget)?`，再 `reduce_basis_mod_two(&negative_eigenspace)?`；
  2. 再算有限域分子：`mod_two_i_plus_coweight_kernel(&involution)?`；
  3. `ModTwoSubquotient::new(numerator, denominator)?`。
  【注释声明】先算分母是为了“在分配有限域分子之前执行 caller-owned 资源预算”。
- 【已实现】预算语义：`IntegerLatticeBudget` 只传入 `negative_coweight_eigenspace`；有限域侧（`ModTwoSubspace::new`、`ModTwoVector::from_ones`、`equations.insert`、`right_kernel`）不接收该预算，而是以 `try_reserve_exact`/`checked_add` 等分配防护报错（`AllocationFailed { requested }` / `ArithmeticOverflow`）。测试 9 锚定：rank 2 恒等对合在 `IntegerLatticeBudget::new(1, 4, 8, 8)` 下返回 `IntegerLatticeResourceLimit { resource: "rank", limit: 1 }`。【推断】`IntegerLatticeBudget::new` 四个位置参数各自的语义未见于本包字节。
- 【已实现】`mod_two_i_plus_coweight_kernel` 的边界：
  - 任一行长度 `!= rank` → `StructureError::InvalidInvolution`；
  - 每行容量 `rank.checked_add(1).ok_or(ArithmeticOverflow)?`；
  - `ones.try_reserve_exact(capacity)` 失败 → `AllocationFailed { requested: capacity }`；
  - 每行收集奇数条目的列索引后，**追加 `row_index` 自身**；【注释声明】因 `from_ones` 对重复索引做 xor-toggle，这实现对角 `+ I` 项（即便 theta 对角已为奇数）；
  - 逐行 `equations.insert(...)?`，最终 `equations.right_kernel()`。
- 【已实现】numerator 方程使用 `involution.coweight_matrix()` 的**行**，不做转置（测试 5 锚定：非对称例中 `ker_F2(I + theta_Y)` 为 `<e0, e2>`，`red_2 ker_Z` 为 `<e0>`，`dimension()==1`，`basis_representatives()==[ambient(3,[2])]`，`ambient(3,[1])` 被拒）。

# 5. `CartanFiber`：元素运算、provenance 语义与错误分支

- 【已实现】`lattice_rank()` = `quotient.ambient_dimension()`；`dimension()` = `quotient.dimension()`（F2 维数）。【注释声明】`dimension` “从不枚举 `2^dimension` 个元素”。
- 【已实现】`identity()` 返回 `Result`：内部 `ModTwoVector::zero(self.dimension())?` 可失败（具体变体不在本包字节内，【推断】为分配类错误）。
- 【已实现】`element_from_ambient`：仅做 `quotient.to_coordinates(representative)?`。错误锚点：向量不在 numerator → `StructureError::NotInModTwoSubspace`（测试 3、5）；维度不符 → `StructureError::RankMismatch { expected, actual }`（测试 8：`expected: 1, actual: 2`）。
- 【已实现】`element_from_coweight_mod_two` 顺序：先 `coweight.rank() != self.lattice_rank()` → `RankMismatch { expected, actual }`；再 `try_reserve_exact(coweight.rank())` → `AllocationFailed { requested: coweight.rank() }`；收集奇数坐标索引后经 `ModTwoVector::from_ones` 委托 `element_from_ambient`。【注释声明】不断言该余特征恰为 theta-fixed；“打包的 numerator 即 mod-two 条件”。测试 8 锚定“rank 检查先于分配”。
- 【已实现】`canonical_representative` / `coordinates` / `add` / `same_class` 均先 `ensure_member`：`Arc::ptr_eq` 失败 → `StructureError::CartanFiberMismatch`（测试 6：两次分别 `build` 的恒等 fiber 元素 `add` 被拒）。
- 【已实现】`add` = 克隆左坐标后 `xor_assign(&right.coordinates)?`（错误可传播）；`same_class` = 坐标相等比较；`coordinates` 返回元素内坐标的借用 `&'e ModTwoVector`。
- 【注释声明】`canonical_representative` 是“documented low-pivot reduction convention”下的确定性 ambient 代表；`coordinates` 中第 `j` 位选择 `basis_representatives()[j]`，所选代表之 XOR 恰为 `canonical_representative`（测试 1 间接锚定 XOR 关系：`add` 后代表为 `ambient(2,[0,1])`）。
- 【已实现】`basis_representatives()` 直接返回 `quotient.basis_representatives()` 的切片。
- 【已实现】`pub(crate) validate_induced_map` 委托 `quotient.validate_induced_map_to(&target.model.quotient, ambient_map)?`。错误锚点（测试 7）：identity→swap 的恒等映射返回 `CartanFiberMapDoesNotDescend { relation: "numerator" }`；反向返回 `relation: "denominator" }`。【注释声明】高层可在证明后按需应用该映射，避免缓存商坐标稠密映射。

# 6. `AdjointCartanFiber`：表示

- 【已实现】字段三元组：`source: CartanFiber`（构造时传入并克隆的 ambient fiber）、`projection: AdjointProjection`、`fiber: CartanFiber`（伴随侧 fiber 本体）。
- 【注释声明】`ambient_fiber()` 的语义：只有由这个精确值铸造的元素才被下降证明覆盖；“分别构建的值相等 fiber 无法表达这一点”。
- 【已实现】`AdjointFiberElement(CartanFiberElement)` 为私有字段元组包装；`PartialEq/Eq` 派生（即沿用内部元素的 ptr_eq 语义）。
- 【已实现】`dimension`、`identity`、`element_from_ambient`、`coordinates`、`canonical_representative`、`add`、`same_class`、`basis_representatives` 全部一行委托给 `self.fiber` 的同名方法（`element_from_coweight_mod_two` 例外，见第 8 节）。
- 【已实现】`AdjointBasedRootDatum` 由 `BasedRootDatum::standard(clone_i32_matrix(source.cartan_matrix())?)?` 构造；`rank()` = `datum.lattice_rank()`。【注释声明】其 character 基是源 datum 的完整 simple-root 基，cocharacter 基为对应 fundamental-coweight 基。

# 7. `AdjointCartanFiber`：构造门控与预算语义

## 7.1 `AdjointFiberBudget`

- 【已实现】三个组成：`integer_lattice: IntegerLatticeBudget`、`max_persistent_entries: usize`、`max_projection_operations: usize`；`new` 为 `const fn`；公开 getter 仅 `integer_lattice_budget()`。
- 【注释声明】这些界限“约束一次计算，不是数学秩上限”；整系数约化单列预算“因为它拥有 Malachite 中间量”；另两项分别覆盖“保留的结构坐标”与“用于证明商下降的直接限制工作”。

## 7.2 `build` 的门控与顺序【已实现】

1. `root_system.datum() != root_involution.involution().datum()` → `StructureError::DatumMismatch`；
2. `source.involution() != root_involution.involution()` → `StructureError::CartanFiberInvolutionMismatch`（测试 4 锚定）；
3. `validate_adjoint_build_budget(root_system.datum(), budget)?`（在任何目标矩阵分配之前的预检；测试 8、9 锚定）；
4. `AdjointProjection::from_source(root_system.datum(), budget.max_projection_operations)?`；
5. `root_basis_action(root_system, root_involution)?`；
6. `coweight_action = transpose_square(&root_action)?`；
7. `LatticeInvolution::new(projection.target_datum().as_based_root_datum(), root_action, coweight_action)?`（weight 用 root_action，coweight 用其转置）；
8. `CartanFiber::build_owned(adjoint_involution, budget.integer_lattice_budget())?`（`pub(crate)` 复用）；
9. `source.validate_induced_map(&fiber, &projection)?`（`pub(crate)` 复用，证明投影沿两个关系下降）；
10. 返回 `Self { source: source.clone(), projection, fiber }`。

【注释声明】第 6 步注释：dual action 本应为 inverse-transpose，但 `RootInvolutionData` 已验证 root action 是对合，故 inverse-transpose 即转置。本包不对该论证做数学验收。

## 7.3 `validate_adjoint_build_budget` 公式与顺序【已实现】

记 `r = source.semisimple_rank()`、`n = source.lattice_rank()`：

1. `r > budget.integer_lattice.max_rank()` → `AdjointFiberResourceLimit { resource: "semisimple rank", limit: max_rank }`；
2. `square = r*r`；`retained = square * ADJOINT_PERSISTENT_SQUARES`（=16）；`copied = r*n`；`persistent = retained + copied`；`persistent > max_persistent_entries` → `AdjointFiberResourceLimit { resource: "persistent entries", limit }`；
3. `descent = (n*n) * r * 2`；`descent > max_projection_operations` → `AdjointFiberResourceLimit { resource: "projection operations", limit }`。

所有乘法/加法经 `checked_product`/`checked_sum`，溢出 → `ArithmeticOverflow`。【注释声明】第 3 步依据：“每个源坐标至多一个 numerator 与一个 denominator 基向量；每次直接投影为每个 adjoint simple root 检查所有源坐标”。【推断】`ADJOINT_PERSISTENT_SQUARES = 16` 的具体构成（哪些 16 个方阵）未在注释中逐项列出。

## 7.4 `root_basis_action` 边界【已实现】

- `rank = root_system.datum().semisimple_rank()`；初始 `zero_square_matrix(rank)?`；
- 对每个 simple root（列）：`root_system.id_of(simple_root)` / `root_involution.image(root)` / `root_system.simple_coordinates(image)` 任一返回 `None` → `StructureError::InvalidRootAutomorphism`；
- `coordinates.len() != rank` → `RankMismatch { expected: rank, actual }`；
- 逐坐标写入 `action[row][column]`。
- 【推断】若 `simple_roots()` 数量超过 `rank`，`action[..][column]` 的直接索引无显式防御（潜在越界 panic 点）；非测试代码全文未见 `panic!`/`assert!`/`unwrap`。

## 7.5 运行时逐次预算：`check_projection_work`【已实现】

- `per_vector = source_lattice_rank * target_rank`（`checked_product`）；`operations = per_vector * vector_count`；`operations > max_projection_operations` → `AdjointFiberResourceLimit { resource: "projection operations", limit }`。
- `map_coweight` 与 `apply_mod_two` 各以 `vector_count = 1` 调用，且都在分配输出之前。

# 8. 两者关系：伴随 fiber 如何在 Cartan fiber 上构造

- 【已实现】`AdjointCartanFiber` 不重新定义子商运算；它通过 `pub(crate)` 的 `CartanFiber::build_owned` 直接构建伴随侧 `CartanFiber`，并通过 `pub(crate)` 的 `CartanFiber::validate_induced_map` 在构造时完成一次性下降证明。伴随对合的 weight/coweight 矩阵来自源 root 作用及其转置（测试 3 锚定：`weight_matrix == [[-1,1],[0,1]]`、`coweight_matrix == [[-1,0],[1,1]]`，验证“不复用 root 行”）。
- 【已实现】`element_from_coweight_mod_two(&AdjointCoweight)` 先做 `Arc::ptr_eq(&self.projection.model, &coweight.model)` 检查，失败 → `DatumMismatch`；通过后委托 `self.fiber.element_from_coweight_mod_two(&coweight.coordinates)`。
- 【已实现】`fiber_map()` 返回 `FiberToAdjoint { source, target, projection }`（三者克隆；`Arc` 共享 model）。`FiberToAdjoint` 无公开构造函数，字段全私有，唯一来源是 `fiber_map()`。
- 【已实现】`FiberToAdjoint::apply` 三步固定顺序：
  1. `self.source.canonical_representative(source)?`（内部 `ensure_member` 强制元素来自构造时的 source model，否则传播 `CartanFiberMismatch`）；
  2. `self.projection.apply_mod_two(&representative)?`；
  3. `self.target.element_from_ambient(projected).map(AdjointFiberElement)`。
  【注释声明】该映射“不保留 dense mod-two 矩阵或缓存的商坐标像”，每次应用按需投影确定性的源代表。
- 【已实现】`AdjointProjection` 实现 `ModTwoAmbientMap`（`source_dimension`/`target_dimension`/`apply`），因此可作为 `&impl ModTwoAmbientMap` 传给 `CartanFiber::validate_induced_map`——这是两个文件之间的 trait 连接点。
- 【注释声明】`AdjointProjection` 是“从 ambient 余特征格到伴随格的限制映射”：`y` 的目标坐标 = 与源 simple roots 的配对；该映射“可以有中心核，且不被呈现为同构”。

# 9. `AdjointProjection` 与 Coweight 绑定语义

- 【已实现】`from_source` 记录 `source_lattice_rank`、克隆 `source.simple_roots()`（`clone_weights`，逐元素 `try_reserve_exact`）、构造 `target_datum`、保存 `max_projection_operations`。
- 【已实现】`source_coweight(coordinates)`：`coordinates.rank() != source_lattice_rank` → `RankMismatch { expected, actual }`；否则返回绑定了本 projection model 的 `AmbientCoweight`。【注释声明】调用者在此“原始坐标边界”提供 datum 断言；后续操作保留并验证该绑定。
- 【已实现】`map_coweight`：`ensure_source`（`Arc::ptr_eq`，否则 `DatumMismatch`，测试 5 锚定）→ `check_projection_work(1)?` → `try_reserve_exact(target_rank)`（`AllocationFailed { requested: target_rank }`）→ 对每个 simple root 调 `pair(root, &source.coordinates)?` 收集坐标 → `Coweight::new`。
- 【已实现】`apply_mod_two`：维度不符 → `RankMismatch`；逐目标坐标计算奇偶：`coefficient % 2 != 0 && source.bit(source_coordinate) == Some(true)` 时翻转 `parity`（`bit` 返回 `Option`，越界为 `None` 而非 panic）→ `ModTwoVector::from_ones(target_rank, ones)`。
- 【已实现】`AmbientCoweight`/`AdjointCoweight` 的 `PartialEq` 均为 `Arc::ptr_eq && coordinates`；`AdjointCoweight::datum()` 返回 `&AdjointBasedRootDatum`。
- 【推断】`from_source` 不校验 `simple_roots().len()` 与目标 rank 的一致；`map_coweight` 的 `try_reserve_exact(target_rank)` 仅为预留，`push` 次数实为 `source_simple_roots.len()`。

# 10. 特征空间 / 基的计算路径汇总

- 【已实现】Cartan 侧分母：`negative_coweight_eigenspace(involution.coweight_matrix(), budget)` 得整系数负特征空间基 → `reduce_basis_mod_two` 得 `red_2 ker_Z(I + theta_Y)`（两函数字节不在本包内；测试 4 注释锚定：`red_2 ker_Z` 与“raw mod-two image of `I+theta_Y`”不同，“后者会给出错误的 rank”）。
- 【已实现】Cartan 侧分子：`mod_two_i_plus_coweight_kernel` 以 coweight 矩阵行的奇数条目加对角 toggle 构造 F2 方程组，`right_kernel()` 得 `ker_F2(I + theta_Y)`。
- 【已实现】商与基：`ModTwoSubquotient::new(numerator, denominator)`；`basis_representatives()`、`canonical_representative()`（low-pivot 约定）、`coordinates()` 均委托 `ModTwoSubquotient`（其内部字节不在本包内）。
- 【已实现】伴随侧复用同一路径：伴随 `LatticeInvolution` 建成后由 `CartanFiber::build_owned` 走相同的“先分母后分子”流程，预算来自 `budget.integer_lattice_budget()`。

# 11. 测试锚点（精确断言）

## 11.1 `cartan_fiber.rs`

| 测试 | 关键断言 |
|---|---|
| `identity_involution_keeps_the_full_mod_two_torus` | rank-2 中心环面恒等：`dimension()==2`；`basis_representatives()==[e0,e1]`；`add(e0,e1)` 的 `canonical_representative()==ambient(2,[0,1])`；`e0` 非 identity |
| `negative_identity_has_a_trivial_fiber` | `-I`（rank 1）：`dimension()==0`；`ambient(1,[0])` 是 identity；其代表为 `ambient(1,[])` |
| `swap_involution_has_no_component_group` | swap（rank 2）：`dimension()==0`；`ambient(2,[0,1])` 是 identity；`ambient(2,[0])` → `Err(NotInModTwoSubspace)` |
| `uses_the_stored_coweight_action_and_negative_eigenlattice` | 非对称例（weight `[[1,0],[2,-1]]`、coweight `[[1,2],[0,-1]]`）：`dimension()==1`；`element_from_ambient(e0) == element_from_coweight_mod_two([0,1])`；两者 `add` 为 identity；`basis==[e1]` |
| `uses_coweight_rows_for_the_mod_two_numerator_without_transposing` | rank-3 非对称：`dimension()==1`；`e0` 为 identity（分母）；`e2` 非 identity；`basis==[e2]`；`e1` → `NotInModTwoSubspace` |
| `rejects_elements_from_a_different_fiber` | 两次 `build` 的恒等 fiber：`add` → `Err(CartanFiberMismatch)` |
| `rejects_ambient_maps_that_do_not_descend_through_both_relations` | identity→swap：`CartanFiberMapDoesNotDescend { relation: "numerator" }`；swap→identity：`relation: "denominator"` |
| `validates_coweight_rank_before_allocating_its_mod_two_image` | `element_from_coweight_mod_two([0,0])` → `RankMismatch { expected: 1, actual: 2 }`；`element_from_ambient(ambient(2,[0]))` 同 |
| `respects_the_caller_owned_exact_lattice_budget` | `IntegerLatticeBudget::new(1,4,8,8)` 下 `build` → `IntegerLatticeResourceLimit { resource: "rank", limit: 1 }` |

## 11.2 `adjoint_fiber.rs`

| 测试 | 关键断言 |
|---|---|
| `identity_a1_projects_the_fiber_generator` | A1 恒等：`target.datum().rank()==1`、`target.dimension()==1`；`fiber_map().apply(source_gen) == element_from_coweight_mod_two(map_coweight(source_coweight([1])))`；代表为 `ambient(1,[0])` |
| `central_coweights_map_to_the_adjoint_fiber_kernel` | rank-2（一个 simple root + 中心方向）：`source.dimension()==2`、`target.dimension()==1`；root 方向映到 `ambient(1,[0])`；中心方向映到 identity；`map(a+b) == map(a)+map(b)` |
| `derives_the_adjoint_coweight_action_instead_of_reusing_root_rows` | A2 twisted（`simple_reflection(0)`）：`fiber.involution().weight_matrix()==[[-1,1],[0,1]]`、`coweight_matrix()==[[-1,0],[1,1]]` |
| `requires_the_source_fiber_for_the_actual_involution_at_construction` | source 用 `-I` 构建、root_involution 为恒等 → `Err(CartanFiberInvolutionMismatch)` |
| `rejects_source_coordinates_bound_to_another_projection` | 跨 projection 复用 `AmbientCoweight` → `Err(DatumMismatch)` |
| `projects_a_nonsymmetric_actual_action_and_intertwines_the_dual_actions` | A1+A2 twisted（`simple_reflection(1)`）：`source.dimension()==1`、`target.dimension()==1`；weight/coweight 矩阵断言；对每个坐标基向量验证 `projection(theta(x)) == theta_adjoint(projection(x))`；`fiber_map` 应用于 `ambient(3,[0])` 后代表为 `ambient(3,[0])` |
| `supports_dynamic_adjoint_rank_above_the_legacy_packed_limit` | rank 33 对角 Cartan：`target.datum().rank()==33`、`dimension()==33`；`fiber_map` 应用于 `ambient(33,[32])` 后代表为 `ambient(33,[32])` |
| `rejects_adjoint_storage_before_allocating_target_matrices` | `AdjointFiberBudget::new(.., 1, 100)` → `AdjointFiberResourceLimit { resource: "persistent entries", limit: 1 }` |
| `rejects_insufficient_projection_work_before_building_the_target` | `AdjointFiberBudget::new(.., 100, 0)` → `AdjointFiberResourceLimit { resource: "projection operations", limit: 0 }` |

# 12. 错误类型与边界条件汇总（本包字节中出现的 `StructureError` 变体）

| 变体 | 出现位置（触发条件） |
|---|---|
| `RankMismatch { expected, actual }` | `element_from_coweight_mod_two`（coweight rank 不符）；`element_from_ambient`/`to_coordinates`（维度不符）；`AdjointProjection::source_coweight`、`apply_mod_two`；`root_basis_action`（坐标长度 ≠ semisimple rank） |
| `AllocationFailed { requested }` | 各处 `try_reserve_exact`（kernel 构造、coweight 收集、克隆辅助、输出分配）；【推断】`ModTwoVector::zero`/`from_ones` 内部失败亦经此路径传播（其实现字节不在本包内） |
| `ArithmeticOverflow` | `checked_add`/`checked_mul` 溢出（kernel 容量、预算公式、`check_projection_work`） |
| `InvalidInvolution` | kernel 构造中行长 ≠ rank；`transpose_square` 非方阵 |
| `CartanFiberMismatch` | `ensure_member`（`Arc::ptr_eq` 失败），传播至 `canonical_representative`/`coordinates`/`add`/`same_class`/`FiberToAdjoint::apply` |
| `NotInModTwoSubspace` | `element_from_ambient` 代表不在 numerator |
| `CartanFiberMapDoesNotDescend { relation }` | `validate_induced_map`；测试锚定 `relation` 取 `"numerator"` / `"denominator"` |
| `IntegerLatticeResourceLimit { resource, limit }` | 整系数预算（测试锚定 `resource: "rank"`） |
| `AdjointFiberResourceLimit { resource, limit }` | `"semisimple rank"` / `"persistent entries"` / `"projection operations"`（含 `check_projection_work` 逐次检查） |
| `DatumMismatch` | `build` 门控 1；`ensure_source`；`element_from_coweight_mod_two` 的 projection model 检查 |
| `CartanFiberInvolutionMismatch` | `build` 门控 2 |
| `InvalidRootAutomorphism` | `root_basis_action` 中 `id_of`/`image`/`simple_coordinates` 返回 `None` |

- 【已实现】两个文件非测试代码中无 `panic!`、`assert!`、`unwrap`；潜在的直接索引点见 §7.4 的【推断】。
- 【已实现】`PartialEq` 语义统一为“`Arc::ptr_eq` + 坐标相等”（`CartanFiberElement`、`AmbientCoweight`、`AdjointCoweight`）；`AdjointFiberElement` 经派生沿用内部元素语义。

# 13. 限制与未覆盖面

1. 【注释声明】KGB 数据不属于本层；grading / weak-real / strong-real 分别在 `CartanGradingData` / `WeakRealFormPartition` / `StrongRealClassification`，其字节不在本包内。
2. 【注释声明】`Y^theta / (I+theta_Y)Y` 与所实现子商“同构但坐标不同”；实现仅承诺子商/low-pivot 坐标约定。本包不对同构断言做数学验收。
3. 【已实现】`element_from_coweight_mod_two` 不验证 theta-fixed（注释明示）；`dimension()` 不枚举元素（注释明示）。
4. 【已实现】`CartanFiber` 无 `PartialEq`；跨 `build` 的 fiber 无法比较，其元素必被 `CartanFiberMismatch` 拒绝（测试锚定）。
5. 【已实现】`build` 的第一个门控（datum 不一致 → `DatumMismatch`）无专门测试锚点；其余门控均有锚点。
6. 【推断】`ADJOINT_PERSISTENT_SQUARES = 16` 的构成未文档化；预算公式为结构性上界估计，本包不做精确性声明。
7. 【推断】`AdjointProjection::from_source` 未校验 simple-root 数量与伴随 rank 一致；`map_coweight`/`apply_mod_two` 的输出长度依赖该隐含一致。
8. 【已实现】`FiberToAdjoint` 每次 `apply` 重新执行“代表→投影→解释”三步（注释明示不缓存）；`apply` 的错误沿三步传播。
9. 【推断】`RootSystem::enumerate(&datum, n)` 第二参数（测试取 2/6/8/66）语义未见于本包字节；`supports_dynamic_adjoint_rank_above_the_legacy_packed_limit` 中“legacy packed limit”仅见于测试名，本包不据此作历史断言。
10. 外部符号（`StructureError` 定义、`ModTwoSubquotient`/`ModTwoSubspace`/`ModTwoVector` 内部、`negative_coweight_eigenspace`、`reduce_basis_mod_two`、`IntegerLatticeBudget`、`LatticeInvolution`、`BasedRootDatum`、`RootSystem`、`RootInvolutionData`、`TwistedInvolution`、`WeylGroup`、`pair`、`Weight`/`Coweight`）的行为只能经调用点与测试断言引用，需对照其各自源文件另行核对。

# 14. 维护者核对清单（建议）

- [ ] 核对 `StructureError` 各变体定义与本包列出的构造点一致（字段名 `expected/actual/requested/resource/limit/relation`）。
- [ ] 核对 `ModTwoSubquotient::{new, to_coordinates, ambient_representative, basis_representatives, validate_induced_map_to, ambient_dimension, dimension}` 与 `ModTwoSubspace::{new, insert, right_kernel}` 的实际签名/错误变体。
- [ ] 核对 `IntegerLatticeBudget::new` 四参数语义与 `max_rank()`，以及测试所用 `(8,128,1_000,128)`、`(64,100_000,100_000,128)`、`(1,4,8,8)` 的含义。
- [ ] 核对 `negative_coweight_eigenspace` / `reduce_basis_mod_two` 的预算执行与 `IntegerLatticeResourceLimit` 的产生点。
- [ ] 核对 `CartanFiber` 文档中的子商公式与 low-pivot 约定描述是否应与上游文档链接（本包不含链接目标）。
- [ ] 核对 `ADJOINT_PERSISTENT_SQUARES = 16` 是否需要逐项注释。
- [ ] 核对 `build` 门控 1（`DatumMismatch`）是否需补测试。
- [ ] 核对 `AdjointProjection` 对 simple-root 数量与目标 rank 一致性的隐含依赖是否需显式防御。
```
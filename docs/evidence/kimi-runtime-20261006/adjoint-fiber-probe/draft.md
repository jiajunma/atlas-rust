---
title: 伴随 Cartan 纤维层：构建、投影与 mod-2 商（adjoint_fiber.rs）
source: atlas-rust/adjoint-fiber
ingestedAt: 2026-10-06T00:00:00Z
---

# 伴随 Cartan 纤维层（`crates/atlas-real-group/src/adjoint_fiber.rs`）

> 本草稿仅依据所给源文件字节整理。文中以【已实现】标注代码中可直接读出的事实，以【推断】标注阅读推断（含文档注释断言）；不对任何数学正确性、性能或完备性作验收声明。

## 1. 裸签名清单（含无注释项）

本文件不定义任何 `trait` 或 `enum`；包含一个对外部 trait 的 impl。以下按出现顺序列出全部条目；标注「私有」者为模块内可见项。

```text
// 常量（私有）
const ADJOINT_PERSISTENT_SQUARES: usize = 16;

// 预算
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdjointFiberBudget {
    integer_lattice: IntegerLatticeBudget,        // 私有字段
    max_persistent_entries: usize,                // 私有字段
    max_projection_operations: usize,             // 私有字段
}
impl AdjointFiberBudget {
    pub const fn new(
        integer_lattice: IntegerLatticeBudget,
        max_persistent_entries: usize,
        max_projection_operations: usize,
    ) -> Self;
    pub fn integer_lattice_budget(&self) -> &IntegerLatticeBudget;
    // 注：max_persistent_entries / max_projection_operations 无公开 getter
}

// 伴随商根系数据
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdjointBasedRootDatum { datum: BasedRootDatum }   // 私有字段
impl AdjointBasedRootDatum {
    fn from_source(source: &BasedRootDatum) -> Result<Self, StructureError>; // 私有
    pub fn as_based_root_datum(&self) -> &BasedRootDatum;
    pub fn rank(&self) -> usize;                            // 返回 datum.lattice_rank()
}

// 投影共享模型（私有结构）
#[derive(Debug)]
struct AdjointProjectionModel {
    source_lattice_rank: usize,
    source_simple_roots: Vec<Weight>,
    target_datum: AdjointBasedRootDatum,
    max_projection_operations: usize,
}

// 绑定到源格子的余权
#[derive(Clone, Debug)]
pub struct AmbientCoweight {
    model: Arc<AdjointProjectionModel>,           // 私有字段
    coordinates: Coweight,                        // 私有字段
}
impl AmbientCoweight {
    pub fn as_coweight(&self) -> &Coweight;
}
impl PartialEq for AmbientCoweight;  // 手写：Arc::ptr_eq(model) && coordinates 相等
impl Eq for AmbientCoweight;

// 伴随基本余权格中的余权
#[derive(Clone, Debug)]
pub struct AdjointCoweight {
    model: Arc<AdjointProjectionModel>,           // 私有字段
    coordinates: Coweight,                        // 私有字段
}
impl AdjointCoweight {
    pub fn datum(&self) -> &AdjointBasedRootDatum;
    pub fn as_coweight(&self) -> &Coweight;
}
impl PartialEq for AdjointCoweight;  // 手写：Arc::ptr_eq(model) && coordinates 相等
impl Eq for AdjointCoweight;

// 限制映射 Y -> P^vee
#[derive(Clone, Debug)]
pub struct AdjointProjection {
    model: Arc<AdjointProjectionModel>,           // 私有字段
}
impl AdjointProjection {
    fn from_source(
        source: &BasedRootDatum,
        max_projection_operations: usize,
    ) -> Result<Self, StructureError>;                               // 私有
    pub fn source_lattice_rank(&self) -> usize;
    pub fn target_datum(&self) -> &AdjointBasedRootDatum;
    pub fn source_coweight(&self, coordinates: Coweight)
        -> Result<AmbientCoweight, StructureError>;
    pub fn map_coweight(&self, source: &AmbientCoweight)
        -> Result<AdjointCoweight, StructureError>;
    fn ensure_source(&self, source: &AmbientCoweight) -> Result<(), StructureError>;   // 私有
    fn apply_mod_two(&self, source: &ModTwoVector) -> Result<ModTwoVector, StructureError>; // 私有
    fn check_projection_work(&self, vector_count: usize) -> Result<(), StructureError>;     // 私有
}
impl ModTwoAmbientMap for AdjointProjection {
    fn source_dimension(&self) -> usize;   // = source_lattice_rank()
    fn target_dimension(&self) -> usize;   // = target_datum().rank()
    fn apply(&self, source: &ModTwoVector) -> Result<ModTwoVector, StructureError>; // 转调 apply_mod_two
}

// 伴随 Cartan 纤维与元素
#[derive(Clone, Debug)]
pub struct AdjointCartanFiber {
    source: CartanFiber,           // 私有字段
    projection: AdjointProjection, // 私有字段
    fiber: CartanFiber,            // 私有字段
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdjointFiberElement(CartanFiberElement); // 私有元组字段
impl AdjointFiberElement {
    pub fn is_identity(&self) -> bool;
}

// 已证明的商兼容映射
#[derive(Clone, Debug)]
pub struct FiberToAdjoint {
    source: CartanFiber,           // 私有字段
    target: CartanFiber,           // 私有字段
    projection: AdjointProjection, // 私有字段
}
impl FiberToAdjoint {
    pub fn apply(&self, source: &CartanFiberElement)
        -> Result<AdjointFiberElement, StructureError>;
}

impl AdjointCartanFiber {
    pub fn build(
        root_system: &RootSystem,
        root_involution: &RootInvolutionData,
        source: &CartanFiber,
        budget: &AdjointFiberBudget,
    ) -> Result<Self, StructureError>;
    pub fn datum(&self) -> &AdjointBasedRootDatum;
    pub fn ambient_fiber(&self) -> &CartanFiber;
    pub fn projection(&self) -> &AdjointProjection;
    pub fn dimension(&self) -> usize;
    pub fn identity(&self) -> Result<AdjointFiberElement, StructureError>;
    pub fn element_from_ambient(&self, representative: ModTwoVector)
        -> Result<AdjointFiberElement, StructureError>;
    pub fn element_from_coweight_mod_two(&self, coweight: &AdjointCoweight)
        -> Result<AdjointFiberElement, StructureError>;
    pub fn coordinates<'e>(&self, element: &'e AdjointFiberElement)
        -> Result<&'e ModTwoVector, StructureError>;
    pub fn canonical_representative(&self, element: &AdjointFiberElement)
        -> Result<ModTwoVector, StructureError>;
    pub fn add(&self, left: &AdjointFiberElement, right: &AdjointFiberElement)
        -> Result<AdjointFiberElement, StructureError>;
    pub fn same_class(&self, left: &AdjointFiberElement, right: &AdjointFiberElement)
        -> Result<bool, StructureError>;
    pub fn basis_representatives(&self) -> &[ModTwoVector];
    pub fn fiber_map(&self) -> FiberToAdjoint;
}

// 私有自由函数
fn validate_adjoint_build_budget(source: &BasedRootDatum, budget: &AdjointFiberBudget)
    -> Result<(), StructureError>;
fn root_basis_action(root_system: &RootSystem, root_involution: &RootInvolutionData)
    -> Result<Vec<Vec<i32>>, StructureError>;
fn clone_i32_matrix(matrix: &[Vec<i32>]) -> Result<Vec<Vec<i32>>, StructureError>;
fn clone_weights(weights: &[Weight]) -> Result<Vec<Weight>, StructureError>;
fn zero_square_matrix(rank: usize) -> Result<Vec<Vec<i32>>, StructureError>;
fn transpose_square(matrix: &[Vec<i32>]) -> Result<Vec<Vec<i32>>, StructureError>;
fn checked_product(left: usize, right: usize) -> Result<usize, StructureError>;
fn checked_sum(left: usize, right: usize) -> Result<usize, StructureError>;

// 测试模块
#[cfg(test)] mod tests { /* 9 个 #[test]，见 §6 */ }
```

## 2. 职责与数据结构

- 【推断，基于文档注释与命名】本模块负责：给定一个已验证的 root-datum involution（`RootInvolutionData`）与已建好的 ambient `CartanFiber`，在伴随半单商（adjoint semisimple quotient）上构造对应的有限 `F_2` Cartan 纤维（`AdjointCartanFiber`），并提供从源纤维到伴随纤维的映射（`FiberToAdjoint`）。
- 【已实现】`AdjointBasedRootDatum` 包装一个 `BasedRootDatum`；其构造仅使用源 datum 的 Cartan 矩阵：`BasedRootDatum::standard(clone_i32_matrix(source.cartan_matrix())?)?`。文档注释称：其 character 基是源 datum 的完整单根基，cocharacter 基是对应的基本余权基【注释断言，本文件内未另证】。
- 【已实现】出处绑定（provenance binding）模型：`AmbientCoweight` 与 `AdjointCoweight` 各自持有 `Arc<AdjointProjectionModel>` 加裸 `Coweight` 坐标。两者手写的 `PartialEq` 要求 `Arc::ptr_eq(&self.model, &other.model)` 且坐标相等。文档注释说明其意图：裸 `Coweight` 无 datum 出处，绑定记录调用方断言，并阻止跨投影复用【意图为注释；机制为代码事实】。
- 【已实现】`AdjointCartanFiber` 持有三份私有状态：源纤维克隆（`source: CartanFiber`）、投影（`projection`）、目标纤维（`fiber: CartanFiber`）。`fiber_map()` 每次调用再次克隆这三者构造 `FiberToAdjoint`。
- 【已实现】`AdjointFiberElement` 是 `CartanFiberElement` 的不透明 newtype；公开操作仅有 `is_identity()`，其余运算（`add`、`coordinates`、`canonical_representative`、`same_class`）由 `AdjointCartanFiber` 委托给内部 `fiber`。
- 【已实现】`FiberToAdjoint` 文档注释明确：不保存稠密 mod-2 矩阵、不缓存商坐标像；构造器已完成下降（descent）证明，故每次 `apply` 可对确定性的源代表元现算投影。

## 3. 构建流程（含投影与 mod-2 商）

`AdjointCartanFiber::build` 的步骤【已实现，按代码顺序】：

1. 前置一致性检查：
   - `root_system.datum() != root_involution.involution().datum()` → `Err(StructureError::DatumMismatch)`；
   - `source.involution() != root_involution.involution()` → `Err(StructureError::CartanFiberInvolutionMismatch)`。
2. `validate_adjoint_build_budget(root_system.datum(), budget)?`（预算细节见 §4）。
3. `AdjointProjection::from_source(root_system.datum(), budget.max_projection_operations)`：
   - 记录 `source_lattice_rank = source.lattice_rank()`；
   - `clone_weights(source.simple_roots())` 逐元素复制（每次 `try_reserve_exact`，失败 → `AllocationFailed`）；
   - 构造 `AdjointBasedRootDatum`（克隆 Cartan 矩阵后 `BasedRootDatum::standard`，错误原样传播）。
4. `root_basis_action(root_system, root_involution)`：以半单秩 `rank` 建 `rank×rank` 零矩阵；对每个单根（作为列 `column`）：
   - `root_system.id_of(simple_root)` 为 `None` → `InvalidRootAutomorphism`；
   - `root_involution.image(root)` 为 `None` → `InvalidRootAutomorphism`；
   - `root_system.simple_coordinates(image)` 为 `None` → `InvalidRootAutomorphism`；
   - `coordinates.len() != rank` → `RankMismatch { expected: rank, actual }`；
   - 逐行写入 `action[row][column]`。
5. `transpose_square(&root_action)` 得到余权作用矩阵；非方阵 → `InvalidInvolution`。代码注释断言：对偶作用本为逆-转置，因 `RootInvolutionData` 已验证根作用是对合，故逆-转置即转置【注释断言；本文件不重新验证对合性】。
6. `LatticeInvolution::new(projection.target_datum().as_based_root_datum(), root_action, coweight_action)?`（错误原样传播，具体变体不可见）。
7. `CartanFiber::build_owned(adjoint_involution, budget.integer_lattice_budget())?`（整数预算在该调用内部生效，细节不可见）。
8. `source.validate_induced_map(&fiber, &projection)?` —— 下降证明钩子；`projection` 以 `ModTwoAmbientMap` 实现者身份传入【推断其用途为验证商下降；函数体不可见】。
9. 返回 `Self { source: source.clone(), projection, fiber }`。

投影的两种形态【已实现】：

- 整数限制映射 `map_coweight`（文档：`Y -> P^vee`）：先 `ensure_source`（`Arc::ptr_eq` 否则 `DatumMismatch`），再 `check_projection_work(1)`，然后按 `target_rank` 预分配坐标向量，对 `source_simple_roots` 中每个根计算 `pair(root, &source.coordinates)?` 并依序 push；结果包成 `Coweight::new(coordinates)`。目标坐标顺序与源 datum 的单根顺序一致【已实现：克隆保序 + 顺序迭代】。
- mod-2 版本 `apply_mod_two`（私有；经 `ModTwoAmbientMap::apply` 或 `FiberToAdjoint` 间接可达）：维度不符 → `RankMismatch { expected: source_lattice_rank, actual: source.dimension() }`；`check_projection_work(1)`；对每个目标坐标（枚举单根得 `target_coordinate`），遍历该根系数：若 `coefficient % 2 != 0` 且 `source.bit(source_coordinate) == Some(true)` 则翻转奇偶位；奇偶为真则把 `target_coordinate` 推入 `ones`；最终 `ModTwoVector::from_ones(target_rank, ones)`。注意【按代码字面】：`source.bit(i)` 返回 `None` 的分支被当作非 1（不翻转）。
- `FiberToAdjoint::apply`：`source.canonical_representative(source_element)?` → `projection.apply_mod_two(&representative)?` → `target.element_from_ambient(projected)?`，包为 `AdjointFiberElement`。
- 文档注释明确：该限制映射可有中心核（central kernel），不作为同构呈现【注释断言；测试 2 提供了核存在的实例，见 §6】。

## 4. 错误分支与预算

`StructureError` 各变体在本文件中的触发点【已实现】：

| 变体 | 触发点 |
|---|---|
| `DatumMismatch` | `build` 中 root system datum ≠ involution datum；`ensure_source` 中 `Arc` 不同源；`element_from_coweight_mod_two` 中 `Arc::ptr_eq` 失败 |
| `CartanFiberInvolutionMismatch` | `build` 中 `source.involution() != root_involution.involution()` |
| `RankMismatch { expected, actual }` | `source_coweight`（坐标秩 ≠ 源格秩）；`apply_mod_two`（向量维度 ≠ 源格秩）；`root_basis_action`（`coordinates.len() != rank`） |
| `AllocationFailed { requested }` | `map_coweight`/`apply_mod_two` 的目标向量预分配；`clone_i32_matrix`/`clone_weights`/`zero_square_matrix` 各处 `try_reserve_exact` 失败 |
| `AdjointFiberResourceLimit { resource, limit }` | 预算三线（见下）；`check_projection_work` 中超 `max_projection_operations`（`resource: "projection operations"`） |
| `InvalidRootAutomorphism` | `root_basis_action` 三处 `Option::None`（`id_of` / `image` / `simple_coordinates`） |
| `InvalidInvolution` | `transpose_square` 检测到有行长度 ≠ 矩阵行数 |
| `ArithmeticOverflow` | `checked_product`/`checked_sum`（`usize::checked_mul`/`checked_add` 溢出） |

预算（`AdjointFiberBudget` 三条线 + 内部整数预算）【已实现】：

- `integer_lattice`：传入 `CartanFiber::build_owned`，并在 `validate_adjoint_build_budget` 中检查 `semisimple_rank > budget.integer_lattice.max_rank()` → `AdjointFiberResourceLimit { resource: "semisimple rank", limit: max_rank }`。
- `max_persistent_entries`：需求量 = `16 * semisimple_rank² + semisimple_rank * lattice_rank`（常数 `ADJOINT_PERSISTENT_SQUARES = 16` 份方阵条目 + 克隆的源单根条目），超限 → `resource: "persistent entries"`。
- `max_projection_operations`：构建期预检下降工作量 = `lattice_rank² * semisimple_rank * 2`，超限 → `resource: "projection operations"`；代码注释解释：每个源坐标至多一个分子与一个分母基向量，每次直接投影对每个伴随单根检查每个源坐标。运行期 `check_projection_work(vector_count)` 按 `lattice_rank * target_rank * vector_count` 计费并与同一上限比较。
- 【已实现】所有预算乘加都经 `checked_product`/`checked_sum`，溢出报 `ArithmeticOverflow`。
- 【已实现】`check_projection_work` 不持有任何计数器状态：每次 `map_coweight`/`apply_mod_two` 调用独立以 `vector_count = 1` 检查，无跨调用累计【推断其语义为「单次调用上限」而非「总量上限」】。

panic/断言面【已实现】：非测试代码无显式 `panic!`/`assert!`；所有堆分配经 `try_reserve_exact` 转为 `AllocationFailed`。`action[row][column] = ...` 与 `transpose[column][row] = ...` 为裸索引写入，其安全性分别由「先建 `rank×rank` 零矩阵 + 校验 `coordinates.len() == rank`」和「`transpose_square` 先拒绝非方阵」保证【推断这两条路径下不会越界；未做穷尽性证明】。

顺序保证【已实现】：`clone_weights` 保序；`map_coweight`/`apply_mod_two` 按存储的单根顺序产生目标坐标。

## 5. 与其它模块的接口（仅调用点可见的事实）

以下仅为本文件调用点暴露的签名/行为线索，未读相应模块源码：

- `crate::mod_two::ModTwoAmbientMap`：含 `source_dimension() -> usize`、`target_dimension() -> usize`、`apply(&ModTwoVector) -> Result<ModTwoVector, StructureError>` 三方法（由本文件的 impl 反推）。
- `crate::pair`：以 `pair(root: &Weight, coweight: &Coweight)` 形式调用，返回 `Result<_, StructureError>`，其 `Ok` 值被 push 进最终传给 `Coweight::new` 的坐标向量。
- `BasedRootDatum`：`standard(Vec<Vec<i32>>) -> Result<_, StructureError>`、`from_simple_data(usize, Vec<Vec<i32>>, Vec<Weight>, Vec<Coweight>) -> Result<_, _>`（测试）、`cartan_matrix()`（结果可作 `&[Vec<i32>]` 切片用）、`lattice_rank()`、`semisimple_rank()`、`simple_roots()`（可切片迭代）；实现 `PartialEq`、`Clone`。
- `RootSystem`：`enumerate(&BasedRootDatum, usize) -> Result<_, _>`（测试）、`datum()`、`id_of(&Weight) -> Option<_>`、`simple_coordinates(_) -> Option<_>`（返回物有 `len()` 与迭代）。
- `RootInvolutionData`：`new(&RootSystem, LatticeInvolution) -> Result<_, _>`（测试）、`involution()`（结果可与 `LatticeInvolution` 做 `!=`）、`image(_) -> Option<_>`。
- `LatticeInvolution`：`new(&BasedRootDatum, Vec<Vec<i32>>, Vec<Vec<i32>>) -> Result<_, StructureError>`、`identity(&BasedRootDatum) -> Result<_, _>`（测试）、`weight_matrix()`、`coweight_matrix()`、`act_on_coweight(&Coweight) -> Result<Coweight, _>`（测试）；实现 `PartialEq`、`Clone`。
- `CartanFiber`：`build(&LatticeInvolution, &IntegerLatticeBudget) -> Result<_, _>`（测试）、`build_owned(LatticeInvolution, &IntegerLatticeBudget) -> Result<_, _>`、`validate_induced_map(&CartanFiber, &projection) -> Result<_, StructureError>`（`projection` 以引用传入，泛型形式不可见）、`involution()`、`dimension()`、`identity()`、`element_from_ambient(ModTwoVector)`、`element_from_coweight_mod_two(&Coweight)`、`coordinates(&CartanFiberElement) -> Result<&ModTwoVector, _>`、`canonical_representative`、`add`、`same_class`、`basis_representatives() -> &[ModTwoVector]`；实现 `Clone`。
- `CartanFiberElement`：`is_identity()`；因 `AdjointFiberElement` 派生 `Eq/PartialEq` 且测试对其断言相等，可知其实现了相应比较【推断】。
- `IntegerLatticeBudget`：`new(usize, usize, usize, usize)`（测试用 `(64, 100_000, 100_000, 128)`，参数名不可见）、`max_rank()`。
- `ModTwoVector`：`from_ones(usize, impl IntoIterator<Item = usize>) -> Result<_, _>`、`dimension()`、`bit(usize) -> Option<bool>`。
- `Weight`：`new(Vec<i32>)`、`rank()`、`as_slice()`（元素为 `i32`）。`Coweight`：`new(Vec<i32>)`、`rank()`。
- 测试专用：`TwistedInvolution::new(&datum, &roots, &involution, simple_reflection)`、`.root_involution()`；`WeylGroup::new(datum).simple_reflection(usize) -> Result<_, _>`。

## 6. 测试锚点（`#[cfg(test)] mod tests`，共 9 个）

公共辅助：`integer_budget() = IntegerLatticeBudget::new(64, 100_000, 100_000, 128)`；`adjoint_budget() = AdjointFiberBudget::new(integer_budget(), 50_000, 100_000)`；`ambient(rank, indices) = ModTwoVector::from_ones(rank, indices).unwrap()`。

1. `identity_a1_projects_the_fiber_generator`：A1（`[[2]]`）+ 恒等对合；断言 `target.datum().rank() == 1`、`target.dimension() == 1`；`fiber_map().apply` 作用于源生成元等于 `element_from_coweight_mod_two(map_coweight(...))`；其 `canonical_representative` 为 `ambient(1, [0])`。
2. `central_coweights_map_to_the_adjoint_fiber_kernel`：`from_simple_data(2, [[2]], [Weight[1,0]], [Coweight[2,0]])`；源维数 2、目标维数 1；根方向映到 `ambient(1,[0])`，中心方向映到 `is_identity()`；并验证 `map(a+b) == map(a)+map(b)`。
3. `derives_the_adjoint_coweight_action_instead_of_reusing_root_rows`：A2 + `TwistedInvolution`（`simple_reflection(0)`）；断言目标 `weight_matrix == [[-1,1],[0,1]]`、`coweight_matrix == [[-1,0],[1,1]]`。
4. `requires_the_source_fiber_for_the_actual_involution_at_construction`：源纤维用不同对合构建 → `Err(CartanFiberInvolutionMismatch)`。
5. `rejects_source_coordinates_bound_to_another_projection`：两次独立 `build` 后用第一个投影绑定的 `AmbientCoweight` 调第二个投影的 `map_coweight` → `Err(DatumMismatch)`。
6. `projects_a_nonsymmetric_actual_action_and_intertwines_the_dual_actions`：A1+A2（分块 Cartan）+ `simple_reflection(1)`；源/目标维数均为 1；`weight_matrix == [[1,0,0],[0,-1,1],[0,0,1]]`、`coweight_matrix == [[1,0,0],[0,-1,0],[0,1,1]]`；对每个坐标基余权验证交织关系 `map(θ·y) == θ_adjoint·map(y)`；`fiber_map` 作用于 `ambient(3,[0])` 的代表元仍为 `ambient(3,[0])`。
7. `supports_dynamic_adjoint_rank_above_the_legacy_packed_limit`：秩 33 的 `2·I` Cartan；目标秩与维数均为 33；第 32 号生成元映到 `ambient(33, [32])`。
8. `rejects_adjoint_storage_before_allocating_target_matrices`：`max_persistent_entries = 1` → `Err(AdjointFiberResourceLimit { resource: "persistent entries", limit: 1 })`。
9. `rejects_insufficient_projection_work_before_building_the_target`：`max_projection_operations = 0` → `Err(AdjointFiberResourceLimit { resource: "projection operations", limit: 0 })`。

## 7. 限制与未覆盖面

- 【已实现】`AdjointFiberBudget` 仅暴露 `integer_lattice_budget()` getter；`max_persistent_entries`、`max_projection_operations` 无公开访问器（本模块内以私有字段读取）。`new` 为 `const fn`，可用于常量上下文。
- 【已实现】`apply_mod_two` 为私有；模块外使用 mod-2 投影只能经 `ModTwoAmbientMap` trait 对象/泛型或 `FiberToAdjoint`。
- 【已实现】`AmbientCoweight` 除 `as_coweight` 外无其它公开操作；`AdjointCoweight` 另有 `datum()`。
- 【已实现】`AdjointCartanFiber` 派生仅 `Clone, Debug`（无 `Eq/PartialEq`）；文档注释强调：分别构建的纤维之间的值相等无法表达出处关系，应使用 `ambient_fiber()` 返回的确切实例。
- 【已实现】未提供纤维元素枚举；`dimension()` 注释明确「从不枚举纤维元素」；`basis_representatives()` 仅给出基代表元切片。
- 【已实现】无 `serde`、`Display`、迭代器等附加实现。
- 测试未覆盖（相对于本文件代码路径）【已实现的事实性陈述】：
  - 错误分支 `AllocationFailed`、`ArithmeticOverflow`、`InvalidRootAutomorphism`、`InvalidInvolution`、`root_basis_action` 内的 `RankMismatch`、`build` 中 `DatumMismatch`（root system 与 involution datum 不一致）均无对应测试；
  - 公开方法 `coordinates`、`same_class`、`basis_representatives`、`identity`（直接调用）、`element_from_ambient`（在 `AdjointCartanFiber` 上直接调用）未在本文件测试中出现；
  - 测试仅覆盖恒等对合与由单反射构造的 `TwistedInvolution`，类型限于 A1、带中心方向的秩 2 datum、A2、A1+A2、33×A1。
- 文档注释中的数学性断言（伴随商基的含义、对偶作用 = 转置的依据、下降已被证明、映射可含中心核等）在本文件内不被重新验证；本包不对其数学正确性作声明。
- 【推断】`check_projection_work` 的计费模型（`lattice_rank × target_rank × vector_count`）是保守上界估计而非精确计数，其依据来自代码注释，未在本文件内证明。
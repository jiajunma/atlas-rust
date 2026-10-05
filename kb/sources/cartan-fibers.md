---
title: Cartan fiber 与伴随 Cartan fiber（cartan_fiber.rs / adjoint_fiber.rs）
source: atlas-rust/cartan-fibers
ingestedAt: 2026-10-05T20:00:00Z
---

# Cartan fiber 与伴随 Cartan fiber（cartan_fiber.rs / adjoint_fiber.rs）

编辑状态：**结构性阅读完成；草案由 Kimi probe 起草，维护者对照源码逐条核对改写**。
本包覆盖 `crates/atlas-real-group/src/` 的 `cartan_fiber.rs`（474 行）与
`adjoint_fiber.rs`（881 行）。它是结构性阅读，不声称 fiber 层的数学验收；
上游引用仅转录自代码文档注释，本包未核对上游字节。

## 概览与层定位

`CartanFiber` 是附着于一个 Cartan 对合的 primary finite component group。
文档注释记录的当前约定公式（`Y` 为余特征格）：

```text
ker_F2(I + theta_Y) / red_2 ker_Z(I + theta_Y)
```

抽象商 `Y^theta / (I + theta_Y)Y` 与之同构但自然坐标不同；实现选择子商坐标
约定（low-pivot 归约基）。层定位（注释原话）：伴随映射在
`AdjointCartanFiber`/`FiberToAdjoint`，grading 在 `CartanGradingData`，弱实
形划分在 `WeakRealFormPartition`，强实层在 `StrongRealClassification`；KGB
数据属于后续层（均不在本包字节内）。

## 裸签名清单（完整）

```rust
// cartan_fiber.rs
pub struct CartanFiber { model: Arc<CartanFiberModel> }        // Clone, Debug
pub struct CartanFiberElement { model, coordinates }           // Clone, Debug;
                                            // PartialEq = ptr_eq && 坐标相等
impl CartanFiber {
    pub fn build(&LatticeInvolution, &IntegerLatticeBudget) -> Result<Self, _>
    pub(crate) fn build_owned(LatticeInvolution, &IntegerLatticeBudget) -> _
    pub fn lattice_rank(&self) -> usize        // = quotient.ambient_dimension()
    pub fn dimension(&self) -> usize           // F2 维数；从不枚举 2^dim 个元素
    pub fn involution(&self) -> &LatticeInvolution
    pub fn identity(&self) -> Result<CartanFiberElement, _>
    pub fn element_from_ambient(&self, ModTwoVector) -> Result<_, _>
    pub fn element_from_coweight_mod_two(&self, &Coweight) -> Result<_, _>
    pub fn canonical_representative(&self, &_) -> Result<ModTwoVector, _>
    pub fn coordinates<'e>(&self, &'e _) -> Result<&'e ModTwoVector, _>
    pub fn add(&self, &_, &_) -> Result<_, _>
    pub fn same_class(&self, &_, &_) -> Result<bool, _>
    pub fn basis_representatives(&self) -> &[ModTwoVector]
    pub(crate) fn validate_induced_map(&self, &Self, &impl ModTwoAmbientMap) -> _
}

// adjoint_fiber.rs
pub struct AdjointFiberBudget { integer_lattice, max_persistent_entries,
    max_projection_operations }                // const new；仅 integer 有 getter
pub struct AdjointBasedRootDatum { datum }     // as_based_root_datum / rank
pub struct AmbientCoweight / AdjointCoweight   // Arc 绑定 + 坐标；Eq 同 ptr_eq 语义
pub struct AdjointProjection { model }         // impl ModTwoAmbientMap
    // source_coweight / map_coweight / source_lattice_rank / target_datum
pub struct AdjointCartanFiber { source, projection, fiber }
    // build / datum / ambient_fiber / projection / dimension / identity /
    // element_from_ambient / element_from_coweight_mod_two / coordinates /
    // canonical_representative / add / same_class / basis_representatives /
    // fiber_map
pub struct AdjointFiberElement(CartanFiberElement)   // 私有字段；is_identity
pub struct FiberToAdjoint { source, target, projection }  // 全私有；apply
const ADJOINT_PERSISTENT_SQUARES: usize = 16
```

## CartanFiber：构造门控与运算

`build_owned` 顺序固定：**先分母后分子**——先
`negative_coweight_eigenspace(coweight_matrix, budget)?` 再
`reduce_basis_mod_two`（整系数分母，预算在此执行；测试锚定 rank 超限即
`IntegerLatticeResourceLimit { resource: "rank" }`），再算
`mod_two_i_plus_coweight_kernel`（对 coweight 矩阵**行**（不转置）的奇数
条目收集后追加行索引自身——`from_ones` 对重复索引 xor-toggle，恰好实现
对角 `+I` 项——`right_kernel()` 得 `ker_F2(I+θ_Y)`），最后
`ModTwoSubquotient::new(numerator, denominator)`。有限域侧不走整数预算，靠
`try_reserve_exact`/`checked_*` 防护（`AllocationFailed`/`ArithmeticOverflow`）。

元素语义：opaque 意味着 provenance 而非保密——`PartialEq` 要求
`Arc::ptr_eq(model)` 且坐标相等，跨 `build` 的同坐标元素必不等
（`CartanFiberMismatch` 测试锚定）。`element_from_ambient` 只做
`to_coordinates`（不在 numerator → `NotInModTwoSubspace`）；
`element_from_coweight_mod_two` 先验 rank 再分配（**不**断言 theta-fixed：
打包的 numerator 即 mod-two 条件）；`canonical_representative` 是 low-pivot
约定下的确定性 ambient 代表，`coordinates` 第 j 位选择
`basis_representatives()[j]`，所选代表之 XOR 恰为 canonical 代表。
`validate_induced_map`（`pub(crate)`）一次性证明 ambient 映射沿子商两个关系
下降（`CartanFiberMapDoesNotDescend { relation: "numerator"/"denominator" }`
均有测试锚点），高层随后按需应用而不缓存稠密商坐标映射。

测试锚点（9 个）：恒等 rank-2 环面 `dimension()==2`、基 `[e0,e1]`；`-I`
平凡（dim 0）；swap 平凡且 `e0` 被拒；非对称例区分存储的 coweight 作用与
负特征格（`red_2 ker_Z` ≠ `I+θ_Y` 的朴素 mod-two 像，后者给出错误的秩）；
rank-3 非对称例锚定分子用行不转置（`ker_F2 = <e0,e2>`，基 `[e2]`）。

## adjoint_fiber.rs：伴随构造

`AdjointBasedRootDatum`：由 `BasedRootDatum::standard(克隆的 Cartan)` 构造；
其 character 基是源 datum 的完整 simple-root 基，cocharacter 基为对应
fundamental-coweight 基（文档声明）。

`AdjointProjection` 是从 ambient 余特征格到伴随格的限制映射：`y` 的目标坐标
= 与源 simple roots 的配对；**可以有中心核，不呈现为同构**（文档声明）。
`source_coweight` 在原始坐标边界绑定 datum（rank 检查），`map_coweight` 先
`ensure_source`（`Arc::ptr_eq`，否则 `DatumMismatch`）再
`check_projection_work(1)`（逐次预算：源秩×目标秩×向量数 ≤
`max_projection_operations`），再逐根 `pair` 收集。`apply_mod_two` 按奇偶
逐目标坐标计算（`bit()` 越界返回 `None` 而非 panic），并实现
`ModTwoAmbientMap`——这是两文件之间的 trait 连接点。

`AdjointCartanFiber::build` 门控顺序固定：

1. `root_system.datum() != root_involution.involution().datum()` →
   `DatumMismatch`；
2. `source.involution() != root_involution.involution()` →
   `CartanFiberInvolutionMismatch`（构造时强制 source fiber 属于实际对合；
   值相等不够——`ambient_fiber()` 注释：只有这个精确值铸造的元素才被下降
   证明覆盖）；
3. `validate_adjoint_build_budget`（在任何目标矩阵分配之前）：记
   `r = semisimple_rank`、`n = lattice_rank`——`r > max_rank` →
   `AdjointFiberResourceLimit{"semisimple rank"}`；`16·r² + r·n >
   max_persistent_entries` → `{"persistent entries"}`；`2·n²·r >
   max_projection_operations` → `{"projection operations"}`（依据注释：每个
   源坐标至多一个分子与一个分母基向量；每次直接投影为每个伴随单根检查
   所有源坐标；`16` 的逐项构成未文档化）；
4. `AdjointProjection::from_source` → `root_basis_action`（逐 simple root：
   `id_of`/`image`/`simple_coordinates` 任一缺失 → `InvalidRootAutomorphism`；
   按列写入作用矩阵）→ `coweight_action = transpose_square(&root_action)`
   （注释：dual 作用本应为 inverse-transpose，而 `RootInvolutionData` 已验证
   root 作用是对合，故 inverse-transpose 即转置）；
5. `LatticeInvolution::new(目标 datum, root_action, coweight_action)` →
   `CartanFiber::build_owned`（复用同一「先分母后分子」流程）→
   `source.validate_induced_map(&fiber, &projection)`（一次性下降证明）。

`FiberToAdjoint`（唯一来源是 `fiber_map()`，字段全私有）的 `apply` 固定
三步：source 的 canonical 代表 → `projection.apply_mod_two` → target 的
`element_from_ambient`；不保留稠密 mod-two 矩阵或缓存像（注释明示），每次
按需投影。

测试锚点（9 个）：A1 恒等投影生成元且 `fiber_map.apply` 与
`element_from_coweight_mod_two(map_coweight(...))` 一致；中心余方向映入核
（中心方向 → identity，root 方向 → 生成元，且保持加法）；A2 twisted 例锚定
伴随 weight/coweight 矩阵分别等于 `[[-1,1],[0,1]]`/`[[-1,0],[1,1]]`
（**推导**而非复用 root 行）；A1+A2 非对称例逐坐标基向量验证
`projection(θ(x)) == θ_adjoint(projection(x))` 的交错关系；rank-33 动态伴随
秩超过历史打包上限仍可构造；两个预算预检测试锚定「拒绝早于目标矩阵分配」。

## 限制与未覆盖面

- 不做数学/正确性验收；`Y^θ/(I+θ_Y)Y` 与子商的同构断言、inverse-transpose
  论证均为注释声明。
- `StructureError` 全变体、`ModTwoSubquotient`/`ModTwoSubspace`/
  `ModTwoVector` 内部、`negative_coweight_eigenspace`/`reduce_basis_mod_two`
  与 `IntegerLatticeBudget` 四参数语义不在本包字节内（integer-lattice 与
  mod-two 各有自己的来源包）。
- 非测试代码无 `panic!`/`assert!`；潜在直接索引点（如 `root_basis_action`
  对列数的隐含一致）依赖构造不变量（阅读观察）。
- 测试未覆盖：`build` 门控 1（`DatumMismatch`）无专门锚点；
  `AdjointProjection::from_source` 未校验 simple-root 数量与目标 rank 的
  一致性（输出长度依赖该隐含一致，阅读观察）。

## 来源与限制

精确读取身份见
[`2026-10-06-cartan-fibers.json`](snapshots/2026-10-06-cartan-fibers.json)：
绑定 Git base、两文件字节 SHA-256 与 Kimi 调用记录。草案由 Kimi probe
（无工具档案）以两文件完整字节起草，维护者对照源码逐条核对改写。本次
知识维护未执行 Atlas、Cargo、测试或 benchmark。

---
title: 对合类型三件套：LatticeInvolution / RootInvolutionData / TwistedInvolution
source: atlas-rust/involution-types
ingestedAt: 2026-10-05T19:40:00Z
---

# 对合类型三件套：LatticeInvolution / RootInvolutionData / TwistedInvolution

编辑状态：**结构性阅读完成；草案由 Kimi probe 起草，维护者对照源码逐条核对改写**。
本包覆盖 `crates/atlas-real-group/src/` 的 `involution.rs`（228 行）、
`twisted_involution.rs`（215 行）与 `root_involution.rs`（354 行）。它是
结构性阅读，不声称对合层的数学验收；上游引用仅转录自代码文档注释，本包
未核对上游字节。三者层层叠加：`LatticeInvolution` 只保证配对保持的对合；
`RootInvolutionData` 在枚举根系上叠加根置换与余根运输；
`TwistedInvolution` 是 distinguished 对合经 Weyl 元平移后仍为对合的载体。

## 裸签名清单（完整）

```rust
// involution.rs
pub struct LatticeInvolution { datum, weight_action, coweight_action }  // 字段私有
impl LatticeInvolution {
    pub fn new(&BasedRootDatum, Vec<Vec<i32>>, Vec<Vec<i32>>) -> Result<Self, _>
    pub fn identity(&BasedRootDatum) -> Result<Self, _>
    pub fn lattice_rank(&self) -> usize
    pub fn datum(&self) -> &BasedRootDatum
    pub fn weight_matrix(&self) -> &[Vec<i32>]
    pub fn coweight_matrix(&self) -> &[Vec<i32>]
    pub fn anti_invariant_rank(&self) -> Result<usize, _>
    pub fn act_on_weight(&self, &Weight) -> Result<Weight, _>
    pub fn act_on_coweight(&self, &Coweight) -> Result<Coweight, _>
}

// root_involution.rs
pub enum RootKind { Imaginary, Real, Complex }   // Clone/Copy/Debug/Eq/PartialEq
pub struct RootInvolutionData { involution, image_by_root, kind_by_root,
    imaginary_simple_roots, real_simple_roots }  // 字段私有
impl RootInvolutionData {
    pub fn new(&RootSystem, LatticeInvolution) -> Result<Self, _>
    pub fn involution(&self) -> &LatticeInvolution
    pub fn image(&self, root: RootId) -> Option<RootId>
    pub fn image_permutation(&self) -> &[RootId]
    pub fn kind(&self, root: RootId) -> Option<RootKind>
    pub fn roots_of_kind(&self, RootKind) -> impl Iterator<Item = RootId> + '_
    pub fn imaginary_simple_roots(&self) -> &[RootId]
    pub fn real_simple_roots(&self) -> &[RootId]
}

// twisted_involution.rs
pub struct TwistedInvolution { weyl_action, root_involution }  // 字段私有
impl TwistedInvolution {
    pub fn new(&BasedRootDatum, &RootSystem, &LatticeInvolution, WeylAction)
        -> Result<Self, _>
    pub fn weyl_action(&self) -> &WeylAction
    pub fn root_involution(&self) -> &RootInvolutionData
    pub fn restricted_roots(&self, &RootSystem) -> Result<RestrictedRootSystem, _>
}
pub(crate) fn compose_matrices(left, right) -> Result<Vec<Vec<i32>>, _>
```

## LatticeInvolution（involution.rs）

配对保持的对合，character/cocharacter 两个作用**分开存储、一起验证**，使配对
保持成为不变量而非调用方约定。文档明言它**不**声称保持有限根系——那更强的
性质由 `RootInvolutionData` 建立。

`new` 的门控顺序固定：两矩阵都须为 `lattice_rank` 阶方阵（否则
`InvalidInvolution`）；先 `weight² = I` 后 `coweight² = I`（`||` 短路，
`i128` 检验累加，溢出传播 `ArithmeticOverflow`）；再验配对保持
`Σ_row W[row][i]·C[row][j] = δ_ij`（即 `W^T·C = I`；不成立 →
`InvalidRootAutomorphism`）。

`anti_invariant_rank`：`X*/ker(1-θ)` 的秩，取自 -1 特征空间，刻意避免浮点：
`(rank - trace(θ)) / 2`，trace 用 `i128` 检验累加；负或奇数 →
`InvalidInvolution`。`act_on_*` 经 `apply_matrix`：行数不符 →
`RankMismatch`，**某行长度不符 → `InvalidInvolution`**（错误类型选择不对称，
阅读观察），点积在 `i128` 累加后 `i32::try_from` 收窄（溢出 →
`ArithmeticOverflow`）。

## RootInvolutionData（root_involution.rs）

在枚举根系上验证对合真是根置换并把每个存储余根运输到像根的余根——第二个
条件是根数据自同构性质：仅配对保持会放过「固定所有根却移动余根中心环面
坐标」的作用（doc 注释的设计动机，有专门测试钉住）。

`new` 的顺序：`DatumMismatch` → `RankMismatch` →
`validate_simple_root_images`（单根级错误**先于**主循环泛型错误：
`SimpleRootImageNotRoot { simple_root }` /
`SimpleCorootImageMismatch { simple_root, image_root }`）→ 逐根主循环
（像非根 → `InvalidRootAutomorphism`；余根运输不符 →
`InvalidRootDatumAutomorphism`）。分类优先级固定：像等于自身 →
`Imaginary`；等于负根（逐坐标 `checked_neg`）→ `Real`；否则 `Complex`。

`subsystem_simple_roots`（imaginary/real 各算一次）：取该类中简单坐标全非负
的根（继承正系），按 `RootId` 升序为候选序；候选可分解为集合内另一成员与
某正坐标向量的差时跳过，否则入选——即子系统在继承正系中的单根。输出按
`RootId` 升序（测试锚定具体顺序）。

访问器对越界 `RootId` 返回 `None`（不 panic）；`roots_of_kind` 惰性按
`RootId` 升序产出。

## TwistedInvolution（twisted_involution.rs）

`w·θ` 再为对合的载体：Atlas Cartan 类最终由其 twisted-共轭轨道的 canonical
代表元编号，而本类型只建立 `(wθ)² = 1` 的根论条件；Cayley/cross 分解在
`CayleyCrossDecomposition`，规范化在 `InnerClass::canonicalize`（均不在本层）。

`new` 的顺序：三个 datum 一致性（任一不符即 `DatumMismatch`，先于一切秩
检查）→ 三个秩检查（root_system → distinguished → weyl_action）→
`compose_matrices(w, θ)` 两格合成（w 左 θ 右）→ **结果重走
`LatticeInvolution::new` 完整门控** → `RootInvolutionData::new` 再验根置换与
余根运输。distinguished 不被存储；`weyl_action` 原样存储并经访问器暴露。

`compose_matrices`（`pub(crate)`）：标准三重循环，`i128` 检验算术 + `i32`
收窄；形状不符报 `RankMismatch { expected: rank, actual: right.len() }`——
`actual` 恒为 `right.len()`，即使真实问题是某行长度不符（阅读观察）。

## 测试锚点（断言值照录）

- 配对保持：rank-2 datum 上 θ = diag(-1,1)，`pair(θ(3,5), θ(7,-11))` =
  `Ok(-34)`；`W=[[-1]], C=[[1]]` → `InvalidRootAutomorphism`；
  `W=[[2]], C=[[0]]` → `InvalidInvolution`。
- A2 上 θ = 负反对角：Real=2、Complex=4、Imaginary=0，real 子系统单根为
  `[id_of([1,1])]`；identity 分类全 Imaginary，imaginary 单根按枚举序为
  `[id_of([0,1]), id_of([1,0])]`。
- 「配对保持但不置换根」的作用（W=[[-1,0],[1,1]]，C=[[-1,1],[0,1]]）过
  `LatticeInvolution::new` 但在 `RootInvolutionData::new` 报
  `SimpleRootImageNotRoot { simple_root: 0 }`；余根运输错误分别对正/负单根
  报 `SimpleCorootImageMismatch`。
- twisted：A1 的 `s0` 平移合法（Real=2）；A2 的三阶元 `s0·s1` →
  `InvalidInvolution`；不同同秩 datum（A2 vs B2）的 Weyl 作用 →
  `DatumMismatch`；带中心环面的 diag(1,-1) distinguished 与 `s0` 合成合法。

## 限制与未覆盖面

- 不做数学/正确性验收；`StructureError` 全变体、`RootSystem::enumerate` 的
  第二参数语义等外部契约不在本包范围。
- 测试未覆盖：`anti_invariant_rank` 全部分支、`act_on_*` 的错误路径、
  `image()`/`kind()` 的越界 `None`、`compose_matrices` 的 `RankMismatch`
  直接触发（在 `TwistedInvolution::new` 内经前置秩检查不可达，但该函数是
  `pub(crate)`）。
- 潜在 panic 路径均为阅读推断：私有辅助的裸下标索引依赖调用点的前置方阵/
  秩检查；字段私有保证实例必经门控构造。
- `RootInvolutionData::new` 不含对「根置换本身为二阶」的独立检查（依赖
  `LatticeInvolution` 的代数对合门控）。

## 来源与限制

精确读取身份见
[`2026-10-06-involution-types.json`](snapshots/2026-10-06-involution-types.json)：
绑定 Git base、三文件字节 SHA-256 与 Kimi 调用记录。草案由 Kimi probe
（无工具档案）以三文件完整字节起草，维护者对照源码逐条核对改写。本次
知识维护未执行 Atlas、Cargo、测试或 benchmark。

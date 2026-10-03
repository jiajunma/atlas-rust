---
title: 紧致 grading：simple-imaginary 根的紧致性位向量
source: atlas-rust/grading
ingestedAt: 2026-10-03T10:32:42Z
---

# 紧致 grading：simple-imaginary 根的紧致性位向量

编辑状态：**结构性阅读，草稿经 Kimi probe 起草、维护者逐条对照源码核对后改写**。
本包解释 `grading.rs` 的位向量纪律与 grading 表；其正确性属于它自己的 HPC
证据链（Cartan/seed gate 等），本包不重述也不扩展。所读字节见
[`snapshots/2026-10-03-grading.json`](snapshots/2026-10-03-grading.json)
（`grading.rs` SHA-256
`a9fe00ba591079815895020e6a11f22ac4a2706a35cfd8592b827eaba6a5c654`，dirty
工作区）。

## Grading：位向量的类型纪律

`Grading` 是 `ModTwoVector` 的 newtype：bit `i` 属于所属模型的
simple-imaginary 根列表的第 `i` 项（该列表是本 crate 确定性根序下
`RootInvolutionData::imaginary_simple_roots` 的副本）。置位表示 NONCOMPACT
（gradings.cpp:20-23）。

**刻意与 ambient coweight 坐标区分**：`CartanFiber` 与 `AdjointCartanFiber`
的 mod-two 坐标索引全 datum 的格坐标或 simple roots，而 grading 只索引
simple-imaginary 位置。在 A2 配 identity involution 等常见情形三者维数相同，
维数检查无法区分它们——必须由类型区分。派生的 `Ord` 是任意的确定性
map-key 序，不是数学序。

## CartanGradingData：一个 Cartan involution 的已验证 grading 表

字段：`imaginary_simple_roots`、`simple_mod_two`、`m_alphas`（ambient fiber
类）、`adjoint_m_alphas`（adjoint fiber 类，用 fundamental-coweight 坐标）、
`base_grading`、`grading_shifts`、`adjoint`。

规范化约定：基点（adjoint fiber 元素零）按 quasisplit normalization 把每个
simple-imaginary 根 grade 为 noncompact，故 `base_grading` 为全一；其余元素的
grading 是对其 canonical ambient representative 的 affine-linear evaluation。

`build(root_system, root_involution, adjoint)` 的两道门：根系统与 involution
数据的 datum 一致（`DatumMismatch`）；adjoint 的 ambient fiber 的 involution
与 involution 数据一致（`CartanFiberInvolutionMismatch`）。**刻意不接收
ambient fiber 参数**：值相等无法表达 fiber 同一性，故 `m_alpha` 对着
`AdjointCartanFiber::ambient_fiber` 构建——即 adjoint descent 被证明时所用
的确切来源。构造拒绝非 faithful 的 shift 矩阵，因此 grading→元素 的逆唯一。

## grading 与元素的互转

- `grading(element)`：计算单个 adjoint fiber 元素的紧致性 grading。
- `element_from_grading(target)`：返回具有所请求 grading 的唯一 adjoint
  fiber 元素。实现是**增广消元**：每列携带其 grading 位加一个标记该 shift
  的 marker 位，归约右端时同时累计求解组合；右端是 target XOR base（base
  全一，故差标记 target 的 compact 位置）；落在 affine image 之外的 grading
  报 `StructureError::ImpossibleGrading`。
- `grading_shift(adjoint_basis_index)`：shift 表访问器，以
  `adjoint_fiber().dimension()` 为上界。

## 来源与限制

- 源码：[grading.rs](../../../crates/atlas-real-group/src/grading.rs)；阅读
  快照 [`2026-10-03-grading.json`](snapshots/2026-10-03-grading.json)。
- 上游行号均转述自源码注释（gradings.cpp），未独立重读上游，随版本演进可能
  漂移。
- 关联：[强实形式分类](strong-real.md)、[Cartan 分类](cartan-classification.md)、
  [弱实形式划分](weak-real-form.md)；`ModTwoVector`/`ModTwoSubspace` 与
  fiber 类型属于后续来源包。
- 本包未执行任何构建、测试或原版运行，不含数学验收、性能或并行结论。
- 起草经由本地 Kimi probe（无工具 profile，`kimi-code/k3-256k`；exit 0，
  147.9s，420 秒期限）。草案由维护者对照源码逐条核对改写；其「待源码核对」
  项中涉及 `build` 的两道门控、`element_from_grading` 的增广消元与
  `ImpossibleGrading` 的内容均已按源码落实，其余骨架内容未采用。调用记录见
  快照的 `kimi_assist`。

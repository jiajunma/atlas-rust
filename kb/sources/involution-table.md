---
title: Twisted involution 表（KGB stage b）
source: atlas-rust/involution-table
ingestedAt: 2026-10-03T10:32:42Z
---

# Twisted involution 表（KGB stage b）

编辑状态：**结构性阅读，草稿经 Kimi probe 起草、维护者逐条对照源码核对后改写**。
本包解释 `involution_table.rs` 的记录格式、播种/传送与访问语义；其正确性属于
它自己的 HPC 证据链（KGB/capacity gate 等），本包不重述也不扩展。所读字节见
[`snapshots/2026-10-03-involution-table.json`](snapshots/2026-10-03-involution-table.json)
（`involution_table.rs` SHA-256
`aab8c87ba1244ea022a35c4539185606d52316821cdc00b2b80f61524b89d8bf`，dirty
工作区）。

## 定位

KGB 构建流水线的 stage b：按 Cartan 类连续存储 twisted involution 的轨道。
每条记录携带：word-level 元素、matrix-level `TwistedInvolution`（theta 加
root 分类）、mod-2 dedup 子空间、$(1+\theta)\rho$、两个长度
（involution length 与 Weyl length），以及 $(1-\theta)X^*$ 的 image-basis 对
（`lift_mat`、`M_real`）。

**播种与传送**：除 image-basis 对外，所有记录字段在入表时由 theta 典范导出；
image-basis 对是例外——在轨道典范 involution 处由 $1-\theta$ 的 echelon
reduction 播种（involutions.cpp:196-208），随后沿 cross-action BFS 传送
（involutions.cpp:242-243），因为该基是路径依赖的且 `y_lift` 的符号依赖于它。

## 编号与存储

编号是调用方的 Cartan 添加顺序（文档纪律：升序 `CartanId`），每条轨道内部按
external-order BFS 排列。`InvolutionId` 跨 Cartan 全局连续递增。
`orbit_slice(cartan)` 返回连续轨道切片及其起始编号。

## 构建

`InvolutionTable::new` 为一个 inner class 建空表（inner class 同时拥有
datum、root system 与 distinguished involution，无需 cross-input gate），并
一次性导出验证过的 simple twist、rank 个 simple-reflection 元素（BFS 边
**不**每次调用重建反射）和来自 positivity slice 的 $2\rho$。

`add_cartan(classification, cartan)` 把一个 Cartan 类的轨道生成为连续切片；
**幂等**（重复添加返回已有切片）；种子与期望大小均来自 classification，生成
的轨道必须恰好填满期望大小。种子经 `WeylElement::from_action` 从矩阵级代表元
转换一次，再应用 (W_length + #Cayley)/2 公式；`CayleyCrossDecomposition`
是 per-class 工具——绝不在每个条目上重建。

## 访问器

- `lookup(&WeylElement)`：以 forward root permutation 为键（stage (a) 已把
  它固定为完整相等性键；同基数的外部系统仍是调用方契约）。
- `record(id)`、`involution_count()`、`root_system()`、`inner_class()`、
  `cartan_of(id)`（轨道切片扫描）。
- `cross(generator, id)`：存储的 cross-action 链接 $s \cdot w \cdot
  \mathrm{twist}(s)$，构建后 O(1)（此处仅转述文档，不作性能结论）。
- `cayley(generator, id)`：Cayley 邻居 $s \cdot w$，经反射乘积加置换查表
  计算；其 Cartan 类尚未添加时返回 `None`——stage-(e) 契约要求先添加该
  form 的 upward-closed Cartan 集合，此后 `None` 即调用方违反不变量。
- `simple_root_kind(id, generator)`：一个访问器覆盖上游的三个
  `is_*_simple` 测试。

## 来源与限制

- 源码：[involution_table.rs](../../../crates/atlas-real-group/src/involution_table.rs)；
  阅读快照
  [`2026-10-03-involution-table.json`](snapshots/2026-10-03-involution-table.json)。
- 上游行号均转述自源码注释（involutions.h/involutions.cpp），未独立重读
  上游，随版本演进可能漂移。
- 关联：[KGB 图结构](kgb-graph-structure.md)、[Weyl 群层](weyl-layer.md)、
  [Cartan 分类](cartan-classification.md)；`ModTwoSubspace`、
  `RealProjection`、`CayleyCrossDecomposition` 的展开属于后续来源包。
- 本包未执行任何构建、测试或原版运行，不含数学验收、性能或并行结论。
- 起草经由本地 Kimi probe（无工具 profile，`kimi-code/k3-256k`；exit 0，
  115.5s，420 秒期限）。草案由维护者对照源码逐条核对改写；其「待源码核对」
  项中涉及 `cross`/`cayley` 签名、`add_cartan` 幂等性、`cartan_of` 语义的
  内容均已按源码落实，其余骨架内容未采用。调用记录见快照的 `kimi_assist`。

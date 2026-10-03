---
title: 强实形式分类：平方类编号与 fiber 轨道
source: atlas-rust/strong-real
ingestedAt: 2026-10-03T10:32:42Z
---

# 强实形式分类：平方类编号与 fiber 轨道

编辑状态：**结构性阅读，草稿经 Kimi probe 起草、维护者逐条对照源码核对后改写**。
本包解释 `strong_real.rs` 的分类结构；其正确性属于它自己的 HPC 证据链
（Cartan/seed gate 等），本包不重述也不扩展。所读字节见
[`snapshots/2026-10-03-strong-real.json`](snapshots/2026-10-03-strong-real.json)
（`strong_real.rs` SHA-256
`34530307523c21e1e47c191c9358a8b54d08187b6d6461eb12a43e69dabb50c0`，dirty
工作区）。

## 定位

强实层构建在已完成的 Cartan 分类之上
（`StrongRealClassification::build(classification, max_fiber_elements)`）。
一个强实形式代表元是某一 square class 的 fiber group 中的一个 $W_{im}$
轨道，lying over one weak real form。

## SquareClassId：编号约定

`SquareClassId` 的编号 IS 该类在 `(adjoint fiber group) / im(toAdjoint)` 的
crate echelon 基下的 coset 坐标整数。stage-(d) 排序审计（`SEED_X0_DESIGN.md`）
证明本 crate 的基选举与上游一致，故这些编号**今天等于**上游 `printStrongReal`
的 `class #N`（output.cpp:524）；该一致性依赖双方共享的 low-pivot RREF 约定，
任一侧换基只会置换标签——partition 结构与所有 size 是约定不变量。

## StrongRealFormRep 与 StrongRealData

`StrongRealFormRep { fiber_orbit, square_class }`：fiber group 中一个
$W_{im}$ 轨道。`fiber_orbit` 的**编号**依赖本 crate 消元选取的具体解（上游
约定不同），但轨道的 class size 不依赖该选择：被 `ker(toAdjoint)` 平移与作用
交换。

`StrongRealData` 是一个 Cartan 类的强实层：平方类集合、逐平方类的 fiber
轨道大小与强代表元。访问器：`square_class_count`/`square_classes`（升序）、
`fiber_orbit_count(square)`、`square_class_representative(square)`（该类编号
最小的局部 weak class——上游 `makeRealFormPartition` 选举它遇到的第一个
form）、`central_square_class(local)`、`strong_real_form(local)`、
`wrf_preimage_mask(local)`（解 $\mathrm{toAdjoint}(y) = \mathrm{wrf\_rep} -
\mathrm{class\_base}$ 的 fiber 元素，innerclass.cpp:1050-1052）、
`fiber_size(local)`、`orbit_elements(square, orbit)`、`weak_real_of_orbit`
（上游 `Fiber::toWeakReal`，cartanclass.cpp:812-817）。

## StrongRealClassification

字段：`per_cartan`、`local_of_form`、`kgb_sizes`、`global_kgb_size`。
`build` 逐 Cartan 计算：取 grading 的 adjoint fiber 与 ambient fiber，
构造 fiber map 的像坐标，以 `ModTwoSubquotient` 求平方商；fiber 维数超过
`MAX_MASK_BITS` 时报 `StrongRealResourceLimit`。

访问器：`strong_real_data(cartan)`、`fiber_size(form, cartan)`（form 不在该
Cartan 时返回 `Some(0)`，因此对全部 Cartan 求和保持正确）、`kgb_size(form)`
（预计算）、`global_kgb_size()`（所有强实形式的总和）。

## StrongRealClassPrint

与上游 `printStrongReal`（output.cpp:503-537）对齐的打印视图：
`class_number()`（类代表元的 `xi_square`，提升到 fundamental fiber，
output.cpp:506-508）、`square()`（possible square 的分子序列，已约化为模分母
的非负剩余，output.cpp:519-521）、`real_forms()`（按 partition 顺序的逐轨道
外部 form 编号，可跨类重复）。

## 来源与限制

- 源码：[strong_real.rs](../../../crates/atlas-real-group/src/strong_real.rs)；
  阅读快照 [`2026-10-03-strong-real.json`](snapshots/2026-10-03-strong-real.json)。
- 上游行号均转述自源码注释（output.cpp/innerclass.cpp/cartanclass.cpp），
  未独立重读上游，随版本演进可能漂移。
- 关联：[KGB 种子](real-form-seed.md)、[Cartan 分类](cartan-classification.md)、
  [KGB 图结构](kgb-graph-structure.md)；`ModTwoSubquotient`、
  `AdjointFiber`、`WeakRealFormPartition` 的展开属于后续来源包。
- 本包未执行任何构建、测试或原版运行，不含数学验收、性能或并行结论。
- 起草经由本地 Kimi probe（无工具 profile，`kimi-code/k3-256k`；exit 0，
  263.0s，420 秒期限）。草案由维护者对照源码逐条核对改写；其「待源码核对」
  项中涉及 `build` 结构、`fiber_size` 的 `Some(0)` 语义、平方商构造的内容
  均已按源码落实，其余骨架内容未采用。调用记录见快照的 `kimi_assist`。

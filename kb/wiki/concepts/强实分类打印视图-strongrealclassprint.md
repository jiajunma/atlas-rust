---
title: 强实分类打印视图 StrongRealClassPrint
summary: 打印视图提供提升到 fundamental fiber 后的平方类号、非负模分母平方分子及按轨道分划顺序排列的外部实形式编号；上游对应关系仅转述自源码注释。
sources:
  - strong-real.md
kind: concept
createdAt: "2026-10-09T15:12:53.975Z"
updatedAt: "2026-10-10T00:52:21.016Z"
tags:
  - 强实形式
  - 打印兼容
  - 证据边界
aliases:
  - 强实分类打印视图-strongrealclassprint
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: 强实分类打印视图 StrongRealClassPrint
summary: StrongRealClassPrint 提供提升到 fundamental fiber 后的类编号、模分母约化的非负平方分子序列，以及按分划顺序排列且可跨类重复的外部实形式编号。
sources:
  - strong-real.md
kind: concept
tags:
  - 强实形式
  - 打印契约
  - 兼容性
aliases:
  - 强实分类打印视图-strongrealclassprint
provenanceState: extracted
---

# 强实分类打印视图 StrongRealClassPrint

`StrongRealClassPrint` 是强实分类的打印视图，提供类编号、可能平方值的分子序列及外部实形式编号。来源将其对应到上游 `printStrongReal`（`output.cpp:503-537`）；这一对应关系转述自源码注释。^[strong-real.md:60-66, strong-real.md:72-73]

## 分类背景

[[强实形式与 fiber 轨道|强实形式]]层建立在已完成的 Cartan 分类之上。一个强实形式代表元对应某个平方类的 fiber group 中的一个 \(W_{im}\) 轨道，并归属于一个弱实形式。每个 Cartan 类的 [[Cartan 类的强实层 StrongRealData|StrongRealData]] 保存平方类集合、逐平方类的 fiber 轨道大小与强代表元。^[strong-real.md:19-22, strong-real.md:34-40]

## 打印字段

`class_number()` 返回类代表元的 `xi_square`，其计算涉及提升到 fundamental fiber。来源将这一约定对应到上游 `output.cpp:506-508`。^[strong-real.md:62-64]

`square()` 返回 possible square 的分子序列，各分子已模分母约化为非负剩余。来源将这一表示约定对应到上游 `output.cpp:519-521`。^[strong-real.md:64-65]

`real_forms()` 返回按 partition 顺序排列的逐轨道外部 form 编号。这些编号可以跨类重复，因此不同类的打印结果可能包含同一个外部实形式编号。^[strong-real.md:65-66]

## 编号约定与不变量

[[平方类编号与换基不变量|SquareClassId]] 是商空间 `(adjoint fiber group) / im(toAdjoint)` 在本 crate 的 echelon 基下的陪集坐标整数。来源记载，stage-(d) 排序审计确认本 crate 与上游的基选举一致，因此在所述版本中，这些编号等于上游 `printStrongReal` 的 `class #N`。这一对应依赖双方共享的 low-pivot RREF 约定；任一侧换基只会置换标签，不改变 partition 结构及各项大小。^[strong-real.md:24-30]

fiber 轨道编号具有另一种选择依赖：`StrongRealFormRep` 的 `fiber_orbit` 编号取决于消元选取的具体解，且本 crate 与上游的约定不同。轨道大小不依赖这一选择，因为由 \(\ker(\mathrm{toAdjoint})\) 给出的平移与作用交换。^[strong-real.md:34-37]

## 证据范围

来源是对 `strong_real.rs` 的结构性阅读，所读字节记录于 dirty 工作区快照 `snapshots/2026-10-03-strong-real.json`。强实分类的正确性属于其自身的 HPC 证据链，包括 Cartan/seed gate 等；该来源不重述或扩展这条证据链。^[strong-real.md:9-15]

上游文件行号均转述自源码注释，未独立重读上游，可能随版本演进而漂移。来源未执行构建、测试或原版运行，不包含数学验收、性能或并行结论，因此上述打印对应关系不应视为本次运行验证结果。^[strong-real.md:70-77]

## Sources

- [strong-real.md](../../sources/strong-real.md)：《强实形式分类：平方类编号与 fiber 轨道》。

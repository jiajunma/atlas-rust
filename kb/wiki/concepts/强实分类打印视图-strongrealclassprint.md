---
title: 强实分类打印视图 StrongRealClassPrint
summary: 打印视图提供提升到 fundamental fiber 后的平方类号、模分母约化为非负剩余的平方分子及按轨道分划顺序排列的外部实形式编号；上游对应关系仅转述自源码注释。
sources:
  - strong-real.md
kind: concept
createdAt: "2026-10-09T15:12:53.975Z"
updatedAt: "2026-10-09T19:36:55.987Z"
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
---

# 强实分类打印视图 StrongRealClassPrint

`StrongRealClassPrint` 是强实分类的打印视图，与上游 `printStrongReal`（`output.cpp:503-537`）的输出约定对齐。它提供类编号、可能平方值的分子序列，以及按 partition 顺序排列的外部实形式编号。^[strong-real.md:60-66]

## 分类背景

[[强实形式与 fiber 轨道|强实形式]]层构建在已完成的 Cartan 分类之上。一个强实形式代表元对应某个平方类的 fiber group 中的一个 $W_{im}$ 轨道，并归属于一个弱实形式。每个 Cartan 类的 [[Cartan 类的强实层 StrongRealData|StrongRealData]] 保存平方类集合、逐平方类的 fiber 轨道大小与强代表元。^[strong-real.md:19-22, strong-real.md:34-40]

## 打印字段

`class_number()` 返回类代表元提升到 fundamental fiber 后的 `xi_square`。来源将这一约定对应到上游 `output.cpp:506-508`。^[strong-real.md:62-64]

`square()` 返回 possible square 的分子序列；分子已按分母约化为非负剩余。来源将这一表示约定对应到上游 `output.cpp:519-521`。^[strong-real.md:64-65]

`real_forms()` 返回按 partition 顺序排列的逐轨道外部 form 编号。这些编号可以跨类重复，因此不同类的打印结果可能包含同一个外部实形式编号。^[strong-real.md:65-66]

## 编号与选择依赖

[[平方类编号与换基不变量|SquareClassId]] 是商空间 `(adjoint fiber group) / im(toAdjoint)` 在 crate echelon 基下的陪集坐标整数。来源记载，stage-(d) 排序审计确认了本 crate 与上游的基选举一致，因此所述版本中的编号等于上游 `printStrongReal` 的 `class #N`。这一对应依赖双方共享的 low-pivot RREF 约定；任一侧换基会置换标签，但不改变 partition 结构及各项大小。^[strong-real.md:24-30]

fiber 轨道编号具有另一种选择依赖：`StrongRealFormRep` 的 `fiber_orbit` 编号取决于消元选取的具体解，且本 crate 与上游的约定不同。轨道大小不依赖这一选择，因为由 $\ker(\mathrm{toAdjoint})$ 给出的平移与作用交换。^[strong-real.md:34-37]

## 证据范围

来源是对 `strong_real.rs` 的结构性阅读，所读字节记录于 dirty 工作区快照。强实分类的正确性属于其自身的 [[HPC 验收证据链]]，该来源不重述或扩展这条证据链。^[strong-real.md:9-15]

上游 `output.cpp`、`innerclass.cpp` 和 `cartanclass.cpp` 的行号均转述自源码注释，未独立重读上游，可能随版本演进而漂移。来源未执行构建、测试或原版运行，也不包含数学验收、性能或并行结论。^[strong-real.md:70-77]

## Sources

- [strong-real.md](strong-real.md)：《强实形式分类：平方类编号与 fiber 轨道》。

---
title: Cartan 类的强实层 StrongRealData
summary: StrongRealData 保存单个 Cartan 类的平方类、fiber 轨道大小和强代表元，并提供弱实形式归属、平方类代表及 toAdjoint 原像查询。
sources:
  - strong-real.md
kind: concept
createdAt: "2026-10-09T15:12:40.298Z"
updatedAt: "2026-10-09T19:36:40.122Z"
tags:
  - Cartan分类
  - 强实形式
  - 数据结构
aliases:
  - cartan-类的强实层-strongrealdata
  - C类S
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# Cartan 类的强实层 StrongRealData

`StrongRealData` 保存单个 Cartan 类的强实层，包括平方类集合、逐平方类的 fiber 轨道大小与强代表元。它建立在已完成的 Cartan 分类之上；一个强实形式代表元是某个平方类的 fiber group 中的一个 $W_{im}$ 轨道，并归属于一个弱实形式。^[strong-real.md:19-22, strong-real.md:39-47]

## 平方类与强代表元

`SquareClassId` 是商空间 $(\text{adjoint fiber group})/\operatorname{im}(\mathrm{toAdjoint})$ 在本 crate 的 echelon 基下的陪集坐标整数。来源记录的排序审计表明，该编号与上游 `printStrongReal` 的 `class #N` 一致；这一对应依赖双方共享的 low-pivot RREF 约定。换基只会置换标签，不改变分区结构与各项大小，详见 [[平方类编号与换基不变量]]。^[strong-real.md:26-30]

强代表元采用 `StrongRealFormRep { fiber_orbit, square_class }` 表示。`fiber_orbit` 的编号依赖消元时选取的具体解，本 crate 与上游的选择约定不同；轨道大小不依赖这一选择，因为 $\ker(\mathrm{toAdjoint})$ 的平移与作用交换。相关概念见 [[强实形式与 fiber 轨道]]。^[strong-real.md:34-37]

## 查询接口

### 平方类与代表元

`square_class_count` 查询平方类数量，`square_classes` 按编号升序枚举平方类。`fiber_orbit_count(square)` 查询指定平方类的 fiber 轨道数；`square_class_representative(square)` 选择该类中编号最小的局部弱实类，上游 `makeRealFormPartition` 则选取遍历时遇到的第一个 form。^[strong-real.md:39-43]

### 局部弱实形式与轨道

`central_square_class(local)`、`strong_real_form(local)` 和 `fiber_size(local)` 提供局部弱实形式对应的平方类、强代表元与 fiber 大小查询。`orbit_elements(square, orbit)` 查询指定轨道的元素，`weak_real_of_orbit` 查询轨道对应的弱实形式，后者对应上游 `Fiber::toWeakReal`。^[strong-real.md:43-47]

`wrf_preimage_mask(local)` 提供满足 $\mathrm{toAdjoint}(y)=\mathrm{wrf\_rep}-\mathrm{class\_base}$ 的 fiber 元素，将局部弱实形式代表元与相应的 fiber 原像联系起来。^[strong-real.md:44-45]

## 在整体分类中的位置

`StrongRealClassification::build(classification, max_fiber_elements)` 逐 Cartan 构建强实层：取得 grading 的 adjoint fiber 与 ambient fiber，构造 fiber map 的像坐标，再利用 [[F₂ 子商的低主元坐标（ModTwoSubquotient）|ModTwoSubquotient]] 求平方商。fiber 维数超过 `MAX_MASK_BITS` 时返回 `StrongRealResourceLimit`。^[strong-real.md:19-20, strong-real.md:51-54]

整体分类通过 `strong_real_data(cartan)` 访问某个 Cartan 类的 `StrongRealData`。整体接口 `fiber_size(form, cartan)` 在 form 不属于该 Cartan 时返回 `Some(0)`，使对全部 Cartan 求和保持正确；`kgb_size(form)` 提供预计算的大小，`global_kgb_size()` 提供所有强实形式的总和。相关汇总见 [[fiber 大小与 KGB 大小汇总]]。^[strong-real.md:56-58]

## 证据边界

本页依据 `strong_real.rs` 的结构性阅读，来源快照来自 dirty 工作区。来源包未执行构建、测试或原版运行，不包含数学验收、性能或并行结论；其正确性依托既有的 Cartan、seed gate 等 HPC 证据链。上游文件行号转述自源码注释，未独立重读上游，可能随版本演进而漂移。^[strong-real.md:9-15, strong-real.md:70-77]

## Sources

- [strong-real.md](strong-real.md) — 强实形式分类：平方类编号与 fiber 轨道。

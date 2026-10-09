---
title: 强实形式与 fiber 轨道
summary: StrongRealFormRep 由平方类及其中的 W_im 轨道确定；轨道编号依赖消元选解，但因 ker(toAdjoint) 平移与作用交换，轨道大小不依赖该选择。
sources:
  - strong-real.md
kind: concept
createdAt: "2026-10-09T15:12:26.826Z"
updatedAt: "2026-10-09T19:36:44.136Z"
tags:
  - 强实形式
  - 纤维轨道
aliases:
  - 强实形式与-fiber-轨道
  - 强F轨
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# 强实形式与 fiber 轨道

强实形式分类建立在已完成的 [[Cartan 分类构造与共享分区|Cartan 分类]]之上。一个强实形式代表元对应某个平方类（square class）的 fiber group 中的一个 $W_{im}$ 轨道，并位于某个弱实形式之上。构造入口为 `StrongRealClassification::build(classification, max_fiber_elements)`。^[strong-real.md:19-22]

## 代表元与编号约定

强实形式代表元表示为 `StrongRealFormRep { fiber_orbit, square_class }`。其中，`fiber_orbit` 的编号依赖消元时选取的具体解，上游采用不同约定；轨道大小则不依赖这一选择，因为来自 $\ker(\mathrm{toAdjoint})$ 的平移与作用交换。^[strong-real.md:34-37]

`SquareClassId` 是商空间 $(\text{adjoint fiber group})/\operatorname{im}(\mathrm{toAdjoint})$ 中陪集在本 crate 阶梯基下的坐标整数。来源记录的排序审计表明，在所读版本中，本 crate 的基选举与上游一致，因此编号等于上游 `printStrongReal` 的 `class #N`。这种一致性依赖双方共享的 low-pivot RREF 约定；换基只会置换标签，不改变分区结构与各项大小。参见 [[平方类编号与换基不变量]]。^[strong-real.md:26-30]

## 与弱实形式的对应

每个 Cartan 类的强实层由 [[Cartan 类的强实层 StrongRealData|StrongRealData]] 保存，包含平方类集合、逐平方类的 fiber 轨道大小及强代表元。`square_classes` 按升序提供平方类，`fiber_orbit_count(square)` 查询指定平方类的轨道数，`square_class_representative(square)` 选取该类中编号最小的局部弱实类；上游 `makeRealFormPartition` 则选取遍历时遇到的第一个 form。^[strong-real.md:39-43]

`central_square_class(local)` 和 `strong_real_form(local)` 提供局部弱实形式对应的平方类与强代表元；`fiber_size(local)` 查询 fiber 大小。`orbit_elements(square, orbit)` 提供指定轨道的元素，`weak_real_of_orbit` 给出轨道所属的弱实形式，对应上游 `Fiber::toWeakReal`。^[strong-real.md:43-47]

`wrf_preimage_mask(local)` 提供满足 $\mathrm{toAdjoint}(y)=\mathrm{wrf\_rep}-\mathrm{class\_base}$ 的 fiber 元素 $y$，将局部弱实形式代表元与相应平方类的基点联系起来。^[strong-real.md:44-45]

## 分类构造与大小汇总

`StrongRealClassification::build` 逐 Cartan 取得 grading 的 adjoint fiber 与 ambient fiber，构造 fiber map 的像坐标，再借助 [[F₂ 子商的低主元坐标（ModTwoSubquotient）|ModTwoSubquotient]] 求平方商。fiber 维数超过 `MAX_MASK_BITS` 时，构造返回 `StrongRealResourceLimit`。^[strong-real.md:51-54]

分类对象通过 `strong_real_data(cartan)` 提供逐 Cartan 的强实数据，并预计算 `kgb_size(form)`。当 form 不属于指定 Cartan 时，`fiber_size(form, cartan)` 返回 `Some(0)`，使对全部 Cartan 的求和保持正确；`global_kgb_size()` 给出所有强实形式的大小总和。参见 [[fiber 大小与 KGB 大小汇总]]。^[strong-real.md:56-58]

## 打印视图

[[强实分类打印视图 StrongRealClassPrint]] 与上游 `printStrongReal` 对齐。`class_number()` 使用提升到 fundamental fiber 后类代表元的 `xi_square`；`square()` 提供 possible square 的分子序列，各项已约化为模分母的非负剩余；`real_forms()` 按 partition 顺序给出逐轨道的外部 form 编号，这些编号可以跨类重复。^[strong-real.md:60-66]

## 证据范围

本页依据对 `strong_real.rs` 的结构性阅读，所读字节来自 dirty 工作区快照。来源中的上游函数位置转述自源码注释，未独立重读上游，行号可能随版本变化。来源未执行构建、测试或原版运行，不提供数学验收、性能或并行结论；分类实现的正确性另属其自身的 [[HPC 验收证据链]]。^[strong-real.md:9-15, strong-real.md:70-77]

## Sources

- [strong-real.md](strong-real.md) — 强实形式分类：平方类编号与 fiber 轨道。

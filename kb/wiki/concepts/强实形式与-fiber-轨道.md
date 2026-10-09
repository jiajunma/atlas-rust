---
title: 强实形式与 fiber 轨道
summary: 强实形式代表元 StrongRealFormRep 由平方类与 fiber group 中的 W_im 轨道确定，并位于一个弱实形式之上；轨道编号依赖求解约定，但轨道大小不受与作用交换的 ker(toAdjoint) 平移影响。
sources:
  - strong-real.md
kind: concept
createdAt: "2026-10-09T15:12:26.826Z"
updatedAt: "2026-10-09T15:12:26.826Z"
tags:
  - 强实形式
  - 群作用
  - 轨道分类
aliases:
  - 强实形式与-fiber-轨道
  - 强F轨
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 强实形式与 fiber 轨道

强实形式分类建立在已完成的 [[Cartan 分类构造与共享分区|Cartan 分类]]之上。一个强实形式代表元对应某个平方类（square class）的 fiber group 中的一个 $W_{im}$ 轨道，并位于某个弱实形式之上。构造入口为 `StrongRealClassification::build(classification, max_fiber_elements)`。^[strong-real.md:19-22]

## 平方类与轨道编号

`SquareClassId` 是商空间
\[
(\text{adjoint fiber group})/\operatorname{im}(\mathrm{toAdjoint})
\]
中陪集在本 crate 阶梯基下的坐标整数。来源记录的排序审计表明，本 crate 与上游采用一致的基选举，因此这些编号在所读版本中等于上游 `printStrongReal` 的 `class #N`。这一对应依赖双方共享的 low-pivot RREF 约定；换基会置换标签，但不改变分区结构与各项大小。参见 [[平方类编号与换基不变量]]。^[strong-real.md:26-30]

强代表元以 `StrongRealFormRep { fiber_orbit, square_class }` 表示。`fiber_orbit` 的编号依赖本 crate 消元时选取的具体解，且上游采用不同约定；轨道大小不依赖这一选择，因为来自 $\ker(\mathrm{toAdjoint})$ 的平移与作用交换。因此，平方类编号的上游一致性不能直接推广为 fiber 轨道编号的一致性。^[strong-real.md:34-37]

## 与弱实形式的对应

每个 Cartan 类的强实层由 `StrongRealData` 保存，内容包括平方类集合、各平方类的 fiber 轨道大小及强代表元。平方类按升序访问；`square_class_representative(square)` 选取该平方类中编号最小的局部弱实类，上游 `makeRealFormPartition` 则选取遍历中遇到的第一个 form。参见 [[Cartan 类的强实层 StrongRealData]]。^[strong-real.md:39-43]

局部弱实形式与强实层之间可通过 `central_square_class(local)`、`strong_real_form(local)` 和 `fiber_size(local)` 查询；`orbit_elements(square, orbit)` 提供轨道元素，`weak_real_of_orbit` 给出轨道所属的弱实形式。`wrf_preimage_mask(local)` 则提供满足
\[
\mathrm{toAdjoint}(y)=\mathrm{wrf\_rep}-\mathrm{class\_base}
\]
的 fiber 元素。^[strong-real.md:43-47]

## 分类构造与大小汇总

`StrongRealClassification::build` 逐 Cartan 取 grading 的 adjoint fiber 与 ambient fiber，构造 fiber map 的像坐标，再借助 `ModTwoSubquotient` 求平方商。fiber 维数超过 `MAX_MASK_BITS` 时返回 `StrongRealResourceLimit`。相关子商表示见 [[F₂ 子商的低主元坐标（ModTwoSubquotient）]]。^[strong-real.md:51-54]

分类对象提供逐 Cartan 的强实数据和 fiber 大小查询，并预计算 `kgb_size(form)`。当 form 不属于指定 Cartan 时，`fiber_size(form, cartan)` 返回 `Some(0)`，以保持对全部 Cartan 求和的正确性；`global_kgb_size()` 汇总所有强实形式的大小。参见 [[fiber 大小与 KGB 大小汇总]]。^[strong-real.md:56-58]

## 打印中的形式编号

[[强实分类打印视图 StrongRealClassPrint]] 与上游 `printStrongReal` 对齐：`class_number()` 使用提升到 fundamental fiber 后类代表元的 `xi_square`，`square()` 给出约化为模分母非负剩余的 possible square 分子序列，`real_forms()` 按 partition 顺序列出逐轨道的外部 form 编号。这些外部 form 编号可以跨类重复。^[strong-real.md:60-66]

## 证据范围

本页依据对 `strong_real.rs` 的结构性阅读，来源记录的字节来自 dirty 工作区快照。上游函数和行号转述自源码注释，未独立重读上游；来源未执行构建、测试或原版运行，因而不提供数学验收、性能或并行结论。^[strong-real.md:9-15, strong-real.md:70-77]

## Sources

- [strong-real.md](strong-real.md) — 强实形式分类：平方类编号与 fiber 轨道。

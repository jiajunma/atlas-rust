---
title: Grading 的位向量类型纪律
summary: Grading 以 newtype 将 simple-imaginary 根位置与 ambient coweight 坐标区分，置位表示非紧致，根索引遵循确定性根序。
sources:
  - grading.md
kind: concept
createdAt: "2026-10-09T14:49:46.123Z"
updatedAt: "2026-10-09T14:49:46.123Z"
tags:
  - Rust类型设计
  - 紧致性
  - 位向量
aliases:
  - grading-的位向量类型纪律
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# Grading 的位向量类型纪律

`Grading` 是 [[F₂ 上的位打包向量（ModTwoVector）|ModTwoVector]] 的 newtype，用于记录 simple-imaginary 根的紧致性。第 `i` 位对应所属模型的 simple-imaginary 根列表中的第 `i` 项；该列表是 `RootInvolutionData::imaginary_simple_roots` 的副本，遵循 crate 的确定性根序。**置位表示非紧致（NONCOMPACT）**，而非紧致性的反面。^[grading.md:17-20]

## 坐标语义与类型边界

Grading 的索引属于 **simple-imaginary 根的位置空间**，必须与 ambient coweight 坐标区分。`CartanFiber` 与 `AdjointCartanFiber` 的模二坐标索引全 datum 的格坐标或 simple roots；grading 则只索引 simple-imaginary 位置。A2 配恒等对合时，这些空间的维数可以相同，因此维数检查不足以识别坐标混用，必须通过类型区分。^[grading.md:22-27]

根列表位置也不能直接当作通常书写的简单根编号。源码中的 A2 恒等对合测试明确以 index 0 对应 α₂，并得到置换矩阵形式的 grading shifts；使用位索引时，应遵循所属模型的根列表顺序。^[grading.md:73-75]

## 构造与排序约定

`from_noncompact` 通过 `ModTwoVector::from_ones` 从非紧致位置集合构造 grading；越界索引经 `toggle` 报出 `IndexOutOfRange`。派生的 `Ord` 仅提供任意但确定性的 map-key 排序，不表示数学上的顺序。^[grading.md:25-27]

位向量具有动态容量。源码中的 33 个 A1 因子测试检查 `noncompact_indices().count() == 33`，用于锚定其不受固定 32/64 位打包限制的行为。^[grading.md:79-79]

## 基点与纤维元素的对应

在 [[Quasisplit 规范化与 grading 的仿射线性求值|quasisplit 规范化]] 下，伴随纤维的零元素把所有 simple-imaginary 根标为非紧致，因此 `base_grading` 是全一向量。其他元素的 grading 通过其 canonical ambient representative 作仿射线性求值：逐根计算 `!dot`，即全一基点与模二配对值的 XOR。^[grading.md:35-38]

`grading(element)` 先取得 canonical representative，外来纤维元素会触发 `CartanFiberMismatch`，随后才逐根配对取反。反向操作 `element_from_grading(target)` 对 `target XOR base` 进行增广消元；由于基点全一，右端标记的是目标中的紧致位置。无法求解时返回 `StructureError::ImpossibleGrading`，可解时则依赖构造阶段验证的 [[Grading shifts 的忠实性不变量|faithful 不变量]] 保证伴随纤维元素的唯一性。^[grading.md:59-67]

## 测试与证据边界

相关测试锚点包括 A1 的规范化及双向往返、A2 恒等对合的四元素双射、A2 扭转时拒绝全紧 grading，以及 A1×A1 交换时 `imaginary_rank == 0`。这些情形同时说明：位向量的语义由模型决定，并非任意 grading 都能对应纤维元素。源码测试尚未覆盖 `element_from_grading` 入口的 `RankMismatch`、build 的两处 `IndexOutOfRange` 及多数溢出或分配分支。^[grading.md:73-81]

来源材料属于结构性源码阅读，未执行构建、测试或原版运行；其中的测试锚点描述不能视为本次运行验证，也不构成数学验收或性能结论。^[grading.md:9-13, grading.md:91-91]

## Sources

- [grading.md](grading.md) — 紧致 grading：simple-imaginary 根的紧致性位向量。

---
title: Grading 的位向量类型纪律
summary: Grading 以独立类型索引 simple-imaginary 根位置，置位表示非紧致；即使维数相同，也不能与纤维坐标混用，其排序仅用于确定性映射键。
sources:
  - grading.md
kind: concept
createdAt: "2026-10-09T14:49:46.123Z"
updatedAt: "2026-10-09T19:29:00.481Z"
tags:
  - grading
  - 类型安全
  - 位向量
aliases:
  - grading-的位向量类型纪律
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# Grading 的位向量类型纪律

`Grading` 是 [[F₂ 上的位打包向量（ModTwoVector）|ModTwoVector]] 的 newtype，用于记录 simple-imaginary 根的紧致性。第 `i` 位对应所属模型的 simple-imaginary 根列表中的第 `i` 项；该列表复制自 `RootInvolutionData::imaginary_simple_roots`，遵循 crate 的确定性根序。**置位表示非紧致（NONCOMPACT）**。^[grading.md:17-20]

## 坐标语义与类型边界

Grading 只索引 **simple-imaginary 根的位置**。`CartanFiber` 与 `AdjointCartanFiber` 的模二坐标则索引全 datum 的格坐标或 simple roots，因此必须将 grading 与 ambient coweight 坐标区分。A2 配恒等对合时，这些空间的维数可以相同，维数检查无法识别坐标混用，需要通过类型区分。^[grading.md:22-27]

位索引遵循模型保存的根列表顺序，不能直接套用通常书写的简单根编号。例如，A2 恒等对合测试中 index 0 对应 α₂，grading shifts 构成置换矩阵。^[grading.md:73-75]

## 构造、排序与容量

`from_noncompact` 通过 `ModTwoVector::from_ones` 从非紧致位置集合构造 grading；越界索引经 `toggle` 报出 `IndexOutOfRange`。派生的 `Ord` 提供任意但确定性的 map-key 排序，不表示数学上的顺序。^[grading.md:25-27]

位向量保持动态容量。来源记录的 33 个 A1 因子测试检查 `noncompact_indices().count() == 33`，作为动态表示行为的测试锚点。^[grading.md:79-79]

## 基点与纤维元素的对应

在 [[Quasisplit 规范化与 grading 的仿射线性求值|quasisplit 规范化]] 下，伴随纤维零元素对应的所有 simple-imaginary 根均为非紧致，因此 `base_grading` 是全一向量。其他元素的 grading 由其 canonical ambient representative 作仿射线性求值：逐根计算 `!dot`，即全一基点与模二配对值的 XOR。^[grading.md:35-38]

`grading(element)` 先取得 canonical representative，再逐根配对取反；外来纤维元素会触发 `CartanFiberMismatch`。这一检查与位向量的坐标类型纪律共同限定了求值输入。^[grading.md:22-27, grading.md:59-60]

反向操作 `element_from_grading(target)` 通过[[通过增广消元反求 grading 对应元素|增广消元]]求解 `target XOR base`。由于基点全一，右端标记目标中的紧致位置；无法求解时返回 `StructureError::ImpossibleGrading`，可解时返回唯一的伴随纤维元素，唯一性依赖构造阶段检查的 [[Grading shifts 的忠实性不变量|faithful 不变量]]。^[grading.md:61-67]

## 测试与证据边界

相关测试包括 A1 的 quasisplit 规范化与双向往返、A2 恒等对合的四元素双射、A2 扭转时拒绝全紧 grading，以及 A1×A1 交换时 `imaginary_rank == 0`。这些用例覆盖不同模型下的 grading 行为，其中 A2 扭转用例明确展示了不可实现的目标 grading。^[grading.md:73-79]

来源列出的未覆盖分支包括 build 的两处 `IndexOutOfRange`、多数溢出或分配分支，以及 `element_from_grading` 入口的 `RankMismatch`。来源属于结构性源码阅读，未执行构建、测试或原版运行；测试锚点描述不构成本次运行验证、数学验收、性能或并行结论。^[grading.md:80-81, grading.md:91-91]

## Sources

- [grading.md](grading.md) — 紧致 grading：simple-imaginary 根的紧致性位向量。

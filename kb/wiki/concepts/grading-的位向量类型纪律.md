---
title: Grading 的位向量类型纪律
summary: Grading 仅索引确定性根序中的 simple-imaginary 根，置位表示非紧致；即使维数相同，也与环境余权和纤维坐标保持类型隔离。
sources:
  - grading.md
kind: concept
createdAt: "2026-10-09T14:49:46.123Z"
updatedAt: "2026-10-09T22:31:05.615Z"
tags:
  - 紧致分级
  - 类型设计
aliases:
  - grading-的位向量类型纪律
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: Grading 的位向量类型纪律
summary: Grading 仅索引确定性根序中的 simple-imaginary 根，置位表示非紧致；即使维数相同，也须与环境余权及纤维坐标保持类型区分。
sources:
  - grading.md
kind: concept
tags:
  - 类型设计
  - 根系
  - 模二线性代数
aliases:
  - grading-的位向量类型纪律
provenanceState: extracted
---

# Grading 的位向量类型纪律

`Grading` 是 [[F₂ 上的位打包向量（ModTwoVector）|ModTwoVector]] 的 newtype，用于记录 simple-imaginary 根的紧致性。第 `i` 位对应所属模型的 simple-imaginary 根列表中的第 `i` 项；该列表复制自 `RootInvolutionData::imaginary_simple_roots`，遵循 crate 的确定性根序。**置位表示非紧致（NONCOMPACT）**。^[grading.md:17-20]

## 坐标语义与类型边界

`Grading` 只索引 simple-imaginary 根的位置。`CartanFiber` 与 `AdjointCartanFiber` 的模二坐标则索引全 datum 的格坐标或 simple roots，因此 grading 必须与 ambient coweight 坐标保持类型区分。在 A2 配恒等对合等情形中，三者维数相同，维数检查无法识别坐标混用。^[grading.md:22-27]

位索引的解释依赖模型保存的根列表顺序。例如，A2 恒等对合测试中 index 0 对应 α₂，grading shifts 构成置换矩阵。不能仅凭位编号将其解释为通常编号下的同序单根。^[grading.md:17-20, grading.md:73-75]

## 构造、排序与容量

`from_noncompact` 经 `ModTwoVector::from_ones` 从非紧致位置集合构造 grading；越界索引经 `toggle` 报出 `IndexOutOfRange`。派生的 `Ord` 仅提供任意但确定性的 map-key 排序，不表示数学上的顺序。^[grading.md:25-27]

位向量保持动态容量。来源记录了 33 个 A1 因子的测试锚点，其中 `noncompact_indices().count() == 33`，用于检查动态表示而非固定宽度打包的容量限制。^[grading.md:79-79]

## 基点与纤维元素的对应

在 [[Quasisplit 规范化与 grading 的仿射线性求值|quasisplit 规范化]] 下，伴随纤维零元素对应的所有 simple-imaginary 根均为非紧致，因此 `base_grading` 是全一向量。其他元素的 grading 由其 canonical ambient representative 作仿射线性求值：逐根计算 `!dot`，即全一基点与模二配对值的 XOR。^[grading.md:35-38]

`grading(element)` 先取得 canonical representative，再逐根配对取反；外来纤维元素触发 `CartanFiberMismatch`。元素来源的检查与坐标类型区分共同约束 grading 的使用，相关构造背景见 [[CartanGradingData 与纤维来源一致性]]。^[grading.md:22-27, grading.md:40-45, grading.md:59-60]

反向操作 `element_from_grading(target)` 通过增广消元求解 `target XOR base`。基点全一，因此右端标记目标中的紧致位置；不可解时返回 `StructureError::ImpossibleGrading`，可解时返回唯一的伴随纤维元素。唯一性依赖构造阶段检查的 [[Grading shifts 的忠实性不变量|faithful 不变量]]，并不意味着任意 grading 都可实现。^[grading.md:61-67]

## 测试与证据边界

相关测试锚点包括 SC A1 的 quasisplit 规范化与双向往返、A2 恒等对合的四元素双射、A2 扭转时拒绝全紧 grading，以及 A1×A1 交换时 `imaginary_rank == 0`。A2 扭转用例还检查伴随纤维维数为零时 `grading_shift(0)` 返回 `None`。^[grading.md:73-79]

来源列出的未覆盖分支包括 build 的两处 `IndexOutOfRange`、多数溢出或分配分支，以及 `element_from_grading` 入口的 `RankMismatch`。材料属于结构性源码阅读，未执行构建、测试或原版运行；上述测试锚点不构成本次运行验证，也不构成数学验收、性能或并行结论。^[grading.md:9-13, grading.md:80-81, grading.md:91-91]

## Sources

- [grading.md](../../sources/grading.md) — 紧致 grading：simple-imaginary 根的紧致性位向量。

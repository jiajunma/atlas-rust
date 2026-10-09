---
title: 保留首批独立列的典范截面（CanonicalModTwoSection）
summary: 保留首批独立输入列并丢弃依赖列，以至多 64 位源掩码对动态目标求解；同一分解可复用，源文所述 3×4 穷举测试锚定数值最小解掩码。
sources:
  - mod-two.md
kind: concept
createdAt: "2026-10-09T15:02:23.782Z"
updatedAt: "2026-10-09T15:02:23.782Z"
tags:
  - 线性映射
  - 典范截面
  - 位掩码
aliases:
  - 保留首批独立列的典范截面canonicalmodtwosection
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 保留首批独立列的典范截面（CanonicalModTwoSection）

`CanonicalModTwoSection` 为 $\mathbb{F}_2$ 线性映射在其像上选择确定性的源代表：按输入列顺序保留首批独立列，丢弃依赖列，并复用一次分解求解同一映射的多个目标。这一选择对应上游 `BinaryMap::section` 的行为。^[mod-two.md:51-60]

## 独立列的选择规则

是否保留一列取决于它相对于已保留输入列的线性独立性。依赖列必须丢弃，即使它的源标记在增广空间中仍然独立。源码注释指出，这一区分固定了可观测的实形种子代表，对应上游 `bitvector.cpp` 中截面构造遗忘零化列的行为；相关背景可参见 [[KGB 种子代表元的可观测影响]]。^[mod-two.md:53-55]

## 表示与求解

源坐标打包为 `u64` 掩码，因此输入列数最多为 64；`columns.len() > 64` 时返回 `ResourceLimitExceeded { limit: 64 }`。目标向量仍采用动态表示，可结合 [[F₂ 上的位打包向量（ModTwoVector）]] 理解：64 列限制针对源坐标掩码，并不要求目标空间至多为 64 维。^[mod-two.md:17-20, mod-two.md:55-59, mod-two.md:89-89]

`solve(target)` 按行升序消元，同时累积源解掩码。最终通过 `Ok(remainder.is_zero().then_some(solution))` 返回结果：若目标属于保留列张成的空间，则得到 `Some(solution)`；否则得到 `None`。同一映射的所有目标共用一次分解。^[mod-two.md:57-60]

## 确定性与测试边界

源码中的穷举测试遍历全部 $2^{12}$ 个 $3\times4$ 映射及每个映射的 8 个目标，锚定该范围内的确定性：有解时返回的解等于数值最小的源掩码。其他测试覆盖 64 列通过、65 列拒绝，以及 130 维动态目标。^[mod-two.md:59-60, mod-two.md:85-89]

这些结论来自源码与测试锚点的结构性阅读。来源材料未执行构建、测试或原版运行，因此不提供数学验收、性能或并行结论；多数 `ArithmeticOverflow` 与 `AllocationFailed` 分支也未被测试覆盖。^[mod-two.md:89-93, mod-two.md:103-103]

## Sources

- [mod-two.md](mod-two.md) — mod-2 线性代数：位打包向量与子空间。

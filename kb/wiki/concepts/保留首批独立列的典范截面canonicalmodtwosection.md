---
title: 保留首批独立列的典范截面（CanonicalModTwoSection）
summary: 保留首批独立输入列，以至多 64 位源掩码为动态维数目标求解；分解可复用，目标不在像中时返回 None。
sources:
  - mod-two.md
kind: concept
createdAt: "2026-10-09T15:02:23.782Z"
updatedAt: "2026-10-10T00:44:17.965Z"
tags:
  - 模二线性代数
  - 典范截面
  - 资源限制
aliases:
  - 保留首批独立列的典范截面canonicalmodtwosection
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: 保留首批独立列的典范截面（CanonicalModTwoSection）
summary: 按输入顺序保留首批独立列，以至多 64 位源掩码求解动态维数目标；分解可复用，目标不在像中时返回 None。
sources:
  - mod-two.md
kind: concept
tags:
  - 模二线性代数
  - 典范截面
  - 资源边界
aliases:
  - 保留首批独立列的典范截面canonicalmodtwosection
---

# 保留首批独立列的典范截面（CanonicalModTwoSection）

`CanonicalModTwoSection` 为 $\mathbb{F}_2$ 线性映射在其像上选择确定性的源代表。它对应上游 `BinaryMap::section`：按输入顺序保留首批独立列，丢弃依赖列，并复用一次分解来求解同一映射的不同目标。^[mod-two.md:51-60]

## 独立列的选择

列是否保留取决于它相对于此前保留列的线性独立性。依赖列被丢弃，即使其源标记在增广空间中仍然独立。源码注释指出，这一区分固定了可观测的实形种子代表，对应上游 `bitvector.cpp` 截面构造中遗忘零化列的行为；相关背景见 [[KGB 种子代表元的可观测影响]]。^[mod-two.md:53-55]

## 表示与容量边界

源坐标打包为 `u64` 掩码，输入列数至多为 64；若 `columns.len() > 64`，返回 `ResourceLimitExceeded { limit: 64 }`。目标向量仍采用动态表示，因此该限制约束源坐标数量，目标维数可以超过 64。来源列出了 130 维动态目标测试；动态向量表示参见 [[F₂ 上的位打包向量（ModTwoVector）]]。^[mod-two.md:55-60, mod-two.md:89-89]

## 求解与分解复用

`solve(target)` 按行升序消元，同时累积源解掩码，最终返回 `Ok(remainder.is_zero().then_some(solution))`。目标属于保留列张成的空间时，结果为 `Ok(Some(solution))`；否则为 `Ok(None)`。同一映射的所有目标可复用已有分解。^[mod-two.md:57-60]

## 确定性与测试证据

源码中的穷举测试遍历全部 $2^{12}$ 个 $3\times4$ 映射及每个映射的 8 个目标，锚定这一范围内的确定性：有解时，解等于数值最小的源掩码。其他测试覆盖 64 列通过、65 列拒绝，以及 130 维动态目标。^[mod-two.md:59-60, mod-two.md:85-89]

上述测试信息来自结构性源码阅读，不代表本次执行结果。来源未执行构建、测试或原版运行，也不包含数学验收、性能或并行结论；整个 `mod_two.rs` 的多数 `ArithmeticOverflow` 与 `AllocationFailed` 分支仍缺少测试覆盖。^[mod-two.md:9-13, mod-two.md:89-93, mod-two.md:103-103]

## Sources

- [mod-two.md](../../sources/mod-two.md) — mod-2 线性代数：位打包向量与子空间。

---
title: 整数对合的 compact/complex/split 分类
summary: 利用 θ+I 的饱和整数核与模二行空间秩计算恒等、交换对和取负因子数量，满足 compact+2·complex+split=rank，但不选取具体分解。
sources:
  - cayley-cross.md
kind: concept
createdAt: "2026-10-09T15:14:41.758Z"
updatedAt: "2026-10-10T02:32:14.499Z"
tags:
  - 整数线性代数
  - 对合分类
aliases:
  - 整数对合的-compactcomplexsplit-分类
  - 整C分
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# 整数对合的 compact/complex/split 分类

整数对合的 compact/complex/split 分类记录整分解中的三类因子数量：`compact` 对应恒等因子，`complex` 对应交换对因子，`split` 对应取负因子。`InvolutionClassification { compact, complex, split }` 只保存计数，刻意不选择或存储具体分解；crate 外只能通过 `classify_involution` 获得分类值。^[cayley-cross.md:63-65]

## 分类公式

设整数对合为 \(\theta\)，底层格的秩为 \(n\)。令 \(p\) 为 \(n\) 减去 \(\theta+I\) 的饱和整数核的秩，令 \(c\) 为 \((\theta+I)\bmod 2\) 的行空间秩，则分类按下式计算。模二秩计算计入所有奇数条目，包括负奇数，相关算法见 [[利用饱和核与模二秩计算整对合分类]]。^[cayley-cross.md:74-78]

\[
\begin{aligned}
p &= n-\operatorname{rank}_{\mathbb Z}\ker(\theta+I),\\
c &= \operatorname{rank}_{\mathbb F_2}\bigl((\theta+I)\bmod 2\bigr),\\
\mathrm{compact} &= p-c,\\
\mathrm{complex} &= c,\\
\mathrm{split} &= n-p-c.
\end{aligned}
\]

这些公式满足维数恒等式 \(\mathrm{compact}+2\,\mathrm{complex}+\mathrm{split}=n\)。其中 `complex` 计数的是交换对，每对占两个格维度。^[cayley-cross.md:63-65, cayley-cross.md:74-78]

## 验证顺序与调用前提

公开入口 `classify_involution(matrix, budget)` 依次执行方阵形状检查、预算检查、对合性检查、构造 \(\theta+I\)，最后调用 `classify_plus_identity`。形状错误首先返回 `InvalidIntegerMatrixShape`；预算门通过 `drop(IntegerMatrix::from_i32_rows(...))` 在立方复杂度的对合检查之前强制执行秩、存储与系数预算。详见 [[对合分类的资源预算与验证顺序]]。^[cayley-cross.md:67-72]

对合性检查使用全程受检的 `i128` 算术计算 \(M^2\)，并与单位阵逐元比较；构造 \(\theta+I\) 时，对角条目使用 `checked_add(1)`。^[cayley-cross.md:70-72]

内部入口 `classify_plus_identity` 为 `pub(crate)`，供 [[中心环面的商对合分类]] 在 Smith 基坐标下复用。调用方负责保证对合前提；该内部入口本身不承担公开入口的完整验证流程。^[cayley-cross.md:74-78]

## 奇偶性与测试锚点

来源列出的分类测试中，\(I_2\) 得到 `(2,0,0)`，A2 反向矩阵 \(\begin{pmatrix}0&-1\\-1&0\end{pmatrix}\) 得到 `(0,1,0)`，结果顺序均为 `(compact, complex, split)`。另两个测试以非对角条目的奇偶差异展示分类结果的变化：奇数条目对应一个交换对，偶数条目对应一个恒等因子和一个取负因子。^[cayley-cross.md:88-89]

\[
\begin{pmatrix}
1&1\\
0&-1
\end{pmatrix}
\longmapsto (0,1,0),
\qquad
\begin{pmatrix}
1&2\\
0&-1
\end{pmatrix}
\longmapsto (1,0,1).
\]

失败路径测试包括：非对合矩阵 \(2I_2\) 返回 `InvalidInvolution`；行长不一致的矩阵在化简之前触发形状错误；秩预算为 1 时，输入 \(I_2\) 返回 `IntegerLatticeResourceLimit`。^[cayley-cross.md:90-91]

## 与 fiber rank 的区别

同一模块中的 `fiber_rank(weight_matrix, budget)` 计算对偶分量群 `dualPi0(-θᵀ)` 的 \(\mathbb F_2\) 维数，即 `Cartan_info` 打印的 fiber size 指数。它的计算目标不同于三类整因子的计数，详见 [[对偶分量群的 fiber rank]]。^[cayley-cross.md:80-83]

`fiber_rank` 不检查对合前提，最终差值使用 `saturating_sub`；`classify_plus_identity` 则使用 `checked_sub`。此外，来源指出 `fiber_rank` 在该文件中没有测试，不能把分类入口的测试覆盖视为对它的验证。^[cayley-cross.md:84-86, cayley-cross.md:100-101]

## 证据边界

来源属于源码结构性阅读，不构成数学或正确性验收。`classify_plus_identity` 仅通过 `classify_involution` 间接获得测试覆盖；是否存在更深的调用链超出材料范围。本次知识维护未执行 Atlas、Cargo、测试或 benchmark，因此上述测试锚点是对既有测试的描述。^[cayley-cross.md:95-103, cayley-cross.md:107-111]

## Sources

- [cayley-cross.md](../../sources/cayley-cross.md) — Cayley/Cross 分解与整对合分类（cayley_cross.rs / involution_classification.rs）。

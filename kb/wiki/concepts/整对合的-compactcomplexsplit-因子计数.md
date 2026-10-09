---
title: 整对合的 compact、complex、split 因子计数
summary: 整对合分类仅记录恒等、交换对与取负三类整分解的数量，不选定具体分解，并满足 compact + 2·complex + split = rank。
sources:
  - cayley-cross.md
kind: concept
createdAt: "2026-10-09T14:43:56.779Z"
updatedAt: "2026-10-09T14:43:56.779Z"
tags:
  - 整对合
  - 整数线性代数
  - 分类
aliases:
  - 整对合的-compactcomplexsplit-因子计数
  - 整C因
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 整对合的 compact、complex、split 因子计数

`InvolutionClassification { compact, complex, split }` 记录整对合的三类整分解因子数：`compact` 对应恒等作用（identity），`complex` 对应交换对（exchanged-pair），`split` 对应取负作用（negated）。该类型刻意不选定或存储具体分解；crate 外只能通过 `classify_involution` 获得分类值。^[cayley-cross.md:63-65]

## 计数公式

设整对合为 \(\theta\)，矩阵维数为 \(n\)。实现先利用 \(\theta+I\) 的饱和核计算 `plus_rank`，再利用模二行空间的秩识别 `complex` 因子，具体公式为：^[cayley-cross.md:74-78]

\[
\begin{aligned}
r_+ &= n-\operatorname{rank}\ker(\theta+I),\\
\mathrm{complex} &= \operatorname{rank}_{\mathbf F_2}
  \bigl((\theta+I)\bmod 2\bigr),\\
\mathrm{compact} &= r_+-\mathrm{complex},\\
\mathrm{split} &= n-r_+-\mathrm{complex}.
\end{aligned}
\]

模二归约以条目的奇偶性为准，负奇数也计为非零。计数满足维数恒等式 \(\mathrm{compact}+2\,\mathrm{complex}+\mathrm{split}=n\)。这一计算机制可参见 [[利用饱和核与模二秩计算整对合分类]]。^[cayley-cross.md:74-78]

## 构造与验证顺序

`classify_involution(matrix, budget)` 首先检查矩阵是否为方阵，形状错误返回 `InvalidIntegerMatrixShape`；随后通过 `IntegerMatrix::from_i32_rows(...)` 强制执行秩、存储和系数预算。预算检查先于立方复杂度的对合检查，因此不满足预算的输入会在昂贵验证之前被拒绝。^[cayley-cross.md:67-72]

通过预算门后，`is_involution` 使用全程 checked 的 `i128` 运算计算 \(M^2\)，并与单位矩阵逐元比较；随后以对角元素 `checked_add(1)` 构造 \(\theta+I\)，交给 `classify_plus_identity` 计算因子数。相关顺序见 [[对合分类的资源预算与验证顺序]]。^[cayley-cross.md:67-72]

`classify_plus_identity` 的可见性为 `pub(crate)`，可供 [[中心环面的商对合分类]] 在 Smith 基坐标下复用。它要求调用方保证输入来自对合；计数减法采用 `checked_sub`。^[cayley-cross.md:74-85]

## 示例与测试锚点

源材料列出的分类测试包括：\(I_2\) 得到 \((2,0,0)\)，而 A2 反向矩阵 \(\begin{pmatrix}0&-1\\-1&0\end{pmatrix}\) 得到 \((0,1,0)\)。这里三元组依次表示 `(compact, complex, split)`。^[cayley-cross.md:63-65, cayley-cross.md:88-89]

奇偶区分测试给出 \(\begin{pmatrix}1&1\\0&-1\end{pmatrix}\mapsto(0,1,0)\)，而 \(\begin{pmatrix}1&2\\0&-1\end{pmatrix}\mapsto(1,0,1)\)，锚定了模二秩对分类结果的影响。^[cayley-cross.md:88-89]

负路径测试包括：非对合 \(2I_2\) 返回 `InvalidInvolution`；不规则行长度矩阵在化简前报告形状错误；秩预算为 1 时，\(I_2\) 返回 `IntegerLatticeResourceLimit`。^[cayley-cross.md:90-91]

## 证据边界

本页依据源码结构性阅读及源材料记录的测试锚点，不代表数学或正确性验收。`classify_plus_identity` 仅通过 `classify_involution` 的测试间接覆盖；此次知识维护未执行 Atlas、Cargo、测试或 benchmark。^[cayley-cross.md:95-101, cayley-cross.md:107-111]

## Sources

- [cayley-cross.md](../../sources/cayley-cross.md)：`involution_classification.rs` 的因子计数、验证顺序、测试锚点与覆盖限制。

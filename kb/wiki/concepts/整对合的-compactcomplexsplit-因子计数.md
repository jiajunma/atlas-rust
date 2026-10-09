---
title: 整对合的 compact、complex、split 因子计数
summary: 通过 θ+I 的饱和整数核及模二行空间秩计算三类因子数量，满足 compact+2·complex+split=rank，但不选定具体分解。
sources:
  - cayley-cross.md
kind: concept
createdAt: "2026-10-09T14:43:56.779Z"
updatedAt: "2026-10-09T22:26:40.852Z"
tags:
  - 整数线性代数
  - 对合分类
aliases:
  - 整对合的-compactcomplexsplit-因子计数
  - 整C因
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: 整对合的 compact、complex、split 因子计数
summary: 整对合分类记录恒等、交换对与取负三类整分解的因子数量，通过饱和核与模二秩计算，不选定具体分解。
sources:
  - cayley-cross.md
kind: concept
tags:
  - 整对合
  - 整数线性代数
  - 分类
aliases:
  - 整对合的-compactcomplexsplit-因子计数
  - 整C因
provenanceState: extracted
---

# 整对合的 compact、complex、split 因子计数

`InvolutionClassification { compact, complex, split }` 记录整对合的三类整分解因子数：`compact` 对应恒等作用（identity），`complex` 对应交换对（exchanged-pair），`split` 对应取负作用（negated）。该类型刻意不选定或存储具体分解；crate 外只能通过 `classify_involution` 获得分类值。^[cayley-cross.md:63-65]

## 计数公式

设整对合为 \(\theta\)，矩阵维数为 \(n\)。`classify_plus_identity` 利用 \(\theta+I\) 的饱和核计算 `plus_rank`，再以模二行空间的秩计算 `complex`，公式如下。^[cayley-cross.md:74-78]

\[
\begin{aligned}
r_+ &= n-\operatorname{rank}\ker(\theta+I),\\
\mathrm{complex} &=
\operatorname{rank}_{\mathbf F_2}\bigl((\theta+I)\bmod 2\bigr),\\
\mathrm{compact} &= r_+-\mathrm{complex},\\
\mathrm{split} &= n-r_+-\mathrm{complex}.
\end{aligned}
\]

模二归约按条目的奇偶性进行，负奇数同样计为非零。结果满足维数恒等式 \(\mathrm{compact}+2\,\mathrm{complex}+\mathrm{split}=n\)，其中每个交换对贡献两个维度。相关计算见 [[利用饱和核与模二秩计算整对合分类]]。^[cayley-cross.md:74-78]

## 构造与验证顺序

`classify_involution(matrix, budget)` 首先检查方阵形状，失败时返回 `InvalidIntegerMatrixShape`；随后通过 `IntegerMatrix::from_i32_rows(...)` 强制执行秩、存储和系数预算。预算门位于立方复杂度的对合检查之前，详见 [[整对合分类的预算门与检查顺序]]。^[cayley-cross.md:67-72]

通过预算门后，`is_involution` 使用全程带溢出检查的 `i128` 运算计算 \(M^2\)，并与单位矩阵逐元比较。随后，对角元素通过 `checked_add(1)` 构造 \(\theta+I\)，再交给 `classify_plus_identity` 计算因子数。^[cayley-cross.md:67-72]

`classify_plus_identity` 为 `pub(crate)`，供 [[中心环面的商对合分类]] 在 Smith 基坐标下复用；对合前提由调用方保证。其计数减法使用 `checked_sub`，区别于同文件中 `fiber_rank` 结尾的 `saturating_sub`。^[cayley-cross.md:74-85]

## 示例与测试锚点

以下三元组均按 `(compact, complex, split)` 排列。源材料记录 \(I_2\mapsto(2,0,0)\)，以及 A2 反向矩阵 \(\begin{pmatrix}0&-1\\-1&0\end{pmatrix}\mapsto(0,1,0)\) 的测试锚点。^[cayley-cross.md:63-65, cayley-cross.md:88-89]

奇偶区分测试中，\(\begin{pmatrix}1&1\\0&-1\end{pmatrix}\mapsto(0,1,0)\)，而 \(\begin{pmatrix}1&2\\0&-1\end{pmatrix}\mapsto(1,0,1)\)。这两例锚定了模二秩对因子计数的影响。^[cayley-cross.md:88-89]

负路径测试包括：非对合 \(2I_2\) 返回 `InvalidInvolution`；不规则行长度矩阵在化简前报告形状错误；秩预算为 1 时，\(I_2\) 返回 `IntegerLatticeResourceLimit`。^[cayley-cross.md:90-91]

## 证据边界

本页依据结构性源码阅读及源材料记录的测试锚点，不构成数学或正确性验收。`classify_plus_identity` 仅经 `classify_involution` 间接覆盖；本次知识维护未执行 Atlas、Cargo、测试或 benchmark。^[cayley-cross.md:95-101, cayley-cross.md:107-111]

源包同时涵盖 Cayley/Cross 分解，但在所读取字节内，两个实现文件互不引用。更深调用链是否存在超出本包范围，不能据此将两条实现线视为已经建立调用关系。^[cayley-cross.md:10-13, cayley-cross.md:102-103]

## Sources

- [cayley-cross.md](../../sources/cayley-cross.md)：整对合因子计数、构造检查顺序、测试锚点与证据边界。

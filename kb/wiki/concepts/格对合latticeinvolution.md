---
title: 格对合（LatticeInvolution）
summary: 分别存储权格与余权格作用，依次验证方阵形状、两作用平方为单位及 W^T·C=I，建立配对保持的对合不变量，但不保证保持根系。
sources:
  - involution-types.md
kind: concept
createdAt: "2026-10-09T14:53:12.743Z"
updatedAt: "2026-10-09T14:53:12.743Z"
tags:
  - 格对合
  - 根数据
  - Rust设计
aliases:
  - 格对合latticeinvolution
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 格对合（LatticeInvolution）

格对合 `LatticeInvolution` 表示根数据上保持配对的格对合。它分别存储 character 格与 cocharacter 格上的作用，并在构造时联合验证，使配对保持成为类型不变量。它本身不保证保持有限根系；根置换与余根运输由更高层的 `RootInvolutionData` 验证。^[involution-types.md:63-65, involution-types.md:82-84]

## 表示与接口

`LatticeInvolution` 的 `datum`、`weight_action` 和 `coweight_action` 字段均为私有。公开接口包括可失败的 `new` 与 `identity` 构造器，以及根数据、格秩、两个作用矩阵的只读访问器；`act_on_weight`、`act_on_coweight` 用于作用于格向量，`anti_invariant_rank` 用于计算反不变秩。^[involution-types.md:21-32]

## 构造不变量与验证顺序

令格秩为 \(r\)，权格与余权格上的作用矩阵分别为 \(W\) 和 \(C\)。`new` 首先要求两个矩阵均为 \(r\times r\) 方阵，否则返回 `InvalidInvolution`；随后按顺序验证 \(W^2=I\) 与 \(C^2=I\)。两项对合检查采用短路逻辑，计算使用受检的 `i128` 累加，算术溢出返回 `ArithmeticOverflow`。^[involution-types.md:67-69]

通过对合检查后，构造器验证配对保持条件
\[
W^{T}C=I,
\qquad
\sum_{\mathrm{row}}W[\mathrm{row}][i]\,C[\mathrm{row}][j]=\delta_{ij}.
\]
该条件不成立时返回 `InvalidRootAutomorphism`。因此，矩阵形状与对合性错误先于配对保持错误被报告；相关分层契约见 [[对合类型的分层构造验证与错误优先级]]。^[involution-types.md:67-71]

## 格向量作用与反不变秩

`act_on_weight` 与 `act_on_coweight` 经由 `apply_matrix` 执行矩阵作用：行数不符返回 `RankMismatch`，某行长度不符则返回 `InvalidInvolution`。点积使用 `i128` 累加，再通过 `i32::try_from` 收窄；溢出返回 `ArithmeticOverflow`。^[involution-types.md:75-78]

`anti_invariant_rank` 计算 \(X^*/\ker(1-\theta)\) 的秩，即 \(-1\) 特征空间的维数，使用精确公式
\[
\operatorname{rank}_{-}(\theta)
=\frac{r-\operatorname{tr}(\theta)}{2}.
\]
迹通过受检的 `i128` 累加计算，避免浮点运算；若分子为负数或奇数，则返回 `InvalidInvolution`。参见 [[对合的反不变秩]]。^[involution-types.md:73-75]

## 与根对合及扭曲对合的关系

配对保持不足以保证根数据自同构。`RootInvolutionData` 在格对合之上验证根被置换，并检查每个存储余根被运输到像根对应的余根；后一条件排除“固定所有根，却移动余根中心环面坐标”的作用。该层还提供 [[对合下的虚根、实根与复根分类]]，但不独立检查根置换的平方为恒等，而是依赖 `LatticeInvolution` 已建立的代数对合条件。^[involution-types.md:82-92, involution-types.md:144-145]

[[扭曲对合（TwistedInvolution）]] 将 Weyl 作用 \(w\) 与 distinguished 对合 \(\theta\) 合成为 \(w\theta\)，矩阵乘法中 \(w\) 在左、\(\theta\) 在右。合成结果重新经过 `LatticeInvolution::new` 的完整验证，再由 `RootInvolutionData::new` 验证根置换与余根运输，从而检查 \((w\theta)^2=1\) 及相应根论条件。^[involution-types.md:104-112]

## 测试锚点与证据边界

源材料记录的测试包括：在秩为 2 的根数据上，\(\theta=\operatorname{diag}(-1,1)\) 作用后的向量配对返回 `Ok(-34)`；\(W=\begin{pmatrix}-1\end{pmatrix},\ C=\begin{pmatrix}1\end{pmatrix}\) 因配对不保持而返回 `InvalidRootAutomorphism`；\(W=\begin{pmatrix}2\end{pmatrix},\ C=\begin{pmatrix}0\end{pmatrix}\) 则因对合性不成立而返回 `InvalidInvolution`。^[involution-types.md:120-122]

另一测试中的矩阵
\[
W=\begin{pmatrix}-1&0\\1&1\end{pmatrix},
\qquad
C=\begin{pmatrix}-1&1\\0&1\end{pmatrix}
\]
能够通过 `LatticeInvolution::new`，却在 `RootInvolutionData::new` 中返回 `SimpleRootImageNotRoot { simple_root: 0 }`，明确展示了格对合与根系保持之间的验证边界。^[involution-types.md:126-129]

本页依据结构性源码阅读材料，不代表数学正确性验收。材料指出，测试尚未覆盖 `anti_invariant_rank` 的全部分支与 `act_on_*` 的错误路径；本次知识维护也未执行 Atlas、Cargo、测试或 benchmark。^[involution-types.md:10-15, involution-types.md:136-139, involution-types.md:155-156]

## Sources

- [involution-types.md](involution-types.md)：对合类型三件套：LatticeInvolution / RootInvolutionData / TwistedInvolution。

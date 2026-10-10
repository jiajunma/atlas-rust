---
title: 格对合（LatticeInvolution）
summary: 分别存储权格与余权格作用，验证两者平方为单位且 WᵀC=I，保证配对保持，但不保证保持根系。
sources:
  - involution-types.md
kind: concept
createdAt: "2026-10-09T14:53:12.743Z"
updatedAt: "2026-10-10T00:36:53.916Z"
tags:
  - 对合
  - 整数格
  - rust
aliases:
  - 格对合latticeinvolution
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: 格对合（LatticeInvolution）
summary: 分别存储权格与余权格作用，依次验证矩阵形状、对合性与配对保持，但不保证保持有限根系。
sources:
  - involution-types.md
kind: concept
tags:
  - 格理论
  - 对合
  - 类型不变量
aliases:
  - 格对合latticeinvolution
---

# 格对合（LatticeInvolution）

`LatticeInvolution` 表示保持配对的格对合。它分别存储 character 格与 cocharacter 格上的作用，并在构造时联合验证，使配对保持成为类型不变量。它本身不保证保持有限根系；根置换与余根运输由 `RootInvolutionData` 进一步验证。^[involution-types.md:63-65, involution-types.md:82-84]

## 表示与接口

类型具有私有字段 `datum`、`weight_action` 和 `coweight_action`。公开接口包括可失败的 `new` 与 `identity` 构造器，以及根数据、格秩和两个作用矩阵的只读访问器。`act_on_weight`、`act_on_coweight` 分别作用于 `Weight` 和 `Coweight`，`anti_invariant_rank` 则计算反不变秩。^[involution-types.md:21-32]

## 构造不变量与验证顺序

令格秩为 \(r\)，权格与余权格作用矩阵分别为 \(W\) 和 \(C\)。`new` 首先要求两矩阵均为 \(r\times r\) 方阵，否则返回 `InvalidInvolution`；随后依次检查 \(W^2=I\) 与 \(C^2=I\)。对合检查采用 `||` 短路逻辑，先检查权格作用，再检查余权格作用；计算使用受检的 `i128` 累加，溢出传播为 `ArithmeticOverflow`。^[involution-types.md:67-69]

通过对合检查后，构造器验证配对保持条件：
\[
W^{T}C=I,
\qquad
\sum_{\mathrm{row}}W[\mathrm{row}][i]\,C[\mathrm{row}][j]=\delta_{ij}.
\]
该条件不成立时返回 `InvalidRootAutomorphism`。因此，矩阵形状与对合性检查先于配对保持检查，相关说明见 [[对合类型的分层构造验证与错误优先级]]。^[involution-types.md:67-71]

## 格向量作用与反不变秩

`act_on_weight` 与 `act_on_coweight` 经由 `apply_matrix` 执行矩阵作用。行数不符返回 `RankMismatch`，某行长度不符则返回 `InvalidInvolution`。点积在 `i128` 中累加，再经 `i32::try_from` 收窄；溢出返回 `ArithmeticOverflow`。^[involution-types.md:75-78]

`anti_invariant_rank` 计算 \(X^*/\ker(1-\theta)\) 的秩，即 \(-1\) 特征空间的维数，采用精确公式：
\[
\operatorname{rank}_{-}(\theta)
=\frac{r-\operatorname{tr}(\theta)}{2}.
\]
迹通过受检的 `i128` 累加计算，避免浮点运算；若分子为负数或奇数，则返回 `InvalidInvolution`。参见 [[对合的反不变秩]]。^[involution-types.md:73-75]

## 与根对合、扭曲对合的关系

`RootInvolutionData` 在格对合之上验证根被置换，并检查每个存储余根被运输到像根对应的余根。仅配对保持不足以保证后一个条件，因为它可能允许“固定所有根，却移动余根中心环面坐标”的作用。根对合层不独立检查根置换的平方为恒等，而是依赖 `LatticeInvolution` 已建立的代数对合条件。^[involution-types.md:82-84, involution-types.md:144-145]

`TwistedInvolution` 将 Weyl 作用 \(w\) 与 distinguished 对合 \(\theta\) 合成为 \(w\theta\)，矩阵乘法中 \(w\) 在左、\(\theta\) 在右。两格上的合成结果重新经过 `LatticeInvolution::new` 的完整验证，再由 `RootInvolutionData::new` 验证根置换与余根运输。该层建立 \((w\theta)^2=1\) 的根论条件；Cayley/cross 分解与规范化由其他层承担，参见 [[扭对合与 Weyl 平移]]。^[involution-types.md:104-112]

## 测试锚点

源材料记录，在秩为 2 的根数据上，\(\theta=\operatorname{diag}(-1,1)\) 作用于向量 \((3,5)\) 和 \((7,-11)\) 后，配对返回 `Ok(-34)`。一维矩阵 \(W=\begin{pmatrix}-1\end{pmatrix}\)、\(C=\begin{pmatrix}1\end{pmatrix}\) 返回 `InvalidRootAutomorphism`；\(W=\begin{pmatrix}2\end{pmatrix}\)、\(C=\begin{pmatrix}0\end{pmatrix}\) 则返回 `InvalidInvolution`。这些断言分别锚定配对保持与对合性检查。^[involution-types.md:120-122]

另一测试使用
\[
W=\begin{pmatrix}-1&0\\1&1\end{pmatrix},
\qquad
C=\begin{pmatrix}-1&1\\0&1\end{pmatrix}.
\]
它们通过 `LatticeInvolution::new`，却在 `RootInvolutionData::new` 中触发 `SimpleRootImageNotRoot { simple_root: 0 }`，展示了配对保持与根系保持之间的验证边界。^[involution-types.md:126-129]

## 证据边界

本页依据结构性源码阅读材料，不代表数学正确性验收。材料中的上游引用仅转录自代码文档注释，未核对上游字节；测试尚未覆盖 `anti_invariant_rank` 的全部分支及 `act_on_*` 的错误路径。私有辅助函数的潜在下标 panic 路径属于阅读推断，其安全性依赖调用点的前置方阵与秩检查；私有字段保证实例经过门控构造。^[involution-types.md:9-15, involution-types.md:136-143]

来源快照绑定 Git base、三个源码文件的字节 SHA-256 与草案调用记录，维护者已对照源码逐条核对改写。本次知识维护未执行 Atlas、Cargo、测试或 benchmark，因此上述测试锚点不表示本次执行结果。^[involution-types.md:149-156]

## Sources

- [involution-types.md](../../sources/involution-types.md)：对合类型三件套：LatticeInvolution / RootInvolutionData / TwistedInvolution。

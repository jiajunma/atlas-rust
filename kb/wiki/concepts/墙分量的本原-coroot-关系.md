---
title: 墙分量的本原 Coroot 关系
summary: labels_for_component 对墙 coroot 列矩阵作 Gauss-Jordan 消元，要求恰有一个自由列，再通过通分、最大公约数约化及符号调整生成本原整数关系。
sources:
  - alcove.md
kind: concept
createdAt: "2026-10-09T14:24:50.483Z"
updatedAt: "2026-10-09T14:24:50.483Z"
tags:
  - 根系
  - 线性代数
  - 精确算术
aliases:
  - 墙分量的本原-coroot-关系
  - 墙C关
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 墙分量的本原 Coroot 关系

墙分量的本原 Coroot 关系由 `labels_for_component` 计算：它将分量中各面墙的 coroot 作为矩阵列，求出唯一独立的线性关系，再将有理系数化为互素的整数系数。这些系数用于 [[墙连通分量与重心分数约束]]，也参与 [[Alcove 根格顶点与基本 Alcove 约化]]。^[alcove.md:132-143, alcove.md:152-162]

## 分量与关系的构造

墙分量由 `root_components` 划分：当 `bracket(α, β) != 0` 时，通过并查集合并两面墙。分量按根首次出现的顺序排列，分量内部按根编号升序排列；`bracket` 查询失败会被 `unwrap_or(0)` 视为不相连。^[alcove.md:128-131]

设分量的墙 coroot 为 \(\alpha_1^\vee,\ldots,\alpha_m^\vee\)，按列组成有理矩阵
\[
A=\begin{bmatrix}\alpha_1^\vee&\cdots&\alpha_m^\vee\end{bmatrix}.
\]
`labels_for_component` 对其执行 Gauss–Jordan 全消元，包括主元行上方的消元，并要求恰有一个自由列。若自由列为 \(f\)，则关系向量取 \(c_f=1\)，各主元列的系数取相应消元行自由列元素的负值，从而得到 \(Ac=0\)。^[alcove.md:138-141]

若自由列数量不是一个，函数返回 `RootSystemInvariantViolation`，其 invariant 为 `"alcove wall component must have one coroot relation"`。因此，这一实现要求关系空间为一维。^[alcove.md:138-141]

## 整数化与本原化

关系向量先通过 `checked_lcm` 对分母通分，再转换为 `i64`，随后用全体系数的最大公约数约化，得到本原整数关系
\[
\sum_{i=1}^{m} c_i\alpha_i^\vee=0.
\]
若首个系数为负，则使用 `checked_neg` 将整个向量取负；首个系数为零时不执行符号翻转。该符号约定依据首元素，而不是首个非零元素。^[alcove.md:141-143]

## 在 Alcove 算法中的用途

在重心计算中，`barycentre_eq` 先将每面墙的分数初始化为 \((0,1)\)，再为各分量计算关系系数。非整值墙的分数被改写为 `1 / (n_off * labels[position])`，整值墙则保持 \((0,1)\)。关系系数由此进入逐墙的重心分数约束。^[alcove.md:132-134]

在根格顶点计算中，`root_vertex_simple` 使用全正的本原关系系数，先丢弃第一面系数为 \(1\) 的墙，再由其余墙构造转置子 Cartan 矩阵。如果不存在系数为 \(1\) 的墙，则报错 `"alcove component has no coefficient-1 wall"`；若初次求得的坐标非整，则依次尝试后续系数为 \(1\) 的墙对应的取值加一修正。^[alcove.md:152-162]

## 证据边界

上述说明来自对 `alcove.rs` 的结构性阅读，不构成数学正确性验收。来源明确指出，`labels_for_component`、墙分量划分、重心分数计算和根格顶点计算均缺少相应单元测试；所列上游行号来自代码注释，未独立核对上游字节。^[alcove.md:10-13, alcove.md:171-176]

## Sources

- [alcove.md](alcove.md)

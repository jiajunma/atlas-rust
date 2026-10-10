---
title: 墙分量的本原 Coroot 关系
summary: labels_for_component 对墙余根列矩阵进行精确消元，要求恰有一个自由列，再通分、约去公因子并调整符号以得到本原整数关系。
sources:
  - alcove.md
kind: concept
createdAt: "2026-10-09T14:24:50.483Z"
updatedAt: "2026-10-09T22:11:12.566Z"
tags:
  - root-systems
  - linear-algebra
aliases:
  - 墙分量的本原-coroot-关系
  - 墙C关
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: 墙分量的本原 Coroot 关系
summary: labels_for_component 对墙余根列矩阵作 Gauss–Jordan 消元，要求唯一自由列，再经通分、最大公约数约化与符号调整构造本原整数关系。
sources:
  - alcove.md
kind: concept
tags:
  - 余根
  - 整数关系
  - 精确线性代数
aliases:
  - 墙分量的本原-coroot-关系
  - 墙C关
---

# 墙分量的本原 Coroot 关系

墙分量的本原 Coroot（余根）关系由 `alcove.rs` 中的私有函数 `labels_for_component` 计算。它以各墙余根为列构造有理矩阵，求取唯一自由方向对应的关系，再将其化为本原整数系数。这些系数用于重心分数约束与根格顶点计算。^[alcove.md:21-24, alcove.md:132-143, alcove.md:152-162]

## 墙分量与列顺序

`root_components` 使用并查集划分墙分量：当 `bracket(α, β) != 0` 时合并两面墙。分量按根首次出现的顺序追加，分量内部按根编号升序排列。若 `bracket` 查询失败，`unwrap_or(0)` 将其视为不相连；该函数不返回 `Result`。这一顺序确定了关系向量中系数的位置。^[alcove.md:128-143]

## 有理关系的求解

设分量中的墙余根为 $\alpha_1^\vee,\ldots,\alpha_m^\vee$，按列组成矩阵 $A=(\alpha_1^\vee\ \cdots\ \alpha_m^\vee)$。`labels_for_component` 使用 `Rational` 执行 Gauss–Jordan 全消元，包括主元行上方的消元，并要求自由列恰好为一个；否则返回 `RootSystemInvariantViolation`，其 invariant 为 `"alcove wall component must have one coroot relation"`。^[alcove.md:138-141]

设唯一自由列为 $f$，函数取关系向量的自由分量 $c_f=1$，并将每个主元列的系数设为对应消元行在自由列上的元素的负值，即 `-matrix[pivot][free]`。这样得到满足 $Ac=0$ 的有理关系。相关计算属于 [[Alcove 算法中的精确有理线性代数]]。^[alcove.md:138-143]

## 整数化与符号约定

函数使用 `checked_lcm` 对关系系数的分母通分，将系数转换为 `i64`，再以全体系数的最大公约数约化。所得整数系数满足 $\sum_i c_i\alpha_i^\vee=0$，且不存在大于 $1$ 的公因子，即为本原整数关系。^[alcove.md:141-143]

符号调整只检查向量的首元素：若首元素为负，则用 `checked_neg` 将整个向量取负；若首元素为零，则不翻转符号。这里的规则并不是寻找首个非零元素，也不能仅凭这一步将实现描述为逐项保证系数为正。^[alcove.md:142-143]

## 在 Alcove 算法中的用途

在 [[墙连通分量与重心分数约束]] 中，`barycentre_eq` 将所有墙的分数初始化为 $(0,1)$，再调用 `labels_for_component`。非整值墙的分数改为 $(1,\texttt{n_off}\times\texttt{labels[position]})$，整值墙保持 $(0,1)$。这些分数随后进入 [[Alcove 重心计算与标准参数重建]] 的逐墙线性方程。^[alcove.md:70-76, alcove.md:132-134]

在 [[Alcove 根格顶点与基本 Alcove 约化]] 中，`root_vertex_simple` 使用全正的本原关系系数，丢弃第一面系数为 $1$ 的墙，由其余墙构造转置子 Cartan 矩阵。若不存在系数为 $1$ 的墙，则报错 `"alcove component has no coefficient-1 wall"`。若初次求得的坐标非整，则依次将后续系数为 $1$ 的墙的取值加一重试；全部尝试仍非整时，报错 `"alcove vertex lies outside the root lattice"`。^[alcove.md:152-162]

## 证据边界

本页依据 `alcove.rs` 的结构性阅读，不构成数学正确性验收。来源明确指出，`labels_for_component`、`root_components`、`barycentre_eq` 与 `root_vertex_simple` 均无相应单元测试；上游位置引用转录自代码注释，未独立核对上游字节。本次知识维护未执行 Atlas、Cargo、测试或 benchmark。^[alcove.md:10-13, alcove.md:173-176, alcove.md:185-190]

## Sources

- [alcove.md](../../sources/alcove.md)：Alcove 几何：alcove_center 与 root_vertex_of_alcove。

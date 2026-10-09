---
title: 墙分量的本原 Coroot 关系
summary: labels_for_component 对墙余根列矩阵作 Gauss-Jordan 消元，要求唯一自由列，再经通分、最大公约数约化和符号调整构造本原整数关系。
sources:
  - alcove.md
kind: concept
createdAt: "2026-10-09T14:24:50.483Z"
updatedAt: "2026-10-09T20:28:39.567Z"
tags:
  - 余根
  - 整数关系
  - 精确线性代数
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
summary: labels_for_component 对墙 coroot 列矩阵作 Gauss–Jordan 消元，要求恰有一个自由列，再通过通分、最大公约数约化及符号调整生成本原整数关系。
sources:
  - alcove.md
kind: concept
tags:
  - 根系
  - 线性代数
  - 精确算术
aliases:
  - 墙分量的本原-coroot-关系
  - 墙C关
---

# 墙分量的本原 Coroot 关系

墙分量的本原 Coroot（余根）关系由 `alcove.rs` 中的私有函数 `labels_for_component` 计算。它将分量各墙的余根作为矩阵列，求出一维关系空间中的有理关系，再化为互素的整数系数。这些系数用于 [[墙连通分量与重心分数约束]]，也参与 [[Alcove 根格顶点与基本 Alcove 约化]]。^[alcove.md:21-24, alcove.md:132-143, alcove.md:152-162]

## 墙分量与列顺序

`root_components` 使用并查集划分墙分量：当 `bracket(α, β) != 0` 时合并两面墙。分量按根首次出现的顺序追加，分量内部按根编号升序排列。若 `bracket` 查询失败，`unwrap_or(0)` 会将其视为不相连；此函数不返回 `Result`。^[alcove.md:128-131]

## 有理关系的求解

设一个分量的墙余根为 \(\alpha_1^\vee,\ldots,\alpha_m^\vee\)，按列组成有理矩阵 \(A=(\alpha_1^\vee\ \cdots\ \alpha_m^\vee)\)。`labels_for_component` 对该矩阵执行 Gauss–Jordan 全消元，包括主元行上方的消元，并要求恰好存在一个自由列。^[alcove.md:138-141]

若自由列为 \(f\)，关系向量取 \(c_f=1\)，每个主元列的系数取对应消元行在自由列上的元素的负值，得到 \(Ac=0\)。若自由列数量不是一个，函数返回 `RootSystemInvariantViolation`，其 invariant 为 `"alcove wall component must have one coroot relation"`。^[alcove.md:138-141]

## 整数化与本原化

有理关系的分母通过 `checked_lcm` 通分，系数转换为 `i64` 后，再除以全体系数的最大公约数。所得本原整数关系满足 \(\sum_{i=1}^{m}c_i\alpha_i^\vee=0\)，其系数没有大于 \(1\) 的公因子。^[alcove.md:141-143]

符号调整依据向量的首元素：若首元素为负，则通过 `checked_neg` 将整个向量取负；若首元素为零，则不翻转符号。因此，此处的具体规则是检查首元素，而非寻找首个非零元素。^[alcove.md:142-143]

## 在 Alcove 算法中的用途

在重心计算中，`barycentre_eq` 先将所有墙的分数初始化为 \((0,1)\)，再为各分量求关系系数。非整值墙的分数改为 \((1,\texttt{n_off}\times\texttt{labels[position]})\)，整值墙仍保持 \((0,1)\)。这些系数由此进入 [[Alcove 重心计算与标准参数重建]] 的逐墙方程。^[alcove.md:70-76, alcove.md:132-134]

在根格顶点计算中，`root_vertex_simple` 使用全正的本原关系系数，丢弃第一面系数为 \(1\) 的墙，并由其余墙构造转置子 Cartan 矩阵；若不存在这样的墙，则报错 `"alcove component has no coefficient-1 wall"`。若初次求得的坐标非整，则依次尝试将后续系数为 \(1\) 的墙的取值加一，寻找整数解；全部尝试仍非整时，报错 `"alcove vertex lies outside the root lattice"`。^[alcove.md:152-162]

## 证据边界

本页依据 `alcove.rs` 的结构性阅读，不构成数学正确性验收。来源明确指出，`labels_for_component`、`root_components`、`barycentre_eq` 与 `root_vertex_simple` 均缺少相应单元测试；上游位置引用转录自代码注释，未独立核对上游字节。^[alcove.md:10-13, alcove.md:171-176]

## Sources

- [alcove.md](../../sources/alcove.md)：Alcove 几何：alcove_center 与 root_vertex_of_alcove。

---
title: Alcove 根格顶点与基本 Alcove 约化
summary: root_vertex_of_alcove 使用朴素有理下取整逐分量求根格顶点，借助转置 Cartan 矩阵求逆及系数为 1 的墙重试整性，使 gamma 减去顶点落入基本 alcove 的 Weyl 轨道。
sources:
  - alcove.md
kind: concept
createdAt: "2026-10-09T14:24:58.442Z"
updatedAt: "2026-10-09T14:24:58.442Z"
tags:
  - alcove几何
  - 根格
  - Weyl群
aliases:
  - alcove-根格顶点与基本-alcove-约化
  - A根A约
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# Alcove 根格顶点与基本 Alcove 约化

`root_vertex_of_alcove` 为给定的有理权 `gamma` 计算根格顶点 `vertex`，使 `gamma - vertex` 落在基本 alcove 的 Weyl 轨道中。它服务于 locator 切片的基本 alcove 约化，是 crate 内可见的辅助函数，返回 `Result<Weight, StructureError>`。^[alcove.md:17-24, alcove.md:44-45, alcove.md:145-150]

## 墙集、分量与取整约定

算法按墙的连通分量分别求顶点，再将各分量的结果相加。墙集由 `wall_set` 提供，但顶点计算**显式忽略整值墙集合 `integrals`**。墙分量通过并查集构造：当 `bracket(α, β) != 0` 时合并两面墙；分量按根首次出现的顺序排列，分量内按根编号升序排列。相关概念见 [[Alcove 墙集与整值墙筛选]]。^[alcove.md:128-131, alcove.md:147-150]

顶点计算使用朴素有理下取整，即 `dot.div_euclid(denominator)`。这一约定区别于对负根作修正的 `floor_eval`，不能将两者混用。^[alcove.md:147-150]

## 单分量的根格顶点构造

`root_vertex_simple` 使用墙 coroot 之间的本原核关系，其系数要求全正。关系计算以各墙 coroot 为矩阵列，经有理 Gauss–Jordan 消元求核；自由列必须恰好一个，再通过分母最小公倍数通分、最大公约数约化及符号调整得到本原关系。详见 [[墙分量的本原 Coroot 关系]]。^[alcove.md:136-143, alcove.md:152-154]

算法首先丢弃按分量顺序遇到的第一面关系系数为 1 的墙。其余墙用于构造转置子 Cartan 矩阵，矩阵元素为 `transposed[row][column] = bracket(id(column), id(row))`；这里的 `bracket` 错误通过 `?` 传播。随后求有理逆矩阵，计算候选系数向量
\[
\mathrm{base}=C^{-T}\,\mathrm{floors}.
\]
其中 `floors` 使用前述朴素下取整值。^[alcove.md:147-157]

候选系数以整数分子和公共分母表示。若存在分量不能被分母整除，算法依次尝试将每面后续关系系数为 1 的墙的取值加 1，并重新计算候选。它选取第一个全部为整数的候选，以 `entry / denominator` 为系数，对**根坐标**作带溢出检查的线性组合，最后转换为 `i32` 返回。这里用于求关系和建立方程的是 coroot，最终组合生成顶点的则是根。^[alcove.md:152-162]

## 精确算术与失败行为

求逆辅助函数 `rational_inverse` 在增广矩阵 `[A|I]` 上执行 Gauss–Jordan 消元，返回满足 `inverse = numerator / d` 的表示。非方阵或缺少主元时返回 `Ok(None)`；正分母约定依赖 `Rational`，代码没有另作断言。相关背景见 [[Alcove 算法中的精确有理线性代数]]。^[alcove.md:164-169]

墙分量的 coroot 关系若不唯一，关系计算返回 `RootSystemInvariantViolation`。单分量顶点构造还会在找不到系数为 1 的墙、生成元 Cartan 矩阵奇异，或全部候选均非整时失败，对应诊断分别为 `"alcove component has no coefficient-1 wall"`、`"alcove generator Cartan matrix is singular"` 和 `"alcove vertex lies outside the root lattice"`。^[alcove.md:138-143, alcove.md:152-162]

## 证据范围

本页依据源码结构性阅读记录，不构成数学正确性验收。来源中的上游 C++ 行号仅转录自代码注释，未核对上游字节；`root_vertex_simple`、`rational_inverse`、墙集及分量计算等相关路径没有单元测试覆盖。来源记录的本次知识维护也未执行 Atlas、Cargo、测试或 benchmark。^[alcove.md:9-13, alcove.md:171-176, alcove.md:185-190]

## Sources

- [alcove.md](alcove.md)：Alcove 几何：alcove_center 与 root_vertex_of_alcove。

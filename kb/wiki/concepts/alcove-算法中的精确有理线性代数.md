---
title: Alcove 算法中的精确有理线性代数
summary: 有理方程求解与矩阵求逆通过精确消元处理唯一性和奇异性，并结合 checked 运算、最小公倍数通分及整个有理数的精确整数转换控制溢出与符号错误。
sources:
  - alcove.md
kind: concept
createdAt: "2026-10-09T14:25:09.328Z"
updatedAt: "2026-10-09T14:25:09.328Z"
tags:
  - 线性代数
  - 精确算术
  - 错误处理
aliases:
  - alcove-算法中的精确有理线性代数
confidence: 1
provenanceState: merged
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# Alcove 算法中的精确有理线性代数

Alcove 模块使用 `Rational` 矩阵消元、分母通分与整数可除性检查，计算重心、墙分量的本原 coroot 关系及根格顶点。这些计算分别支撑 [[Alcove 重心计算与标准参数重建]]、[[墙分量的本原 Coroot 关系]] 和 [[Alcove 根格顶点与基本 Alcove 约化]]。^[alcove.md:63-89, alcove.md:136-169]

## 重心方程与唯一解

`alcove_center` 根据墙集与逐墙分数构造线性方程组。每面墙的系数是 coroot 坐标乘以 `scale`，右端为 `floor_eval_nbr * scale + fracs.0`；随后为 `datum.radical_basis()?` 的每个元素补充方程，其系数乘以 `gamma.denominator()`，右端取与 `gamma.numerator()` 的点积。相关整数乘法和点积采用 checked 算术，溢出报告 `ArithmeticOverflow`。^[alcove.md:70-76]

`solve_rational_system(rows, columns)` 将增广 `Rational` 矩阵消元至约化阶梯形。任一未知量列找不到主元，或消元后出现系数全为零而右端非零的行，都返回 `None`。因此，解不唯一与方程不自洽在调用方统一转换为 `RepInvariantViolation`，错误信息为 `"alcove center equations have no unique solution"`。^[alcove.md:77-79, alcove.md:164-166]

## 本原 coroot 关系

`labels_for_component` 以一个墙分量的 coroot 为列构造 `Rational` 矩阵，执行包含主元行上方消元的 Gauss–Jordan 全消元。消元后必须恰有一个自由列，否则返回 `RootSystemInvariantViolation`，错误信息为 `"alcove wall component must have one coroot relation"`。^[alcove.md:136-141]

关系向量的自由列坐标设为 1，主元列坐标取 `-matrix[pivot][free]`。随后通过 `checked_lcm` 通分、转换为 `i64`，再以全体坐标的 `gcd` 约化为本原整数向量；首元素为负时，用 `checked_neg` 将整体取负，首元素为零时不改变符号。^[alcove.md:141-143]

该关系用于重心的逐墙分数：`barycentre_eq` 将结果初始化为 `(0, 1)`，只把非整值墙改写为 `(1, n_off * labels[position])`，整值墙保持初值。相关分量组织见 [[墙连通分量与重心分数约束]]。^[alcove.md:126-134]

## 精确矩阵求逆与根格整性

`rational_inverse` 在增广矩阵 `[A|I]` 上执行 Gauss–Jordan 消元。非方阵或某列找不到主元时返回 `Ok(None)`；成功结果为整数矩阵与公共分母 `(numerator, d)`，满足
\[
A^{-1}=\frac{\mathrm{numerator}}{d}.
\]
其中 \(d>0\) 依赖 `Rational` 的正分母约定，代码没有显式断言。^[alcove.md:166-169]

`root_vertex_simple` 使用系数全正的本原 coroot 关系，丢弃第一面系数为 1 的墙，以其余墙构造转置子 Cartan 矩阵。其条目为 `bracket(id(column), id(row))`，并传播 `bracket` 的错误；随后精确求逆，计算
\[
\mathrm{base}=C^{-T}\,\mathrm{floors}.
\]
缺少系数为 1 的墙或矩阵奇异时，分别报告相应的不变量错误。^[alcove.md:152-157]

整性通过公共分母的可除性检查判定。若初始 `base` 非整，算法依次将每面后续系数为 1 的墙的取值加 1 重试。第一个整数候选以 `entry / denominator` 为系数，对根坐标进行 checked 线性组合，再转为 `i32`；若所有候选均非整，则报告 `"alcove vertex lies outside the root lattice"`。这里的墙取值采用朴素有理下取整 `dot.div_euclid(denominator)`，不同于负根修正版 `floor_eval`。^[alcove.md:147-162]

## 通分与符号保留

重心解的各分量分母通过 `checked_lcm` 合并。放大后的有理数必须使用 `i64::try_from(&scaled)` 对整个值进行精确整数转换；代码注释指出，单独读取 `numerator_ref` 会把负的 alcove 中心变成正值。这一转换步骤同时关系到整数表示和符号保留。^[alcove.md:80-83]

## 证据范围

本页依据对 `alcove.rs` 的结构性阅读，不构成数学正确性验收。源文件中的上游行号来自代码注释，材料未独立核对上游字节；测试仅包含分母界边界与不自洽超定方程组用例，未覆盖 `labels_for_component`、`root_vertex_simple`、`rational_inverse` 或 `alcove_center` 端到端行为。^[alcove.md:9-13, alcove.md:60-61, alcove.md:171-176]

材料另将 `solve_rational_system` 对短行的越界访问和 `gcd(i64::MIN, 0)` 的 `abs()` 列为潜在 panic 路径；这些是阅读推断，文件内未见相应防护。^[alcove.md:177-180]

## Sources

- [alcove.md](alcove.md) — Alcove 几何：alcove_center 与 root_vertex_of_alcove

---
title: Alcove 算法中的精确有理线性代数
summary: 精确消元检查方程的唯一性、一致性与矩阵可逆性，结合 checked 运算、最小公倍数通分及整个有理数的精确整数转换控制溢出与负号丢失风险。
sources:
  - alcove.md
kind: concept
createdAt: "2026-10-09T14:25:09.328Z"
updatedAt: "2026-10-09T22:11:43.406Z"
tags:
  - linear-algebra
  - exact-arithmetic
aliases:
  - alcove-算法中的精确有理线性代数
confidence: 1
provenanceState: merged
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: Alcove 算法中的精确有理线性代数
summary: Alcove 算法通过精确有理消元求解重心方程、提取本原余根关系并计算根格顶点，以唯一性检查、公共分母整除检查和 checked 算术维护计算约束。
sources:
  - alcove.md
kind: concept
tags:
  - 线性代数
  - 精确算术
  - 错误处理
aliases:
  - alcove-算法中的精确有理线性代数
---

# Alcove 算法中的精确有理线性代数

Alcove 模块使用 `Rational` 矩阵消元、分母通分和整数可除性检查，计算重心、墙分量的本原 coroot（余根）关系及根格顶点。这些计算分别支撑 [[Alcove 重心计算与标准参数重建]]、[[墙分量的本原 Coroot 关系]] 和 [[Alcove 根格顶点与基本 Alcove 约化]]。^[alcove.md:63-89, alcove.md:136-169]

## 重心方程与唯一解

`alcove_center` 根据墙集与逐墙分数组装线性方程组。每面墙对应一行，系数为 coroot 坐标乘以 `scale`，右端为 `floor_eval_nbr * scale + fracs.0`。随后为 `datum.radical_basis()?` 的每个元素补充一行：系数乘以 `gamma.denominator()`，右端取该元素与 `gamma.numerator()` 的点积。相关整数运算采用 checked 算术，溢出报告 `ArithmeticOverflow`。^[alcove.md:70-76]

`solve_rational_system(rows, columns)` 将增广 `Rational` 矩阵消元至约化阶梯形。任一未知量列找不到主元，或消元后出现系数全为零而右端非零的行，都返回 `None`。调用方将解不唯一与方程不自洽统一转换为 `RepInvariantViolation`，错误信息为 `"alcove center equations have no unique solution"`。^[alcove.md:77-79, alcove.md:164-166]

## 本原余根关系

`labels_for_component` 以墙分量的各个 coroot 为列构造 `Rational` 矩阵，执行包含主元行上方消元的 Gauss–Jordan 全消元。自由列必须恰好有一个，否则返回 `RootSystemInvariantViolation`，错误信息为 `"alcove wall component must have one coroot relation"`。^[alcove.md:136-141]

关系向量在自由列处取 1，在主元列处取 `-matrix[pivot][free]`。随后用 `checked_lcm` 通分、转为 `i64`，再按全体坐标的 `gcd` 约化为本原整数向量；首元素为负时通过 `checked_neg` 将整体取负，首元素为零时不改变符号。^[alcove.md:141-143]

这些关系系数用于 [[墙连通分量与重心分数约束]]：`barycentre_eq` 将结果初始化为 `(0, 1)`，仅将非整值墙改写为 `(1, n_off * labels[position])`，整值墙保持初值。^[alcove.md:126-134]

## 精确求逆与根格整性

`rational_inverse` 在增广矩阵 `[A|I]` 上执行 Gauss–Jordan 消元。非方阵或找不到主元列时返回 `Ok(None)`；成功时返回整数矩阵与公共分母 `(numerator, d)`，满足 $A^{-1}=\mathrm{numerator}/d$。其中 $d>0$ 依赖 `Rational` 的正分母约定，代码未显式断言。^[alcove.md:166-169]

`root_vertex_simple` 使用系数全正的本原 coroot 关系，丢弃第一面系数为 1 的墙，以其余墙构造转置子 Cartan 矩阵。矩阵条目为 `bracket(id(column), id(row))`，此处通过 `?` 传播配对错误；求逆后计算 $\mathrm{base}=C^{-T}\,\mathrm{floors}$。缺少系数为 1 的墙时报 `"alcove component has no coefficient-1 wall"`，矩阵奇异时报 `"alcove generator Cartan matrix is singular"`。^[alcove.md:152-157]

算法通过各分量是否能被公共分母整除来判定 `base` 的整性。若初始结果非整，则依次将每面后续系数为 1 的墙的取值加 1 重试。首个整数候选以 `entry / denominator` 为系数，对根坐标进行 checked 线性组合，再转为 `i32`；全部候选非整时报 `"alcove vertex lies outside the root lattice"`。^[alcove.md:158-162]

这里的墙取值采用朴素有理下取整 `dot.div_euclid(denominator)`，并非负根修正版 `floor_eval`。`root_vertex_of_alcove` 显式忽略墙集计算返回的 `integrals`，逐分量求顶点后求和，使 `gamma - vertex` 落在基本 alcove 的 Weyl 轨道。^[alcove.md:145-150]

## 通分与符号保留

重心解的各分量分母通过 `checked_lcm` 合并。放大后的有理数使用 `i64::try_from(&scaled)` 对整个值进行精确整数转换；代码注释指出，单独读取 `numerator_ref` 会把负的 alcove 中心变成正值。因此，转换必须保留整个有理数的符号。^[alcove.md:80-83]

## 证据范围与限制

本页依据 `alcove.rs` 的结构性阅读，不构成数学正确性验收。材料中的上游行号转录自代码注释，未独立核对上游字节。测试模块仅包含分母界边界与不自洽超定方程组两个用例；`labels_for_component`、`root_vertex_simple`、`rational_inverse`、`checked_dot` 及 `alcove_center` 端到端行为均无单元测试覆盖。^[alcove.md:9-13, alcove.md:60-61, alcove.md:171-176]

来源另将 `solve_rational_system` 对短行的越界访问，以及 `gcd(i64::MIN, 0)` 中的 `abs()` 列为潜在 panic 路径。这些属于来源中的阅读推断，文件内未见相应防护。^[alcove.md:177-180]

## Sources

- [alcove.md](../../sources/alcove.md) — Alcove 几何：alcove_center 与 root_vertex_of_alcove。

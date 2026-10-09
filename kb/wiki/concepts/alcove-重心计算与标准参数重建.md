---
title: Alcove 重心计算与标准参数重建
summary: alcove_center 通过墙方程与 radical_basis 约束求唯一有理解，保留 KGB 坐标和 lambda_rho，并在修正子空间校验后通过 sr_gamma 重建参数；来源仅支持结构性说明，未作数学验收。
sources:
  - alcove.md
kind: concept
createdAt: "2026-10-09T14:24:20.323Z"
updatedAt: "2026-10-09T14:24:20.323Z"
tags:
  - alcove几何
  - 表示论
  - Rust实现
aliases:
  - alcove-重心计算与标准参数重建
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# Alcove 重心计算与标准参数重建

`alcove_center` 返回包含标准参数 `z.gamma()` 的 alcove 的重心所对应的标准参数。它保留 KGB 坐标与 `lambda_rho`，仅替换无穷小特征（infinitesimal character），服务于形变预处理与整值 datum 规范化。结果通过 `RepContext::sr_gamma` 重建，使打包扭数据与派生高度保持规范形式。^[alcove.md:17-24, alcove.md:63-66]

## 接口与计算顺序

接口为 `alcove_center(rc: &RepContext<'_>, z: &StandardRepr) -> Result<StandardRepr, StructureError>`，关联 [[RepContext 借用上下文与一致性约束]] 与 [[StandardRepr 标准表示参数]]。计算依次完成墙集选择、重心方程构造、精确求解与通分、修正量校验，最后重建参数。^[alcove.md:28-30, alcove.md:68-89]

### 墙集与重心分数

首先构造 `RootNumbering`，调用 `wall_set` 得到墙集 `walls` 与整值墙集 `integrals`，再由 `barycentre_eq` 计算每面墙对应的分数。墙的连通分量由非零根配对关系确定；整值墙的分数保持 `(0, 1)`，非整值墙则设为 `(1, n_off * labels[position])`。相关概念见 [[Alcove 墙集与整值墙筛选]] 与 [[墙连通分量与重心分数约束]]。^[alcove.md:70-71, alcove.md:126-134]

这里的 `labels` 来自各分量 coroot 的本原关系：以 coroot 为列建立有理矩阵并全消元，要求恰有一个自由列，再将核向量通分、用全体系数的最大公因数约化，并在首元素为负时整体取负。自由列数量不符时返回 `RootSystemInvariantViolation`。详见 [[墙分量的本原 Coroot 关系]]。^[alcove.md:136-143]

### 线性方程组

每面墙贡献一行方程：coroot 各坐标乘以分数的分母 `scale` 作为系数，右端为 `floor_eval_nbr * scale + fracs.0`。这些整数乘加均使用 checked 运算，溢出返回 `ArithmeticOverflow`。随后对 `datum.radical_basis()?` 的每个元素添加一行：系数乘以原 `gamma` 的分母，右端取该元素与 `gamma` 分子的 checked 点积。^[alcove.md:72-76]

`solve_rational_system(&rows, rank)` 使用精确有理数消元，要求解唯一且方程自洽。任一列找不到主元，或消元后出现系数全零而右端非零的行，均返回 `None`；调用方将其转为 `RepInvariantViolation`，错误文本为 `"alcove center equations have no unique solution"`。相关机制见 [[Alcove 算法中的精确有理线性代数]]。^[alcove.md:77-79, alcove.md:164-166]

### 通分与符号保留

求得有理解后，通过 `checked_lcm` 求各分量分母的公分母。每个分量乘以公分母后，使用 `i64::try_from(&scaled)` 对整个有理数进行精确整数转换。源码注释特别指出，仅使用 `numerator_ref` 会把负的 alcove 中心变为正值，因此此处的转换方式承担符号保留职责。^[alcove.md:80-83]

## 修正量校验与参数重建

通分得到 `centered_gamma` 后，算法调用 `rc.theta(z)?` 获取对合，并逐行检查 `theta.weight_matrix()` 与 `centered_gamma - gamma` 的乘积是否为零。源材料将此步骤标记为“−θ 不动子空间校验”；失败时返回 `RepInvariantViolation`，错误文本为 `"alcove correction lies outside the -theta fixed subspace"`。^[alcove.md:84-88]

最后调用 `rc.sr_gamma(z.x(), &lambda_rho, &centered_gamma)`。这一重建步骤保留原参数的 KGB 坐标与 `lambda_rho`，同时使与新无穷小特征相关的打包扭数据和派生高度保持规范形式。^[alcove.md:65-66, alcove.md:89-89]

## 证据与覆盖边界

本说明依据对 `alcove.rs` 的结构性阅读，不代表数学正确性验收。材料中的上游 C++ 行号来自源码注释，未独立核对上游字节；`RepContext`、`StandardRepr` 等外部类型的完整契约也不在该材料范围内。^[alcove.md:9-13, alcove.md:185-190]

该模块仅有分母界边界与不自洽超定方程组两个单元测试；墙集、连通分量、重心分数、关系标签及 `alcove_center` 端到端均缺少单元测试。材料还记录了潜在下标越界等 panic 路径，因此不能将 `Result` 返回类型理解为所有失败均已转化为结构化错误。^[alcove.md:60-61, alcove.md:173-180]

## Sources

- [alcove.md](../../sources/alcove.md) — Alcove 几何：alcove_center 与 root_vertex_of_alcove。

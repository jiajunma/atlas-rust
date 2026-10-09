---
title: Alcove 重心计算与标准参数重建
summary: alcove_center 联立墙方程与 radical_basis 约束求唯一有理解，经通分及修正校验后用 sr_gamma 重建参数，保留 KGB 坐标和 lambda_rho；本来源不构成数学验收。
sources:
  - alcove.md
kind: concept
createdAt: "2026-10-09T14:24:20.323Z"
updatedAt: "2026-10-09T20:28:44.170Z"
tags:
  - Alcove几何
  - 表示参数
  - 精确线性代数
aliases:
  - alcove-重心计算与标准参数重建
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: Alcove 重心计算与标准参数重建
summary: alcove_center 通过墙方程与 radical_basis 约束求唯一有理解，保留 KGB 坐标和 lambda_rho，经修正量校验后通过 sr_gamma 重建标准参数；来源仅支持结构性说明，未作数学验收。
sources:
  - alcove.md
kind: concept
tags:
  - alcove几何
  - 表示论
  - Rust实现
aliases:
  - alcove-重心计算与标准参数重建
---

# Alcove 重心计算与标准参数重建

`alcove_center` 返回包含 `z.gamma()` 的 alcove 的重心所对应的标准参数，用于形变预处理与整值 datum 规范化。它保留原参数的 KGB 坐标与 `lambda_rho`，仅替换无穷小特征（infinitesimal character），并通过 `RepContext::sr_gamma` 重建，使打包扭数据与派生高度保持规范形式。^[alcove.md:17-24, alcove.md:63-66]

## 接口与计算顺序

函数签名为 `alcove_center(rc: &RepContext<'_>, z: &StandardRepr) -> Result<StandardRepr, StructureError>`，涉及 [[RepContext 借用上下文与一致性约束]] 与 [[StandardRepr 标准表示参数]]。计算顺序固定为：选墙、构造重心方程、求唯一有理解、通分、校验修正量，最后重建参数。^[alcove.md:28-30, alcove.md:68-89]

### 墙集与重心分数

首先构造 `RootNumbering`，调用 `wall_set` 得到墙集 `walls` 与整值墙集 `integrals`，再由 `barycentre_eq` 计算逐墙分数。`root_components` 按 `bracket(α, β) != 0` 合并墙，分量按根首次出现的顺序排列，分量内部按编号升序。整值墙的分数保持 `(0, 1)`，非整值墙则设为 `(1, n_off * labels[position])`。相关概念见 [[Alcove 墙集与整值墙筛选]] 与 [[墙连通分量与重心分数约束]]。^[alcove.md:70-71, alcove.md:128-134]

`labels` 来自墙分量的本原 coroot 关系：以各墙 coroot 为列建立有理矩阵，进行 Gauss–Jordan 全消元，要求恰有一个自由列。关系向量通分并按全体系数的最大公因数约化；首元素为负时整体取负。自由列数量不符时返回 `RootSystemInvariantViolation`。详见 [[墙分量的本原 Coroot 关系]]。^[alcove.md:136-143]

### 方程构造与唯一解

每面墙贡献一行方程：coroot 各坐标乘以逐墙分数的分母 `scale` 作为系数，右端为 `floor_eval_nbr * scale + fracs.0`。这些整数乘加使用 checked 运算，溢出返回 `ArithmeticOverflow`。随后，对 `datum.radical_basis()?` 的每个元素追加一行：系数乘以原 `gamma` 的分母，右端取该元素与 `gamma` 分子的 checked 点积。^[alcove.md:72-76]

`solve_rational_system(&rows, rank)` 对增广有理矩阵消元至约化阶梯形。任一列找不到主元，或消元后出现系数全零而右端非零的行，都返回 `None`。调用方将解不唯一与方程不自洽统一转为 `RepInvariantViolation`，错误文本为 `"alcove center equations have no unique solution"`。^[alcove.md:77-79, alcove.md:164-166]

### 通分与符号保留

求得有理解后，算法通过 `checked_lcm` 计算各分量分母的公分母。每个分量乘以公分母后，使用 `i64::try_from(&scaled)` 对整个有理数进行精确整数转换。源码注释指出，仅使用 `numerator_ref` 会把负的 alcove 中心变为正值，因此这一转换方式用于保留符号。^[alcove.md:80-83]

## 修正量校验与参数重建

得到 `centered_gamma` 后，算法调用 `rc.theta(z)?`，逐行验证 `theta.weight_matrix()` 与 `centered_gamma - gamma` 的乘积为零。源材料将此步骤称为“−θ 不动子空间校验”；失败时返回 `RepInvariantViolation`，错误文本为 `"alcove correction lies outside the -theta fixed subspace"`。这里保留材料记录的具体检查条件与命名。^[alcove.md:84-88]

最后调用 `rc.sr_gamma(z.x(), &lambda_rho, &centered_gamma)`，以原 KGB 坐标、原 `lambda_rho` 和新的无穷小特征重建标准参数，同时保持打包扭数据与派生高度的规范形式。^[alcove.md:65-66, alcove.md:89-89]

## 证据与覆盖边界

本说明依据 `crates/atlas-real-group/src/alcove.rs` 的结构性阅读，不构成 alcove 计算的数学验收。源材料中的上游 C++ 行号转录自代码注释，未独立核对上游字节；`RepContext`、`StandardRepr`、`RationalWeight` 与 `RootSystem` 等外部类型的完整契约不在该材料范围内。^[alcove.md:9-13, alcove.md:185-190]

模块仅有分母界边界与不自洽超定方程组两个单元测试。墙集、连通分量、重心分数、关系标签、`checked_dot` 及 `alcove_center` 端到端均缺少单元测试；分母边界的具体覆盖见 [[Alcove 分母界守卫]]。^[alcove.md:60-61, alcove.md:93-97, alcove.md:173-176]

材料还记录了潜在 panic 路径，包括根编号映射缺键、编号越界、`difference.numerator()[row]` 越界及求解器访问短行。部分辅助函数对失败采用静默兜底，例如 `root_components` 将失败的 `bracket` 当作零处理；这些路径与显式错误传播并存，是否刻意尚未确认。^[alcove.md:128-131, alcove.md:177-181]

## Sources

- [alcove.md](../../sources/alcove.md) — Alcove 几何：alcove_center 与 root_vertex_of_alcove。

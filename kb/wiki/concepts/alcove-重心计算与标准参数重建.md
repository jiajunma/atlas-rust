---
title: Alcove 重心计算与标准参数重建
summary: alcove_center 联立墙分数与 radical_basis 约束求唯一有理解，经精确通分和 sr_gamma 重建参数，保留 KGB 坐标与 lambda_rho；本包仅提供结构性阅读证据。
sources:
  - alcove.md
kind: concept
createdAt: "2026-10-09T14:24:20.323Z"
updatedAt: "2026-10-10T00:13:12.222Z"
tags:
  - alcove
  - 精确线性代数
  - 表示参数
aliases:
  - alcove-重心计算与标准参数重建
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: Alcove 重心计算与标准参数重建
summary: alcove_center 联立墙分数方程与 radical_basis 约束求唯一有理解，经通分和修正量校验后重建标准参数，保留 KGB 坐标与 lambda_rho；证据限于结构性阅读。
sources:
  - alcove.md
kind: concept
tags:
  - alcove-geometry
  - representation-theory
aliases:
  - alcove-重心计算与标准参数重建
---

# Alcove 重心计算与标准参数重建

`alcove_center` 返回包含 `z.gamma()` 的 alcove 的重心所对应的标准参数，用于形变预处理与整值 datum 规范化。它保留原参数的 KGB 坐标和 `lambda_rho`，仅替换无穷小特征（infinitesimal character），再通过 `RepContext::sr_gamma` 重建，使打包扭数据与派生高度保持规范形式。^[alcove.md:17-24, alcove.md:65-66]

## 接口与计算流程

函数接口为 `alcove_center(rc: &RepContext<'_>, z: &StandardRepr) -> Result<StandardRepr, StructureError>`，涉及 [[RepContext 借用上下文与一致性约束]] 和 [[StandardRepr 标准表示参数]]。计算顺序固定为：选择墙集、构造重心方程、求唯一有理解、通分、校验修正量，最后重建参数。^[alcove.md:28-30, alcove.md:68-89]

### 墙集与逐墙分数

算法先构造 `RootNumbering`，再由 `wall_set` 得到墙集 `walls` 和整值墙集 `integrals`，随后调用 `barycentre_eq` 计算逐墙分数。整值墙始终属于墙集；墙的选择规则可结合 [[Alcove 墙集与整值墙筛选]] 阅读。^[alcove.md:70-71, alcove.md:119-124]

`root_components` 按 `bracket(α, β) != 0` 合并墙，分量按根首次出现顺序追加，分量内部按编号升序排列。`barycentre_eq` 将所有分数初始化为 `(0, 1)`，仅把非整值墙改写为 `(1, n_off * labels[position])`；整值墙保持零分数。^[alcove.md:128-134]

`labels_for_component` 以分量各墙的余根为列构造有理矩阵，经 Gauss–Jordan 全消元求关系向量。自由列必须恰好一个，否则返回 `RootSystemInvariantViolation`，错误文本为 `"alcove wall component must have one coroot relation"`。关系向量取自由列系数为 1、主元列系数为 `-matrix[pivot][free]`，再通分、转为 `i64` 并按全体系数的最大公因数约化；首元素为负时整体取负，首元素为零时不取负。^[alcove.md:136-143]

### 方程构造与唯一解

每面墙贡献一行方程：系数为余根各坐标乘以 `scale`，右端为 `floor_eval_nbr * scale + fracs.0`。这些整数乘加使用 checked 运算，溢出返回 `ArithmeticOverflow`。随后，对 `datum.radical_basis()?` 的每个元素追加一行：系数乘以原 `gamma` 的分母，右端为该元素与 `gamma` 分子的 checked 点积。^[alcove.md:72-76]

`solve_rational_system(&rows, rank)` 将增广有理矩阵消元至约化阶梯形。任一列找不到主元，或消元后存在系数全零而右端非零的行，都会返回 `None`。调用方将解不唯一与方程不自洽统一报告为 `RepInvariantViolation`，错误文本为 `"alcove center equations have no unique solution"`。^[alcove.md:77-79, alcove.md:164-166]

### 通分与符号保留

求得有理解后，算法通过 `checked_lcm` 求各分量分母的公倍数。每个分量乘以公分母后，使用 `i64::try_from(&scaled)` 对整个有理数进行精确整数转换。源码注释明确指出，单用 `numerator_ref` 会把负的 alcove 中心变正，因此此处转换必须保留整个有理数的符号。^[alcove.md:80-83]

## 修正量校验与参数重建

得到 `centered_gamma` 后，算法调用 `rc.theta(z)?`，逐行检查 `theta.weight_matrix()` 与 `centered_gamma - gamma` 的乘积是否为零。来源将此步骤称为“−θ 不动子空间校验”；失败时返回 `RepInvariantViolation`，错误文本为 `"alcove correction lies outside the -theta fixed subspace"`。应区分这一命名与来源记录的实际检查条件，相关问题见 [[Alcove 修正的负对合不动子空间校验歧义]]。^[alcove.md:84-88]

最后调用 `rc.sr_gamma(z.x(), &lambda_rho, &centered_gamma)`，使用原 KGB 坐标、原 `lambda_rho` 和新的无穷小特征重建标准参数，使打包扭数据与派生高度保持规范形式。^[alcove.md:65-66, alcove.md:89-89]

## 失败边界与证据范围

除显式错误外，来源还列出阅读推断的潜在 panic 路径，包括根编号映射缺键、编号越界、`difference.numerator()[row]` 越界、求解器访问短行，以及 `gcd(i64::MIN, 0)` 的 `abs()`。辅助函数存在静默兜底，例如 `root_components` 将失败的 `bracket` 视为零，即视为不相连；这些处理与显式错误传播并存，是否刻意尚未确认。^[alcove.md:128-131, alcove.md:177-181]

模块仅包含分母界边界和不自洽超定方程组两个单元测试。`wall_set`、`root_components`、`barycentre_eq`、`labels_for_component`、`checked_dot` 以及 `alcove_center` 端到端均无单元测试，参见 [[Alcove 算法的测试覆盖与失败边界]]。^[alcove.md:60-61, alcove.md:173-176]

本页依据 `crates/atlas-real-group/src/alcove.rs` 的结构性阅读，不构成 alcove 计算的数学验收。上游行号仅转录自代码注释，未独立核对上游字节；`RepContext`、`StandardRepr`、`RationalWeight` 和 `RootSystem` 等外部类型契约不在来源范围内。本次知识维护未执行 Atlas、Cargo、测试或 benchmark。^[alcove.md:9-13, alcove.md:185-190]

## Sources

- [alcove.md](../../sources/alcove.md) — Alcove 几何：alcove_center 与 root_vertex_of_alcove。

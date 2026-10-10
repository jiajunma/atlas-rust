---
title: Alcove 算法的测试覆盖与失败边界
summary: 本包仅记录分母界与不自洽方程组两个测试，主要几何路径缺少单元测试，且存在静默兜底与潜在下标或 abs 溢出路径；未执行测试或完成数学验收。
sources:
  - alcove.md
kind: concept
createdAt: "2026-10-10T00:13:46.097Z"
updatedAt: "2026-10-10T00:13:46.097Z"
tags:
  - alcove
  - 测试覆盖
  - 错误处理
  - 证据边界
aliases:
  - alcove-算法的测试覆盖与失败边界
confidence: 1
provenanceState: merged
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

# Alcove 算法的测试覆盖与失败边界

Alcove 模块的测试仅覆盖分母界边界与不自洽超定方程组两个用例，主要几何算法仍缺少单元测试。其失败行为包括显式错误、求解器返回无解标记、潜在 panic，以及部分查询失败时的静默兜底。现有材料属于结构性源码阅读，不构成数学正确性验收。^[alcove.md:9-13, alcove.md:60-61, alcove.md:171-181]

## 已有测试与覆盖缺口

分母守卫 `denominator_exceeds_alcove_bound` 判断分母是否严格超过 $2^{\mathrm{rank}}$。测试固定了 `rank = 62` 时 $2^{62}$ 与 $2^{62}+1$ 的边界，并验证 `rank = 63`、`64` 时，即使分母为 `i64::MAX` 也返回 `false`。秩至少为 63 时，正阈值超出 `i64` 范围；提前返回避免了有符号移位产生错误阈值。^[alcove.md:91-97]

另一项测试针对不自洽超定方程组。被测求解器 `solve_rational_system` 使用有理数 Gauss 消元：任一未知量列找不到主元，或消元后出现系数全零而右端非零的行，均返回 `None`。现有测试清单未表明这些失败路径已全部覆盖。^[alcove.md:60-61, alcove.md:164-166]

来源明确列出的测试缺口包括 `wall_set`、`root_components`、`barycentre_eq`、`labels_for_component`、`root_vertex_simple`、`rational_inverse`、`checked_dot`，以及 `alcove_center` 的端到端行为。相关算法可参见 [[Alcove 墙集与整值墙选择]]、[[Alcove 根格顶点与基本 Alcove 约化]] 和 [[Alcove 重心计算与标准参数重建]]。^[alcove.md:174-176]

## 显式失败与错误传播

### 重心计算

`alcove_center` 组装墙方程与 radical 约束时使用 checked 算术，溢出返回 `ArithmeticOverflow`。求解器返回 `None` 时，统一转换为 `RepInvariantViolation`，错误文本为 `"alcove center equations have no unique solution"`；不自洽与解不唯一共享这一错误。通分使用 `checked_lcm`，并对完整有理数作精确整数转换，以保留负数符号。^[alcove.md:70-83]

重心求出后，代码逐行检查 `theta.weight_matrix()` 与 `centered_gamma - gamma` 的乘积是否为零；失败返回 `RepInvariantViolation`，文本为 `"alcove correction lies outside the -theta fixed subspace"`。来源将这一步称为“−θ 不动子空间校验”，其命名与所记录检查式之间的歧义参见 [[Alcove 修正的负对合不动子空间校验歧义]]。^[alcove.md:84-88]

### 墙关系与根格顶点

`labels_for_component` 将墙的余根作为矩阵列，要求消元后恰有一个自由列。否则返回 `RootSystemInvariantViolation`，文本为 `"alcove wall component must have one coroot relation"`。关系向量随后经过分母通分、`i64` 转换、最大公约数约化及必要的 checked 取负。^[alcove.md:136-143]

`root_vertex_simple` 的失败边界依次涉及：不存在系数为 1 的墙、生成元 Cartan 矩阵奇异，以及全部候选顶点均不在根格内。对应错误文本分别为 `"alcove component has no coefficient-1 wall"`、`"alcove generator Cartan matrix is singular"` 和 `"alcove vertex lies outside the root lattice"`。构造转置 Cartan 矩阵时，`bracket` 查询错误通过 `?` 传播；成功候选以 checked 根坐标线性组合构造，并转换为 `i32`。^[alcove.md:152-162]

`rational_inverse` 对非方阵或缺少主元列返回 `Ok(None)`。成功结果以整数分子矩阵和公共分母表示逆矩阵；分母为正依赖 `Rational` 的分母约定，代码未显式断言该条件。^[alcove.md:166-169]

## 潜在 panic 与静默兜底

来源将若干未受文件内防护的路径标为阅读推断：`RootNumbering::new` 查找负根对应正根时缺键、`id()` 下标越界、`alcove_center` 访问 `difference.numerator()[row]` 越界、求解器访问短行越界，以及 `gcd(i64::MIN, 0)` 中的 `abs()`。这些是潜在失败点，并非已执行测试确认的故障。^[alcove.md:177-180]

根编号构造器返回 `Self`，没有可恢复错误返回值；它对 `is_positive` 和 `simple_coordinates` 的查询失败使用默认值，并隐含根总数等于正根数两倍的假设。负根匹配缺键及编号越界则直接触发下标 panic，详见 [[RootNumbering 根编号与 RootNbr 顺序]]。^[alcove.md:101-110]

静默兜底也存在于几何辅助函数：`wall_set` 构造余根表时跳过缺失余根的根；`root_components` 将失败的 `bracket` 查询视为零，即不建立连接，而且不返回 `Result`。这与 `root_vertex_simple` 显式传播 `bracket` 错误的行为不同；来源未确认这种差异是否刻意设计。^[alcove.md:114-115, alcove.md:128-131, alcove.md:154-156, alcove.md:181]

## 证据范围

本页依据对 `crates/atlas-real-group/src/alcove.rs` 的结构性阅读。上游 C++ 行号仅转录自代码注释，未独立核对上游字节；`RepContext`、`StandardRepr`、`RationalWeight` 和 `RootSystem` 等外部类型契约不在来源包范围内。本次知识维护未执行 Atlas、Cargo、测试或 benchmark，因此测试清单和失败路径分析均不能表述为本次运行通过或数学验收结果。^[alcove.md:9-13, alcove.md:185-190]

## Sources

- [alcove.md](../../sources/alcove.md) — Alcove 几何：alcove_center 与 root_vertex_of_alcove。

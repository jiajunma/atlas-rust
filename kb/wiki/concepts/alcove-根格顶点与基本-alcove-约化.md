---
title: Alcove 根格顶点与基本 Alcove 约化
summary: 逐墙分量使用朴素有理下取整、转置 Cartan 矩阵求逆及系数为 1 的墙重试整性，以构造用于基本 alcove 约化的根格顶点。
sources:
  - alcove.md
  - atlas-core-domain-seams.md
kind: concept
createdAt: "2026-10-09T14:24:58.442Z"
updatedAt: "2026-10-10T00:13:40.612Z"
tags:
  - alcove
  - 根格
  - 精确线性代数
aliases:
  - alcove-根格顶点与基本-alcove-约化
  - A根A约
confidence: 1
provenanceState: merged
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: Alcove 根格顶点与基本 Alcove 约化
summary: 逐墙分量使用朴素有理下取整、转置 Cartan 矩阵精确求逆及标签为 1 的墙重试整性，构造用于基本 alcove 约化的根格顶点。
sources:
  - alcove.md
  - atlas-core-domain-seams.md
kind: concept
tags:
  - alcove-geometry
  - integer-lattices
aliases:
  - alcove-根格顶点与基本-alcove-约化
provenanceState: extracted
---

# Alcove 根格顶点与基本 Alcove 约化

`root_vertex_of_alcove` 为有理权 `gamma` 计算根格顶点 `vertex`，使 `gamma - vertex` 落在基本 alcove 的 Weyl 轨道中。它服务于 locator 切片的基本 alcove 约化，是 `atlas-real-group` 内部可见的辅助函数，返回 `Result<Weight, StructureError>`。^[alcove.md:17-24, alcove.md:44-45, alcove.md:147-150]

## 墙分量与取整约定

算法使用 `wall_set` 返回的墙集，**显式忽略整值墙集合 `integrals`**，逐分量计算顶点后求和。`root_components` 通过并查集合并满足 `bracket(α, β) != 0` 的墙；分量按根首次出现顺序追加，分量内部按根编号升序排列。墙集构造参见 [[Alcove 墙集与整值墙筛选]]。^[alcove.md:128-131, alcove.md:147-150]

每面墙的取值采用朴素有理下取整 `dot.div_euclid(denominator)`。源码注释明确区分了这一计算与对负根作修正的 `floor_eval`；顶点算法使用前者。^[alcove.md:147-150]

## 单分量顶点构造

`root_vertex_simple` 使用墙余根之间系数全正的本原关系。`labels_for_component` 以各墙余根为列构造有理矩阵，进行 Gauss–Jordan 全消元，要求恰有一个自由列。关系向量将自由列系数设为 1，主元列系数取相应消元条目的相反数，再经最小公倍数通分、转换为 `i64`、最大公约数约化；首元素为负时整体取负。^[alcove.md:138-143, alcove.md:152-154]

算法丢弃分量中**第一面关系系数为 1 的墙**，由其余墙构造转置子 Cartan 矩阵：
`transposed[row][column] = bracket(id(column), id(row))`。
此处 `bracket` 的错误通过 `?` 传播。求逆后，初始候选系数向量为 \(\mathrm{base}=C^{-T}\mathrm{floors}\)，其中 `floors` 为保留墙的朴素下取整值。^[alcove.md:147-157]

候选以整数分子和公共分母表示。若有分量不能被分母整除，算法依次尝试将每面后续关系系数为 1 的墙的取值加 1，重新检查整性。首个全部为整数的候选以 `entry / denominator` 为系数，对**根坐标**进行带溢出检查的线性组合，最后转换为 `i32` 返回。因此，关系计算使用余根，最终顶点组合使用根。^[alcove.md:152-162]

## 精确求逆与失败行为

`rational_inverse` 在增广矩阵 `[A|I]` 上执行 Gauss–Jordan 消元，返回整数分子矩阵与公共分母 `d`，满足 `inverse = numerator / d`。非方阵或缺少主元时返回 `Ok(None)`；`d > 0` 依赖 `Rational` 的正分母约定，代码没有显式断言。^[alcove.md:164-169]

墙关系计算若不能得到恰好一个自由列，返回 `RootSystemInvariantViolation`，诊断为 `"alcove wall component must have one coroot relation"`。单分量顶点构造还明确区分三种失败：不存在系数为 1 的墙、转置子 Cartan 矩阵奇异，以及全部候选均不在根格中，对应诊断分别为 `"alcove component has no coefficient-1 wall"`、`"alcove generator Cartan matrix is singular"` 和 `"alcove vertex lies outside the root lattice"`。^[alcove.md:138-143, alcove.md:152-162]

不同阶段的错误处理并不一致：分量划分用 `unwrap_or(0)` 将 `bracket` 失败视为不相连，单分量矩阵构造则传播该错误。来源将静默兜底与显式报错并存记录为阅读观察，未确认其是否刻意设计。^[alcove.md:128-131, alcove.md:154-156, alcove.md:181-181]

## 两份移植的对应关系

`atlas-core/src/domain_builtins.rs` 也包含 `root_vertex_simple`，唯一调用方是 `"alcove_root_vertex"` 派发臂。该版本同样剔除第一面标签为 1 的墙，用 `inverse_cartan` 求转置子 Cartan 矩阵的精确逆，先检查未移位解，再对其余标签为 1 的墙逐一重试；全部失败时报告 `"no root lattice vertex found for alcove component"`，成功时将系数乘根向量累加到结果。^[atlas-core-domain-seams.md:63-79]

两份实现的转置构造逐元相同，标签为 1 的墙的重试逻辑也等价；`atlas-core` 版本省略 `index > chosen` 检查，是因为所选墙已经是第一面标签为 1 的墙。差异在于错误通道（`Result<_, String>` 与 `StructureError`）、预算纪律（无与 `try_reserve_exact`），以及 `bracket` 失败处理（`unwrap_or(0)` 与 `?` 传播）。这些差异被记录为漂移风险，详见 [[Alcove 根格顶点双重移植的语义漂移风险]]。^[atlas-core-domain-seams.md:70-79]

## 证据范围

本页依据两份 Rust 实现的结构性阅读记录，不构成数学正确性或语言兼容性验收。两处源码注释引用的上游 `alcoves.cpp` 行号略有不同，来源均未独立核对上游字节；两份实现的并存与差异是阅读观察，不是缺陷判定。^[alcove.md:9-13, atlas-core-domain-seams.md:70-72, atlas-core-domain-seams.md:113-118]

`alcove.rs` 的单元测试仅覆盖分母界边界和不自洽超定方程组；`wall_set`、`root_components`、`labels_for_component`、`root_vertex_simple` 与 `rational_inverse` 等相关路径没有单元测试覆盖。该来源记录的知识维护未执行 Atlas、Cargo、测试或 benchmark，参见 [[Alcove 算法的测试覆盖与失败边界]]。^[alcove.md:60-61, alcove.md:173-176, alcove.md:185-190]

## Sources

- [alcove.md](../../sources/alcove.md) — Alcove 几何：alcove_center 与 root_vertex_of_alcove。
- [atlas-core-domain-seams.md](../../sources/atlas-core-domain-seams.md) — 领域层接缝：值提取器、alcove 助手与 Weyl 词/生成元校验。

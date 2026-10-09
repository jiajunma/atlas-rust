---
title: Alcove 根格顶点与基本 Alcove 约化
summary: 逐墙分量使用朴素有理下取整、转置 Cartan 矩阵求逆及系数为 1 的墙重试整性，构造根格顶点，使平移后的 gamma 落在基本 alcove 的 Weyl 轨道。
sources:
  - alcove.md
kind: concept
createdAt: "2026-10-09T14:24:58.442Z"
updatedAt: "2026-10-09T22:11:28.218Z"
tags:
  - alcove-geometry
  - integer-lattices
aliases:
  - alcove-根格顶点与基本-alcove-约化
  - A根A约
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: Alcove 根格顶点与基本 Alcove 约化
summary: root_vertex_of_alcove 逐分量使用朴素有理下取整、转置 Cartan 矩阵求逆及单位标签墙的整性重试，构造用于基本 alcove 约化的根格顶点。
sources:
  - alcove.md
kind: concept
tags:
  - Alcove几何
  - 根格
  - Cartan矩阵
aliases:
  - alcove-根格顶点与基本-alcove-约化
provenanceState: extracted
---

# Alcove 根格顶点与基本 Alcove 约化

`root_vertex_of_alcove` 为有理权 `gamma` 计算根格顶点 `vertex`，使 `gamma - vertex` 落在基本 alcove 的 Weyl 轨道中。它服务于 locator 切片的基本 alcove 约化，是 crate 内可见的辅助函数，返回 `Result<Weight, StructureError>`。^[alcove.md:17-24, alcove.md:44-45, alcove.md:147-150]

## 墙集、分量与取整约定

算法使用 `wall_set` 的墙集，显式忽略其返回的整值墙集合 `integrals`，逐分量计算顶点后求和。`root_components` 使用并查集，在 `bracket(α, β) != 0` 时合并墙；分量按根首次出现顺序追加，分量内部按根编号升序排列。墙集的构造见 [[Alcove 墙集与整值墙筛选]]。^[alcove.md:128-131, alcove.md:147-150]

每面墙的取值采用朴素有理下取整 `dot.div_euclid(denominator)`。源码注释明确指出，此处不使用对负根作修正的 `floor_eval`；这一差别是顶点算法需要保留的取整约定。^[alcove.md:147-150]

## 单分量顶点构造

`root_vertex_simple` 使用墙余根之间系数全正的本原关系。关系计算以各墙余根为矩阵列，在有理数上进行 Gauss–Jordan 全消元，要求恰有一个自由列；关系向量随后经最小公倍数通分、转为 `i64`、最大公约数约化，并在首元素为负时整体取负。详见 [[墙分量的本原 Coroot 关系]]。^[alcove.md:138-143, alcove.md:152-154]

算法先丢弃分量中第一面关系系数为 1 的墙，再由其余墙构造转置子 Cartan 矩阵，具体为 `transposed[row][column] = bracket(id(column), id(row))`。此处 `bracket` 的错误通过 `?` 传播。求逆后，候选系数向量为 \(\mathrm{base}=C^{-T}\mathrm{floors}\)，其中 `floors` 为墙的朴素下取整值。^[alcove.md:147-157]

候选系数以整数分子和公共分母表示。若有分量不能被分母整除，算法依次尝试将每面后续关系系数为 1 的墙的取值加 1，重新检查整性。第一个全部为整数的候选以 `entry / denominator` 为系数，对**根坐标**进行带溢出检查的线性组合，最后转换为 `i32` 返回。因此，关系计算使用余根，最终顶点组合使用根。^[alcove.md:152-162]

## 精确求逆与失败行为

`rational_inverse` 在增广矩阵 `[A|I]` 上执行 Gauss–Jordan 消元，返回整数分子矩阵与公共分母 `d`，满足 `inverse = numerator / d`。非方阵或缺少主元时返回 `Ok(None)`；`d > 0` 依赖 `Rational` 的正分母约定，代码没有显式断言。相关主题见 [[Alcove 算法中的精确有理线性代数]]。^[alcove.md:164-169]

墙分量关系计算若不能得到恰好一个自由列，返回 `RootSystemInvariantViolation`，诊断为 `"alcove wall component must have one coroot relation"`。^[alcove.md:138-143]

单分量构造包含三个明确失败条件：没有系数为 1 的墙时报告 `"alcove component has no coefficient-1 wall"`；转置子 Cartan 矩阵奇异时报告 `"alcove generator Cartan matrix is singular"`；全部候选均非整时报告 `"alcove vertex lies outside the root lattice"`。^[alcove.md:152-162]

错误处理在不同阶段并不相同：分量划分将 `bracket` 失败通过 `unwrap_or(0)` 视为不相连，而单分量矩阵构造传播该错误。来源将静默兜底与显式报错并存记录为阅读观察，未确认其是否刻意设计。^[alcove.md:128-131, alcove.md:154-156, alcove.md:181-181]

## 证据范围

本页依据 `crates/atlas-real-group/src/alcove.rs` 的结构性阅读记录，不构成 alcove 计算的数学正确性验收。来源中的上游 C++ 行号仅转录自代码注释，未独立核对上游字节。^[alcove.md:9-13]

来源记录的单元测试仅覆盖分母界边界和不自洽超定方程组；`wall_set`、`root_components`、`labels_for_component`、`root_vertex_simple` 与 `rational_inverse` 等相关路径没有单元测试覆盖。本次知识维护也未执行 Atlas、Cargo、测试或 benchmark。^[alcove.md:60-61, alcove.md:173-176, alcove.md:185-190]

## Sources

- [alcove.md](../../sources/alcove.md) — Alcove 几何：alcove_center 与 root_vertex_of_alcove。

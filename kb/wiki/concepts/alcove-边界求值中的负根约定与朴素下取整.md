---
title: Alcove 边界求值中的负根约定与朴素下取整
summary: 墙筛选将负根的整值余数记为分母，而根格顶点构造显式采用朴素有理下取整并忽略整值墙集合，两条路径的边界约定不可混用。
sources:
  - alcove.md
kind: concept
createdAt: "2026-10-10T01:44:03.607Z"
updatedAt: "2026-10-10T01:44:03.607Z"
tags:
  - Alcove几何
  - 边界语义
  - 精确算术
aliases:
  - alcove-边界求值中的负根约定与朴素下取整
confidence: 1
provenanceState: merged
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# Alcove 边界求值中的负根约定与朴素下取整

Alcove 计算在不同步骤采用不同的边界求值约定：`wall_set` 在负根取整值时调整余数，而 `root_vertex_of_alcove` 明确使用朴素有理下取整。区分这两种规则，是理解墙选择与根格顶点计算的关键。^[alcove.md:120-132, alcove.md:153-158]

## 墙选择中的负根整值约定

`wall_set` 通过 `frac_eval_value` 为每个根计算 `(remainder, denominator)`。通常余数由 `rem_euclid` 得到，但负根在求值为整数时，会将余数从 `0` 改写为 `denominator`；正根整值时仍取 `0`。因此，正负根在整数边界上具有不同的余数表示。^[alcove.md:124-126]

墙选择过程中，被选中的根都会加入 `walls`，只有余数为 `0` 的根才同时加入 `integrals`，始终满足 `integrals ⊆ walls`。负根整值时的余数改写因而直接影响整值墙的判定。墙的筛选还会移除满足余根差条件的候选根，详见 [[Alcove 墙集与整值墙筛选]]。^[alcove.md:127-132]

这一分类也用于重心约束：`barycentre_eq` 将所有墙的分数初值设为 `(0, 1)`，仅改写非整值墙的分数，整值墙保持原值。相关流程见 [[墙连通分量与重心分数约束]]。^[alcove.md:140-142]

## 根格顶点中的朴素下取整

`root_vertex_of_alcove` 逐分量求顶点并求和，使 `gamma - vertex` 落在基本 alcove 的 Weyl 轨道中。它显式忽略 `wall_set` 返回的 `integrals`，每层取值采用 `dot.div_euclid(denominator)`，即朴素有理下取整。代码注释特别强调，此处不采用负根修正版 `floor_eval`。^[alcove.md:153-158]

单分量算法 `root_vertex_simple` 丢弃第一面关系系数为 `1` 的墙，以其余墙构造转置子 Cartan 矩阵，并计算 `base = C^{-T} · floors`。若结果非整，则依次将后续系数为 `1` 的墙的取值加 `1` 重试；首个整值结果用于根坐标的线性组合，全部失败则报告顶点不在根格内。详见 [[Alcove 根格顶点与基本 Alcove 约化]]。^[alcove.md:160-170]

## 与重心方程的区别

`alcove_center` 的墙方程使用 `floor_eval_nbr * scale + fracs.0` 作为右端，其中 `fracs` 来自 `barycentre_eq`。这与根格顶点路径明确要求的朴素下取整应分别理解；来源并未完整展开 `floor_eval_nbr` 的实现公式，不能仅凭 `frac_eval_value` 的余数改写补写其全部行为。^[alcove.md:70-76, alcove.md:124-126, alcove.md:155-158]

## 证据边界

来源属于结构性阅读，不构成数学正确性验收；负根修正版与朴素下取整的上游对应位置仅转录自代码注释，未独立核对上游字节。`wall_set`、`barycentre_eq`、`root_vertex_simple` 及 `alcove_center` 端到端均无单元测试，本次知识维护也未执行测试。参见 [[Alcove 算法的测试覆盖与失败边界]]。^[alcove.md:9-13, alcove.md:181-184, alcove.md:193-198]

## Sources

- [alcove.md](../../sources/alcove.md) — Alcove 几何：alcove_center 与 root_vertex_of_alcove。

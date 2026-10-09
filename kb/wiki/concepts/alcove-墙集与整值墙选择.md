---
title: Alcove 墙集与整值墙选择
summary: wall_set 依据根的分数求值分层选择墙，并用 coroot 差关系筛除候选；负根整值求值改记为分母，最终整值墙集包含于墙集。
sources:
  - alcove.md
kind: concept
createdAt: "2026-10-09T14:24:33.148Z"
updatedAt: "2026-10-09T14:24:33.148Z"
tags:
  - alcove几何
  - 根系
  - 算法
aliases:
  - alcove-墙集与整值墙选择
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# Alcove 墙集与整值墙选择

`wall_set` 根据根系与有理权重选择 alcove 的墙集 `walls`，并标记其中的整值墙 `integrals`，始终满足 `integrals ⊆ walls`。它是 crate 内部接口，为 alcove 重心计算与根格顶点计算提供墙集。^[alcove.md:39-40, alcove.md:70-71, alcove.md:112-124, alcove.md:145-150]

## 输入与编号

函数接收 `RootSystem`、`RootNumbering` 和 `RationalWeight`，返回 `Result<(BTreeSet<usize>, BTreeSet<usize>), StructureError>`；两个集合分别保存墙与整值墙的根编号。[[RootNumbering 根编号与 RootNbr 顺序]]先按简单坐标之和、再按从最后坐标向前的反向字典序排列正根；正根位于 `[npos, total)`，负根位于 `[0, npos)`，对应负根的编号顺序与正根相反。^[alcove.md:39-40, alcove.md:99-107]

## 分数取值与整值边界

算法先收集全部可取得的 coroot 坐标向量，形成 `coroot_table: BTreeSet`，缺失 coroot 的根由 `filter_map` 跳过。随后对每个根调用 `frac_eval_value`，得到 `(remainder, denominator)`。^[alcove.md:114-118]

分数取值采用 `rem_euclid`，但整值边界具有正负根不对称的约定：正根整值时余数为 `0`，负根整值时则改写为 `denominator`。因此，后续以 `remainder == 0` 标记的整值墙受这一边界约定约束，不能直接理解为所有原始配对值为整数的根。^[alcove.md:116-121]

## 逐层选择与筛除

外层循环按元组字典序选出最小 level，并通过稳定排序将最小层候选移到前段。内层逐个取出该层的根 \(\alpha\)：无条件加入 `walls`；若其余数为 `0`，同时加入 `integrals`。^[alcove.md:119-121]

每选定一面墙 \(\alpha\)，算法便扫描剩余候选根 \(\beta\)，逐坐标计算 \(\alpha^\vee-\beta^\vee\)。若差向量属于 `coroot_table`，就丢弃 \(\beta\)；否则保留。若被丢弃的候选也在当前最小层中，则以 `saturating_sub` 更新该层计数 `n_min`。这一筛除规则使每层只保留不能由已选墙减去一个 coroot 到达的候选根。^[alcove.md:121-124]

## 下游用途

在[[Alcove 重心计算与标准参数重建]]中，`wall_set` 的两个输出一同进入 `barycentre_eq`。该函数将逐墙分数初始化为 `(0, 1)`，仅改写非整值墙为 `(1, n_off * labels[position])`；整值墙始终保持 `(0, 1)`。墙的分量划分与分数约束见[[墙连通分量与重心分数约束]]。^[alcove.md:70-71, alcove.md:126-134]

[[Alcove 根格顶点与基本 Alcove 约化]]中的 `root_vertex_of_alcove` 则显式忽略 `integrals`。其逐墙取值使用朴素有理下取整 `dot.div_euclid(denominator)`，源码注释明确将其与负根修正版 `floor_eval` 区分。^[alcove.md:145-150]

## 证据与覆盖边界

本页依据对 `alcove.rs` 的结构性阅读，不构成数学或正确性验收；材料中的上游行号来自代码注释，未独立核对上游字节。`wall_set`、墙分量划分、重心分数计算及 `alcove_center` 端到端均无单元测试覆盖，现有测试仅涉及分母界边界与不自洽超定方程组。^[alcove.md:9-13, alcove.md:60-61, alcove.md:171-176]

## Sources

- [alcove.md](../../sources/alcove.md)

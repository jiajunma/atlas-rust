---
title: Alcove 墙集与整值墙选择
summary: wall_set 按根的分数求值分层选择墙，以余根差的成员关系筛除候选；负根的整值求值改记为分母，整值墙集包含于墙集。
sources:
  - alcove.md
kind: concept
createdAt: "2026-10-09T14:24:33.148Z"
updatedAt: "2026-10-09T20:28:39.247Z"
tags:
  - Alcove几何
  - 根系
  - 墙集算法
aliases:
  - alcove-墙集与整值墙选择
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: Alcove 墙集与整值墙选择
summary: wall_set 依据根的分数求值分层选择墙，并用 coroot 差关系筛除候选；负根整值求值改记为分母，整值墙集始终包含于墙集。
sources:
  - alcove.md
kind: concept
tags:
  - alcove几何
  - 根系
  - 算法
aliases:
  - alcove-墙集与整值墙选择
---

# Alcove 墙集与整值墙选择

`wall_set` 根据根系与有理权重选择 alcove 的墙集 `walls`，并记录整值墙集 `integrals`，始终满足 `integrals ⊆ walls`。它是 crate 内部接口，为 alcove 重心计算与根格顶点计算提供墙集。^[alcove.md:21-24, alcove.md:39-40, alcove.md:70-71, alcove.md:119-124, alcove.md:147-150]

## 输入与根编号

函数接收 `RootSystem`、`RootNumbering` 和 `RationalWeight`，返回 `Result<(BTreeSet<usize>, BTreeSet<usize>), StructureError>`。两个集合分别保存墙与整值墙的根编号。^[alcove.md:39-40, alcove.md:70-71]

本模块的 `RootNumbering` 先按简单坐标之和 `level` 升序排列正根，同层再从最后一个坐标向前逐坐标比较。正根占编号段 `[npos, total)`，负根占 `[0, npos)`，对应负根的编号顺序与正根相反。相关编号概念见 [[RootNumbering 根编号与 RootNbr 顺序]]。^[alcove.md:99-107]

## 分数求值与整值边界

算法先将全部可取得的 coroot 坐标向量收集为 `coroot_table: BTreeSet`；缺失 coroot 的根由 `filter_map` 跳过。每个根随后经 `frac_eval_value` 得到 `(remainder, denominator)`。^[alcove.md:114-118]

分数求值使用 `rem_euclid`，但正负根在整值边界上采用不同约定：正根整值时余数为 `0`，负根整值时则改写为 `denominator`。后续加入 `integrals` 的条件是 `remainder == 0`，因此这里的“整值墙”保留了这一正负根不对称的边界约定。^[alcove.md:116-121]

## 逐层选择与候选筛除

外层循环按元组字典序选出最小 level，并以稳定排序将最小层候选移到前段。内层逐个取出该层的根 \(\alpha\)：一律加入 `walls`；若余数为 `0`，同时加入 `integrals`。^[alcove.md:119-121]

每选定一面墙 \(\alpha\)，算法便扫描剩余候选根 \(\beta\)，逐坐标计算 \(\alpha^\vee-\beta^\vee\)。若差向量属于 `coroot_table`，便丢弃 \(\beta\)；否则保留。若被丢弃的候选也属于当前最小层，则通过 `saturating_sub` 减少该层计数 `n_min`。因此，每层只保留不能由已选墙减去一个 coroot 到达的候选根。^[alcove.md:121-124]

## 下游用途

在 [[Alcove 重心计算与标准参数重建]] 中，墙集与整值墙集用于 `barycentre_eq` 的逐墙分数计算。结果初始化为 `(0, 1)`，仅将非整值墙改写为 `(1, n_off * labels[position])`；整值墙保持 `(0, 1)`。分量划分与对应约束见 [[墙连通分量与重心分数约束]]，标签构造见 [[墙分量的本原 Coroot 关系]]。^[alcove.md:70-71, alcove.md:126-143]

[[Alcove 根格顶点与基本 Alcove 约化]] 中的 `root_vertex_of_alcove` 则显式忽略 `integrals`。它逐分量求顶点后求和，使 `gamma - vertex` 落在基本 alcove 的 Weyl 轨道；取值使用朴素有理下取整 `dot.div_euclid(denominator)`，源码注释将其与负根修正版 `floor_eval` 明确区分。^[alcove.md:145-150]

## 证据与覆盖边界

本页依据 `alcove.rs` 的结构性阅读，不构成 alcove 计算的数学或正确性验收。材料中的上游位置转录自代码注释，未独立核对上游字节。^[alcove.md:9-13, alcove.md:173-173]

现有两个单元测试仅覆盖分母界边界与不自洽超定方程组；`wall_set`、`root_components`、`barycentre_eq`、`labels_for_component` 及 `alcove_center` 端到端均无单元测试覆盖。本次知识维护未执行 Atlas、Cargo、测试或 benchmark。^[alcove.md:60-61, alcove.md:174-176, alcove.md:185-190]

## Sources

- [alcove.md](../../sources/alcove.md) — Alcove 几何：alcove_center 与 root_vertex_of_alcove。

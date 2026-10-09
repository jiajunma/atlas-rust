---
title: Alcove 墙集与整值墙选择
summary: wall_set 按分数求值分层选择墙，以余根差的成员关系筛除候选；负根整值求值记为分母，整值墙集包含于墙集。
sources:
  - alcove.md
kind: concept
createdAt: "2026-10-09T14:24:33.148Z"
updatedAt: "2026-10-09T22:11:12.603Z"
tags:
  - alcove-geometry
  - root-systems
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
summary: wall_set 按分数求值分层选择 alcove 墙，以余根差的成员关系筛除候选；负根整值求值改记为分母，整值墙集始终包含于墙集。
sources:
  - alcove.md
kind: concept
tags:
  - Alcove几何
  - 根系
  - 墙集算法
aliases:
  - alcove-墙集与整值墙选择
---

# Alcove 墙集与整值墙选择

`wall_set` 根据根系与有理权重选择 alcove 的墙集 `walls`，并记录整值墙集 `integrals`，始终满足 `integrals ⊆ walls`。它是 crate 内部接口，为 alcove 重心计算与根格顶点计算提供墙集。^[alcove.md:21-24, alcove.md:70-71, alcove.md:119-124, alcove.md:147-150]

## 接口与根编号

函数接收 `RootSystem`、`RootNumbering` 和 `RationalWeight` 的引用，返回 `Result<(BTreeSet<usize>, BTreeSet<usize>), StructureError>`；两个集合依次表示墙集与整值墙集，元素为根编号。^[alcove.md:39-40, alcove.md:70-71]

本模块的 `RootNumbering` 先按简单坐标之和 `level` 升序排列正根，同层再从最后一个坐标向前逐坐标比较。正根占编号段 `[npos, total)`，负根占 `[0, npos)`，对应负根的编号顺序与正根相反。相关概念见 [[RootNumbering 根编号与 RootNbr 顺序]]。^[alcove.md:99-107]

## 分数求值与整值边界

算法先将全部可取得的余根（coroot）坐标向量收集为 `coroot_table: BTreeSet`；缺失余根的根由 `filter_map` 跳过。随后，每个根经 `frac_eval_value` 得到 `(remainder, denominator)`，供分层选择使用。^[alcove.md:114-119]

分数求值使用 `rem_euclid`，但整值边界具有正负根不对称的约定：正根整值时余数为 `0`，负根整值时则改写为 `denominator`。加入 `integrals` 的条件是 `remainder == 0`，因此该集合中的“整值墙”采用这一边界约定，不能仅按求值是否为整数理解。^[alcove.md:116-121]

## 逐层选择与候选筛除

外层循环按元组字典序取最小 level，并通过稳定排序将最小层候选移到前段。内层逐个取出该层根 \(\alpha\)：一律加入 `walls`，若余数为 `0`，则同时加入 `integrals`。^[alcove.md:119-121]

每选定一面墙 \(\alpha\)，算法便扫描剩余候选根 \(\beta\)，逐坐标计算 \(\alpha^\vee-\beta^\vee\)。若差向量属于 `coroot_table`，就丢弃 \(\beta\)；否则保留。若被丢弃的候选也属于当前最小层，则用 `saturating_sub` 减少该层计数 `n_min`。筛除方向固定为“已选墙的余根减去候选余根”，每层只保留不能由已选墙减去一个余根到达的候选。^[alcove.md:121-124]

## 下游用途

在 [[Alcove 重心计算与标准参数重建]] 中，`walls` 与 `integrals` 一起传入 `barycentre_eq`，生成逐墙分数。结果初始化为 `(0, 1)`，仅将非整值墙改写为 `(1, n_off * labels[position])`，整值墙保持 `(0, 1)`。相关分量约束与标签构造见 [[墙连通分量与重心分数约束]]、[[墙分量的本原 Coroot 关系]]。^[alcove.md:70-71, alcove.md:126-143]

[[Alcove 根格顶点与基本 Alcove 约化]] 中的 `root_vertex_of_alcove` 则显式忽略 `integrals`。它逐分量求顶点后求和，使 `gamma - vertex` 落在基本 alcove 的 Weyl 轨道；各层取值采用朴素有理下取整 `dot.div_euclid(denominator)`，源码注释明确将其与负根修正版 `floor_eval` 区分。^[alcove.md:145-150]

## 证据与覆盖边界

本页依据 `crates/atlas-real-group/src/alcove.rs` 的结构性阅读，不构成 alcove 计算的数学或正确性验收。材料中的上游位置引用转录自代码注释，未独立核对上游字节。^[alcove.md:9-13, alcove.md:173-173]

该文件的两个单元测试仅覆盖分母界边界与不自洽超定方程组；`wall_set`、`root_components`、`barycentre_eq`、`labels_for_component` 及 `alcove_center` 端到端均无单元测试覆盖。来源记录的本次知识维护未执行 Atlas、Cargo、测试或 benchmark。^[alcove.md:60-61, alcove.md:174-176, alcove.md:185-190]

## Sources

- [alcove.md](../../sources/alcove.md) — Alcove 几何：alcove_center 与 root_vertex_of_alcove。

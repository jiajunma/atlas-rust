---
title: Alcove 墙集与整值墙选择
summary: wall_set 按分数求值分层筛选墙，以余根差的成员关系排除候选；负根整值求值记为分母，整值墙集包含于墙集。
sources:
  - alcove.md
kind: concept
createdAt: "2026-10-09T14:24:33.148Z"
updatedAt: "2026-10-10T00:13:14.223Z"
tags:
  - alcove
  - 根系
  - 墙集
aliases:
  - alcove-墙集与整值墙选择
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: Alcove 墙集与整值墙选择
summary: wall_set 按分数求值分层选择墙，以余根差的成员关系筛除候选；负根整值求值记为分母，整值墙集始终包含于墙集。
sources:
  - alcove.md
kind: concept
tags:
  - alcove-geometry
  - root-systems
aliases:
  - alcove-墙集与整值墙选择
---

# Alcove 墙集与整值墙选择

`wall_set` 根据根系与有理权重选择 alcove 墙集 `walls`，同时记录整值墙集 `integrals`，满足 `integrals ⊆ walls`。它是 `atlas-real-group` 的 crate 内部接口，为重心计算与根格顶点计算提供墙集。^[alcove.md:17-24, alcove.md:70-71, alcove.md:119-124, alcove.md:147-150]

## 接口与根编号

函数接收 `RootSystem`、`RootNumbering` 和 `RationalWeight` 的引用，返回 `Result<(BTreeSet<usize>, BTreeSet<usize>), StructureError>`。两个集合依次为墙集与整值墙集，使用根编号表示元素。^[alcove.md:39-40, alcove.md:70-71]

本模块的 `RootNumbering` 将正根先按简单坐标之和 `level` 升序排列，同层再从最后一个坐标向前逐坐标比较。正根占编号段 `[npos, total)`，负根占 `[0, npos)`，对应负根的编号顺序与正根相反。相关编号主题见 [[RootNumbering 根编号与 RootNbr 顺序]]。^[alcove.md:99-107]

## 分数求值与整值边界

算法先把全部可取得的余根坐标向量收集为 `coroot_table: BTreeSet`；缺失余根的根由 `filter_map` 跳过。随后，每个根经 `frac_eval_value` 得到 `(remainder, denominator)`，用于后续分层选择。^[alcove.md:114-119]

求值采用 `rem_euclid`，但正负根的整值边界约定不同：正根整值时余数为 `0`，负根整值时则改写为 `denominator`。只有被选中且余数为 `0` 的根才进入 `integrals`；因此，这里的“整值墙”遵循该边界约定，并非收集所有求值为整数的根。^[alcove.md:116-121]

## 分层选择与候选筛除

外层循环按元组字典序确定最小层，并通过稳定排序将该层候选移到前段。内层逐个取出最小层根 $\alpha$：一律加入 `walls`，若 `remainder == 0`，则同时加入 `integrals`。^[alcove.md:119-121]

每选定一面墙 $\alpha$，算法就扫描剩余候选根 $\beta$，逐坐标计算 $\alpha^\vee-\beta^\vee$。若差向量属于 `coroot_table`，则丢弃 $\beta$；否则保留。筛除方向固定为“已选墙的余根减去候选余根”。若被丢弃的候选也属于当前最小层，则用 `saturating_sub` 减少该层计数 `n_min`。^[alcove.md:121-124]

## 下游用途

在 [[Alcove 重心计算与标准参数重建]] 中，`walls` 与 `integrals` 交给 `barycentre_eq` 生成逐墙分数。结果初始化为 `(0, 1)`；每个墙分量求得标签后，仅将非整值墙改写为 `(1, n_off * labels[position])`，整值墙保持 `(0, 1)`。^[alcove.md:70-71, alcove.md:132-134]

墙分量由 `root_components` 按 `bracket(α, β) != 0` 合并，分量按根首次出现顺序追加，分量内部按编号升序排列。`bracket` 失败会被 `unwrap_or(0)` 视为不相连；此函数不返回 `Result`。^[alcove.md:128-131]

在 [[Alcove 根格顶点与基本 Alcove 约化]] 中，`root_vertex_of_alcove` 显式忽略 `integrals`，逐分量求顶点后求和，使 `gamma - vertex` 落在基本 alcove 的 Weyl 轨道。各层取值使用朴素有理下取整 `dot.div_euclid(denominator)`；源码注释明确将其与负根修正版 `floor_eval` 区分。^[alcove.md:145-150]

## 证据与覆盖边界

来源是对 `crates/atlas-real-group/src/alcove.rs` 的结构性阅读，不构成 alcove 计算的数学或正确性验收。材料中的上游行号转录自代码注释，未独立核对上游字节。^[alcove.md:9-13, alcove.md:173-173]

文件中的两个单元测试仅覆盖分母界边界与不自洽超定方程组；`wall_set`、`root_components`、`barycentre_eq`、`labels_for_component` 及 `alcove_center` 端到端均无单元测试覆盖。来源记录的知识维护未执行 Atlas、Cargo、测试或 benchmark，参见 [[Alcove 算法的测试覆盖与失败边界]]。^[alcove.md:60-61, alcove.md:174-176, alcove.md:185-190]

## Sources

- [alcove.md](../../sources/alcove.md) — Alcove 几何：alcove_center 与 root_vertex_of_alcove。

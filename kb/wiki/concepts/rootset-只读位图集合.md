---
title: RootSet 只读位图集合
summary: RootSet 按稳定根序保存位图，公开成员查询对越界返回 false、迭代按索引升序，构造与插入仅供内部使用。
sources:
  - root-system.md
kind: concept
createdAt: "2026-10-09T15:12:14.761Z"
updatedAt: "2026-10-09T22:48:58.177Z"
tags:
  - 位图
  - 集合
  - API设计
aliases:
  - rootset-只读位图集合
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: RootSet 只读位图集合
summary: RootSet 按稳定根序存储位图，公开查询对越界返回 false，迭代按索引升序，构造与插入仅供内部使用。
sources:
  - root-system.md
kind: concept
tags:
  - 数据结构
  - 位图
  - Rust设计
aliases:
  - rootset-只读位图集合
---

# RootSet 只读位图集合

`RootSet` 是普通根系中按稳定根序组织的位图集合，用于表示 `RootSystem` 的根与余根梯子底成员。它提供成员查询和升序迭代，构造后没有公开修改入口；来源将其对应到上游 `RootNbrSet`。^[root-system.md:87-92, root-system.md:104-109]

## 表示与接口

`RootSet` 包含 `blocks` 与 `len` 字段，公开接口为 `contains`、`len`、`is_empty` 和 `iter`。构造方法 `with_capacity` 与插入方法 `insert` 均为私有接口，调用方没有公开的构造或修改入口。^[root-system.md:28-35, root-system.md:124-127]

集合所用索引遵循 `RootSystem` 的稳定根序：根、余根与简单根坐标三张表索引对齐，由同一个 `RootId` 索引。最终根序按根的环境坐标字典序排列，由闭包 `BTreeMap` 的键序决定，与 BFS 发现顺序无关；`RootSet::iter` 按索引升序遍历。相关约定见 [[RootId 与根系索引对齐]]。^[root-system.md:18-24, root-system.md:55-62, root-system.md:106-109]

## 查询与安全边界

`contains` 对越界索引返回 `false`。私有 `insert` 直接索引位图块，越界会触发 panic，但它仅在 `build_ladder_bottoms` 内以 `beta < count` 调用；来源据此将该越界路径判为不可达。公开查询与内部插入具有不同的边界处理方式。^[root-system.md:106-109]

## 梯子底集合

`min_roots[alpha]` 标记所有满足“`β - α` 不是根”的 `β`，其中包含 `alpha` 自身；`min_coroots` 在配对余根上定义同一关系。两张表在构造时全量预计算，对每对 `(α, β)` 分别计算根与余根的坐标差，没有惰性选项。参见 [[Root ladder bottom 集与固定宽度成员查询]]。^[root-system.md:89-92, root-system.md:128-129]

构造时，根差通过二分查找判定成员关系；余根表跟随根序，并非按自身坐标排序，因此先建立坐标到下标的映射。若坐标差的 `checked_sub` 溢出，该差的成员查询按 `false` 处理：所存坐标均为 `i32`，超出范围的精确差不可能等于任何已存根或余根。分配等其他错误仍然传播，详见 [[Rust ladder 成员查询的选择性溢出处理]]。^[root-system.md:92-96]

这里的 `false` 表示“坐标差不是已存根或余根”，因此对应的 `β` 满足梯子底集合的条件；它与 `RootSet::contains` 对越界索引返回 `false` 属于不同层面的查询语义。^[root-system.md:89-95, root-system.md:106-109]

## 测试与证据边界

来源列出的梯子底测试包括 B2/G2 的 oracle 成员集合、A2/B2/G2 的逐对暴力对照、梯子底访问器对越界 `RootId(6)` 返回 `None`，以及纯环面空表。坐标边界测试使用 $m \in \{0, 2^{30}-1, 2^{30}, \mathrm{i32::MAX}\}$ 与两种配置组成八组用例，固定“不拒绝、两表均为全集合”的行为。^[root-system.md:95-102]

源码注释声称原 Atlas 接受全部十一个坐标边界用例，而该文件实际固定八组；来源将数量差异记录为待核事项。这些记录属于结构性阅读与测试锚点整理，不构成根系层的数学验收。上游对应关系和 HPC 捕获编号仅转录自代码注释，未核对上游字节；本次知识维护也未执行 Atlas、Cargo、测试或 benchmark。^[root-system.md:9-14, root-system.md:100-102, root-system.md:135-139]

## Sources

- [root-system.md](../../sources/root-system.md) — 普通根系的确定性枚举：RootSystem、RootId 与梯子底表。

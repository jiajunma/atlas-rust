---
title: RootSet 只读位图集合
summary: RootSet 以稳定根序上的位图存储成员，公开 contains 对越界返回 false、iter 按索引升序遍历，构造与插入仅供内部使用。
sources:
  - root-system.md
kind: concept
createdAt: "2026-10-09T15:12:14.761Z"
updatedAt: "2026-10-09T15:12:14.761Z"
tags:
  - 数据结构
  - 位图
  - Rust设计
aliases:
  - rootset-只读位图集合
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# RootSet 只读位图集合

`RootSet` 是普通根系中按稳定根序组织的只读位图集合，对应上游的 `RootNbrSet`。它提供成员查询与升序迭代，构造后没有公开插入途径；在 `RootSystem` 中用于表示梯子底成员集合。^[root-system.md:87-92, root-system.md:104-109]

## 表示与索引顺序

`RootSet` 包含 `blocks` 与 `len` 字段，公开接口为 `contains`、`len`、`is_empty` 和 `iter`；`with_capacity` 与 `insert` 均为私有接口，也没有公开构造或修改入口。^[root-system.md:28-35, root-system.md:124-127]

集合所依托的根序来自 `RootSystem`：根、余根与简单根坐标三张表索引对齐，统一使用 `RootId` 索引。最终根序由闭包 `BTreeMap` 的环境坐标字典序决定，与 BFS 发现顺序无关；`RootSet::iter` 按该索引升序迭代。相关索引约定见 [[RootId 与根系索引对齐]]。^[root-system.md:18-24, root-system.md:55-62, root-system.md:106-109]

## 查询与安全边界

`contains` 对越界索引返回 `false`。私有 `insert` 直接索引位图块，越界可能触发 panic，但其调用仅发生在 `build_ladder_bottoms` 内，并满足 `beta < count`；来源据此将该越界路径判为不可达。公开只读接口使调用方无法直接插入成员。^[root-system.md:106-109]

## 在梯子底表中的用途

`min_roots[alpha]` 标记所有满足“`β - α` 不是根”的 `β`，其中包含 `alpha` 自身；`min_coroots` 在配对余根上定义同一关系。两张表在构造时全量预计算，对每对 `(α, β)` 分别计算根与余根的坐标差，没有惰性选项。参见 [[Root ladder bottom 集与固定宽度成员查询]]。^[root-system.md:89-92, root-system.md:128-129]

构造梯子底集合时，根差通过二分查找判定是否为根；余根表跟随根序而非自身坐标序，因此先建立坐标到下标的映射。若 `checked_sub` 溢出，该差的成员查询按 `false` 处理，因为超出 `i32` 范围的精确差不可能等于任何已存根或余根；分配等其他错误仍然传播。此处的 `false` 指坐标差不是已存根或余根，相关处理见 [[Rust ladder 成员查询的选择性溢出处理]]。^[root-system.md:92-96]

## 测试与证据边界

来源列出的梯子底测试包括 B2/G2 的 oracle 成员集合、A2/B2/G2 的逐对暴力对照、越界 `RootId(6)` 返回 `None`、纯环面空表，以及四个坐标边界值与两种配置组成的八组用例；边界用例固定“不拒绝、两表均为全集合”的行为。^[root-system.md:95-102]

这些记录属于结构性阅读与测试锚点整理，不构成根系层的数学验收。上游对应关系和 HPC 捕获编号仅转录自代码注释，来源未核对上游字节；本次知识维护也未执行 Atlas、Cargo、测试或 benchmark。^[root-system.md:9-14, root-system.md:135-139]

## Sources

- [root-system.md](root-system.md) — 普通根系的确定性枚举：RootSystem、RootId 与梯子底表

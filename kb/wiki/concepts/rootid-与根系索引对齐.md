---
title: RootId 与根系索引对齐
summary: RootId 同时索引根、余根和简单坐标三张对齐表；公开访问器通过 Option 或 Result 处理越界，简单根 ID 则保持生成器顺序。
sources:
  - root-system.md
kind: concept
createdAt: "2026-10-09T15:11:51.382Z"
updatedAt: "2026-10-09T15:11:51.382Z"
tags:
  - Rust设计
  - 根系
  - 索引管理
aliases:
  - rootid-与根系索引对齐
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# RootId 与根系索引对齐

`RootId` 是普通根系 `RootSystem` 的索引类型。同一个 ID 同时定位根、对应余根及该根的简单根坐标；三张表按索引对齐，因此余根表遵循对应根的顺序，而不独立按余根坐标排序。^[root-system.md:18-24, root-system.md:89-92]

## 类型与访问边界

`RootId` 定义为 `pub struct RootId(pub(crate) usize)`，具备 `Copy`、`Ord` 和 `Hash`。crate 外通过 `from_usize` 构造、通过 `index()` 读回数值；ID 是否落在具体根系的有效范围内，由访问器检查，越界通过 `Option` 或 `Result` 表达。^[root-system.md:21-23, root-system.md:28-42]

`RootSystem` 的索引表字段均为私有。根可以通过 `roots()` 或 `root(id)` 访问；余根不提供整表切片，只能通过 `coroot(id)` 或 `entries()` 获取。^[root-system.md:34-42, root-system.md:126-127]

## 确定性编号与对齐保证

根系从 [[BasedRootDatum：带基根数据与构造不变量|BasedRootDatum]] 枚举得到。构造先为每个简单根播种正负记录，简单坐标分别为 \(\pm e_i\)，再通过简单反射进行广度优先闭包。最终根序由 `BTreeMap` 的环境坐标键序决定，是字典序升序，与 BFS 的发现顺序无关。^[root-system.md:18-20, root-system.md:55-62]

闭包插入先检查自配对 \(\langle\alpha,\alpha^\vee\rangle=2\)，再检查是否存在相同根坐标。若根坐标重复，对应余根与简单坐标必须逐字节一致，否则报告 `"coroot agreement"` 不变量错误；重复候选不计入根数。这些检查维护了同一根键所关联记录的一致性。^[root-system.md:64-69]

## 不同顺序的职责

`RootId` 的编号顺序、简单生成元顺序和正负根分类分别维护。`simple_root_ids()` 按生成器顺序提供简单根 ID，便于下降查询；`is_positive` 查询预计算标志，正根判据是简单坐标中存在正分量。由于根按环境坐标排序，不能以编号前后两半划分正负根。^[root-system.md:78-82]

`id_of` 在有序根表上执行二分查找，将根坐标转换为 `RootId`。余根表跟随根序，构建梯子底表时则另建一次“余根坐标 → 下标”映射，以支持余根成员查询。^[root-system.md:78-80, root-system.md:89-92]

## 配对、作用与集合

`bracket(root, coroot)` 遵循 Atlas 的“根在左、余根在右”配对约定。任一 ID 越界都会返回 `IndexOutOfRange`，两侧分别使用各自表长作为 `upper_bound`。^[root-system.md:78-79]

`action_permutation` 先验证 datum 一致性，再逐根施加 Weyl 作用并反查根 ID，形成根索引上的排列；相关概念见 [[Weyl 作用到根排列的转换]]。`combine_roots` 则逐坐标执行 checked 加减，再调用 `id_of` 判断结果是否为根，其中 `Ok(None)` 表示非根。^[root-system.md:82-85]

[[RootSet 只读位图集合|RootSet]] 使用同一稳定根序表示集合：成员查询越界返回 `false`，迭代按索引升序进行，构造后没有公开修改入口。梯子底表 `min_roots` 和 `min_coroots` 为每根各保存一张这样的集合，可参见 [[Root ladder bottom 集与固定宽度成员查询]]。^[root-system.md:18-21, root-system.md:106-109]

## 测试与证据边界

来源列出的测试锚点包括 A2 字典序与简单坐标、简单根 ID、A2/B2 根与余根配对坐标、每根 `bracket(id,id)==Ok(2)`、配对越界错误，以及 A2 全部六个 Weyl 作用的运输一致性；梯子底访问另有越界 `RootId(6)` 返回 `None` 的测试。^[root-system.md:98-102, root-system.md:114-120]

这些内容属于源码结构性阅读与测试锚点记录，不构成根系层的数学验收。来源未核对所引上游字节，也未在本次知识维护中执行 Atlas、Cargo、测试或 benchmark。^[root-system.md:9-14, root-system.md:135-139]

## Sources

- [root-system.md](../../sources/root-system.md) — 普通根系的确定性枚举：RootSystem、RootId 与梯子底表。

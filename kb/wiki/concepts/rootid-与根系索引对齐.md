---
title: RootId 与根系索引对齐
summary: RootId 同时索引根、余根及简单坐标三张对齐表，访问器处理越界，简单根 ID 则保持生成器顺序。
sources:
  - root-system.md
kind: concept
createdAt: "2026-10-09T15:11:51.382Z"
updatedAt: "2026-10-09T21:10:11.835Z"
tags:
  - 根系
  - 类型设计
aliases:
  - rootid-与根系索引对齐
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: RootId 与根系索引对齐
summary: RootId 同时索引根、余根和简单坐标三张对齐表；编号由根的环境坐标字典序决定，简单根 ID 则保持生成器顺序，访问器负责越界检查。
sources:
  - root-system.md
kind: concept
tags:
  - Rust设计
  - 根系
  - 索引管理
aliases:
  - rootid-与根系索引对齐
---

# RootId 与根系索引对齐

`RootId` 是普通根系 `RootSystem` 的索引类型。同一个 ID 同时定位根、对应余根及该根的简单根坐标。三张表按索引对齐，其中根表按环境坐标字典序升序排列；余根表跟随对应根的顺序，不独立按余根坐标排序。^[root-system.md:18-24, root-system.md:89-92]

## 类型与访问边界

`RootId` 定义为 `pub struct RootId(pub(crate) usize)`，具备 `Copy`、`Ord` 和 `Hash`。crate 外通过 `from_usize` 构造、通过 `index()` 读回数值；具体 ID 是否越界由访问器检查，以 `Option` 或 `Result` 表达。^[root-system.md:21-23, root-system.md:28-42]

`RootSystem` 的字段均为私有。根可通过 `roots()` 或 `root(id)` 访问，简单坐标通过 `simple_coordinates` 查询；余根不提供整表切片，只能通过 `coroot(id)` 或 `entries()` 获取。^[root-system.md:34-42, root-system.md:126-127]

## 确定性编号与记录一致性

根系从 [[BasedRootDatum：带基根数据与构造不变量|BasedRootDatum]] 枚举得到。构造先为每个简单根播种正负记录，简单坐标分别为 \(\pm e_i\)，再通过简单反射进行广度优先闭包。最终编号由闭包 `BTreeMap` 的环境根坐标键序决定，与 BFS 的发现顺序无关，参见 [[根系的 BFS 反射闭包枚举]]。^[root-system.md:18-20, root-system.md:55-62]

闭包插入先检查自配对 \(\langle\alpha,\alpha^\vee\rangle=2\)，再检查重复根坐标。即使候选重复，也必须执行自配对检查；重复键对应的余根与简单坐标必须逐字节一致，否则报告 `"coroot agreement"` 不变量错误。重复候选不计入根数。来源将这些一致性检查描述为防御性检查，测试通过直接注入私有 `Closure` 覆盖相应拒绝路径。^[root-system.md:64-69]

## 编号、生成器顺序与正性

`RootId` 的编号顺序与简单生成器顺序分别维护。crate 内部的 `simple_root_ids()` 按生成器顺序提供简单根 ID，供下降查询使用，避免每次重新二分查找。`is_positive` 查询预计算标志，其正根判据是简单坐标中存在正分量；由于根按环境坐标排序，不能将编号前后两半直接视为负根和正根。参见 [[根系正性判定与环境坐标排序的分离]]。^[root-system.md:39-42, root-system.md:78-82]

`id_of` 在有序根表上二分查找，将根坐标转换为 ID。余根表不按自身坐标排序，因此构建梯子底表时另建一次“余根坐标 → 下标”映射，用于余根成员查询。^[root-system.md:78-80, root-system.md:89-92]

## 配对、作用与集合

`bracket(root, coroot)` 遵循 Atlas 的“根在左、余根在右”配对约定。任一 ID 越界均返回 `IndexOutOfRange`，两侧分别使用各自表长作为 `upper_bound`。^[root-system.md:78-79]

`action_permutation` 先验证 datum 一致性，再逐根施加 Weyl 作用并反查 ID。实现不额外复查余根运输；来源转录的代码注释解释，`WeylAction` 由简单反射词构成，其余权生成器与闭包使用的对偶反射相同。`combine_roots` 则逐坐标执行 checked 加减，再通过 `id_of` 判断成员关系，`Ok(None)` 表示结果不是根。^[root-system.md:82-85]

[[RootSet 只读位图集合|RootSet]] 使用稳定根序表示集合：`contains` 对越界返回 `false`，`iter` 按索引升序迭代，构造后没有公开修改入口。`min_roots` 与 `min_coroots` 为每根分别保存梯子底集合，参见 [[Root ladder bottom 集与固定宽度成员查询]]。^[root-system.md:18-21, root-system.md:106-109]

## 测试与证据边界

来源记录的测试锚点包括 A2 字典序与简单坐标、简单根 ID、A2/B2 根与余根配对坐标、每根 `bracket(id,id)==Ok(2)`、配对越界错误，以及 A2 全部六个 Weyl 作用的运输一致性。梯子底访问另有越界 `RootId(6)` 返回 `None` 的测试；来源未见直接触发 `DatumMismatch` 或 `InvalidRootAutomorphism` 的测试路径。^[root-system.md:98-102, root-system.md:114-120, root-system.md:130-131]

这些记录属于结构性源码阅读，不构成根系层的数学验收。来源中的上游位置与 HPC 捕获编号转录自代码注释，未核对上游字节；本次知识维护也未执行 Atlas、Cargo、测试或 benchmark。^[root-system.md:9-14, root-system.md:135-139]

## Sources

- [root-system.md](../../sources/root-system.md) — 普通根系的确定性枚举：RootSystem、RootId 与梯子底表。

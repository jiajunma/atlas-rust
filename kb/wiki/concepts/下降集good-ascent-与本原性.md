---
title: 下降集、good ascent 与本原性
summary: 本原性要求 good(x) 与 desc(y) 不交，极端性要求 desc(x) 包含 desc(y)，ImaginaryTypeII 既不属于下降也不属于 good ascent。
sources:
  - kl-support.md
kind: concept
createdAt: "2026-10-09T14:55:49.771Z"
updatedAt: "2026-10-09T20:58:26.720Z"
tags:
  - KL支撑
  - 本原性
aliases:
  - 下降集good-ascent-与本原性
  - 下A与
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: 下降集、good ascent 与本原性
summary: 本原性要求 good(x) 与目标下降集不相交，extremal 判定要求 desc(x) 包含目标下降集；ImaginaryTypeII 既不是下降，也不是 good ascent。
sources:
  - kl-support.md
kind: concept
tags:
  - KL算法
  - 本原性
  - 下降集
---

# 下降集、good ascent 与本原性

`KlSupport<B: BlockTopology>` 为块中每个元素预计算下降集（tau-invariant）与 good-ascent 集，并利用它们判定本原性、建立本原索引。元素 `x` 是否本原取决于给定的目标下降集；KLV 多项式 \(P_{x,y}\) 存放在第 `y` 列的 `prim_index(x, desc(y))` 处。^[kl-support.md:16-21]

## 下降集与 good ascent

记 `desc(x)` 为元素 `x` 的下降集，`good(x)` 为其 good-ascent 集。构造时逐生成元分类：状态满足 `is_descent()` 时加入下降集；否则，仅在状态不是 `ImaginaryTypeII` 时加入 good-ascent 集。**`ImaginaryTypeII` 既不是下降，也不是 good ascent**。^[kl-support.md:32-37]

两个集合以 [[RankFlags：简单生成元位集]] 表示，底层为 `u32`，要求 rank ≤ 32。`contains(other)` 检查是否包含 `other`，即超集关系；`first_bit` 按生成元编号升序寻找最低置位；`intersect` 与 `difference` 分别返回交集和差集。^[kl-support.md:23-28]

## 本原性与 extremal 判定

给定目标下降集 `desc_y`，`is_primitive(x, desc_y)` 的条件为 \(good(x)\cap desc_y=\varnothing\)。换言之，`x` 的任何 good ascent 都不能属于目标下降集。当 `desc_y = desc(y)` 时，这就是“`x` 对 `y` 的下降集本原”的含义。^[kl-support.md:19-21, kl-support.md:41-43]

`is_extremal(x, desc_y)` 检查 \(desc(x)\supseteq desc_y\)；`ascent_descent(x,y)` 返回 \(desc(y)\setminus desc(x)\) 的最低置位。因此，extremal 判定检查下降集包含关系，`ascent_descent` 则从差集中选择编号最小的生成元。^[kl-support.md:41-43]

## 唯一上升像与本原索引

`unique_ascent(s,z)` 对复上升返回 cross 像，对虚 I 型上升返回第一个 Cayley 像，其余情形返回 `None`。`prim_back_up` 原地向下搜索本原元素：先自减再判定，失败时传入的 `*x` 已被置为 0。这一修改属于调用契约，详见 [[唯一上升像与本原元素回退]]。^[kl-support.md:43-45]

[[本原索引表的惰性构建]] 由幂等的 `prepare_prim_index(desc_y)` 完成。算法按元素编号降序扫描，为本原元素记录已遇到的更大本原元素数；遇到 `RealNonparity` 或没有唯一上升像的情形时记为 `DEAD_END`（`usize::MAX`），否则沿唯一上升链继承索引。扫描结束后，将哨兵改写为 `range`，其余槽位改写为 `range - 1 - slot`。^[kl-support.md:47-53]

调用 `prim_index`、`nr_of_primitives`、`col_size` 或 `self_index` 前，必须对同一下降集完成准备，否则会因映射缺键而 panic。降序扫描还依赖“上升像序号更大”的假定；来源将其标为阅读观察，并指出没有显式防护。^[kl-support.md:53-55, kl-support.md:70-71]

## 构造前提与证据边界

[[KlSupport 的拓扑构造门控]] 在预计算前集中检查 rank ≤ 32、元素长度存在且非降、逐生成元的 descent／Cayley／inverse Cayley 数据存在，以及 cross 与 Cayley 像目标位于块内。长度非降由构造期强制保证。^[kl-support.md:32-39]

来源属于结构性阅读，不构成 KL 支撑层的数学验收；上游行号仅转录自代码注释，未核对上游字节。四个单元测试覆盖 `RankFlags` 基本位操作，以及 rank 超容量、长度顺序错误、cross 目标越界三条构造拒绝路径；构造成功路径、全部判定方法和整个本原索引机制没有直接单元测试，来源说明这些部分经 KL 层 HPC 门覆盖。详见 [[KL 支撑层的测试覆盖与证据边界]]。^[kl-support.md:9-12, kl-support.md:57-62]

## Sources

- [kl-support.md](../../sources/kl-support.md) — 逐块 KL 支撑数据：KlSupport 与 RankFlags（kl_support.rs）

---
title: 下降集、good ascent 与本原性
summary: 元素 x 相对 desc(y) 本原当且仅当 good(x) 与 desc(y) 不相交，极端性要求 desc(x) 包含 desc(y)；ImaginaryTypeII 既不属于下降也不属于 good ascent。
sources:
  - kl-support.md
kind: concept
createdAt: "2026-10-09T14:55:49.771Z"
updatedAt: "2026-10-09T14:55:49.771Z"
tags:
  - KL算法
  - 本原性
  - 下降集
aliases:
  - 下降集good-ascent-与本原性
  - 下A与
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 下降集、good ascent 与本原性

`KlSupport<B: BlockTopology>` 为块中每个元素预计算下降集（tau-invariant）与 good-ascent 集，并据此判定本原性、建立本原索引。KLV 多项式 `P_{x,y}` 存放在第 `y` 列的 `prim_index(x, desc(y))` 处，因此本原性是相对于目标元素下降集的判定。^[kl-support.md:16-21]

## 下降集与 good ascent

记 `desc(x)` 为元素 `x` 的下降集，`good(x)` 为其 good-ascent 集。构造时逐个检查简单生成元：若对应状态满足 `is_descent()`，则加入下降集；否则，仅当状态不是 `ImaginaryTypeII` 时加入 good-ascent 集。`ImaginaryTypeII` 既不是下降，也不是 good ascent，因此 good-ascent 集不能直接视为下降集的补集。^[kl-support.md:32-37]

这两个集合使用 [[RankFlags：简单生成元位集]] 表示，其底层是 `u32`，要求 rank ≤ 32。`contains(other)` 表示超集判定，`first_bit` 返回最低置位，`intersect` 与 `difference` 分别计算交集和差集。^[kl-support.md:23-28]

## 本原性与 extremal 判定

给定目标下降集 `desc_y`，`x` 为本原元素，当且仅当其 good ascent 中没有任何生成元属于 `desc_y`，即 \(good(x)\cap desc_y=\varnothing\)。当 `desc_y = desc(y)` 时，这就是“`x` 对 `y` 的下降集本原”的含义。^[kl-support.md:19-21, kl-support.md:41-43]

`is_extremal(x, desc_y)` 检查 \(desc(x)\supseteq desc_y\)；`ascent_descent(x,y)` 则返回 \(desc(y)\setminus desc(x)\) 的最低置位。前者检验下降集的包含关系，后者按生成元编号寻找 `y` 中存在而 `x` 中不存在的首个下降。^[kl-support.md:41-43]

## 唯一上升像与本原索引

`unique_ascent(s,z)` 对复上升返回 cross 像，对虚 I 型上升返回第一个 Cayley 像，其他情形返回 `None`。`prim_back_up` 原地向下搜索：先自减再判定，失败时也会将传入的 `*x` 置为 0；这一修改属于调用契约。相关操作见 [[唯一上升像与本原元素回退]]。^[kl-support.md:43-45]

本原索引由 `prepare_prim_index(desc_y)` 幂等地懒填充。算法按元素编号降序扫描，为本原元素记录已遇到的更大本原元素数；非本原元素遇到 `RealNonparity` 或没有唯一上升像时记为 `DEAD_END`，否则沿唯一上升链继承索引。扫描结束后，将哨兵改写为 `range`，其余槽位改写为 `range - 1 - slot`。^[kl-support.md:47-53]

调用 `prim_index`、`nr_of_primitives`、`col_size` 或 `self_index` 前，必须对同一下降集完成准备，否则会因映射缺键而 panic。该降序算法还依赖“上升像序号更大”的假定；来源将其标为阅读观察，并指出没有显式防护。^[kl-support.md:53-55, kl-support.md:70-71]

## 构造前提与证据边界

[[KlSupport 的拓扑构造门控]] 在预计算前集中验证 rank 上限、元素长度存在且非降、所需生成元状态与链接存在，以及 cross 和 Cayley 像目标位于块内。长度顺序是构造时强制保证的不变量。^[kl-support.md:32-39]

来源属于结构性阅读，不构成数学正确性验收，引用的上游行号也未核对原始字节。其列出的四个单元测试覆盖位操作与三条构造拒绝路径；构造成功路径、全部判定方法和整个本原索引机制没有直接单元测试，来源仅说明这些部分经 KL 层 HPC 门覆盖。详见 [[KL 支撑层的测试覆盖与证据边界]]。^[kl-support.md:9-12, kl-support.md:57-62]

## Sources

- [kl-support.md](kl-support.md) — 逐块 KL 支撑数据：KlSupport 与 RankFlags（kl_support.rs）

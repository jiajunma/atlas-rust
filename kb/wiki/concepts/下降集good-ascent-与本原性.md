---
title: 下降集、good ascent 与本原性
summary: 本原性要求 good(x) 与 desc(y) 不交，极端性要求 desc(x) 包含 desc(y)，ImaginaryTypeII 同时被排除在下降集与 good-ascent 集之外。
sources:
  - kl-support.md
kind: concept
createdAt: "2026-10-09T14:55:49.771Z"
updatedAt: "2026-10-10T00:39:16.687Z"
tags:
  - KL
  - 算法
aliases:
  - 下降集good-ascent-与本原性
  - 下A与
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
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

`KlSupport<B: BlockTopology>` 为每个块元素预计算下降集（tau-invariant）与 good-ascent 集，并用它们判定本原性、建立本原索引。**本原性相对于给定的目标下降集定义**：`x` 对 `y` 的下降集本原，当且仅当 `x` 的 good ascent 中没有一个属于 `y` 的下降集。KLV 多项式 \(P_{x,y}\) 存放在第 `y` 列的 `prim_index(x, desc(y))` 处。^[kl-support.md:16-21]

## 下降集与 good ascent

记 \(\mathrm{desc}(x)\) 为元素 `x` 的下降集，\(\mathrm{good}(x)\) 为其 good-ascent 集。构造时逐生成元分类：状态满足 `is_descent()` 时加入下降集；否则，仅在状态不是 `ImaginaryTypeII` 时加入 good-ascent 集。因此，**`ImaginaryTypeII` 既不是下降，也不是 good ascent**；good-ascent 集并非所有非下降生成元的集合。^[kl-support.md:32-37]

这两个集合使用 [[RankFlags：简单生成元位集]] 表示，底层为私有 `u32`，要求 rank ≤ 32。`contains(other)` 表示超集判定，`first_bit` 按编号升序寻找最低置位，`intersect` 与 `difference` 返回新集合。`set` 和 `is_set` 自身不检查边界，`KlSupport` 的使用路径依靠构造门控保证秩不超限。^[kl-support.md:23-28]

## 三种判定语义

给定目标下降集 \(D=\mathrm{desc}(y)\)，`is_primitive(x, D)` 检查 \(\mathrm{good}(x)\cap D=\varnothing\)，即目标下降集中不存在 `x` 的 good ascent。^[kl-support.md:19-21, kl-support.md:41-43]

`is_extremal(x, D)` 检查 \(\mathrm{desc}(x)\supseteq D\)，即 `x` 的下降集包含整个目标下降集。它与本原性的判定条件不同：extremal 使用下降集的包含关系，本原性使用 good-ascent 集与目标下降集的不相交关系。^[kl-support.md:41-43]

`ascent_descent(x, y)` 返回差集 \(\mathrm{desc}(y)\setminus\mathrm{desc}(x)\) 的最低置位，即该差集中编号最小的生成元。^[kl-support.md:27-28, kl-support.md:41-42]

## [[唯一上升像与本原元素回退|唯一上升像与本原元素回退]]

`unique_ascent(s, z)` 对复上升返回 cross 像，对虚 I 型上升返回第一个 Cayley 像，其余情形返回 `None`。`prim_back_up` 原地向下搜索本原元素，先自减再判定；搜索失败时，传入的 `*x` 已被置为 0。这一原地修改属于调用契约，参见 [[唯一上升像与本原元素回退]]。^[kl-support.md:43-45]

## 本原索引的懒填充

`prepare_prim_index(desc_y)` 是幂等的准备操作，按元素编号降序扫描。本原元素记录已见的更大本原元素数；需要归约的元素遇到 `RealNonparity` 或没有唯一上升像时，使用 `DEAD_END`（`usize::MAX`）哨兵，否则沿唯一上升链继承索引。扫描结束后，将哨兵改写为 `range`，其余槽位反转为 `range - 1 - slot`。^[kl-support.md:47-53]

调用 `prim_index`、`nr_of_primitives`、`col_size` 或 `self_index` 前，必须对同一下降集完成准备，否则会因映射缺键而 panic；其中只有 `prim_index` 的文档注释显式声明这一前置条件。降序扫描依赖“上升像序号更大”的假定，来源将其标为阅读观察，并指出没有显式防护。^[kl-support.md:53-55, kl-support.md:70-71]

## 构造前提与证据边界

[[KlSupport 的拓扑构造门控]] 在预计算前运行 `validate_topology`，检查 rank ≤ 32、逐元素长度存在且非降、逐生成元的 descent／Cayley／inverse Cayley 数据存在，以及 cross 与 Cayley 像目标位于块内。长度非降由构造期强制保证；本原索引对上升像编号的假定则没有显式防护。^[kl-support.md:32-39, kl-support.md:70-71]

来源中的四个单元测试覆盖 `RankFlags` 基本位操作，以及秩超容量、长度顺序错误、cross 目标越界三条构造拒绝路径。构造成功路径、全部判定方法和整个本原索引机制没有直接单元测试；来源说明这些部分经 KL 层 HPC 门覆盖，参见 [[KL 支撑层的测试覆盖与证据边界]]。^[kl-support.md:57-62]

本页依据对 `kl_support.rs` 的结构性阅读，不构成 KL 支撑层的数学验收。上游行号仅转录自代码注释，未核对上游字节；`BlockDescent` 完整变体集及 `is_descent()` 的定义也不在本来源的阅读范围内。本次知识维护未执行 Atlas、Cargo、测试或 benchmark。^[kl-support.md:9-12, kl-support.md:64-69, kl-support.md:75-79]

## Sources

- [kl-support.md](../../sources/kl-support.md) — 逐块 KL 支撑数据：KlSupport 与 RankFlags（kl_support.rs）。

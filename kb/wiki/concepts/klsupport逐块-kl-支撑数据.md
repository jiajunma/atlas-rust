---
title: KlSupport：逐块 KL 支撑数据
summary: KlSupport 为块元素预计算下降集、good-ascent 集和长度边界，并以本原索引定位 KLV 多项式的列内存储位置。
sources:
  - kl-support.md
kind: concept
createdAt: "2026-10-09T14:55:42.771Z"
updatedAt: "2026-10-09T20:58:21.750Z"
tags:
  - KL支撑
  - 数据结构
aliases:
  - klsupport逐块-kl-支撑数据
  - KK支
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: KlSupport：逐块 KL 支撑数据
summary: KlSupport 预计算块元素的下降集、good-ascent 集和 length-stop 表，并以懒填充的本原索引组织 KLV 多项式列；来源仅完成结构性阅读，不构成数学验收。
sources:
  - kl-support.md
kind: concept
createdAt: "2026-10-09T14:55:42.771Z"
updatedAt: "2026-10-09T14:55:42.771Z"
tags:
  - KL算法
  - Rust设计
  - 数据结构
aliases:
  - klsupport逐块-kl-支撑数据
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# KlSupport：逐块 KL 支撑数据

`KlSupport<B: BlockTopology>` 为每个块提供 KL 算法所需的支撑数据：各元素的下降集（tau-invariant）、good-ascent 集、按长度定位的 length-stop 表，以及按给定下降集组织的本原索引表。KLV 多项式 `P_{x,y}` 存放在第 `y` 列的 `prim_index(x, desc(y))` 位置。^[kl-support.md:16-21]

## 构造与存储约束

`new` 首先执行 `validate_topology`，集中检查拓扑不变量，以避免列填充深处发生 panic：rank 不超过 32；各元素长度存在且按元素顺序非降；逐生成元所需的 descent、Cayley 与 inverse Cayley 数据存在；cross 与 Cayley 像的目标均在块内。相关边界见 [[KlSupport 的拓扑构造门控]]。^[kl-support.md:30-35]

下降集与 good-ascent 集使用 [[RankFlags：简单生成元位集]] 表示。其底层为私有 `u32`，类型非 `Copy`；`contains(other)` 判断是否包含 `other`，`first_bit` 在固定的 `0..32` 范围内查找最低置位，`intersect` 与 `difference` 返回新值。`set` 和 `is_set` 不检查边界，KlSupport 的使用路径依靠构造门控保证 rank 容量约束。^[kl-support.md:23-28]

构造时，满足 `is_descent()` 的生成元进入下降集；否则，仅在类型不是 `ImaginaryTypeII` 时进入 good-ascent 集。因此，`ImaginaryTypeII` 既不是下降，也不是 good ascent。^[kl-support.md:35-37]

[[length-stop 长度边界表]] 中，`length_stop[l]` 指向首个长度至少为 `l` 的元素，表末尾追加块大小 `size`。这一定位依赖构造期强制的长度非降顺序；本原索引表则采用懒填充。^[kl-support.md:37-39]

## 下降、本原性与上升像

给定目标下降集 `desc_y`，`is_extremal(x, desc_y)` 判断 `desc(x) ⊇ desc_y`，`is_primitive(x, desc_y)` 判断 `good(x) ∩ desc_y` 是否为空。也就是说，`x` 对 `y` 的下降集本原，当且仅当 `x` 的 good ascent 中没有一个属于 `y` 的下降。参见 [[下降集、good ascent 与本原性]]。^[kl-support.md:19-20, kl-support.md:41-43]

`ascent_descent(x, y)` 返回 `desc(y) − desc(x)` 的最低置位。`unique_ascent(s, z)` 对复上升返回 cross 像，对虚 I 型上升返回第一个 Cayley 像，其余情况返回 `None`。^[kl-support.md:41-44]

`prim_back_up` 原地向下搜索，先自减再判定；失败时 `*x` 已被置为 0。调用方必须保留这一原地修改契约，相关操作见 [[唯一上升像与本原元素回退]]。^[kl-support.md:44-45]

## 本原索引的懒填充

`prepare_prim_index(desc_y)` 是幂等操作，按元素序号降序扫描。本原元素先记录已遇到的更大本原元素数；非本原元素遇到 `RealNonparity` 或无唯一上升像时，记录 `DEAD_END` 哨兵（`usize::MAX`），否则沿 `unique_ascent` 链继承索引。扫描结束后，以本原元素总数 `range` 重写结果：哨兵变为 `range`，其余槽位变为 `range - 1 - slot`。参见 [[本原索引表的惰性构建]]。^[kl-support.md:47-53]

`prim_index`、`nr_of_primitives`、`col_size` 和 `self_index` 均要求先针对同一下降集完成 prepare，否则会因 map 缺键而 panic；其中只有 `prim_index` 的文档注释显式声明这一前置条件。^[kl-support.md:53-55]

降序扫描依赖“上升像序号更大”的假定。来源将此标记为阅读观察，代码没有显式防护；它与构造期已强制检查的长度非降约束应分别理解。^[kl-support.md:70-71]

## 测试与证据边界

来源记录了 4 个基于 `FakeTopology` 的测试锚点：RankFlags 基本位操作（包括空集被任何集合包含），以及 rank 为 33、长度不满足非降顺序、cross 目标越界三条构造拒绝路径。`new` 的成功路径、全部判定方法和整个本原索引机制没有直接单元测试；来源说明它们经 KL 层的 HPC 门覆盖。参见 [[KL 支撑层的测试覆盖与证据边界]]。^[kl-support.md:57-62]

本页依据对 `crates/atlas-real-group/src/kl_support.rs` 的结构性阅读，不构成 KL 支撑层的数学验收。来源中的上游文件行号仅转录自代码注释，未核对上游字节；`BlockDescent` 的完整变体集及 `is_descent()` 定义也不在该来源的阅读范围内。^[kl-support.md:9-12, kl-support.md:66-69]

来源通过快照 `2026-10-06-kl-support.json` 绑定 Git base、文件字节 SHA-256 与 Kimi 调用记录；草案经维护者对照源码逐条核对改写。该次知识维护未执行 Atlas、Cargo、测试或 benchmark。^[kl-support.md:75-79]

## Sources

- [kl-support.md](../../sources/kl-support.md) — 逐块 KL 支撑数据：KlSupport 与 RankFlags（kl_support.rs）。

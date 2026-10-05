---
title: 逐块 KL 支撑数据：KlSupport 与 RankFlags（kl_support.rs）
source: atlas-rust/kl-support
ingestedAt: 2026-10-06T00:20:00Z
---

# 逐块 KL 支撑数据：KlSupport 与 RankFlags（kl_support.rs）

编辑状态：**结构性阅读完成；草案由 Kimi probe 起草，维护者对照源码逐条核对改写**。
本包覆盖 `crates/atlas-real-group/src/kl_support.rs`（467 行）。它是结构性
阅读，不声称 KL 支撑层的数学验收；上游引用（klsupport.h/cpp、blocks.h、
kl.cpp 行号）仅转录自代码注释，本包未核对上游字节。

## 概览

`KlSupport<B: BlockTopology>` 为每个块元素预计算（klsupport.cpp:63-89/
46-62/109-144）：下降集（tau-invariant）与 good-ascent 集、length-stop 表、
以及本原索引表（块元素 → 其在给定下降集的本原元素列表中的位置）。
核心定义（klsupport.h `is_primitive`）：`x` 对 `y` 的下降集**本原**，当且
仅当 `x` 的 good ascent 中没有一个是 `y` 的下降。存储约定：KLV 多项式
`P_{x,y}` 存放在第 `y` 列的 `prim_index(x, desc(y))` 处（kl.cpp:124-148）。

## RankFlags：简单生成元的 u32 位集

与上游 KL 算法的 `RankFlags` 语义一致，**rank ≤ 32**。私有 `bits: u32`；
`set`/`is_set` 无边界检查（KlSupport 使用路径上由构造门控保证 rank ≤ 32）。
`contains(other)` 是**超集**判定；`first_bit` 在固定 `0..32` 内升序找最低
置位；`intersect`/`difference` 返回新值。非 `Copy`。

## KlSupport：构造门控与判定语义

`new` 先跑 `validate_topology`（所有拓扑不变量在可失败边界集中检查，避免
列填充深处的 panic）：rank ≤ 32；逐元素长度存在且**非降**（顺序保证在构造
期强制）；逐生成元 descent/cayley/inverse_cayley 存在；cross 与 Cayley 像的
目标都在块内。然后逐元素分类：`is_descent()` → 下降集；否则且非
`ImaginaryTypeII` → good-ascent 集——**ImaginaryTypeII 既不是下降也不是
good ascent**。length_stop 表：`length_stop[l]` = 首个长度 ≥ l 的元素
（构造依赖非降序），末尾追加 `size`。`max_length` 被计算后显式丢弃
（`let _ = max_length`——已记录为清理候选）。`prim_index` 懒填充。

判定语义（均为一行组合）：`ascent_descent(x,y)` = desc(y) − desc(x) 的最低
置位；`is_extremal(x, desc_y)` = desc(x) ⊇ desc_y；`is_primitive` =
good(x) ∩ desc_y 为空；`unique_ascent(s, z)`：复上升取 cross 像、虚 I 型
上升取第一个 Cayley 像，其余 `None`。`prim_back_up` 原地向下走（先自减再
判定；失败时 `*x` 已置 0——原地修改是契约）。

## 本原索引表（懒填充）

`prepare_prim_index(desc_y)`：幂等；**降序**扫描，本原元素记录「已见更大
本原元素数」；`RealNonparity` 或无唯一上升像 → `DEAD_END` 哨兵
（`usize::MAX`）；否则沿 `unique_ascent` 链继承（隐含假定上升像序号更大，
阅读观察）；结尾 `range` 重写：哨兵 → `range`，其余反转为
`range - 1 - slot`。`prim_index`/`nr_of_primitives`/`col_size`/
`self_index` 都要求先对同一下降集 prepare，否则 map 缺键即 panic（仅
`prim_index` 的文档注释显式写出该前置条件）。

## 测试锚点（4 个，经 FakeTopology 替身）

`RankFlags` 基本位操作（含「空集被任何集合包含」）；三条构造拒绝路径：
rank 33 超容量、长度非非降、cross 链接目标越界。注意覆盖盲区：`new` 的
成功路径、全部判定方法与整个本原索引机制均无单元测试（它们经 KL 层的
HPC 门覆盖）。

## 限制与未覆盖面

- 不做数学/正确性验收；上游行号引用未核实。
- rank 硬上限 32（构造期拒绝更大）。
- `BlockDescent` 的完整变体集与 `is_descent()` 定义在 `crate::block`（本包
  未读）；`BlockTopology` 是 sealed trait（测试替身需实现私有 `Sealed`）。
- 顺序保证：长度非降由构造门控强制；本原索引的降序扫描依赖「上升像序号
  更大」这一阅读观察，无显式防护。

## 来源与限制

精确读取身份见
[`2026-10-06-kl-support.json`](snapshots/2026-10-06-kl-support.json)：绑定
Git base、文件字节 SHA-256 与 Kimi 调用记录。草案由 Kimi probe（无工具
档案）以完整文件字节起草，维护者对照源码逐条核对改写。本次知识维护未执行
Atlas、Cargo、测试或 benchmark。

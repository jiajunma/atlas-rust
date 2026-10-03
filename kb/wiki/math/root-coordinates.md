---
id: root-coordinates
type: concept
note_status: reviewed
aliases: [根坐标, Root coordinates, Ambient lattice]
source_snapshot: sources/snapshots/2026-10-03-root-ladder-after-v3.json
---

# 根坐标与格坐标

在这个实现中，根的环境格坐标和它相对于简单根的坐标承担不同职责。阅读排序、正根判断和算术边界时必须先确定所用坐标系。

## 两种坐标

设简单根为 $\alpha_1,\ldots,\alpha_r$。根 $\beta$ 的简单根坐标是展开
$\beta=\sum_i c_i\alpha_i$ 的系数。环境格坐标则使用根所处的完整格的基；其维数是 `lattice_rank`，简单根的数量是 `semisimple_rank`。带中心环面时，两者可以不同。

这是本项目的记号解释，依据 `BasedRootDatum` 的字段、构造检查及 `RootSystem` 的存储方式；本页不声称给出了任意根数据的完整分类或证明。

## 在 Rust 中的含义

`BasedRootDatum::from_simple_data` 检查每个简单根和简单余根的环境维数，并检查配对与给定 Cartan 矩阵一致。`RootSystem` 按同一个 `RootId` 对齐保存 `roots`、`coroots`、`simple_coordinates`，并额外预计算：

- `positive`：逐根的正根标志（在简单根坐标基下判定）；
- `simple_ids`：生成次序下简单根的稳定编号，descent 查询不必再逐个二分；
- `min_roots`/`min_coroots`：ladder-bottom 表，对应原版 `RootSystem` 构造器的
  `d_minRoots`/`d_minCoroots`（rootdata.h:154-157）。

当前根数组按环境坐标字典序排列，`id_of` 使用二分查找。正负根标志来自简单根坐标，因此不能从环境坐标排序直接推断“数组前一半就是负根”。余根随根的编号排列，本身不保证按余根坐标排序。

这些是对所记录源码字节的观察；源文件变化后需要重新核对。`root_datum.rs`
自初始快照以来未变；`root_system.rs` 的当前字节是 AFTER-v3 限定接受的
ladder 修复版本（`cc6a1764…`，见快照）。

## 来源与关联

- [root_datum.rs](../../../crates/atlas-real-group/src/root_datum.rs)：`BasedRootDatum`、`from_simple_data`、两个 rank 方法（字节同初始快照）。
- [root_system.rs](../../../crates/atlas-real-group/src/root_system.rs)：`RootSystem`、`from_closure`、`id_of`、`min_roots_for`。
- 来源快照：[2026-10-03 root-ladder AFTER-v3](../../sources/snapshots/2026-10-03-root-ladder-after-v3.json)（本页 `root_system.rs` 的读取身份）；[初始快照](../../sources/snapshots/2026-10-01-initial.json)（其余文件，字节未变）。均为所读字节身份，不是执行验证。
- [Ladder 成员判定](../algorithms/ladder-bottom-membership.md)使用这两种坐标的区别；[两版比较](../comparisons/root-ladder-cpp-rust.md)说明固定宽度表示造成的实际问题。

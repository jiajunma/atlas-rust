---
id: root-coordinates
type: concept
note_status: draft
aliases: [根坐标, Root coordinates, Ambient lattice]
source_snapshot: sources/snapshots/2026-10-01-initial.json
---

# 根坐标与格坐标

在这个实现中，根的环境格坐标和它相对于简单根的坐标承担不同职责。阅读排序、正根判断和算术边界时必须先确定所用坐标系。

## 两种坐标

设简单根为 $\alpha_1,\ldots,\alpha_r$。根 $\beta$ 的简单根坐标是展开
$\beta=\sum_i c_i\alpha_i$ 的系数。环境格坐标则使用根所处的完整格的基；其维数是 `lattice_rank`，简单根的数量是 `semisimple_rank`。带中心环面时，两者可以不同。

这是本项目的记号解释，依据 `BasedRootDatum` 的字段、构造检查及 `RootSystem` 的存储方式；本页不声称给出了任意根数据的完整分类或证明。

## 在 Rust 中的含义

`BasedRootDatum::from_simple_data` 检查每个简单根和简单余根的环境维数，并检查配对与给定 Cartan 矩阵一致。`RootSystem` 另外保存 `roots`、`coroots` 和 `simple_coordinates`，由同一个 `RootId` 对齐。

当前根数组按环境坐标字典序排列，`id_of` 使用二分查找。正负根标志来自简单根坐标，因此不能从环境坐标排序直接推断“数组前一半就是负根”。余根随根的编号排列，本身不保证按余根坐标排序。

这些是对所记录源码字节的观察；源文件变化后需要重新核对。

## 来源与关联

- [root_datum.rs](../../../crates/atlas-real-group/src/root_datum.rs)：`BasedRootDatum`、`from_simple_data`、两个 rank 方法。
- [root_system.rs](../../../crates/atlas-real-group/src/root_system.rs)：`RootSystem`、`from_closure`、`id_of`。
- [来源快照](../../sources/snapshots/2026-10-01-initial.json)：所读文件的精确身份，不是执行验证。
- [Ladder 成员判定](../algorithms/ladder-bottom-membership.md)使用这两种坐标的区别；[两版比较](../comparisons/root-ladder-cpp-rust.md)说明固定宽度表示造成的实际问题。

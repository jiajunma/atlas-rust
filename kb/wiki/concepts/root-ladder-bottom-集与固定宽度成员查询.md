---
title: Root ladder bottom 集与固定宽度成员查询
summary: 对完整存储的有限根集，B_α={β∈R | β−α∉R}；精确坐标差若超出 i32 范围，必不属于已存集合，因此 β 应进入 bottom 集，此推导不适用于一般向量运算。
sources:
  - root-ladder-overflow-repair.md
kind: concept
createdAt: "2026-10-09T15:10:39.774Z"
updatedAt: "2026-10-09T15:10:39.774Z"
tags:
  - 根系
  - 数学不变量
  - 整数溢出
aliases:
  - root-ladder-bottom-集与固定宽度成员查询
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# Root ladder bottom 集与固定宽度成员查询

Root ladder bottom 集通过根之间的差是否仍属于根集来定义。固定宽度整数实现必须保持这一数学成员关系：当精确坐标差超出已存向量的坐标表示范围时，该差不属于存储集合，不能仅因此判定整个根系构造失败。^[root-ladder-overflow-repair.md:13-15,41-49]

## 数学定义

对完整存储的有限根集 \(R\subseteq\mathbb Z^d\) 和 \(\alpha\in R\)，其 ladder bottom 集为
\[
B_\alpha=\{\beta\in R\mid \beta-\alpha\notin R\}.
\]
因此，判断 \(\beta\) 是否属于 \(B_\alpha\)，关键是判断数学整数中的精确差 \(\beta-\alpha\) 是否属于 \(R\)。^[root-ladder-overflow-repair.md:41-49]

## 固定宽度坐标下的成员判定

Rust 将每个已存根和余根的坐标表示为 `i32`。若精确差 \(\beta-\alpha\) 的任一坐标超出 `i32` 可表示区间，它就不可能等于任何已存向量。因此，此次成员查询的结果必为 `false`，相应的 \(\beta\) 应进入 bottom 集。这里的溢出提供了“不属于集合”的充分依据。^[root-ladder-overflow-repair.md:46-49]

这一推导依赖于查询对象是完整的 `i32` 存储集合，且查询目标是精确差的成员关系。它不允许使用 wrapping 或 saturating 算术，也不能推广到一般向量减法、反射、root combination、seed negation 或构造输入验证；分配失败和其他错误仍须传播。^[root-ladder-overflow-repair.md:51-53]

## Rust 实现中的局部处理

修复局限于 `build_ladder_bottoms` 的两次成员查询：`subtract_coordinates` 成功时，保留 root 二分查找或 coroot map 查找；仅当错误恰为 `StructureError::ArithmeticOverflow` 时，将成员关系解释为 `false`。`AllocationFailed` 和其他错误继续返回，root 与 coroot 查询独立执行，root 溢出不能跳过 coroot 查询。详见 [[Rust ladder 成员查询的选择性溢出处理]]。^[root-ladder-overflow-repair.md:74-83]

共享的 `difference` 缓冲区在每次 helper 调用开始时清空。溢出可能留下部分坐标前缀，但错误分支不会读取它，下一次调用也会再次清空缓冲区。修复不改变 `subtract_coordinates`、reflection、`combine_roots`、数据布局、排序或 public API。^[root-ladder-overflow-repair.md:78-86]

## 与 original Atlas 的关系

冻结于 commit `7e1b958c7aa9456769cc9cf09ac1542814b4800a` 的 original Atlas 先在抽象简单根坐标 `Byte_vector` 中建立 simple-root ladder，再通过 Weyl reflection permutation 将表传送到所有正根和负根。含 torus factor 的环境格嵌入产生的大坐标不参与这一阶段的 ladder 减法。相关构造见 [[Original Atlas 的抽象坐标 ladder 构造]]。^[root-ladder-overflow-repair.md:57-62]

Rust 采用逐对相减环境格坐标的策略，内部实现与原版不同，但两者的可观测契约都是同一个根／余根成员关系。修复的要求是固定宽度表示不能改变该关系，并不要求 Rust 改用 C++ 的整体数据结构。^[root-ladder-overflow-repair.md:64-66]

## 验证范围与限制

边界 fixture 包含 11 个 A1+torus case，覆盖 root/coroot 交换、两种 numbering、阈值下方、阈值上方及 `i32::MAX`，并保留 recovery marker 719。其 original oracle 来自历史 capture job 3868832；BEFORE-v3 job 3873400 在未修复生产代码上确认了两条 domain 回归和一条 core full-stream 回归按预期失败。相关证据见 [[A1 加中心环面边界 fixture 与历史 oracle]] 与 [[坐标边界修复的 tests-first 验证链]]。^[root-ladder-overflow-repair.md:92-107]

来源记录 AFTER-v3 job 3875239 已获限定接受，但同一来源对正式登记存在表述差异：前部将 acceptance-index 登记排除在接受范围外，后部则记录 entry `0003-a1-torus-root-coroot-ladder-boundary` 已追加且标记为 `accepted`、`math_pass`。这一状态差异应与数学推导分开阅读，参见 [[Root ladder 修复的限定接受与账本状态歧义]]。^[root-ladder-overflow-repair.md:19-34,115-123]

证据范围限于 A1+中心环面坐标边界 fixture 及相关回归守卫；AFTER 作业没有重跑原版 oracle，也不证明一般 root system 正确性、更高 rank 或 KLV 等下游数学结果。该修复不是性能优化：过去提前失败的极值输入现在会完成 \(O(|R|^2)\) 表构造，可能消耗更多时间；性能和内存结论需要受控测量。^[root-ladder-overflow-repair.md:85-88,124-130]

## Sources

- [Root ladder 固定宽度坐标溢出修复](root-ladder-overflow-repair.md)

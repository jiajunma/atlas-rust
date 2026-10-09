---
title: Original Atlas 的抽象坐标 ladder 构造
summary: Original Atlas 使用抽象简单根坐标 Byte_vector 构造 ladder，再通过 Weyl reflection permutation 扩展至所有根，使环境格中的大坐标不参与该阶段的减法。
sources:
  - root-ladder-overflow-repair.md
kind: concept
createdAt: "2026-10-09T15:10:41.042Z"
updatedAt: "2026-10-09T15:10:41.042Z"
tags:
  - 算法设计
  - Weyl群
  - 基线对齐
aliases:
  - original-atlas-的抽象坐标-ladder-构造
  - OA的L构
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# Original Atlas 的抽象坐标 ladder 构造

Original Atlas 先在抽象简单根坐标中建立 simple-root ladder，再通过 Weyl reflection permutation 将表传送到所有正根和负根。这一构造使含中心环面因子的环境格嵌入所产生的大坐标不进入 ladder 减法。本文对应的冻结版本为 commit `7e1b958c7aa9456769cc9cf09ac1542814b4800a`，源码位置为 `sources/structure/rootdata.cpp:238-317`。^[root-ladder-overflow-repair.md:55-62]

## 数学对象

对完整存储的有限根集 \(R\subseteq\mathbb Z^d\) 和根 \(\alpha\in R\)，ladder bottom 集定义为满足 \(\beta-\alpha\notin R\) 的根 \(\beta\) 的集合，即下式。^[root-ladder-overflow-repair.md:41-44]

\[
B_\alpha=\{\beta\in R\mid \beta-\alpha\notin R\}.
\]

因此，构造的核心是精确的根或余根成员关系；Original Atlas 与 Rust 可以采用不同的坐标和存储策略，但固定宽度表示不能改变这一可观测契约。相关数学边界见 [[Root ladder bottom 集与固定宽度成员查询]]。^[root-ladder-overflow-repair.md:64-66]

## 抽象坐标构造与传送

Original Atlas 的构造分为两个阶段：首先用抽象简单根坐标 `Byte_vector` 建立 simple-root ladder；随后用 Weyl reflection permutation 将表传送到所有正根和负根。抽象简单根坐标阶段不读取含 torus factor 的环境格嵌入坐标，因此这些嵌入中的大坐标不会参与该阶段的 ladder 减法。^[root-ladder-overflow-repair.md:57-62]

## 与 Rust 环境格坐标查询的区别

来源记录中的 Rust 实现逐对相减环境格坐标，以查询差是否属于根集或余根集。原先的错误是：在构造 root/coroot ladder bottom 表时，将环境格坐标差的 `i32` 溢出视为整个 `RootSystem` 构造失败。Original Atlas 的抽象坐标构造避开了这一环境格大坐标减法路径。^[root-ladder-overflow-repair.md:13-15, root-ladder-overflow-repair.md:57-66]

对于完整存储为 `i32` 坐标的集合，若数学整数中的精确差有任一坐标超出 `i32` 可表示区间，该差就不可能等于任何已存向量。因此，此成员查询应返回 `false`，相应的 \(\beta\) 应进入 bottom 集。这个结论仅适用于该成员查询，不允许 wrapping 或 saturating 算术，也不能推广到一般向量减法、反射、root combination、seed negation 或输入验证；分配失败及其他错误仍须传播。^[root-ladder-overflow-repair.md:46-53]

Rust 的局部修复据此只调整 `build_ladder_bottoms` 中的两次成员查询：仅将 `StructureError::ArithmeticOverflow` 解释为不属于集合，保留其他错误传播，并独立执行 root 与 coroot 查询。这并不要求把 Rust 整体改写为 C++ 数据结构；实现细节见 [[Rust ladder 成员查询的选择性溢出处理]]。^[root-ladder-overflow-repair.md:64-66, root-ladder-overflow-repair.md:74-83]

## 兼容性证据与范围

相关解释器 fixture 包含 11 个 A1+torus case，覆盖 root/coroot 交换、两种 numbering、阈值下方、阈值上方及 `i32::MAX`，并保留 recovery marker `719`。其 original oracle 来自历史 original-backed capture job `3868832`，记录绑定了 case、oracle binary/source 和 raw stream；该捕获不是候选修复的 AFTER 执行。参见 [[A1 加中心环面边界 fixture 与历史 oracle]]。^[root-ladder-overflow-repair.md:92-98]

这些证据不能扩展为一般 root system 正确性、更高 rank、KLV、unitarity、Hodge、associated cycle 或 AV-ann 的证明，也不支持性能或内存结论。来源对 acceptance-index 状态保留了不同表述：前部限定接受范围明确排除登记，后文则记载已追加 accepted/math_pass entry；相关状态应结合 [[Root ladder 修复的限定接受与账本状态歧义]] 阅读。^[root-ladder-overflow-repair.md:30-37, root-ladder-overflow-repair.md:115-130, root-ladder-overflow-repair.md:141-143]

## Sources

- [Root ladder 固定宽度坐标溢出修复](../../sources/root-ladder-overflow-repair.md)

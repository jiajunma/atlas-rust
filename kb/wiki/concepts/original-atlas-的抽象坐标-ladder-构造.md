---
title: Original Atlas 的抽象坐标 ladder 构造
summary: 原版先以抽象简单根坐标 Byte_vector 构建 ladder，再通过 Weyl 反射置换扩展至全部根，环境格嵌入产生的大坐标不参与该阶段减法。
sources:
  - root-ladder-overflow-repair.md
kind: concept
createdAt: "2026-10-09T15:10:41.042Z"
updatedAt: "2026-10-09T21:09:42.575Z"
tags:
  - Atlas
  - 根系
  - 算法设计
aliases:
  - original-atlas-的抽象坐标-ladder-构造
  - OA的L构
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: Original Atlas 的抽象坐标 ladder 构造
summary: Original Atlas 在抽象简单根坐标中建立 simple-root ladder，再通过 Weyl 反射置换传送到所有正负根，使环境格嵌入的大坐标不参与该阶段的减法。
sources:
  - root-ladder-overflow-repair.md
kind: concept
tags:
  - 算法设计
  - Weyl群
  - 基线对齐
aliases:
  - original-atlas-的抽象坐标-ladder-构造
---

# Original Atlas 的抽象坐标 ladder 构造

Original Atlas 使用抽象简单根坐标 `Byte_vector` 建立 simple-root ladder，再通过 Weyl reflection permutation（Weyl 反射置换）将表传送到所有正根和负根。含环面因子的环境格嵌入所产生的大坐标不参与这一阶段的 ladder 减法。本文对应冻结版本 `7e1b958c7aa9456769cc9cf09ac1542814b4800a`，源码位置为 `sources/structure/rootdata.cpp:238-317`。^[root-ladder-overflow-repair.md:57-62]

## 数学对象与成员关系

对完整存储的有限根集 \(R\subseteq\mathbb Z^d\) 和 \(\alpha\in R\)，ladder bottom 集定义如下，即收集所有减去 \(\alpha\) 后不再属于根集的根 \(\beta\)。^[root-ladder-overflow-repair.md:41-44]

\[
B_\alpha=\{\beta\in R\mid \beta-\alpha\notin R\}.
\]

构造需要保持精确的根或余根成员关系。Original Atlas 与 Rust 可以使用不同的坐标和存储策略，但固定宽度表示不能改变这一可观测契约；相关边界见 [[Root ladder bottom 集与固定宽度成员查询]]。^[root-ladder-overflow-repair.md:64-66]

## 抽象坐标构造与传送

原版首先在抽象简单根坐标 `Byte_vector` 中建立 simple-root ladder，随后使用 Weyl 反射置换把表传送到全部正负根。抽象坐标阶段不读取含 torus factor 的环境格嵌入坐标，因此这些嵌入中的大坐标不会进入该阶段的减法。^[root-ladder-overflow-repair.md:59-62]

## 与 Rust 环境格查询的关系

来源记录中的 Rust 实现逐对相减环境格坐标。其原有错误是在构造 root/coroot ladder bottom 表时，把坐标差的 `i32` 溢出视为整个 `RootSystem` 构造失败；原版的抽象坐标构造避开了这一环境格大坐标减法路径。^[root-ladder-overflow-repair.md:13-15, root-ladder-overflow-repair.md:57-66]

对于完整存储为 `i32` 坐标的根或余根集合，若数学整数中的精确差有任一坐标超出 `i32` 可表示区间，该差就不可能等于任何已存向量。因此，这一成员查询应返回 `false`，相应的 \(\beta\) 应进入 bottom 集。该论证仅适用于这种成员查询，不允许 wrapping 或 saturating 算术，也不适用于一般向量减法、反射、root combination、seed negation 或输入验证；分配失败及其他错误仍须传播。^[root-ladder-overflow-repair.md:46-53]

Rust 的局部修复只调整 `build_ladder_bottoms` 中的两次成员查询：减法成功时保持原有查找，仅在 `StructureError::ArithmeticOverflow` 时判定为不属于集合，其他错误继续返回。root 与 coroot 查询独立执行，root 溢出不会跳过 coroot 查询。这一修复不要求将 Rust 整体改写为 C++ 数据结构，详见 [[Rust ladder 成员查询的选择性溢出处理]]。^[root-ladder-overflow-repair.md:64-66, root-ladder-overflow-repair.md:74-83]

## 兼容性证据与限制

相关 fixture 包含 11 个 A1+torus case，覆盖 root/coroot 交换、两种编号、阈值下方、阈值上方及 `i32::MAX`，并保留 recovery marker `719`。original oracle 来自历史 original-backed capture job `3868832`，其记录绑定 case、oracle binary/source 和 raw stream；该捕获不是修复的 AFTER 执行，AFTER 作业也没有重跑原版 oracle。参见 [[A1 加中心环面边界 fixture 与历史 oracle]]。^[root-ladder-overflow-repair.md:92-98, root-ladder-overflow-repair.md:124-130]

来源前部将 AFTER-v3 的限定接受范围写为不含 acceptance-index 登记，后部则记载已追加 `0003-a1-torus-root-coroot-ladder-boundary` 条目，状态为 `accepted`／`math_pass`。这两处状态表述应保留区分，参见 [[Root ladder 修复的限定接受与账本状态歧义]]。^[root-ladder-overflow-repair.md:30-37, root-ladder-overflow-repair.md:115-123]

证据仅覆盖 A1+中心环面坐标边界 fixture 及相关回归守卫，不能推广为一般 root system 正确性、更高 rank、KLV、unitarity、Hodge、associated cycle 或 AV-ann 的证明，也不支持性能或内存结论。完整 Rust 测试套件属于回归守卫，并非 oracle 证明。^[root-ladder-overflow-repair.md:124-130, root-ladder-overflow-repair.md:141-143]

## Sources

- [Root ladder 固定宽度坐标溢出修复](../../sources/root-ladder-overflow-repair.md)

---
title: KLV 表的幂等逐列填充算法
summary: fill 跳过已完成列，按直接递归条件选择 recursion_column 加 complete_primitives，或进入包含 nice and real 与 endgame 情形的一般递归路径。
sources:
  - kl-polynomial-table.md
kind: concept
createdAt: "2026-10-09T14:55:25.082Z"
updatedAt: "2026-10-09T14:55:25.082Z"
tags:
  - KLV表
  - 递归算法
  - 增量计算
aliases:
  - klv-表的幂等逐列填充算法
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# KLV 表的幂等逐列填充算法

KLV 表通过 `fill(limit)` 按列计算块中的多项式 $P_{x,y}$。填充范围为 `[0, limit)`；`limit == 0` 特指填满全部列。算法跳过已经填充的列，因此重复调用具有幂等性，可以逐步推进计算范围。^[kl-polynomial-table.md:101-104]

## 列存储与计算状态

表以 $y$ 为列组织数据：多项式按 $x$ 在该列 descent set 中的 primitive-index 位置存储，列内保存的是[[KLV 多项式去重池]]的索引；μ-系数另存为非零的 `MuPair { x, coef }`。`holes[y] == true` 表示该列尚未计算。这一布局将列的计算状态与多项式、μ-系数的存储分开，相关结构见[[KLV 表的逐列存储与句柄设计]]。^[kl-polynomial-table.md:76-85]

## 逐列填充与递归分派

每个待填充列由 `fill_kl_column` 处理。它首先为该列的 descent set 准备 primitive 索引，然后根据是否存在直接递归选择计算路径；这一准备步骤与[[下降集、good ascent 与本原性]]及[[KLV 表的 primitive 投影与访问语义]]相关。^[kl-polynomial-table.md:101-110]

`first_direct_recursion` 寻找第一个使 $y$ 具有 complex descent 或 real type-I descent 的生成元 $s$。找到后，算法调用 `recursion_column`，再调用 `complete_primitives`。^[kl-polynomial-table.md:106-108]

若不存在这样的直接递归，算法改用 `new_recursion_column`，按 $x$ 区分 `recursion.pdf` 中的 “nice and real” 与 “endgame” 两种情形，μ-修正由此进入一般路径。相关多项式操作见[[KLV 递归与 μ-修正的多项式运算]]。^[kl-polynomial-table.md:109-110]

## 查询与填充状态的区别

`mu(x, y)` 不能用于判断列是否已填充：列不存在时返回 `None`，在已有列中找不到非零 μ-系数时也返回 `None`，因而无法据此区分“μ=0”与“未填充”。未计算状态由 `holes` 单独标记。^[kl-polynomial-table.md:76-79, kl-polynomial-table.md:94-95]

## 证据范围

来源包说明了逐列推进、跳过已填充列及递归分派的代码结构，但没有展开两条递归路径的完整公式与判定条件。`first_endgame_pair` 和 real-II cross 的 `UndefBlock` 边界涉及历史修复，其 tests-first 证据链另见项目交接记录，不能仅凭本页概述确认这些边界的正确性。^[kl-polynomial-table.md:112-114]

本页依据结构性源码阅读；来源包没有执行构建、测试或原版运行，也不提供数学验收、性能或并行结论。KLV 计算正确性属于独立的 [[HPC 验收证据链]]，本页不扩展其验收范围。^[kl-polynomial-table.md:9-17, kl-polynomial-table.md:135-135]

## Sources

- [KLV 多项式的存储与逐列计算](kl-polynomial-table.md)

---
title: KLV 表的幂等逐列填充算法
summary: fill 跳过已完成列，先准备 primitive 索引，再依据直接递归条件选择专用路径或包含 nice and real 与 endgame 的一般路径。
sources:
  - kl-polynomial-table.md
kind: concept
createdAt: "2026-10-09T14:55:25.082Z"
updatedAt: "2026-10-09T22:35:24.482Z"
tags:
  - KLV表
  - 递归算法
aliases:
  - klv-表的幂等逐列填充算法
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: KLV 表的幂等逐列填充算法
summary: fill 跳过已填充列，逐列准备 primitive 索引，再按下降类型选择直接递归或包含 nice and real 与 endgame 情形的一般递归路径。
sources:
  - kl-polynomial-table.md
kind: concept
tags:
  - KLV表
  - 递归算法
  - 增量计算
aliases:
  - klv-表的幂等逐列填充算法
---

# KLV 表的幂等逐列填充算法

KLV 表通过 `fill(limit)` 按列计算块中的多项式 $P_{x,y}$。填充范围为 `[0, limit)`，其中 `limit == 0` 特指填满全部列。已填充列会被跳过，因此重复调用具有幂等性，也支持逐步扩大填充范围。^[kl-polynomial-table.md:101-104]

## 列存储与计算状态

表以 $y$ 为列组织数据，列内按 $x$ 在该列下降集（descent set）中的 primitive-index 位置保存[[KLV 多项式去重池]]的索引。μ-系数单独存储，只记录非零的 `MuPair { x, coef }`；`holes[y] == true` 表示该列尚未计算。相关布局见[[KLV 表的逐列存储与句柄设计]]。^[kl-polynomial-table.md:76-85]

## 逐列填充与递归分派

每个待填充列由 `fill_kl_column` 处理：先为该列的下降集准备 primitive 索引，再根据是否存在直接递归选择计算路径。索引的查询规则见[[KLV 表的 primitive 投影与访问语义]]。^[kl-polynomial-table.md:89-93, kl-polynomial-table.md:101-110]

`first_direct_recursion` 寻找第一个使 $y$ 具有 complex descent 或 real type-I descent 的生成元 $s$。若找到，算法先调用 `recursion_column`，再调用 `complete_primitives`。“第一个满足条件”的生成元选择顺序是该分派规则的一部分。^[kl-polynomial-table.md:106-108]

若找不到这样的生成元，则调用 `new_recursion_column`，按 $x$ 区分 `recursion.pdf` 中的 “nice and real” 与 “endgame” 两种情形；μ-修正由此进入一般路径。相关运算见[[KLV 递归与 μ-修正的多项式运算]]。^[kl-polynomial-table.md:109-110]

## 查询与填充状态的区别

`mu(x, y)` 不能用于判断列是否已填充：列不存在时返回 `None`，在已有列中查不到非零 μ-系数时也返回 `None`，因此调用方无法据此区分“μ=0”与“未填充”。未计算状态由 `holes` 单独标记。^[kl-polynomial-table.md:76-79, kl-polynomial-table.md:94-95]

## 证据范围与限制

来源包说明了逐列推进、跳过已填充列及递归分派的代码结构，没有展开两条递归路径的完整公式与判定条件。`first_endgame_pair` 和 real-II cross 的 `UndefBlock` 边界涉及历史修复，其 tests-first 证据链另见项目交接记录；本页不将结构说明作为这些边界的正确性验证。^[kl-polynomial-table.md:112-114]

本页依据结构性源码阅读。来源包未执行构建、测试或原版运行，不包含数学验收、性能或并行结论；KLV 计算正确性属于独立的 [[HPC 验收证据链]]，本页不扩展其范围。来源中的上游行号转述自源码注释，未独立重读上游，可能随版本演进而漂移。^[kl-polynomial-table.md:9-17, kl-polynomial-table.md:130-135]

## Sources

- [KLV 多项式的存储与逐列计算](../../sources/kl-polynomial-table.md)

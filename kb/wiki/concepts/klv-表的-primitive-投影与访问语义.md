---
title: KLV 表的 primitive 投影与访问语义
summary: kl_pol 通过 primitive 投影访问池索引并处理恒等项、零项及 UndefBlock 哨兵；mu 返回 None 时不能区分零系数与未填充状态。
sources:
  - kl-polynomial-table.md
kind: concept
createdAt: "2026-10-09T14:55:13.420Z"
updatedAt: "2026-10-09T14:55:13.420Z"
tags:
  - KLV表
  - primitive投影
  - 边界语义
aliases:
  - klv-表的-primitive-投影与访问语义
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# KLV 表的 primitive 投影与访问语义

KLV 表按列存储多项式 $P_{x,y}$。访问某个条目时，`x` 先经 primitive 投影转换为列内位置，再取得多项式池索引；因此，列的存储位置并不直接等于块元素下标 `x`。理解这一投影及其边界规则，是正确使用 `kl_pol`、`mu` 和 `prim_map` 的前提。^[kl-polynomial-table.md:76-79, kl-polynomial-table.md:89-97]

## 列布局与池索引

列 $y$ 的多项式条目按 `x` 在该列 descent set 中的 primitive-index 位置排列，条目保存的是 `KlIndex`，而非多项式本体。调用方通过 `pool()` 取回多项式；[[KLV 多项式去重池]]的初始化约定为索引 0 表示零多项式、索引 1 表示常数 1。μ-系数则另存为非零的 `MuPair { x, coef }`。^[kl-polynomial-table.md:68-79]

表中 `holes[y] == true` 表示列 $y$ 尚未计算。逐列填充时，`fill_kl_column` 先为该列的 descent set 准备 primitive 索引，再选择递归路径；相关流程见 [[KLV 表的幂等逐列填充算法]]。^[kl-polynomial-table.md:76-79, kl-polynomial-table.md:101-110]

## `kl_pol(x, y)` 的投影与边界规则

`kl_pol(x, y)` 首先通过 `primitive_index_of` 将 `x` 投影到与列 $y$ 的 descent set 对应的 primitive 索引位置。普通的 `x` 越界或列 `y` 不存在会报错；特殊哨兵 `UndefBlock`，即 `x == support.size()`，则映射到原语计数。^[kl-polynomial-table.md:89-93]

当投影位置超出列长时，访问器采用零或恒等值回退规则。这一边界包含 $\ell(x)\geq\ell(y)$ 和 primitivisation 失败的情形：若 `x` 恰好投影到 `y` 自身，返回 $P_{y,y}=1$ 对应的池索引 1；否则返回零多项式对应的池索引 0。判定依据是投影结果是否落在 `y` 自身，而不只是原始下标是否满足 `x == y`。^[kl-polynomial-table.md:89-93]

## μ 查询与非零位图

`mu(x, y)` 在列不存在时返回 `None`；列存在时，在线性扫描该列的 `MuPair` 后返回查找结果。由于表只保存非零 μ-系数，查不到同样返回 `None`。因此，调用方不能仅凭 `None` 区分“μ 为零”与“尚未填充”。^[kl-polynomial-table.md:94-95]

`mu_column(y)` 返回该列全部非零 μ-对；`prim_map(y)` 则提供该列的非零-KL 位图。它们分别暴露 μ-系数的稀疏记录与 KL 条目的非零分布，具体存储结构可参见 [[KLV 表的逐列存储与句柄设计]]。^[kl-polynomial-table.md:76-85, kl-polynomial-table.md:94-97]

## 初始化约束与证据边界

零与恒等值回退依赖多项式池的种子索引约定。`KlHashTable::new()` 建立零、一种子，而派生的 `KlHashTable::default()` 得到空池；若调用路径使用后者，便会破坏“0=零、1=一”的索引约定。来源指出了这一风险，但未确认存在这样的调用点。^[kl-polynomial-table.md:59-61, kl-polynomial-table.md:68-72]

本页依据源码结构性阅读说明访问契约，不构成数学正确性验收。来源未执行构建、测试或原版运行；其中上游行号转述自源码注释，未独立重读上游，可能随版本变化而漂移。^[kl-polynomial-table.md:9-17, kl-polynomial-table.md:130-135]

## Sources

- [KLV 多项式的存储与逐列计算](kl-polynomial-table.md)

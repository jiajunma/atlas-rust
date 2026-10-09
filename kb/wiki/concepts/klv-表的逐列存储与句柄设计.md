---
title: KLV 表的逐列存储与句柄设计
summary: KlTableHandle 按列保存 primitive 位置对应的多项式池索引及非零 μ 对，以 holes 标记未计算列，并支持借用或 Arc 共享句柄。
sources:
  - kl-polynomial-table.md
kind: concept
createdAt: "2026-10-09T14:55:09.968Z"
updatedAt: "2026-10-09T22:35:16.101Z"
tags:
  - KLV表
  - 存储布局
  - Rust设计
aliases:
  - klv-表的逐列存储与句柄设计
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: KLV 表的逐列存储与句柄设计
summary: KlTableHandle 按块逐列保存 primitive 位置对应的多项式池索引与非零 μ 对，以 holes 标记未计算列，并支持借用和 Arc 共享句柄。
sources:
  - kl-polynomial-table.md
kind: concept
tags:
  - KLV表
  - 存储布局
  - Rust设计
aliases:
  - klv-表的逐列存储与句柄设计
---

# KLV 表的逐列存储与句柄设计

`KlTable` 为一个块计算并存储 KLV 多项式 $P_{x,y}$。表以 $y$ 为列，多项式条目保存去重池索引，非零 μ-系数单独存储。实际承载数据的是 `KlTableHandle<B: BlockTopology>`，借用别名与共享别名提供不同的句柄形式。^[kl-polynomial-table.md:76-85]

## 列布局与多项式池

列 $y$ 的多项式数据 `d_KL[y]` 按 $x$ 在该列 descent set 中的 primitive-index 位置索引，并非直接按块元素编号 $x$ 索引。μ-系数保存在 `d_mu[y]`，仅记录非零的 `MuPair { x, coef }`；`holes[y] == true` 表示该列尚未计算。投影规则见 [[KLV 表的 primitive 投影与访问语义]]。^[kl-polynomial-table.md:76-79, kl-polynomial-table.md:89-93]

[[KLV 多项式去重池]] `KlHashTable` 由 `pool: Vec<KlPol>` 与 `HashMap<KlPol, usize>` 组成，通过 `match_pol` 按多项式内容去重。`kl_pol` 返回池索引，调用方经 `pool()` 取回多项式本体。^[kl-polynomial-table.md:68-72]

经 `new()` 初始化的池将零多项式固定在索引 0、常数 1 固定在索引 1。派生的 `Default` 则创建不含这两个种子的空池，与 `new()` 语义不同；若使用 `default()` 构造，就不能依赖上述索引约定。^[kl-polynomial-table.md:59-61, kl-polynomial-table.md:68-72]

## 泛型句柄与构造入口

实际存储类型 `KlTableHandle<B: BlockTopology>` 标记为 `#[doc(hidden)]`，字段包括 `support: KlSupport<B>`、`holes`、`columns`、`mu_columns` 和 `pool`。其中 `B` 满足块拓扑约束，支撑数据由 `KlSupport<B>` 承载；相关概念见 [[BlockTopology 只读块拓扑接口]] 与 [[KlSupport：逐块 KL 支撑数据]]。^[kl-polynomial-table.md:81-85]

`KlTable<'a, B = &'a BlockGraph>` 是源码兼容别名，默认采用借用的 `BlockGraph`；`SharedKlTable = KlTableHandle<Arc<PartialBlock>>` 持有共享句柄。构造入口中，`new` 从 `&BlockGraph` 借用构造，`from_handle` 从任意满足约束的句柄构造。^[kl-polynomial-table.md:81-85]

## 访问与计算状态

`kl_pol(x, y)` 先通过 `primitive_index_of` 投影 $x$。普通的 `x` 越界或列 `y` 不存在时报错；`UndefBlock` 哨兵，即 `x == support.size()`，则映射到原语计数。投影位置超出列长时，包括 $\ell(x)\geq\ell(y)$ 与 primitivisation 失败的情形，若 $x$ 恰好投影到 $y$ 自身，则返回表示 $P_{y,y}=1$ 的池索引 1，否则返回零多项式的池索引 0。^[kl-polynomial-table.md:89-93]

`mu(x, y)` 线性查找该列的非零 μ-对；列不存在或找不到条目均返回 `None`，因此调用方不能据此区分“μ 为零”与“尚未填充”。`mu_column(y)` 返回该列全部非零 μ-对，`prim_map(y)` 返回该列的非零 KL 位图；未计算状态由 `holes` 标记。^[kl-polynomial-table.md:76-79, kl-polynomial-table.md:94-97]

`fill(limit)` 计算 `[0, limit)` 范围内的列，`limit == 0` 表示填满全部列，已填充列会被跳过。每列先准备对应 descent set 的 primitive 索引，再分派递归路径，从而支持幂等的逐列推进。具体过程见 [[KLV 表的幂等逐列填充算法]]。^[kl-polynomial-table.md:101-110]

## 证据边界

本页依据 `kl_polynomial.rs` 与 `kl_table.rs` 的结构性阅读。来源包未执行构建、测试或原版运行，不提供数学验收、性能或并行结论；KLV 计算正确性属于独立的 HPC 证据链，本页不扩展其接受范围。来源中的上游行号转述自源码注释，未独立重读上游，可能随版本演进漂移。^[kl-polynomial-table.md:9-17, kl-polynomial-table.md:130-135]

## Sources

- [kl-polynomial-table.md](../../sources/kl-polynomial-table.md) — KLV 多项式的存储与逐列计算。

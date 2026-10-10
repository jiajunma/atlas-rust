---
title: KLV 表的逐列存储与句柄设计
summary: KlTableHandle 按列保存 primitive 位置对应的池索引与非零 μ 对，以 holes 标记未计算列，并支持借用或 Arc 共享句柄。
sources:
  - kl-polynomial-table.md
kind: concept
createdAt: "2026-10-09T14:55:09.968Z"
updatedAt: "2026-10-10T00:39:15.369Z"
tags:
  - KLV
  - 存储
  - Rust
aliases:
  - klv-表的逐列存储与句柄设计
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
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

`KlTable` 为一个块计算并存储 KLV 多项式 $P_{x,y}$。表以 $y$ 为列，多项式条目保存去重池索引，非零 μ-系数另行存储。实际存储类型是 `KlTableHandle<B: BlockTopology>`，通过类型别名支持借用与共享句柄。^[kl-polynomial-table.md:76-85]

## 列布局与多项式池

列 $y$ 的 `d_KL[y]` 按 $x$ 在该列 descent set 中的 primitive-index 位置索引，而非直接按块元素编号 $x$ 索引。`d_mu[y]` 仅保存非零 μ-系数对 `MuPair { x, coef }`；`holes[y] == true` 表示该列尚未计算。相关投影规则见 [[KLV 表的 primitive 投影与访问语义]]。^[kl-polynomial-table.md:76-79, kl-polynomial-table.md:89-93]

[[KLV 多项式去重池]] `KlHashTable` 由 `pool: Vec<KlPol>` 与 `HashMap<KlPol, usize>` 组成，`match_pol` 按多项式内容去重。表的 `kl_pol` 访问器返回池索引，调用方经 `pool()` 取回多项式本体。^[kl-polynomial-table.md:68-72]

使用 `new()` 初始化时，池索引 0 对应零多项式，索引 1 对应常数 1。派生的 `Default` 则创建不含这两个种子的空池，与 `new()` 语义不同；若存在 `default()` 调用点，上述索引约定将被破坏。^[kl-polynomial-table.md:59-61, kl-polynomial-table.md:68-72]

## 泛型句柄与构造入口

`KlTableHandle<B: BlockTopology>` 标记为 `#[doc(hidden)]`，字段包括 `support: KlSupport<B>`、`holes`、`columns`、`mu_columns` 和 `pool`。块拓扑类型参数与支撑数据分别关联 [[BlockTopology 只读块拓扑接口]] 和 [[KlSupport：逐块 KL 支撑数据]]。^[kl-polynomial-table.md:81-85]

`KlTable<'a, B = &'a BlockGraph>` 是源码兼容别名，默认使用借用的 `BlockGraph`；`SharedKlTable = KlTableHandle<Arc<PartialBlock>>` 持有共享句柄。`new` 从 `&BlockGraph` 借用构造，`from_handle` 从任意满足类型约束的句柄构造。^[kl-polynomial-table.md:81-85]

## 访问语义与计算状态

`kl_pol(x, y)` 先通过 `primitive_index_of` 将 $x$ 投影到 $y$ 的 descent set。普通的 `x` 越界或列 `y` 不存在时报错；`UndefBlock` 哨兵，即 `x == support.size()`，映射到原语计数。投影位置超出列长时，包括 $\ell(x)\geq\ell(y)$ 与 primitivisation 失败的情形，若 $x$ 恰好投影到 $y$ 自身，则返回表示 $P_{y,y}=1$ 的池索引 1，否则返回零多项式的池索引 0。^[kl-polynomial-table.md:89-93]

`mu(x, y)` 在线性查找中寻找该列的非零 μ-对；列不存在或未找到条目时均返回 `None`，因此不能据此区分“μ 为零”与“尚未填充”。`mu_column(y)` 返回该列全部非零 μ-对，`prim_map(y)` 返回该列的非零 KL 位图；列是否尚未计算由 `holes` 标记。^[kl-polynomial-table.md:76-79, kl-polynomial-table.md:94-97]

`fill(limit)` 计算 `[0, limit)` 内的列，`limit == 0` 表示填满全部列，已填充列会被跳过。每列先准备对应 descent set 的 primitive 索引，再分派递归路径，形成幂等的逐列推进；详见 [[KLV 表的幂等逐列填充算法]]。^[kl-polynomial-table.md:101-110]

## 证据边界

本页依据 `kl_polynomial.rs` 与 `kl_table.rs` 的结构性阅读。来源包未执行构建、测试或原版运行，不提供数学验收、性能或并行结论；KLV 计算正确性属于独立的 HPC 证据链，本页不扩展其接受范围。来源中的上游行号转述自源码注释，未独立重读上游，可能随版本演进漂移。^[kl-polynomial-table.md:9-17, kl-polynomial-table.md:130-135]

## Sources

- [kl-polynomial-table.md](../../sources/kl-polynomial-table.md) — KLV 多项式的存储与逐列计算。

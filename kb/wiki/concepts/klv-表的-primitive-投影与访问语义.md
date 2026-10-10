---
title: KLV 表的 primitive 投影与访问语义
summary: kl_pol 经 primitive 投影处理恒等项、零项及 UndefBlock 哨兵；mu 返回 None 无法区分零系数与未填充状态。
sources:
  - kl-polynomial-table.md
kind: concept
createdAt: "2026-10-09T14:55:13.420Z"
updatedAt: "2026-10-10T00:38:34.363Z"
tags:
  - KLV
  - 索引
  - 接口语义
aliases:
  - klv-表的-primitive-投影与访问语义
  - K表P投
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: KLV 表的 primitive 投影与访问语义
summary: kl_pol 经 primitive 投影返回多项式池索引，处理恒等项、零项和 UndefBlock 哨兵；mu 的 None 无法区分零系数与未填充状态。
sources:
  - kl-polynomial-table.md
kind: concept
tags:
  - KLV表
  - 查询语义
aliases:
  - klv-表的-primitive-投影与访问语义
provenanceState: extracted
---

# KLV 表的 primitive 投影与访问语义

KLV 表按列存储多项式 $P_{x,y}$。`kl_pol(x, y)` 先通过 `primitive_index_of` 将 `x` 投影到列 $y$ 的 descent set 所对应的 primitive 索引位置，再返回多项式池索引。因此，块元素下标、列内 primitive 位置与多项式池索引是不同的索引层次。^[kl-polynomial-table.md:76-79, kl-polynomial-table.md:89-93]

## 列布局与池索引

列 $y$ 按 primitive-index 位置保存池索引，调用方通过 `pool()` 取回多项式本体。[[KLV 多项式去重池]]按多项式内容去重，约定索引 0 为零多项式、索引 1 为常数 1。μ-系数另存为非零的 `MuPair { x, coef }`；相关布局见 [[KLV 表的逐列存储与句柄设计]]。^[kl-polynomial-table.md:68-85]

`holes[y] == true` 表示列 $y$ 尚未计算。计算一列时，`fill_kl_column` 先准备该列 descent set 的 primitive 索引，再选择递归路径。`fill(limit)` 计算 `[0, limit)` 内的列，`limit == 0` 表示全部填满，且跳过已填充列，形成[[KLV 表的幂等逐列填充算法]]。^[kl-polynomial-table.md:76-79, kl-polynomial-table.md:101-110]

## `kl_pol(x, y)` 的边界规则

`kl_pol(x, y)` 对普通的 `x` 越界或不存在的列 `y` 报错。特殊哨兵 `UndefBlock`，即 `x == support.size()`，则映射到原语计数，具有单独的处理约定。^[kl-polynomial-table.md:89-93]

当投影位置超出列长时，访问器返回恒等项或零项。这一边界包含 $\ell(x)\geq\ell(y)$ 与 primitivisation 失败的情形：若 `x` 恰好投影到 `y` 自身，则返回 $P_{y,y}=1$ 对应的池索引 1；否则返回零多项式对应的池索引 0。恒等项的判定依据是投影结果是否为 `y` 自身。^[kl-polynomial-table.md:89-93]

## μ 查询与非零位图

`mu(x, y)` 在列不存在时返回 `None`；列存在时，线性查找其中的 `MuPair`。由于表中只保存非零 μ-系数，查不到同样返回 `None`，调用方不能仅凭这一结果区分“μ 为零”与“尚未填充”。^[kl-polynomial-table.md:94-95]

`mu_column(y)` 返回该列全部非零 μ-对；`prim_map(y)` 返回该列的非零-KL 位图。两者分别提供 μ-系数记录与 KL 条目的非零分布。^[kl-polynomial-table.md:94-97]

## 初始化约束

零项与恒等项的回退依赖固定池索引。`KlHashTable::new()` 建立零、一种子，而派生的 `KlHashTable::default()` 得到空池；若存在使用后者的调用点，“0=零、1=一”的约定就会被破坏。来源将此记录为条件性风险，未确认存在这样的调用点。^[kl-polynomial-table.md:59-61, kl-polynomial-table.md:68-72, kl-polynomial-table.md:89-93]

## 证据边界

本页依据 `kl_polynomial.rs` 与 `kl_table.rs` 的结构性阅读说明访问契约，不构成 KLV 数学正确性验收。来源未执行构建、测试或原版运行；其中上游行号转述自源码注释，未独立重读上游，可能随版本演进而漂移。^[kl-polynomial-table.md:9-17, kl-polynomial-table.md:124-135]

## Sources

- [KLV 多项式的存储与逐列计算](../../sources/kl-polynomial-table.md)

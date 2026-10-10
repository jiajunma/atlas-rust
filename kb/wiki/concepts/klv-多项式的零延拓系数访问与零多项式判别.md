---
title: KLV 多项式的零延拓系数访问与零多项式判别
summary: coefficient 越界时返回 0，add/sub 依赖这一零延拓行为；零多项式的 degree 返回 0，须用 is_zero 与常数多项式区分。
sources:
  - kl-polynomial-table.md
kind: concept
createdAt: "2026-10-09T19:31:56.807Z"
updatedAt: "2026-10-09T19:31:56.807Z"
tags:
  - KLV多项式
  - 访问语义
aliases:
  - klv-多项式的零延拓系数访问与零多项式判别
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# KLV 多项式的零延拓系数访问与零多项式判别

`KlPol` 的系数访问采用零延拓语义：`coefficient()` 在下标超出存储范围时返回 0。零多项式则以空向量表示，必须通过 `is_zero()` 判别，不能仅根据 `degree()` 的返回值判断。^[kl-polynomial-table.md:21-27, kl-polynomial-table.md:55-56]

## 存储与零延拓

`KlPol(Vec<i32>)` 按次数从低到高存储系数。零多项式的向量为空；非零多项式的最高次系数必须非零，各次运算通过 `trim` 维持这一不变量。相关表示约定见 [[KLV 多项式的表示与不变量]]。^[kl-polynomial-table.md:21-27]

`coefficient()` 的文档写有“panics if out of range”，但实际实现越界返回 0，应以实现为准。`add` 与 `sub` 的正确性依赖这一行为：访问未存储的高次项时，其系数按零处理。这是 [[KLV 递归与 μ-修正的多项式运算]] 所依赖的基础访问语义。^[kl-polynomial-table.md:31-38, kl-polynomial-table.md:55-56]

## 零多项式判别

`degree()` 对零多项式返回 0，沿用 `polynomials.h` 的约定。因此，次数返回值 0 无法区分零多项式与非零常数多项式；调用方需要使用 `is_zero()` 作出区分。空向量表示与次数接口的约定应同时理解。^[kl-polynomial-table.md:23-25]

在 [[KLV 多项式去重池]] 中，`KlHashTable::new()` 将零多项式固定在索引 0、常数 1 固定在索引 1。不过，派生的 `Default` 构造得到空池，并不建立这两个种子；不能将 `default()` 与 `new()` 视为具有相同的索引约定。^[kl-polynomial-table.md:59-61, kl-polynomial-table.md:68-72]

## 测试与证据边界

来源列出的四个多项式模块测试覆盖池种子序号、乘以 $1+q$、在 $q=-1$ 处求值和一个 `sub_shifted` 用例；其中未列出 `coefficient()` 零延拓或 `is_zero()` 的直接测试，并明确将 `add`、`sub` 列为未测面。因此，上述访问与判别语义依据源码结构性阅读，不应表述为已获完整测试验证。^[kl-polynomial-table.md:55-56, kl-polynomial-table.md:118-123]

该来源未执行构建、测试或原版运行，也不提供数学验收、性能或并行结论；进一步的证据范围可参见 [[KLV 多项式引擎的测试覆盖与证据边界]]。^[kl-polynomial-table.md:135-135]

## Sources

- [kl-polynomial-table.md](kl-polynomial-table.md) — KLV 多项式的存储与逐列计算。

---
title: 由 Lie 类型构造根数据
summary: build_datum 在单连通情形使用权格基、伴随情形使用根格基，校验简单数据，并将环面因子置于半单类型之后。
sources:
  - atlas-core-domain-construction.md
kind: concept
createdAt: "2026-10-09T14:28:07.919Z"
updatedAt: "2026-10-09T20:31:58.428Z"
tags:
  - 根数据
  - 构造管线
aliases:
  - 由-lie-类型构造根数据
  - 由L类
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

# 由 Lie 类型构造根数据

`build_datum` 根据 Lie 类型构造带基根数据，通过 `simply` 选择单连通或伴随情形的格基，再校验简单根数据并确定 isogeny 分类。该函数位于 `crates/atlas-core/src/domain_builtins.rs` 的 1628 行起。^[atlas-core-domain-construction.md:10-14, atlas-core-domain-construction.md:16-22]

## 格基与简单数据

构造首先通过 `block_cartan` 得到半单部分的 Cartan 矩阵。单连通情形使用**权格基**：简单根取 Cartan 矩阵的行，简单余根取基向量；伴随情形使用**根格基**：简单根取基向量，简单余根取 Cartan 矩阵的列。两种情形的坐标均以零填充到格秩，根与余根所用的行、列方向必须保持这一约定。^[atlas-core-domain-construction.md:18-20]

## 构造校验与 isogeny 分类

简单数据交由 `BasedRootDatum::from_simple_data` 校验，相关类型见 [[BasedRootDatum：带基根数据与构造不变量]]。当 `lattice_rank != semisimple` 时，isogeny 标记为 `Other`；否则调用 `classify_isogeny` 分类。^[atlas-core-domain-construction.md:21-22]

## 环面因子的顺序

源材料引用的上游 `RootDatum::type`（`rootdata.cpp:1016`）将 T1 环面因子追加在半单类型之后；`build_datum` 构造的简单数据也采用这一顺序。因此，构造结果不保留输入中环面因子与半单因子交错的排列，但不会改变调用方持有的 `LieType` 值。^[atlas-core-domain-construction.md:23-25]

## 相邻构造路径

中间中央商由 `build_quotient_datum` 和 `build_quotient_from_handle` 处理，其覆盖范围不限于单连通与伴随两个端点。显式根、余根矩阵则由 `build_explicit_datum` 接收，并保留空维矩阵的维数。参见 [[中央商与显式矩阵根数据构造]]。^[atlas-core-domain-construction.md:27-33]

## 证据边界

本页依据构造管线的结构性阅读，不构成数学验收。源材料中的上游行号属于实现方的移植陈述，构造兼容性仍以 HPC 差分门为准；相关证据要求见 [[HPC 验收证据链]]。^[atlas-core-domain-construction.md:9-14, atlas-core-domain-construction.md:73-75]

## Sources

- [领域构造管线（domain_builtins.rs 1628–2527）](atlas-core-domain-construction.md)

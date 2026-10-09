---
title: 由 Lie 类型构造根数据
summary: build_datum 按单连通或伴随情形选择权格基或根格基，校验简单根数据并分类 isogeny；环面因子排在半单类型之后。
sources:
  - atlas-core-domain-construction.md
kind: concept
createdAt: "2026-10-09T14:28:07.919Z"
updatedAt: "2026-10-09T14:28:07.919Z"
tags:
  - 根数据
  - Lie类型
  - 构造算法
aliases:
  - 由-lie-类型构造根数据
  - 由L类
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 由 Lie 类型构造根数据

`build_datum` 根据 Lie 类型构造带基根数据，以 `simply` 选择单连通或伴随情形的格基，再通过 `BasedRootDatum::from_simple_data` 校验。该流程位于 `crates/atlas-core/src/domain_builtins.rs` 的 1628 行起。^[atlas-core-domain-construction.md:16-22]

## 格基与简单数据

构造首先通过 `block_cartan` 得到半单部分的 Cartan 矩阵。单连通情形使用**权格基**：简单根取 Cartan 矩阵的行，简单余根取基向量；伴随情形使用**根格基**：简单根取基向量，简单余根取 Cartan 矩阵的列。两种情形的坐标都以零填充到格秩，因此必须保留根与余根所用的行、列方向。^[atlas-core-domain-construction.md:18-20]

## 构造校验与 isogeny 分类

简单数据交由 `BasedRootDatum::from_simple_data` 校验，相关构造约束见 [[BasedRootDatum：带基根数据与构造不变量]]。当 `lattice_rank != semisimple` 时，isogeny 标记为 `Other`；否则调用 `classify_isogeny` 分类。^[atlas-core-domain-construction.md:21-22]

## 环面因子的顺序

源材料引用的上游 `RootDatum::type`（`rootdata.cpp:1016`）将 T1 环面因子追加在半单类型之后；`build_datum` 构造的简单数据也采用这一顺序。因此，构造结果不保留输入中环面因子与半单因子交错的排列，但不会改变调用方持有的 LieType 值。^[atlas-core-domain-construction.md:23-25]

## 相邻构造路径与证据边界

中央商与显式矩阵输入由其他构造路径处理：`build_quotient_datum` 和 `build_quotient_from_handle` 支持中间中央商，`build_explicit_datum` 接收显式根、余根矩阵，并保留空维矩阵的维数。参见 [[中央商与显式矩阵根数据构造]]。^[atlas-core-domain-construction.md:27-33]

本页依据构造管线的结构性阅读，不构成数学验收。源材料中的上游行号属于实现方的移植陈述，构造兼容性仍以 HPC 差分门为准；相关证据要求见 [[HPC 验收证据链]]。^[atlas-core-domain-construction.md:9-14, atlas-core-domain-construction.md:73-75]

## Sources

- [领域构造管线（domain_builtins.rs 1628–2527）](atlas-core-domain-construction.md)

---
title: 领域值提取器与隐式 LieType 转换
summary: 共享提取器接受 LieType 或解析字符串，并将无法收窄为 usize 的整数报告为非负机器整数要求。
sources:
  - atlas-core-domain-seams.md
kind: concept
createdAt: "2026-10-09T20:33:25.003Z"
updatedAt: "2026-10-10T01:43:59.192Z"
tags:
  - 领域内建
  - 类型转换
aliases:
  - 领域值提取器与隐式-lietype-转换
  - 领L转
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# 领域值提取器与隐式 LieType 转换

领域值提取器是 `domain_builtins.rs` 中由派发臂与构造管线共用的 `Value` → 领域类型转换助手，负责 LieType、机器整数与矩阵参数的提取，并处理相应的类型、形状和数值范围检查。^[atlas-core-domain-seams.md:21-41]

## 隐式 LieType 转换

`as_lie_type` 对 `LieType` 值直接传递，对 `String` 值调用 `parse_lie_type` 解析，是上游隐式 string→LieType 转换在领域层的落点。其他类型的值产生 Type 类诊断：`expected a Lie type, found …`。^[atlas-core-domain-seams.md:25-27]

## 机器整数收窄

`as_usize` 将 BigInt 收窄为 `usize`；转换失败时报 `expected a nonnegative machine integer`，要求输入能够表示为非负机器整数。^[atlas-core-domain-seams.md:28-29]

## 矩阵提取与形状检查

`as_matrix` 先检查方阵形状。对于类型化的 `Value::Matrix`，错误包含实际行列数：`expected a square mat; received a {}x{} matrix`；对于嵌套表，错误为 `expected a square mat`。两条输入路径保留不同的诊断措辞。^[atlas-core-domain-seams.md:30-32]

`explicit_datum_matrix` 对类型化 `Matrix` 返回 `Cow::Borrowed`，实现零拷贝借用；对嵌套表则先按行提取，再按列主序通过 `Matrix::from_columns` 装配。空表产生运行时错误：`Implicit conversion to matrix for an empty set of vectors`。^[atlas-core-domain-seams.md:33-36]

### 空维保留与校验顺序

`as_matrix_rows` 将 `0×N` 类型化矩阵表示为 N 个空行，避免下游维度校验将其误判为 `0×0`。来源将此处理定位为原版接受 `N×0` 简单根／余根矩阵的适配层，回归锚点为 `zero_row_matrices_do_not_collapse_into_zero_by_zero_shapes`。相关细节见 [[显式根数据矩阵适配与空维保留]]。^[atlas-core-domain-seams.md:37-40]

矩形校验先于逐元素 `i32` 收窄；元素收窄失败时报 `matrix entry out of range`。形状检查与数值范围检查因此具有明确的先后顺序。^[atlas-core-domain-seams.md:41-41]

## 根数据构造中的使用

显式 RootDatum 派发臂同时使用 `as_matrix_rows` 提取 lattice 基，并使用 `explicit_datum_matrix` 提取简单根／余根矩阵；另有多个派发臂直接使用 `as_matrix_rows`。构造背景可参见 [[中央商与显式矩阵根数据构造]]。^[atlas-core-domain-seams.md:42-45]

## 证据边界

来源属于对当前工作区代码的结构性阅读，不声称语言或数学验收。上游 C++／CWEB 行号转述自 Rust 源码注释，未独立重读上游，可能随版本漂移；值提取器与校验助手的诊断措辞以 HPC 语料门为行为权威，参见 [[HPC 验收证据链]]。^[atlas-core-domain-seams.md:19-19, atlas-core-domain-seams.md:115-118]

## Sources

- [atlas-core-domain-seams.md](../../sources/atlas-core-domain-seams.md) — 领域层接缝：值提取器、alcove 助手与 Weyl 词／生成元校验。

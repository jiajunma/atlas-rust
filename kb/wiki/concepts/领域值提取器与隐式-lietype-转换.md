---
title: 领域值提取器与隐式 LieType 转换
summary: 共享提取器将 Value 转为领域参数：LieType 可直接提取或由字符串解析，整数转 usize 失败时报告非负机器整数要求。
sources:
  - atlas-core-domain-seams.md
kind: concept
createdAt: "2026-10-09T20:33:25.003Z"
updatedAt: "2026-10-09T20:33:25.003Z"
tags:
  - 领域桥接
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
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

# 领域值提取器与隐式 LieType 转换

领域值提取器是 `domain_builtins.rs` 中由派发臂与构造管线共用的 `Value` → 领域类型转换助手。它们处理 Lie 类型、机器整数和矩阵输入，并在转换边界执行类型、形状与数值范围检查。^[atlas-core-domain-seams.md:21-41]

## 隐式 LieType 转换

`as_lie_type` 对 `LieType` 值直接传递，对 `String` 值调用 `parse_lie_type` 解析，是上游隐式 string→LieType 强制转换在领域层的落点。其他值产生 Type 类诊断 `expected a Lie type, found …`。^[atlas-core-domain-seams.md:25-27]

## 机器整数收窄

`as_usize` 将 BigInt 收窄为 `usize`；转换失败时报 `expected a nonnegative machine integer`。这一步同时限制输入的非负性与机器整数可表示范围。^[atlas-core-domain-seams.md:28-29]

## 矩阵提取与形状检查

`as_matrix` 先检查方阵形状。对类型化的 `Value::Matrix`，错误信息包含实际行列数：`expected a square mat; received a {}x{} matrix`；嵌套表路径则报 `expected a square mat`。两种输入表示保留了不同的诊断措辞。^[atlas-core-domain-seams.md:30-32]

`explicit_datum_matrix` 对类型化 `Matrix` 返回 `Cow::Borrowed`，实现零拷贝借用；对嵌套表则先按行提取，再按列主序通过 `Matrix::from_columns` 装配。空表产生运行时错误 `Implicit conversion to matrix for an empty set of vectors`。^[atlas-core-domain-seams.md:33-36]

### 空维保留与校验顺序

`as_matrix_rows` 将 `0×N` 类型化矩阵表示为 N 个空行，避免下游维度校验将其误判为 `0×0`。来源将这一处理定位为 empty-root-datum 证据的适配层：原版接受 `N×0` 的简单根／余根矩阵；对应回归锚点是 `zero_row_matrices_do_not_collapse_into_zero_by_zero_shapes`。^[atlas-core-domain-seams.md:37-40]

矩阵路径先完成矩形校验，再逐元素收窄至 `i32`；元素超出范围时报 `matrix entry out of range`。这一顺序是输入验证契约的一部分。^[atlas-core-domain-seams.md:41-41]

## 根数据构造中的衔接

显式 RootDatum 派发臂同时使用 `as_matrix_rows` 提取 lattice 基，并使用 `explicit_datum_matrix` 提取简单根与简单余根矩阵，可结合 [[中央商与显式矩阵根数据构造]] 阅读。此外，`as_matrix_rows` 也由多个其他派发臂直接消费。^[atlas-core-domain-seams.md:42-45]

## 证据边界

这些说明来自当前工作区字节的结构性阅读，不构成语言或数学验收。来源中的上游 C++／CWEB 行号转述自 Rust 源码注释，未独立重读上游；值提取器与校验助手的诊断措辞仍以 HPC 语料门为行为权威，相关证据纪律见 [[HPC 验收证据链]]。^[atlas-core-domain-seams.md:9-19, atlas-core-domain-seams.md:113-118]

## Sources

- [atlas-core-domain-seams.md](../../sources/atlas-core-domain-seams.md) — 领域层接缝：值提取器、alcove 助手与 Weyl 词／生成元校验。

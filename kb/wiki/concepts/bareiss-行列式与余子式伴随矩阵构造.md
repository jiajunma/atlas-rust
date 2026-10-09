---
title: Bareiss 行列式与余子式伴随矩阵构造
summary: bareiss_det 采用无分数消元及下方换行选主元，adjugate_det 以转置余子式构造伴随矩阵；实现使用 i64，注释限定秩不超过 9，空矩阵行列式为 1。
sources:
  - atlas-core-domain-seams.md
kind: concept
createdAt: "2026-10-09T20:33:29.359Z"
updatedAt: "2026-10-09T20:33:29.359Z"
tags:
  - 精确线性代数
  - 行列式
  - 中心分类
aliases:
  - bareiss-行列式与余子式伴随矩阵构造
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

# Bareiss 行列式与余子式伴随矩阵构造

`domain_builtins.rs` 中的 `bareiss_det` 与 `adjugate_det` 是 alcove/FPP 助手，分别负责无分数行列式计算，以及通过余子式构造伴随矩阵。它们供[[中心分类器与根格陪集制表|中心分类器]]使用，支撑其伴随矩阵／行列式表示。^[atlas-core-domain-seams.md:49-62]

## Bareiss 行列式计算

`bareiss_det` 使用无分数 Bareiss 消元。当主元为零时，仅在当前行下方寻找可交换的行；找到后交换行并翻转行列式符号，找不到则返回行列式为零。每一步以保存于 `previous` 的上一主元作精确除法，其整除性由 Sylvester 恒等式保证。^[atlas-core-domain-seams.md:51-54]

消元完成后，结果为 `sign * a[n-1][n-1]`，其中 `sign` 记录行交换产生的符号变化。空矩阵的行列式约定为 \(1\)。^[atlas-core-domain-seams.md:51-54]

## 余子式伴随矩阵

`adjugate_det` 调用 `bareiss_det` 计算行列式，并按余子式展开构造伴随矩阵。若 \(M_{ji}\) 表示删除第 \(j\) 行、第 \(i\) 列后的子式，则
\(\operatorname{adj}(A)_{ij}=(-1)^{i+j}M_{ji}\)；这里的下标交换体现了伴随矩阵是余子式矩阵的转置。^[atlas-core-domain-seams.md:55-59]

实现的外层下标命名为 `column`，内层下标命名为 `row`，符号由 `(row+column)%2` 决定，与上述转置约定一致。当行列式非零时，逆矩阵由 \(\operatorname{adj}(A)/\det(A)\) 得到。空矩阵返回 `(vec![], 1)`。^[atlas-core-domain-seams.md:55-59]

## 算术范围与调用位置

这些助手全部使用 `i64` 算术；`adjugate_det` 的源码注释给出的适用范围为秩不超过 \(9\)。这一范围是注释中的适用说明。^[atlas-core-domain-seams.md:55-59]

调用位置包括 `CenterClassifier::new`（源码第 6105 行）及分类器内部的第 6272、6296 行：第 6272 行调用后弃用行列式，第 6296 行同时使用伴随矩阵与行列式。中心分类器材料中“adjugate／行列式表示与上游 `C_denom` 一致”的说明，具体落在这两个助手上。^[atlas-core-domain-seams.md:60-62]

## 证据边界

本说明依据当前工作区源码的结构性阅读，不构成语言或数学验收。来源中的上游 C++／CWEB 行号转述自 Rust 源码注释，未独立重读上游，因此上述实现说明不能单独作为上游兼容性验收结论。^[atlas-core-domain-seams.md:19-19, atlas-core-domain-seams.md:113-118]

## Sources

- [atlas-core-domain-seams.md](../../sources/atlas-core-domain-seams.md) — 领域层接缝：值提取器、alcove 助手与 Weyl 词／生成元校验。

---
title: Bareiss 行列式与余子式伴随矩阵构造
summary: Bareiss 消元在零主元时向下寻找换行并翻转符号，以前一主元作精确除法；伴随矩阵转置余子式，空矩阵行列式为 1，使用 i64 算术且注释限定秩不超过 9。
sources:
  - atlas-core-domain-seams.md
kind: concept
createdAt: "2026-10-09T20:33:29.359Z"
updatedAt: "2026-10-10T01:52:57.426Z"
tags:
  - 整数线性代数
  - 行列式
  - 伴随矩阵
aliases:
  - bareiss-行列式与余子式伴随矩阵构造
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# Bareiss 行列式与余子式伴随矩阵构造

`domain_builtins.rs` 中的 `bareiss_det` 与 `adjugate_det` 属于 alcove/FPP 助手，分别计算行列式和联合构造伴随矩阵与行列式。它们供[[中心分类器与根格陪集制表|中心分类器]]使用，支撑其伴随矩阵／行列式表示。^[atlas-core-domain-seams.md:49-62]

## Bareiss 行列式计算

`bareiss_det` 使用无分数 Bareiss 消元。主元为零时，只在当前行下方寻找可交换的行；找到后交换行并翻转符号，找不到则返回行列式为零。每一步以 `previous` 保存的上一主元作除数，Sylvester 恒等式保证除法精确整除。^[atlas-core-domain-seams.md:51-54]

消元结束后，结果为 `sign * a[n-1][n-1]`，其中 `sign` 记录行交换引起的符号变化。空矩阵的行列式返回 $1$。^[atlas-core-domain-seams.md:51-54]

## 余子式伴随矩阵构造

`adjugate_det` 调用 `bareiss_det` 计算行列式，并通过余子式展开构造伴随矩阵。若 $M_{ji}$ 表示删除第 $j$ 行、第 $i$ 列所得子矩阵的行列式，则伴随矩阵条目为 $\operatorname{adj}(A)_{ij}=(-1)^{i+j}M_{ji}$。下标交换体现了伴随矩阵是余子式矩阵的转置。^[atlas-core-domain-seams.md:55-59]

代码将外层下标命名为 `column`、内层下标命名为 `row`，用 `(row+column)%2` 确定符号，与上述转置约定一致。当行列式非零时，逆矩阵为 $\operatorname{adj}(A)/\det(A)$。空矩阵返回 `(vec![], 1)`。^[atlas-core-domain-seams.md:55-59]

## 算术范围与调用位置

这些助手的全部算术使用 `i64`。`adjugate_det` 的源码注释给出的适用范围为秩不超过 $9$；该范围是注释中的适用说明。^[atlas-core-domain-seams.md:55-59]

调用位置包括 `CenterClassifier::new`（源码第 6105 行）以及分类器内部的第 6272、6296 行。其中，第 6272 行丢弃返回的行列式，第 6296 行同时使用伴随矩阵和行列式。来源将中心分类器中“adjugate／行列式表示与上游 `C_denom` 一致”的说明具体关联到这两个助手。^[atlas-core-domain-seams.md:60-62]

## 证据边界

来源属于当前工作区源码的结构性阅读，不构成语言或数学验收。材料中的上游 C++／CWEB 行号转述自 Rust 源码注释，未独立重读上游，可能随版本漂移。^[atlas-core-domain-seams.md:9-19, atlas-core-domain-seams.md:124-127]

## Sources

- [atlas-core-domain-seams.md](../../sources/atlas-core-domain-seams.md) — 领域层接缝：值提取器、alcove 助手与 Weyl 词／生成元校验（domain_builtins.rs 三处缝隙）。

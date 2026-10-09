---
title: ParabolicPieces 的抛物分解与排序键
summary: ParabolicPieces 以内部层级的极小右陪集代表元索引形成唯一分解键，其字典序用于复现上游 Weyl 元素排序及 KGB 重编号。
sources:
  - weyl-layer.md
kind: concept
createdAt: "2026-10-09T15:17:46.829Z"
updatedAt: "2026-10-09T21:14:29.087Z"
tags:
  - Weyl群
  - 抛物分解
  - 排序契约
aliases:
  - parabolicpieces-的抛物分解与排序键
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: ParabolicPieces 的抛物分解与排序键
summary: ParabolicPieces 将 Weyl 元素的唯一抛物分解编码为内部层级顺序的 piece 索引列表，其字典序复现上游 WeylElt 排序，供 involution 排序及 KGB 重编号使用。
sources:
  - weyl-layer.md
kind: concept
tags:
  - 抛物子群
  - 规范排序
  - KGB图
  - 基线对齐
---

# ParabolicPieces 的抛物分解与排序键

`ParabolicPieces` 复刻上游 transducer 的 `EltPiece` 索引，将 Weyl 元素转换为按内部层级排列的 piece 列表。该列表对应元素的唯一分解，其字典序与上游 `WeylElt::operator<` 一致，并用于 involution 排序及 KGB 重编号。^[weyl-layer.md:100-105]

## 抛物分解与键结构

`ParabolicPieces::key` 返回按 internal-level 顺序排列的 piece 索引列表，对应唯一分解 \(w = w_1\cdots w_n\)。来源将其中的 \(w_i\) 描述为右陪集 \(W_{i-1}.w\) 的最小代表元；比较这些索引列表的字典序，即得到上游的 Weyl 元素排序。^[weyl-layer.md:100-104]

## 内部生成元顺序

piece 索引采用 `WeylInterface::new(cartan)` 保存的内部生成元重编号。Dynkin 分量按分类顺序排列；每个分量的 Bourbaki `position` 在 A/E/F/G 型中直接使用，在 B/C/D 型中反转。`outward()` 对应上游 `d_out`，给出 internal → datum 生成元的映射。^[weyl-layer.md:95-99]

同一内部顺序也决定 [[Weyl 元素的规范词]]：`canonical_word` 逐次剥离最小内部左下降生成元，得到内部生成元顺序下字典序最小的约化词，并检查每步长度恰好减一，否则返回 `WeylElementInvariantViolation`。规范词的选取与 `ParabolicPieces` 的内部序 piece 索引，是本移植保留内部重编号的两种可观察效果。^[weyl-layer.md:89-99]

## 排序语义与用途

piece 列表的字典序作为 `Cartan_orbits::comparer` 对 involution 排序时的次级判据（tie-break），供 KGB 图重编号原样使用。这一排序关系连接了 Weyl 元素的组合表示与后续编号，可结合 [[Twisted involution 表与 Cartan 轨道存储]]、[[KGB 图与弱实形式]] 阅读。^[weyl-layer.md:100-105]

## 证据边界

来源将 `EltPiece` 索引对应到 `weyl.cpp:289-416`，将 involution 排序对应到 `involutions.cpp:420-428`。这些上游位置均转述自所读源码注释，未独立重读上游代码，行号可能随版本演进而漂移。^[weyl-layer.md:100-105, weyl-layer.md:115-116]

本页依据结构性源码阅读说明分解、排序及兼容关系。来源包未执行构建、测试或原版运行，不提供数学验收、性能或并行结论；Weyl 层的正确性仍属于其自身的 HPC 证据链。^[weyl-layer.md:9-12, weyl-layer.md:121-121]

## Sources

- [weyl-layer.md](../../sources/weyl-layer.md) — Weyl 群层：矩阵作用与词级元素的双层结构。

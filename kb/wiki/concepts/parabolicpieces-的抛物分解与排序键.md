---
title: ParabolicPieces 的抛物分解与排序键
summary: ParabolicPieces 以内部层级的极小右陪集代表元索引编码唯一分解，其字典序用于复现上游 Weyl 元素排序及 KGB 重编号。
sources:
  - weyl-layer.md
kind: concept
createdAt: "2026-10-09T15:17:46.829Z"
updatedAt: "2026-10-10T00:56:29.256Z"
tags:
  - Weyl群
  - 抛物分解
  - 排序
aliases:
  - parabolicpieces-的抛物分解与排序键
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: ParabolicPieces 的抛物分解与排序键
summary: ParabolicPieces 将 Weyl 元素的唯一抛物分解编码为内部层级顺序的 piece 索引列表，其字典序复现上游 WeylElt 排序，供对合排序及 KGB 重编号使用。
sources:
  - weyl-layer.md
kind: concept
tags:
  - Weyl群
  - 抛物分解
  - 排序
---

# ParabolicPieces 的抛物分解与排序键

`ParabolicPieces` 复刻上游 transducer 的 `EltPiece` 索引，以按内部层级排列的 piece 列表表示 Weyl 元素的唯一分解。列表的字典序对应上游 `WeylElt::operator<`，用于对合排序，并由 KGB 图重编号原样使用。^[weyl-layer.md:100-105]

## 抛物分解与键结构

`ParabolicPieces::key` 返回按 internal-level 顺序排列的 piece 索引列表，对应唯一分解 \(w=w_1\cdots w_n\)。来源将 \(w_i\) 描述为右陪集 \(W_{i-1}.w\) 的最小代表元；比较这些列表的字典序，即复现上游 Weyl 元素的排序规则。^[weyl-layer.md:100-104]

## 内部生成元顺序

piece 索引采用的内部顺序由 `WeylInterface::new(cartan)` 保存的生成元重编号决定：Dynkin 分量按分类顺序排列，各分量的 Bourbaki `position` 在 A/E/F/G 型中直接使用，在 B/C/D 型中反转。`outward()` 对应上游 `d_out`，给出 internal → datum 生成元的映射。^[weyl-layer.md:95-99]

同一内部顺序也用于 [[Weyl 元素的规范词]]。`canonical_word` 逐次剥离最小内部左下降生成元，得到内部生成元顺序下字典序最小的约化词；每步长度必须恰好减一，否则报 `WeylElementInvariantViolation`。规范词的选择与 `ParabolicPieces` 的内部序 piece 索引，是本移植保留内部重编号的两种可观察效果。^[weyl-layer.md:89-99]

## 排序用途

piece 列表的字典序是 `Cartan_orbits::comparer` 对 involution 排序时的次级判据（tie-break），供 KGB 图重编号使用。相关背景可参见 [[Twisted involution 表与 Cartan 轨道存储]] 和 [[KGB 图与弱实形式]]。^[weyl-layer.md:100-105]

## 证据边界

来源将 `EltPiece` 索引对应到 `weyl.cpp:289-416`，将对合排序对应到 `involutions.cpp:420-428`。这些上游行号转述自源码注释，未独立重读上游文件，可能随版本演进而漂移。^[weyl-layer.md:100-105, weyl-layer.md:115-116]

本页依据结构性源码阅读说明分解与排序契约。来源包未执行构建、测试或原版运行，不提供数学验收、性能或并行结论；Weyl 层的正确性属于其自身的 [[HPC 验收证据链]]，本材料不扩展该证据范围。^[weyl-layer.md:9-12, weyl-layer.md:121-121]

## Sources

- [weyl-layer.md](../../sources/weyl-layer.md) — Weyl 群层：矩阵作用与词级元素的双层结构。

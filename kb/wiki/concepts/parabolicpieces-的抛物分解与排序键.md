---
title: ParabolicPieces 的抛物分解与排序键
summary: ParabolicPieces 用内部层级的最小右陪集代表元分解生成 piece 索引列表，其字典序复现上游 WeylElt 排序并供 involution 排序及 KGB 重编号使用。
sources:
  - weyl-layer.md
kind: concept
createdAt: "2026-10-09T15:17:46.829Z"
updatedAt: "2026-10-09T15:17:46.829Z"
tags:
  - 抛物子群
  - 规范排序
  - KGB图
  - 基线对齐
aliases:
  - parabolicpieces-的抛物分解与排序键
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# ParabolicPieces 的抛物分解与排序键

`ParabolicPieces` 复刻上游 transducer 的 `EltPiece` 索引，将 Weyl 元素转换为按内部层级排列的 piece 列表。该列表既描述元素的唯一分解，也提供与上游 `WeylElt::operator<` 一致的排序键，用于后续 involution 排序与 KGB 重编号。^[weyl-layer.md:100-105]

## 抛物分解与键结构

`ParabolicPieces::key` 返回按 internal-level 顺序排列的 piece 列表，对应唯一分解
\[
w = w_1\cdots w_n,
\]
其中 \(w_i\) 是右陪集 \(W_{i-1}.w\) 的最小代表元。键中的各项是上游 `EltPiece` 的索引；比较这些列表的字典序，即复现上游 Weyl 元素的大小关系。^[weyl-layer.md:100-104]

## 内部生成元顺序

piece 索引依赖 `WeylInterface::new(cartan)` 保存的内部生成元重编号。Dynkin 分量按分类顺序排列；每个分量的 Bourbaki `position` 在 A/E/F/G 型中直接使用，在 B/C/D 型中反转。`outward()` 对应上游 `d_out`，给出 internal → datum 生成元的映射，因此内部层级顺序需要与 datum 生成元编号区分。^[weyl-layer.md:95-99]

同一内部顺序也决定 [[Weyl 元素的规范词]]：`canonical_word` 逐次剥离最小内部左下降生成元，得到该顺序下字典序最小的约化词。规范词选取与 `ParabolicPieces` 的 piece 索引，是本移植保留上游内部重编号的两种可观察效果。^[weyl-layer.md:89-99]

## 排序语义与用途

piece 列表的字典序对应上游 `WeylElt::operator<`，并作为 `Cartan_orbits::comparer` 对 involution 排序时的次级判据（tie-break）。KGB 图的重编号直接沿用这一排序结果；因此，内部生成元顺序及 piece 索引是编号兼容性的一部分。相关主题包括 [[Twisted involution 表与 Cartan 轨道存储]] 与 [[KGB 图与弱实形式]]。^[weyl-layer.md:100-105]

## 证据边界

上述上游对应关系来自所读 Rust 源码注释：`EltPiece` 索引指向 `weyl.cpp:289-416`，involution 排序指向 `involutions.cpp:420-428`。来源包未独立重读这些上游代码，行号可能随版本变化；也未执行构建、测试或原版运行，因此本页描述的是结构与兼容性意图，不构成数学验收或性能结论。^[weyl-layer.md:100-105, weyl-layer.md:115-121]

## Sources

- [weyl-layer.md](weyl-layer.md) — Weyl 群层：矩阵作用与词级元素的双层结构。

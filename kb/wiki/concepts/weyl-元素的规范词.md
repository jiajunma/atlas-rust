---
title: Weyl 元素的规范词
summary: canonical_word 接受外部编号的任意词，包括非约化词，经 inner_mult 重建元素后按 piece 递增顺序拼接选定词并映射回外部编号，得到仅依赖元素的规范词。
sources:
  - weyl-transducer.md
kind: concept
createdAt: "2026-10-09T15:19:26.860Z"
updatedAt: "2026-10-09T15:19:26.860Z"
tags:
  - Weyl群
  - 规范词
  - 规范化算法
aliases:
  - weyl-元素的规范词
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# Weyl 元素的规范词

Weyl 元素的规范词由 `CompactWeyl::canonical_word(external_word)` 计算：输入是使用外部生成元编号表示的任意词，不要求是约化词；输出只依赖该词表示的 Weyl 元素，不依赖输入词的选取。^[weyl-transducer.md:51-56]

## 表示基础

[[Weyl 群的紧凑 Transducer 表示]]将一个 Weyl 元素存储为固定栈数组 `[u8; WEYL_MAX_RANK]`，第 \(i\) 项索引抛物子商 \(W_{i-1}\backslash W_i\) 的极小陪集代表元。规范词的构造以这些 piece 为基础，从各 piece 选定的词恢复整个元素的词。^[weyl-transducer.md:19-22, weyl-transducer.md:53-55]

## 构造过程

`canonical_word` 先通过 `inner_mult` 从输入词重建元素，再按 piece 索引递增的顺序拼接各 piece 选定的词，最后通过 `d_out` 将字母映射回外部生成元编号。这一“先重建元素、再输出选定词”的过程使结果不依赖输入词的具体表达。^[weyl-transducer.md:53-56]

内部编号与外部编号需要区分：`CompactWeyl::new(cartan)` 先分类 Dynkin 图，再反转 B/C/D 型的顺序以得到内部序，并为每个内部生成元构造 transducer。`d_out()` 承担 internal→external 的编号映射，`piece_offset(i)` 则把 piece 的局部字母转换为全局内部编号；相关背景见 [[CompactWeyl 构造与生成元编号映射]]。^[weyl-transducer.md:44-49]

## 与根置换的关系

同一 piece 表示也用于计算元素对根的作用。`piece_root_permutations` 为每个 `(transducer, piece)` 预先组合简单反射的根置换，元素的根置换再由这些置换复合得到，无需矩阵。此机制与规范词共享 piece 表示基础，详见 [[基于 Piece 的根置换预组合]]。^[weyl-transducer.md:56-58]

## 证据范围

本页依据的材料是对 `weyl_transducer.rs` 的结构性阅读，快照记录的是 dirty 工作区字节。材料中的上游 `weyl.cpp` 行号转述自源码注释，未独立重读上游；材料也未执行构建、测试或原版运行，因此不构成数学验收、性能或并行结论。^[weyl-transducer.md:9-15, weyl-transducer.md:62-68]

## Sources

- [weyl-transducer.md](weyl-transducer.md) — Compact Weyl 群的 transducer 表示。

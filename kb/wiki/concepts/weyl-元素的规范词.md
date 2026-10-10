---
title: Weyl 元素的规范词
summary: canonical_word 从任意外部编号词重建元素，按 piece 递增序拼接选定词并映回外部编号，使结果仅依赖元素本身。
sources:
  - weyl-transducer.md
kind: concept
createdAt: "2026-10-09T15:19:26.860Z"
updatedAt: "2026-10-10T00:57:36.842Z"
tags:
  - Weyl群
  - 规范词
  - 算法
aliases:
  - weyl-元素的规范词
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: Weyl 元素的规范词
summary: canonical_word 从任意外部编号词重建 Weyl 元素，按 piece 递增序拼接选定词并映回外部编号，结果只依赖元素本身。
sources:
  - weyl-transducer.md
kind: concept
tags:
  - Weyl群
  - 规范词
  - 算法
aliases:
  - weyl-元素的规范词
provenanceState: extracted
---

# Weyl 元素的规范词

Weyl 元素的规范词由 `canonical_word(external_word)` 计算。输入可以是使用外部生成元编号的任意词，不要求约化；输出只依赖该词表示的元素，不依赖输入词的选取。^[weyl-transducer.md:51-56]

## 表示基础

[[Weyl 群的紧凑 Transducer 表示]]采用 du Cloux / van Leeuwen 的抛物子商表示。一个 Weyl 元素存储为固定栈数组 `[u8; WEYL_MAX_RANK]`，第 $i$ 项索引抛物子商 $W_{i-1}\backslash W_i$ 的极小陪集代表元。规范词由这些 piece 各自选定的词按序拼接而成。^[weyl-transducer.md:19-22, weyl-transducer.md:53-55]

## 构造过程与编号

`canonical_word` 先通过 `inner_mult` 从输入词重建元素，再按 piece 索引递增的顺序拼接各 piece 选定的词，并通过 `d_out` 将字母映射回外部生成元编号。这一流程使同一元素的不同输入词得到相同的规范词。^[weyl-transducer.md:53-56]

内部编号由 `CompactWeyl::new(cartan)` 的构造流程确定：先分类 Dynkin 图，再反转 B/C/D 型的顺序以得到内部序，最后为每个内部生成元构造一个 transducer。`d_out()` 提供内部到外部的编号映射，`piece_offset(i)` 则将 piece 的局部字母转换为全局内部编号；详见 [[CompactWeyl 构造与生成元编号映射]]。^[weyl-transducer.md:44-49]

## 与根置换的关系

同一 piece 表示也支持计算元素对根的作用。`piece_root_permutations` 为每个 `(transducer, piece)` 预先组合简单反射的根置换，整个元素的根置换再由这些置换复合得到，无需矩阵。详见 [[基于 Piece 的根置换预组合]]。^[weyl-transducer.md:56-58]

## 证据范围

来源材料属于对 `weyl_transducer.rs` 的结构性阅读，阅读快照记录的是 dirty 工作区字节。材料中的上游 `weyl.cpp` 行号转述自源码注释，未独立重读上游，可能随版本演进而漂移。该材料未执行构建、测试或原版运行，不提供数学验收、性能或并行结论。^[weyl-transducer.md:9-15, weyl-transducer.md:62-68]

## Sources

- [weyl-transducer.md](../../sources/weyl-transducer.md) — Compact Weyl 群的 transducer 表示。

---
title: 基于 Piece 的根置换预组合
summary: piece_root_permutations 为每个 transducer 的各个 piece 预组合简单反射根置换，再通过置换复合得到元素的根作用，无需矩阵表示。
sources:
  - weyl-transducer.md
kind: concept
createdAt: "2026-10-09T15:19:25.935Z"
updatedAt: "2026-10-09T19:38:50.735Z"
tags:
  - 根系
  - 置换
  - 预计算
aliases:
  - 基于-piece-的根置换预组合
  - 基P的
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# 基于 Piece 的根置换预组合

基于 Piece 的根置换预组合是紧凑 Weyl 群表示中构造元素根作用的方法。`piece_root_permutations` 为每个 `(transducer, piece)` 预组合一个根置换，其内容是简单反射根置换的复合；整个 Weyl 元素的根置换再由这些预组合置换复合得到，无需矩阵。^[weyl-transducer.md:56-58]

## 表示基础

[[Weyl 群的紧凑 Transducer 表示]]采用 du Cloux / van Leeuwen 的抛物子商表示。一个 Weyl 元素存储为固定栈数组 `[u8; WEYL_MAX_RANK]`，第 \(i\) 项索引抛物子商 \(W_{i-1}\backslash W_i\) 的极小陪集代表元。这种分解为按 piece 预组合根置换提供了表示基础。^[weyl-transducer.md:19-22, weyl-transducer.md:56-58]

每个抛物子商对应一个 `Transducer`。`CompactWeyl::new(cartan)` 先分类 Dynkin 图，再反转 B/C/D 型的生成元顺序得到内部序，最后为每个内部生成元构造 transducer。`piece_offset(i)` 将 piece 的局部字母转换为全局内部编号，`d_out()` 将内部编号映射为外部编号，参见 [[CompactWeyl 构造与生成元编号映射]]。^[weyl-transducer.md:44-49]

## 预组合与元素根作用

构造分为两个层次：首先，为各个 `(transducer, piece)` 组合其简单反射对应的根置换；随后，将元素所对应的 piece 根置换复合为元素的根置换。这里直接使用根置换的复合完成作用构造，不需要通过矩阵计算元素的根作用。^[weyl-transducer.md:56-58]

## 与规范词的关系

[[Weyl 元素的规范词]]也利用同一 piece 分解。`canonical_word(external_word)` 接受任意外部编号词，不要求输入已经约化；它经 `inner_mult` 重建元素，再按 piece 递增顺序拼接选定的 piece words，并通过 `d_out` 将字母映射回外部编号。所得规范词仅依赖元素本身，不依赖输入词的选取；根置换预组合则利用各 piece 的简单反射置换构造元素作用。^[weyl-transducer.md:53-58]

## 证据边界

来源是对 `weyl_transducer.rs` 的结构性阅读，所读字节记录于 dirty 工作区快照。模块正确性归属于其自身的 [[HPC 验收证据链]]，包括 capacity gate；来源未执行构建、测试或原版运行，因此本文不提供数学验收、实测性能或并行能力结论。^[weyl-transducer.md:9-15, weyl-transducer.md:68-72]

## Sources

- [weyl-transducer.md](weyl-transducer.md) — Compact Weyl 群的 transducer 表示。

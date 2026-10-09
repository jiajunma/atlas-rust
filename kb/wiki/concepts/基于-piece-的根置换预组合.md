---
title: 基于 Piece 的根置换预组合
summary: piece_root_permutations 为每个 transducer 的各个 piece 预组合简单反射根置换，再通过置换复合获得元素的根作用，无需矩阵表示。
sources:
  - weyl-transducer.md
kind: concept
createdAt: "2026-10-09T15:19:25.935Z"
updatedAt: "2026-10-09T21:15:51.128Z"
tags:
  - 根置换
  - 预计算
  - Weyl群
aliases:
  - 基于-piece-的根置换预组合
  - 基P的
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: 基于 Piece 的根置换预组合
summary: piece_root_permutations 为每个 transducer 的各个 piece 预组合简单反射根置换，再通过置换复合得到元素的根作用，无需矩阵表示。
sources:
  - weyl-transducer.md
kind: concept
tags:
  - 根系
  - 置换
  - 预计算
aliases:
  - 基于-piece-的根置换预组合
provenanceState: extracted
---

# 基于 Piece 的根置换预组合

基于 Piece 的根置换预组合是[[Weyl 群的紧凑 Transducer 表示]]中构造元素根作用的方法。`piece_root_permutations` 为每个 `(transducer, piece)` 预组合一个根置换，即对应简单反射根置换的复合；整个 Weyl 元素的根置换再由这些置换复合得到，无需矩阵。^[weyl-transducer.md:56-58]

## 表示基础与编号

紧凑 Weyl 群采用 du Cloux / van Leeuwen 的抛物子商表示。一个元素存储为固定栈数组 `[u8; WEYL_MAX_RANK]`，第 \(i\) 项索引抛物子商 \(W_{i-1}\backslash W_i\) 的极小陪集代表元。`WEYL_MAX_RANK = 32` 是表示上界，并非元素枚举预算。^[weyl-transducer.md:19-22, weyl-transducer.md:28-32]

每个抛物子商对应一个 `Transducer`。`CompactWeyl::new(cartan)` 先分类 Dynkin 图，再反转 B/C/D 型得到内部生成元顺序，最后为每个内部生成元构造 transducer。`piece_offset(i)` 将 piece 的局部字母转换为全局内部编号，`d_out()` 将内部编号映射为外部编号，参见[[CompactWeyl 构造与生成元编号映射]]。^[weyl-transducer.md:44-49]

## 两层置换复合

预组合在 piece 层完成：对每个 `(transducer, piece)`，将简单反射对应的根置换复合为一个可用于元素作用构造的根置换。元素层随后复合其各个 piece 的根置换，得到整个元素对根的置换。两个层次均以根置换复合为基础。^[weyl-transducer.md:56-58]

## 与规范词的关系

[[Weyl 元素的规范词]]也使用 piece words。`canonical_word(external_word)` 接受任意外部编号词，不要求输入已经约化；它通过 `inner_mult` 重建元素，再按 piece 递增顺序拼接选定的 piece words，并通过 `d_out` 将字母映射回外部编号。结果仅依赖元素本身，不依赖输入词的选取。^[weyl-transducer.md:53-56]

规范词流程给出元素的词表示，`piece_root_permutations` 则预组合各 piece 对应的简单反射根置换，用于得到元素的根作用。^[weyl-transducer.md:53-58]

## 证据边界

来源是对 `weyl_transducer.rs` 的结构性阅读，所读字节记录于 `2026-10-03-weyl-transducer.json` 快照，属于 dirty 工作区。模块正确性另有自身的 HPC 证据链，包括 capacity gate；该来源不重述或扩展这些证据。^[weyl-transducer.md:9-15]

来源未执行构建、测试或原版运行，不包含数学验收、性能或并行结论。其上游 `weyl.cpp` 行号转述自源码注释，未经独立重读核对，可能随版本演进发生漂移。^[weyl-transducer.md:65-68]

## Sources

- [weyl-transducer.md](../../sources/weyl-transducer.md) — Compact Weyl 群的 transducer 表示。

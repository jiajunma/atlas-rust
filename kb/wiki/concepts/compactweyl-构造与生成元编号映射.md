---
title: CompactWeyl 构造与生成元编号映射
summary: CompactWeyl 从 Cartan 矩阵分类 Dynkin 图，反转 B/C/D 型的生成元次序，再逐内部生成元构造 transducer；d_out 与 piece_offset 分别处理内外编号及局部到全局内部编号的映射。
sources:
  - weyl-transducer.md
kind: concept
createdAt: "2026-10-09T15:19:25.783Z"
updatedAt: "2026-10-09T15:19:25.783Z"
tags:
  - Weyl群
  - 构造算法
  - 编号映射
aliases:
  - compactweyl-构造与生成元编号映射
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# CompactWeyl 构造与生成元编号映射

`CompactWeyl` 使用 du Cloux / van Leeuwen 的 transducer（parabolic-subquotient）表示。Weyl 元素由固定栈数组表示，第 \(i\) 项索引抛物子商 \(W_{i-1}\backslash W_i\) 的极小陪集代表元；构造过程为内部生成元建立相应的 transducer，并提供内部编号与外部编号之间的映射。^[weyl-transducer.md:19-22, weyl-transducer.md:44-49]

## 构造流程

`CompactWeyl::new(cartan)` 分三步完成构造：首先分类 Dynkin 图，其次反转 B、C、D 型的生成元顺序以得到内部序，最后为每个内部生成元构造一个 transducer。内部序因此需要与外部编号明确区分。^[weyl-transducer.md:44-49]

每个抛物子商对应一个 `Transducer`，保存 `offset`、`limit`、`lengths`、`rights` 和平铺的 `table`。表项小于 `size` 时表示 shift；大于或等于 `size` 时表示 transduction，输出生成元由 `out = entry - size` 解码。详见 [[Transducer 转移表编码]]。^[weyl-transducer.md:44-46]

## 生成元编号映射

`d_out()` 提供 **internal → external** 的生成元编号映射；`piece_offset(i)` 则将 piece 的局部字母翻译为全局内部编号。这两个接口分别处理内部序到外部序的转换，以及局部编号到全局内部编号的转换。^[weyl-transducer.md:48-49]

相关的 `coxeter_entry(letter, i, j)` 按连通分型的类型字母和 Bourbaki 序生成元返回 Coxeter 矩阵项。它先交换下标使 \(a \le b\)，再对线性图按下标差分派，对 D、E 型按分叉规则处理。其具体规则见 [[Coxeter 矩阵的分型查表]]。^[weyl-transducer.md:35-40]

## 规范词中的编号转换

`canonical_word(external_word)` 接受外部编号的任意词，输入不必是约化词。它通过 `inner_mult` 重建元素，再按 piece 递增顺序拼接各 piece 的选定词，最后通过 `d_out` 将字母映射回外部编号。因此，输出规范词只依赖元素本身，不依赖输入词的选取；参见 [[Weyl 群生成元的规范词构造]]。^[weyl-transducer.md:51-58]

## 表示边界与证据范围

`WeylElt = [u8; WEYL_MAX_RANK]`，其中 `WEYL_MAX_RANK = 32`。该常量是表示上界，不是元素枚举预算；源码注释以 complex rank 6 使用 12 个 pieces 为例说明这一点。相关容量约束见 [[WeylElt 的固定数组与容量边界]]。^[weyl-transducer.md:28-33]

本页依据结构性源码阅读材料。材料记录的源码字节来自 dirty 工作区；其中上游 `weyl.cpp` 行号仅转述自源码注释，未独立重读上游。该材料未执行构建、测试或原版运行，不能据此宣称数学验收、性能或并行结论。^[weyl-transducer.md:9-15, weyl-transducer.md:60-68]

## Sources

- [weyl-transducer.md](weyl-transducer.md) — Compact Weyl 群的 transducer 表示。

---
title: 基于 Piece 的根置换预组合
summary: piece_root_permutations 为每个 transducer 的各个 piece 预组合简单反射的根置换，再以这些置换的复合构造元素的根作用，无需矩阵表示。
sources:
  - weyl-transducer.md
kind: concept
createdAt: "2026-10-09T15:19:25.935Z"
updatedAt: "2026-10-09T15:19:25.935Z"
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
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 基于 Piece 的根置换预组合

基于 Piece 的根置换预组合是紧凑 Weyl 群表示中构造元素根置换的方法：`piece_root_permutations` 为每个 `(transducer, piece)` 预先组合一个根置换，各置换由简单反射的根置换复合而成；随后通过复合这些预组合结果得到整个 Weyl 元素的根置换，无需矩阵。^[weyl-transducer.md:56-58]

## 表示基础

[[Weyl 群的紧凑 Transducer 表示]]采用 du Cloux / van Leeuwen 的 parabolic-subquotient 表示。一个 Weyl 元素存储为固定栈数组 `[u8; WEYL_MAX_RANK]`，第 \(i\) 项索引抛物子商 \(W_{i-1}\backslash W_i\) 的极小陪集代表元。Piece 因而是元素表示的组成单位，也是根置换预组合的单位。^[weyl-transducer.md:19-22, weyl-transducer.md:56-58]

每个抛物子商对应一个 `Transducer`。构造 `CompactWeyl` 时，先分类 Dynkin 图，再反转 B/C/D 型得到内部生成元顺序，最后为每个内部生成元构造 transducer。`piece_offset(i)` 将 piece 的局部字母转换为全局内部编号，`d_out()` 则将内部编号映射到外部编号；这些映射与[[CompactWeyl 构造与生成元编号映射]]相关。^[weyl-transducer.md:44-49]

## 与规范词的关系

[[Weyl 元素的规范词]]与根置换预组合都利用 piece 分解。`canonical_word(external_word)` 接受任意外部编号词，不要求输入已约化；它经 `inner_mult` 重建元素，再按 piece 递增顺序拼接选定的 piece words，并通过 `d_out` 将字母映射回外部编号。所得规范词仅依赖元素本身。根置换路径则预组合各 piece 对应的简单反射作用，并将这些置换复合为元素的根置换。^[weyl-transducer.md:53-58]

## 证据边界

本概念的来源是对 `weyl_transducer.rs` 的结构性阅读，所读字节记录于 dirty 工作区快照。来源未执行构建、测试或原版运行，因此这里描述的是表示与构造机制，不构成数学验收、实测性能或并行能力结论；相关正确性应由独立的 [[HPC 验收证据链]]支持。^[weyl-transducer.md:9-15, weyl-transducer.md:68-72]

## Sources

- [weyl-transducer.md](weyl-transducer.md) — Compact Weyl 群的 transducer 表示。

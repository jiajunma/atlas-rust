---
title: Transducer 转移表编码
summary: 每个抛物子商使用一个 Transducer，平铺表中 entry<size 表示 shift，entry≥size 表示 transduction，输出生成元由 entry−size 解码。
sources:
  - weyl-transducer.md
kind: concept
createdAt: "2026-10-09T15:19:17.619Z"
updatedAt: "2026-10-09T19:38:40.891Z"
tags:
  - 状态转移
  - 表编码
  - 抛物子商
aliases:
  - transducer-转移表编码
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# Transducer 转移表编码

`Transducer` 是 [[Weyl 群的紧凑 Transducer 表示]]中对应单个抛物子商的结构。其平铺转移表 `table` 以条目值与 `size` 的大小关系区分 shift 和 transduction，并通过减去 `size` 解码 transduction 的输出编号。^[weyl-transducer.md:44-49]

## 表结构与编码规则

每个 `Transducer` 包含 `offset`、`limit`、`lengths`、`rights` 和平铺的 `table`。对于表条目 `entry`，`entry < size` 表示 shift；`entry >= size` 表示 transduction，此时输出编号为 `out = entry - size`。^[weyl-transducer.md:44-46]

相关类型定义为 `Generator = usize`、`EltPiece = u16`。转移表使用两个哨兵：`UNDEF_PIECE = u16::MAX` 与 `UNDEF_GEN = u16::MAX`。^[weyl-transducer.md:28-33]

## 子商表示与编号转换

紧凑 Weyl 元素由固定数组 `WeylElt = [u8; WEYL_MAX_RANK]` 表示，第 `i` 项索引抛物子商 $W_{i-1}\backslash W_i$ 的极小陪集代表元，乘法经由各生成元的 transducer 完成。`WEYL_MAX_RANK = 32` 是表示上界，不是元素枚举预算；相关边界见 [[WeylElt 的固定数组与容量边界]]。^[weyl-transducer.md:19-24, weyl-transducer.md:28-32]

`CompactWeyl::new(cartan)` 先分类 Dynkin 图，再反转 B/C/D 型的生成元顺序得到内部序，最后为每个内部生成元构造一个 transducer。`d_out()` 将内部编号映射到外部编号，`piece_offset(i)` 则将 piece 的局部字母转换为全局内部编号。相关构造见 [[CompactWeyl 构造与生成元编号映射]]。^[weyl-transducer.md:44-49]

## 证据边界

本文依据 `weyl_transducer.rs` 的结构性阅读材料。来源未执行构建、测试或原版运行，不提供数学验收、性能或并行结论；其中上游 `weyl.cpp` 行号转述自源码注释，未独立重读上游，可能随版本演进而漂移。^[weyl-transducer.md:9-15, weyl-transducer.md:62-68]

## Sources

- [weyl-transducer.md](weyl-transducer.md) — Compact Weyl 群的 transducer 表示。

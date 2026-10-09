---
title: Transducer 转移表编码
summary: 每个抛物子商对应一个 Transducer，其平铺转移表以 entry<size 表示 shift，以 entry≥size 表示 transduction，并通过 entry-size 解码输出生成元。
sources:
  - weyl-transducer.md
kind: concept
createdAt: "2026-10-09T15:19:17.619Z"
updatedAt: "2026-10-09T15:19:17.619Z"
tags:
  - Transducer
  - 转移表
  - 数据结构
aliases:
  - transducer-转移表编码
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# Transducer 转移表编码

`Transducer` 是 [[Weyl 群的紧凑 Transducer 表示]]中对应单个抛物子商（parabolic subquotient）的结构。其平铺转移表 `table` 用条目值与 `size` 的大小关系区分 shift 和 transduction，并在 transduction 条目中编码输出生成元。^[weyl-transducer.md:44-49]

## 表结构与编码规则

每个 `Transducer` 包含 `offset`、`limit`、`lengths`、`rights` 和平铺的 `table`。对于表条目 `entry`，当 `entry < size` 时，该条目表示 shift；当 `entry >= size` 时，该条目表示 transduction，输出编号通过 `out = entry - size` 恢复。^[weyl-transducer.md:44-46]

相关类型中，`Generator = usize`，`EltPiece = u16`；`UNDEF_PIECE` 与 `UNDEF_GEN` 均取 `u16::MAX`，用作 transducer 表的哨兵。^[weyl-transducer.md:28-33]

## 子商与编号转换

紧凑表示将一个 Weyl 元素存为固定数组，其第 `i` 项索引抛物子商 $W_{i-1}\backslash W_i$ 的极小陪集代表元；乘法通过各生成元的 transducer 完成。转移表因此服务于逐个子商的元素表示与乘法。^[weyl-transducer.md:19-24]

`CompactWeyl::new(cartan)` 先分类 Dynkin 图，再反转 B/C/D 型的生成元顺序得到内部序，最后为每个内部生成元构造一个 transducer。编号转换由两个接口承担：`d_out()` 将内部编号映射到外部编号，`piece_offset(i)` 将 piece 的局部字母转换为全局内部编号。相关构造见 [[CompactWeyl 构造与生成元编号映射]]。^[weyl-transducer.md:44-49]

## 证据边界

本文依据的是源码结构性阅读材料。来源未执行构建、测试或原版运行，不提供数学验收或性能结论；其中上游 `weyl.cpp` 行号来自 Rust 源码注释，未独立重读上游，可能随版本变化而漂移。^[weyl-transducer.md:9-15, weyl-transducer.md:65-68]

## Sources

- [weyl-transducer.md](weyl-transducer.md) — Compact Weyl 群的 transducer 表示。

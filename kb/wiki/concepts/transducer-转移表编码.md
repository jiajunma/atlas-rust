---
title: Transducer 转移表编码
summary: 每个抛物子商对应一个 Transducer，平铺表中 entry<size 表示 shift，entry≥size 表示 transduction，输出生成元由 entry−size 解码。
sources:
  - weyl-transducer.md
kind: concept
createdAt: "2026-10-09T15:19:17.619Z"
updatedAt: "2026-10-10T00:57:36.353Z"
tags:
  - Weyl群
  - 数据结构
  - 转移表
aliases:
  - transducer-转移表编码
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: Transducer 转移表编码
summary: 每个抛物子商对应一个 Transducer，平铺转移表通过 entry 与 size 的大小关系区分 shift 和 transduction，并以 entry−size 解码输出编号。
sources:
  - weyl-transducer.md
kind: concept
tags:
  - Transducer
  - 数据结构
  - 编码
aliases:
  - transducer-转移表编码
provenanceState: extracted
---

# Transducer 转移表编码

`Transducer` 是 [[Weyl 群的紧凑 Transducer 表示]]中对应单个抛物子商的结构。它将 shift 与 transduction 编码在同一张平铺转移表 `table` 中，通过条目 `entry` 与 `size` 的大小关系区分两类转移。^[weyl-transducer.md:44-46]

## 表结构与解码规则

每个 `Transducer` 包含 `offset`、`limit`、`lengths`、`rights` 和平铺的 `table`。当 `entry < size` 时，条目表示 shift；当 `entry >= size` 时，条目表示 transduction，输出编号由 `out = entry - size` 解码。^[weyl-transducer.md:44-46]

相关类型为 `Generator = usize` 和 `EltPiece = u16`，均为 `pub(crate)`。转移表使用 `UNDEF_PIECE = u16::MAX` 与 `UNDEF_GEN = u16::MAX` 作为哨兵。^[weyl-transducer.md:28-33]

## 抛物子商与编号转换

紧凑 Weyl 元素使用固定栈数组 `WeylElt = [u8; WEYL_MAX_RANK]` 表示，第 $i$ 项索引抛物子商 $W_{i-1}\backslash W_i$ 的极小陪集代表元，乘法经由各生成元的 transducer 完成。`WEYL_MAX_RANK = 32` 是表示上界，不是元素枚举预算；相关限制见 [[WeylElt 的固定数组与容量边界]]。^[weyl-transducer.md:19-22, weyl-transducer.md:28-32]

`CompactWeyl::new(cartan)` 先分类 Dynkin 图，再反转 B/C/D 型得到内部序，最后为每个内部生成元构造一个 transducer。`d_out()` 将内部编号映射到外部编号；`piece_offset(i)` 将 piece 的局部字母转换为全局内部编号。详见 [[CompactWeyl 构造与生成元编号映射]]。^[weyl-transducer.md:44-49]

## 与规范词的衔接

`canonical_word(external_word)` 接收使用外部编号的任意词，不要求输入已经约化。它经 `inner_mult` 重建元素，再按 piece 递增序拼接选定的 piece words，并通过 `d_out` 将字母映射回外部编号。结果只依赖元素本身，不依赖输入词的选取，参见 [[Weyl 元素的规范词]]。^[weyl-transducer.md:53-56]

## 证据边界

本页依据 `weyl_transducer.rs` 的结构性阅读材料，所读快照记录的是 dirty 工作区字节。模块自身的 HPC 正确性证据链未在该材料中重述或扩展；材料未执行构建、测试或原版运行，不提供数学验收、性能或并行结论。^[weyl-transducer.md:9-15, weyl-transducer.md:68-68]

来源中的上游 `weyl.cpp` 行号转述自源码注释，未独立重读上游，可能随版本演进而漂移。^[weyl-transducer.md:65-65]

## Sources

- [weyl-transducer.md](../../sources/weyl-transducer.md) — Compact Weyl 群的 transducer 表示。

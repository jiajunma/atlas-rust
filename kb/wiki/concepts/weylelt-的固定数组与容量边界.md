---
title: WeylElt 的固定数组与容量边界
summary: WeylElt 使用 [u8; WEYL_MAX_RANK] 固定栈数组，支持枚举与 twisted scan 中的零堆分配；WEYL_MAX_RANK=32 是表示上界，不能解释为元素枚举预算。
sources:
  - weyl-transducer.md
kind: concept
createdAt: "2026-10-09T15:19:00.355Z"
updatedAt: "2026-10-09T15:19:00.355Z"
tags:
  - Rust设计
  - 内存表示
  - 容量约束
aliases:
  - weylelt-的固定数组与容量边界
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# WeylElt 的固定数组与容量边界

`WeylElt` 是紧凑 Weyl 群表示中的元素类型，定义为 `[u8; WEYL_MAX_RANK]`。它采用固定栈数组，对应 C++ 的 `std::array<unsigned char, RANK_MAX>`；元素表示在枚举与 twisted scan 中无需堆分配。^[weyl-transducer.md:19-22, weyl-transducer.md:28-30]

## 数组坐标的含义

这一表示采用 du Cloux / van Leeuwen 的 transducer（parabolic-subquotient）方法。数组第 \(i\) 项索引抛物子商 \(W_{i-1}\backslash W_i\) 的极小陪集代表元，而非存储矩阵。乘法通过各生成元的 transducer 完成，源码文档将其复杂度描述为 \(O(\mathrm{length})\)。相关整体设计见 [[Weyl 群的紧凑 Transducer 表示]]。^[weyl-transducer.md:19-24]

## 表示容量与枚举预算

`WEYL_MAX_RANK = 32`，对应上游 `utilities/constants.h` 中的 `RANK_MAX`。这个常量限定固定数组的表示容量，**不是元素枚举预算**。源码注释特别指出，complex rank 6 会使用 12 个 pieces，因此不能直接把示例中的 rank 数值当作 piece 数量。^[weyl-transducer.md:31-32]

元素数组与转移表使用不同的整数类型：`WeylElt` 的数组项为 `u8`，而 `EltPiece = u16`；`Generator = usize`，这些类型均为 `pub(crate)`。转移表中的 `UNDEF_PIECE` 和 `UNDEF_GEN` 均取 `u16::MAX`，属于表的哨兵值。理解容量边界时，需要区分固定元素数组与 [[Transducer 转移表编码]] 中的类型及哨兵。^[weyl-transducer.md:28-33]

## Piece 与生成元编号

每个抛物子商对应一个 `Transducer`。`CompactWeyl::new(cartan)` 先分类 Dynkin 图，再反转 B/C/D 型以确定内部顺序，最后为每个内部生成元构造 transducer。`piece_offset(i)` 将 piece 的局部字母转换为全局内部编号，`d_out()` 则将内部编号映射为外部编号；这一编号层次由 [[CompactWeyl 构造与生成元编号映射]] 进一步说明。^[weyl-transducer.md:44-49]

固定数组中的 piece 也支撑规范词恢复：`canonical_word` 接受任意外部编号词，经 `inner_mult` 重建元素，再按 piece 递增顺序拼接选定的 piece words，并用 `d_out` 恢复外部编号。结果只依赖元素本身，不依赖输入词是否约化或如何选取，参见 [[Weyl 元素的规范词]]。^[weyl-transducer.md:53-56]

## 证据边界

本页依据的材料是结构性源码阅读，所记录的 `weyl_transducer.rs` 字节来自 dirty 工作区快照。容量门控等正确性问题属于该模块自身的 [[HPC 验收证据链]]；该材料未执行构建、测试或原版运行，因此不能据此宣称容量门控已通过验收，或作出数学正确性、实测性能与并行结论。^[weyl-transducer.md:9-15, weyl-transducer.md:68-68]

## Sources

- [weyl-transducer.md](weyl-transducer.md)

---
title: WeylElt 的固定数组与容量边界
summary: WeylElt 使用固定栈数组 [u8; 32]，枚举时无需为元素分配堆内存；32 是表示容量上界，与元素枚举预算不同。
sources:
  - weyl-transducer.md
kind: concept
createdAt: "2026-10-09T15:19:00.355Z"
updatedAt: "2026-10-10T00:57:18.922Z"
tags:
  - Weyl群
  - Rust
  - 资源限制
aliases:
  - weylelt-的固定数组与容量边界
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: WeylElt 的固定数组与容量边界
summary: WeylElt 用固定栈数组 [u8; 32] 表示 Weyl 元素，在枚举与 twisted scan 中无需为元素表示分配堆内存；32 是表示上界，不是元素枚举预算。
sources:
  - weyl-transducer.md
kind: concept
tags:
  - Rust设计
  - 存储表示
  - 容量约束
aliases:
  - weylelt-的固定数组与容量边界
provenanceState: extracted
---

# WeylElt 的固定数组与容量边界

`WeylElt` 是 [[Weyl 群的紧凑 Transducer 表示]] 使用的元素类型，定义为 `[u8; WEYL_MAX_RANK]`，其中 `WEYL_MAX_RANK = 32`。它采用固定栈数组，对应 C++ 的 `std::array<unsigned char, RANK_MAX>`；在枚举与 twisted scan 中，元素表示无需堆分配。^[weyl-transducer.md:19-22, weyl-transducer.md:28-32]

## 数组坐标的含义

该表示采用 du Cloux / van Leeuwen 的 transducer（parabolic-subquotient）方法。数组第 $i$ 项索引抛物子商 $W_{i-1}\backslash W_i$ 的极小陪集代表元；乘法通过各生成元的 transducer 完成，来源将其复杂度描述为 $O(\mathrm{length})$。^[weyl-transducer.md:19-24]

## 表示容量与枚举预算

`WEYL_MAX_RANK = 32` 对应上游 `utilities/constants.h` 的 `RANK_MAX`。源码注释明确将其限定为**表示上界，而非元素枚举预算**，并举例说明 complex rank 6 使用 12 个 pieces。理解容量时需要保留 rank 与 piece 数量的这一区别。^[weyl-transducer.md:31-32]

元素数组项与转移表相关类型也有区别：`WeylElt` 的数组项为 `u8`，`EltPiece = u16`，`Generator = usize`；这三个类型别名均为 `pub(crate)`。`UNDEF_PIECE` 与 `UNDEF_GEN` 都取 `u16::MAX`，用作 transducer 表的哨兵，参见 [[Transducer 转移表编码]]。^[weyl-transducer.md:28-33]

## Piece 与生成元编号

每个抛物子商对应一个 `Transducer`。`CompactWeyl::new(cartan)` 先分类 Dynkin 图，再反转 B/C/D 型以确定内部顺序，最后为每个内部生成元构造 transducer。`piece_offset(i)` 将 piece 的局部字母转换为全局内部编号，`d_out()` 将内部编号映射为外部编号，详见 [[CompactWeyl 构造与生成元编号映射]]。^[weyl-transducer.md:44-49]

各 piece 还用于恢复 [[Weyl 元素的规范词]]。`canonical_word(external_word)` 接受任意外部编号词，不要求输入约化；它经 `inner_mult` 重建元素，再按 piece 递增顺序拼接选定的 piece words，并通过 `d_out` 将字母映射回外部编号。结果只依赖元素本身，不依赖输入词的选择。^[weyl-transducer.md:53-56]

## 证据边界

来源属于结构性源码阅读，所读 `weyl_transducer.rs` 字节记录于 dirty 工作区快照。容量门控等正确性问题属于模块自身的 HPC 证据链，来源没有重述或扩展这些结论。^[weyl-transducer.md:9-15]

来源未执行构建、测试或原版运行，不提供数学验收、实测性能或并行结论。涉及的上游行号转述自源码注释，未独立重读上游，可能随版本演进而漂移。^[weyl-transducer.md:65-68]

## Sources

- [weyl-transducer.md](../../sources/weyl-transducer.md) — Compact Weyl 群的 transducer 表示。

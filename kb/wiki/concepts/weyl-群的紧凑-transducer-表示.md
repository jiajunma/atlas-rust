---
title: Weyl 群的紧凑 Transducer 表示
summary: 采用 du Cloux / van Leeuwen 的抛物子商表示，以固定栈数组的各项索引 W_{i-1}\W_i 的极小陪集代表元，并通过逐生成元 transducer 实现乘法；文档标注复杂度为 O(length)，本包不提供性能验收。
sources:
  - weyl-transducer.md
kind: concept
createdAt: "2026-10-09T15:19:00.139Z"
updatedAt: "2026-10-09T15:19:00.139Z"
tags:
  - Weyl群
  - 紧凑表示
  - 抛物子商
aliases:
  - weyl-群的紧凑-transducer-表示
  - W群T表
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# Weyl 群的紧凑 Transducer 表示

Weyl 群的紧凑 Transducer 表示采用 du Cloux / van Leeuwen 的抛物子商（parabolic-subquotient）方法，将群元素编码为固定数组，通过逐生成元的转移器完成乘法。Rust 模块 `weyl_transducer.rs` 实现了这一表示，对应上游 `structure/weyl.cpp`。^[weyl-transducer.md:19-24]

## 元素表示与容量边界

一个 Weyl 元素的类型为 `WeylElt = [u8; WEYL_MAX_RANK]`。数组第 \(i\) 项索引抛物子商 \(W_{i-1}\backslash W_i\) 的极小陪集代表元；这一固定栈数组对应 C++ 的 `std::array<unsigned char, RANK_MAX>`，在枚举和 twisted scan 中无需为元素表示分配堆内存。^[weyl-transducer.md:19-22, weyl-transducer.md:28-30]

`WEYL_MAX_RANK = 32` 是表示上界，不是元素枚举预算；例如 complex rank 6 使用 12 个 pieces。模块还定义 `Generator = usize`、`EltPiece = u16`，并以 `UNDEF_PIECE = u16::MAX` 和 `UNDEF_GEN = u16::MAX` 作为转移表哨兵。相关容量约束见 [[WeylElt 的固定数组与容量边界]]。^[weyl-transducer.md:28-33]

## 转移表与构造流程

每个抛物子商对应一个 `Transducer`，包含 `offset`、`limit`、`lengths`、`rights` 和平铺的 `table`。表项 `entry < size` 表示 shift；`entry >= size` 表示 transduction，输出生成元由 `out = entry - size` 解码。参见 [[Transducer 转移表编码]]。^[weyl-transducer.md:44-46]

`CompactWeyl::new(cartan)` 先分类 Dynkin 图，再反转 B/C/D 型的生成元顺序以获得内部编号，最后为每个内部生成元构造一个转移器。`d_out()` 将内部编号映射为外部编号，`piece_offset(i)` 则将 piece 的局部字母转换为全局内部编号。参见 [[CompactWeyl 构造与生成元编号映射]]。^[weyl-transducer.md:46-49]

模块的 `coxeter_entry(letter, i, j)` 根据连通分型的类型字母及 Bourbaki 序生成元返回 Coxeter 矩阵项。它先交换下标使 \(a\le b\)，再对非 D/E 型按下标距离和类型分派，对 D/E 型采用分叉规则；具体规则见 [[Coxeter 矩阵的分型查表]]。^[weyl-transducer.md:35-40]

## 规范词与根置换

`canonical_word(external_word)` 接受外部编号的任意词，输入不必约化。它先通过 `inner_mult` 重建群元素，再按 piece 递增顺序拼接选定的 piece words，并通过 `d_out` 将字母映射回外部编号。结果只依赖群元素本身，不依赖输入词的选择。参见 [[Weyl 元素的规范词]]。^[weyl-transducer.md:51-56]

`piece_root_permutations` 为每个 `(transducer, piece)` 预先组合一个根置换，该置换由简单反射的根置换复合得到。群元素的根置换再由这些预组合置换复合而成，无需使用矩阵。参见 [[基于 Piece 的根置换预组合]]。^[weyl-transducer.md:56-58]

## 效率说明与证据边界

来源将通过逐生成元转移器执行的乘法描述为 \(O(\mathrm{length})\)，并转述模块文档中的 E6 示例：对于含 51840 个元素的群，紧凑表示的整群枚举比矩阵表示便宜得多。这是文档层面的说明，不能视为该来源提供的实测性能结论。^[weyl-transducer.md:19-24]

该来源属于结构性源码阅读，所记录的 `weyl_transducer.rs` 字节来自 dirty 工作区，并有对应快照。来源未执行构建、测试或原版运行，不提供数学验收、性能或并行结论；正确性仍属于独立的 [[HPC 验收证据链]]。所引上游行号来自源码注释，未独立重读上游，可能随版本变化而漂移。^[weyl-transducer.md:9-15, weyl-transducer.md:62-68]

## Sources

- [Compact Weyl 群的 transducer 表示](weyl-transducer.md)

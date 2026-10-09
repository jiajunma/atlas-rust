---
title: Weyl 群的紧凑 Transducer 表示
summary: 以抛物子商极小陪集代表元的索引表示 Weyl 元素，并通过逐生成元 transducer 实现乘法；来源中的复杂度与枚举成本说明不构成性能验收。
sources:
  - weyl-transducer.md
kind: concept
createdAt: "2026-10-09T15:19:00.139Z"
updatedAt: "2026-10-09T21:15:29.301Z"
tags:
  - Weyl群
  - 算法
  - 紧凑表示
aliases:
  - weyl-群的紧凑-transducer-表示
  - W群T表
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: Weyl 群的紧凑 Transducer 表示
summary: 以固定数组保存抛物子商的极小陪集代表元索引，通过逐生成元 transducer 实现乘法、规范词恢复及根置换组合；结构性阅读不构成性能或数学验收。
sources:
  - weyl-transducer.md
kind: concept
tags:
  - Weyl群
  - 抛物子商
  - 紧凑表示
aliases:
  - weyl-群的紧凑-transducer-表示
  - W群T表
---

# Weyl 群的紧凑 Transducer 表示

Weyl 群的紧凑 Transducer 表示采用 du Cloux / van Leeuwen 的抛物子商（parabolic-subquotient）方法，将群元素编码为固定栈数组，通过逐生成元转移器完成乘法。Rust 模块 `weyl_transducer.rs` 实现这一表示，对应上游 `structure/weyl.cpp`。^[weyl-transducer.md:19-24]

## 元素表示与容量边界

群元素类型为 `WeylElt = [u8; WEYL_MAX_RANK]`，数组第 \(i\) 项索引抛物子商 \(W_{i-1}\backslash W_i\) 的极小陪集代表元。它对应 C++ 的 `std::array<unsigned char, RANK_MAX>`；在枚举和 twisted scan 中，这一元素表示无需堆分配。^[weyl-transducer.md:19-22, weyl-transducer.md:28-30]

`WEYL_MAX_RANK = 32` 是表示上界，不是元素枚举预算；例如 complex rank 6 使用 12 个 pieces。模块还定义 `Generator = usize`、`EltPiece = u16`，并将 `UNDEF_PIECE` 与 `UNDEF_GEN` 均设为 `u16::MAX`，用作转移表哨兵。相关约束见 [[WeylElt 的固定数组与容量边界]]。^[weyl-transducer.md:28-33]

## 转移表与构造

每个抛物子商对应一个 `Transducer`，保存 `offset`、`limit`、`lengths`、`rights` 和平铺的 `table`。表项 `entry < size` 表示 shift；`entry >= size` 表示 transduction，输出生成元由 `out = entry - size` 解码。参见 [[Transducer 转移表编码]]。^[weyl-transducer.md:44-46]

`CompactWeyl::new(cartan)` 依次分类 Dynkin 图、反转 B/C/D 型的生成元顺序以得到内部序，再为每个内部生成元构造一个转移器。`d_out()` 将内部编号映射为外部编号，`piece_offset(i)` 将 piece 局部字母转换为全局内部编号。参见 [[CompactWeyl 构造与生成元编号映射]]。^[weyl-transducer.md:46-49]

`coxeter_entry(letter, i, j)` 根据连通分型的类型字母与 Bourbaki 序生成元返回 Coxeter 矩阵项。它先交换下标使 \(a\le b\)：非 D/E 型按距离 \(b-a\) 分派，对角项为 1，距离至少为 2 时为 2，相邻项按类型取 3、4 或 6；B/C 型的 \((0,1)\) 与 F 型的 \((1,2)\) 取 4，G 型相邻项取 6。D/E 型采用分叉规则。参见 [[Coxeter 矩阵的分型查表]]。^[weyl-transducer.md:35-40]

## 规范词与根置换

`canonical_word(external_word)` 接受外部编号的任意词，输入不必约化。它先通过 `inner_mult` 重建群元素，再按 piece 递增顺序拼接选定的 piece words，并通过 `d_out` 将字母映射回外部编号。结果只依赖群元素本身，不依赖输入词的选取。参见 [[Weyl 元素的规范词]]。^[weyl-transducer.md:51-56]

`piece_root_permutations` 为每个 `(transducer, piece)` 预组合一个根置换，即相应简单反射根置换的复合。群元素的根置换再由这些预组合置换复合得到，无需矩阵。参见 [[基于 Piece 的根置换预组合]]。^[weyl-transducer.md:56-58]

## 效率说明与证据边界

来源将逐生成元转移器实现的乘法描述为 \(O(\mathrm{length})\)，并转述模块文档的 E6 示例：对于含 51840 个元素的群，这种表示的整群枚举比矩阵表示便宜得多。这属于文档说明，来源没有据此给出实测性能结论。^[weyl-transducer.md:19-24]

来源属于结构性源码阅读，所读 `weyl_transducer.rs` 字节来自 dirty 工作区，绑定快照 `snapshots/2026-10-03-weyl-transducer.json`，SHA-256 为 `4bb3e72b293e6fa9f4709c996bdef388a14ec3cb81ddce2b61d85acea2718783`。正确性归于该模块自己的 HPC 证据链，包括 capacity gate 等；本来源不重述或扩展其结论。^[weyl-transducer.md:9-15]

来源未执行构建、测试或原版运行，不包含数学验收、性能或并行结论。上游行号均转述自源码注释，未独立重读上游，可能随版本演进而漂移。^[weyl-transducer.md:60-68]

## Sources

- [Compact Weyl 群的 transducer 表示](../../sources/weyl-transducer.md)

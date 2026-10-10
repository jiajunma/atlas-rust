---
title: CompactWeyl 构造与生成元编号映射
summary: CompactWeyl 先分类 Dynkin 图，再反转 B/C/D 型内部编号并逐生成元构造 transducer，通过 d_out 与 piece_offset 转换编号。
sources:
  - weyl-transducer.md
kind: concept
createdAt: "2026-10-09T15:19:25.783Z"
updatedAt: "2026-10-10T00:57:41.678Z"
tags:
  - Weyl群
  - Dynkin图
  - 编号映射
aliases:
  - compactweyl-构造与生成元编号映射
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: CompactWeyl 构造与生成元编号映射
summary: CompactWeyl 先分类 Dynkin 图、反转 B/C/D 型得到内部序，再逐内部生成元构造 transducer；piece_offset 与 d_out 分别完成局部到全局内部、内部到外部的编号转换。
sources:
  - weyl-transducer.md
kind: concept
tags:
  - Weyl群
  - 构造算法
  - 生成元编号
aliases:
  - compactweyl-构造与生成元编号映射
provenanceState: extracted
---

# CompactWeyl 构造与生成元编号映射

`CompactWeyl` 实现 du Cloux / van Leeuwen 的 transducer（抛物子商）表示。Weyl 元素由固定栈数组表示，第 $i$ 项索引抛物子商 $W_{i-1}\backslash W_i$ 的极小陪集代表元。构造器建立内部生成元序及对应的 transducer，并通过编号映射衔接 piece 局部字母与外部生成元。^[weyl-transducer.md:19-22, weyl-transducer.md:44-49]

## 构造流程

`CompactWeyl::new(cartan)` 按顺序完成三步：分类 Dynkin 图；反转 B/C/D 型以得到内部序；为每个内部生成元构造一个 transducer。内部生成元序与外部编号通过 `d_out()` 连接。^[weyl-transducer.md:46-49]

每个抛物子商对应一个 `Transducer`，包含 `offset`、`limit`、`lengths`、`rights` 和平铺的 `table`。表项 `entry < size` 表示 shift，`entry >= size` 表示 transduction，后者按 `out = entry - size` 解码；详见 [[Transducer 转移表编码]]。^[weyl-transducer.md:44-46]

## 编号映射方向

`piece_offset(i)` 将 piece 的局部字母转换为**全局内部编号**；`d_out()` 将**内部编号转换为外部编号**。前者处理 piece 局部编号与全局内部编号之间的转换，后者处理内部序与外部序之间的转换。^[weyl-transducer.md:48-49]

相关的 `coxeter_entry(letter, i, j)` 根据连通分型的类型字母和 Bourbaki 序生成元返回 Coxeter 矩阵项。它先交换下标使 $a \le b$，再对非 D/E 型按下标差分派，对 D/E 型按分叉规则处理；详见 [[Coxeter 矩阵的分型查表]]。^[weyl-transducer.md:37-40]

## 规范词与根置换

`canonical_word(external_word)` 接受采用外部编号的任意词，输入不必约化。它先经 `inner_mult` 重建元素，再按 piece 递增顺序拼接各 piece 的选定词，并用 `d_out` 将字母映射回外部编号。结果只依赖元素本身，不依赖输入词的选取，参见 [[Weyl 元素的规范词]]。^[weyl-transducer.md:53-56]

`piece_root_permutations` 为每个 `(transducer, piece)` 预组合简单反射的根置换；元素的根置换由这些置换复合得到，无需矩阵。相关机制见 [[基于 Piece 的根置换预组合]]。^[weyl-transducer.md:56-58]

## 表示边界与证据范围

元素类型为 `WeylElt = [u8; WEYL_MAX_RANK]`，其中 `WEYL_MAX_RANK = 32`。该常量是表示上界，不是元素枚举预算；源码注释以 complex rank 6 使用 12 个 pieces 为例说明这一点。详见 [[WeylElt 的固定数组与容量边界]]。^[weyl-transducer.md:28-33]

本页依据 `weyl_transducer.rs` 的结构性阅读材料，所读字节来自 dirty 工作区，记录于 `2026-10-03-weyl-transducer.json` 快照。源材料中的上游 `weyl.cpp` 行号转述自源码注释，未独立重读上游，可能随版本演进漂移。材料未执行构建、测试或原版运行，不提供数学验收、性能或并行结论；模块正确性属于其独立的 HPC 证据链。^[weyl-transducer.md:9-15, weyl-transducer.md:62-68]

## Sources

- [weyl-transducer.md](../../sources/weyl-transducer.md) — Compact Weyl 群的 transducer 表示。

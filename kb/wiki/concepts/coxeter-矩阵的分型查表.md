---
title: Coxeter 矩阵的分型查表
summary: coxeter_entry 根据连通 Dynkin 类型及 Bourbaki 生成元编号，按线性邻接或 D/E 型分叉规则返回 Coxeter 矩阵项。
sources:
  - weyl-transducer.md
kind: concept
createdAt: "2026-10-09T15:18:59.898Z"
updatedAt: "2026-10-10T00:57:17.358Z"
tags:
  - Coxeter矩阵
  - Dynkin图
aliases:
  - coxeter-矩阵的分型查表
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: Coxeter 矩阵的分型查表
summary: coxeter_entry 按连通 Dynkin 类型及 Bourbaki 生成元编号，通过线性邻接或 D/E 型分叉规则返回 Coxeter 矩阵项。
sources:
  - weyl-transducer.md
kind: concept
tags:
  - Coxeter矩阵
  - Dynkin图
  - 生成元编号
aliases:
  - coxeter-矩阵的分型查表
provenanceState: extracted
---

# Coxeter 矩阵的分型查表

`coxeter_entry(letter, i, j)` 根据连通 Dynkin 分型的类型字母与 Bourbaki 序生成元下标，返回 Coxeter 矩阵项。它属于 [[Weyl 群的紧凑 Transducer 表示]]模块，源码注释将其对应到上游 `weyl.cpp:191-215`。^[weyl-transducer.md:19-24, weyl-transducer.md:35-40]

## 查表规则

函数先交换下标，使 \(a \le b\)。对于非 D/E 型的线性图，按下标差 \(b-a\) 分派：对角项为 1，相邻项按类型与位置取 3、4 或 6，间隔至少为 2 时取 2。具体规则如下。^[weyl-transducer.md:37-40]

| 适用情形 | 条件 | 返回值 |
| --- | --- | --- |
| 非 D/E 型的对角位置 | \(b-a=0\) | 1 |
| B/C 型的特殊相邻位置 | \((a,b)=(0,1)\) | 4 |
| F 型的特殊相邻位置 | \((a,b)=(1,2)\) | 4 |
| G 型的相邻位置 | \(b-a=1\) | 6 |
| 非 D/E 型的其余相邻位置 | \(b-a=1\) | 3 |
| 非 D/E 型的非相邻位置 | \(b-a\ge 2\) | 2 |

D/E 型另按分叉规则处理。来源未展开具体分叉下标条件，因此本页不列出 D/E 型的完整查表公式。^[weyl-transducer.md:37-40]

## 编号与构造上下文

查表使用 Bourbaki 序生成元下标。[[CompactWeyl 构造与生成元编号映射]]还涉及内部序：`CompactWeyl::new(cartan)` 先分类 Dynkin 图，再反转 B/C/D 型取得内部序，最后为每个内部生成元构造一个 transducer。`d_out()` 提供内部到外部的编号映射，`piece_offset(i)` 将 piece 局部字母转换为全局内部编号。^[weyl-transducer.md:37-49]

## 证据边界

来源属于结构性源码阅读，`coxeter_entry` 分派说明已由维护者对照 Rust 源码核对。所读字节来自 dirty 工作区，并由阅读快照记录；上游 C++ 行号仅转述自源码注释，未独立重读上游，可能随版本演进而漂移。^[weyl-transducer.md:9-15, weyl-transducer.md:62-72]

来源未执行构建、测试或原版运行，不提供数学验收、性能或并行结论。模块正确性另属其自身的 [[HPC 验收证据链]]，本页不扩展该证据范围。^[weyl-transducer.md:10-11, weyl-transducer.md:68-68]

## Sources

- [weyl-transducer.md](../../sources/weyl-transducer.md) — Compact Weyl 群的 transducer 表示。

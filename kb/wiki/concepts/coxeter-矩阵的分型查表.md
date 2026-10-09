---
title: Coxeter 矩阵的分型查表
summary: coxeter_entry 根据连通 Dynkin 类型与 Bourbaki 生成元编号，按线性图邻接关系或 D/E 型分叉规则返回 Coxeter 矩阵项。
sources:
  - weyl-transducer.md
kind: concept
createdAt: "2026-10-09T15:18:59.898Z"
updatedAt: "2026-10-09T19:38:32.338Z"
tags:
  - Coxeter矩阵
  - Dynkin图
  - 生成元编号
aliases:
  - coxeter-矩阵的分型查表
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# Coxeter 矩阵的分型查表

`coxeter_entry(letter, i, j)` 根据连通 Dynkin 分型的类型字母与 Bourbaki 序生成元下标，返回对应的 Coxeter 矩阵项。它属于 [[Weyl 群的紧凑 Transducer 表示]]的实现，源码注释对应上游 `weyl.cpp:191-215`。^[weyl-transducer.md:19-24, weyl-transducer.md:35-40]

## 查表规则

函数先交换下标，使 \(a \le b\)。对于非 D/E 型的线性图，随后按下标差 \(b-a\) 分派：对角项为 1，相邻项按类型取 3、4 或 6，间隔至少为 2 时取 2。^[weyl-transducer.md:37-40]

| 适用情形 | 条件 | 返回值 |
| --- | --- | --- |
| 非 D/E 型线性图 | \(b-a=0\) | 1 |
| B/C 型的特殊相邻位置 | \((a,b)=(0,1)\) | 4 |
| F 型的特殊相邻位置 | \((a,b)=(1,2)\) | 4 |
| G 型的相邻位置 | \(b-a=1\) | 6 |
| 非 D/E 型的其余相邻位置 | \(b-a=1\) | 3 |
| 非 D/E 型线性图 | \(b-a\ge 2\) | 2 |

上述相邻项的特殊值由类型及位置共同决定。D/E 型另按分叉规则处理；来源未展开具体的分叉下标条件。^[weyl-transducer.md:37-40]

## 编号与构造上下文

查表使用 Bourbaki 序生成元下标。[[CompactWeyl 构造与生成元编号映射]]还涉及内部编号：`CompactWeyl::new(cartan)` 先分类 Dynkin 图，再反转 B/C/D 型以取得内部序，最后为每个内部生成元构造一个 transducer。`d_out()` 提供内部到外部的编号映射，`piece_offset(i)` 将 piece 局部字母转换为全局内部编号。^[weyl-transducer.md:37-49]

## 证据边界

来源中关于 `coxeter_entry` 分派的说明已由维护者对照 Rust 源码核对。上游 C++ 行号仅转述自源码注释，未独立重读上游，可能随版本变化。来源未执行构建、测试或原版运行，因此不提供数学验收、性能或并行结论。^[weyl-transducer.md:65-72]

## Sources

- [weyl-transducer.md](weyl-transducer.md) — Compact Weyl 群的 transducer 表示。

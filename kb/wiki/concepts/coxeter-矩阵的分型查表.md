---
title: Coxeter 矩阵的分型查表
summary: coxeter_entry 根据连通 Dynkin 分型及 Bourbaki 生成元编号计算 Coxeter 矩阵项，在线性图中按编号距离与类型分派，并为 D/E 型采用分叉规则。
sources:
  - weyl-transducer.md
kind: concept
createdAt: "2026-10-09T15:18:59.898Z"
updatedAt: "2026-10-09T15:18:59.898Z"
tags:
  - Coxeter矩阵
  - Dynkin图
  - 查表算法
aliases:
  - coxeter-矩阵的分型查表
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# Coxeter 矩阵的分型查表

`coxeter_entry(letter, i, j)` 根据连通分型的类型字母和 Bourbaki 序生成元下标，返回对应的 Coxeter 矩阵项。该函数位于 Compact Weyl 的 transducer 实现中，源码注释对应上游 `weyl.cpp:191-215`。^[weyl-transducer.md:35-40]

## 查表规则

函数先交换下标，使其满足 \(a \le b\)。对于非 D/E 型的线性图，按下标差 \(b-a\) 分派：相同下标返回 1，相邻下标按类型返回 3、4 或 6，间隔至少为 2 时返回 2。^[weyl-transducer.md:37-40]

| 适用情形 | 条件 | 返回值 |
| --- | --- | --- |
| 非 D/E 型线性图 | \(b-a=0\) | 1 |
| 非 D/E 型线性图 | \(b-a=1\) | 按类型取 3、4 或 6 |
| 非 D/E 型线性图 | \(b-a\ge 2\) | 2 |
| B/C 型的相邻位置 | \((a,b)=(0,1)\) | 4 |
| F 型的相邻位置 | \((a,b)=(1,2)\) | 4 |
| G 型的相邻位置 | \(b-a=1\) | 6 |

表中的特殊相邻位置决定何时取 4 或 6；D/E 型则按分叉规则处理。提供的来源未展开 D/E 型的具体分叉下标条件。^[weyl-transducer.md:37-40]

## 编号与构造上下文

查表使用 Bourbaki 序生成元下标。[[CompactWeyl 构造与生成元编号映射]]还涉及另一层内部编号：`CompactWeyl::new(cartan)` 先分类 Dynkin 图，再反转 B/C/D 型以取得内部序，最后为每个内部生成元构造 transducer；`d_out()` 提供 internal→external 编号映射，`piece_offset(i)` 将 piece 局部字母转换为全局内部编号。^[weyl-transducer.md:37-40, weyl-transducer.md:44-49]

该查表属于[[Weyl 群的紧凑 Transducer 表示]]的实现内容。模块采用 du Cloux / van Leeuwen 的 parabolic-subquotient 表示，用固定数组记录各层极小陪集代表元的索引，并通过逐生成元 transducer 完成乘法。^[weyl-transducer.md:19-24]

## 证据边界

来源对 `coxeter_entry` 分派的说明已由维护者对照 Rust 源码核对；上游 C++ 行号仅转述自源码注释，未独立重读上游，可能随版本变化。该来源未执行构建、测试或原版运行，因此本页不据此声称数学验收或性能结论。^[weyl-transducer.md:65-72]

## Sources

- [weyl-transducer.md](weyl-transducer.md)

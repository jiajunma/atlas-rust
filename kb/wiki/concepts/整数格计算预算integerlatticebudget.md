---
title: 整数格计算预算（IntegerLatticeBudget）
summary: 以矩阵维数、存活工作条目、初等操作次数及中间系数位长约束单次精确计算，属于计算预算而非数学秩限制。
sources:
  - integer-lattice.md
kind: concept
createdAt: "2026-10-09T14:51:18.596Z"
updatedAt: "2026-10-09T22:32:14.232Z"
tags:
  - 整数格
  - 资源预算
aliases:
  - 整数格计算预算integerlatticebudget
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: 整数格计算预算（IntegerLatticeBudget）
summary: 以矩阵维数、存活工作条目总量、初等操作次数和中间系数位长约束单次精确整数格计算，属于计算预算而非数学秩限制。
sources:
  - integer-lattice.md
kind: concept
tags:
  - 整数格
  - 资源预算
aliases:
  - 整数格计算预算integerlatticebudget
provenanceState: extracted
---

# 整数格计算预算（IntegerLatticeBudget）

`IntegerLatticeBudget` 是精确整数格线性代数的计算预算类型，用于约束单次计算的资源使用，**不构成数学秩限制**。相关基础设施基于 Malachite 大整数 `Integer`，采用预算先行的设计：构造与收集入口先校验形状或预算，再触碰存储。^[integer-lattice.md:19-27]

## 预算维度

预算包含四个私有字段，构造函数 `new` 声明为 `pub const fn`。四个字段分别约束每个矩阵的维数、存活工作条目总量、初等操作次数与中间系数位长。^[integer-lattice.md:24-27]

| 字段 | 约束对象 |
| --- | --- |
| `max_rank` | 单次计算中每个矩阵的维数 |
| `max_entries` | 存活工作条目总量 |
| `max_steps` | 初等操作次数 |
| `max_coefficient_bits` | 中间系数位长 |

## 校验与存储操作的顺序

`IntegerMatrix` 是行主序精确整数矩阵。其 `from_i32_entries` 先通过 `checked_shape` 校验形状，再分配存储并逐元素转换；形状不符时报告 `InvalidIntegerMatrixShape`。^[integer-lattice.md:31-35]

[[关系格封装与构造预检]] 将检查提前到分配、复制、迭代和收集之前。`preflight_shape` 在分配或复制任何条目前校验形状；`from_i32_iter` 在推进迭代器或保留存储之前拒绝超大形状；`RelationGenerator::try_collect` 仅在借用生成元描述符的计数及其蕴含的分子矩阵通过关系格预算后才执行收集。^[integer-lattice.md:49-53]

## 算法中的预算使用

`saturated_kernel(matrix, budget)` 先执行预算检查，再进行保持幺模右因子 \(V\) 的行列混合约化。矩阵对角化后，\(V\) 中对应零对角元的列构成完整整数核；实现有意不采用“有理行约化再通分”的方法。^[integer-lattice.md:39-41]

[[承载可观测量的适配基（adapted_basis）|adapted_basis]] 在消元前预检工作矩阵、基、逆及对角线的合计条目数，再逐行执行 `gcd_row_to_pivot`，并循环调用 `find_small_remainder` 消元。这里的条目预算覆盖多份工作数据。^[integer-lattice.md:57-64]

该适配基固定 [[stable_log：选举的稳定对数|stable_log]] 代表元，进而固定 `g_rho_check` 与下游每个 `torus_factor` 有理量，因此所选基承载可观测量。实现伴随跟踪 LEFT 变换的逆，并保留上游主元策略；预算预检是这一具体算法实现的一部分。^[integer-lattice.md:57-64]

## 证据边界

来源材料属于对 `integer_lattice.rs` 的结构性阅读，所读字节记录于 `2026-10-03-integer-lattice.json`，来自 dirty 工作区。正确性归属独立的 [[HPC 验收证据链]]，包括 fundamental-lattice 与 torus gate 等；来源包不重述或扩展这些证据。^[integer-lattice.md:9-15]

来源包未执行构建、测试或原版运行，不包含数学验收、性能或并行结论。上游 `matreduc.h`／`matreduc.cpp` 的行号转述自源码注释，未独立重读上游，可能随版本演进而漂移。^[integer-lattice.md:70-80]

## Sources

- [integer-lattice.md](../../sources/integer-lattice.md) — 精确整数格线性代数：预算、饱和核与可观测基。

---
title: 整数格计算预算（IntegerLatticeBudget）
summary: 以矩阵维数、存活工作条目总量、初等操作次数和中间系数位长限制单次精确计算；这些约束属于计算预算，不代表数学秩限制。
sources:
  - integer-lattice.md
kind: concept
createdAt: "2026-10-09T14:51:18.596Z"
updatedAt: "2026-10-09T14:51:18.596Z"
tags:
  - 整数格
  - 资源预算
  - 精确算术
aliases:
  - 整数格计算预算integerlatticebudget
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 整数格计算预算（IntegerLatticeBudget）

`IntegerLatticeBudget` 是精确整数格线性代数的计算资源约束类型。相关运算基于 Malachite 大整数 `Integer`；预算约束的是单次计算的资源使用，而不是数学上允许的秩。该基础设施采用预算先行的设计：构造与收集入口先检查形状或预算，再操作存储。^[integer-lattice.md:19-27]

## 预算维度

预算包含四个私有字段，分别约束矩阵维数、存活工作条目总量、初等操作次数和中间系数位长；构造函数 `new` 声明为 `pub const fn`。^[integer-lattice.md:24-27]

| 字段 | 约束对象 |
| --- | --- |
| `max_rank` | 单次计算中每个矩阵的维数 |
| `max_entries` | 存活工作条目的总量 |
| `max_steps` | 初等操作次数 |
| `max_coefficient_bits` | 中间系数位长 |

这里的 `max_rank` 应按计算预算理解，不能将其解释为整数格理论的秩限制。^[integer-lattice.md:24-27]

## 校验与分配顺序

`IntegerMatrix::from_i32_entries` 先通过 `checked_shape` 校验形状，再分配存储并逐元素转换；形状不符时报告 `InvalidIntegerMatrixShape`。这一顺序体现了在分配之前检查输入的约定。^[integer-lattice.md:31-35]

[[关系格封装与构造预检]]进一步将检查放在复制、迭代和收集之前：`preflight_shape` 在分配或复制任何条目前校验形状；`from_i32_iter` 在推进迭代器或保留存储之前拒绝超大形状；`RelationGenerator::try_collect` 只有在生成元描述符计数及其蕴含的分子矩阵通过关系格预算后才执行收集。^[integer-lattice.md:47-53]

## 算法中的预算使用

`saturated_kernel(matrix, budget)` 先执行预算检查，再进行保持幺模右因子 \(V\) 的行列混合约化。矩阵对角化后，\(V\) 中对应零对角元的列构成完整整数核；该算法有意不采用“有理行约化再通分”的方法。^[integer-lattice.md:39-41]

[[承载可观测量的适配基（adapted_basis）|adapted_basis]] 在消元前预检工作矩阵、基、逆及对角线的条目总量，然后逐行执行 `gcd_row_to_pivot`，并循环调用 `find_small_remainder` 消元。因此，其条目预算覆盖这些工作数据的总量。^[integer-lattice.md:55-64]

## 证据边界

来源包属于对 `integer_lattice.rs` 的结构性阅读，所记录字节来自 dirty 工作区快照。该包未执行构建、测试或原版运行，因此本页描述的是预算设计与实现行为，不构成数学验收、性能或并行结论；相关正确性证据仍属于独立的 [[HPC 验收证据链]]。^[integer-lattice.md:9-15, integer-lattice.md:80-85]

## Sources

- [integer-lattice.md](integer-lattice.md) — 精确整数格线性代数：预算、饱和核与可观测基。

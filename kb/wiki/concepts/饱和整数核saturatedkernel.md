---
title: 饱和整数核（saturated_kernel）
summary: 在预算检查后执行跟踪幺模右因子 V 的行列混合约化，以 V 中对应零对角元的列生成完整整数核。
sources:
  - integer-lattice.md
kind: concept
createdAt: "2026-10-09T14:51:31.058Z"
updatedAt: "2026-10-09T19:30:16.312Z"
tags:
  - 整数格
  - 饱和核
  - 矩阵约化
aliases:
  - 饱和整数核saturatedkernel
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# 饱和整数核（saturated_kernel）

`saturated_kernel(matrix, budget)` 计算饱和整数核的基，属于 `integer_lattice.rs` 中基于 Malachite 大整数 `Integer` 的精确格线性代数功能。它先检查计算预算，再通过行列混合约化跟踪幺模右因子，最终得到完整整数核。^[integer-lattice.md:19-20, integer-lattice.md:39-41]

## 约化方法

算法在行列混合约化过程中保持幺模右因子 \(V\)。矩阵对角化后，\(V\) 中对应零对角元的列构成完整整数核的基。实现刻意不采用“先做有理行约化，再通分”的方法。^[integer-lattice.md:39-41]

## 矩阵输入与计算预算

底层 `IntegerMatrix` 是 Malachite 整数上的行主序精确矩阵，公开 `rows`、`columns` 和 `entries`。其 `from_i32_entries` 构造器先经 `checked_shape` 校验形状，再分配存储并逐元素转换；形状不符时返回 `InvalidIntegerMatrixShape`。相关表示见 [[精确整数矩阵与 Bézout 幺模变换]]。^[integer-lattice.md:31-35]

[[整数格计算预算（IntegerLatticeBudget）]] 约束单次计算的每个矩阵维数（`max_rank`）、存活工作条目总量（`max_entries`）、初等操作次数（`max_steps`）和中间系数位长（`max_coefficient_bits`）。这些是计算预算，而非数学秩限制；`saturated_kernel` 在约化前先通过预算检查。^[integer-lattice.md:24-27, integer-lattice.md:39-41]

## 相关格运算

[[余特征作用的负特征整数子格]] 对应的 `negative_coweight_eigenspace` 从余特征作用本身计算 \(\ker_{\mathbb Z}(I+\theta_Y)\)。`LatticeInvolution::coweight_matrix()` 已存储余特征上的对偶作用，因此该函数有意不再转置矩阵。^[integer-lattice.md:43-45]

`reduce_basis_mod_two` 将整数基模 \(2\) 归约，仅保留其在 \(Y/2Y\) 中的张成。它与饱和整数核计算同属来源所述的核与归约功能。^[integer-lattice.md:37-45]

## 证据范围

来源属于结构性源码阅读，记录了 dirty 工作区的源码字节及阅读快照；其中关于 `saturated_kernel` 幺模右因子的描述已经维护者对照源码核实。^[integer-lattice.md:9-15, integer-lattice.md:81-85]

来源包未执行构建、测试或原版运行，不提供数学验收、性能或并行结论。该基础设施的正确性属于独立的 [[HPC 验收证据链]]，包括 fundamental-lattice 与 torus gate 等，来源包未重述或扩展这些证据。^[integer-lattice.md:10-12, integer-lattice.md:80-80]

## Sources

- [integer-lattice.md](integer-lattice.md) — 精确整数格线性代数：预算、饱和核与可观测基。

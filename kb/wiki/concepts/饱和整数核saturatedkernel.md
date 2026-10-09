---
title: 饱和整数核（saturated_kernel）
summary: 通过行列混合约化跟踪幺模右因子 V，取其对应零对角元的列生成完整整数核，避免采用有理行约化后通分的方法。
sources:
  - integer-lattice.md
kind: concept
createdAt: "2026-10-09T14:51:31.058Z"
updatedAt: "2026-10-09T14:51:31.058Z"
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
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 饱和整数核（saturated_kernel）

`saturated_kernel(matrix, budget)` 用于计算饱和整数核的基，是 `integer_lattice.rs` 中基于 Malachite 大整数 `Integer` 的精确格线性代数功能。其实现先检查计算预算，再进行保持幺模右因子的矩阵约化。^[integer-lattice.md:19-20, integer-lattice.md:39-41]

## 约化方法

算法采用行列混合约化，并在过程中保持幺模右因子 \(V\)。矩阵对角化后，取 \(V\) 中对应零对角元的列，得到完整整数核的基。实现刻意不采用“先做有理行约化，再通分”的路径。^[integer-lattice.md:39-41]

## 输入与计算预算

底层 `IntegerMatrix` 是 Malachite 整数上的行主序精确矩阵，公开 `rows`、`columns` 和 `entries`。其 `from_i32_entries` 构造器先通过 `checked_shape` 校验形状，再分配存储并逐元素转换；形状不符时返回 `InvalidIntegerMatrixShape`。^[integer-lattice.md:31-35]

`IntegerLatticeBudget` 限制单次计算的每个矩阵维数（`max_rank`）、存活工作条目总量（`max_entries`）、初等操作次数（`max_steps`）以及中间系数位长（`max_coefficient_bits`）。这些约束是计算预算，而非数学上的秩限制；`saturated_kernel` 在开始约化前检查预算。^[integer-lattice.md:24-27, integer-lattice.md:39-41]

## 相关格运算

[[余特征作用的负特征整数子格]]对应的 `negative_coweight_eigenspace` 从余特征作用计算
\(\ker_{\mathbb Z}(I+\theta_Y)\)。由于 `LatticeInvolution::coweight_matrix()` 已存储余特征上的对偶作用，该函数有意不再转置矩阵。另一个相关操作 `reduce_basis_mod_two` 将整数基模 \(2\) 归约，仅保留其在 \(Y/2Y\) 中的张成。^[integer-lattice.md:42-45]

## 证据范围

本文依据的来源包属于结构性源码阅读，记录的是 dirty 工作区字节及其快照。来源包未执行构建、测试或原版运行，因此不提供数学验收、性能或并行结论；相关正确性证据属于独立的 [[HPC 验收证据链]]，来源包未重述或扩展这些结果。^[integer-lattice.md:9-15, integer-lattice.md:80-80]

## Sources

- [integer-lattice.md](integer-lattice.md) — 精确整数格线性代数：预算、饱和核与可观测基

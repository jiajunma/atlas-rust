---
title: 饱和整数核（saturated_kernel）
summary: 在预算检查后进行跟踪幺模右因子 V 的行列混合约化，以零对角元对应的 V 列构造完整整数核。
sources:
  - integer-lattice.md
kind: concept
createdAt: "2026-10-09T14:51:31.058Z"
updatedAt: "2026-10-09T20:55:21.302Z"
tags:
  - 整数格
  - 核空间
aliases:
  - 饱和整数核saturatedkernel
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: 饱和整数核（saturated_kernel）
summary: 在计算预算检查后，通过跟踪幺模右因子 V 的行列混合约化，以 V 中对应零对角元的列构成完整整数核的基。
sources:
  - integer-lattice.md
kind: concept
createdAt: "2026-10-09T14:51:31.058Z"
updatedAt: "2026-10-10"
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

`saturated_kernel(matrix, budget)` 计算饱和整数核的基，属于 `integer_lattice.rs` 中基于 Malachite 大整数 `Integer` 的精确格线性代数功能。它先检查计算预算，再进行保持幺模右因子的行列混合约化，以得到完整整数核。^[integer-lattice.md:19-20, integer-lattice.md:39-41]

## 约化方法

算法在行列混合约化过程中跟踪并保持幺模右因子 \(V\)。矩阵对角化后，\(V\) 中对应零对角元的列构成完整整数核的基。实现刻意不采用“先做有理行约化，再通分”的方法。^[integer-lattice.md:39-41]

## 矩阵输入与计算预算

底层 `IntegerMatrix` 是 Malachite 整数上的行主序精确矩阵，公开 `rows`、`columns` 和 `entries`。其 `from_i32_entries` 构造器先通过 `checked_shape` 校验形状，再分配存储并逐元素转换；形状不符时返回 `InvalidIntegerMatrixShape`。相关表示见 [[精确整数矩阵与 Bézout 幺模变换]]。^[integer-lattice.md:31-35]

[[整数格计算预算（IntegerLatticeBudget）]] 约束单次计算的每个矩阵维数（`max_rank`）、存活工作条目总量（`max_entries`）、初等操作次数（`max_steps`）和中间系数位长（`max_coefficient_bits`）。这些约束是计算预算，而非数学秩限制；`saturated_kernel` 在开始约化前先通过预算检查。^[integer-lattice.md:24-27, integer-lattice.md:39-41]

## 相关格运算

[[余特征作用的负特征整数子格]] 对应的 `negative_coweight_eigenspace` 从余特征作用本身计算 \(\ker_{\mathbb Z}(I+\theta_Y)\)。`LatticeInvolution::coweight_matrix()` 已经存储余特征上的对偶作用，因此该函数有意不再转置矩阵。^[integer-lattice.md:43-45]

[[整数基的模 2 归约]] 对应的 `reduce_basis_mod_two` 将整数基模 \(2\) 归约，只保留其在 \(Y/2Y\) 中的张成。^[integer-lattice.md:42-42]

## 证据范围

来源属于结构性源码阅读，记录了 dirty 工作区的源码字节及阅读快照；其中关于 `saturated_kernel` 幺模右因子的描述已经维护者逐条对照源码核实。该记录不应视为已提交源码的证明。^[integer-lattice.md:9-15, integer-lattice.md:81-85]

来源包未执行构建、测试或原版运行，不提供数学验收、性能或并行结论。该基础设施的正确性属于独立的 HPC 证据链，包括 fundamental-lattice 与 torus gate 等；本来源包未重述或扩展这些证据。^[integer-lattice.md:10-12, integer-lattice.md:80-80]

## Sources

- [integer-lattice.md](../../sources/integer-lattice.md) — 精确整数格线性代数：预算、饱和核与可观测基。

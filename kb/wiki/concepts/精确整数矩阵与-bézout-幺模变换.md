---
title: 精确整数矩阵与 Bézout 幺模变换
summary: IntegerMatrix 使用 Malachite 大整数与行主序存储，构造先检查形状；BezoutTransform 通过扩展 gcd 和精确商构造幺模变换系数。
sources:
  - integer-lattice.md
kind: concept
createdAt: "2026-10-09T14:51:22.619Z"
updatedAt: "2026-10-10T00:35:44.800Z"
tags:
  - 精确算术
  - 整数矩阵
aliases:
  - 精确整数矩阵与-bézout-幺模变换
  - 精B幺
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: 精确整数矩阵与 Bézout 幺模变换
summary: IntegerMatrix 使用 Malachite 大整数和行主序存储，在分配前校验形状；私有 BezoutTransform 通过扩展 gcd 与精确商携带幺模变换系数。
sources:
  - integer-lattice.md
kind: concept
tags:
  - 精确算术
  - 整数矩阵
  - 幺模变换
aliases:
  - 精确整数矩阵与-bézout-幺模变换
  - 精B幺
provenanceState: extracted
---

# 精确整数矩阵与 Bézout 幺模变换

`IntegerMatrix` 与私有辅助类型 `BezoutTransform` 属于 `integer_lattice.rs` 的精确整数格线性代数基础设施。该模块以 Malachite 大整数 `Integer` 为算术基础，构造与收集入口先执行形状或预算校验，再触碰存储。^[integer-lattice.md:19-20, integer-lattice.md:29-35]

## 矩阵表示与构造

`IntegerMatrix` 采用行主序存储，`rows`、`columns` 和 `entries` 均为公开字段。`from_i32_entries` 按“`checked_shape` 校验 → 分配存储 → 逐元素转换”的顺序构造矩阵；形状不符时报告 `InvalidIntegerMatrixShape`。^[integer-lattice.md:31-35]

`IntegerLatticeBudget` 约束单次计算的每个矩阵维数、存活工作条目总量、初等操作次数和中间系数位长，分别对应 `max_rank`、`max_entries`、`max_steps` 与 `max_coefficient_bits`。这些是**计算预算，而非数学秩限制**；四个字段均为私有，构造函数 `new` 为 `pub const fn`。^[integer-lattice.md:22-27]

## Bézout 幺模变换系数

`BezoutTransform` 是私有类型，携带与两个元素关联的幺模变换系数。其中，`s`、`t` 由 `extended_gcd` 得到，`u`、`v` 为除以最大公因数 gcd 所得的精确商。^[integer-lattice.md:33-35]

## 饱和整数核与后续归约

`saturated_kernel(matrix, budget)` 先检查预算，再进行保持幺模右因子 $V$ 的行列混合约化。矩阵对角化后，$V$ 中对应零对角元的列构成完整整数核。实现刻意不采用“有理行约化再通分”的路径。^[integer-lattice.md:39-41]

[[整数基的模 2 归约|reduce_basis_mod_two]] 将整数基模 2 归约，仅保留其在 $Y/2Y$ 中的张成。`negative_coweight_eigenspace` 则直接从余特征作用计算 $\ker_{\mathbb Z}(I+\theta_Y)$：由于 `LatticeInvolution::coweight_matrix()` 已存储余特征上的对偶作用，该函数有意不再转置，详见[[余特征作用的负特征整数子格]]。^[integer-lattice.md:42-45]

## 关系格封装与可观测基

[[关系格封装与构造预检]]通过 `RelationMatrix(IntegerMatrix)` 等类型组织关系格数据。`preflight_shape` 在分配或复制任何条目前检查形状；`from_i32_iter` 在推进迭代器或保留存储前拒绝超大形状。`RelationGenerator::try_collect` 仅在借用生成元描述符的计数及其蕴含的分子矩阵通过关系格预算后执行收集。^[integer-lattice.md:47-53]

[[承载可观测量的适配基（adapted_basis）]]伴随跟踪左变换的逆，而非事后求逆，并保留上游主元策略，包括首个最小 gcd 种子、`find_small_remainder` 的 rotate 步骤和 kept-rows-first 重排。所选基固定 `stable_log` 代表元，进而固定 `g_rho_check` 与下游各个 `torus_factor` 有理量；实现先预检工作矩阵、基、逆和对角线的条目总量，再进行消元。^[integer-lattice.md:55-64]

## 证据边界

本页依据对 `integer_lattice.rs` 的结构性阅读，所读字节记录于 `snapshots/2026-10-03-integer-lattice.json`，对应 dirty 工作区。该基础设施的正确性属于独立的 HPC 证据链，包括 fundamental-lattice 与 torus gate；本来源不重述或扩展这些证据。^[integer-lattice.md:9-15]

来源未执行构建、测试或原版运行，不提供数学验收、性能或并行结论。上游 `matreduc.h`、`matreduc.cpp` 的行号转述自源码注释，未独立重读上游，可能随版本演进漂移。^[integer-lattice.md:70-80]

## Sources

- [integer-lattice.md](../../sources/integer-lattice.md) — 精确整数格线性代数：预算、饱和核与可观测基。

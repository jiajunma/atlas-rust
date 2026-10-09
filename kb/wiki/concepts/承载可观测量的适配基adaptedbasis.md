---
title: 承载可观测量的适配基（adapted_basis）
summary: adapted_basis 同步跟踪左变换的逆并保留上游主元与重排策略，因为选定的基固定 stable_log 代表元及下游 g_rho_check、torus_factor 有理量。
sources:
  - integer-lattice.md
kind: concept
createdAt: "2026-10-09T14:51:49.689Z"
updatedAt: "2026-10-09T14:51:49.689Z"
tags:
  - 适配基
  - 基线对齐
  - 可观测量
  - 矩阵约化
aliases:
  - 承载可观测量的适配基adaptedbasis
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 承载可观测量的适配基（adapted_basis）

`adapted_basis` 是精确整数格线性代数中的适配基算法，基于 Malachite 大整数 `Integer` 实现。它忠实移植上游 `matreduc::adapted_basis`，保留具体的变换跟踪方式与主元策略，因为所选基承载下游可观测量。^[integer-lattice.md:19-20, integer-lattice.md:55-64]

## 可观测性与选基约束

适配基固定 [[stable_log：选举的稳定对数|stable_log]] 的代表元，进而固定 `g_rho_check` 与下游每个 `torus_factor` 有理量。因此，具体选基规则属于需要保留的实现行为，而不仅是生成某个基的内部步骤；其关联背景包括 [[KGB 种子代表元的可观测影响]]。^[integer-lattice.md:57-64]

## 变换跟踪与主元策略

算法在归约过程中同步跟踪左变换（LEFT）的逆，而不是完成归约后再求逆。主元策略逐字遵循上游，包括尽量少用行操作（“minimal use of row operations”）、选择首个最小 gcd 种子、`find_small_remainder` 中的 rotate 步骤，以及 kept-rows-first 重排。^[integer-lattice.md:57-64]

实现先预检工作矩阵、基、逆与对角线所需的条目总量，再逐行调用 `gcd_row_to_pivot`，并循环使用 `find_small_remainder` 消元。预算检查因此先于这些归约步骤。^[integer-lattice.md:63-64]

## 计算预算与派生接口

整数格基础设施的 `IntegerLatticeBudget` 约束每个矩阵维数 `max_rank`、存活工作条目总量 `max_entries`、初等操作次数 `max_steps` 和中间系数位长 `max_coefficient_bits`。这些约束是计算预算，不是数学秩限制；适配基的条目总量预检属于这一资源控制体系。^[integer-lattice.md:22-27, integer-lattice.md:63-64]

相关派生函数包括 `adapted_relation_basis`、`filter_relation_units`、`replace_relation_generators`、`annihilator_modulo` 和 `quotient_relation_basis`，可结合 [[关系格封装与构造预检]] 理解其接口背景。^[integer-lattice.md:47-53, integer-lattice.md:66-68]

## 证据范围

来源包记录的是对 `integer_lattice.rs` 的结构性阅读，所读字节来自 dirty 工作区快照。维护者已对照源码核实适配基的预算预检与主元策略；上游 `matreduc.cpp:262-336` 和 `matreduc.h:70-122` 的定位则转述自源码注释，未独立重读上游，行号可能随版本变化。^[integer-lattice.md:9-15, integer-lattice.md:57-64, integer-lattice.md:72-76, integer-lattice.md:81-85]

该来源包未执行构建、测试或原版运行，不提供数学验收、性能或并行结论。正确性属于独立的 [[HPC 验收证据链]]，包括 fundamental-lattice 与 torus gate；本文不扩展其验收范围。^[integer-lattice.md:10-11, integer-lattice.md:80-80]

## Sources

- [精确整数格线性代数：预算、饱和核与可观测基](integer-lattice.md)

---
title: 承载可观测量的适配基（adapted_basis）
summary: adapted_basis 同步跟踪左变换的逆并保留上游主元与重排策略，选定基影响 stable_log、g_rho_check 及下游 torus_factor；本来源仅提供结构性阅读证据。
sources:
  - integer-lattice.md
kind: concept
createdAt: "2026-10-09T14:51:49.689Z"
updatedAt: "2026-10-09T19:30:38.855Z"
tags:
  - 适配基
  - 幺模变换
  - 兼容性
aliases:
  - 承载可观测量的适配基adaptedbasis
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# 承载可观测量的适配基（adapted_basis）

`adapted_basis` 是精确整数格线性代数中的适配基算法，所在基础设施使用 Malachite 大整数 `Integer`。实现忠实移植上游 `matreduc::adapted_basis`，保留变换跟踪、主元选择与重排策略，因为选定的基会固定下游可观测量。^[integer-lattice.md:19-20, integer-lattice.md:55-64]

## 选基的可观测影响

适配基固定 [[stable_log：选举的稳定对数|stable_log]] 的代表元，进而固定 `g_rho_check` 与下游每个 `torus_factor` 有理量。这是来源将所选基称为“承载可观测量”（OBSERVABLE-BEARING）的原因，也是保留具体选基策略的动机；相关背景见 [[KGB 种子代表元的可观测影响]]。^[integer-lattice.md:57-64]

## 变换跟踪与归约策略

算法在归约过程中同步跟踪左变换（LEFT）的逆。主元策略沿用上游的具体操作约定：尽量少用行操作、选择首个最小 gcd 种子、执行 `find_small_remainder` 中的 rotate 步骤，以及采用 kept-rows-first 重排。^[integer-lattice.md:57-60]

实现先预检工作矩阵、基、逆与对角线的条目总量，再逐行调用 `gcd_row_to_pivot`，并循环使用 `find_small_remainder` 消元。左变换的逆在这些操作中伴随维护，无需在归约结束后另行求逆。^[integer-lattice.md:57-64]

## 计算预算与派生接口

适配基的条目预检属于 [[整数格计算预算（IntegerLatticeBudget）]] 体系。该预算约束单次计算的每个矩阵维数 `max_rank`、存活工作条目总量 `max_entries`、初等操作次数 `max_steps` 和中间系数位长 `max_coefficient_bits`；这些字段表示计算资源限制，不是数学秩限制。^[integer-lattice.md:24-27, integer-lattice.md:63-64]

相关派生函数包括 `adapted_relation_basis`、`filter_relation_units`、`replace_relation_generators`、`annihilator_modulo` 和 `quotient_relation_basis`。其接口背景可结合 [[关系格封装与构造预检]] 阅读；关系格构造器同样强调在分配、复制或收集条目前完成形状与预算检查。^[integer-lattice.md:49-53, integer-lattice.md:66-68]

## 证据范围

来源是对 `integer_lattice.rs` 的结构性阅读，基于 dirty 工作区快照；维护者已对照源码核实适配基的预算预检与主元策略。上游定位为 `matreduc.cpp:262-336`（含 gcd 辅助函数）和 `matreduc.h:70-122`，这些位置转述自源码注释，未独立重读上游，行号可能随版本变化。^[integer-lattice.md:9-15, integer-lattice.md:57-60, integer-lattice.md:72-76, integer-lattice.md:81-85]

来源包未执行构建、测试或原版运行，不提供数学验收、性能或并行结论。正确性属于独立的 [[HPC 验收证据链]]，包括 fundamental-lattice 与 torus gate 等；本页不扩展这些证据的范围。^[integer-lattice.md:10-11, integer-lattice.md:80-80]

## Sources

- [精确整数格线性代数：预算、饱和核与可观测基](integer-lattice.md)

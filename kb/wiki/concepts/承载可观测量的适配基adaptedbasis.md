---
title: 承载可观测量的适配基（adapted_basis）
summary: adapted_basis 同步跟踪左变换的逆并保留上游主元与重排策略，所选基影响 stable_log、g_rho_check 和 torus_factor；本包仅提供结构性阅读证据。
sources:
  - integer-lattice.md
kind: concept
createdAt: "2026-10-09T14:51:49.689Z"
updatedAt: "2026-10-09T22:32:41.549Z"
tags:
  - 整数格
  - 适配基
  - 兼容性
aliases:
  - 承载可观测量的适配基adaptedbasis
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: 承载可观测量的适配基（adapted_basis）
summary: adapted_basis 同步跟踪左变换的逆并保留上游主元与重排策略，所选基固定 stable_log 代表元及下游有理量；来源仅提供结构性阅读证据。
sources:
  - integer-lattice.md
kind: concept
tags:
  - 整数格
  - 基选择
  - 上游兼容
aliases:
  - 承载可观测量的适配基adaptedbasis
provenanceState: extracted
---

# 承载可观测量的适配基（adapted_basis）

`adapted_basis` 属于基于 Malachite 大整数 `Integer` 的精确整数格线性代数基础设施。来源将其描述为上游 `matreduc::adapted_basis` 的忠实移植：保留变换跟踪、主元选择与重排策略，因为具体选定的基承载下游可观测量。^[integer-lattice.md:19-20, integer-lattice.md:55-64]

## 选基的可观测影响

所选基固定 [[stable_log：选举的稳定对数|stable_log]] 的代表元，进而固定 `g_rho_check` 与下游每个 `torus_factor` 有理量。来源将这一性质称为 **OBSERVABLE-BEARING**（承载可观测量），并明确列为保留上游选基策略的动机；相关背景见 [[KGB 种子代表元的可观测影响]]。^[integer-lattice.md:60-64]

## 变换跟踪与归约策略

算法在归约过程中同步跟踪左变换（LEFT）的逆，而非归约完成后另行求逆。主元策略保留上游的具体约定：尽量少用行操作（minimal use of row operations）、选择首个最小 gcd 种子、执行 `find_small_remainder` 的 rotate 步骤，以及采用 kept-rows-first 重排。^[integer-lattice.md:57-60]

实现首先预检工作矩阵、基、逆与对角线所需的条目总量，然后逐行调用 `gcd_row_to_pivot`，并循环使用 `find_small_remainder` 消元。^[integer-lattice.md:63-64]

## 计算预算与派生接口

`IntegerLatticeBudget` 约束单次计算的每个矩阵维数 `max_rank`、存活工作条目总量 `max_entries`、初等操作次数 `max_steps` 和中间系数位长 `max_coefficient_bits`。这些是计算预算，不是数学秩限制；`adapted_basis` 的条目总量预检覆盖其工作矩阵及辅助存储。^[integer-lattice.md:24-27, integer-lattice.md:63-64]

派生函数族包括 `adapted_relation_basis`、`filter_relation_units`、`replace_relation_generators`、`annihilator_modulo` 和 `quotient_relation_basis`。相关接口见 [[关系格封装与构造预检]]：关系矩阵构造器在分配或复制条目前检查形状，生成元收集入口则在收集前检查描述符计数及其蕴含的分子矩阵预算。^[integer-lattice.md:49-53, integer-lattice.md:66-68]

## 证据范围与限制

来源是对 `integer_lattice.rs` 的结构性阅读，所读字节来自 dirty 工作区，记录于 `snapshots/2026-10-03-integer-lattice.json`。维护者对照源码逐条核对并改写草稿，其中适配基的预算预检与主元策略已按源码落实。^[integer-lattice.md:9-15, integer-lattice.md:81-85]

上游定位为 `matreduc.cpp:262-336`（含 gcd 辅助函数）和 `matreduc.h:70-122`。这些位置转述自源码注释，来源未独立重读上游文件，行号可能随版本演进而漂移。^[integer-lattice.md:57-60, integer-lattice.md:75-76]

来源包未执行构建、测试或原版运行，不提供数学验收、性能或并行结论。该基础设施的正确性属于独立的 HPC 证据链，包括 fundamental-lattice 与 torus gate 等；本页不重述或扩展这些证据。^[integer-lattice.md:10-11, integer-lattice.md:80-80]

## Sources

- [精确整数格线性代数：预算、饱和核与可观测基](../../sources/integer-lattice.md)

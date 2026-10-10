---
title: 对偶分量群的 fiber rank
summary: fiber_rank 对 q=−θᵀ 计算模二核维数减去正特征整数格模二像维数，使用饱和减法且不检查对合前提；来源文件没有相关测试，本次亦未执行验证。
sources:
  - cayley-cross.md
kind: concept
createdAt: "2026-10-09T14:44:11.199Z"
updatedAt: "2026-10-10T02:22:35.495Z"
tags:
  - 对偶分量群
  - 整数格
  - 证据边界
aliases:
  - 对偶分量群的-fiber-rank
  - 对FR
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# 对偶分量群的 fiber rank

`fiber_rank(weight_matrix, budget)` 位于 `involution_classification.rs`，计算对偶分量群 `dualPi0(-θᵀ)` 的 \(\mathbf F_2\) 维数，即 `Cartan_info` 打印的 fiber size 指数。源码注释引用 `tori.cpp:162-173` 与 `subquotient.h:79`；来源材料仅转录这些上游引用。^[cayley-cross.md:10-13, cayley-cross.md:80-86]

## 计算公式

令 \(\theta\) 为输入的权格作用矩阵，\(q=-\theta^T\)。`fiber_rank` 计算两项维数之差：\(\dim_{\mathbf F_2}\ker((q+I)\bmod 2)-\dim_{\mathbf F_2}\operatorname{span}(\operatorname{plusBasis}(q)\bmod 2)\)。其中，`plusBasis(q)` 对应 \(q-I\) 的整数饱和核。^[cayley-cross.md:80-84]

第一项借助模二子空间秩求得；第二项先求 \(q-I\) 的饱和核，再通过 `reduce_basis_mod_two` 将核基约化到模二空间，计算其张成空间的维数。这里同时使用整数格上的核与模二空间上的秩。^[cayley-cross.md:82-84]

## 算术行为与调用前提

实现末步使用 `saturating_sub`，两项相减时采用饱和减法，不因该减法报错。同文件的 `classify_plus_identity` 则使用 `checked_sub`；来源将这一差异记录为阅读观察。后者的分类算法另见 [[利用饱和核与模二秩计算整对合分类]]。^[cayley-cross.md:74-85]

`fiber_rank` 自身不检查对合前提。相比之下，`classify_involution` 先检查方阵形状，再执行资源预算门，随后使用全程 checked 的 `i128` 算术逐元验证 \(M^2=I\)。这些检查属于另一个函数，不能视为 `fiber_rank` 已提供的保证。^[cayley-cross.md:67-72, cayley-cross.md:85-86]

## 测试与证据边界

所读文件中没有针对 `fiber_rank` 的测试。文件中的六个测试锚点覆盖整对合分类的恒等、交换型、[[整对合分类中的奇偶区分|奇偶区分]]、非对合拒绝、矩阵形状错误及预算门；`classify_plus_identity` 也仅经 `classify_involution` 间接覆盖。这些测试不构成 `fiber_rank` 的直接测试证据。^[cayley-cross.md:88-91, cayley-cross.md:100-101]

本页依据结构性源码阅读，不构成数学或正确性验收。来源经过维护者对照源码逐条核对改写，但该次知识维护未执行 Atlas、Cargo、测试或 benchmark，因此测试锚点描述不代表本次执行结果。^[cayley-cross.md:9-13, cayley-cross.md:95-96, cayley-cross.md:107-111]

## Sources

- [cayley-cross.md](../../sources/cayley-cross.md) — Cayley/Cross 分解与整对合分类（cayley_cross.rs / involution_classification.rs）。

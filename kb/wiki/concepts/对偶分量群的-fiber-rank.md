---
title: 对偶分量群的 fiber rank
summary: fiber_rank 对 q=−θᵀ 计算模二核维数与正特征整数格模二像维数之差，末步使用 saturating_sub；函数不检查对合前提且本文件无相关测试。
sources:
  - cayley-cross.md
kind: concept
createdAt: "2026-10-09T14:44:11.199Z"
updatedAt: "2026-10-09T20:49:43.180Z"
tags:
  - 分量群
  - 模二线性代数
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
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: 对偶分量群的 fiber rank
summary: fiber_rank 通过模二核与整数饱和核的模二像计算 dualPi0(-θᵀ) 的维数；末步使用饱和减法，不检查对合前提，且来源文件中无直接测试。
sources:
  - cayley-cross.md
kind: concept
tags:
  - 分量群
  - 有限域
  - 测试覆盖
---

# 对偶分量群的 fiber rank

`fiber_rank(weight_matrix, budget)` 位于 `involution_classification.rs`，计算对偶分量群 `dualPi0(-θ^T)` 的 \(\mathbf F_2\) 维数，即 `Cartan_info` 打印的 fiber size 指数。源码注释引用 `tori.cpp:162-173` 与 `subquotient.h:79`；这些上游引用由来源材料转录，未作为独立核验结果。^[cayley-cross.md:10-13, cayley-cross.md:80-86]

## 计算公式

令 \(q=-\theta^T\)，函数采用以下维数差公式。^[cayley-cross.md:80-84]

\[
\operatorname{fiber\_rank}
=
\dim_{\mathbf F_2}\ker\bigl((q+I)\bmod 2\bigr)
-
\dim_{\mathbf F_2}
\operatorname{span}\bigl(\operatorname{plusBasis}(q)\bmod 2\bigr).
\]

第一项通过模二子空间秩计算；第二项先求 \(q-I\) 的整数饱和核，再用 `reduce_basis_mod_two` 将核基约化到模二空间，计算其张成空间的维数。此处结合了整数格上的核计算与有限域上的秩计算，可参见 [[整数基的模 2 归约]] 与 [[环面对合的 dualPi0 子商构造]]。^[cayley-cross.md:80-84]

## 算术与调用前提

实现末步使用 `saturating_sub`：若第一项小于第二项，差值饱和为零，不因该减法报错。同文件的 `classify_plus_identity` 则使用 `checked_sub`；来源将这一差异记录为代码阅读观察。^[cayley-cross.md:83-86]

`fiber_rank` 自身不检查对合前提。相比之下，`classify_involution` 依次检查方阵形状、执行资源预算门，再以全程 checked 的 `i128` 算术逐项验证 \(M^2=I\)。不能将后者的检查流程视为 `fiber_rank` 已提供的保证；相关顺序见 [[对合分类的资源预算与验证顺序]]。^[cayley-cross.md:67-72, cayley-cross.md:85-86]

## 测试与证据边界

来源明确指出，`fiber_rank` 在该文件中没有任何测试；`classify_plus_identity` 也仅经 `classify_involution` 间接覆盖。因此，同文件的整对合分类测试不构成 `fiber_rank` 的直接测试覆盖。^[cayley-cross.md:88-91, cayley-cross.md:100-101]

本页依据结构性源码阅读，不构成数学或正确性验收。来源记录了维护者对照源码核对改写的过程，但本次知识维护未执行 Atlas、Cargo、测试或 benchmark。^[cayley-cross.md:9-13, cayley-cross.md:95-96, cayley-cross.md:107-111]

## Sources

- [cayley-cross.md](../../sources/cayley-cross.md)：Cayley/Cross 分解与整对合分类（`cayley_cross.rs` / `involution_classification.rs`）。

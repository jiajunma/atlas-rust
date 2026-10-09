---
title: 对偶分量群的 fiber rank
summary: fiber_rank 对 q=−θᵀ 计算模二核维数与正特征整数格模二像维数之差，使用 saturating_sub，未检查对合前提且本文件没有相关测试。
sources:
  - cayley-cross.md
  - twisted-involution-trio.md
kind: concept
createdAt: "2026-10-09T14:44:11.199Z"
updatedAt: "2026-10-09T22:26:51.077Z"
tags:
  - 分量群
  - 模二线性代数
  - 证据边界
aliases:
  - 对偶分量群的-fiber-rank
  - 对FR
confidence: 1
provenanceState: merged
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: 对偶分量群的 fiber rank
summary: fiber_rank 通过模二核与正特征整数格的模二像计算 dualPi0(-θᵀ) 的维数；末步使用饱和减法，不检查对合前提，且缺少直接测试。
sources:
  - cayley-cross.md
  - twisted-involution-trio.md
kind: concept
tags:
  - 分量群
  - 模二线性代数
  - 证据边界
aliases:
  - 对偶分量群的-fiber-rank
provenanceState: extracted
---

# 对偶分量群的 fiber rank

`fiber_rank(weight_matrix, budget)` 位于 `involution_classification.rs`，计算对偶分量群 `dualPi0(-θᵀ)` 的 \(\mathbf F_2\) 维数，即 `Cartan_info` 打印的 fiber size 指数。源码注释引用 `tori.cpp:162-173` 与 `subquotient.h:79`；来源材料仅转录这些引用，未将其作为独立核验结果。^[cayley-cross.md:10-13, cayley-cross.md:80-86]

## 计算公式

令 \(q=-\theta^T\)，其中 \(\theta\) 为输入的权格作用矩阵。函数所计算的数学量是模二核维数与正特征整数格模二像维数之差。^[cayley-cross.md:80-84]

\[
\operatorname{fiber\_rank}
=
\dim_{\mathbf F_2}\ker\bigl((q+I)\bmod 2\bigr)
-
\dim_{\mathbf F_2}
\operatorname{span}\bigl(\operatorname{plusBasis}(q)\bmod 2\bigr).
\]

第一项通过模二像秩求得，代码中的计算为 `kernel_dim = rank − image.rank()`。第二项先求 \(q-I\) 的整数饱和核，得到 `plusBasis(q)`，再用 `reduce_basis_mod_two` 将核基约化到模二空间，求其张成空间的维数；相关过程见 [[整数基的模 2 归约]]。^[cayley-cross.md:82-84, twisted-involution-trio.md:54-58]

## 算术行为与调用前提

末步使用 `saturating_sub`：若第一项小于第二项，结果钳为零，不因该减法报错。同文件的 `classify_plus_identity` 则使用 `checked_sub`，下溢会产生 `IntegerLatticeInvariantViolation`。来源将这一防御策略差异记录为阅读观察，未确定它属于有意分层还是实现漂移。^[twisted-involution-trio.md:49-58, twisted-involution-trio.md:79-84]

饱和减法只适用于最后一步。`kernel_dim` 使用普通减法，构造相关矩阵时的取负与加减 1 使用普通 `i32` 算术；来源指出，极端输入可能在 debug 模式下触发溢出 panic，这些操作没有溢出防护。^[twisted-involution-trio.md:54-58]

`fiber_rank` 自身不检查对合前提。相比之下，`classify_involution` 先检查方阵形状，再执行资源预算门，随后以 checked `i128` 算术逐项验证 \(M^2=I\)。后者的检查流程不能视为 `fiber_rank` 已提供的保证，详见 [[整对合分类的预算门与检查顺序]]。^[cayley-cross.md:67-72, cayley-cross.md:85-86]

## 测试与证据边界

来源明确指出，`fiber_rank` 在所读文件中没有任何测试。该文件列出的六个测试针对整对合分类；`classify_plus_identity` 也仅经 `classify_involution` 间接覆盖，因此这些测试不构成 `fiber_rank` 的直接覆盖。^[cayley-cross.md:88-91, cayley-cross.md:100-101, twisted-involution-trio.md:60-64]

本页依据结构性源码阅读，不构成数学或正确性验收。两份来源均记录了维护者对照源码核对改写的过程，且此次知识维护未执行 Atlas、Cargo、测试或 benchmark。^[cayley-cross.md:9-13, cayley-cross.md:107-111, twisted-involution-trio.md:9-13, twisted-involution-trio.md:88-92]

## Sources

- [cayley-cross.md](../../sources/cayley-cross.md)：Cayley/Cross 分解与整对合分类。
- [twisted-involution-trio.md](../../sources/twisted-involution-trio.md)：扭对合、对合分类与环境根反射字。

---
title: 对偶分量群的 fiber rank
summary: fiber_rank 对 q=−θᵀ 计算模二核维数与正特征整数格模二像维数之差，使用 saturating_sub，不检查对合前提且本文件无相关测试。
sources:
  - cayley-cross.md
  - twisted-involution-trio.md
kind: concept
createdAt: "2026-10-09T14:44:11.199Z"
updatedAt: "2026-10-10T00:29:48.852Z"
tags:
  - 对偶分量群
  - 整数格
  - 模二线性代数
aliases:
  - 对偶分量群的-fiber-rank
  - 对FR
confidence: 1
provenanceState: merged
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: 对偶分量群的 fiber rank
summary: fiber_rank 计算 dualPi0(-θᵀ) 的模二维数；末步使用饱和减法，不检查对合前提，且所读文件没有相关测试。
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
---

# 对偶分量群的 fiber rank

`fiber_rank(weight_matrix, budget)` 位于 `involution_classification.rs`，计算对偶分量群 `dualPi0(-θᵀ)` 的 \(\mathbf F_2\) 维数，即 `Cartan_info` 打印的 fiber size 指数。源码注释引用 `tori.cpp:162-173` 与 `subquotient.h:79`；来源材料仅转录这些引用，并未独立核验上游实现。^[cayley-cross.md:10-13, cayley-cross.md:80-86]

## 计算公式与实现

令 \(\theta\) 为输入的权格作用矩阵，\(q=-\theta^T\)。函数对应的维数公式为 \(\dim_{\mathbf F_2}\ker((q+I)\bmod 2)-\dim_{\mathbf F_2}\operatorname{span}(\operatorname{plusBasis}(q)\bmod 2)\)，其中 `plusBasis(q)` 由 \(q-I\) 的整数饱和核得到。^[cayley-cross.md:80-84]

第一项通过模二像秩计算，代码使用 `kernel_dim = rank − image.rank()`。第二项先求 \(q-I\) 的饱和核，再通过 `reduce_basis_mod_two` 将整数核基约化到模二空间，计算其张成空间的维数；相关操作见 [[整数基的模 2 归约]]。^[cayley-cross.md:82-84, twisted-involution-trio.md:54-58]

## 算术行为与调用前提

实现最后使用 `saturating_sub` 求两项之差：若前项小于后项，结果钳为零，不因该减法报错。同文件的 `classify_plus_identity` 则使用 `checked_sub`，下溢会返回 `IntegerLatticeInvariantViolation`。来源将这一差异记为阅读观察，未确定它属于有意分层还是实现漂移。^[twisted-involution-trio.md:49-58, twisted-involution-trio.md:79-84]

末步的饱和减法不代表整个函数都具有溢出防护。`kernel_dim = rank − image.rank()` 使用普通减法，相关取负与加减 1 使用普通 `i32` 算术；来源指出，极端输入可能在 debug 模式下触发溢出 panic。^[twisted-involution-trio.md:54-58]

`fiber_rank` 自身不检查对合前提。相比之下，`classify_involution` 依次检查方阵形状、执行资源预算门，再用 checked `i128` 算术验证 \(M^2=I\)。这些检查属于另一个函数，不能视为 `fiber_rank` 已提供的保证；详见 [[整对合分类的预算门与检查顺序]]。^[cayley-cross.md:67-72, cayley-cross.md:85-86]

## 测试与证据边界

所读 `involution_classification.rs` 中没有任何针对 `fiber_rank` 的测试。文件中的六个测试针对整对合分类，覆盖恒等、交换型、奇偶区分、非对合拒绝、矩阵形状错误及预算门；`classify_plus_identity` 也仅经 `classify_involution` 间接覆盖。这些测试不构成 `fiber_rank` 的直接覆盖。^[cayley-cross.md:88-91, cayley-cross.md:100-101, twisted-involution-trio.md:60-64]

本页依据结构性源码阅读，不构成数学或正确性验收。两份来源均记录了维护者对照源码逐条核对改写的过程；该次知识维护未执行 Atlas、Cargo、测试或 benchmark，因此测试描述也不代表本次执行结果。^[cayley-cross.md:9-13, cayley-cross.md:107-111, twisted-involution-trio.md:9-13, twisted-involution-trio.md:88-92]

## Sources

- [cayley-cross.md](../../sources/cayley-cross.md)：Cayley/Cross 分解与整对合分类。
- [twisted-involution-trio.md](../../sources/twisted-involution-trio.md)：扭对合、对合分类与环境根反射字。

---
title: 对偶分量群的 fiber rank
summary: fiber_rank 对 q = −θᵀ 计算 dim ker((q+I) mod 2) − dim span(plusBasis(q)) mod 2，末步使用 saturating_sub；函数未检查对合前提，且本文件没有相关测试。
sources:
  - cayley-cross.md
kind: concept
createdAt: "2026-10-09T14:44:11.199Z"
updatedAt: "2026-10-09T14:44:11.199Z"
tags:
  - 分量群
  - 有限域
  - 测试覆盖
aliases:
  - 对偶分量群的-fiber-rank
  - 对FR
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 对偶分量群的 fiber rank

`fiber_rank(weight_matrix, budget)` 计算对偶分量群 `dualPi0(-θ^T)` 的 F₂ 维数，即 `Cartan_info` 打印的 fiber size 指数。该函数位于 `involution_classification.rs`；源码注释引用 `tori.cpp:162-173` 与 `subquotient.h:79`。^[cayley-cross.md:80-86]

## 计算公式与实现

令 \(q=-\theta^T\)，则函数采用公式
\[
\operatorname{fiber\_rank}
=
\dim_{\mathbf F_2}\ker\bigl((q+I)\bmod 2\bigr)
-
\dim_{\mathbf F_2}\operatorname{span}\bigl(\operatorname{plusBasis}(q)\bmod 2\bigr).
\]
其中，第一项通过模二子空间秩计算；第二项先求 \(q-I\) 的饱和核，再用 `reduce_basis_mod_two` 将所得基约化到模二空间并计算其张成空间的维数。这里同时涉及整数格上的饱和核与 F₂ 上的秩计算，可关联阅读 [[利用饱和核与模二秩计算整对合分类]]。^[cayley-cross.md:80-84]

实现最终使用 `saturating_sub` 相减：若被减数小于减数，结果饱和为零，不因该差值报错。这与同文件中 `classify_plus_identity` 使用 `checked_sub` 的处理方式不同；来源将这一差异记录为代码阅读观察。^[cayley-cross.md:83-86]

## 调用前提与证据边界

`fiber_rank` 自身不检查对合前提。因此，不能把它与 `classify_involution` 的验证流程混同：后者依次执行方阵形状检查、资源预算门以及使用 checked i128 算术的 \(M^2=I\) 检查。相关验证顺序见 [[对合分类的资源预算与验证顺序]]。^[cayley-cross.md:67-72, cayley-cross.md:85-86]

来源明确指出，`fiber_rank` 在该文件中没有任何测试；同文件的整对合分类测试不能作为该函数的直接测试覆盖。本来源属于结构性阅读，不构成数学或正确性验收，本次知识维护也未执行 Atlas、Cargo、测试或 benchmark。^[cayley-cross.md:95-101, cayley-cross.md:107-111]

## Sources

- [cayley-cross.md](cayley-cross.md)：Cayley/Cross 分解与整对合分类（`cayley_cross.rs` / `involution_classification.rs`）。

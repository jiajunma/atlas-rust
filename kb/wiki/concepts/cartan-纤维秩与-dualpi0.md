---
title: Cartan 纤维秩与 dualPi0
summary: fiber_rank 计算 q=−θᵀ 对应 dualPi0(q) 的 F₂ 维数，即 dim ker((q+I) mod 2)−dim span(plusBasis(q)) mod 2，作为 Cartan 纤维大小指数；该函数缺少测试且混用普通算术与饱和减法。
sources:
  - twisted-involution-trio.md
kind: concept
createdAt: "2026-10-09T15:14:45.406Z"
updatedAt: "2026-10-09T15:14:45.406Z"
tags:
  - Cartan理论
  - 拓扑
  - 纤维秩
aliases:
  - cartan-纤维秩与-dualpi0
  - C纤D
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# Cartan 纤维秩与 dualPi0

Cartan 纤维秩由 `involution_classification.rs` 中的 `fiber_rank` 计算，表示 `dualPi0(−θᵀ)` 的 F₂ 维数，也是 `Cartan_info` 打印的纤维大小指数。它属于 Cartan 分类与拓扑所使用的基础计算。^[twisted-involution-trio.md:54-58, twisted-involution-trio.md:79-81]

## 计算公式

令 \(q=-\theta^{\mathsf T}\)，则纤维秩为

\[
\operatorname{fiber\_rank}(\theta)
=
\dim_{\mathbf F_2}\ker\bigl((q+I)\bmod 2\bigr)
-
\dim_{\mathbf F_2}\operatorname{span}
\bigl(\operatorname{plusBasis}(q)\bmod 2\bigr).
\]

计算从模二核的维数中减去 `plusBasis(q)` 模二后所张成空间的维数；实现中核维数通过 `rank − image.rank()` 得到。^[twisted-involution-trio.md:54-58]

## 与整数对合分类的关系

同一文件中的 `InvolutionClassification` 保存整数对合分解为恒等、交换对和取负因子时唯一确定的三个秩，但不选择或存储分解本身。其分类计算使用 \(\theta+I\) 的饱和核与模二像秩，相关方法见 [[利用饱和核与模二秩计算整对合分类]]。`fiber_rank` 则针对 \(q=-\theta^{\mathsf T}\) 计算上述模二维数差。^[twisted-involution-trio.md:41-58]

## 算术与错误处理边界

`fiber_rank` 的最终维数差使用 `saturating_sub`：若减法本应得到负值，结果会被钳为零，而不会报告错误。相比之下，分类计算中的三处 `checked_sub` 下溢会返回 `IntegerLatticeInvariantViolation`。两条路径的防御策略并不一致，不能将分类路径的错误保证直接套用于纤维秩计算。^[twisted-involution-trio.md:49-58]

核维数计算中的 `rank − image.rank()` 使用普通减法；取负及加减 1 使用普通 `i32` 算术，没有溢出防护，极端输入在 debug 模式下可能因溢出而 panic。理解相关接口时，应同时关注 [[对合分类的资源预算与验证顺序]] 与这些具体算术限制。^[twisted-involution-trio.md:54-58]

## 证据与覆盖范围

来源记录了整数对合分类的六个测试，但 `fiber_rank` 整体没有测试覆盖。本材料属于维护者对照源码核对后的结构性阅读，不构成数学验收；本次知识维护也未执行 Atlas、Cargo、测试或 benchmark。因此，上述公式与实现边界是源码阅读结论，不能据此声称纤维秩已经得到运行验证。^[twisted-involution-trio.md:9-13, twisted-involution-trio.md:60-64, twisted-involution-trio.md:88-92]

## Sources

- [twisted-involution-trio.md](twisted-involution-trio.md)

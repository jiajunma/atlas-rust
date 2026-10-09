---
title: Cartan 纤维秩与 dualPi0
summary: fiber_rank 计算 q=−θᵀ 对应 dualPi0(q) 的 F₂ 维数，作为纤维大小指数，但混用普通算术与饱和减法且缺少测试。
sources:
  - twisted-involution-trio.md
kind: concept
createdAt: "2026-10-09T15:14:45.406Z"
updatedAt: "2026-10-09T21:12:24.934Z"
tags:
  - Cartan纤维
  - 有限域线性代数
  - 算术边界
aliases:
  - cartan-纤维秩与-dualpi0
  - C纤D
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: Cartan 纤维秩与 dualPi0
summary: fiber_rank 计算 dualPi0(−θᵀ) 的 F₂ 维数，作为 Cartan 纤维大小指数；实现混用普通算术与饱和减法，且缺少直接测试。
sources:
  - twisted-involution-trio.md
kind: concept
tags:
  - Cartan理论
  - 拓扑
  - 纤维秩
aliases:
  - cartan-纤维秩与-dualpi0
---

# Cartan 纤维秩与 dualPi0

Cartan 纤维秩由 `involution_classification.rs` 中的 `fiber_rank` 计算，表示 \(\operatorname{dualPi0}(-\theta^{\mathsf T})\) 的 \(\mathbf F_2\) 维数，也是 `Cartan_info` 打印的纤维大小指数。这一函数服务于 Cartan 分类与拓扑计算。^[twisted-involution-trio.md:54-58, twisted-involution-trio.md:79-81]

## 计算公式

令 \(q=-\theta^{\mathsf T}\)。`fiber_rank` 从 \((q+I)\bmod 2\) 的核维数中，减去 `plusBasis(q)` 模二后张成空间的维数，公式如下。^[twisted-involution-trio.md:54-56]

\[
\operatorname{fiber\_rank}(\theta)
=
\dim_{\mathbf F_2}\ker\bigl((q+I)\bmod 2\bigr)
-
\dim_{\mathbf F_2}\operatorname{span}
\bigl(\operatorname{plusBasis}(q)\bmod 2\bigr),
\qquad q=-\theta^{\mathsf T}.
\]

实现中的核维数通过 `kernel_dim = rank − image.rank()` 得到；最终维数差使用 `saturating_sub`，因此实现对异常维数关系的处理需要与数学公式分开理解。^[twisted-involution-trio.md:54-58]

## 与整数对合分类的关系

同一文件中的 `InvolutionClassification` 保存整数对合分解为恒等、交换对和取负因子时唯一确定的三个秩，分别对应 compact、complex、split；它刻意不选择或存储分解本身。相关概念见 [[整数对合的 compact/complex/split 分类]]。^[twisted-involution-trio.md:39-42]

分类计算使用 \(\theta+I\) 的饱和核求得 `plus_rank`，并以 \((\theta+I)\bmod 2\) 的像秩确定 `complex`，再计算 `compact` 与 `split`。`fiber_rank` 则针对 \(q=-\theta^{\mathsf T}\) 求上述模二维数差；两者虽位于同一文件，算术防御策略并不相同。^[twisted-involution-trio.md:49-58]

## 算术与错误处理边界

`fiber_rank` 的最终减法使用 `saturating_sub`：若被减数小于减数，结果会被钳为零，而不会报告错误。相比之下，分类路径的三处 `checked_sub` 下溢会返回 `IntegerLatticeInvariantViolation`。^[twisted-involution-trio.md:49-58]

核维数计算中的 `rank − image.rank()` 使用普通减法；取负及加减 1 使用普通 `i32` 算术，没有溢出防护，极端输入在 debug 模式下可能因溢出而 panic。分类入口另有形状检查、预算闸门与对合检验，详见 [[整对合分类的预算门与检查顺序]]；这些检查不改变材料明确指出的 `fiber_rank` 算术限制。^[twisted-involution-trio.md:44-58]

## 测试与证据范围

来源记录了整数对合分类的六个测试，覆盖恒等、交换对、奇偶差异、非对合、形状错误和秩预算，但明确指出 `fiber_rank` 整体没有测试覆盖。分类测试不能作为该函数已获直接验证的证据。^[twisted-involution-trio.md:60-64]

本页依据维护者对照源码核对后的结构性阅读，不构成数学验收。来源快照记录绑定 Git base、三个文件的字节 SHA-256 与草案生成记录；本次知识维护未执行 Atlas、Cargo、测试或 benchmark。相关限制可结合 [[结构性源码阅读的验证与覆盖限制]] 理解。^[twisted-involution-trio.md:9-13, twisted-involution-trio.md:88-92]

## Sources

- [twisted-involution-trio.md](../../sources/twisted-involution-trio.md) — 扭对合、对合分类与环境根反射字。

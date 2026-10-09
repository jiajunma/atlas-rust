---
title: Weyl 字的前向交叉作用顺序
summary: crossed_word 按切片顺序折叠单生成元交叉作用，非交换情形下顺序影响结果，非法生成元仅在执行到对应位置时报错。
sources:
  - error-global-tits.md
kind: concept
createdAt: "2026-10-09T14:46:21.407Z"
updatedAt: "2026-10-09T20:51:30.289Z"
tags:
  - Weyl群
  - 执行顺序
aliases:
  - weyl-字的前向交叉作用顺序
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: Weyl 字的前向交叉作用顺序
summary: crossed_word 按切片顺序折叠单生成元交叉作用，顺序可能影响结果，非法生成元在执行到对应位置时才触发错误。
sources:
  - error-global-tits.md
kind: concept
tags:
  - Weyl群
  - 作用顺序
  - 算法语义
aliases:
  - weyl-字的前向交叉作用顺序
---

# Weyl 字的前向交叉作用顺序

全局 Tits 传输中的 `crossed_word` 按 Weyl 字的切片顺序执行交叉作用：从第一个生成元开始，依次处理至字尾。这一约定匹配上游 `cross_act(GlobalTitsElement&, const WeylWord&)`；执行顺序可能影响结果，A2 测试明确区分了前向与逆向执行。^[error-global-tits.md:92-97, error-global-tits.md:131-132]

## 顺序与单步作用

设字为 `[i₀, i₁, …, iₖ₋₁]`，初始元素为 `x₀`。执行过程为 `xⱼ₊₁ = xⱼ.crossed_generator(iⱼ)`，最终得到 `xₖ`。“前向”指切片遍历顺序，每一步使用前一步的结果；单生成元方法采用 `&self -> Result<Self>`，返回新值而不修改原值。^[error-global-tits.md:90-97]

作用对象是[[全局 Tits 元素的精确有理环面表示]]：`GlobalTitsElement` 保存完整有理余特征，包括中心坐标，以及一个 twisted involution。环面坐标采用 `[0,2)` 中的典范代表元，向纤维 mod-two 商的规约发生在之后。^[error-global-tits.md:60-65]

单生成元交叉作用记为 `s * (t,w) * δ(s)`。对于单根 \(\alpha_i\)，复根分支执行 \(t\leftarrow t-\langle\alpha_i,t\rangle\alpha_i^\vee\)；虚根分支要求配对为整数，再执行 \(t\leftarrow t+(1-\langle\alpha_i,t\rangle)\alpha_i^\vee\)；实根分支保持环面坐标不变。具体分支见[[按根类型划分的单生成元交叉作用]]。^[error-global-tits.md:75-85]

随后，环面坐标逐项模 2 规范化，Weyl 分量更新为 \(s_i\circ w\circ s_{\delta(i)}\)，并通过 `TwistedInvolution::new` 重建。因此，字的顺序规定了完整单步变换的先后关系。^[error-global-tits.md:87-97]

## 校验与错误传播

`crossed_word` 在遍历前执行一次 `validate_context`，每次 `crossed_generator` 又重新执行该校验。Weyl 作用或根对合的 datum 与 `inner_class.datum()` 不同，报告 `DatumMismatch`；否则检查 \(w\cdot\delta\) 的权与余权矩阵是否均与存储对合一致，不一致时报 `DistinguishedInvolutionMismatch`。参见[[全局 Tits 传输的上下文一致性校验]]。^[error-global-tits.md:77-79, error-global-tits.md:94-97, error-global-tits.md:101-104]

字中的生成元不在入口统一预检。只有折叠执行到 `generator >= semisimple_rank` 的位置时，才报告 `IndexOutOfRange`；单步执行还可能因无法确定根类型而报告 `InvalidRootAutomorphism`，或因虚根配对不整而报告 `InvalidStrongTorusFactor`。^[error-global-tits.md:77-85, error-global-tits.md:94-97]

## 测试与证据边界

`a2_word_execution_is_forward_and_noncommuting` 以 `assert_ne!(forward, reverse)` 锚定前向与逆向结果不同的具体案例；`rank_zero_and_an_empty_word_are_identity_transport` 覆盖秩零与空字的恒等传输情形。前者提供顺序敏感的测试锚点，不代表所有字反转后都会得到不同结果。^[error-global-tits.md:120-123, error-global-tits.md:131-132]

源材料明确将非空字的错误传播列为未覆盖路径。本页依据结构性源码阅读，不声称全局 Tits 传输已获数学验收；该次知识维护未执行 Atlas、Cargo、测试或 benchmark。完整覆盖限制见[[全局 Tits 传输的测试锚点与证据边界]]。^[error-global-tits.md:10-14, error-global-tits.md:140-142, error-global-tits.md:158-162]

## Sources

- [error-global-tits.md](../../sources/error-global-tits.md) — StructureError 错误分类学与全局 Tits 交叉作用传输层（error.rs / global_tits.rs）

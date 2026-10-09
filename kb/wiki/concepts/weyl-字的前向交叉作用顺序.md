---
title: Weyl 字的前向交叉作用顺序
summary: crossed_word 按切片顺序折叠单生成元交叉作用，顺序可能影响结果，非法生成元在执行到对应位置时才触发错误。
sources:
  - error-global-tits.md
kind: concept
createdAt: "2026-10-09T14:46:21.407Z"
updatedAt: "2026-10-09T14:46:21.407Z"
tags:
  - Weyl群
  - 作用顺序
  - 算法语义
aliases:
  - weyl-字的前向交叉作用顺序
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# Weyl 字的前向交叉作用顺序

在全局 Tits 传输层中，`crossed_word` 按 Weyl 字的切片顺序依次执行交叉作用：先处理第一个生成元，再处理第二个，直至字尾。这一约定匹配上游 `cross_act(GlobalTitsElement&, const WeylWord&)`；交换执行顺序可能改变结果，已有 A2 测试明确区分前向与逆向执行。^[error-global-tits.md:92-97, error-global-tits.md:131-132]

## 顺序与单步作用

设字为 `[i₀, i₁, …, iₖ₋₁]`，初始元素为 `x₀`。其执行过程可写为 `xⱼ₊₁ = xⱼ.crossed_generator(iⱼ)`，最终返回 `xₖ`。这里的“前向”指切片遍历顺序；每一步都作用于前一步所得的元素。单生成元方法采用 `&self -> Result<Self>`，返回新值而不修改原值。^[error-global-tits.md:90-97]

每一步交叉作用记为 `s * (t,w) * δ(s)`，其中环面分量按根类型更新：复根使用 `t ← t − ⟨α,t⟩α∨`；虚根要求配对为整数，再使用 `t ← t + (1 − ⟨α,t⟩)α∨`；实根保持环面坐标不变。随后所有坐标逐项模 2 规范化，Weyl 分量更新为 `sᵢ ∘ w ∘ s_{δ(i)}`，并通过 `TwistedInvolution::new` 重建。因此，字的执行顺序规定了这些完整单步变换的先后关系。^[error-global-tits.md:75-90]

该过程作用于[[全局 Tits 元素的精确有理环面表示]]：载体保留完整有理余特征，包括中心坐标，环面坐标取 `[0,2)` 中的典范代表元；向纤维 mod-two 商的规约发生在之后。^[error-global-tits.md:60-65]

## 校验与错误传播

`crossed_word` 在遍历前执行一次上下文校验，每次 `crossed_generator` 又重新校验。校验要求 Weyl 作用与根对合使用相同的根数据，并要求 `w·δ` 的权与余权矩阵均与存储对合一致；不满足时分别报告 `DatumMismatch` 或 `DistinguishedInvolutionMismatch`。相关契约见[[全局 Tits 传输的上下文一致性校验]]。^[error-global-tits.md:77-79, error-global-tits.md:94-97, error-global-tits.md:101-104]

字中的生成元不会在入口统一预检：只有折叠执行到非法生成元时，才报告 `IndexOutOfRange`。虚根分支还可能因配对不整而报告 `InvalidStrongTorusFactor`。这些失败条件属于逐步执行过程的一部分。^[error-global-tits.md:77-85, error-global-tits.md:94-97]

## 测试与证据边界

`a2_word_execution_is_forward_and_noncommuting` 使用 `assert_ne!(forward, reverse)` 锚定前向执行顺序及其不可随意反转的性质；另有 `rank_zero_and_an_empty_word_are_identity_transport` 覆盖秩零与空字的恒等传输情形。这些是具体测试锚点，不能据此断言每个字的前向与逆向结果都不同。^[error-global-tits.md:120-123, error-global-tits.md:131-132]

源材料明确指出，非空字的错误传播路径尚未被测试覆盖。本资料属于结构性源码阅读，不声称全局 Tits 传输已获数学验收；此次知识维护也未执行测试或 benchmark。更多覆盖范围见[[全局 Tits 传输的测试锚点与证据边界]]。^[error-global-tits.md:10-14, error-global-tits.md:140-142, error-global-tits.md:158-162]

## Sources

- [error-global-tits.md](../../sources/error-global-tits.md) — StructureError 错误分类学与全局 Tits 交叉作用传输层（error.rs / global_tits.rs）

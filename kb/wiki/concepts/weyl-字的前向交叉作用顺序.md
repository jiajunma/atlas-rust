---
title: Weyl 字的前向交叉作用顺序
summary: crossed_word 按切片顺序逐生成元折叠交叉作用，非交换情形下顺序影响结果，非法生成元仅在执行到相应位置时报错。
sources:
  - error-global-tits.md
kind: concept
createdAt: "2026-10-09T14:46:21.407Z"
updatedAt: "2026-10-10T00:31:32.441Z"
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
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: Weyl 字的前向交叉作用顺序
summary: crossed_word 按切片顺序逐生成元执行交叉作用；非交换情形下顺序影响结果，非法生成元在执行到对应位置时才报错。
sources:
  - error-global-tits.md
kind: concept
tags:
  - Weyl群
  - 执行顺序
  - 错误处理
aliases:
  - weyl-字的前向交叉作用顺序
---

# Weyl 字的前向交叉作用顺序

全局 Tits 传输中的 `crossed_word` 按 Weyl 字的**切片顺序**逐个执行交叉作用。这一约定匹配上游 `cross_act(GlobalTitsElement&, const WeylWord&)`；A2 非交换测试明确区分前向与逆向执行的结果。^[error-global-tits.md:92-97, error-global-tits.md:131-132]

## 前向折叠的含义

设字为 \( [i_0,i_1,\ldots,i_{k-1}] \)，初始元素为 \(x_0\)，则依次计算 \(x_{j+1}=x_j.\mathrm{crossed\_generator}(i_j)\)，最终返回 \(x_k\)。“前向”指从切片首项向末项遍历，每一步使用前一步返回的元素。单生成元方法采用 `&self -> Result<Self>`，原值不变。^[error-global-tits.md:90-97]

作用对象 [[全局 Tits 元素的精确有理环面表示|GlobalTitsElement]] 保存完整有理余特征（含中心坐标）及 twisted involution。环面坐标取 \([0,2)\) 中的典范代表元，向纤维 mod-two 商的规约发生在之后。^[error-global-tits.md:60-65]

## 每一步执行的变换

单生成元交叉作用记为 \(s*(t,w)*\delta(s)\)。对于单根 \(\alpha_i\)，复根分支执行 \(t\leftarrow t-\langle\alpha_i,t\rangle\alpha_i^\vee\)；虚根分支要求配对为整数，再执行 \(t\leftarrow t+(1-\langle\alpha_i,t\rangle)\alpha_i^\vee\)；实根分支保持环面坐标不变。具体规则见 [[按根类型划分的单生成元交叉作用]]。^[error-global-tits.md:75-85]

每一步随后将环面坐标逐项模 2 规范化，并将 Weyl 作用更新为 \(s_i\circ w\circ s_{\delta(i)}\)，再经 `TwistedInvolution::new` 重建。因此，前向遍历规定的是这些完整单步变换的执行次序。^[error-global-tits.md:87-97]

## 校验与错误传播

`crossed_word` 在遍历前调用一次 `validate_context`，每次 `crossed_generator` 又重新校验。Weyl 作用或根对合的 datum 与 `inner_class.datum()` 不同时，返回 `DatumMismatch`；否则比较 \(w\cdot\delta\) 的权与余权矩阵和存储的对合矩阵，任一不一致即返回 `DistinguishedInvolutionMismatch`。参见 [[全局 Tits 传输的上下文一致性校验]]。^[error-global-tits.md:77-79, error-global-tits.md:94-97, error-global-tits.md:101-104]

字中的生成元不会在入口统一预检：只有折叠执行到 `generator >= semisimple_rank` 的位置时，才返回 `IndexOutOfRange`。单步执行还可能因无法确定根类型而返回 `InvalidRootAutomorphism`，或因虚根配对不整而返回 `InvalidStrongTorusFactor`。^[error-global-tits.md:77-85, error-global-tits.md:94-97]

## 测试与证据边界

`a2_word_execution_is_forward_and_noncommuting` 通过 `assert_ne!(forward, reverse)` 锚定前向与逆向结果不同的 A2 案例；`rank_zero_and_an_empty_word_are_identity_transport` 覆盖秩零与空字的恒等传输情形。这些是源材料记录的测试锚点。^[error-global-tits.md:120-123, error-global-tits.md:131-132]

非空字的错误传播被明确列为未覆盖路径。本页依据结构性源码阅读，不声称全局 Tits 传输已获数学验收；该次知识维护未执行 Atlas、Cargo、测试或 benchmark。更多覆盖限制见 [[全局 Tits 传输的测试锚点与证据边界]]。^[error-global-tits.md:10-14, error-global-tits.md:140-142, error-global-tits.md:158-162]

## Sources

- [error-global-tits.md](../../sources/error-global-tits.md) — StructureError 错误分类学与全局 Tits 交叉作用传输层（error.rs / global_tits.rs）。

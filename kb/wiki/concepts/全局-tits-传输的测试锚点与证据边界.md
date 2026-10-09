---
title: 全局 Tits 传输的测试锚点与证据边界
summary: 源码列出的 10 个测试涵盖规范化、根类型分支、非交换执行顺序、余根方向及中心坐标等行为，但部分错误路径未覆盖，结构性阅读不构成测试执行或数学验收。
sources:
  - error-global-tits.md
kind: concept
createdAt: "2026-10-09T14:47:00.754Z"
updatedAt: "2026-10-09T14:47:00.754Z"
tags:
  - 测试覆盖
  - 证据边界
  - Tits交叉作用
aliases:
  - 全局-tits-传输的测试锚点与证据边界
  - 全T传
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 全局 Tits 传输的测试锚点与证据边界

全局 Tits 传输由 `global_tits.rs` 实现，用于全局 Tits 交叉作用的精确有理环面传输。其载体 `GlobalTitsElement` 保留含中心坐标的完整有理余特征，并将环面坐标规范化到 `[0, 2)`；向纤维 mod-two 商的规约发生在后续阶段。相关表示见[[全局 Tits 元素的精确有理环面表示]]。^[error-global-tits.md:58-65]

## 测试锚点

源文件中的测试模块包含 10 个 `#[test]`，覆盖坐标规范化、根类型分支、作用顺序及部分输入错误。来源材料仅报告结构性阅读与源码核对，本次知识维护未执行测试，因此这些锚点不能视为测试通过记录或 Tits 传输的数学验收。^[error-global-tits.md:9-14, error-global-tits.md:118-138, error-global-tits.md:158-162]

### 坐标规范化与边界情形

`normalizes_every_coordinate_modulo_two` 检查 `(−1/2, 9/2)` 规范化为 `(3/2, 1/2)`；`rank_zero_and_an_empty_word_are_identity_transport` 覆盖秩为零与空字的恒等传输；`a1_with_central_torus_preserves_the_central_coordinate` 检查带中心环面的 A1 情形，输入 `(0, 7/3)` 得到 `(1, 1/3)`，其中中心坐标按 mod-2 保留。^[error-global-tits.md:122-123, error-global-tits.md:134-135]

### 根类型分支

虚根分支要求根与环面因子的配对为整数，然后以 `1 − pairing` 为系数加上余根。对应的两个 adjoint A1 测试分别检查紧因子 `1/2` 不变、非紧因子 `0 → 1`，以及坐标 `1/4` 因不满足整性门槛而精确返回 `InvalidStrongTorusFactor`。^[error-global-tits.md:83-84, error-global-tits.md:124-127]

实根测试 `a1_real_cross_leaves_both_components_unchanged` 检查 A1 实根交叉后两个分量均不变。复根测试 `a2_complex_cross_reflects_the_rational_coweight` 检查 `(1/3, 1/2)` 经 `crossed_generator(1)` 变为 `(5/6, 3/2)`，并检查 Weyl 作用为 `s1∘s0∘s1`；B2 测试进一步锚定更新使用余根方向。根类型背景见[[对合下的虚根、实根与复根分类]]。^[error-global-tits.md:128-133]

### Weyl 字的执行顺序

`crossed_word` 按切片顺序逐个执行 `crossed_generator`，与上游 `cross_act(GlobalTitsElement&, const WeylWord&)` 的顺序约定一致。`a2_word_execution_is_forward_and_noncommuting` 通过 `assert_ne!(forward, reverse)` 锚定非交换情形下正向与逆向执行的差异，相关约定见[[Weyl 字的前向交叉作用顺序]]。^[error-global-tits.md:92-97, error-global-tits.md:131-132]

### 输入与上下文错误

`rejects_rank_generator_datum_and_distinguished_mismatches` 对四类错误作精确断言：`RankMismatch`、`IndexOutOfRange`、`DatumMismatch` 和 `DistinguishedInvolutionMismatch`；最后一种使用 A2 交换对合构造。上下文校验同时检查 datum 一致性，以及 `w·δ` 的 weight/coweight 矩阵是否与存储的对合矩阵一致，详见[[全局 Tits 传输的上下文一致性校验]]。^[error-global-tits.md:101-104, error-global-tits.md:136-138]

## 未覆盖分支与实现边界

明确未覆盖的分支包括 `InvalidRootAutomorphism`、`InvalidBasedAutomorphism`、`rational_pair` 与 `add_scaled_coroot` 内部的 `RankMismatch`、`try_capacity` 失败路径，以及非空字的错误传播。两个辅助函数的同秩条件已由调用点保证，但这不等于这些内部错误分支已有测试。^[error-global-tits.md:140-142]

非法生成元不会在 `crossed_word` 入口统一预检，而是在逐步折叠到该生成元时返回 `IndexOutOfRange`。单生成元越界错误有测试锚点，非空字中的传播路径仍属于上述未覆盖范围。^[error-global-tits.md:94-97, error-global-tits.md:136-142]

`global_tits.rs` 直接构造 7 个 `StructureError` 变体；`try_capacity`、`compose_matrices`、`WeylAction::*` 和 `TwistedInvolution::new` 的错误还会经 `?` 传播，但来源包没有枚举这些传播错误的具体变体。`error.rs` 自身没有测试，因此本包不能证明[[StructureError 统一错误分类学]]的整体覆盖情况。^[error-global-tits.md:112-120]

直接索引 `datum.simple_roots()[generator]` 的保护依赖 `generator < semisimple_rank`；来源包没有核查 `semisimple_rank ≤ simple_roots().len()` 不变量，将其列为潜在 panic 面的阅读观察。此外，`GlobalTitsElement` 的派生 `Eq` 依赖 `RationalCoweight` 与 `TwistedInvolution` 的相等定义，而这些定义不在本包范围内。^[error-global-tits.md:144-151]

## 证据身份与适用范围

来源包覆盖 `error.rs` 的 322 行与 `global_tits.rs` 的 551 行，后者包含测试模块。读取身份记录于快照 `2026-10-06-error-global-tits.json`，绑定 Git base、两文件字节 SHA-256 与 Kimi 调用记录；草案经维护者对照源码逐条核对改写。本包支持对实现结构、测试断言和覆盖缺口的说明，不声称错误覆盖面或 Tits 传输的数学验收，也不包含本次 Atlas、Cargo、测试或 benchmark 的执行结果。^[error-global-tits.md:9-14, error-global-tits.md:158-162]

## Sources

- [error-global-tits.md](../../sources/error-global-tits.md)

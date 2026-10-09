---
title: 全局 Tits 传输的测试锚点与证据边界
summary: 来源记录十个测试锚点，覆盖规范化、根类型、执行顺序与中心坐标，但仍缺部分错误路径覆盖，结构性阅读不构成测试通过或数学验收证据。
sources:
  - error-global-tits.md
kind: concept
createdAt: "2026-10-09T14:47:00.754Z"
updatedAt: "2026-10-09T22:28:36.961Z"
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
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: 全局 Tits 传输的测试锚点与证据边界
summary: global_tits.rs 的十个测试锚定环面规范化、根类型分支、前向执行顺序与部分错误路径；来源仅为结构性阅读，不构成测试通过记录或数学验收。
sources:
  - error-global-tits.md
kind: concept
tags:
  - 测试覆盖
  - 证据边界
  - Tits交叉作用
aliases:
  - 全局-tits-传输的测试锚点与证据边界
---

# 全局 Tits 传输的测试锚点与证据边界

`global_tits.rs` 实现全局 Tits 交叉作用的精确有理环面传输。`GlobalTitsElement` 保留含中心坐标的完整有理余特征，将环面坐标规范化到 `[0, 2)`，之后才向纤维的 mod-two 商规约，详见 [[全局 Tits 元素的精确有理环面表示]]。^[error-global-tits.md:58-65]

来源记录该模块的十个 `#[test]`，但本次知识维护仅完成结构性阅读与源码核对，未执行 Atlas、Cargo、测试或 benchmark。下述内容说明测试断言和覆盖范围，不代表测试已通过，也不构成 Tits 传输的数学验收。^[error-global-tits.md:9-14, error-global-tits.md:118-120, error-global-tits.md:158-162]

## 测试锚点

### 坐标规范化与边界情形

`normalizes_every_coordinate_modulo_two` 检查 `(−1/2, 9/2) → (3/2, 1/2)`；`rank_zero_and_an_empty_word_are_identity_transport` 锚定秩零与空字的恒等传输。^[error-global-tits.md:122-123]

`a1_with_central_torus_preserves_the_central_coordinate` 检查带中心环面的 A1 情形：`(0, 7/3) → (1, 1/3)`。中心坐标在模 2 意义下保留，其数值代表元仍会规范化。^[error-global-tits.md:134-135]

### 虚根、实根与复根分支

虚根分支要求根与环面因子的配对为整数，随后以 `1 − pairing` 为系数加上余根。`a1_imaginary_cross_distinguishes_compact_and_noncompact_factors` 在伴随型 A1 中检查紧因子 `1/2` 不变、非紧因子 `0 → 1`；`a1_imaginary_cross_requires_an_integral_root_pairing` 检查坐标 `1/4` 精确返回 `InvalidStrongTorusFactor`。^[error-global-tits.md:83-84, error-global-tits.md:124-127]

`a1_real_cross_leaves_both_components_unchanged` 检查 A1 实根交叉后环面与扭曲对合两个分量均不变。`a2_complex_cross_reflects_the_rational_coweight` 检查 `(1/3, 1/2)` 经 `crossed_generator(1)` 变为 `(5/6, 3/2)`，Weyl 作用为 `s1∘s0∘s1`。^[error-global-tits.md:128-130]

复根分支采用精确有理更新 \(t \leftarrow t-\langle\alpha,t\rangle\alpha^\vee\)。`b2_complex_cross_uses_the_coroot_not_the_root_direction` 专门锚定更新使用余根方向；根类型背景见 [[对合下的虚根、实根与复根分类]]。^[error-global-tits.md:79-85, error-global-tits.md:133-133]

### Weyl 字的执行顺序

`crossed_word` 按切片顺序逐个执行 `crossed_generator`，文档注明匹配上游 `cross_act(GlobalTitsElement&, const WeylWord&)`。`a2_word_execution_is_forward_and_noncommuting` 以 `assert_ne!(forward, reverse)` 锚定非交换情形下正向与逆向执行的差异，详见 [[Weyl 字的前向交叉作用顺序]]。^[error-global-tits.md:92-97, error-global-tits.md:131-132]

### 输入与上下文错误

`rejects_rank_generator_datum_and_distinguished_mismatches` 对四个错误作精确断言：`RankMismatch`、`IndexOutOfRange`、`DatumMismatch` 和 `DistinguishedInvolutionMismatch`，最后一种使用 A2 交换对合构造。上下文校验要求 datum 一致，并比较 `w·δ` 的 weight/coweight 矩阵与存储的对合矩阵，参见 [[全局 Tits 传输的上下文一致性校验]]。^[error-global-tits.md:101-104, error-global-tits.md:136-138]

## 覆盖缺口

来源明确列出的未覆盖路径包括 `InvalidRootAutomorphism`、`InvalidBasedAutomorphism`、`rational_pair` 与 `add_scaled_coroot` 内部的 `RankMismatch`、`try_capacity` 失败，以及非空字的错误传播。两个辅助函数的同秩条件已由调用点保证，但其内部错误分支没有测试覆盖。^[error-global-tits.md:140-142]

`crossed_word` 不在入口统一预检非法生成元，而是在折叠执行到该生成元时返回 `IndexOutOfRange`。来源同时记录单生成元越界的精确错误断言，以及非空字错误传播的覆盖缺口。^[error-global-tits.md:94-97, error-global-tits.md:136-142]

`global_tits.rs` 直接构造七个 `StructureError` 变体；`try_capacity`、`compose_matrices`、`WeylAction::*` 和 `TwistedInvolution::new` 的错误还会经 `?` 传播，具体变体不在来源包范围内。`error.rs` 自身没有测试，其余 46 个变体的构造点分散在其他模块；完整分类见 [[StructureError 统一错误分类学]]。^[error-global-tits.md:112-120, error-global-tits.md:154-154]

## 实现与证据边界

`datum.simple_roots()[generator]` 等直接索引的越界保护依赖 `generator < semisimple_rank`。来源未核查 `semisimple_rank ≤ simple_roots().len()` 不变量，仅将其列为潜在 panic 面的阅读观察。`GlobalTitsElement` 的派生 `Eq` 依赖 `RationalCoweight` 与 `TwistedInvolution` 的相等定义，这些定义也不在本包范围内。^[error-global-tits.md:144-151]

来源覆盖 `error.rs` 的 322 行与 `global_tits.rs` 的 551 行，后者包含测试模块。快照 `2026-10-06-error-global-tits.json` 绑定 Git base、两文件字节 SHA-256 与 Kimi 调用记录；草案由维护者对照源码逐条核对改写。这些记录说明阅读对象与证据来源，不声称错误覆盖面或数学验收。^[error-global-tits.md:9-14, error-global-tits.md:158-162]

## Sources

- [error-global-tits.md](../../sources/error-global-tits.md) — StructureError 错误分类学与全局 Tits 交叉作用传输层（error.rs / global_tits.rs）。

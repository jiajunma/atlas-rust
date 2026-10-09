---
title: atlas-core 回归测试库的家族组织
summary: 语言层回归库按 session、typed、domain_builtins 与 session_fixture_tests 四个模块组织，并以测试名前缀呈现功能家族；本来源仅完成结构性阅读。
sources:
  - atlas-core-regression-library.md
kind: concept
createdAt: "2026-10-09T14:31:56.011Z"
updatedAt: "2026-10-09T14:31:56.011Z"
tags:
  - 回归测试
  - 测试组织
  - 语言层
aliases:
  - atlas-core-回归测试库的家族组织
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# atlas-core 回归测试库的家族组织

atlas-core 的语言层回归测试库由 `session.rs`、`typed.rs`、`domain_builtins.rs` 和 `session_fixture_tests.rs` 四个测试模块构成。这里的家族组织依据测试名称前缀及模块职责描述；来源完成了结构性阅读，不包含逐测试内容审查，也不代表测试已通过。^[atlas-core-regression-library.md:9-13]

## 模块与测试家族

`session.rs` 包含 201 个测试。按名称前缀统计，较大的家族包括 `named`（10）、`while`（8），以及 `polynomial`、`operator`、`generic`、`for`（各 7）；其次有 `weyl`、`torus`、`returns`（各 6），`completion`、`byte`（各 5），以及 `recursive`、`psp4`、`overload`、`matrix`（各 4）。^[atlas-core-regression-library.md:17-20]

`typed.rs` 包含 133 个测试，以 `convert*`（31）和 `overload*`（15）为较大的家族；其他家族包括 `evaluate*`（10），`multi*`、`expect*`（各 8），`matrix*`、`domain*`（各 6），`execute*`（5），以及 `arbitrary*`、`apply*`（各 4）。^[atlas-core-regression-library.md:23-25]

`domain_builtins.rs` 包含 92 个测试。家族涉及提取助手 `as_*`（18）、构造 `build_*`（16），以及 `block*`（15）、`print*`（11）、`weyl*`（10）、`twisted*`、`relation*`（各 9）、`cartan*`（7）、`validate*`、`simple*`、`root*`、`lazy*`（各 6）。这些数字是来源列出的前缀家族计数。^[atlas-core-regression-library.md:26-29]

`session_fixture_tests.rs` 包含 17 个测试，经 `session::run_source` 执行完整流程，将命令恢复、类型转换和求值一起纳入回归。该模块刻意不保留已移除的动态求值器作为第二实现，相关主题见 [[会话全流程回归测试]]。^[atlas-core-regression-library.md:30-32]

## 原版背书的回归模式

`session.rs` 中的原版背书回归通过 `include_str!` 载入 `tests/math/generics/` 下的 fixture，并逐字节比对 `.oracle.stdout` 与 `.oracle.stderr`；`weyl_context_core_cold_dual_original` 是这一模式的示例。语言层共有 244 处 `include_str!` 将 fixture 直接绑定到测试，相关主题见 [[原版背书的语言层回归测试]]。^[atlas-core-regression-library.md:20-22, atlas-core-regression-library.md:39-39]

每个 HPC 差分发现的差异都必须先在测试库中形成回归。fixture 和 `.oracle.*` 金标存放于 `tests/math/generics/`，金标仅来自原版的完整捕获，例如 Weyl A1 的 v8 冻结 goldens，不能以 Rust 输出充当金标。参见 [[基于原版金标的差分回归]]。^[atlas-core-regression-library.md:36-38]

## 证据与覆盖边界

[[HPC 验收证据链]] 包括 `tests/reference/hpc/` 下的分阶段证据 JSON、只追加的验收账本 `tests/reference/hpc/math_acceptance_index_2026_10_01.json`，以及 `docs/HANDOFF.md` 中的衔接记录；知识库不改写验收账本。^[atlas-core-regression-library.md:40-42]

上述单元测试及清单计数只对应 Git base `964f0033` 所标识的快照字节。测试的可执行正确性以 HPC 门验收为准，来源提及的门包括 632 项清单与 `atlas-core-test-inventory`；本页所依据的结构性材料不声称任何测试通过。^[atlas-core-regression-library.md:43-45]

各测试涉及的生产代码由会话、类型核心、表达式转换、求值、领域值、构造、形变、派发、校验打印及子群等所属来源包分别说明。`session_frame.rs` 的 18 个测试归入会话帧包，不属于这里的四模块范围，相关主题见 [[会话帧驱动的 CLI 执行模型]]。^[atlas-core-regression-library.md:49-55]

## Sources

- [atlas-core-regression-library.md](atlas-core-regression-library.md)

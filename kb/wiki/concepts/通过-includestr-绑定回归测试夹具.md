---
title: 通过 include_str! 绑定回归测试夹具
summary: 该快照的语言层使用 244 处 include_str! 绑定夹具，session 回归以此加载输入并逐字节比较原版 stdout 与 stderr。
sources:
  - atlas-core-regression-library.md
kind: concept
createdAt: "2026-10-09T14:32:10.647Z"
updatedAt: "2026-10-09T22:17:42.460Z"
tags:
  - Rust
  - 测试夹具
aliases:
  - 通过-includestr-绑定回归测试夹具
  - 通I绑
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: 通过 include_str! 绑定回归测试夹具
summary: atlas-core 语言层通过 include_str! 将夹具直接绑定进回归测试，并逐字节比对原版输出金标；引用计数不代表测试通过。
sources:
  - atlas-core-regression-library.md
kind: concept
tags:
  - Rust
  - 测试夹具
  - 回归测试
aliases:
  - 通过-includestr-绑定回归测试夹具
provenanceState: extracted
---

# 通过 include_str! 绑定回归测试夹具

atlas-core 语言层使用 `include_str!` 将测试夹具（fixture）直接绑定进测试，来源快照记录了 244 处此类引用。原版背书回归的典型模式是载入 `tests/math/generics/` 下的夹具，并将输出与 `.oracle.stdout`、`.oracle.stderr` 金标逐字节比对。^[atlas-core-regression-library.md:17-22, atlas-core-regression-library.md:36-39]

## 夹具与原版金标

`session.rs` 中的 `weyl_context_core_cold_dual_original` 是这一模式的示例。输入夹具与标准输出、标准错误金标共同构成回归依据，相关背景见 [[原版背书的语言层回归测试]]。^[atlas-core-regression-library.md:17-22]

每个 HPC 差分发现的差异都须先在测试库中形成回归，夹具与 `.oracle.*` 金标存放于 `tests/math/generics/`。金标只能来自原版的完整捕获，例如 Weyl A1 的 v8 冻结 goldens，不能使用 Rust 输出作为金标；这是 [[基于原版金标的差分回归]] 的来源约束。^[atlas-core-regression-library.md:36-38]

## 会话全流程回归

相关测试库中的 `session_fixture_tests.rs` 包含 17 个测试，经 `session::run_source` 执行完整会话路径，将命令恢复、类型转换和求值一起纳入回归。该模块刻意不保留已移除的动态求值器作为第二实现；其组织与执行范围可参见 [[atlas-core 回归测试库的家族组织]] 和 [[会话全流程回归测试]]。^[atlas-core-regression-library.md:30-32]

## 证据范围与验收

来源材料是对 `session.rs`、`typed.rs`、`domain_builtins.rs` 和 `session_fixture_tests.rs` 四个语言层测试模块的结构性阅读，不包含逐测试内容。引用数量及单元、清单计数仅描述 git base `964f0033` 对应的快照字节，不能据此断言任何测试通过；测试的可执行正确性由 HPC 门验收，例如 632 项清单与 `atlas-core-test-inventory`。^[atlas-core-regression-library.md:9-13, atlas-core-regression-library.md:39-45]

相关 [[HPC 验收证据链]] 包括 `tests/reference/hpc/` 下的分阶段证据 JSON、仅追加的验收账本 `tests/reference/hpc/math_acceptance_index_2026_10_01.json`，以及 `docs/HANDOFF.md` 中的衔接记录。知识库不改写该验收账本。^[atlas-core-regression-library.md:40-42]

## Sources

- [atlas-core-regression-library.md](../../sources/atlas-core-regression-library.md)：回归测试库地图——家族组织与原版背书模式。

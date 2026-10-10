---
title: 通过 include_str! 绑定回归测试夹具
summary: 该快照的语言层通过 244 处 include_str! 绑定夹具，session 回归范式比较原版 .oracle.stdout 与 .oracle.stderr 的精确字节。
sources:
  - atlas-core-regression-library.md
kind: concept
createdAt: "2026-10-09T14:32:10.647Z"
updatedAt: "2026-10-10T00:20:51.104Z"
tags:
  - 回归测试
  - Rust
aliases:
  - 通过-includestr-绑定回归测试夹具
  - 通I绑
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
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

atlas-core 语言层通过 `include_str!` 将测试夹具（fixture）直接绑定进测试，来源快照记录了 244 处此类引用。`session.rs` 中原版背书回归的典型模式是加载 `tests/math/generics/` 下的夹具，并将输出与 `.oracle.stdout`、`.oracle.stderr` 金标逐字节比对。^[atlas-core-regression-library.md:17-22, atlas-core-regression-library.md:36-39]

## 夹具与原版金标

`weyl_context_core_cold_dual_original` 是上述模式的示例。夹具提供测试输入，原版标准输出与标准错误金标提供比对依据，相关背景见 [[原版背书的语言层回归测试]]。^[atlas-core-regression-library.md:20-22]

每个 HPC 差分发现的差异都必须先在测试库中形成回归。夹具与 `.oracle.*` 金标存放于 `tests/math/generics/`；金标只能来自原版的完整捕获，例如 Weyl A1 的 v8 冻结 goldens，不能使用 Rust 输出作为金标。这是 [[基于原版金标的差分回归]] 的来源约束。^[atlas-core-regression-library.md:36-38]

## 会话全流程回归

相关测试库中的 `session_fixture_tests.rs` 包含 17 个测试，通过 `session::run_source` 执行完整会话路径，将命令恢复、类型转换和求值一起纳入回归。该模块刻意不保留已移除的动态求值器作为第二实现；参见 [[会话全流程回归测试]]。^[atlas-core-regression-library.md:30-32]

## 证据范围与验收

来源材料是对 `session.rs`、`typed.rs`、`domain_builtins.rs` 和 `session_fixture_tests.rs` 四个语言层测试模块的结构性阅读，不包含逐测试内容。模块组织可参见 [[atlas-core 回归测试库的家族组织]]。^[atlas-core-regression-library.md:9-13, atlas-core-regression-library.md:15-32]

244 处引用以及单元、清单计数仅描述 git base `964f0033` 对应的快照字节，不能据此断言任何测试通过。测试的可执行正确性以 HPC 门验收为准，来源列举的检查包括 632 项清单和 `atlas-core-test-inventory`。^[atlas-core-regression-library.md:39-45]

[[HPC 验收证据链]] 包括 `tests/reference/hpc/` 下的分阶段证据 JSON、仅追加的验收账本 `tests/reference/hpc/math_acceptance_index_2026_10_01.json`，以及 `docs/HANDOFF.md` 中的衔接记录。知识库不改写该验收账本。^[atlas-core-regression-library.md:40-42]

## Sources

- [atlas-core-regression-library.md](../../sources/atlas-core-regression-library.md)：回归测试库地图——家族组织与原版背书模式。

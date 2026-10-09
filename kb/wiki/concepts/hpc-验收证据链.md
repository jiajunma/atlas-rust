---
title: HPC 验收证据链
summary: 验收证据由 tests/reference/hpc/ 的分阶段 JSON、只追加的 math_acceptance_index_2026_10_01.json 账本及 docs/HANDOFF.md 衔接记录组成，知识库不改写验收账本。
sources:
  - atlas-core-regression-library.md
kind: concept
createdAt: "2026-10-09T14:32:09.842Z"
updatedAt: "2026-10-09T14:32:09.842Z"
tags:
  - HPC
  - 验收证据
  - 可追溯性
aliases:
  - hpc-验收证据链
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# HPC 验收证据链

HPC 验收证据链将差分发现、原版金标回归、分阶段证据、验收账本与交接记录联系起来。对于 atlas-core 语言层，测试库的结构性阅读与测试计数只描述特定源码快照；测试的可执行正确性以 HPC 门验收为准。^[atlas-core-regression-library.md:9-13,34-45]

## 从差分发现到回归测试

每个 HPC 差分发现的差异都必须先在测试库中形成回归。fixture 与 `.oracle.*` 金标保存在 `tests/math/generics/`；金标只能来自原版的完整捕获，例如 Weyl A1 的 v8 冻结 goldens，不能使用 Rust 输出作为金标。该规则是[[原版背书的语言层回归测试]]的证据基础。^[atlas-core-regression-library.md:36-38]

典型回归通过 `include_str!` 载入 fixture，并逐字节比对 `.oracle.stdout` 与 `.oracle.stderr`，例如 `weyl_context_core_cold_dual_original`。本快照的语言层共有 244 处 `include_str!` 将 fixture 直接绑定到测试中。^[atlas-core-regression-library.md:20-22,39-39]

## 验收记录的组成

证据链包括 `tests/reference/hpc/` 下的分阶段证据 JSON、`tests/reference/hpc/math_acceptance_index_2026_10_01.json` 验收账本，以及 `docs/HANDOFF.md` 的衔接记录。验收账本采用仅追加（append-only）方式维护，知识库不改写该账本。^[atlas-core-regression-library.md:40-42]

## 测试组织与证据边界

[[atlas-core 回归测试库的家族组织]]涵盖 `session.rs`、`typed.rs`、`domain_builtins.rs` 与 `session_fixture_tests.rs` 四个测试模块。其中，`session_fixture_tests.rs` 经由 `session::run_source` 将命令恢复、类型转换与求值纳入全程回归，并刻意不保留已移除的动态求值器作为第二实现。^[atlas-core-regression-library.md:9-13,30-32]

单元测试与清单计数只标识 Git 基线 `964f0033` 对应的本快照字节，不能据此断言测试通过。来源材料将 632 项清单、`atlas-core-test-inventory` 等 HPC 门作为可执行正确性的验收依据，但自身明确不声称任何测试已经通过。^[atlas-core-regression-library.md:43-45]

## Sources

- [atlas-core-regression-library.md](atlas-core-regression-library.md)

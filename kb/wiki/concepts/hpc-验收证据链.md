---
title: HPC 验收证据链
summary: 分阶段 HPC 证据 JSON、只追加的数学验收账本与 docs/HANDOFF.md 衔接记录共同形成验收证据链，知识库不改写账本。
sources:
  - atlas-core-regression-library.md
kind: concept
createdAt: "2026-10-09T14:32:09.842Z"
updatedAt: "2026-10-10T00:20:54.177Z"
tags:
  - HPC
  - 证据管理
aliases:
  - hpc-验收证据链
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: HPC 验收证据链
summary: 原版金标回归、分阶段 HPC 证据 JSON、只追加的验收账本与交接记录共同支撑验收追溯；结构性阅读与测试计数不代表测试通过。
sources:
  - atlas-core-regression-library.md
kind: concept
tags:
  - HPC
  - 证据管理
  - 验收
aliases:
  - hpc-验收证据链
---

# HPC 验收证据链

HPC 验收证据链将差分发现形成的回归测试，与分阶段证据 JSON、只追加的验收账本及交接记录衔接起来。测试库的结构性阅读和清单计数仅描述来源快照，测试的可执行正确性由 HPC 门验收，不能据此声称测试通过。^[atlas-core-regression-library.md:34-45]

## 从差分发现到原版金标回归

每个 HPC 差分发现的差异都必须先在测试库中形成回归。fixture 与 `.oracle.*` 金标存放于 `tests/math/generics/`；金标只来自原版的完整捕获，例如 Weyl A1 的 v8 冻结 goldens，不能使用 Rust 输出作为金标。相关主题见 [[原版背书的语言层回归测试]]。^[atlas-core-regression-library.md:36-38]

典型回归使用 `include_str!` 载入 fixture，并逐字节比对 `.oracle.stdout` 与 `.oracle.stderr`，例如 `weyl_context_core_cold_dual_original`。来源快照记录了语言层 244 处 `include_str!`，将 fixture 直接绑定到测试。^[atlas-core-regression-library.md:20-22, atlas-core-regression-library.md:39-39]

## 验收记录的组成

验收证据链包含三个记录位置：`tests/reference/hpc/` 下的分阶段证据 JSON、`tests/reference/hpc/math_acceptance_index_2026_10_01.json` 验收账本，以及 `docs/HANDOFF.md` 中的衔接记录。验收账本采用只追加（append-only）方式维护，知识库不改写账本。^[atlas-core-regression-library.md:40-42]

## 测试组织与证据边界

[[atlas-core 回归测试库的家族组织]]描述四个语言层测试模块：`session.rs` 的 201 个测试、`typed.rs` 的 133 个测试、`domain_builtins.rs` 的 92 个测试，以及 `session_fixture_tests.rs` 的 17 个测试。来源仅覆盖模块组织，不包含逐测试内容，其编辑状态是结构性阅读完成。^[atlas-core-regression-library.md:9-13]

其中，`session_fixture_tests.rs` 经 `session::run_source` 执行[[会话全流程回归测试]]，将命令恢复、类型转换与求值一起纳入回归，并刻意不保留已移除的动态求值器作为第二实现。^[atlas-core-regression-library.md:30-32]

上述单元测试与清单计数仅标识 Git 基线 `964f0033` 对应的来源快照字节。测试可执行正确性的依据是 HPC 门，例如 632 项清单与 `atlas-core-test-inventory`；本来源明确不声称任何测试通过，因此这些数量不能作为通过结果。^[atlas-core-regression-library.md:43-45]

## Sources

- [atlas-core-regression-library.md](../../sources/atlas-core-regression-library.md) — atlas-core 回归测试库地图：家族组织与原版背书模式。

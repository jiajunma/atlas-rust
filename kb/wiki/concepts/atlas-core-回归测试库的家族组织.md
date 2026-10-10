---
title: atlas-core 回归测试库的家族组织
summary: 四个测试模块按测试名前缀组织会话、类型转换、求值和领域内建回归；本来源仅覆盖组织结构，不覆盖逐测试内容。
sources:
  - atlas-core-regression-library.md
kind: concept
createdAt: "2026-10-09T14:31:56.011Z"
updatedAt: "2026-10-10T00:20:43.033Z"
tags:
  - 回归测试
  - 语言实现
aliases:
  - atlas-core-回归测试库的家族组织
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: atlas-core 回归测试库的家族组织
summary: atlas-core 语言层回归库按四个测试模块及测试名前缀组织，结合原版金标与 HPC 验收证据；结构性计数不代表测试通过。
sources:
  - atlas-core-regression-library.md
kind: concept
tags:
  - 回归测试
  - 语言实现
aliases:
  - atlas-core-回归测试库的家族组织
---

# atlas-core 回归测试库的家族组织

atlas-core 语言层回归测试库地图覆盖 `session.rs`、`typed.rs`、`domain_builtins.rs` 和 `session_fixture_tests.rs` 四个测试模块，描述模块组织与测试家族。来源属于结构性阅读，不包含逐测试内容审查；测试的可执行正确性由 HPC 门验收。^[atlas-core-regression-library.md:9-13]

## 模块与测试家族

`session.rs` 包含 201 个测试。按名称前缀统计，较大的家族为 `named`（10）、`while`（8），以及 `polynomial`、`operator`、`generic`、`for`（各 7）；其他家族包括 `weyl`、`torus`、`returns`（各 6），`completion`、`byte`（各 5），以及 `recursive`、`psp4`、`overload`、`matrix`（各 4）。^[atlas-core-regression-library.md:17-20]

`typed.rs` 包含 133 个测试。主要家族包括 `convert*`（31）、`overload*`（15）、`evaluate*`（10）、`multi*`、`expect*`（各 8）、`matrix*`、`domain*`（各 6）、`execute*`（5），以及 `arbitrary*`、`apply*`（各 4）。^[atlas-core-regression-library.md:23-25]

`domain_builtins.rs` 包含 92 个测试。来源列举的家族包括提取助手 `as_*`（18）、构造 `build_*`（16），以及 `block*`（15）、`print*`（11）、`weyl*`（10）、`twisted*`、`relation*`（各 9）、`cartan*`（7），还有 `validate*`、`simple*`、`root*`、`lazy*`（各 6）。^[atlas-core-regression-library.md:26-29]

`session_fixture_tests.rs` 包含 17 个测试，经 `session::run_source` 执行完整流程，将命令恢复、类型转换和求值一起纳入回归。该模块刻意不保留已移除的动态求值器作为第二实现，参见 [[会话全流程回归测试]]。^[atlas-core-regression-library.md:30-32]

## 原版背书的回归模式

`session.rs` 的原版背书回归通过 `include_str!` 载入 `tests/math/generics/` 中的 fixture，并逐字节比对 `.oracle.stdout` 与 `.oracle.stderr`；`weyl_context_core_cold_dual_original` 是这一模式的示例。语言层共有 244 处 `include_str!` 将 fixture 直接绑定到测试，相关主题见 [[原版背书的语言层回归测试]]。^[atlas-core-regression-library.md:20-22, atlas-core-regression-library.md:39-39]

每个 HPC 差分发现的差异都须先在测试库中形成回归。fixture 与 `.oracle.*` 金标存放于 `tests/math/generics/`；金标只来自原版的完整捕获，例如 Weyl A1 的 v8 冻结 goldens，不能用 Rust 输出充当金标。参见 [[基于原版金标的差分回归]]。^[atlas-core-regression-library.md:36-38]

## 验收证据与覆盖边界

[[HPC 验收证据链]] 包括 `tests/reference/hpc/` 下的分阶段证据 JSON、只追加的验收账本 `tests/reference/hpc/math_acceptance_index_2026_10_01.json`，以及 `docs/HANDOFF.md` 中的衔接记录。知识库不改写该验收账本。^[atlas-core-regression-library.md:40-42]

上述单元测试及清单计数仅对应 Git base `964f0033` 的本快照字节。测试的可执行正确性以 HPC 门为准，来源列举了 632 项清单和 `atlas-core-test-inventory`；这些结构性计数不代表测试通过，来源也未声称任何测试通过。^[atlas-core-regression-library.md:43-45]

各测试覆盖的生产代码由会话、类型核心、表达式转换、求值、领域值、构造、形变、派发、校验打印及子群等所属来源包分别说明。`session_frame.rs` 的 18 个测试归入会话帧包，不在本页四模块范围内，可结合 [[会话帧驱动的 CLI 执行模型]] 阅读。^[atlas-core-regression-library.md:49-55]

## Sources

- [atlas-core-regression-library.md](../../sources/atlas-core-regression-library.md) — 回归测试库地图：四个语言层测试模块的家族组织与原版背书模式。

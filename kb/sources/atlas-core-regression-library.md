---
title: 回归测试库地图（session/typed/domain_builtins/session_fixture_tests 的测试模块）——家族组织与原版背书模式
source: atlas-rust/atlas-core-regression-library
ingestedAt: 2026-10-09T19:00:00Z
---

# 回归测试库地图（atlas-core 的测试模块）

编辑状态：**结构性阅读完成；维护者直接撰写（无 Kimi 调用）**。本包覆盖
语言层的四个测试模块的组织（**不含**逐测试内容）：`session.rs`
（179–3889，201 个测试）、`typed.rs`（13741–18519，133 个）、
`domain_builtins.rs`（18442–22771，92 个）、`session_fixture_tests.rs`
（450 行，17 个）。结构性阅读；测试的可执行正确性由 HPC 门验收。

## 各模块的组织

- `session.rs` 测试（201 个）：按前缀的最大族 `named`（10）、`while`（8）、
  `polynomial`/`operator`/`generic`/`for`（各 7）、`weyl`/`torus`/`returns`
  （各 6）、`completion`/`byte`（各 5）、`recursive`/`psp4`/`overload`/
  `matrix`（各 4）等。原版背书回归的范式：`include_str!` 载入
  `tests/math/generics/` 的 fixture + 逐字节比对 `.oracle.stdout`/
  `.oracle.stderr`（如 `weyl_context_core_cold_dual_original`）。
- `typed.rs` 测试（133 个）：`convert*`（31）、`overload*`（15）、
  `evaluate*`（10）、`multi*`（8）、`expect*`（8）、`matrix*`/`domain*`
  （各 6）、`execute*`（5）、`arbitrary*`/`apply*`（各 4）等。
- `domain_builtins.rs` 测试（92 个）：`as_*`（18，提取助手）、`build_*`
  （16，构造）、`block*`（15）、`print*`（11）、`weyl*`（10）、
  `twisted*`（9）、`relation*`（9）、`cartan*`（7）、`validate*`（6）、
  `simple*`/`root*`/`lazy*`（各 6）等。
- `session_fixture_tests.rs`（17 个）：**经 `session::run_source` 全程**
  的回归（命令恢复 + 类型转换 + 求值一起），刻意不保留已移除的动态
  求值器作为第二实现。

## 原版背书与证据链

- 每个 HPC 差分发现的差异先在测试库落成回归（硬规则 7）：fixture 与
  `.oracle.*` 金标在 `tests/math/generics/`；金标只来自原版的完整捕获
  （如 Weyl A1 的 v8 冻结 goldens），**永不**用 Rust 输出当金标。
- 语言层 244 处 `include_str!` 把 fixture 直接绑进测试。
- 验收证据链：`tests/reference/hpc/` 的分阶段证据 JSON +
  `tests/reference/hpc/math_acceptance_index_2026_10_01.json`（append-only
  验收账本，KB 不改写）+ docs/HANDOFF.md 的衔接记录。
- 单元/清单计数只标识本快照字节（git base `964f0033`）；测试的可执行
  正确性以 HPC 门（如 632 项清单、atlas-core-test-inventory）为准——
  本包**不**声称任何测试通过。

## 边界

各测试覆盖的生产代码在其所属包（[会话](atlas-core-session.md)、
[typed-core](atlas-core-typed-core.md)、[convert-expr](atlas-core-convert-expr.md)、
[typed-eval](atlas-core-typed-eval.md)、[领域值](atlas-core-domain-values.md)、
[构造](atlas-core-domain-construction.md)、[形变](atlas-core-deformation-cache.md)、
[派发](atlas-core-domain-dispatch.md)、[校验打印](atlas-core-domain-validate-print.md)、
[子群](atlas-core-weyl-subgroup.md) 等）。`session_frame.rs` 的 18 个测试
在 [会话帧包](atlas-core-session-frame.md)。

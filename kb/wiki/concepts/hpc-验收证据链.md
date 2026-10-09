---
title: HPC 验收证据链
summary: 分阶段 HPC 证据 JSON、只追加的数学验收账本及 docs/HANDOFF.md 衔接记录共同构成验收证据链，知识库不改写账本。
sources:
  - atlas-core-regression-library.md
  - tests-math-library.md
kind: concept
createdAt: "2026-10-09T14:32:09.842Z"
updatedAt: "2026-10-09T22:18:03.134Z"
tags:
  - HPC
  - 证据管理
  - 验收
aliases:
  - hpc-验收证据链
confidence: 1
provenanceState: merged
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: HPC 验收证据链
summary: 分阶段 HPC 证据、只追加的哈希链验收账本与 HANDOFF 记录共同支撑验收追溯；原版完整流提供差分金标，结构性阅读、清单规模与 harness 失败均不能替代数学验收。
sources:
  - atlas-core-regression-library.md
  - tests-math-library.md
kind: concept
tags:
  - HPC
  - 验收证据
  - 可追溯性
aliases:
  - hpc-验收证据链
---

# HPC 验收证据链

HPC 验收证据链连接差分发现、原版金标回归、分阶段证据 JSON、只追加的验收账本与交接记录。它区分测试库的结构性描述与实际验收：测试组织、源码阅读和清单数量不代表测试通过，验收状态以账本与 HANDOFF 为准。^[atlas-core-regression-library.md:34-45, tests-math-library.md:89-97]

## 从差分发现到回归

每个 HPC 差分发现的差异都必须先在测试库中形成回归。fixture 与 `.oracle.*` 金标保存在 `tests/math/generics/`；原版可执行文件是语言 oracle，金标必须在新进程中捕获原版完整流，包含拒绝与恢复标记，不能使用 Rust 输出作为金标。相关主题见 [[原版背书的语言层回归测试]]。^[atlas-core-regression-library.md:36-38, tests-math-library.md:91-92]

典型回归通过 `include_str!` 载入 fixture，并逐字节比对 `.oracle.stdout` 与 `.oracle.stderr`，例如 `weyl_context_core_cold_dual_original`。atlas-core 来源快照记录了语言层 244 处 `include_str!`，将 fixture 直接绑定到测试。^[atlas-core-regression-library.md:20-22, atlas-core-regression-library.md:39-39]

期望结果的载体也需区分：渐进秩门中的 `.expect.json` 提供结构化断言，`.oracle.stdout` 保存原版逐字节流，两者不是同一种期望表示。^[tests-math-library.md:58-67]

## 分阶段证据、账本与交接记录

验收记录由 `tests/reference/hpc/` 下的分阶段证据 JSON、`tests/reference/hpc/math_acceptance_index_2026_10_01.json` 验收账本，以及 `docs/HANDOFF.md` 的衔接记录组成。账本采用只追加（append-only）方式维护，知识库不改写账本。^[atlas-core-regression-library.md:40-42]

每条验收通过追加哈希链记录，绑定确切源码、报告、独立审查、所选用例 ID、断言与限制。每个已发布后缀都必须有永久前缀检查点，否则只能视为草稿。^[tests-math-library.md:83-87]

数学测试库的 README 是逐弧线运行日志，其规则是最新 HANDOFF 与回执优先于旧记录。来源包不复制账本内容，也不自行确定验收状态；具体状态仍以账本与 HANDOFF 为准。^[tests-math-library.md:16-18, tests-math-library.md:97-97]

## 源码版本与快照范围

证据中的版本需要按其角色解释。`baseline.json` 记录的 oracle pin 为 `7e1b958c`，Rust 侧的 `05625c5d` 则是建立基准时保存的 main 分支历史字节，并非当前 HEAD；参见 [[双仓库基准版本钉定]]。^[tests-math-library.md:35-37]

数学测试库的结构快照包含未提交内容：`tests/math/catalog.json`、`tests/math/generics/catalog.json` 与 `tests/math/README.md` 存在 owner 修改，`tests/math/rank6/catalog.json` 尚未跟踪。因此，该来源描述的是取证时的工作区字节，不能当作已提交内容或数学验收结果。^[tests-math-library.md:9-18]

## 测试组织与证据边界

[[atlas-core 回归测试库的家族组织]]覆盖 `session.rs`、`typed.rs`、`domain_builtins.rs` 与 `session_fixture_tests.rs` 四个模块的组织，不包含逐测试内容。其中，`session_fixture_tests.rs` 经 `session::run_source` 将命令恢复、类型转换与求值纳入[[会话全流程回归测试]]，并刻意不保留已移除的动态求值器作为第二实现。^[atlas-core-regression-library.md:9-13, atlas-core-regression-library.md:30-32]

atlas-core 来源中的单元测试与清单计数仅标识 Git 基线 `964f0033` 对应的快照字节。来源以 632 项清单、`atlas-core-test-inventory` 等 HPC 门作为测试可执行正确性的验收依据，但明确不声称任何测试通过。同样，数学测试库的输入数、目录数与 HPC JSON 记录数属于覆盖清单，不能换算成正确性百分比。^[atlas-core-regression-library.md:43-45, tests-math-library.md:93-94]

harness 失败必须与数学失败区分。checker 自检失败、标签错配等既不是数学失败，也不能充当数学证据；不可变的失败阶段应保留冻结记录。^[tests-math-library.md:95-96]

## Sources

- [atlas-core-regression-library.md](../../sources/atlas-core-regression-library.md) — atlas-core 回归测试库地图：家族组织与原版背书模式。
- [tests-math-library.md](../../sources/tests-math-library.md) — 数学测试与基准库结构：目录、证据载体与验收纪律。

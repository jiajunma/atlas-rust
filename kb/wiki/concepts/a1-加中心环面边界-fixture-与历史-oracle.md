---
title: A1 加中心环面边界 fixture 与历史 oracle
summary: 11 个 A1+torus case 覆盖 root/coroot 交换、两种编号及 i32 边界，Rust 完整输出与历史 original-backed capture 比较；AFTER 作业未重跑原版，证据不推广至更高 rank 或一般根系。
sources:
  - root-ladder-overflow-repair.md
kind: concept
createdAt: "2026-10-09T15:11:16.618Z"
updatedAt: "2026-10-09T15:11:16.618Z"
tags:
  - 测试夹具
  - Oracle溯源
  - 适用范围
aliases:
  - a1-加中心环面边界-fixture-与历史-oracle
confidence: 1
provenanceState: merged
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# A1 加中心环面边界 fixture 与历史 oracle

A1 加中心环面边界 fixture 用于检验 root/coroot ladder 构造中的固定宽度坐标边界行为。它针对的错误是：Rust 将环境格坐标差的 `i32` 溢出视为整个 `RootSystem` 构造失败，而在完整存储的 `i32` 根或余根集合中，超出表示范围的精确差只能表示“不属于该集合”。^[root-ladder-overflow-repair.md:13-15, root-ladder-overflow-repair.md:41-53]

## Fixture 覆盖

测试文件 `tests/math/generics/root_ladder_coordinate_boundary.atlas` 包含 11 个 A1+torus case，覆盖 root/coroot 交换、两种 numbering、阈值下方、阈值上方以及 `i32::MAX`，并保留 recovery marker `719`。这些用例聚焦于 A1 加中心环面的坐标边界。^[root-ladder-overflow-repair.md:92-95]

对应的 [[Root ladder bottom 集与固定宽度成员查询]] 定义为 \(B_\alpha=\{\beta\in R\mid\beta-\alpha\notin R\}\)。当精确差的某个坐标超出 `i32` 范围时，成员查询应返回 `false`，使相应的 \(\beta\) 进入 bottom 集；这一结论不授权 wrapping 或 saturating 算术，也不适用于一般向量减法、反射或构造输入验证。^[root-ladder-overflow-repair.md:41-53]

## 历史 oracle 与来源绑定

fixture 的原版 oracle 来自历史 original-backed capture job `3868832`。其来源记录为 [`math_weyl_context_capture_2026_09_30.json`](../../tests/reference/hpc/math_weyl_context_capture_2026_09_30.json)，绑定 case、oracle binary/source 和 raw stream；原版完整 stdout 的 SHA-256 为 `3a7fdade43c46cf4f3048b52cf81f282db060012951ee559296f017f7d3eab80`。该捕获不是候选修复的 AFTER 执行，AFTER 作业也没有重新运行原版 oracle。^[root-ladder-overflow-repair.md:92-98, root-ladder-overflow-repair.md:124-130]

冻结的 original Atlas commit 为 `7e1b958c7aa9456769cc9cf09ac1542814b4800a`。其 `sources/structure/rootdata.cpp:238-317` 先在抽象简单根坐标 `Byte_vector` 中建立 simple-root ladder，再通过 Weyl reflection permutation 传送到所有正根和负根；中心环面嵌入产生的大环境格坐标不进入这一减法阶段。Rust 虽采用环境格坐标逐对相减，仍须保持相同的根/余根成员关系，参见 [[Original Atlas 的抽象坐标 ladder 构造]]。^[root-ladder-overflow-repair.md:57-66]

## Tests-first 与 AFTER 证据

BEFORE-v3 job `3873400` 的独立检查记录为 [`math_ladder_boundary_before_v3_2026_10_01.json`](../../tests/reference/hpc/math_ladder_boundary_before_v3_2026_10_01.json)。在未修复生产代码时，检查精确观察到 2 个 domain 失败与 1 个 core full-stream 失败，且 `0 ignored`；65 个 harness checker 通过，测试 inventory 为 atlas-real-group `521`、atlas-core `630`。这证明三条回归能够暴露原错误，属于 [[坐标边界修复的 tests-first 验证链]]，不能单独作为修复通过的证据。^[root-ladder-overflow-repair.md:100-107]

AFTER-v3 job `3875239` 以 `COMPLETED 0:0` 结束，独立检查记录 [`math_ladder_boundary_after_v3_2026_10_01.json`](../../tests/reference/hpc/math_ladder_boundary_after_v3_2026_10_01.json) 给出 `LADDER_BOUNDARY_AFTER_ACCEPTED`。记录涵盖 95 项 harness checker、完整 stager 清点 `70/67`、domain `521/2/521`、core `630/1/630`，以及源码和最终完整性复核。接受的 `root_system.rs` 字节 SHA-256 为 `cc6a1764e1c2425f7de8b4c27ca34a8bdc855c764d6db2a6ab9d6092e7b8cfe9`。^[root-ladder-overflow-repair.md:19-28]

## 接受范围与状态歧义

来源后段记载 acceptance-index entry `0003-a1-torus-root-coroot-ladder-boundary`，状态为 `acceptance: accepted`、`status: math_pass`。该条目绑定 AFTER-v3 报告、独立 review 与 canonical 源文件清单，断言原版接受全部 11 个 case 的 22 条记录、root/coroot 两条 kernel 回归及两个 crate 完整套件通过、Rust 完整流与历史捕获的原版一致，并确认三条 tests-first 回归在修复前失败。^[root-ladder-overflow-repair.md:115-126]

来源同时保留了前段“接受范围不含 acceptance-index 登记”的表述，与后段已登记条目的记载存在状态口径差异；应连同 [[Root ladder 修复的限定接受与账本状态歧义]] 阅读，不能将整份来源视为无歧义的同步状态说明。^[root-ladder-overflow-repair.md:9-11, root-ladder-overflow-repair.md:30-37, root-ladder-overflow-repair.md:115-123]

证据仅限这一 A1+中心环面坐标边界 fixture，不证明一般 root system 正确性，不覆盖更高 rank、KLV、unitarity、Hodge、associated cycle 或 AV-ann，也不构成性能或并行验收。完整 Rust 测试套件提供回归守卫，而非 oracle 证明；历史 oracle、AFTER 执行和独立 review 的角色应在 [[HPC 验收证据链]] 中保持区分。^[root-ladder-overflow-repair.md:109-113, root-ladder-overflow-repair.md:124-130]

## Sources

- [root-ladder-overflow-repair.md](../../sources/root-ladder-overflow-repair.md) — Root ladder 固定宽度坐标溢出修复。

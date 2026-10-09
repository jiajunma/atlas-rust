---
title: A1 加中心环面边界 fixture 与历史 oracle
summary: 11 个 A1+torus case 覆盖根余根交换、两种编号与 i32 边界，Rust 完整流对照历史原版捕获；AFTER 未重跑原版，证据不推广至更高 rank。
sources:
  - root-ladder-overflow-repair.md
kind: concept
createdAt: "2026-10-09T15:11:16.618Z"
updatedAt: "2026-10-09T22:48:24.752Z"
tags:
  - 测试夹具
  - 差分验证
  - 证据边界
aliases:
  - a1-加中心环面边界-fixture-与历史-oracle
  - A加F与O
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: A1 加中心环面边界 fixture 与历史 oracle
summary: 11 个 A1+torus case 检验 root/coroot ladder 的 i32 坐标边界；Rust 完整输出与历史原版捕获比较，接受范围仅限该 fixture，来源仍保留账本登记状态的口径差异。
sources:
  - root-ladder-overflow-repair.md
kind: concept
tags:
  - 回归测试
  - oracle
  - 证据范围
aliases:
  - a1-加中心环面边界-fixture-与历史-oracle
---

# A1 加中心环面边界 fixture 与历史 oracle

A1 加中心环面边界 fixture 检验 root/coroot ladder 构造中的固定宽度坐标边界。它针对的错误是：Rust 将环境格坐标差的 `i32` 溢出视为整个 `RootSystem` 构造失败，而对完整存储的 `i32` 根或余根集合，超出表示范围的精确差必定不属于该集合。^[root-ladder-overflow-repair.md:13-15, root-ladder-overflow-repair.md:41-53]

## Fixture 与数学判据

测试文件 `tests/math/generics/root_ladder_coordinate_boundary.atlas` 包含 11 个 A1+torus case，覆盖 root/coroot 交换、两种编号、阈值下方、阈值上方和 `i32::MAX`，并保留 recovery marker `719`。^[root-ladder-overflow-repair.md:92-95]

对完整存储的有限根集 \(R\subseteq\mathbb Z^d\) 和 \(\alpha\in R\)，ladder bottom 集定义为 \(B_\alpha=\{\beta\in R\mid\beta-\alpha\notin R\}\)。如果精确差的某个坐标超出 `i32` 范围，成员查询应返回 `false`，相应的 \(\beta\) 应进入 bottom 集，详见 [[Root ladder bottom 集与固定宽度成员查询]]。这一结论仅适用于该成员查询，不允许 wrapping 或 saturating 算术，也不能推广到一般向量减法、反射、root combination、seed negation 或构造输入验证。^[root-ladder-overflow-repair.md:41-53]

修复仅调整 `build_ladder_bottoms` 的两次成员查询：减法成功时保持原有查找方式，仅将 `StructureError::ArithmeticOverflow` 解释为不属于集合，分配失败及其他错误继续传播。root 与 coroot 查询独立执行，root 溢出不会跳过 coroot 查询；参见 [[Rust ladder 成员查询的选择性溢出处理]]。^[root-ladder-overflow-repair.md:74-83]

## 历史 oracle 的来源

原版 oracle 来自历史 original-backed capture job `3868832`。[捕获记录](../../tests/reference/hpc/math_weyl_context_capture_2026_09_30.json) 绑定 case、oracle binary/source 和 raw stream；原版完整 stdout 的 SHA-256 为 `3a7fdade43c46cf4f3048b52cf81f282db060012951ee559296f017f7d3eab80`。该捕获不是修复后的 AFTER 执行，AFTER 作业也没有重新运行原版 oracle。^[root-ladder-overflow-repair.md:92-98, root-ladder-overflow-repair.md:124-130]

冻结的 original Atlas commit 为 `7e1b958c7aa9456769cc9cf09ac1542814b4800a`。其 `sources/structure/rootdata.cpp:238-317` 先在抽象简单根坐标 `Byte_vector` 中建立 simple-root ladder，再用 Weyl reflection permutation 将表传送到所有正根和负根。中心环面嵌入产生的大环境格坐标不参与该减法阶段。Rust 使用环境格坐标逐对相减，但应保持相同的根/余根成员关系；详见 [[Original Atlas 的抽象坐标 ladder 构造]]。^[root-ladder-overflow-repair.md:57-66]

## Tests-first 与 AFTER 验证

BEFORE-v3 job `3873400` 的[独立检查记录](../../tests/reference/hpc/math_ladder_boundary_before_v3_2026_10_01.json) 表明：未修复生产代码时，精确出现 2 个 domain 失败和 1 个 core full-stream 失败，且 `0 ignored`；65 个 harness checker 通过，测试 inventory 为 atlas-real-group `521`、atlas-core `630`。这是三条回归能够暴露原错误的 tests-first 证据，不是修复后的通过结果。^[root-ladder-overflow-repair.md:100-107]

AFTER-v3 job `3875239` 以 `COMPLETED 0:0` 结束，[独立检查记录](../../tests/reference/hpc/math_ladder_boundary_after_v3_2026_10_01.json) 状态为 `LADDER_BOUNDARY_AFTER_ACCEPTED`。95 项 harness checker、完整 stager 清点 `70/67`、domain `521/2/521`、core `630/1/630` 以及源码和最终完整性复核全部通过。接受的 `root_system.rs` 字节 SHA-256 为 `cc6a1764e1c2425f7de8b4c27ca34a8bdc855c764d6db2a6ab9d6092e7b8cfe9`；相关过程见 [[坐标边界修复的 tests-first 验证链]]。^[root-ladder-overflow-repair.md:19-28]

## 账本记载与证据边界

来源后段记载 acceptance-index 已追加 entry `0003-a1-torus-root-coroot-ladder-boundary`，其状态为 `acceptance: accepted`、`status: math_pass`，并绑定 AFTER-v3 报告、独立 review 与 canonical 源文件清单。该条目断言：原版接受全部 11 个 case 的 22 条记录，两条 root/coroot kernel 回归及两个 crate 完整套件通过，Rust 完整流与历史原版捕获一致，且三条 tests-first 回归在修复前确实失败。^[root-ladder-overflow-repair.md:115-126]

来源前段仍保留“接受范围不含 acceptance-index 登记”的表述，后段则记载已登记条目；这两种口径在同一来源中并存，应保留其区别，参见 [[Root ladder 修复的限定接受与账本状态歧义]]。^[root-ladder-overflow-repair.md:9-11, root-ladder-overflow-repair.md:30-37, root-ladder-overflow-repair.md:115-123]

接受范围仅限 A1+中心环面坐标边界 fixture，不证明一般 root system 正确性，不覆盖更高 rank、KLV、unitarity、Hodge、associated cycle 或 AV-ann，也不构成性能或并行验收。Rust 完整套件是回归守卫，而非 oracle 证明；AFTER execution/report 不能自我验收，独立 review 也不能绕过正式账本。^[root-ladder-overflow-repair.md:109-113, root-ladder-overflow-repair.md:124-130]

## Sources

- [root-ladder-overflow-repair.md](../../sources/root-ladder-overflow-repair.md) — Root ladder 固定宽度坐标溢出修复。

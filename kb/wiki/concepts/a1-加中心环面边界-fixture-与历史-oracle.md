---
title: A1 加中心环面边界 fixture 与历史 oracle
summary: 11 个 A1+torus case 覆盖根余根交换、两种编号及 i32 边界，Rust 完整流对照历史原版捕获；AFTER 未重跑原版，证据不推广至更高 rank。
sources:
  - root-ladder-overflow-repair.md
kind: concept
createdAt: "2026-10-09T15:11:16.618Z"
updatedAt: "2026-10-10T00:51:21.159Z"
tags:
  - 回归测试
  - baseline
  - 证据边界
aliases:
  - a1-加中心环面边界-fixture-与历史-oracle
  - A加F与O
confidence: 1
provenanceState: merged
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: A1 加中心环面边界 fixture 与历史 oracle
summary: 11 个 A1+torus case 覆盖 root/coroot 交换、两种编号与 i32 坐标边界；Rust 完整输出对照历史原版捕获，AFTER 未重跑原版，接受范围不推广至更高 rank。
sources:
  - root-ladder-overflow-repair.md
kind: concept
tags:
  - 测试夹具
  - 差分验证
  - 证据边界
aliases:
  - a1-加中心环面边界-fixture-与历史-oracle
provenanceState: extracted
---

# A1 加中心环面边界 fixture 与历史 oracle

A1 加中心环面边界 fixture 用于检验 root/coroot ladder 构造中的 `i32` 坐标边界错误：Rust 原先将环境格坐标差溢出视为整个 `RootSystem` 构造失败。测试以历史 original Atlas 输出为 oracle，核对修复后的 Rust 完整输出；其证据范围限定于这一坐标边界问题。^[root-ladder-overflow-repair.md:13-15, root-ladder-overflow-repair.md:124-130]

## Fixture 覆盖与数学判据

测试文件 `tests/math/generics/root_ladder_coordinate_boundary.atlas` 包含 11 个 A1+torus case，覆盖 root/coroot 交换、两种 numbering、阈值下方、阈值上方及 `i32::MAX`，并保留 recovery marker `719`。来源中的账本条目记载，原版接受了全部 11 个 case 的 22 条记录。^[root-ladder-overflow-repair.md:92-95, root-ladder-overflow-repair.md:124-126]

对完整存储的有限根集 \(R\subseteq\mathbb Z^d\) 和 \(\alpha\in R\)，ladder bottom 集定义为 \(B_\alpha=\{\beta\in R\mid\beta-\alpha\notin R\}\)。若精确差的某个坐标超出 `i32` 范围，它就不可能等于任何已存向量，因此成员查询应返回 `false`，相应的 \(\beta\) 应进入 bottom 集。详见 [[Root ladder bottom 集与固定宽度成员查询]]。^[root-ladder-overflow-repair.md:41-49]

这一结论仅适用于完整 `i32` 存储集合上的精确差成员查询，不允许 wrapping 或 saturating 算术，也不能推广到一般向量减法、反射、root combination、seed negation 或构造输入验证。修复只在 `build_ladder_bottoms` 的两次查询中将 `StructureError::ArithmeticOverflow` 解释为不属于集合；分配失败及其他错误继续传播，root 溢出也不会跳过独立的 coroot 查询。参见 [[Rust ladder 成员查询的选择性溢出处理]]。^[root-ladder-overflow-repair.md:51-53, root-ladder-overflow-repair.md:74-83]

## 历史 oracle 的来源

原版 oracle 来自历史 original-backed capture job `3868832`。[捕获记录](../../tests/reference/hpc/math_weyl_context_capture_2026_09_30.json) 绑定了 case、oracle binary/source 与 raw stream；原版完整 stdout 的 SHA-256 为 `3a7fdade43c46cf4f3048b52cf81f282db060012951ee559296f017f7d3eab80`。这次捕获不是候选修复的 AFTER 执行，AFTER 作业也没有重新运行原版 oracle。^[root-ladder-overflow-repair.md:92-98, root-ladder-overflow-repair.md:124-130]

冻结的 original Atlas commit 为 `7e1b958c7aa9456769cc9cf09ac1542814b4800a`。其 `sources/structure/rootdata.cpp:238-317` 先用抽象简单根坐标 `Byte_vector` 建立 simple-root ladder，再通过 Weyl reflection permutation 将表传送到所有正根和负根。中心环面嵌入产生的大环境格坐标不参与这一减法阶段。Rust 虽采用环境格坐标逐对相减，仍须保持相同的根/余根成员关系；相关算法见 [[Original Atlas 的抽象坐标 ladder 构造]]。^[root-ladder-overflow-repair.md:57-66]

## Tests-first 与 AFTER 验证

BEFORE-v3 job `3873400` 的[独立检查记录](../../tests/reference/hpc/math_ladder_boundary_before_v3_2026_10_01.json) 表明，未修复生产代码时精确出现 2 个 domain 失败和 1 个 core full-stream 失败，且 `0 ignored`。65 个 harness checker 通过，测试 inventory 为 atlas-real-group `521`、atlas-core `630`。这些结果证明三条回归能暴露原错误，属于 tests-first 证据，并非修复后的通过结果。^[root-ladder-overflow-repair.md:100-107]

AFTER-v3 job `3875239` 以 `COMPLETED 0:0` 结束，[独立检查记录](../../tests/reference/hpc/math_ladder_boundary_after_v3_2026_10_01.json) 状态为 `LADDER_BOUNDARY_AFTER_ACCEPTED`。95 项 harness checker、完整 stager 清点 `70/67`、domain `521/2/521`、core `630/1/630`，以及源码和最终完整性复核全部通过。被接受的 `root_system.rs` 字节 SHA-256 为 `cc6a1764e1c2425f7de8b4c27ca34a8bdc855c764d6db2a6ab9d6092e7b8cfe9`。详见 [[坐标边界修复的 tests-first 验证链]]。^[root-ladder-overflow-repair.md:19-28]

## 账本记载与接受边界

来源后段记载 acceptance-index 已追加条目 `0003-a1-torus-root-coroot-ladder-boundary`，状态为 `acceptance: accepted`、`status: math_pass`。条目 SHA-256 为 `1628ee21c71a91376a02982c38404cee958cb6183c57f791a42c4a668a1efc12`，绑定 AFTER-v3 报告 `771fc790dd4340f408235e50c3f6eee754ebe4c850cbad36d4af4f902a24627c`、独立 review `3c0eed61cc5bf6af809096da73ac4bf1d8217b5cc81954a2e8000e5da44f9b58` 与 canonical 源文件清单 `d2a6367379432c1ed09b90c903a072cfce0fb463349032100dd644db7c3fc213`。^[root-ladder-overflow-repair.md:115-123]

该条目记载的通过范围包括两条 root/coroot kernel 回归、两个 crate 完整测试套件、Rust 完整流与历史原版捕获一致，以及三条 tests-first 回归在修复前确实失败。不过，来源前段仍保留“接受范围不含 acceptance-index 登记”的表述，与后段登记记载并存；这一状态口径差异见 [[Root ladder 修复的限定接受与账本状态歧义]]。^[root-ladder-overflow-repair.md:9-11, root-ladder-overflow-repair.md:30-37, root-ladder-overflow-repair.md:115-130]

接受范围仅限 A1+中心环面坐标边界 fixture，不构成一般 root system、KGB、KLV、unitarity、Hodge、associated cycle 或 AV-ann 正确性验收，也不覆盖更高 rank、性能、内存或并行结论。Rust 完整套件是回归守卫，而非 oracle 证明；AFTER execution/report 不能自我验收，独立 review 也不能绕过正式账本。^[root-ladder-overflow-repair.md:30-34, root-ladder-overflow-repair.md:109-113, root-ladder-overflow-repair.md:124-130]

## Sources

- [root-ladder-overflow-repair.md](../../sources/root-ladder-overflow-repair.md) — Root ladder 固定宽度坐标溢出修复。

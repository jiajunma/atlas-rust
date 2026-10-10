---
title: Root ladder 修复的限定接受与账本状态歧义
summary: 来源确认 AFTER-v3 的限定接受，但前文排除 acceptance-index 登记，后文又称条目已登记为 accepted/math_pass，账本状态叙述仍需核对。
sources:
  - root-ladder-overflow-repair.md
kind: concept
createdAt: "2026-10-09T15:11:23.576Z"
updatedAt: "2026-10-10T00:51:24.653Z"
tags:
  - 知识维护
  - 验收状态
  - 证据边界
aliases:
  - root-ladder-修复的限定接受与账本状态歧义
  - RL修
confidence: 1
provenanceState: ambiguous
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: Root ladder 修复的限定接受与账本状态歧义
summary: AFTER-v3 已有坐标边界修复的限定接受记录；来源前段排除 acceptance-index 登记，后段声明条目已登记为 accepted/math_pass，两者的状态衔接仍需核对。
sources:
  - root-ladder-overflow-repair.md
kind: concept
tags:
  - 验收范围
  - 证据审查
  - 来源歧义
aliases:
  - root-ladder-修复的限定接受与账本状态歧义
provenanceState: ambiguous
---

# Root ladder 修复的限定接受与账本状态歧义

Root ladder 固定宽度坐标溢出修复的 AFTER-v3 已有明确的限定接受记录。来源前段将 acceptance-index 登记排除在接受范围之外，后段又声明正式条目已追加，状态为 `accepted`／`math_pass`。因此，修复验证结果、接受范围和账本登记声明需要分别呈现。^[root-ladder-overflow-repair.md:9-11, root-ladder-overflow-repair.md:19-34, root-ladder-overflow-repair.md:115-130]

## 修复对象与版本边界

对完整存储的有限根集 \(R\subseteq\mathbb Z^d\)，ladder bottom 集定义为 \(B_\alpha=\{\beta\in R\mid\beta-\alpha\notin R\}\)。当所有已存坐标均为 `i32` 时，超出该类型表示范围的精确差不可能属于已存集合。修复仅将 `build_ladder_bottoms` 两次成员查询中的 `StructureError::ArithmeticOverflow` 解释为“不属于”，保持 root 与 coroot 查询独立，并继续传播分配失败及其他错误。该规则不适用于一般向量减法、反射或输入验证；详见 [[Root ladder bottom 集与固定宽度成员查询]]、[[Rust ladder 成员查询的选择性溢出处理]]。^[root-ladder-overflow-repair.md:41-53, root-ladder-overflow-repair.md:74-83]

接受绑定于 `crates/atlas-real-group/src/root_system.rs` 的特定字节，SHA-256 为 `cc6a1764e1c2425f7de8b4c27ca34a8bdc855c764d6db2a6ab9d6092e7b8cfe9`；候选 Git base 为 `eba9c7ea080e61de9d4b105fbf54589c44a10b87`。来源保留候选快照，并另以 2026-10-03 阅读快照记录状态推进。^[root-ladder-overflow-repair.md:27-34, root-ladder-overflow-repair.md:70-72]

## 限定接受的证据链

BEFORE-v3（job `3873400`）在未修复生产代码上精确观察到两条 domain 失败和一条 core full-stream 失败，且 `0 ignored`。65 个 harness checker 通过，测试 inventory 为 atlas-real-group `521`、atlas-core `630`。这是三条回归能够暴露原问题的 tests-first 证据，不是修复后的通过结果；参见 [[坐标边界修复的 tests-first 验证链]]。^[root-ladder-overflow-repair.md:100-107]

AFTER-v3（job `3875239`）最终状态为 `COMPLETED 0:0`。[独立检查记录](../../tests/reference/hpc/math_ladder_boundary_after_v3_2026_10_01.json) 标记为 `LADDER_BOUNDARY_AFTER_ACCEPTED`：95 项 harness checker、完整 stager 清点 `70/67`、domain `521/2/521` 与 core `630/1/630` 全部通过，源码与最终完整性复核也通过。^[root-ladder-overflow-repair.md:19-28]

解释器 fixture 包含 11 个 A1+torus case，覆盖 root/coroot 交换、两种编号、阈值两侧及 `i32::MAX`，并保留 recovery marker `719`。原版 oracle 来自历史 capture job `3868832`，其[捕获记录](../../tests/reference/hpc/math_weyl_context_capture_2026_09_30.json) 绑定 case、oracle binary/source 与 raw stream。AFTER 作业没有重跑原版，而是将 Rust 完整流与历史捕获比较；详见 [[A1 加中心环面边界 fixture 与历史 oracle]]。^[root-ladder-overflow-repair.md:92-98, root-ladder-overflow-repair.md:124-130]

## 账本登记的两种口径

来源开头及“2026-10-03 状态推进”节明确表示，接受范围不含 acceptance-index 登记。来源同时说明，后续保留的“候选”“尚未”等旧措辞，仅在该状态推进节所述范围内被新的接受记录取代。^[root-ladder-overflow-repair.md:9-11, root-ladder-overflow-repair.md:30-37]

来源后段则声明 acceptance index 已追加 entry `0003-a1-torus-root-coroot-ladder-boundary`，字段为 `acceptance: accepted`、`status: math_pass`，并给出下表中的 SHA-256 绑定信息。^[root-ladder-overflow-repair.md:115-123]

| 证据对象 | SHA-256 |
| --- | --- |
| acceptance-index entry | `1628ee21c71a91376a02982c38404cee958cb6183c57f791a42c4a668a1efc12` |
| AFTER-v3 报告 | `771fc790dd4340f408235e50c3f6eee754ebe4c850cbad36d4af4f902a24627c` |
| 独立 review | `3c0eed61cc5bf6af809096da73ac4bf1d8217b5cc81954a2e8000e5da44f9b58` |
| canonical 源文件清单 | `d2a6367379432c1ed09b90c903a072cfce0fb463349032100dd644db7c3fc213` |

该 entry 的断言包括：原版接受全部 11 个 case 的 22 条记录，两条 root/coroot kernel 回归及两个 crate 完整套件通过，Rust 完整流与捕获的原版一致，以及三条 tests-first 回归在修复前确实失败。^[root-ladder-overflow-repair.md:124-126]

“不含登记的限定接受”与“已完成登记”可能描述不同阶段或不同层次，但来源没有明确交代两者的衔接。本页因此保留口径歧义：**AFTER-v3 的限定接受已有记录；正式登记在来源后段被声明完成，但这一转述不等于对账本当前状态的独立核实。** 前段排除登记的措辞也不足以推出“确定未登记”。

## 接受范围与使用限制

前段限定接受覆盖两条 root/coroot 坐标边界 kernel 回归、对应的 original-backed 解释器回归、两个 crate 的完整 Rust 测试套件及目录治理 harness。后段所述账本条目仍将数学断言限定于 A1+中心环面坐标边界 fixture；完整 Rust 套件只是回归守卫，并非一般正确性的 oracle 证明。^[root-ladder-overflow-repair.md:30-34, root-ladder-overflow-repair.md:124-130]

这些证据不证明一般 root system、KGB、KLV、unitarity、Hodge、associated cycle 或 AV-ann 正确性，也不支持更高 rank、性能、内存或并行验收结论。来源要求 AFTER execution/report 不能自我验收，独立 review 也不能绕过正式账本；相关原则参见 [[HPC 验收证据链]]。^[root-ladder-overflow-repair.md:109-113, root-ladder-overflow-repair.md:127-130, root-ladder-overflow-repair.md:141-143]

修复本身不是性能优化：此前因溢出早退的极值输入现在会完成 \(O(|R|^2)\) 表构造，可能使用更多时间。性能或内存结论仍需受控测量，不能由回归通过或源码哈希绑定推出。^[root-ladder-overflow-repair.md:85-88]

## Sources

- [Root ladder 固定宽度坐标溢出修复](../../sources/root-ladder-overflow-repair.md)

---
title: Root ladder 修复的限定接受与账本状态歧义
summary: 来源确认 AFTER-v3 在坐标边界范围内通过独立检查，但前文称接受范围不含 acceptance-index 登记，后文又称条目 0003-a1-torus-root-coroot-ladder-boundary 已登记为 accepted/math_pass；账本状态叙述需核对，且均不支持一般正确性或性能结论。
sources:
  - root-ladder-overflow-repair.md
kind: concept
createdAt: "2026-10-09T15:11:23.576Z"
updatedAt: "2026-10-09T15:11:23.576Z"
tags:
  - 验收状态
  - 证据审查
  - 来源歧义
aliases:
  - root-ladder-修复的限定接受与账本状态歧义
confidence: 0.98
provenanceState: ambiguous
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# Root ladder 修复的限定接受与账本状态歧义

Root ladder 固定宽度坐标溢出修复的 AFTER-v3 已获得限定范围的接受，但来源包对 acceptance-index 登记状态保留了不一致的表述：开头明确排除登记，后文却记载正式 entry 已追加。阅读时应区分修复的验证范围、独立检查结果与账本登记状态，不能把它们合并为一般数学正确性结论。^[root-ladder-overflow-repair.md:9-11, root-ladder-overflow-repair.md:19-34, root-ladder-overflow-repair.md:115-130]

## 修复对象与接受范围

对完整存储的有限根集 \(R\subseteq\mathbb Z^d\)，ladder bottom 集定义为 \(B_\alpha=\{\beta\in R\mid\beta-\alpha\notin R\}\)。当已存坐标均为 `i32` 时，若精确整数差的某个坐标超出 `i32` 范围，该差必不属于已存集合。修复据此将两处成员查询中的 `StructureError::ArithmeticOverflow` 解释为“不属于”，同时继续传播分配失败和其他错误，并独立执行 root 与 coroot 查询。相关概念见 [[Root ladder bottom 集与固定宽度成员查询]] 与 [[Rust ladder 成员查询的选择性溢出处理]]。^[root-ladder-overflow-repair.md:41-53, root-ladder-overflow-repair.md:74-83]

AFTER-v3（job `3875239`）最终状态为 `COMPLETED 0:0`，独立检查记录的状态为 `LADDER_BOUNDARY_AFTER_ACCEPTED`。来源记载 95 项 harness checker、完整 stager 清点 70/67、domain 521/2/521 与 core 630/1/630 全部通过，并通过源码与最终完整性复核。接受绑定的是候选快照中的特定 `root_system.rs` 字节，而不是任意后续版本。^[root-ladder-overflow-repair.md:19-28]

该接受范围限于两条 root/coroot 坐标边界 kernel 回归、对应的 original-backed 解释器回归、两个 crate 的完整 Rust 测试套件和目录治理 harness；不涵盖一般 root system、KGB、KLV、unitarity、Hodge、associated cycle 或 AV-ann 正确性，也不涵盖更高 rank、性能或内存结论。^[root-ladder-overflow-repair.md:30-34]

## tests-first 与历史 oracle 证据

BEFORE-v3（job `3873400`）在未修复生产代码上确认了预期的两条 domain 失败和一条 core full-stream 失败，且 `0 ignored`；其作用是证明回归测试能捕获原问题，不能作为修复后通过的证据。这一证据与 AFTER-v3 构成 [[坐标边界修复的 tests-first 验证链]]。^[root-ladder-overflow-repair.md:100-107, root-ladder-overflow-repair.md:19-28]

解释器 fixture 包含 11 个 A1+torus case，覆盖 root/coroot 交换、两种 numbering、阈值两侧和 `i32::MAX`，并保留 recovery marker `719`。原版 oracle 来自历史 capture job `3868832`；AFTER 作业没有重新运行原版，而是比较 Rust 完整流与此前捕获的原版输出。相关背景见 [[A1 加中心环面边界 fixture 与历史 oracle]]。^[root-ladder-overflow-repair.md:92-98, root-ladder-overflow-repair.md:124-130]

## 账本状态歧义

来源包开头和“2026-10-03 状态推进”节均将 acceptance-index 登记排除在接受范围之外；后文却明确声称 acceptance index 已追加 entry `0003-a1-torus-root-coroot-ladder-boundary`，其字段为 `acceptance: accepted`、`status: math_pass`。来源还说明旧有“候选”“尚未”措辞仅在状态推进节的范围内被取代，但这不能直接解释上述两处登记表述为何不同。^[root-ladder-overflow-repair.md:9-11, root-ladder-overflow-repair.md:30-37, root-ladder-overflow-repair.md:115-118]

后文的登记声明提供了 entry、AFTER-v3 报告、独立 review 和 canonical 源文件清单的 SHA-256，可用于定位其声称的证据链。其中 entry 哈希为 `1628ee21c71a91376a02982c38404cee958cb6183c57f791a42c4a668a1efc12`。这些信息支持准确转述“来源后文记载已登记”，但来源包自身仍保留前述相反措辞。^[root-ladder-overflow-repair.md:115-123, root-ladder-overflow-repair.md:30-34]

因此，本页将状态表述为：**AFTER-v3 的限定接受已有明确记录；正式账本登记在来源后文被声明完成，但包内状态描述尚未统一。** 仅凭本包不宜把歧义消解为“确定未登记”，也不宜声称已独立核实账本当前状态。

## 证据使用边界

即使采用后文的已登记声明，其 entry 仍只覆盖 A1+中心环面坐标边界 fixture。完整 Rust 测试套件是回归守卫，不是一般正确性的 oracle 证明；该 entry 也明确排除更高 rank、性能和并行验收。来源同时要求 AFTER execution/report 不得自我验收，独立 review 不能绕过正式账本。这些约束属于 [[HPC 验收证据链]] 的必要边界。^[root-ladder-overflow-repair.md:109-113, root-ladder-overflow-repair.md:124-130]

## Sources

- [Root ladder 固定宽度坐标溢出修复](../../sources/root-ladder-overflow-repair.md)

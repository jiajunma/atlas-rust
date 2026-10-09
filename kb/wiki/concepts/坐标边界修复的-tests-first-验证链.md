---
title: 坐标边界修复的 tests-first 验证链
summary: BEFORE-v3 确认未修复代码的两条 kernel 回归及一条完整流回归失败，AFTER-v3 验证修复后通过，并结合完整套件、保留门禁与源码完整性检查限定证据范围。
sources:
  - root-ladder-overflow-repair.md
kind: concept
createdAt: "2026-10-09T15:11:21.755Z"
updatedAt: "2026-10-09T21:10:03.620Z"
tags:
  - 回归测试
  - HPC
  - 验证流程
aliases:
  - 坐标边界修复的-tests-first-验证链
  - 坐T验
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: 坐标边界修复的 tests-first 验证链
summary: BEFORE-v3 确认三条回归在未修复源码上按预期失败，AFTER-v3 验证修复、完整测试套件及历史原版输出一致性；独立检查与账本记录共同限定证据范围。
sources:
  - root-ladder-overflow-repair.md
kind: concept
tags:
  - 回归测试
  - HPC验证
  - 证据链
aliases:
  - 坐标边界修复的-tests-first-验证链
---

# 坐标边界修复的 tests-first 验证链

坐标边界修复的 tests-first 验证链针对 root/coroot ladder bottom 构造中的 `i32` 坐标差溢出：先证明新增回归在未修复源码上按预期失败，再验证修复后的定向回归、完整 Rust 测试套件和历史原版输出一致性。执行报告、独立 review 与正式账本承担不同职责，不能互相替代。^[root-ladder-overflow-repair.md:13-15, root-ladder-overflow-repair.md:100-130]

## 修复对象与数学依据

对完整存储的有限根集 \(R\subseteq\mathbb Z^d\) 和 \(\alpha\in R\)，ladder bottom 集为 \(B_\alpha=\{\beta\in R\mid\beta-\alpha\notin R\}\)。若所有已存坐标均为 `i32`，而精确整数差的某个坐标超出其表示范围，该差就不可能属于已存集合。因此，这次成员查询应返回 `false`，使相应的 \(\beta\) 进入 bottom 集，而不应导致整个 `RootSystem` 构造失败。参见 [[Root ladder bottom 集与固定宽度成员查询]]。^[root-ladder-overflow-repair.md:13-15, root-ladder-overflow-repair.md:41-49]

生产修复仅调整 `build_ladder_bottoms` 的两次成员查询：减法成功时沿用原有查找；恰为 `StructureError::ArithmeticOverflow` 时解释为非成员；分配失败及其他错误继续传播。root 与 coroot 查询独立执行，前者溢出不会跳过后者。该规则不适用于一般向量减法、反射、root combination、seed negation 或输入验证，也不允许 wrapping/saturating 算术。详见 [[Rust ladder 成员查询的选择性溢出处理]]。^[root-ladder-overflow-repair.md:51-53, root-ladder-overflow-repair.md:74-83]

## 历史 oracle 与测试增量

fixture `tests/math/generics/root_ladder_coordinate_boundary.atlas` 包含 11 个 A1+torus case，覆盖 root/coroot 交换、两种编号、阈值两侧及 `i32::MAX`，并保留 recovery marker `719`。原版完整 stdout 的 SHA-256 为 `3a7fdade43c46cf4f3048b52cf81f282db060012951ee559296f017f7d3eab80`。^[root-ladder-overflow-repair.md:92-95]

oracle 来自历史 original-backed capture job `3868832`；其记录 `tests/reference/hpc/math_weyl_context_capture_2026_09_30.json` 绑定 case、oracle binary/source 和 raw stream。该历史捕获不是候选 AFTER 执行，AFTER 作业也没有重新运行原版 oracle。参见 [[A1 加中心环面边界 fixture 与历史 oracle]]。^[root-ladder-overflow-repair.md:96-98, root-ladder-overflow-repair.md:124-130]

tests-first 的测试代码增量、session 回归和 fixture 构成独立于生产修复的固定增量，由 BEFORE 证据单独约束。这样，修复前的失败与修复后的通过分别具有对应记录。^[root-ladder-overflow-repair.md:19-28, root-ladder-overflow-repair.md:74-76]

## BEFORE-v3：确认回归检出原错误

BEFORE-v3 job `3873400` 的独立检查确认：65 个 harness checker 通过，测试 inventory 为 atlas-real-group `521` 项、atlas-core `630` 项；未修复生产代码精确出现 2 个 domain 失败和 1 个 core full-stream 失败，`0 ignored`。这些预期失败证明三条回归能够检出原错误，属于 tests-first 证据，不能记作 AFTER 通过。^[root-ladder-overflow-repair.md:100-107]

对应记录为 `tests/reference/hpc/math_ladder_boundary_before_v3_2026_10_01.json`，SHA-256 为 `608e996acac10ea1bacca177468fcd83f3ec39c3e579899440e11aab674fc37f`。^[root-ladder-overflow-repair.md:100-103]

## AFTER-v3：验证修复与保留门禁

AFTER 的送审条件包括三条定向回归全部通过、两个 crate 的完整 `521/630` 测试实际全部通过、Rust 与历史原版完整流相等、相关溢出负例和 retained gates 保留，以及源码、补丁和报告哈希闭合。AFTER execution/report 不能自我验收，独立 review 也不能绕过正式账本。^[root-ladder-overflow-repair.md:109-113]

AFTER-v3 job `3875239` 最终状态为 `COMPLETED 0:0`。独立检查记录 `tests/reference/hpc/math_ladder_boundary_after_v3_2026_10_01.json` 的状态为 `LADDER_BOUNDARY_AFTER_ACCEPTED`，确认 95 项 harness checker、完整 stager 清点 `70/67`、domain `521/2/521` 与 core `630/1/630` 全部通过，并完成源码及最终完整性复核。该记录的 SHA-256 为 `a459fa08117ff8a721181d349467ebdd9267e798cd25b5bcb1380996c2d15e15`。^[root-ladder-overflow-repair.md:19-28]

验证绑定的候选 Git base 为 `eba9c7ea080e61de9d4b105fbf54589c44a10b87`，接受的 `crates/atlas-real-group/src/root_system.rs` 字节 SHA-256 为 `cc6a1764e1c2425f7de8b4c27ca34a8bdc855c764d6db2a6ab9d6092e7b8cfe9`。源码版本绑定限定了这些结果所针对的具体实现。^[root-ladder-overflow-repair.md:27-28, root-ladder-overflow-repair.md:70-72]

## 账本记录与状态歧义

来源后段记载 acceptance-index 已追加 entry `0003-a1-torus-root-coroot-ladder-boundary`，状态为 `acceptance: accepted`、`status: math_pass`。该条目绑定 AFTER-v3 报告、独立 review 与 canonical 源文件清单；来源给出的对应标识如下。^[root-ladder-overflow-repair.md:115-123]

| 对象 | SHA-256 |
| --- | --- |
| acceptance-index entry | `1628ee21c71a91376a02982c38404cee958cb6183c57f791a42c4a668a1efc12` |
| AFTER-v3 报告 | `771fc790dd4340f408235e50c3f6eee754ebe4c850cbad36d4af4f902a24627c` |
| 独立 review | `3c0eed61cc5bf6af809096da73ac4bf1d8217b5cc81954a2e8000e5da44f9b58` |
| canonical 源文件清单 | `d2a6367379432c1ed09b90c903a072cfce0fb463349032100dd644db7c3fc213` |

来源前部仍将限定接受描述为“不含 acceptance-index 登记”，与后段已登记的记载存在状态口径差异。两种表述应同时保留，不能将来源视为无歧义的同步状态说明；参见 [[Root ladder 修复的限定接受与账本状态歧义]]。^[root-ladder-overflow-repair.md:9-11, root-ladder-overflow-repair.md:30-37, root-ladder-overflow-repair.md:115-123]

## 证据适用范围

账本叙述覆盖原版接受全部 11 个 case 的 22 条记录、root/coroot 两条 kernel 回归通过、两个 crate 完整套件通过、Rust 完整流与历史原版捕获一致，以及三条 tests-first 回归在修复前确实失败。接受范围仅限 A1+中心环面坐标边界 fixture；完整 Rust 套件提供回归守卫，而非 oracle 证明。^[root-ladder-overflow-repair.md:124-130]

这些证据不证明一般 root system 或 KGB 正确性，不覆盖更高 rank、KLV、unitarity、Hodge、associated cycle、AV-ann，也不构成性能、内存或并行验收。修复后，过去提前失败的极值输入会完成 \(O(|R|^2)\) 表构造，可能耗费更多时间；性能或内存结论仍需受控测量。^[root-ladder-overflow-repair.md:85-88, root-ladder-overflow-repair.md:127-143]

## Sources

- [Root ladder 固定宽度坐标溢出修复](../../sources/root-ladder-overflow-repair.md)

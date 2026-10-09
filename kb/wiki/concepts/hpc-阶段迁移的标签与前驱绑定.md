---
title: HPC 阶段迁移的标签与前驱绑定
summary: 阶段版本迁移必须同步 sbatch 标签并审计历史 validator 的前驱绑定；失败阶段没有 report 时应省略相关字段，保留真实证据链。
sources:
  - weyl-context-identity-and-sharing.md
kind: concept
createdAt: "2026-10-09T21:13:50.470Z"
updatedAt: "2026-10-09T21:13:50.470Z"
tags:
  - HPC
  - validation
  - provenance
aliases:
  - hpc-阶段迁移的标签与前驱绑定
confidence: 1
provenanceState: merged
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

# HPC 阶段迁移的标签与前驱绑定

HPC 阶段迁移不仅涉及源码和检查器，还必须同步迁移 sbatch 标签、输出路径约定及前驱证据绑定。Weyl context 修复的 AFTER 阶段表明，这些约束若不一致，作业可能在任何数学检查之前失败；此类 harness 失败不能作为数学回归或修复有效性的证据。^[weyl-context-identity-and-sharing.md:22-43]

## 阶段标签的一致性

AFTER-v2 job `3890580` 在拓扑验证阶段失败：stager 已更新 `SLURM_OUTPUT_PATTERN`，但 sbatch 的 `--job-name` 与 `--output` 仍使用 v1 标签。实际生成的 `weyl-context-core-after-v1-3890580.out` 被 `validate_stage_topology` 判为意外持久文件。该作业尚未进入任何 gate，也没有生成 `report.json`；失败被分类为 `HARNESS_SBATCH_LABEL_MISMATCH_AT_TOPOLOGY_VALIDATION_BEFORE_ANY_GATE`。^[weyl-context-identity-and-sharing.md:28-34]

after-v3 同步迁移了 sbatch 标签，并新增检查器，将两个标签钉到由 `STAGE_NAME` 派生的后缀。这使阶段名称与调度输出约定成为可检查的不变量，相关 checker 分组由 29 项增至 30 项，总数由 124 项增至 125 项。^[weyl-context-identity-and-sharing.md:35-43]

## 不可变前驱与历史验证器

后继阶段必须绑定实际存在的前驱证据。after-v3 将失败的 after-v2 作为不可变前驱；由于 after-v2 没有报告，其 `PREDECESSOR` 直接省略 `report_sha256` 与 `report_bytes`，不构造虚假的报告哈希。同时，新增的 `validate_after_v2_failure` 被接入两条运行路径和 driver gates。^[weyl-context-identity-and-sharing.md:35-39]

历史验证器若引用可变的裸 `PREDECESSOR`，可能在阶段推进后错误地读取新前驱。after-v3 通过 `AFTER_V1_PREDECESSOR` 做本地重绑定，使 `validate_after_v1_failure` 的函数体保持逐字节不变。来源据此要求：启动前审计所有引用裸 `PREDECESSOR` 的历史验证器，并完成对应的本地重绑定。^[weyl-context-identity-and-sharing.md:38-39, weyl-context-identity-and-sharing.md:53-55]

阶段版本与证据 schema 版本也不应混同。此次迁移仍使用 `atlas-stage-creation-predecessor-v12`，该值由 `progressive_submit.STAGE_CREATION_PREDECESSOR_SCHEMA` 钉住，不随 stage 编号递增；`progressive_submit` 另行补充了已退役 after-v1 的 scoped campaign 文件。^[weyl-context-identity-and-sharing.md:41-43]

## 源码身份与冻结预测

AFTER-v1 job `3890328` 暴露了另一种绑定错误：driver 将最终修复 manifest 的摘要与 tests-only 的 `REGRESSION_SOURCE` 常量比较。HPC 实际重建结果逐字节匹配 `AFTER_SOURCE_MANIFEST_SHA256`，但错误断言使作业在运行任何 checker 或 Atlas 命令之前失败。修正进入 v2，原失败证据及完整报告保持冻结。^[weyl-context-identity-and-sharing.md:22-27]

较早的 AFTER-v1 准备工作还修正了缺失的 scoped campaign 文件、未随前驱链增长更新的 ledger 长度断言，以及 `validate_before_v3_failure` 对已推进的裸 `PREDECESSOR` 的引用。历史证据中的 `expected_test_counts_after` 则改用历史字面量验证，避免以当前可变常量解释旧预测。这与 [[源码预测与原版捕获的证据分离]] 一致：后继阶段应保留历史记录的原有含义。^[weyl-context-identity-and-sharing.md:126-136]

## 验证范围与后续状态

after-v3 的本地六套件共 125 项测试通过，来源另注明已知的 0444 环境证据检查在 HPC 上为绿色。60 输入 payload 经自带 stager 验证后，停在 HPC-only 的 `validate_parent_objects` 哨兵。由于 SecureLink 隧道中断，当时没有创建远端 stage、intent 或 job，因此这些准备结果不能视为该阶段已经完成 HPC 执行。^[weyl-context-identity-and-sharing.md:44-51]

后续 AFTER-v5 job `3900050` 最终以 `COMPLETED 0:0` 完成，验收记录为 `math_weyl_context_core_after_v5_acceptance_2026_10_06.json`，报告 SHA 前缀为 `3288480d…`。来源记载 cold dual 完全字节相等，prewarmed dual 的 stdout、退出码及有序错误摘要一致；接受范围仍限于 A1 语义，不授予缓存、性能、内存、更高 rank 或更广数学 release。标签与前驱绑定保障证据链的正确衔接，但不扩大其结论范围。^[weyl-context-identity-and-sharing.md:63-69]

## Sources

- [weyl-context-identity-and-sharing.md](../../sources/weyl-context-identity-and-sharing.md) — Weyl 对象身份、dual 历史与安全共享边界。

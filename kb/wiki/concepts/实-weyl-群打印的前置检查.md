---
title: 实 Weyl 群打印的前置检查
summary: print_real_Weyl 在分发分支内按上游包装器的措辞与顺序执行检查，防止外来外部形式号被 ExternalFormOrder 静默翻译。
sources:
  - atlas-core-domain-validate-print.md
kind: concept
createdAt: "2026-10-09T14:30:11.117Z"
updatedAt: "2026-10-09T14:30:11.117Z"
tags:
  - Weyl群
  - 实形
  - 参数校验
aliases:
  - 实-weyl-群打印的前置检查
  - 实W群
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 实 Weyl 群打印的前置检查

`print_real_Weyl` 的前置检查位于 `print_text` 按名称分发的对应分支内。检查须按上游包装器的措辞与顺序先执行；否则，外来的外部形式号会经 `ExternalFormOrder` 静默翻译。^[atlas-core-domain-validate-print.md:58-70]

## 检查位置与顺序

这里的兼容契约同时涉及检查的位置、诊断措辞和执行顺序。`print_real_Weyl` 在自身分支内先完成检查，以防外部形式号进入静默翻译路径。相关编号背景可参见 [[弱实形式的外部编号与严格排序]]。^[atlas-core-domain-validate-print.md:69-70]

不能将所有打印入口视为具有相同的检查规则：同一分发面中的 `print_X` 在上游没有检查，而 `print_blockstabilizer` 也在上游没有检查，块仅提供它的两个实形。各入口应保留各自的契约。^[atlas-core-domain-validate-print.md:68-72]

## 证据边界

本来源属于结构性阅读，不声称数学验收；它说明了 `print_real_Weyl` 检查的放置要求及省略检查的后果，但未列出具体检查条件、完整诊断文本或逐项顺序。上游引用属于实现方的移植陈述，打印与校验的兼容性仍以 HPC 语料门为准，参见 [[HPC 验收证据链]]。^[atlas-core-domain-validate-print.md:9-12, atlas-core-domain-validate-print.md:69-70, atlas-core-domain-validate-print.md:80-81]

## Sources

- [校验与打印（domain_builtins.rs 9730–12263）](atlas-core-domain-validate-print.md)

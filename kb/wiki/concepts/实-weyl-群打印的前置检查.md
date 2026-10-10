---
title: 实 Weyl 群打印的前置检查
summary: print_real_Weyl 在分发臂内按上游包装器措辞与顺序执行检查，防止外来外部形式号被 ExternalFormOrder 静默翻译。
sources:
  - atlas-core-domain-validate-print.md
kind: concept
createdAt: "2026-10-09T14:30:11.117Z"
updatedAt: "2026-10-10T00:18:41.502Z"
tags:
  - Weyl群
  - 输入校验
  - 打印契约
aliases:
  - 实-weyl-群打印的前置检查
  - 实W群
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: 实 Weyl 群打印的前置检查
summary: print_real_Weyl 在分发分支内按上游包装器的措辞与顺序先执行检查，防止外来的外部形式号被 ExternalFormOrder 静默翻译。
sources:
  - atlas-core-domain-validate-print.md
kind: concept
tags:
  - Weyl群
  - 实形式
  - 参数校验
aliases:
  - 实-weyl-群打印的前置检查
provenanceState: extracted
---

# 实 Weyl 群打印的前置检查

`print_real_Weyl` 的前置检查位于 `print_text` 按名称分发的对应分支内。检查按上游包装器的措辞与顺序先执行，防止外来的外部形式号经 `ExternalFormOrder` 静默翻译。^[atlas-core-domain-validate-print.md:58-70]

## 检查位置与顺序

这一契约同时约束检查的位置、诊断措辞和执行顺序：检查在 `print_real_Weyl` 分支内先完成，而不能让外来的外部形式号未经检查就进入编号翻译路径。相关编号背景可参见 [[弱实形式的外部编号与严格排序]]。^[atlas-core-domain-validate-print.md:69-70]

同一打印分发面中的入口具有不同的检查契约：`print_X` 在上游没有检查，且每次调用新建 `InvolutionTable`；`print_blockstabilizer` 在上游也没有检查，块仅提供它的两个实形。因此，不能将 `print_real_Weyl` 的前置检查要求直接推广到所有打印入口。^[atlas-core-domain-validate-print.md:68-72]

## 证据边界

来源说明了检查的放置要求及省略检查的后果，但未列出 `print_real_Weyl` 的具体检查条件、完整诊断文本或逐项检查顺序。它属于结构性阅读，不声称数学验收。^[atlas-core-domain-validate-print.md:9-12, atlas-core-domain-validate-print.md:69-70]

来源中的上游引用属于实现方的移植陈述；打印与校验的兼容性仍以 HPC 语料门为准，参见 [[HPC 验收证据链]]。这些结构说明本身不构成兼容性验收结果。^[atlas-core-domain-validate-print.md:80-81]

## Sources

- [校验与打印（domain_builtins.rs 9730–12263）](../../sources/atlas-core-domain-validate-print.md)

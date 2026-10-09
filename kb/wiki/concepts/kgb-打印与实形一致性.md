---
title: KGB 打印与实形一致性
summary: print_KGB 单参打印全部元素，选择打印要求元素属于同一实形，且 print_kgb 先输出 kgbsize 行与 Base grading 头。
sources:
  - atlas-core-domain-validate-print.md
kind: concept
createdAt: "2026-10-09T14:30:07.277Z"
updatedAt: "2026-10-09T22:16:07.922Z"
tags:
  - KGB
  - 实形式
  - 打印契约
aliases:
  - kgb-打印与实形一致性
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: KGB 打印与实形一致性
summary: print_KGB 支持单参数打印全部元素，选择打印要求元素属于同一实形；print_kgb 先输出 kgbsize 行与 Base grading 头。
sources:
  - atlas-core-domain-validate-print.md
kind: concept
tags:
  - KGB
  - 实形
  - 输出格式
aliases:
  - kgb-打印与实形一致性
provenanceState: extracted
---

# KGB 打印与实形一致性

`print_KGB` 由 `print_text` 按名称分发。单参数形式打印全部 KGB 元素；选择形式由 `print_KGB_selection_wrapper` 处理，要求所列元素属于**同一实形**。^[atlas-core-domain-validate-print.md:56-63]

## 选择打印的一致性检查

选择列表包含跨实形元素时，报错文本为 `"Real form mismatch when printing KGB element"`。这一契约明确限制选择打印的元素归属；相关概念可参见 [[KGB 图与弱实形式]]。^[atlas-core-domain-validate-print.md:60-63]

## 输出顺序

底层 `print_kgb` 先打印 `kgbsize` 行与 `Base grading` 头。源材料将这一顺序对应到上游 `kgb_io.cpp:140-148`，未列出后续元素行的完整布局。^[atlas-core-domain-validate-print.md:60-63]

## 证据边界

本页依据 `crates/atlas-core/src/domain_builtins.rs` 校验与打印部分的结构性阅读，不代表数学验收。源材料中的上游行号引用属于实现方的移植陈述；打印与校验兼容性仍以 HPC 语料门为准，相关背景见 [[HPC 验收证据链]]。^[atlas-core-domain-validate-print.md:9-12, atlas-core-domain-validate-print.md:74-81]

## Sources

- [校验与打印（domain_builtins.rs 9730–12263）——46 臂 validate、块打印机与 print_text 面](../../sources/atlas-core-domain-validate-print.md)

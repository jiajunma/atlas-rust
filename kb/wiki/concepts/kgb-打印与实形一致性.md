---
title: KGB 打印与实形一致性
summary: print_KGB 单参打印全部元素，选择打印要求元素属于同一实形，print_kgb 先输出 kgbsize 行及 Base grading 头。
sources:
  - atlas-core-domain-validate-print.md
kind: concept
createdAt: "2026-10-09T14:30:07.277Z"
updatedAt: "2026-10-10T00:18:35.562Z"
tags:
  - KGB
  - 打印契约
  - 实形
aliases:
  - kgb-打印与实形一致性
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: KGB 打印与实形一致性
summary: print_KGB 单参数形式打印全部元素，选择打印要求元素属于同一实形；print_kgb 先输出 kgbsize 行与 Base grading 头。
sources:
  - atlas-core-domain-validate-print.md
kind: concept
tags:
  - KGB
  - 实形式
  - 打印契约
aliases:
  - kgb-打印与实形一致性
provenanceState: extracted
---

# KGB 打印与实形一致性

`print_KGB` 由 `print_text` 按名称分发。单参数形式打印全部 KGB 元素；选择形式通过 `print_KGB_selection_wrapper` 处理，要求所列元素属于**同一实形**。^[atlas-core-domain-validate-print.md:56-63]

## 选择打印的一致性检查

选择列表包含跨实形元素时，报错文本为 `"Real form mismatch when printing KGB element"`。这一检查约束选择打印的元素归属；相关背景可参见 [[KGB 图与弱实形式]]。^[atlas-core-domain-validate-print.md:60-63]

## 输出顺序

底层 `print_kgb` 先打印 `kgbsize` 行与 `Base grading` 头。源材料将这一顺序对应到上游 `kgb_io.cpp:140-148`，未提供后续元素行的完整布局。^[atlas-core-domain-validate-print.md:60-63]

## 证据边界

本页依据 `crates/atlas-core/src/domain_builtins.rs` 校验与打印部分的结构性阅读，不代表数学验收。源材料中的上游行号引用属于实现方的移植陈述；打印与校验兼容性以 HPC 语料门为准，参见 [[HPC 验收证据链]]。^[atlas-core-domain-validate-print.md:9-12, atlas-core-domain-validate-print.md:76-81]

## Sources

- [校验与打印（domain_builtins.rs 9730–12263）——46 臂 validate、块打印机与 print_text 面](../../sources/atlas-core-domain-validate-print.md)

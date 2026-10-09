---
title: KGB 打印与实形一致性
summary: print_KGB 支持单参打印全部元素，选择打印要求所有元素属于同一实形；print_kgb 在元素内容之前输出 kgbsize 行与 Base grading 头。
sources:
  - atlas-core-domain-validate-print.md
kind: concept
createdAt: "2026-10-09T14:30:07.277Z"
updatedAt: "2026-10-09T14:30:07.277Z"
tags:
  - KGB
  - 实形
  - 输出格式
aliases:
  - kgb-打印与实形一致性
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# KGB 打印与实形一致性

`print_KGB` 通过 `print_text` 按名称分发。单参数形式打印全部 KGB 元素；选择形式由 `print_KGB_selection_wrapper` 处理，要求所列元素属于同一实形。实形一致性是选择打印的明确约束。^[atlas-core-domain-validate-print.md:56-63]

## 选择打印的一致性检查

选择打印遇到跨实形元素时，报错文本为 `"Real form mismatch when printing KGB element"`。因此，选择列表不能混合不同实形的 KGB 元素；这一约束与 [[KGB 图与弱实形式]] 所涉及的实形归属相关。^[atlas-core-domain-validate-print.md:60-63]

## 输出顺序

底层 `print_kgb` 先输出 `kgbsize` 行与 `Base grading` 头。源材料将这一顺序对应到上游 `kgb_io.cpp:140-148`，但没有给出后续元素行的完整布局。^[atlas-core-domain-validate-print.md:60-63]

## 证据边界

本说明依据 `domain_builtins.rs` 校验与打印部分的结构性阅读，不代表数学验收。源材料中的上游行号属于实现方的移植陈述；打印与校验兼容性仍以 HPC 语料门为准，相关验收背景见 [[HPC 验收证据链]]。^[atlas-core-domain-validate-print.md:9-12, atlas-core-domain-validate-print.md:80-81]

## Sources

- [校验与打印（domain_builtins.rs 9730–12263）——46 臂 validate、块打印机与 print_text 面](atlas-core-domain-validate-print.md)

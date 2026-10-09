---
title: Weyl 词条目的有序校验
summary: check_weyl_word 先将条目转换为无符号整数以拒绝负数，再检查其小于半单秩，并保留非法条目的诊断契约。
sources:
  - atlas-core-domain-values.md
kind: concept
createdAt: "2026-10-09T20:34:24.725Z"
updatedAt: "2026-10-09T20:34:24.725Z"
tags:
  - Weyl群
  - 输入校验
  - 错误诊断
aliases:
  - weyl-词条目的有序校验
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

# Weyl 词条目的有序校验

`check_weyl_word` 对 Weyl 词条目采用有先后顺序的合法性检查：先将条目转换为 unsigned，拒绝负数；再要求条目严格小于半单秩。这一契约对应上游 `atlas-types.w:2344–2359`。^[atlas-core-domain-values.md:93-95]

## 校验顺序与边界

设半单秩为 \(r\)，合法条目 \(i\) 必须满足 \(0 \le i < r\)。负数在 unsigned 转换阶段被拒绝；通过转换的条目仍须接受秩上界检查，因此 \(i=r\) 也不合法。这里使用的是半单秩。来源给出的诊断形式为 `Illegal Weyl word entry i (should be <r)`。^[atlas-core-domain-values.md:93-95]

## 与 Weyl 身份及规范词的关系

条目的数值合法性与 Weyl 元素之间的兼容性由不同检查承担。`weyl_group_compatible` 使用抽象群的 `Arc::ptr_eq(group)` 判定兼容；`require_weyl_compatible` 在无值门之前拒绝不兼容的二元关系或乘积，并报告 `Weyl group mismatch`。相关语义见 [[Weyl 元素兼容性与跨坐标词重放]]。^[atlas-core-domain-values.md:87-95]

`WeylEltValue` 在构造时计算并冻结 canonical 既约词，之后 `Display` 与 `word` 只读取该词。上下文携带的内部生成元重编号固定上游 canonical-word 的选择，参见 [[Weyl 元素的规范词]]。^[atlas-core-domain-values.md:53-58, atlas-core-domain-values.md:92-92]

## 证据范围

来源仅明确了单个条目“先 unsigned 转换、再检查半单秩上界”的顺序，没有给出多个非法条目同时出现时的遍历或报错优先级。本包属于结构性阅读，派发表与测试不在覆盖范围内；其引用的 Weyl owner/dual 语义验收限定于 A1，不能据此宣称一般根系下的全面验收。^[atlas-core-domain-values.md:9-17, atlas-core-domain-values.md:93-95, atlas-core-domain-values.md:119-126]

## Sources

- [atlas-core-domain-values.md](../../sources/atlas-core-domain-values.md) — 领域值与 Weyl 身份（domain_builtins.rs 上部）。

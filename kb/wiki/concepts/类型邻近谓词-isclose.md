---
title: 类型邻近谓词 is_close
summary: is_close 用三比特表示双向可转换性与邻近性，优先处理相等关系，再处理 void、*、递归名、原始类型查表以及行和元组的分量规则。
sources:
  - atlas-core-support-layer.md
kind: concept
createdAt: "2026-10-09T14:35:48.242Z"
updatedAt: "2026-10-09T14:35:48.242Z"
tags:
  - 类型关系
  - 邻近谓词
  - 递归类型
aliases:
  - 类型邻近谓词-isclose
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 类型邻近谓词 is_close

`is_close(x, y) -> u8` 是 `coercions.rs` 中的类型邻近谓词，以三比特同时表示两个方向的可转换性及类型邻近性。源材料将其对应到上游 `axis-types.w:3246-3285`。^[atlas-core-support-layer.md:58-61]

## 返回值

返回值的 `0x1` 位表示左侧类型可转换为右侧类型，`0x2` 位表示右侧类型可转换为左侧类型，`0x4` 位表示两者邻近。两类型相等时返回 `0x7`，即三个标志均置位；这一相等处理也涵盖 `void` 与递归身份。^[atlas-core-support-layer.md:58-61]

## 判定顺序与结构规则

相等判定先于边界检查与类型展开。`void` 和 `*` 只与自身邻近；在未由相等判定返回的情况下，递归名的 `Tabled`／`Applied` 情形返回 `0`，否则展开一层后继续检查。该顺序与递归类型的身份处理相关，可参见 [[Type 类型表示与语义等值]] 和 [[Atlas 类型模型与递归类型图]]。^[atlas-core-support-layer.md:58-61]

原始类型端点通过转换表检查；行类型逐分量处理；元组类型对各分量的判定结果取按位 AND。因此，元组最终保留的每个标志都必须得到所有分量的支持。^[atlas-core-support-layer.md:58-61]

## 与转换表及其他谓词的关系

强转表包含 29 条注册，通过 `OnceLock` 单次构建，并保留上游注册顺序。`coercion_between` 与 `row_coercion` 均采用首中即返的线性扫描；这一顺序会影响 [[列表显示的行转换选择]]。转换节点由类型化管线带入，本模块负责回答转换适用性。^[atlas-core-support-layer.md:42-57, atlas-core-support-layer.md:65-66]

同模块的 `broader_eq(a, b)` 用于平衡序判定：`void` 最宽、`*` 最窄，原始类型吸收所有可转入者，行与元组逐分量处理，函数则要求参数类型相等。这与 `is_close` 用三比特分别表达方向性转换和邻近性的接口有所区别。^[atlas-core-support-layer.md:58-64]

## 证据边界

本源包属于结构性阅读记录，不声明语言或数学验收。测试引用了零参重载原地替换和递归名名义身份的原版背书证据，但上游行号对应仍属于实现方的移植陈述；行为兼容性以 HPC 语料门为准，可参见 [[原版背书的语言层回归测试]] 与 [[HPC 验收证据链]]。^[atlas-core-support-layer.md:9-12, atlas-core-support-layer.md:65-66, atlas-core-support-layer.md:70-74]

## Sources

- [atlas-core-support-layer.md](atlas-core-support-layer.md)

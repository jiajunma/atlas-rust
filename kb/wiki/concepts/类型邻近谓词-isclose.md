---
title: 类型邻近谓词 is_close
summary: is_close 用三比特编码双向可转换性与邻近性，先判等，再处理边界类型、递归名义身份、单层展开及容器分量。
sources:
  - atlas-core-support-layer.md
kind: concept
createdAt: "2026-10-09T14:35:48.242Z"
updatedAt: "2026-10-09T22:20:28.029Z"
tags:
  - 类型系统
  - 重载
aliases:
  - 类型邻近谓词-isclose
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: 类型邻近谓词 is_close
summary: is_close 用三比特表达双向可转换性与邻近性，优先判等，再处理边界类型、递归名、原始类型及复合类型。
sources:
  - atlas-core-support-layer.md
kind: concept
tags:
  - 类型系统
  - 类型关系
  - 递归类型
aliases:
  - 类型邻近谓词-isclose
---

# 类型邻近谓词 is_close

`is_close(x, y) -> u8` 是 `coercions.rs` 中的类型邻近谓词，用三比特同时表达两个方向的可转换性与邻近性。源材料将其对应到上游 `axis-types.w:3246-3285`。^[atlas-core-support-layer.md:58-61]

## 返回值

返回值中的 `0x1` 表示左侧类型可转换为右侧类型，`0x2` 表示右侧类型可转换为左侧类型，`0x4` 表示两者邻近。类型相等时返回 `0x7`，即三个标志均置位；相等判定包含 `void` 与递归身份，相关概念见 [[Type 类型表示与语义等值]]。^[atlas-core-support-layer.md:58-61]

## 判定顺序与结构规则

**相等判定先于边界检查与类型展开。** `void` 与 `*` 只与自身邻近。相等判定未返回时，递归名的 `Tabled`／`Applied` 情形返回 `0`，否则展开一层再检查。这一顺序保留了递归身份相等时返回 `0x7` 的规则。^[atlas-core-support-layer.md:58-61]

原始类型端点通过转换表检查；行类型逐分量处理；元组类型将各分量的返回值按位 AND。因此，元组结果中的每个标志都要求各分量共同支持。^[atlas-core-support-layer.md:58-61]

## 与转换表及宽度关系的联系

转换表包含 29 条注册，由 `OnceLock` 单次构建，并保持上游注册顺序。`coercion_between` 与 `row_coercion` 均采用首中即返的线性扫描；这一顺序使 `mat` 上下文的列表显示先选择 `[vec]` 到 `mat` 的转换，详见 [[列表显示的行转换选择]]。^[atlas-core-support-layer.md:42-57]

同模块的 `broader_eq(a, b)` 表达类型的平衡序：`void` 最宽、`*` 最窄，原始类型吸收所有可转入者，行与元组逐分量处理，函数要求参数类型相等。`is_close` 则分别报告双向可转换性与邻近性；尤其不能将 `broader_eq` 中 `void` 最宽的规则套用到 `is_close` 的邻近判定。^[atlas-core-support-layer.md:58-64]

本模块只回答转换适用性；转换节点随类型化管线到达。^[atlas-core-support-layer.md:65-66]

## 证据边界

来源属于结构性阅读记录，不声称语言或数学验收。模块测试引用了原版背书证据，包括 `3856293` 的零参重载原地替换与 `3856744` 的递归名名义身份，可结合 [[原版背书的语言层回归测试]] 理解。^[atlas-core-support-layer.md:9-12, atlas-core-support-layer.md:65-66]

源材料中的上游行号属于实现方的移植陈述，行为兼容仍以 HPC 语料门为准。字节数与哈希仅标识该快照的字节，所列 Git base 为 `964f0033`。^[atlas-core-support-layer.md:70-74]

## Sources

- [atlas-core-support-layer.md](../../sources/atlas-core-support-layer.md) — 支撑层：结构化诊断、源位置与强转表／邻近谓词。

---
title: 类型邻近谓词 is_close
summary: is_close 用三比特编码双向可转换性与邻近性，优先判等，再按边界类型、递归身份、原始类型、行及元组规则计算。
sources:
  - atlas-core-support-layer.md
kind: concept
createdAt: "2026-10-09T14:35:48.242Z"
updatedAt: "2026-10-09T20:44:05.393Z"
tags:
  - 类型系统
  - 类型关系
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
summary: is_close 用三比特表示双向可转换性与邻近性，先判相等，再处理边界类型、递归名、原始类型及复合类型。
sources:
  - atlas-core-support-layer.md
kind: concept
tags:
  - 类型关系
  - 邻近谓词
  - 递归类型
aliases:
  - 类型邻近谓词-isclose
---

# 类型邻近谓词 is_close

`is_close(x, y) -> u8` 是 `coercions.rs` 中的类型邻近谓词，以三比特同时表达两个方向的可转换性及邻近性。源材料将其对应到上游 `axis-types.w:3246-3285`。^[atlas-core-support-layer.md:58-61]

## 返回值

返回值中，`0x1` 表示左侧类型可转换为右侧类型，`0x2` 表示右侧类型可转换为左侧类型，`0x4` 表示两者邻近。类型相等时返回 `0x7`，即三个标志均置位；相等判定包含 `void` 与递归身份。相关等值语义见 [[Type 类型表示与语义等值]]。^[atlas-core-support-layer.md:58-61]

## 判定顺序与结构规则

相等判定先于边界检查和类型展开。`void` 与 `*` 只与自身邻近；在相等判定未返回的情况下，递归名的 `Tabled`／`Applied` 情形返回 `0`，否则展开一层后继续检查。因此，不能跳过相等判定而直接将递归名判为不邻近。^[atlas-core-support-layer.md:58-61]

原始类型端点通过转换表检查；行类型逐分量处理；元组类型将各分量的结果按位 AND。元组结果中的每个标志因而都需要各分量共同支持。^[atlas-core-support-layer.md:58-61]

## 与转换表及宽度关系的联系

[[有序强制转换注册表]] 包含 29 条注册，由 `OnceLock` 单次构建，并保留上游注册顺序。`coercion_between` 与 `row_coercion` 都采用首中即返的线性扫描；这一顺序使 `mat` 上下文的列表显示先选择 `[vec]` 到 `mat` 的转换，相关规则见 [[列表显示的行转换选择]]。^[atlas-core-support-layer.md:42-57]

同模块的 [[类型宽度关系 broader_eq|broader_eq(a, b)]] 表达平衡序：`void` 最宽、`*` 最窄，原始类型吸收所有可转入者，行与元组逐分量处理，函数要求参数类型相等。`is_close` 则通过三比特分别报告双向可转换性与邻近性。^[atlas-core-support-layer.md:58-64]

本模块负责回答转换适用性；转换节点由类型化管线带入，参见 [[类型化转换与求值管线]]。^[atlas-core-support-layer.md:65-66]

## 证据边界

来源属于结构性阅读记录，不声明语言或数学验收。测试引用了原版背书证据，包括 `3856293` 的零参重载原地替换与 `3856744` 的递归名名义身份；这些引用应结合 [[原版背书的语言层回归测试]] 理解。^[atlas-core-support-layer.md:9-12, atlas-core-support-layer.md:65-66]

源材料中的上游行号对应属于实现方的移植陈述，行为兼容仍以 HPC 语料门为准；快照字节及哈希只标识所检查的版本，所列 Git base 为 `964f0033`。^[atlas-core-support-layer.md:70-74]

## Sources

- [atlas-core-support-layer.md](../../sources/atlas-core-support-layer.md) — 支撑层：结构化诊断、源位置与强转表／邻近谓词。

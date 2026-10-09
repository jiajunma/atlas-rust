---
title: 类型宽度关系 broader_eq
summary: broader_eq 定义 void 最宽、* 最窄的平衡序，按可转换性及容器分量比较类型宽度，并要求函数参数类型相等。
sources:
  - atlas-core-support-layer.md
kind: concept
createdAt: "2026-10-09T14:35:46.143Z"
updatedAt: "2026-10-09T20:44:04.961Z"
tags:
  - 类型系统
  - 类型关系
aliases:
  - 类型宽度关系-broadereq
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: 类型宽度关系 broader_eq
summary: broader_eq 定义 void 最宽、* 最窄的平衡序，原始类型吸收可转入者，行和元组逐分量比较，函数比较要求参数类型相等。
sources:
  - atlas-core-support-layer.md
kind: concept
tags:
  - 类型关系
  - 平衡序
  - 函数类型
aliases:
  - 类型宽度关系-broadereq
provenanceState: extracted
---

# 类型宽度关系 broader_eq

`broader_eq(a, b)` 是 `coercions.rs` 中的类型宽度比较谓词，采用“平衡序”判断类型 `a` 是否至少与 `b` 一样宽。来源将其对应到上游 `axis-types.w:3339-3364`。^[atlas-core-support-layer.md:62-64]

## 比较规则

`void` 是最宽类型，`*` 是最窄类型。原始类型吸收所有可转换到它的类型，因此比较时需要保留转换方向：可转入某个原始类型的类型，由该原始类型吸收。^[atlas-core-support-layer.md:62-64]

行与元组按分量比较。函数类型的比较要求**参数类型相等**，参数之间存在转换关系不能替代这一条件；相关等值语义可参见 [[Type 类型表示与语义等值]]。^[atlas-core-support-layer.md:62-64]

## 与邻近关系的区别

同一模块中的 [[类型邻近谓词 is_close|is_close(x, y)]] 返回三比特结果：`0x1` 表示左侧可转到右侧，`0x2` 表示右侧可转到左侧，`0x4` 表示二者邻近。相等类型返回 `0x7`，且相等判断先于边界检查与展开。^[atlas-core-support-layer.md:58-61]

两种关系对边界类型的处理不同：`is_close` 规定 `void` 与 `*` 只与自身邻近，`broader_eq` 则将它们分别置于最宽和最窄的位置。因此，不能用邻近关系的边界规则解释类型宽度关系。^[atlas-core-support-layer.md:58-64]

## 模块职责与证据边界

`coercions.rs` 同时提供 [[有序强制转换注册表]] 与类型关系的适用性判断；转换节点由 [[类型化转换与求值管线]] 承接，本模块只回答适用性。^[atlas-core-support-layer.md:40-66]

本页依据支撑层源码包的结构性阅读，不代表语言或数学验收。上游行号对应关系属于实现方的移植陈述，行为兼容仍以 HPC 语料门为准。^[atlas-core-support-layer.md:9-12, atlas-core-support-layer.md:70-74]

## Sources

- [atlas-core-support-layer.md](../../sources/atlas-core-support-layer.md) — 支撑层：结构化诊断、源位置与强转表／邻近谓词。

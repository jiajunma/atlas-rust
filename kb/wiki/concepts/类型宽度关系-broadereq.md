---
title: 类型宽度关系 broader_eq
summary: broader_eq 定义 void 最宽、* 最窄的平衡序，按可转换性及容器分量比较类型宽度，并要求函数参数类型相等。
sources:
  - atlas-core-support-layer.md
kind: concept
createdAt: "2026-10-09T14:35:46.143Z"
updatedAt: "2026-10-09T22:20:28.041Z"
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
  - 类型系统
  - 类型关系
  - 平衡序
aliases:
  - 类型宽度关系-broadereq
provenanceState: extracted
---

# 类型宽度关系 broader_eq

`broader_eq(a, b)` 是 `coercions.rs` 中的类型宽度比较谓词，采用“平衡序”比较类型；来源将其对应到上游 `axis-types.w:3339-3364`。其核心规则是 `void` 最宽、`*` 最窄，并按类型结构处理可转换性与分量关系。^[atlas-core-support-layer.md:62-64]

## 比较规则

`void` 位于最宽端，`*` 位于最窄端。原始类型吸收所有可转换到它的类型，因此转换方向是比较规则的一部分：这里关注的是哪些类型能够**转入**该原始类型。^[atlas-core-support-layer.md:62-64]

行与元组逐分量比较。函数类型的比较要求**参数类型相等**，参数之间存在转换关系不能替代这一条件；相关类型等值概念可参见 [[Type 类型表示与语义等值]]。^[atlas-core-support-layer.md:62-64]

## 与邻近关系的区别

同一模块中的 `is_close(x, y) -> u8` 用三个比特分别表达不同关系：`0x1` 表示左侧可转到右侧，`0x2` 表示右侧可转到左侧，`0x4` 表示二者邻近。相等类型返回 `0x7`，且相等判断先于边界检查与递归类型展开。^[atlas-core-support-layer.md:58-61]

两者对边界类型的处理不同：`is_close` 中，`void` 与 `*` 只与自身邻近；`broader_eq` 则将它们分别置于最宽和最窄的位置。因此，邻近关系的边界规则不能直接用于解释类型宽度关系。^[atlas-core-support-layer.md:58-64]

## 模块职责与证据边界

`coercions.rs` 提供有序强转表以及相关类型关系谓词，只回答转换的适用性；转换节点随类型化管线到达。强转表保持上游注册顺序，`coercion_between` 与 `row_coercion` 都采用首中即返的线性扫描。^[atlas-core-support-layer.md:40-45, atlas-core-support-layer.md:65-66]

本页依据支撑层源码包的结构性阅读，不代表语言或数学验收。上游行号对应关系属于实现方的移植陈述，行为兼容仍以 HPC 语料门为准。^[atlas-core-support-layer.md:9-12, atlas-core-support-layer.md:70-74]

## Sources

- [atlas-core-support-layer.md](../../sources/atlas-core-support-layer.md) — 支撑层：结构化诊断、源位置与强转表／邻近谓词。

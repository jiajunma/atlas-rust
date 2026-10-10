---
title: 类型宽度关系 broader_eq
summary: broader_eq 定义 void 最宽、* 最窄的平衡序，按可转换性及容器分量比较类型宽度，并要求函数参数类型相等。
sources:
  - atlas-core-support-layer.md
kind: concept
createdAt: "2026-10-09T14:35:46.143Z"
updatedAt: "2026-10-10T00:23:20.475Z"
tags:
  - 类型系统
  - 类型宽度
aliases:
  - 类型宽度关系-broadereq
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
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

`broader_eq(a, b)` 是 `coercions.rs` 中的类型宽度比较谓词，采用“平衡序”：`void` 最宽，`*` 最窄，其余类型按可转换性和类型结构比较。来源将其对应到上游 `axis-types.w:3339-3364`。^[atlas-core-support-layer.md:62-64]

## 比较规则

原始类型吸收所有可转换到它的类型。因此，比较时关注的是哪些类型能够**转入**该原始类型，转换方向是宽度关系的一部分。^[atlas-core-support-layer.md:62-64]

行与元组逐分量比较。函数类型的比较则要求**参数类型相等**，参数之间存在转换关系不能替代这一条件；相关背景可参见 [[Type 类型表示与语义等值]]。^[atlas-core-support-layer.md:62-64]

## 与邻近关系的区别

同一模块中的 `is_close(x, y) -> u8` 用三个比特表达关系：`0x1` 表示左侧可转到右侧，`0x2` 表示右侧可转到左侧，`0x4` 表示二者邻近。类型相等时返回 `0x7`，这一判断包括 `void` 与递归身份，且先于边界检查和递归类型展开。^[atlas-core-support-layer.md:58-61]

两者对边界类型的处理不同：`is_close` 中，`void` 与 `*` 只与自身邻近；`broader_eq` 则将它们分别置于最宽和最窄的位置。邻近关系的边界规则因此不能直接用于解释类型宽度关系。^[atlas-core-support-layer.md:58-64]

## 模块职责与证据边界

`coercions.rs` 提供强转表和相关谓词，只回答转换适用性，转换节点随类型化管线到达。强转表保留上游注册顺序，`coercion_between` 与 `row_coercion` 都采用首中即返的线性扫描；列表显示中的相关选择见 [[列表显示的行转换选择]]。^[atlas-core-support-layer.md:40-45, atlas-core-support-layer.md:55-57, atlas-core-support-layer.md:65-66]

本页依据支撑层源码包的结构性阅读，不代表语言或数学验收。上游行号对应关系属于实现方的移植陈述，行为兼容仍以 HPC 语料门为准，可结合 [[HPC 验收证据链]] 理解证据范围。^[atlas-core-support-layer.md:9-12, atlas-core-support-layer.md:70-74]

## Sources

- [atlas-core-support-layer.md](../../sources/atlas-core-support-layer.md) — 支撑层：结构化诊断、源位置与强转表／邻近谓词。

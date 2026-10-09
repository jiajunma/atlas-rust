---
title: 类型宽度关系 broader_eq
summary: broader_eq 定义 void 最宽、* 最窄的平衡序，原始类型吸收可转入者，行和元组逐分量比较，函数比较要求参数类型相等。
sources:
  - atlas-core-support-layer.md
kind: concept
createdAt: "2026-10-09T14:35:46.143Z"
updatedAt: "2026-10-09T14:35:46.143Z"
tags:
  - 类型关系
  - 平衡序
  - 函数类型
aliases:
  - 类型宽度关系-broadereq
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 类型宽度关系 broader_eq

`broader_eq(a, b)` 是类型比较中的“平衡序”谓词，用于判断类型 `a` 是否至少与 `b` 一样宽。源码材料将其对应到上游 `axis-types.w:3339-3364`，并规定了边界类型、原始类型、行、元组与函数的比较规则。^[atlas-core-support-layer.md:62-64]

## 比较规则

`void` 是最宽类型，`*` 是最窄类型。原始类型吸收所有可转换到它的类型，因此转换方向是判断原始类型宽度关系的依据。^[atlas-core-support-layer.md:62-64]

行与元组按分量比较。函数类型的比较则要求参数类型相等；不能仅凭参数之间存在转换关系，就认定满足这一要求。类型表示与等值语义可参见 [[Type 类型表示与语义等值]]。^[atlas-core-support-layer.md:62-64]

## 与邻近关系及强转的区别

同一模块中的 `is_close(x, y)` 返回三比特结果，分别表示左侧可转到右侧、右侧可转到左侧，以及二者邻近。它规定 `void` 与 `*` 只与自身邻近；`broader_eq` 则将二者分别作为最宽和最窄类型。因此，邻近关系与类型宽度关系具有不同的边界规则。^[atlas-core-support-layer.md:58-64]

`broader_eq` 所在的 `coercions.rs` 提供强转表及相关适用性判断，转换节点由类型化管线承接。理解这些判断与可执行转换之间的职责边界时，可参见 [[TypedExpr 可执行表达式树]]。^[atlas-core-support-layer.md:40-66]

## 证据边界

本页依据支撑层源码包的结构性阅读说明，不代表语言或数学验收。上游行号对应关系属于实现方的移植陈述，行为兼容仍以 HPC 语料门为准，参见 [[HPC 验收证据链]]。^[atlas-core-support-layer.md:9-12, atlas-core-support-layer.md:70-74]

## Sources

- [atlas-core-support-layer.md](atlas-core-support-layer.md)

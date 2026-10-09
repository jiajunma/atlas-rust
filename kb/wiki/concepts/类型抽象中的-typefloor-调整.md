---
title: 类型抽象中的 type_floor 调整
summary: convert_expr_context 调整类型下限，使类型抽象内的 return 操作数能够引用外层函数要求，并按该要求实际的 fixed 下限解释类型变量。
sources:
  - atlas-core-convert-expr.md
kind: concept
createdAt: "2026-10-09T14:26:53.412Z"
updatedAt: "2026-10-09T14:26:53.412Z"
tags:
  - 类型系统
  - 类型抽象
  - 转换上下文
aliases:
  - 类型抽象中的-typefloor-调整
  - 类T调
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 类型抽象中的 type_floor 调整

`type_floor` 调整是 `convert_expr_context` 在类型抽象内处理 `return` 操作数时的上下文规则：操作数可以引用外层函数的类型要求，其中的类型变量按该要求**实际的 fixed 下限**解读。这一处理对应上游 `convert_expr` 的规则。^[atlas-core-convert-expr.md:23-25]

## 转换入口与上下文传递

`convert_expr(expression, required: &mut Type, analysis)` 将所需类型 `required` 装入共享的 `ConversionType`，使用当前 `analysis.type_floor`，并在转换完成后写回。因此，入口建立的类型上下文与调度器针对类型抽象内 `return` 的下限调整，是理解这一规则的两个环节。相关整体机制见 [[convert_expr 的 in/out 类型模式与单遍转换]]。^[atlas-core-convert-expr.md:21-25]

转换遍采用单遍检查与合成，in/out 类型模式只通过 `specialise` 变异；`conform_types` 依次尝试特化、强制转换，否则报告类型错误。`type_floor` 调整处于这一转换框架之中。^[atlas-core-convert-expr.md:12-17]

## 表达式族之间的衔接

`return` 属于 `atom` 转换族，`TypeAbstraction` 与 `Cast` 属于 `type_context` 转换族。调度器将已调整的共享转换上下文传递给各族助手；十二个族助手的拆分是机械分区，保留原有分支体与语义。^[atlas-core-convert-expr.md:26-38]

## 证据边界

现有材料记录了调整规则及转换入口，但属于结构性阅读，不构成语言行为验收。上游行号是实现方的移植陈述，转换行为兼容仍以 HPC 语料门为准；相关证据要求可参见 [[HPC 验收证据链]]。^[atlas-core-convert-expr.md:9-17, atlas-core-convert-expr.md:67-69]

## Sources

- [atlas-core-convert-expr.md](atlas-core-convert-expr.md)

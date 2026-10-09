---
title: 类型抽象中的 type_floor 调整
summary: convert_expr_context 调整类型下限，使类型抽象内的 return 操作数可引用外层函数要求，并按该要求实际的 fixed 下限解释类型变量。
sources:
  - atlas-core-convert-expr.md
kind: concept
createdAt: "2026-10-09T14:26:53.412Z"
updatedAt: "2026-10-09T20:31:02.786Z"
tags:
  - 类型作用域
  - 表达式转换
aliases:
  - 类型抽象中的-typefloor-调整
  - 类T调
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: 类型抽象中的 type_floor 调整
summary: convert_expr_context 调整类型下限，使类型抽象内的 return 操作数能够引用外层函数要求，并按该要求实际的 fixed 下限解释类型变量。
sources:
  - atlas-core-convert-expr.md
kind: concept
tags:
  - 类型系统
  - 类型抽象
  - 转换上下文
aliases:
  - 类型抽象中的-typefloor-调整
---

# 类型抽象中的 type_floor 调整

`type_floor` 调整是 `convert_expr_context` 处理类型抽象内 `return` 操作数时的上下文规则：操作数可以引用外层函数的类型要求，其中的类型变量按该要求**实际的 fixed 下限**解读。源材料将这一规则对应到上游 `convert_expr`。^[atlas-core-convert-expr.md:23-25]

## 转换入口与类型要求

`convert_expr(expression, required: &mut Type, analysis)` 将所需类型 `required` 装入共享的 `ConversionType`，采用当前 `analysis.type_floor`，并在转换完成后写回。入口传入的当前下限与调度器针对类型抽象内 `return` 的调整，共同构成这一上下文传递机制。^[atlas-core-convert-expr.md:21-25]

整体转换采用单遍检查与合成，in/out 类型模式只通过 `specialise` 变异；`conform_types` 先尝试特化，其次尝试强制转换，否则报告类型错误。`type_floor` 调整属于这一转换框架，相关机制见 [[convert_expr 的 in/out 类型模式与单遍转换]]。^[atlas-core-convert-expr.md:12-17]

## 表达式族之间的衔接

`return` 位于 `atom` 转换族，`TypeAbstraction` 与 `Cast` 位于 `type_context` 转换族。十二个族助手保留原有分支体，并使用已经调整的共享转换上下文；这种拆分属于机械分区，不改变语义。^[atlas-core-convert-expr.md:26-38]

十二个族助手均标记为 `#[inline(never)]`。源材料记录，相关栈问题的 GDB 陷阱命中的是分析帧，而非求值器，因此这里的函数划分涉及转换分析阶段。^[atlas-core-convert-expr.md:26-29]

## 证据边界

本说明依据 `typed.rs` 转换遍的结构性阅读，记录了入口、调度器与上下文规则，不构成语言行为验收。上游行号属于实现方的移植陈述，转换行为兼容仍以 HPC 语料门为准；相关证据要求参见 [[HPC 验收证据链]]。^[atlas-core-convert-expr.md:9-17, atlas-core-convert-expr.md:64-69]

## Sources

- [atlas-core-convert-expr.md](../../sources/atlas-core-convert-expr.md)：转换遍 convert_expr——in/out 类型模式、族划分与赋值助手契约。

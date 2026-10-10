---
title: 类型抽象中的 type_floor 调整
summary: convert_expr_context 调整类型下限，使类型抽象内的 return 操作数引用外层函数要求，并按该要求实际的 fixed 下限解释变量。
sources:
  - atlas-core-convert-expr.md
kind: concept
createdAt: "2026-10-09T14:26:53.412Z"
updatedAt: "2026-10-10T00:15:36.556Z"
tags:
  - 类型系统
  - 作用域
aliases:
  - 类型抽象中的-typefloor-调整
  - 类T调
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: 类型抽象中的 type_floor 调整
summary: convert_expr_context 调整类型下限，使类型抽象内的 return 操作数可引用外层函数要求，并按该要求实际的 fixed 下限解释类型变量。
sources:
  - atlas-core-convert-expr.md
kind: concept
tags:
  - 类型系统
  - 作用域
aliases:
  - 类型抽象中的-typefloor-调整
provenanceState: extracted
---

# 类型抽象中的 type_floor 调整

`type_floor` 调整是 `convert_expr_context` 的转换上下文规则：当 `return` 操作数位于类型抽象内时，它可以引用外层函数的类型要求；其中的类型变量按该要求**实际的 fixed 下限**解读。源材料将这一规则对应到上游 `convert_expr`。^[atlas-core-convert-expr.md:23-25]

## 入口与上下文传递

`convert_expr(expression, required: &mut Type, analysis)` 将所需类型 `required` 装入共享的 `ConversionType`，使用当前 `analysis.type_floor`，并在转换完成后写回。随后，`convert_expr_context` 按上述规则调整转换上下文中的类型下限。^[atlas-core-convert-expr.md:21-25]

这一规则属于 [[convert_expr 的 in/out 类型模式与单遍转换]]：转换遍同时完成类型检查与合成，in/out 类型模式只通过 `specialise` 变异；`conform_types` 先尝试特化，否则尝试强制转换，两者均失败时报告类型错误。^[atlas-core-convert-expr.md:12-14]

## 表达式族之间的衔接

`return` 由 `atom` 族处理，`TypeAbstraction` 与 `Cast` 由 `type_context` 族处理。十二个族助手保留原有分支体，并使用已经调整的共享转换上下文；这种拆分是机械分区，不改变语义。^[atlas-core-convert-expr.md:26-38]

十二个族助手均标记为 `#[inline(never)]`。源材料记录，相关栈问题中的 GDB 陷阱命中的是分析帧，而非求值器；这里的上下文传递与族划分属于表达式转换分析阶段。^[atlas-core-convert-expr.md:26-29]

## 证据边界

本页依据 `crates/atlas-core/src/typed.rs` 转换遍的结构性阅读，不构成语言行为验收。源材料中的上游行号属于实现方的移植陈述；转换行为兼容仍以 HPC 语料门为准，参见 [[HPC 验收证据链]]。^[atlas-core-convert-expr.md:9-17, atlas-core-convert-expr.md:64-69]

## Sources

- [atlas-core-convert-expr.md](../../sources/atlas-core-convert-expr.md)：转换遍 convert_expr（typed.rs 中部）——in/out 类型模式、族划分与赋值助手契约。

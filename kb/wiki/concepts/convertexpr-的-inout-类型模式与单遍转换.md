---
title: convert_expr 的 in/out 类型模式与单遍转换
summary: convert_expr 将 required 封装为共享 ConversionType 并在转换后写回，一遍完成检查与合成；conform_types 依次尝试特化、强转和类型错误。
sources:
  - atlas-core-convert-expr.md
kind: concept
createdAt: "2026-10-09T14:26:58.958Z"
updatedAt: "2026-10-09T14:26:58.958Z"
tags:
  - 类型检查
  - 表达式转换
  - Rust
aliases:
  - convertexpr-的-inout-类型模式与单遍转换
  - C的I类
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# convert_expr 的 in/out 类型模式与单遍转换

`convert_expr` 是 `typed.rs` 中的表达式转换入口，在一遍转换中同时执行类型检查与类型合成。其核心是 in/out 类型模式：调用方通过可变的 `required` 传入所需类型，转换过程使用共享类型上下文，并在完成后写回类型。源码自述该设计镜像上游 `axis.w:272–487`；类型模式仅经 `specialise` 变异。^[atlas-core-convert-expr.md:12-17, atlas-core-convert-expr.md:21-22]

## 类型模式与类型协调

入口签名中的 `required: &mut Type` 同时承担输入要求与输出类型的角色。`convert_expr` 将其装入共享的 `ConversionType`，使用当前 `analysis.type_floor`，随后执行转换并写回结果。这让类型要求参与转换过程，而检查与合成在同一遍中完成。^[atlas-core-convert-expr.md:12-14, atlas-core-convert-expr.md:21-22]

`conform_types` 按固定顺序协调类型：先尝试特化；不能特化时尝试强制转换；两者均不可行时报告类型错误。生成的转换节点在求值时调用已注册的转换函数。^[atlas-core-convert-expr.md:13-16]

## 转换上下文与类型下限

`convert_expr_context` 负责调度，并调整 `type_floor`。当 `return` 操作数处于类型抽象内部时，它仍可引用外层函数的类型要求；这些要求中的变量按其实际 fixed 下限解读。相关上下文约束可参见 [[Analysis 转换期上下文与活类型要求]]。^[atlas-core-convert-expr.md:23-25]

## 表达式族的机械分区

转换器按表达式种类拆成 12 个族助手：`atom`、`type_context`、`display`、`binding`、`assignment`、`subscription`、`slice`、`application`、`control`、`loop`、`tagged_case` 和 `case`，覆盖标量、绑定、赋值、调用、控制流、循环与分支匹配等形式。^[atlas-core-convert-expr.md:31-38]

这些助手全部标记为 `#[inline(never)]`，保留原有分支体及已调整的共享转换上下文；分区属于机械拆分，不改变语义。其背景是一次栈问题定位：GDB 陷阱命中的是分析帧，而非求值器。^[atlas-core-convert-expr.md:26-29]

## 所需类型如何影响转换

列表显示体现了上下文对元素定型的影响：在非行上下文中，转换器查询 `row_coercion`。例如，`mat: [[1,2]]` 会将元素定型为 `vec`。相关规则见 [[列表显示的行转换选择]]。^[atlas-core-convert-expr.md:14-15]

`while` 的转换模式同样由所需上下文决定：`void` 上下文使循环体采用 `void` 上下文，并允许异构分支；`int` 上下文选择计数模式；其他情况选择行模式，并带有 `reversed` 标志。行模式先尝试 `[*]` 特化，失败后回退到 `row_coercion`。^[atlas-core-convert-expr.md:40-44]

循环条件先进行 a-priori 转换，再检查是否为 `bool`；类型不符时使用上游措辞 `"found … while … was needed."`。循环层位于整棵 `do` 树之外，压平守卫不得越过这一边界，详见 [[while 转换的上下文模式与循环边界]]。^[atlas-core-convert-expr.md:40-44]

## 证据范围

本说明依据对 `typed.rs` 第 2757–5973 行的结构性阅读，覆盖转换入口、调度器、12 个表达式族助手及赋值助手群。内建注册表、`TypedExpr` 求值实现与测试不属于本包的展开范围；上游行号是实现方的移植陈述，不能据此宣称语言兼容性已获验收，转换行为兼容仍以 HPC 语料门为准。^[atlas-core-convert-expr.md:9-17, atlas-core-convert-expr.md:64-69]

## Sources

- [atlas-core-convert-expr.md](atlas-core-convert-expr.md)

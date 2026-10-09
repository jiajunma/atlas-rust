---
title: convert_expr 的 in/out 类型模式与单遍转换
summary: convert_expr 通过共享 ConversionType 单遍完成检查与合成并写回所需类型；conform_types 依次尝试特化、强转和类型错误。
sources:
  - atlas-core-convert-expr.md
kind: concept
createdAt: "2026-10-09T14:26:58.958Z"
updatedAt: "2026-10-09T22:13:26.400Z"
tags:
  - 类型检查
  - 编译器
aliases:
  - convertexpr-的-inout-类型模式与单遍转换
  - C的I类
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: convert_expr 的 in/out 类型模式与单遍转换
summary: convert_expr 将所需类型封装为共享 ConversionType，单遍完成类型检查与合成后写回；类型协调依次尝试特化、强制转换和类型错误。
sources:
  - atlas-core-convert-expr.md
kind: concept
tags:
  - 类型检查
  - 表达式转换
  - Rust
---

# convert_expr 的 in/out 类型模式与单遍转换

`convert_expr` 是 `typed.rs` 中的表达式转换入口，在一遍转换中同时完成类型检查与类型合成。其核心是 **in/out 类型模式**：调用方通过 `required: &mut Type` 传入所需类型，转换器将其封装为共享的 `ConversionType`，并在转换结束后写回。源码自述这一设计镜像上游 `axis.w:272–487`，类型模式仅经 `specialise` 变异。^[atlas-core-convert-expr.md:12-17, atlas-core-convert-expr.md:21-22]

## 类型协调与共享上下文

`conform_types` 按固定顺序协调类型：先尝试特化，失败后尝试强制转换，两者均不可行时报告类型错误。生成的转换节点在求值时调用已注册的转换函数。^[atlas-core-convert-expr.md:13-16]

入口封装 `required` 时使用当前的 `analysis.type_floor`。调度器 `convert_expr_context` 负责调整类型下限：当 `return` 操作数位于类型抽象内部时，它仍可引用外层函数的类型要求，其中的变量按这些要求实际的 fixed 下限解读。相关背景见 [[Analysis 转换期上下文与活类型要求]]。^[atlas-core-convert-expr.md:21-25]

## 表达式族的机械分区

转换器按表达式种类划分为 12 个族助手：`atom`、`type_context`、`display`、`binding`、`assignment`、`subscription`、`slice`、`application`、`control`、`loop`、`tagged_case` 和 `case`。这些族覆盖原子表达式、类型上下文、显示、绑定、赋值、下标、切片、调用、控制流、循环及分类分支。^[atlas-core-convert-expr.md:31-38]

12 个助手全部标记为 `#[inline(never)]`，保留原有分支体及已经调整的共享转换上下文，属于不改变语义的机械拆分。其背景是一次栈问题定位：GDB 陷阱命中的是分析帧，而非求值器。^[atlas-core-convert-expr.md:26-29]

## 所需类型对转换的影响

列表显示在非行上下文中查询 `row_coercion`；例如，要求 `mat` 类型的嵌套列表显示会将元素定型为 `vec`。详见 [[列表显示的行转换选择]]。^[atlas-core-convert-expr.md:14-15]

`while` 的 `WhileMode` 同样由所需上下文决定：`void` 上下文使循环体采用 `void` 上下文，并允许异构分支；`int` 上下文选择计数模式；其他情况选择行模式，并带有 `reversed` 标志。行模式先尝试 `[*]` 特化，失败后回退到 `row_coercion`。^[atlas-core-convert-expr.md:40-44]

循环条件先进行 a-priori 转换，再检查是否为 `bool`，类型不符时使用上游措辞 `"found … while … was needed."`。循环层包围整棵 `do` 树，压平守卫不得越过这一边界，参见 [[while 转换的上下文模式与循环边界]]。^[atlas-core-convert-expr.md:40-44]

## 赋值转换的配套契约

`set x := value` 与裸 `x := value` 刻意共享 `convert_simple_assignment` 路径。`lookup_assignable` 按局部遮蔽全局的规则查找目标，并使用赋值专用的未定义或常量诊断，引用整个表达式的紧凑渲染。^[atlas-core-convert-expr.md:48-51]

分量赋值受类型权限约束：行、`vec`、`mat` 的列或双下标项，以及 `KTypePol[KType]`、`ParamPol[Param]` 允许分量赋值；`ratvec` 在上游只读。详见 [[分量赋值的类型权限]]。^[atlas-core-convert-expr.md:52-55]

`resolve_projector` 使用保留的类型定义解析字段投影，而非投影函数的当前值；复制的或泛型定义也可匹配，因此具名接收方本身不能唯一确定字段选择。详见 [[基于保留类型定义的投影解析]]。^[atlas-core-convert-expr.md:56-58]

`factor_transform_call` 始终保留二元调用的两个操作数，不应用上游将 `x+1` 改写为 `succ(x)` 的丢参数优化。详见 [[变换调用中的二元操作数保留]]。^[atlas-core-convert-expr.md:59-60]

## 证据范围

本页依据对 `crates/atlas-core/src/typed.rs` 第 2757–5973 行的结构性阅读，范围包括转换入口、调度器、12 个表达式族转换函数及赋值助手群。内建注册表、`TypedExpr` 的求值实现和 133 个测试另待分包，不属于本包展开审查的内容。^[atlas-core-convert-expr.md:9-11, atlas-core-convert-expr.md:64-66]

来源中的上游行号属于实现方的移植陈述；结构性阅读不代表语言验收，转换行为兼容仍以 HPC 语料门为准。快照字节数与哈希仅标识所读字节，不能独立证明行为正确。^[atlas-core-convert-expr.md:17-17, atlas-core-convert-expr.md:67-69]

## Sources

- [atlas-core-convert-expr.md](../../sources/atlas-core-convert-expr.md) — 转换遍 convert_expr（typed.rs 中部）——in/out 类型模式、族划分与赋值助手契约。

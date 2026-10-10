---
title: TypedContext 会话状态与启动初始化
summary: TypedContext 汇集类型、绑定、求值及重载状态，按当前绑定记录位置；Default 播种补全名，new 再播种系统变量并将 prelude_log 标为常量。
sources:
  - atlas-core-typed-core.md
kind: concept
createdAt: "2026-10-09T14:37:26.378Z"
updatedAt: "2026-10-10T00:24:39.681Z"
tags:
  - 会话状态
  - 初始化
aliases:
  - typedcontext-会话状态与启动初始化
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: TypedContext 会话状态与启动初始化
summary: TypedContext 汇集类型、全局绑定、求值及重载状态，按当前绑定记录类型位置；Default 播种补全名，new 进一步播种系统变量并将 prelude_log 标为常量。
sources:
  - atlas-core-typed-core.md
kind: concept
tags:
  - 会话状态
  - 启动初始化
aliases:
  - typedcontext-会话状态与启动初始化
provenanceState: extracted
---

# TypedContext 会话状态与启动初始化

`TypedContext` 保存类型化会话的类型表、全局绑定、求值上下文、重载状态和报告详略设置，并记录当前标识符绑定的源码位置。启动初始化分为两步：`Default` 播种启动补全清单，`new()` 再播种系统变量，并将 `prelude_log` 标为常量。^[atlas-core-typed-core.md:100-106]

## 会话状态与绑定位置

核心字段包括 `types`、`globals`、`evaluation`（`EvaluationContext`）和 `overloads`。`verbosity` 控制报告详略，`set quiet` 对应 `0`，`set verbose` 对应 `1`。重载状态的有序合并、变体替换和缓存纪律见 [[OverloadState 有序重载管理]]。^[atlas-core-typed-core.md:81-89, atlas-core-typed-core.md:100-103]

`type_locations` 中的位置属于**当前标识符绑定**，而非可能被复用的类型表槽，因此重定义后必须报告新绑定的位置。相关的 [[IdTable 与 TypeCell 的绑定身份和类型精化]] 规则规定：每次定义持有新分配的 cell，重定义只更换名称对应的绑定，已经转换的代码仍保留原先捕获的 cell。^[atlas-core-typed-core.md:75-77, atlas-core-typed-core.md:102-103]

## 启动补全与系统变量

`STARTUP_COMPLETION_NAMES` 遵循上游 `main_hash_table` 的顺序：先列出 35 个关键字，再列出 21 个原始类型名，其后按上游注册顺序排列。`Default` 用该清单初始化补全；成功执行 `forget` 时，只更新受影响的名称，不隐藏其他尚未移植的启动名称。^[atlas-core-typed-core.md:96-105]

三个启动系统变量属于会话全局，不包含在 `STARTUP_COMPLETION_NAMES` 中，由 `TypedContext::new()` 播种。`new()` 还将 `prelude_log` 标记为 `const`。启动补全清单的播种与系统变量的播种因此分属不同的初始化步骤。^[atlas-core-typed-core.md:96-106]

## 与转换和事件层的衔接

[[Analysis 转换期上下文与活类型要求]] 中的 `Analysis` 持有 `types`、`globals` 和 `overloads` 的引用，并维护局部绑定、常量名、最近函数的活结果要求、循环深度及词法类型阈值。这些是表达式转换期间使用的分析状态。^[atlas-core-typed-core.md:62-73]

[[TypedCommandEvent 类型化命令事件]] 包含 `Diagnostic`、`Value`、`ReportLine`、`ReportBytes` 和 `Output`。其中值事件携带类型，供会话层判断 `is_void`；`report()` 将非法 UTF-8 字节分流到 `ReportBytes`，而不改写原值。^[atlas-core-typed-core.md:91-95]

## 证据范围

本页依据 `typed.rs` 上部核心数据结构与字段面的结构性阅读。来源未覆盖 `convert_expr`、内建注册表、`TypedExpr` 求值实现，以及 `EvaluationContext`、`frames.rs` 执行层的实现细节。上游行号属于实现方的移植陈述，行为兼容仍以 HPC 语料门为准；本页不构成语言或数学验收。^[atlas-core-typed-core.md:9-15, atlas-core-typed-core.md:108-114]

## Sources

- [atlas-core-typed-core.md](../../sources/atlas-core-typed-core.md) — 类型化管线核心数据结构（typed.rs 上部）——TypedExpr 树、Analysis/OverloadState 与 TypedContext。

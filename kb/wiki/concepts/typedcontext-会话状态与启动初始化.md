---
title: TypedContext 会话状态与启动初始化
summary: TypedContext 汇集类型、全局绑定、求值和重载状态，按当前绑定记录类型位置；Default 播种补全名，new 进一步播种系统变量并将 prelude_log 标为常量。
sources:
  - atlas-core-typed-core.md
kind: concept
createdAt: "2026-10-09T14:37:26.378Z"
updatedAt: "2026-10-09T14:37:26.378Z"
tags:
  - 会话上下文
  - 启动初始化
  - 绑定位置
aliases:
  - typedcontext-会话状态与启动初始化
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# TypedContext 会话状态与启动初始化

`TypedContext` 保存类型化会话的类型表、全局绑定、求值上下文、重载状态和报告配置，并维护类型绑定的源码位置。启动初始化分为两步：`Default` 播种启动补全清单，`new()` 再播种系统变量，并将 `prelude_log` 标为常量。^[atlas-core-typed-core.md:100-106]

## 会话状态

`TypedContext` 的核心字段为 `types`、`globals`、`evaluation`（`EvaluationContext`）与 `overloads`；`verbosity` 控制报告详略，`set quiet` 对应 `0`，`set verbose` 对应 `1`。重载状态的合并、替换与缓存纪律见 [[OverloadState 有序重载管理]]。^[atlas-core-typed-core.md:79-89, atlas-core-typed-core.md:100-103]

`type_locations` 将源码位置关联到当前标识符绑定，而非可能被复用的类型表槽。因此，重定义之后必须报告新绑定的位置。相关绑定身份规则见 [[IdTable 与 TypeCell 的绑定身份和类型精化]]：重定义更换名称所指向的 cell，而已经转换的代码保留原先捕获的 cell。^[atlas-core-typed-core.md:75-77, atlas-core-typed-core.md:102-103]

## 启动补全与系统变量

`STARTUP_COMPLETION_NAMES` 按上游 `main_hash_table` 顺序排列：先是 35 个关键字，再是 21 个原始类型名，之后按上游注册顺序排列。`Default` 用它播种启动补全清单；成功执行 `forget` 时只更新受影响的名称，不隐藏其他尚未移植的启动名称。^[atlas-core-typed-core.md:96-105]

三个启动系统变量属于会话全局，不包含在启动补全清单中，由 `TypedContext::new()` 单独播种。`new()` 还将 `prelude_log` 标记为 `const`。因此，补全清单初始化与系统变量绑定是两个明确区分的步骤。^[atlas-core-typed-core.md:96-106]

## 与转换和事件层的衔接

[[Analysis 转换期上下文与活类型要求]] 中的 `Analysis` 持有 `types`、`globals` 和 `overloads` 的引用，并维护局部绑定、当前函数结果要求、循环深度和词法类型阈值。这些字段描述表达式转换期间的上下文需求。^[atlas-core-typed-core.md:62-73]

[[TypedCommandEvent 类型化命令事件]] 提供诊断、值、报告行、报告字节及输出等事件。值事件携带类型，供会话层判断 `is_void`；`report()` 将非法 UTF-8 字节分流至 `ReportBytes`，不改写原值。^[atlas-core-typed-core.md:91-95]

## 证据范围

本页依据对 `typed.rs` 上部数据结构与字段面的结构性阅读。所提供材料未覆盖 `convert_expr`、内建注册表、`TypedExpr` 求值实现，以及 `EvaluationContext` 和 `frames.rs` 执行层；其中的上游行号属于实现方移植陈述，行为兼容仍以 HPC 语料门为准，不能由本页的结构说明推定语言或数学验收。^[atlas-core-typed-core.md:9-15, atlas-core-typed-core.md:108-114]

## Sources

- [atlas-core-typed-core.md](atlas-core-typed-core.md) — 类型化管线核心数据结构（typed.rs 上部）——TypedExpr 树、Analysis/OverloadState 与 TypedContext。

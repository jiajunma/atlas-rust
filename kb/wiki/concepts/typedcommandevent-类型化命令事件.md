---
title: TypedCommandEvent 类型化命令事件
summary: 以诊断、携带类型与位置的值、报告和输出事件连接类型化管线与会话层，并将报告中的非法 UTF-8 字节分流至 ReportBytes。
sources:
  - atlas-core-typed-core.md
kind: concept
createdAt: "2026-10-09T14:37:11.993Z"
updatedAt: "2026-10-09T22:21:30.156Z"
tags:
  - 命令事件
  - 字节保真
aliases:
  - typedcommandevent-类型化命令事件
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: TypedCommandEvent 类型化命令事件
summary: 类型化命令事件包含诊断、携带类型和源码跨度的值、报告及输出；会话层据值类型判断 is_void，非法 UTF-8 报告字节由 ReportBytes 保留。
sources:
  - atlas-core-typed-core.md
kind: concept
tags:
  - 命令事件
  - 会话接口
  - 字节保真
aliases:
  - typedcommandevent-类型化命令事件
provenanceState: extracted
---

# TypedCommandEvent 类型化命令事件

`TypedCommandEvent` 是 Atlas 类型化管线中的命令事件类型，包含 `Diagnostic`、`Value { value, type_, span }`、`ReportLine`、`ReportBytes` 和 `Output` 五种变体，用于表示诊断、值、报告与输出。^[atlas-core-typed-core.md:91-95]

## 值事件与会话处理

`Value` 同时携带值 `value`、类型 `type_` 和源码跨度 `span`。类型信息参与会话层的处理：会话层据此判断 `is_void`。因此，该事件保留的不只是求值结果，还包括会话处理所需的类型信息。^[atlas-core-typed-core.md:93-95]

## 报告的 UTF-8 分流

`report()` 根据 UTF-8 有效性分流报告内容，非法 UTF-8 字节进入 `ReportBytes`，值不被改写。相关会话输出主题可参见 [[SessionEvent 与字节保留输出]]。^[atlas-core-typed-core.md:93-95]

## 所属上下文

源材料将该事件类型与 [[TypedContext 会话状态与启动初始化]] 一并介绍。`TypedContext` 持有类型表、全局绑定、求值上下文、重载状态、详细程度和类型位置等字段；其中类型位置属于当前标识符绑定，重定义时必须报告新位置。^[atlas-core-typed-core.md:91-103]

## 证据边界

本页依据 `crates/atlas-core/src/typed.rs` 上部数据结构的结构性阅读，不构成语言或数学验收。源包未覆盖 `convert_expr`、内建注册表、`TypedExpr` 求值实现及执行层；上游行号对应关系属于实现方的移植陈述，行为兼容仍以 HPC 语料门为准，参见 [[HPC 验收证据链]]。^[atlas-core-typed-core.md:9-15, atlas-core-typed-core.md:108-114]

## Sources

- [atlas-core-typed-core.md](../../sources/atlas-core-typed-core.md) — 类型化管线核心数据结构（typed.rs 上部）：TypedExpr 树、Analysis/OverloadState 与 TypedContext。

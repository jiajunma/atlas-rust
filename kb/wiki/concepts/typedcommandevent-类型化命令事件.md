---
title: TypedCommandEvent 类型化命令事件
summary: 以诊断、携带类型与位置的值、报告和输出事件连接类型化管线与会话层，并将报告中的非法 UTF-8 字节分流至 ReportBytes。
sources:
  - atlas-core-typed-core.md
kind: concept
createdAt: "2026-10-09T14:37:11.993Z"
updatedAt: "2026-10-09T14:37:11.993Z"
tags:
  - 命令事件
  - 会话接口
  - 字节保真
aliases:
  - typedcommandevent-类型化命令事件
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# TypedCommandEvent 类型化命令事件

`TypedCommandEvent` 是 Atlas 类型化管线中的命令事件类型，包含诊断、携带类型的值、报告文本、报告字节及输出等变体。它将命令结果以不同事件形式交给会话层处理。^[atlas-core-typed-core.md:91-95]

## 事件变体

该类型的变体为 `Diagnostic`、`Value { value, type_, span }`、`ReportLine`、`ReportBytes` 和 `Output`。其中，`Value` 除值本身外，还携带类型与源码跨度；会话层依据其中的类型判断 `is_void`。因此，值事件中的类型信息直接参与会话层的处理。^[atlas-core-typed-core.md:93-95]

## 报告文本与字节保留

`report()` 根据 UTF-8 有效性分流报告内容：非法 UTF-8 字节进入 `ReportBytes`，值不被改写。这一行为可结合 [[SessionEvent 与字节保留输出]] 理解其在会话输出中的关联。^[atlas-core-typed-core.md:93-95]

## 上下文与证据边界

源材料将该事件类型与 [[TypedContext 会话状态与启动初始化]] 一并介绍；`TypedContext` 持有类型表、全局绑定、求值上下文、重载状态、详细程度及类型位置等字段。^[atlas-core-typed-core.md:91-106]

本页依据的是 `typed.rs` 上部数据结构的结构性阅读，不能据此宣称语言行为兼容已获验收。转换遍、内建注册表、`TypedExpr` 求值实现及执行层均不在该源包的覆盖范围内；行为兼容仍以 HPC 语料门为准，相关证据要求见 [[HPC 验收证据链]]。^[atlas-core-typed-core.md:9-15, atlas-core-typed-core.md:108-114]

## Sources

- [atlas-core-typed-core.md](atlas-core-typed-core.md) — 类型化管线核心数据结构（typed.rs 上部）。

---
title: OverloadState 有序重载管理
summary: 将静态启动重载、forget 移除记录与用户变体合并为有序列表，新增变体执行精确参数孪生替换、过近邻歧义检查或有序插入。
sources:
  - atlas-core-typed-core.md
kind: concept
createdAt: "2026-10-09T14:37:09.595Z"
updatedAt: "2026-10-09T14:37:09.595Z"
tags:
  - 重载解析
  - 有序合并
  - 上下文状态
aliases:
  - overloadstate-有序重载管理
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# OverloadState 有序重载管理

`OverloadState` 是每个类型化上下文的重载状态，对应上游 `global.w:446–560` 的 overload 表。它将静态启动注册表与后续移除、用户变体合并，在解析时提供一个有序列表。源码包覆盖其在 `typed.rs` 中的 641–1055 行。^[atlas-core-typed-core.md:79-89]

## 有序视图与更新规则

启动注册表保持静态；`forget name @ type` 记录移除，`set` 记录用户变体。解析使用合并后的有序列表，因此启动变体与用户变体共同参与重载管理。^[atlas-core-typed-core.md:81-83]

`add_user` 根据合并视图重放上游单表 `add`（`global.w:1004–1023`）的规则：参数精确相同的孪生变体原地替换，其中启动孪生通过 `forgotten` 隐藏；遇到过近邻则产生上游歧义错误，并保留完整错误措辞；其余变体按有序位置插入。该方法返回更新前后的变体数，用于选择报告措辞。^[atlas-core-typed-core.md:86-89]

## 缓存与失效

`OverloadState.views` 由 `TypeTable` 的 `revision` Arc 守护；指针不等时清空缓存。缓存只保存不可变、未移位的签名与来源索引。事务性的 `Clone` 不携带缓存，缓存被视为可丢弃的一次性状态。相关背景见 [[TypeTable 修订身份与缓存失效]]。^[atlas-core-typed-core.md:83-85]

转换期的 `Analysis` 另有 `overload_views`，用于以 `Rc` 共享未移位的有序签名视图。该缓存只保存结构签名，不保存推断类型或跨命令解析结果。由于公开的 `Analysis` 引用可在克隆上重新绑定，实现使用 `std::ptr::eq` 检查类型表与重载表的同一性，不符时清空缓存。参见 [[Analysis 转换期上下文与活类型要求]]。^[atlas-core-typed-core.md:62-73]

## 上下文归属

`TypedContext` 持有 `overloads` 字段，与 `types`、`globals` 和求值上下文共同组成会话状态；`Analysis` 则持有类型表、全局绑定表和重载状态的引用，以供转换期使用。参见 [[TypedContext 会话状态与启动初始化]]。^[atlas-core-typed-core.md:64-71, atlas-core-typed-core.md:100-106]

## 证据边界

来源将上述缓存纪律标为已验收的跨命令设计，但本包整体属于结构性阅读，不声称语言或数学验收。`convert_expr`、内建注册表及 `TypedExpr` 求值实现不在本包范围内；上游行号是实现方的移植陈述，行为兼容仍以 HPC 语料门为准。^[atlas-core-typed-core.md:9-15, atlas-core-typed-core.md:83-85, atlas-core-typed-core.md:108-114]

## Sources

- [atlas-core-typed-core.md](../../sources/atlas-core-typed-core.md) — 类型化管线核心数据结构（typed.rs 上部）。

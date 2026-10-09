---
title: readline_completions 的补全快照前缀过滤
summary: readline_completions 按输入前缀过滤命令层保存在求值上下文中的补全候选快照。
sources:
  - atlas-core-builtin-registry.md
kind: concept
createdAt: "2026-10-09T18:27:44.197Z"
updatedAt: "2026-10-09T18:27:44.197Z"
tags:
  - 命令补全
  - 求值上下文
aliases:
  - readlinecompletions-的补全快照前缀过滤
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# readline_completions 的补全快照前缀过滤

`readline_completions` 通过 `BuiltinImpl::Completions` 实现，按前缀过滤命令层存放在求值上下文中的补全候选快照。源材料将这一行为对应到上游 `global.w:3546–3561`。^[atlas-core-builtin-registry.md:35-36]

## 实现分发与可见性

`Completions` 是 `BuiltinImpl` 的一个独立实现分支。内建元数据中的 `overload_visible` 字段控制条目是否出现在重载解析与补全中；该可见性字段与 `readline_completions` 对候选快照执行的前缀过滤分别见于注册元数据和实现分发描述。相关结构见 [[内建函数元数据与实现分发]]。^[atlas-core-builtin-registry.md:20-36]

## 补全清单与注册表的边界

`builtin_registry()` 使用 `OnceLock<Vec<Builtin>>` 一次构建，包含 479 个条目、240 个不同名字，但尚未覆盖上游完整启动清单。来源另提到 `STARTUP_COMPLETION_NAMES` 包含按上游哈希序排列的 309 个启动名，并显式记录注册缺口。因此，理解补全候选时需要区分启动补全名清单与已实现的注册条目，参见 [[内建注册表的启动清单与覆盖边界]]。^[atlas-core-builtin-registry.md:69-76]

补全名清单也不是领域调用的无值策略清单。`DomainNoValue` 的 `Skip`、`Validate` 和 `BuildAndDrop` 决定领域调用在 `NoValue` 级别执行多少工作；例如 `orientation_nr` 注册为 `BuildAndDrop`，而非 `Skip`。这些行为应结合 [[领域调用的无值门策略]] 单独理解。^[atlas-core-builtin-registry.md:42-47]

## 证据边界

来源属于结构性阅读，不声称语言或数学验收；逐条目语义、`TypedExpr` 求值实现及相关测试不在该材料的覆盖范围内。上游行号属于实现方的移植陈述，行为兼容仍以 HPC 语料门为准，参见 [[HPC 验收证据链]]。^[atlas-core-builtin-registry.md:9-16, atlas-core-builtin-registry.md:78-83]

## Sources

- [atlas-core-builtin-registry.md](atlas-core-builtin-registry.md) — 内建注册表：实现分发、无值门策略与启动清单。

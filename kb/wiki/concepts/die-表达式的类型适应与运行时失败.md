---
title: Die 表达式的类型适应与运行时失败
summary: Die 能通过任何所需类型的分析，但求值时抛出 I die；来源仅记录节点契约，不构成语言兼容性验收。
sources:
  - atlas-core-syntax.md
kind: concept
createdAt: "2026-10-10T00:23:50.042Z"
updatedAt: "2026-10-10T00:23:50.042Z"
tags:
  - 类型检查
  - 求值语义
  - 错误处理
aliases:
  - die-表达式的类型适应与运行时失败
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

# Die 表达式的类型适应与运行时失败

`Die` 是 Atlas 语法前端的表达式节点，能够通过任何所需类型的分析，但在求值时抛出 `I die`。它将类型分析阶段的适应能力与运行时失败行为分开：通过类型分析不意味着求值能够成功返回。^[atlas-core-syntax.md:32-50]

## 类型适应

`Die` 不受某一种所需类型的限制；来源明确说明，它可以通过任何所需类型的分析，并将这一行为对应到上游 `axis.w:630`。这一规则描述的是类型分析的接纳能力，并不表示它能产生任意类型的正常值。^[atlas-core-syntax.md:45-47]

## 运行时失败与实现职责

求值到 `Die` 时会抛出 `I die`。语法前端将其表示为带有源码跨度（span）的 `Expr` 变体；AST 节点的语义判定由 `typed.rs` 中的转换与求值逻辑承担。相关执行层可参见 [[TypedExpr 可执行表达式树]] 与 [[TypedExpr 六族求值与值需求层级]]。^[atlas-core-syntax.md:16-16, atlas-core-syntax.md:45-47, atlas-core-syntax.md:106-108]

## 测试与证据边界

来源记载语法前端共有 51 个测试，覆盖表达式、命令及诊断形状，但未单独列出 `Die` 的测试锚点，因此不能据此宣称其类型适应或运行时失败已有专项验证。本页依据结构性阅读；与上游文法的对应属于实现方移植陈述，语言兼容性仍以 HPC 语言语料门为准。^[atlas-core-syntax.md:9-14, atlas-core-syntax.md:104-110]

## Sources

- [atlas-core-syntax.md](../../sources/atlas-core-syntax.md) — 语法前端（syntax.rs + grammar.lalrpop）——AST 面、LALRPOP 适配与 Bison 风格诊断。

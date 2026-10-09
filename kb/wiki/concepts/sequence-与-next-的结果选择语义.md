---
title: Sequence 与 Next 的结果选择语义
summary: Sequence 在 void 上下文中求第一个表达式的效果并返回第二个值，Next 返回第一个值且仍对第二个表达式求效果。
sources:
  - atlas-core-syntax.md
kind: concept
createdAt: "2026-10-09T20:44:22.816Z"
updatedAt: "2026-10-09T20:44:22.816Z"
tags:
  - 求值顺序
  - 语言语义
  - 抽象语法树
aliases:
  - sequence-与-next-的结果选择语义
  - S与N的
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

# Sequence 与 Next 的结果选择语义

`Sequence` 与 `Next` 都是 Atlas 语法前端的表达式节点，但选择不同表达式的值作为结果：`Sequence` 产出第二个表达式的值，`Next` 产出第一个表达式的值；另一个表达式仍为其效果而求值。^[atlas-core-syntax.md:41-50]

## Sequence：保留第二个结果

`Sequence` 的第一个表达式在 `void` 上下文中求值，用于执行效果；整个序列产出第二个表达式的值。因此，第一个表达式的求值仍是序列语义的一部分，但它的值不是序列结果。^[atlas-core-syntax.md:41-41]

## Next：保留第一个结果

`Next` 产出第一个表达式的值，第二个表达式仍为其效果而求值。选择第一个结果并不意味着跳过第二个表达式：结果选择与效果求值在此分别承担不同职责。^[atlas-core-syntax.md:49-50]

## 实现分层与证据边界

这两个节点属于 `Expr` AST，均带有源码跨度 `span`。语法前端描述节点结构，AST 节点的语义判定则位于负责转换与求值的 `typed.rs`；相关分层可参见 [[类型化转换与求值管线]]。^[atlas-core-syntax.md:16-16, atlas-core-syntax.md:32-50, atlas-core-syntax.md:106-108]

来源记录了覆盖表达式、命令与诊断形状的 51 个测试，但未单独列出 `Sequence` 或 `Next` 的专项测试。该材料属于结构性阅读，不声称语言验收；语法兼容仍以 HPC 语言语料门为准。^[atlas-core-syntax.md:9-14, atlas-core-syntax.md:104-110]

## Sources

- [atlas-core-syntax.md](../../sources/atlas-core-syntax.md) — 语法前端（syntax.rs + grammar.lalrpop）。

---
title: IntCase 的索引分支与越界回绕
summary: IntCase 使用零基索引选择分支，then 接收负索引、else 接收越界索引，两者均缺省时采用取模回绕；本包仅记录语义说明。
sources:
  - atlas-core-syntax.md
kind: concept
createdAt: "2026-10-09T20:44:11.486Z"
updatedAt: "2026-10-09T20:44:11.486Z"
tags:
  - 控制流
  - 分支选择
  - 语言语义
aliases:
  - intcase-的索引分支与越界回绕
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

# IntCase 的索引分支与越界回绕

`IntCase` 是 Atlas 语法前端的整数索引分支节点，按 **0 基索引**选择分支。其边界规则为：`then` 接收负值，`else` 接收越界值；两者都缺省时，采用取模回绕。^[atlas-core-syntax.md:47-49]

## 索引与边界规则

正常分支选择从索引 0 开始。对于边界输入，来源分别列出负值的 `then` 分支与越界值的 `else` 分支，并明确将取模回绕限定在 **`then` 和 `else` 均缺省**的情形。来源没有说明仅缺省其中一个分支时，未被接收的边界值如何处理，因此不能将回绕规则推广到该情形。^[atlas-core-syntax.md:47-49]

## AST 与语义层的职责

`IntCase` 属于每个变体都携带 span 的 `Expr` AST。它与按标签判别的 `Case`、按变体位置应用分支函数的 `UnionCase` 并列，但分支选择依据是整数索引。^[atlas-core-syntax.md:16-18, atlas-core-syntax.md:47-49]

语法前端负责表达式结构；AST 节点的语义判定位于 `typed.rs`，涵盖转换与求值。理解 `IntCase` 的实现时，应结合 [[类型化转换与求值管线]]，区分语法节点的表示与后续语义处理。^[atlas-core-syntax.md:106-108]

## 证据边界

来源记录了语法前端的 51 个测试，覆盖表达式、命令与诊断形状，但未单独列出 `IntCase` 的测试用例。该材料属于结构性阅读，不声称语言验收；文法与上游 `parser.y` 的对应属于实现方移植陈述，语法兼容仍以 HPC 语言语料门为准。^[atlas-core-syntax.md:9-14, atlas-core-syntax.md:104-110]

## Sources

- [atlas-core-syntax.md](../../sources/atlas-core-syntax.md)：语法前端（syntax.rs + grammar.lalrpop）。

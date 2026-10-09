---
title: Do 守卫体的共享词法作用域
summary: while 控制树中的 Do 节点要求 let 或 case 引入的词法帧同时包住两个表达式，Dont 则仅由 while 的 do_expr 文法接纳。
sources:
  - atlas-core-syntax.md
kind: concept
createdAt: "2026-10-09T20:44:25.217Z"
updatedAt: "2026-10-09T20:44:25.217Z"
tags:
  - 控制流
  - 词法作用域
  - 语法前端
aliases:
  - do-守卫体的共享词法作用域
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

# Do 守卫体的共享词法作用域

`Do` 是 Atlas 语法树中位于 `While` 控制树内的守卫体节点。它的关键作用域约束是：词法 `let`／`case` 帧必须**同时包住两个表达式**。源材料将这一要求对应到上游 `axis.w:5887–5898`。^[atlas-core-syntax.md:42-44]

## 共享作用域约束

`Do` 的作用域要求涉及两个表达式共同处于词法帧的覆盖范围内，而非仅让其中一个表达式处于该帧中。实现或移植这一节点时，必须保留 `let`／`case` 帧对两者的共同包围关系；这是源材料明确指出的语义约束。^[atlas-core-syntax.md:43-44]

`Do` 属于带有源码跨度（span）的 `Expr` 变体。语法前端负责形成该节点，而 AST 节点的语义判定位于 `typed.rs` 的转换与求值阶段，可结合 [[类型化转换与求值管线]] 理解这一职责划分。^[atlas-core-syntax.md:16-16, atlas-core-syntax.md:43-44, atlas-core-syntax.md:106-108]

## 与 While 控制结构的关系

`While` 将每次迭代的体值收集到一行中；`while do…` 形式省略条件时，条件默认为 `true`。`Do` 则表示这一控制树内部的守卫体，因此解释其共享词法作用域时，需要保留它在循环结构中的位置。^[atlas-core-syntax.md:42-44]

同一控制结构还包含受文法位置限制的 `Dont`：它仅被 `while` 的 `do_expr` 文法接纳。循环退出由 `Break` 表示，重复的 `break` token 解开 `levels+1` 层循环，被解开层的当次迭代不贡献值；相关行为见 [[多层 Break 的循环退出与值收集]]。^[atlas-core-syntax.md:44-46]

## 证据范围

源材料是对 `syntax.rs` 与 `grammar.lalrpop` 的结构性阅读，未声称语言验收。所列 51 个测试覆盖表达式、命令与诊断形状，但材料没有单独列出 `Do` 共享作用域的专门测试；上游对应关系属于实现方的移植陈述，语法兼容仍以 HPC 语言语料门为准。^[atlas-core-syntax.md:9-14, atlas-core-syntax.md:104-110]

## Sources

- [atlas-core-syntax.md](../../sources/atlas-core-syntax.md) — 语法前端（syntax.rs + grammar.lalrpop）。

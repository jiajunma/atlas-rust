---
title: 递归函数字面量 RecLambda
summary: rec_fun 声明的结果类型作为函数体的上下文类型，函数名在体内绑定到闭包自身；本来源仅说明语法与语义契约，未构成语言兼容验收。
sources:
  - atlas-core-syntax.md
kind: concept
createdAt: "2026-10-09T22:20:35.550Z"
updatedAt: "2026-10-09T22:20:35.550Z"
tags:
  - 语法前端
  - 递归函数
  - 类型分析
aliases:
  - 递归函数字面量-reclambda
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

# 递归函数字面量 RecLambda

`RecLambda` 是 Atlas 语法前端中表示递归函数字面量的 `Expr` 变体，携带源码位置 `span`。其语法形式为 `rec_fun name(params) result: body`：声明的结果类型作为函数体的上下文类型，函数名 `name` 在体内绑定到闭包自身。^[atlas-core-syntax.md:16-18, atlas-core-syntax.md:32-37]

## 自身绑定与结果类型

语法前端区分非递归函数字面量 `Lambda` 与递归函数字面量 `RecLambda`。后者通过体内的 `name` 绑定提供对当前闭包自身的引用；声明的 `result` 则为函数体提供类型分析所用的上下文类型。相关的类型分析背景可参见 [[Analysis 转换期上下文与活类型要求]]。^[atlas-core-syntax.md:35-37]

参数由 `LambdaParam` 表示，可采用 `Typed(TypedParam)`，即 `type pattern` 形式，也可采用 `Tuple` 解构。模式层支持名称和元组解构等形式，用于表达参数的绑定结构。^[atlas-core-syntax.md:52-59]

## 语法与语义的职责边界

`RecLambda` 属于语法 AST；AST 节点的语义判定由 `typed.rs` 中的转换与求值层负责。词法处理位于 `lex.rs`，逐命令驱动位于 `session.rs`。因此，语法节点的存在与字段约定，应与后续转换、求值行为分开理解；相关主题见 [[TypedExpr 可执行表达式树]] 与 [[TypedExpr 六族求值与值需求层级]]。^[atlas-core-syntax.md:106-108]

## 证据范围

来源记录了覆盖表达式、命令和诊断形状的 51 个测试，但未单独列出 `RecLambda` 的测试锚点。材料属于结构性源码阅读，不声称语言验收；文法与上游 `parser.y` 的逐条对应属于实现方移植陈述，语法兼容仍以 HPC 语言语料门为准。^[atlas-core-syntax.md:9-14, atlas-core-syntax.md:104-110]

## Sources

- [atlas-core-syntax.md](../../sources/atlas-core-syntax.md) — 语法前端（syntax.rs + grammar.lalrpop）：AST 面、LALRPOP 适配与 Bison 风格诊断。

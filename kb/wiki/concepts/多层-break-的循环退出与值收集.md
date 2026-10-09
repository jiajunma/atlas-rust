---
title: 多层 Break 的循环退出与值收集
summary: 重复 break token 编码退出 levels+1 层循环，被退出层的当前迭代不贡献值，与循环体值收集规则共同定义退出语义。
sources:
  - atlas-core-syntax.md
kind: concept
createdAt: "2026-10-09T20:44:31.582Z"
updatedAt: "2026-10-09T20:44:31.582Z"
tags:
  - 控制流
  - 循环
  - 语言语义
aliases:
  - 多层-break-的循环退出与值收集
  - 多B的
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

# 多层 Break 的循环退出与值收集

`Break` 是 Atlas 语法前端的循环退出表达式。重复的 `break` token 用于表达多层退出：节点中的 `levels` 对应解开 `levels + 1` 层循环，且被解开各层的当次迭代都不贡献值。^[atlas-core-syntax.md:43-45]

## 退出层数与值收集

`While` 将每次迭代的体值收集到一行中；`while do…` 省略条件时，条件默认为 `true`。多层 `Break` 对这一收集过程施加明确限制：退出所涉及的循环层，其当次迭代值不会加入结果。^[atlas-core-syntax.md:41-45]

`levels` 不是直接的退出层数，实际退出层数必须加一。因此，`levels = 0` 表示解开一层，`levels = 1` 表示解开两层；多层退出不能仅按最内层循环的一次退出理解。^[atlas-core-syntax.md:44-45]

## 与其他控制表达式的关系

循环体内还可能出现影响值选择或作用域的表达式。[[Sequence 与 Next 的结果选择语义|Sequence]] 对第一个表达式在 `void` 上下文中求效果，并产出第二个表达式的值；`Next` 则产出第一个表达式的值，第二个仍求效果。它们规定表达式的结果选择，而 `Break` 规定退出层数及被退出层的当次迭代不贡献值。^[atlas-core-syntax.md:41-50]

`Do` 是 `while` 控制树内的守卫体，其词法 `let`／`case` 帧必须同时包住两个表达式，参见 [[Do 守卫体的共享词法作用域]]。这一作用域要求与多层退出的值收集规则同属循环控制语义，但分别约束名字可见性和迭代结果。^[atlas-core-syntax.md:43-45]

## 实现与证据边界

`Break` 属于每个变体都带有 span 的 `Expr` AST。语法前端负责构造节点，AST 节点的语义判定由 `typed.rs` 的转换与求值层负责；词法处理位于 `lex.rs`，逐命令驱动位于 `session.rs`。^[atlas-core-syntax.md:16-18, atlas-core-syntax.md:43-45, atlas-core-syntax.md:106-108]

来源记录了覆盖表达式、命令和诊断形状的 51 个测试，但未单独列出多层 `Break` 的测试用例。该材料属于结构性阅读，不声称语言验收；与上游文法的对应属于实现方移植陈述，语法兼容仍以 HPC 语言语料门为准。^[atlas-core-syntax.md:9-14, atlas-core-syntax.md:104-110]

## Sources

- [atlas-core-syntax.md](../../sources/atlas-core-syntax.md) — 语法前端（syntax.rs + grammar.lalrpop）——AST 面、LALRPOP 适配与 Bison 风格诊断。

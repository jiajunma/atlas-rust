---
title: 类型抽象节点 TypeAbstraction
summary: TypeAbstraction 将词法刚性类型变量在离开体时抽象为类型方案，节点本身在类型分析阶段消失。
sources:
  - atlas-core-syntax.md
kind: concept
createdAt: "2026-10-09T22:20:39.334Z"
updatedAt: "2026-10-09T22:20:39.334Z"
tags:
  - 类型系统
  - 类型抽象
  - 抽象语法树
aliases:
  - 类型抽象节点-typeabstraction
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

# 类型抽象节点 TypeAbstraction

`TypeAbstraction` 是 Atlas 语法前端的 `Expr` 变体，用于在离开表达式体时，将词法刚性类型变量抽象成类型方案（scheme）。它与其他表达式节点一样携带源码跨度（span），并在类型分析阶段消失。源材料将这一行为对应到上游 `axis.w:4109`。^[atlas-core-syntax.md:16-18, atlas-core-syntax.md:38-39]

## 语法表示与类型分析

`TypeAbstraction` 在 AST 中保留类型抽象这一结构，而节点的语义判定由 `typed.rs` 中的转换与求值层承担。因此，理解该节点需要区分语法树中的显式表示与类型分析时完成的抽象操作；相关概念可参见 [[TypeScheme 与类型变量作用域]]。^[atlas-core-syntax.md:38-39, atlas-core-syntax.md:106-108]

类型表达式由独立的 `TypeExpr` 表示，其中包括 `Variable{index}`、`Applied{name,args}`、基本类型、行、元组、联合及函数类型等。`TypeExpr::Variable` 是类型表达式中的变量形式，`Expr::TypeAbstraction` 则是表达式层中执行上述抽象的节点。^[atlas-core-syntax.md:38-39, atlas-core-syntax.md:60-64]

## 与其他函数相关节点的区别

`Lambda` 表示非递归函数字面量；`RecLambda` 使用声明的结果类型作为函数体的上下文类型，并在体内将函数名绑定到闭包自身。`TypeAbstraction` 的说明则聚焦于词法刚性类型变量到 scheme 的抽象，三者在 AST 中是不同变体。^[atlas-core-syntax.md:35-39]

`OperatorCast`（`f@ T (T)`）按参数类型选择全局重载，但不调用它；其显式形式参数是自由变量，并不构成抽象。这一规则应与 `TypeAbstraction` 的类型变量抽象行为分别理解。^[atlas-core-syntax.md:33-39]

## 证据边界

来源是对 `syntax.rs` 与 `grammar.lalrpop` 的结构性阅读，不声称语言验收。材料列出的 51 个测试覆盖表达式、命令和诊断形状，但未单独列出 `TypeAbstraction` 的测试锚点；文法与上游的对应关系属于实现方移植陈述，语法兼容仍以 HPC 语言语料门为准，参见 [[HPC 验收证据链]]。^[atlas-core-syntax.md:9-14, atlas-core-syntax.md:104-110]

## Sources

- [atlas-core-syntax.md](../../sources/atlas-core-syntax.md) — 语法前端（syntax.rs + grammar.lalrpop）——AST 面、LALRPOP 适配与 Bison 风格诊断。

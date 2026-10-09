---
title: OperatorCast 的按类型重载选择
summary: OperatorCast 表达按参数类型选择全局重载而不执行调用，显式形式参数中的类型变量保持自由，不在该节点中抽象。
sources:
  - atlas-core-syntax.md
kind: concept
createdAt: "2026-10-09T20:44:12.169Z"
updatedAt: "2026-10-09T20:44:12.169Z"
tags:
  - 类型系统
  - 重载解析
  - 语法前端
aliases:
  - operatorcast-的按类型重载选择
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

# OperatorCast 的按类型重载选择

`OperatorCast` 是 Atlas 语法前端中用于按参数类型选择全局重载的表达式节点，形式为 `f@ T (T)`。它只进行重载选择，不调用所选函数。^[atlas-core-syntax.md:32-35]

## 类型选择与自由变量

`OperatorCast` 的选择依据是参数类型。其显式形式参数是**自由变量**，不构成抽象；因此，不能将这一语法解释为创建函数字面量或引入参数绑定的操作。^[atlas-core-syntax.md:34-35]

语法前端分别提供 `OperatorCast`、`Call` 与 `Lambda` 节点：`OperatorCast` 负责按类型选择重载，`Call` 表示调用，`Lambda` 表示非递归函数字面量。此外，`TypeAbstraction` 负责在离开体时将词法刚性类型变量抽象成 scheme，与 `OperatorCast` 的自由变量约定不同。^[atlas-core-syntax.md:34-39]

## 与其他算符节点的区别

`OperatorCall` 保存 `FormulaOperator` 与参数；除以 `Unary`／`Binary` 表示的 Not、And、Or 外，其余算符走 `OperatorCall`。`OperatorCast` 则是独立的按类型选择节点。普通 `Cast` 也有独立形式 `type: expr`，不应与 `f@ T (T)` 混同。^[atlas-core-syntax.md:32-40]

全局重载表的一个入口是 `Set` 命令：它采用与 `let` 相同的声明形式，并行绑定到全局表，其中函数型叶子进入重载表，其余进入标识符表。这为理解 `OperatorCast` 所选择的全局重载提供了上下文，可结合 [[有序强制转换注册表]] 与 [[函数调用分派与内建参数解包]] 阅读。^[atlas-core-syntax.md:68-75]

## 实现与证据边界

`OperatorCast` 属于每个变体均携带 span 的 `Expr` AST。源包覆盖 `syntax.rs` 与 `grammar.lalrpop`，而 AST 节点的语义判定位于 `typed.rs` 的转换与求值阶段；相关层次见 [[类型化转换与求值管线]]。^[atlas-core-syntax.md:9-16, atlas-core-syntax.md:106-108]

来源记录了 51 个覆盖表达式、命令及诊断形状的测试，但没有列出 `OperatorCast` 专属测试或完整的重载匹配与失败规则。该材料属于结构性阅读，不声称语言验收；文法与上游 `parser.y` 的对应属于实现方移植陈述，语法兼容仍以 HPC 语言语料门为准。^[atlas-core-syntax.md:9-14, atlas-core-syntax.md:104-110]

## Sources

- [atlas-core-syntax.md](../../sources/atlas-core-syntax.md) — 语法前端（syntax.rs + grammar.lalrpop）。

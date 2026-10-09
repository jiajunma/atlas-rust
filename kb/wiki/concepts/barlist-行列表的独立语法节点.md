---
title: BarList 行列表的独立语法节点
summary: BarList 将非空的 int 行段保留为独立 AST 节点，使后续转换保留原版诊断并避免受用户 ^ 或 mat 重载影响；源码移植陈述不构成兼容验收。
sources:
  - atlas-core-syntax.md
kind: concept
createdAt: "2026-10-09T20:44:05.712Z"
updatedAt: "2026-10-09T20:44:05.712Z"
tags:
  - 语法前端
  - 抽象语法树
  - 兼容性
aliases:
  - barlist-行列表的独立语法节点
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

# BarList 行列表的独立语法节点

`BarList` 是 Atlas 语法前端中表示 `[a,b | c,d]` 行列表的独立 `Expr` 节点，与普通 `List` 分开表示，并和其他表达式变体一样携带源码跨度（span）。其每段必须非空，且静态类型为 `int` 行。^[atlas-core-syntax.md:16-23]

## 独立节点的设计目的

上游 `parser.y:370–376` 通过隐藏内建 `"transpose "` 将行列表脱糖为 `transpose (mat: …)`。Rust 前端保留独立的 `BarList` 节点，使后续转换能够保留 oracle 的精确诊断，并避免受到用户对 `^` 或 `mat` 的重载影响。这里的独立表示承担了诊断与转换行为的兼容职责。^[atlas-core-syntax.md:18-22]

## 解析与语义处理的边界

语法前端由 `syntax.rs` 与 `grammar.lalrpop` 组成，适配层将有状态的 Atlas 词法流转换为解析器使用的带跨度 token 流，使词法器的上下文敏感规则不进入文法动作。`BarList` 属于这一前端的 AST 表示。^[atlas-core-syntax.md:9-23]

AST 节点的语义判定由 `typed.rs` 中的转换与求值承担，词法处理位于 `lex.rs`，逐命令驱动位于 `session.rs`。因此，行列表的语法表示与后续类型、求值处理分属不同层次，可结合 [[类型化转换与求值管线]] 阅读。^[atlas-core-syntax.md:104-109]

## 测试与证据边界

来源记录语法前端有 51 个测试，覆盖表达式、命令和诊断形状，但未单独列出 `BarList` 的测试用例或通过结果。该材料属于结构性源码阅读，不声称语言验收；文法与上游 `parser.y` 的对应关系是实现方的移植陈述，语法兼容仍以 HPC 语言语料门为准。^[atlas-core-syntax.md:9-14, atlas-core-syntax.md:104-110]

## Sources

- [atlas-core-syntax.md](../../sources/atlas-core-syntax.md) — 语法前端（syntax.rs + grammar.lalrpop）——AST 面、LALRPOP 适配与 Bison 风格诊断。

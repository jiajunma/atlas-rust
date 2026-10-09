---
title: 有状态词法流的 LALRPOP 惰性适配
summary: TokenStream 将 Atlas 词法 token 惰性转换为携带位置的 ParserToken 流，使上下文敏感词法规则独立于文法动作，并将命令边界保留给词法器。
sources:
  - atlas-core-syntax.md
kind: concept
createdAt: "2026-10-09T14:36:14.673Z"
updatedAt: "2026-10-09T14:36:14.673Z"
tags:
  - LALRPOP
  - 词法分析
  - 解析器适配
aliases:
  - 有状态词法流的-lalrpop-惰性适配
  - 有L惰
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 有状态词法流的 LALRPOP 惰性适配

有状态词法流的 LALRPOP 惰性适配，是 Atlas 语法前端将词法器输出转换为带源码位置的解析器 token 流的机制。文法由 LALRPOP 生成，独立适配层负责衔接有状态的 Atlas 词法流，使词法器的上下文敏感规则不进入文法动作。^[atlas-core-syntax.md:9-14]

## Token 流转换

`TokenStream` 将 `Token` 序列惰性转换为 LALRPOP 接收的 `(offset, ParserToken, offset)` 流。`ParserToken` 表示文法使用的 token 集，并以 `SpannedValue` 携带载荷；`parser_operator` 负责算符名映射，`Display` 提供文法可读名称。相关概念见 [[Atlas 词法 Token 模型]] 与 [[Atlas 语法前端与运算符优先级归约]]。^[atlas-core-syntax.md:93-96]

## 命令边界与活跃类型环境

命令边界仍由词法器掌握，适配层不预先切分剩余源码，因为会话状态可能在下一条命令开始前改变。`parse_command` / `parse_command_in` 使用活跃的 `TypeTable`，使分类能够看到先前的类型声明；惰性流同时拥有命令局部的词法作用域。这一纪律与 [[TypeTable 的稳定身份与活跃绑定]] 及 [[会话帧驱动的 CLI 执行模型]] 相关。^[atlas-core-syntax.md:77-82]

## 多行片段与类型作用域

会话入口 `parse_command_fragment_in` 支持未完成片段的继续解析：当片段尚未完成、`allow_more` 为真，且 `scope.has_virtual_group()` 表明解析器持有的类型规格尚未闭合时，返回 `Ok(None)`。物理换行不会终止这种类型规格；下一行到来后，使用保留的前缀重试，完整解析前不求值任何 AST。EOF 与非 EOF 语法错误仍按正常路径诊断，参见 [[多行命令的增量解析]]。^[atlas-core-syntax.md:83-86]

解析器持有的词法类型作用域由 `syntax/type_scope.rs` 实现，文法源位于 `grammar.lalrpop`。^[atlas-core-syntax.md:101-102]

## 表达式入口与诊断适配

`parse_expression` / `parse_expression_in` 用于解析表达式。会话层通过这些入口校验 `>file` / `>>file` 的重定向体，并与活跃环境共享类型表；解析发生在输出 sink 打开之前。相关执行纪律见 [[单命令输出重定向的执行纪律]]。^[atlas-core-syntax.md:87-89]

诊断层通过 `syntax_error` 及 `bison_syntax_message`、`bison_eof_message`、`bison_expecting`、`bison_token_name`，将 LALRPOP 的 `ParseError` 转换为 Bison 风格措辞，如 `syntax error, unexpected … [expecting …]`。诊断类别维持兼容边界，具体措辞可在 oracle 比对后精化，参见 [[Bison 风格语法诊断兼容层]]。^[atlas-core-syntax.md:97-100]

## 职责与证据边界

该前端的 51 个测试覆盖表达式、命令和诊断形状。AST 节点的语义判定由 `typed.rs` 承担，词法处理位于 `lex.rs`，逐命令驱动位于 `session.rs`，重定向与包含处理位于 `session_frame.rs`。文法与上游 `parser.y` 的逐条对应属于实现方的移植陈述，语法兼容仍以 HPC 语言语料门为准；本来源的结构性阅读不构成语言验收。^[atlas-core-syntax.md:9-14, atlas-core-syntax.md:104-110]

## Sources

- [atlas-core-syntax.md](atlas-core-syntax.md) — 语法前端（syntax.rs + grammar.lalrpop）——AST 面、LALRPOP 适配与 Bison 风格诊断

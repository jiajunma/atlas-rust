---
title: Bison 风格语法诊断兼容层
summary: syntax_error 及 bison_* 辅助函数将 LALRPOP ParseError 渲染为 Bison 风格的 unexpected／expecting 消息，并保留诊断类别的兼容边界。
sources:
  - atlas-core-syntax.md
kind: concept
createdAt: "2026-10-09T14:36:30.929Z"
updatedAt: "2026-10-09T14:36:30.929Z"
tags:
  - 错误诊断
  - Bison
  - 兼容性
aliases:
  - bison-风格语法诊断兼容层
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# Bison 风格语法诊断兼容层

Bison 风格语法诊断兼容层将 LALRPOP 的 `ParseError` 转换为 Atlas 上游风格的语法错误消息，采用 `syntax error, unexpected … [expecting …]` 措辞。其兼容边界包括诊断类别；具体措辞仍可在 oracle 比对后精化。^[atlas-core-syntax.md:91-100]

## 解析适配与诊断生成

[[Atlas 语法前端与运算符优先级归约|Atlas 语法前端]]使用 LALRPOP 文法，并通过独立适配层将有状态的 Atlas 词法流转换为带源码位置的 token 流。适配层的独立性使词法器的上下文敏感规则不进入文法动作。^[atlas-core-syntax.md:9-14]

`TokenStream` 惰性地将 `Token` 序列翻译为 `(offset, ParserToken, offset)` 流。`ParserToken` 表示文法使用的 token 集，携带 `SpannedValue` 载荷；`parser_operator` 负责算符名映射，`Display` 提供文法可读名称。诊断生成由 `syntax_error` 与 `bison_syntax_message`、`bison_eof_message`、`bison_expecting`、`bison_token_name` 共同承担，将解析器错误渲染为 Bison 措辞。^[atlas-core-syntax.md:93-100]

## 命令边界与未完成输入

诊断依赖解析入口对输入状态的处理。词法器保留命令边界的控制权，适配层不预切分剩余源码，因为会话状态可能在下一条命令前改变。`parse_command` 与 `parse_command_in` 使用活跃的类型表，使词法分类能够看到先前的类型声明。^[atlas-core-syntax.md:77-82]

会话入口 `parse_command_fragment_in` 在输入未完成、允许继续输入且 `scope.has_virtual_group()` 为真时返回 `Ok(None)`。这表示解析器持有的类型规格尚未闭合：物理换行不会终止它，下一行会保留前缀重试，完整解析前不求值任何 AST。EOF 与非 EOF 语法错误仍照常诊断；这一行为与[[多行命令的增量解析]]相关。^[atlas-core-syntax.md:83-86]

## 测试与证据边界

源材料记录了 51 个覆盖表达式、命令及诊断形状的测试，但该材料仅报告结构性阅读完成，不声称语言验收。文法与上游 `parser.y` 的逐条对应属于实现方的移植陈述，语法兼容仍以 HPC 语言语料门为准；因此，采用 Bison 措辞本身不能作为完整语法兼容的验收结论。^[atlas-core-syntax.md:9-14, atlas-core-syntax.md:104-110]

AST 节点的语义判定由 `typed.rs` 负责，词法处理位于 `lex.rs`，逐命令驱动位于 `session.rs`，重定向与包含处理位于 `session_frame.rs`。这些职责划分限定了本兼容层的解释范围：它描述解析错误的呈现，而完整语言行为还涉及其他层。^[atlas-core-syntax.md:106-109]

## Sources

- [语法前端（syntax.rs + grammar.lalrpop）——AST 面、LALRPOP 适配与 Bison 风格诊断](atlas-core-syntax.md)

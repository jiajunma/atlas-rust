---
title: Atlas 词法 Token 模型
summary: TokenKind 区分关键字、原始类型、标识符、运算符、指令等类别，Token 同时保存精确源拼写 lexeme、词法解码值 value 与位置 span。
sources:
  - atlas-core-lex.md
kind: concept
createdAt: "2026-10-09T14:31:20.211Z"
updatedAt: "2026-10-09T14:31:20.211Z"
tags:
  - 词法分析
  - Token模型
  - Rust
aliases:
  - atlas-词法-token-模型
  - A词T模
confidence: 0.99
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# Atlas 词法 Token 模型

Atlas 词法 Token 模型定义了扫描器向解析器提供的词法单元、源拼写与位置数据。`lex.rs` 中的扫描器刻意保留状态，以支持嵌套注释以及解析器或 REPL 的逐 token 消费；`tokenize` 则是取得完整 token 流的兼容便利接口。^[atlas-core-lex.md:9-13]

## Token 的结构与种类

每个 `Token` 包含 `kind`、`lexeme`、`value` 和 `span`。`lexeme` 保留精确源拼写，字符串的拼写包含引号；`value` 承载解码后的内容，来源明确描述了字符串解码，并另行说明指令 token 的 `value` 携带扫描到的文件名。^[atlas-core-lex.md:21-27]

`TokenKind` 共定义 12 个变体，覆盖词语、字面量、算符、标点、指令以及流边界。^[atlas-core-lex.md:17-23]

| 变体 | 用途 |
| --- | --- |
| `Keyword(String)` | 保留字 |
| `PrimitiveType(String)` | 原始类型名 |
| `Identifier` | 标识符 |
| `Integer` | 整数字面量 |
| `String` | 字符串字面量 |
| `Operator(String)` | 算符 |
| `OperatorBecomes(String)` | `+:=` 一类算符赋值 |
| `Punctuation(char)` | 标点 |
| `Directive(DirectiveKind)` | 命令首的文件指令 |
| `Unsupported(String)` | 不支持的字符 |
| `Newline` | 可终止当前命令的换行 |
| `Eof` | 输入结束 |

## 保留字与原始类型名

`KEYWORDS` 包含 35 个保留字，涉及声明、控制流、逻辑操作及会话命令，例如 `set`、`let`、`if`、`while`、`and`、`return`、`set_type` 和 `forget`。^[atlas-core-lex.md:28-30]

原始类型名共 21 个，按 `Prim::ALL` 的顺序预先保留：`void`、`int`、`rat`、`string`、`bool`、`vec`、`mat`、`ratvec`、`LieType`、`RootDatum`、`WeylElt`、`InnerClass`、`RealForm`、`CartanClass`、`KGBElt`、`Block`、`Split`、`KType`、`KTypePol`、`Param`、`ParamPol`；即使对应值层尚未实现，也维持与上游一致的位置约定。（来源包原误记为 20 个；经对照 git base `964f0033` 与当前字节的 `PRIMITIVE_TYPES` 实际声明均为 21 个，包已更正。）^[atlas-core-lex.md:31-34]

## 算符与 token 边界

`OperatorBecomes` 将算符及其后的 `:=` 融合为一个 token，两者之间允许空白和注释。`finish_operator` 在探测到该后缀时完成融合；否则回退偏移并产生普通算符。裸 `!` 是 const 模式标记，不能作为公式算符或算符赋值的开头。^[atlas-core-lex.md:19-20, atlas-core-lex.md:55-58]

关系符采用独立规则：由 `<`、`=`、`>` 构成的极大连续段形成一个算符，且不与 `:=` 融合。单个尖括号同时承担类型参数定界符的角色，不抑制换行；`:=`、`->`、`~[`、`\%` 和 `##` 也有专门处理分支。^[atlas-core-lex.md:59-62]

## 换行与命令边界

`Newline` 仅表示能够终止当前命令的换行。词法器通过嵌套栈和续行抑制状态决定是否输出它：只有 `nesting` 与 `prevent_termination` 都为空时，换行才成为可观察 token。分组、`let` 和块结构，以及部分关键字、标点和算符，都会影响这一判断；这也是[[多行命令的增量解析]]所依赖的词法边界。^[atlas-core-lex.md:38-57]

遇到不支持的字符时，词法器清空嵌套与换行抑制状态，使下一个换行能够终止当前命令。显式调用 `recover_command` 则会丢弃当前物理行剩余内容，并重置续行、嵌套和命令首状态。^[atlas-core-lex.md:63-64, atlas-core-lex.md:78-78]

## 指令与字符串载荷

`Directive` 仅在命令首 token 位置识别，分为 `Include`（`<`，已完成则跳过）、`ForceInclude`（`<<`）、`ToFile`（`>`，截断）和 `AddToFile`（`>>`，追加）。扫描器在文件名前跳过空白与注释，允许空文件名，并保留该行剩余内容供会话帧校验，详见[[命令首指令与文件名扫描]]。^[atlas-core-lex.md:21-25, atlas-core-lex.md:68-71]

未加引号的文件名允许字母、数字和 `.-+~_=!?@#$%&|`，不允许斜杠，因此子目录路径必须加引号。^[atlas-core-lex.md:35-36]

字符串通过双写引号进行转义。换行导致字符串未闭合时，扫描器报告 `Lexical` warning，消息为 `"Closing string denotation."`，同时将恢复出的 `String` token 放入 `pending`，使解析器仍能使用该 token 和后续命令，详见[[字符串转义与未闭合恢复]]。^[atlas-core-lex.md:74-77]

## 消费接口与证据边界

[[TokenCursor 单 Token 前瞻]]提供 `peek` 与 `bump`，并将词法错误像 token 一样缓存，保证两者观察到同一结果。`tokenize` 采用全有或全无的返回方式；`tokenize_with_diagnostics` 返回 `LexOutput { tokens, diagnostics }`，按遭遇顺序保留诊断及所有可恢复 token。^[atlas-core-lex.md:79-83]

来源记录了覆盖状态机分支的 23 个测试，但这不构成语言兼容性验收。词法器不做语义判定；文法适配、逐命令消费和文件指令执行分别由 `syntax.rs`、`session.rs` 与 `session_frame.rs` 承担。上游行号对应实现方的移植陈述，词法兼容仍以 HPC 语言语料门为准。^[atlas-core-lex.md:85-91]

## Sources

- [atlas-core-lex.md](atlas-core-lex.md)

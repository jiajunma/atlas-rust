---
title: Atlas 词法 Token 模型
summary: TokenKind 区分 12 类 token，Token 保存精确源拼写、词法解码值及源码跨度，原始类型名在值层建立前即按固定顺序保留。
sources:
  - atlas-core-lex.md
kind: concept
createdAt: "2026-10-09T14:31:20.211Z"
updatedAt: "2026-10-09T22:17:19.051Z"
tags:
  - 词法分析
  - 类型系统
aliases:
  - atlas-词法-token-模型
  - A词T模
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: Atlas 词法 Token 模型
summary: TokenKind 区分关键字、原始类型、标识符、算符、指令及流边界；Token 保存精确源拼写、词法载荷与位置，有状态扫描决定换行和命令边界。
sources:
  - atlas-core-lex.md
kind: concept
tags:
  - 词法分析
  - Token模型
  - Rust
aliases:
  - atlas-词法-token-模型
provenanceState: extracted
---

# Atlas 词法 Token 模型

Atlas 的词法 Token 模型定义了 `lex.rs` 扫描器提供的词法单元及其源信息。扫描器刻意保留状态，以支持嵌套注释和解析器或 REPL 的逐 token 消费；`tokenize` 是获取完整 token 流的兼容便利接口。^[atlas-core-lex.md:9-13]

## Token 结构与类别

每个 `Token` 包含 `kind`、`lexeme`、`value` 与 `span`。`lexeme` 保留精确源拼写，字符串包含引号；`value` 用于词法解码载荷。来源在字段说明中提到字符串解码，并在指令说明中明确指出，指令 token 的 `value` 携带扫描到的文件名。^[atlas-core-lex.md:21-27]

`TokenKind` 共包含 12 个变体，涵盖词语、字面量、算符、标点、指令及流边界；其中 `Newline` 只对应能够终止当前命令的换行。^[atlas-core-lex.md:17-23, atlas-core-lex.md:40-46]

| 变体 | 含义 |
| --- | --- |
| `Keyword(String)` | 保留字 |
| `PrimitiveType(String)` | 原始类型名 |
| `Identifier` | 标识符 |
| `Integer` | 整数字面量 |
| `String` | 字符串字面量 |
| `Operator(String)` | 算符 |
| `OperatorBecomes(String)` | `+:=` 一类算符赋值 |
| `Punctuation(char)` | 标点 |
| `Directive(DirectiveKind)` | 仅在命令首识别的文件指令 |
| `Unsupported(String)` | 不支持的字符 |
| `Newline` | 可终止当前命令的换行 |
| `Eof` | 输入结束 |

## 保留字与原始类型名

`KEYWORDS` 包含 35 个保留字：`quit`、`set`、`let`、`in`、`begin`、`end`、`if`、`then`、`else`、`elif`、`fi`、`and`、`or`、`not`、`next`、`do`、`dont`、`from`、`downto`、`while`、`for`、`od`、`case`、`esac`、`rec_fun`、`true`、`false`、`die`、`break`、`return`、`set_type`、`any_type`、`whattype`、`showall`、`forget`。^[atlas-core-lex.md:28-30]

`PRIMITIVE_TYPES` 按 `Prim::ALL` 顺序保留 21 个原始类型名：`void`、`int`、`rat`、`string`、`bool`、`vec`、`mat`、`ratvec`、`LieType`、`RootDatum`、`WeylElt`、`InnerClass`、`RealForm`、`CartanClass`、`KGBElt`、`Block`、`Split`、`KType`、`KTypePol`、`Param`、`ParamPol`。这些名称在对应值层存在之前就按位置保留，以维持上游约定。^[atlas-core-lex.md:31-34]

## 算符的识别边界

`OperatorBecomes` 将算符与后续 `:=` 融合为一个 token，两者之间允许空白和注释。`finish_operator` 探测成功时完成融合并设置 `:` 续行状态；否则回退偏移并产生普通算符，注释消费失败也会回退。裸 `!` 是 const 模式标记，不能作为公式算符或算符赋值的开头。^[atlas-core-lex.md:19-20, atlas-core-lex.md:55-58]

关系符采用独立规则：由 `<`、`=`、`>` 构成的极大连续段形成一个算符，且不与 `:=` 融合。单个尖括号同时是类型参数定界符，不抑制换行。`:=` 自身设置 `:` 续行状态；`->`、`~[`、`\%` 和 `##` 各有专门分支。^[atlas-core-lex.md:59-62]

## 换行与命令边界

[[换行抑制状态机]]决定物理换行是否成为 `Newline`。`skip_space` 仅在 `nesting` 栈与 `prevent_termination` 都为空时输出换行；分组、`let`、块结构及续行 token 都会影响这一判断。`nesting` 区分 Group、Let 与 Block，相关状态还包括 `previous_termination` 和 `at_command_start`。^[atlas-core-lex.md:38-52]

算符通常抑制换行，但若前一个终止符是 `.`，算符就不再抑制，因此 `.` 加算符的序列不会吞掉换行。遇到不支持的字符时，词法器清空嵌套和抑制状态，使下一个换行可以终止命令；`recover_command` 则丢弃当前物理行剩余内容，并重置续行、嵌套和命令首状态。^[atlas-core-lex.md:53-54, atlas-core-lex.md:63-64, atlas-core-lex.md:78-78]

## 指令、字符串与注释

`Directive` 仅在命令首 token 位置识别，包括 `Include`（`<`，已完成则跳过）、`ForceInclude`（`<<`）、`ToFile`（`>`，截断）与 `AddToFile`（`>>`，追加）。扫描文件名前会跳过空白和注释，此处遇到换行表示空文件名；名字可加引号或裸写，也可为空。扫描器不消费该行剩余部分，后续由会话帧校验，详见[[命令首指令与文件名扫描]]。^[atlas-core-lex.md:21-25, atlas-core-lex.md:68-71]

裸文件名允许字母、数字及 `.-+~_=!?@#$%&|`，不包含斜杠，因此子目录路径必须加引号。^[atlas-core-lex.md:35-36]

字符串以双写引号进行转义，换行会使字符串不闭合。扫描器报告 `Lexical` warning `"Closing string denotation."`，同时将恢复出的 `String` token 放入 `pending`，使该 token 与后续命令仍可供解析器使用，详见[[字符串转义与未闭合恢复]]。^[atlas-core-lex.md:74-77]

花括号注释支持嵌套；未闭合时产生 `Lexical` 错误，消息指出注释开始的行列位置：`"Comment that started on line X, column Y is never closed."`，参见[[可嵌套注释]]。^[atlas-core-lex.md:72-73]

## 消费接口与证据边界

[[TokenCursor 单 Token 前瞻]]提供 `peek` 与 `bump`，将词法错误像 token 一样缓存，使两者观察到同一结果。`tokenize` 采用全有或全无的返回方式；`tokenize_with_diagnostics` 返回 `LexOutput { tokens, diagnostics }`，保留所有可恢复 token，并按遭遇顺序保留诊断。^[atlas-core-lex.md:79-83]

来源记录了覆盖状态机分支的 23 个测试，但属于结构性阅读，不构成语言验收。词法器不做语义判定：文法适配位于 `syntax.rs`，逐命令消费位于 `session.rs`，包含与重定向语义位于 `session_frame.rs`。上游行号引用属于实现方的移植陈述，词法兼容仍以 HPC 语言语料门为准。^[atlas-core-lex.md:9-13, atlas-core-lex.md:85-91]

## Sources

- [atlas-core-lex.md](../../sources/atlas-core-lex.md) — 有状态词法器（lex.rs）：TokenKind、换行抑制状态机及指令、字符串和注释边界。

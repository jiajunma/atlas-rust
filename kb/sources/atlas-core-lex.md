---
title: 有状态词法器（lex.rs）——TokenKind 面、换行抑制状态机、指令与字符串/注释边界
source: atlas-rust/atlas-core-lex
ingestedAt: 2026-10-09T10:20:00Z
---

# 有状态词法器（lex.rs）

编辑状态：**结构性阅读完成；维护者直接撰写（无 Kimi 调用）**。本包覆盖
`crates/atlas-core/src/lex.rs`（992 行：生产面 1–658 行 + 23 个测试）。
文件自述：扫描器**刻意有状态**——Atlas 注释可嵌套，输入可由解析器或
REPL 逐 token 消费；`tokenize` 只是要完整 token 流的兼容便利接口。
结构性阅读，不声称语言验收。

## Token 面

- `TokenKind` 12 变体：`Keyword(String)`、`PrimitiveType(String)`、
  `Identifier`、`Integer`、`String`、`Operator(String)`、
  `OperatorBecomes(String)`（`+:=` 这类 operate-assign 拼写——算符与
  `:=` 之间允许空白和注释，融合为**一个** token，lexer.w:507-516）、
  `Punctuation(char)`、`Directive(DirectiveKind)`（仅在命令首 token 识别，
  镜像上游词法器的 initial-state 规则；`value` 携带扫描到的文件名）、
  `Unsupported(String)`、`Newline`、`Eof`。
- `DirectiveKind`：`Include`（`<`，已完成则跳过）、`ForceInclude`
  （`<<`）、`ToFile`（`>`，截断）、`AddToFile`（`>>`，追加）。
- `Token`：`kind` + `lexeme`（**精确源拼写**，字符串含引号）+
  `value`（仅在有词法解码时——目前是字符串）+ `span`。
- `KEYWORDS`：35 个保留字（quit/set/let/in/begin/end/if/then/else/elif/
  fi/and/or/not/next/do/dont/from/downto/while/for/od/case/esac/rec_fun/
  true/false/die/break/return/set_type/any_type/whattype/showall/forget）。
- `PRIMITIVE_TYPES`：21 个上游原始类型名，按 `Prim::ALL` 顺序
  （void/int/rat/string/bool/vec/mat/ratvec/LieType/RootDatum/WeylElt/
  InnerClass/RealForm/CartanClass/KGBElt/Block/Split/KType/KTypePol/Param/
  ParamPol）——在值层存在**之前**就按位置保留，与上游一致。
- `FILE_NAME_CHARS = ".-+~_=!?@#$%&|"`：未加引号文件名的字母表 = 字母
  数字 + 这些字符，**不含斜杠**（子目录路径必须加引号）。

## 换行抑制状态机（核心不变量）

只有"能终止当前命令的换行"才是可观察 token；分组构造与续行 token 抑制
换行。`Lexer` 状态：`offset`、`ended`、`pending`、`nesting`
（Group/Let/Block 栈）、`prevent_termination`、`previous_termination`、
`at_command_start`。

- `skip_space`：空白/注释内部消费；换行仅在 `nesting` 为空且
  `prevent_termination` 为空时透出（否则吃掉）。
- `apply_keyword`：`let` 压 Let；`begin/if/while/for/case` 压 Block；
  `in`（栈顶是 Let 时）弹栈并置 `I`；`end/fi/od/esac` 弹栈；
  `and/or/not` 置 `~`；`whattype` 置 `W`；`set` 置 `S`；`set_type` 置
  `T`。
- `apply_punctuation`：`(`/`[` 压 Group；`)`/`]` 弹；`,`/`;`/`.`/`:`
  各置自身；`~` 置 `~`。
- `operator_termination`：若**前一个**终止符是 `.`，则算符不再抑制
  （`.`+算符序列不吞换行）；否则抑制该算符字符。
- `finish_operator`：算符后若跨空白/注释跟 `:=` 则融合为
  `OperatorBecomes` 并置 `:`（注释消费失败会回退）；否则回退偏移、发
  普通算符并带续行符。
- 裸 `!`：const-模式标记，**永不**是公式算符或 operate-assign 头。
- 关系符 `<`/`>`/`=`：`<=>` 上的极大连续段是**一个**算符
  （lexer.w:720-768）；单个尖括号同时是类型参数定界符，**不**吞换行；
  永不与 `:=` 融合。`:=` 自身置 `:`。`->`、`~[`、`\%`、`##` 各有专门
  分支。
- 不支持的字符是词法死胡同：清空嵌套与抑制状态，使下一个换行能终止
  （oracle 报语法错误并在下一换行恢复；探针 ``(` `` 后 `2`）。

## 边界：指令、注释、字符串

- `consume_directive`（命令首的 `<`/`>`）： doubled 得 Force/Add；名字前
  跳过空白**和注释**（此处换行 = 空文件名）；名字可加引号或按
  FILE_NAME_CHARS 的裸读（可为空）；**该行其余部分不消费**——会话帧
  校验后续。
- `consume_comment`：`{}` 可**嵌套**；未闭合 → Lexical 错误
  "Comment that started on line X, column Y is never closed."
- `consume_string`：引号以**双写**转义；换行使串不闭合；未闭合时报
  warning（Lexical，"Closing string denotation."）且**恢复出的 String
  token 进入 pending**——报告错误但 token 与后续命令仍可供解析器使用
  （REPL 行为）。
- `recover_command`：丢弃当前物理行其余部分并重置续行/嵌套/命令首状态。
- `TokenCursor`：`peek`/`bump` 单 token 前瞻；**词法错误像 token 一样被
  缓存**，peek 与 bump 看到同一结果。
- `tokenize`：全有或全无的兼容便利；`tokenize_with_diagnostics` 返回
  `LexOutput{tokens, diagnostics}`——诊断按遭遇序保留，所有可恢复
  token 保留。

## 测试锚点与限制

23 个测试（660–992 行）覆盖上述状态机分支。词法器本身不做语义判定；
文法适配在 `syntax.rs`（LALRPOP spanned-token 流），逐命令消费在
`session.rs`，指令的包含/重定向语义在 `session_frame.rs`（各有包）。
上游行号引用（lexer.w 等）是**实现方移植陈述**；词法兼容以 HPC 语言
语料门为准。字节数/哈希只标识本快照字节（git base `964f0033`）。

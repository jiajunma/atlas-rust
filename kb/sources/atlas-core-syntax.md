---
title: 语法前端（syntax.rs + grammar.lalrpop）——AST 面、LALRPOP 适配与 Bison 风格诊断
source: atlas-rust/atlas-core-syntax
ingestedAt: 2026-10-09T10:40:00Z
---

# 语法前端（syntax.rs + grammar.lalrpop）

编辑状态：**结构性阅读完成；维护者直接撰写（无 Kimi 调用）**。本包覆盖
`crates/atlas-core/src/syntax.rs`（4659 行：生产面 1–3261 行 + 51 个测试）
与 `grammar.lalrpop`（1096 行，LALRPOP 生成文法的源）。文件自述：文法由
LALRPOP 生成，适配层把**有状态** Atlas 词法流翻成解析器的 spanned-token
流——适配层独立意味着词法器的上下文敏感规则不会漏进文法动作。结构性
阅读，不声称语言验收。

## AST 面（Expr，每个变体带 span）

标量/容器：`Integer`/`Boolean`/`String`（值 + span）、`Tuple`、`List`、
`BarList`（`[a,b | c,d]` 行列表，parser.y:370-376：上游经隐藏
`"transpose "` 内建脱糖为 `transpose (mat: …)`；此处是独立节点，使转换
保留 oracle 的精确诊断且不受用户 `^`/`mat` 重载影响；每段非空且静态是
int 行）、`Subscription`（含 `reversed` 反向标志）、`Slice`（一维 +
`M[rlo:rhi, clo:chi]` 二维列界，省略界已由解析器填零）。

名字与赋值族：`Identifier`、`Assignment`、`MultiAssignment`
（`set pattern := value`，parser.y:264——目标是**已存在**变量，区别于
let 引入新变量）、`ComponentAssignment`（`a[i] := c`；文法限制目标为裸
标识符，`M[i,j]` 以元组下标解析、由类型分析拒绝）、
`ComponentTransform`（`a[i] op:= e`）、`FieldAssignment`/`FieldTransform`
（`p.f := e` / `p.f op:= e`）。

函数与控制：`Let`（binding_groups 分组）、`Unary`/`Binary`（仅 Not /
And/Or——其余算符走 `OperatorCall`）、`OperatorCall`（FormulaOperator +
参数）、`OperatorCast`（`f@ T (T)`：按参数类型选全局重载而不调用；显式
形式参数是**自由变量**，不是抽象）、`Call`、`Lambda`（非递归函数字面量）、
`RecLambda`（`rec_fun name(params) result: body`：声明的结果类型是体的
上下文类型，`name` 在体内绑定到闭包自身）、`Return`、`Group`、
`TypeAbstraction`（词法刚性类型变量在离开体时抽象成 scheme；该节点在
类型分析中消失，axis.w:4109）、`Conditional`（elif 在解析时脱糖为嵌套；
缺 else = void 值，parser.y:415）、`Cast`（`type: expr`，make_cast）、
`Sequence`（第一个表达式对 void 上下文求效果，序列产出第二个的值）、
`While`（每次迭代的体值收进一行；`while do…` 的条件缺省为 true）、
`Do`（while 控制树内的守卫体；其词法 let/case 帧必须**同时包住**两个
表达式，axis.w:5887-5898）、`For`/`CountedFor`、`Break`（重复 break
token：解开 `levels+1` 层循环；被解开层的当次迭代不贡献值）、`Dont`
（仅 while 的 do_expr 文法接纳）、`Die`（通过任何所需类型的分析；求值
抛 `I die`，axis.w:630）、`Case`（标签判别）、`IntCase`（0 基索引选支；
`then` 收负值、`else` 收越界、两者都缺时取模回绕）、`UnionCase`
（按变体位置应用分支函数）、`Next`（产出**第一个**的值，第二个仍求
效果）。

## Pattern / LambdaParam / TypeExpr

- `Pattern`：`Discard`（`type .` 匿名丢弃，**不是**元组模式的成员）、
  `Omitted`（空 pat_list 槽，如 `(a, , c)`：消费一个元组分量但不约束
  类型不绑名）、`Name`（`x`/`!x`/算符符号——符号标志 0x8 与常量标志
  0x4 独立，上游位域）、`Tuple`（`(p,…)` 解构；`: t`/`: !t` 另绑整体，
  文法使 `whole` 必为 `Name`）。
- `LambdaParam`：`Typed(TypedParam)`（`type pattern`）或 `Tuple` 解构。
- `TypeExpr`（parser.y:792-818）：`Variable{index}`、`Applied{name,args}`、
  `Primitive(Prim)`、`Row`、`WildRow`（`[*]` 未定分量行）、`Void`、
  `Tuple`、`Union`、`Function`、`Named`（typedef 名：单名 set_type 的别名
  或括号形式的 tabled 类型）。长度 1 的元组/联合由文法辅助折叠，镜像
  `types::Type`。

## Command 面与解析入口

`Command`：`Expression`、`Define`、`Declare`、`SetType`（只有**括号**形式
进 tabled 类型表——启用递归与 case 判别）、`Whattype`、`Forget`/
`ForgetOverload`（永不报错：未知名字只报 `not known`）、`Set`（与 let
相同的声明形式，并行绑入全局表；函数型叶子进重载表，其余进标识符表）、
`PolymorphicSet`（逗号兄弟**各自独立**执行：一个失败不回滚先前者也不
阻止后来者）、`SetOption`（`set quiet`/`set verbose`；未知选项打印
`'X' is not something one can set` 并中止命令，不动 verbosity）、
`ShowOverloads`、`ShowAll`。

入口（关键纪律：词法器仍拥有命令边界，适配层**绝不**预切分剩余源——
会话状态可在下一命令前改变）：

- `parse`（整源，测试/工具用）。
- `parse_command` / `parse_command_in`（带活跃 TypeTable——分类必须看到
  先前的类型声明；惰性流拥有命令局部词法作用域）。
- `parse_command_fragment_in`（会话用）：未完成时若 `allow_more` 且
  `scope.has_virtual_group()`（解析器持有的类型规格未闭合）→ `Ok(None)`：
  **物理换行不终止未完成的解析器持有类型规格**，下一行用保留的前缀
  重试；在完整前没有任何 AST 被求值。EOF/非 EOF 语法错误照常诊断。
- `parse_expression` / `parse_expression_in`：`>file`/`>>file` 的重定向体
  在上游是表达式（parser.y:180-181），在 sink 打开前解析
  （main.w:498-511）——会话层用它校验重定向体，且与活环境同表。

## LALRPOP 适配与 Bison 风格诊断

- `TokenStream`（1107+）：把 `Token` 序列懒译成 LALRPOP 的
  `(offset, ParserToken, offset)` 流；`ParserToken`（881+）是文法 token
  集（带 `SpannedValue` 载荷），`parser_operator` 做算符名映射，
  Display 给出文法可读名。
- `syntax_error` + `bison_syntax_message`/`bison_eof_message`/
  `bison_expecting`/`bison_token_name`：把 LALRPOP 的 ParseError 渲染成
  **Bison 措辞**（"syntax error, unexpected … [expecting …]"），诊断类别
  保持兼容边界（措辞可在 oracle 比对后精化）。
- `grammar.lalrpop`（1096 行）：生成的文法源；`syntax/type_scope.rs`
  （216 行）是解析器持有的词法类型作用域（lexer.w:464-614）。

## 测试锚点与限制

51 个测试（3262 行起）覆盖表达式/命令/诊断形状。AST 节点的语义判定在
`typed.rs`（转换+求值）；词法在 `lex.rs`；逐命令驱动在 `session.rs`；
重定向/包含在 `session_frame.rs`（各有包）。文法与上游 parser.y 的逐条
对应是**实现方移植陈述**；语法兼容以 HPC 语言语料门为准。字节数/哈希
只标识本快照字节（git base `964f0033`）。

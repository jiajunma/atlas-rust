---
title: 会话帧：文件包含、输出重定向与顶层输出面（session_frame.rs）
source: atlas-rust/atlas-core-session-frame
ingestedAt: 2026-10-09T10:05:00Z
---

# 会话帧：文件包含、输出重定向与顶层输出面（session_frame.rs）

编辑状态：**结构性阅读完成；维护者直接撰写（无 Kimi 调用）**。本包覆盖
`crates/atlas-core/src/session_frame.rs`（993 行：生产面 1–588 行 +
18 个测试）。文件自述：移植上游输入机器（buffer.w / main.w）——
`<file` 包含（含 include-once 簿记）、`<<file` 强制重读、`>file`/
`>>file` 单命令输出重定向、`Starting to read from file …`/
`Completely read file …` 框架行、`Value:` 打印与 void 抑制、包含内出错
的 abandon 级联、以及决定会话退出状态的 clean 标志。结构性阅读，不声称
语言或数学验收。

## 文件接口与重定向纪律

- `FileProvider::read(path) -> Option<String>`：CLI 用文件系统实现
  （**有损 UTF-8**——上游是字节导向的，游离字节不得变成打开失败）；
  测试用内存映射。`None` = 打开失败。
- `FileSink`（`open(path, append)`/`write_bytes`/`close`）：sink 只在
  命令体**解析成功之后、求值之前**打开（创建或截断）——语法错误不留
  文件，而求值失败仍留下带部分输出的文件（上游行为）。

## 状态与包含语义

- `SessionFrame` 持有：provider/sink、`search_path`（每条前缀已 `/`
  结尾，空前缀=cwd **最后**才试）、`completed`（include-once 的
  BTreeSet）、`active`（当前包含栈：循环守卫 + 深度 + 回溯命名）、
  `TypedContext`、`clean`、`quitting`、`next_source_id`、`registry`
  （SourceId→FileRecord：origin + 预处理后文本 + line_map）。
- `MAX_INCLUDE_DEPTH = 64`：上游至此必有问题；循环守卫管真实递归，此
  上限管病态 provider。
- 包含判定链（`include`）：typed 名已在 `completed` → 静默跳过（不打开、
  不打印）；`resolve` 失败 → Io 诊断 `failed to open input file …` +
  Abort；解析后路径已完成 → 跳过；**已在 active 栈中 → 静默跳过且算
  成功**（循环守卫）；超深 → Io 诊断 + Abort；否则打印 Starting 框架行、
  压栈、递归 `run_stream`，Finished 时记 completed 并打印 Completely。
- `resolve`：按 search_path 顺序，每条先试原名、名字不带 `.at` 时再试
  追加 `.at`。
- 上游 clean 纪律：`clean=false` 只在语法/类型/求值错误（上游 yyerror
  与求值 catch）时置位；**打开失败与 abandon 级联刻意不弄脏会话**
  （测试 `missing_file_aborts_the_include_stack_but_not_the_session`
  断言 `is_clean()` 仍真）。
- `quit` 是单 token 命令（`is_quit`）；包含内的 quit 结束**整个**会话
  （`run_top_level` 在 quitting 后不再喂流）。

## 命令执行与顶层输出面（execute）

- `SessionEvent::Value` 且非 void → 打印 `Value: <display>`；void（空
  元组）不打印。
- `ReportLine`/`ReportBytes` → 按**当前包含深度**缩进（每层两空格），
  ReportBytes 经 `SessionEvent::output` 保留字节。
- `Diagnostic`：`kind != Io` → `clean=false` 且本命令 Abort；**Io 诊断
  不弄脏**（打开失败类）。定义报告因此只在框架层缩进，输出格式集中
  一层。
- `run_redirect_command`：`>file EXPR`/`>>file EXPR` 的体先按**表达式**
  解析（parser.y:180-181 `TOFILE expr`），解析失败不建文件
  （`> "x" set qfc = 10` 在 `=` 处报 `syntax error, unexpected '='`，
  见 eval/file_commands_b9 的拒绝形状）；解析成功后 sink 才打开；
  **打开失败只留一行裸 stderr（`Failed to open {name}`）且会话保持
  clean**；命令的 Output/OutputBytes 事件写入 sink，其余事件照常上行；
  `sink.close()` 无条件。
- `abandon`：用 `lexer.offset()-1` 回退到正在读的行，经 `line_map`
  换算物理行号，发 `Abandoning reading of file '{origin}' at line
  {physical}`；各外层各自补发——级联**最内层先读**。
- 词法层诊断分流：warning（如未闭合字符串提示）只报告、不弄脏、不
  abort；真词法错误置 dirty 且（深度 >0 时）触发 abandon 级联。

## preprocess：续行与物理行映射

每行**先**剥尾空白，再把剥后形如 `\` 结尾的行与下一行直接拼接（不插入
任何内容，故 `foo\ ` 仍续行）；返回重写文本与"重写行→首个物理行"的
`line_map`（`describe`/`abandon` 用它把诊断/放弃行号映射回物理行）。
`describe_bytes` 的输出形如 `{Kind} error at {origin}:{physical}:{column}:
{msg}` + 源行 +  caret；无 span 的诊断只有裸 `{Kind} error:` 头（oracle
本来就这么裸打；差分线束的 stderr 文法以头为键）。

## 测试锚点（18 个，全部在本文件内）

字节串三件套（原始字节输出比对 `string_byte_raw_output.atlas` 的冻结
原版字节、重定向保留嵌套值字节、运行时错误与 back_trace 保留原始字节）；
包含框架行/once+force/解析顺序/缺失中止/级联内先/循环静默/quit 传播；
重定向五件套（先于求值打开、类型环境、体先解析、求值失败关 sink、打开
失败跳过且 clean）；深度缩进；void 抑制；续行；尾 token 语法错；
物理行报告。这些是**单元级**行为锚点；端到端的文件命令兼容仍以 HPC
语料门为准（eval/file_commands_b9 是唯一允许写 `/tmp` 的用例，见根
AGENTS.md 的语料清单条目）。

## 边界与限制

- 会话层外层循环与 SessionEvent 定义见 [会话包](atlas-core-session.md)；
  表达式解析在 `syntax.rs`，执行在 `typed.rs`。
- 上游行号引用（buffer.w/main.w/parser.y）是**实现方移植陈述**；端到端
  可见行为以 HPC 差分语料为准。
- 字节数/哈希只标识本快照字节（git base `964f0033`）；哈希不证明字节
  正确。

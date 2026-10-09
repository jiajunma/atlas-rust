---
title: 会话外层循环与 SessionEvent 面（session.rs）——逐命令执行、字节保留输出与回归测试库
source: atlas-rust/atlas-core-session
ingestedAt: 2026-10-09T09:55:00Z
---

# 会话外层循环与 SessionEvent 面（session.rs）

编辑状态：**结构性阅读完成；维护者直接撰写（无 Kimi 调用）**。本包覆盖
`crates/atlas-core/src/session.rs`（3889 行）：生产面只有第 1–178 行，
其余是 201 个 `#[test]` 组成的语言层回归库。文件自述定位："Stateful,
command-at-a-time Atlas execution"——Atlas 的词法分类依赖先前命令留下的
状态，因此本模块拥有外层循环且**永不**预切分整个源文件。结构性阅读，
不声称语言或数学验收。

## SessionEvent：会话层的输出类型

6 个变体（全部带 `span: SourceSpan`）：

- `Value { value, is_void_type, .. }`：唯一保留 void 标志的变体（由
  `session_event` 从 `TypedCommandEvent::Value` 的 `type_.is_void()` 提升）。
- `Output { text: String, .. }` / `ReportLine { text: String, .. }`：文本面。
- `OutputBytes { text: AtlasString, .. }` / `ReportBytes { .. }`：**字节
  保留面**——`SessionEvent::output()` 构造器先尝试 `String::from_utf8`，
  合法进 `Output`，非法则原始字节进 `OutputBytes`（值不被替换字符改写）。
  这是"字节串需要完整的值到输出边界"纪律的会话端点。
- `Diagnostic(Diagnostic)`：诊断直接透传。

## 外层循环（run_source / run_source_with_context）

`run_source` 给每个源一个全新 `TypedContext`；`run_source_with_context`
支持调用方持有的上下文。循环按 token 分流：

- `Newline` → `execute_tokens`（命令边界：换行结束当前命令缓冲）。
- `Eof` → 执行剩余缓冲并退出。
- `Unsupported(_)` → Syntax 诊断 "unexpected token" + `lexer.recover_command()`。
- `Directive(_)`（文件包含指令）→ **Io 诊断**"file inclusion is only
  available through a session frame"——会话层本身拒绝指令，包含由
  `session_frame.rs` 拥有。
- 词法 `Err(diagnostic)` → 直接入事件流；其余 token 入命令缓冲。

## 两个关键辅助

- `next_session_token`：**在消费时刻**记录标识符
  （`context.note_completion_token`），而非预切分——被包含文件与后续命令
  必须保留原始的首次使用顺序（补全顺序的语言层纪律）。
- `execute_tokens`：
  - `allow_more` = 终止符是 `Newline`；`parse_command_fragment_in(..,
    context.types(), allow_more)` 返回 `Ok(None)` 时**保留前缀、不求值、
    无诊断**——多行命令在同一缓冲上累积（raw 与文件会话同此路径）。
  - `Command::SetType` 的 span 用**真实词法终止符**重建：上游 parser.y 把
    命令换行计入 set_type 的 `@$`；本实现按终止符行列重算（Newline 时列
    +1），不猜语义 token 之后的列。
  - `context.execute` 出错时先 `drain_failed_printed(span)` 再发诊断：
    上游求值中途的 stdout 写在运行时错误后仍然存活（ext_kl.cpp:947 的
    顺序）——先排空已打印内容，再落诊断。

## 回归测试库（179–3889 行，201 个测试）

按名字前缀的最大族：`named`（10）、`while`（8）、`polynomial`/`operator`/
`generic`/`for`（各 7）、`weyl`/`torus`/`returns`（各 6）、`completion`/
`byte`（各 5）、`recursive`/`psp4`/`overload`/`matrix`（各 4）等。HPC
差分发现的每个差异先在这里落成**原版背书**的回归（硬规则 7）：范式是
`include_str!` 载入 `tests/math/generics/` 的 fixture + 逐字节比对
`.oracle.stdout`/`.oracle.stderr`——例如
`weyl_context_core_cold_dual_original`（796 行起）绑定 A1 cold_dual 的
冻结 goldens。这些测试的可执行正确性由 HPC 门验证（本快照字节 =
632 项清单中的同一 session.rs，见下方交叉引用）。

## 边界与限制

- 转换与求值在 `typed.rs`；文件包含/输出重定向在 `session_frame.rs`；
  词法在 `lex.rs`、文法适配在 `syntax.rs`；领域内建在
  `domain_builtins.rs`——各待自己的包。
- 文件头与注释中的上游行号（axis.w/ext_kl.cpp 等）是**实现方的移植
  陈述**，其语义等值以 HPC 差分门为准：Weyl owner/dual 语义的 A1 限定
  验收见 `tests/reference/hpc/math_weyl_context_core_after_v5_acceptance_2026_10_06.json`。
- 测试名/家族计数只标识本快照字节（git base `964f0033`，session.rs
  sha256 `969cdb27…`，与已验收的 after-v5 清单一致）；哈希不证明字节
  正确。

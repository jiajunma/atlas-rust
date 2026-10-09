---
title: SessionFrame 会话帧与文件包含语义
summary: SessionFrame 通过完成集与活动栈实现 include-once、强制重读、循环静默跳过、64 层深度限制及包含内 quit 的全会话传播。
sources:
  - atlas-core-session-frame.md
kind: concept
createdAt: "2026-10-09T14:34:07.962Z"
updatedAt: "2026-10-09T22:19:11.501Z"
tags:
  - 会话管理
  - 文件包含
aliases:
  - sessionframe-会话帧与文件包含语义
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: SessionFrame 会话帧与文件包含语义
summary: SessionFrame 管理文件搜索、include-once 簿记和包含栈，支持强制重读、循环静默跳过、64 层深度限制及包含内 quit 的全会话传播，并区分包含中止与 clean 状态。
sources:
  - atlas-core-session-frame.md
kind: concept
tags:
  - 会话管理
  - 文件包含
  - Rust
aliases:
  - sessionframe-会话帧与文件包含语义
provenanceState: extracted
---

# SessionFrame 会话帧与文件包含语义

`SessionFrame` 是文件包含、单命令输出重定向和顶层输出的管理层，移植上游 `buffer.w` / `main.w` 输入机器的相关行为。它支持 `<file` 的 include-once 簿记、`<<file` 强制重读、包含开始与完成提示、出错后的 abandon 级联，以及影响会话退出状态的 `clean` 标志。^[atlas-core-session-frame.md:9-16]

## 状态与读取接口

会话帧通过 `completed` 和 `active` 分别记录已完成的包含与当前包含栈。`completed` 是用于 include-once 的 `BTreeSet`；`active` 用于循环守卫、深度记录和回溯命名。其他状态包括 `provider`、`sink`、`search_path`、`TypedContext`、`clean`、`quitting`、`next_source_id`，以及将 `SourceId` 映射到来源、预处理文本和 `line_map` 的注册表。^[atlas-core-session-frame.md:29-33]

`FileProvider::read(path) -> Option<String>` 以 `None` 表示打开失败。CLI 的文件系统实现采用有损 UTF-8 解码，避免将游离字节变成打开失败；测试使用内存映射实现。^[atlas-core-session-frame.md:20-22]

## 文件搜索与包含判定

搜索路径的非空前缀以 `/` 结尾，代表当前工作目录的空前缀最后尝试。`resolve` 按搜索路径顺序查找，每条路径下先试原名；名称不带 `.at` 时，再试追加 `.at` 的名称。详见 [[包含文件的搜索路径解析]]。^[atlas-core-session-frame.md:29-30, atlas-core-session-frame.md:41-42]

普通包含首先检查输入的文件名是否已在 `completed` 中；若已完成，则静默跳过，不打开文件，也不打印提示。否则调用 `resolve`：解析失败产生 `failed to open input file …` 的 Io 诊断并返回 Abort；解析成功后，再检查解析所得路径是否已经完成。`<<file` 则提供强制重读入口。^[atlas-core-session-frame.md:12-13, atlas-core-session-frame.md:36-40]

通过完成状态检查后，若文件已在 `active` 栈中，则静默跳过并视为成功。随后检查 `MAX_INCLUDE_DEPTH = 64` 的深度限制；超限产生 Io 诊断并返回 Abort。循环守卫处理真实递归，深度上限约束病态 provider。^[atlas-core-session-frame.md:34-40]

实际进入文件时，会话帧打印 `Starting to read from file …`，压入包含栈，并递归调用 `run_stream`。返回 Finished 时，才将文件记入 `completed` 并打印 `Completely read file …`。^[atlas-core-session-frame.md:13-14, atlas-core-session-frame.md:39-40]

## 包含中止、clean 与退出

包含中止与会话是否保持 `clean` 是不同状态。语法、类型和求值错误会将 `clean` 置为 `false`；打开失败与 abandon 级联本身不弄脏会话。测试 `missing_file_aborts_the_include_stack_but_not_the_session` 明确断言，缺失文件导致包含栈中止后，`is_clean()` 仍为真。参见 [[会话 clean 标志与诊断分流]]。^[atlas-core-session-frame.md:43-46]

执行层收到非 Io 的 `Diagnostic` 时，会设置 `clean=false` 并使当前命令 Abort；Io 诊断不弄脏会话。词法 warning 只报告，不弄脏会话，也不触发 Abort；真正的词法错误会置 dirty，并在包含深度大于零时触发 abandon 级联。^[atlas-core-session-frame.md:56-58, atlas-core-session-frame.md:69-70]

abandon 使用 `lexer.offset()-1` 回退到正在读取的行，通过 `line_map` 换算物理行号，再输出 `Abandoning reading of file '{origin}' at line {physical}`。各外层包含分别补发提示，形成由最内层向外的 [[嵌套包含的 abandon 级联]]。^[atlas-core-session-frame.md:66-68]

`quit` 是由 `is_quit` 识别的单 token 命令。在包含文件内部执行 `quit` 会结束整个会话；`quitting` 置位后，`run_top_level` 不再继续喂入输入流。^[atlas-core-session-frame.md:47-48]

## 续行与物理行映射

预处理先剥除每行尾部空白，再将以反斜杠 `\` 结尾的行与下一行直接拼接，不插入任何内容。因此，反斜杠后带尾部空格仍可续行。返回的 `line_map` 将重写后的行映射到首个物理行，供 `describe` 和 abandon 恢复物理行号。^[atlas-core-session-frame.md:74-76]

带 span 的诊断输出错误种类、来源、物理行号、列号和消息，并附源行及 caret 指示符；无 span 的诊断只有裸 `{Kind} error:` 头。^[atlas-core-session-frame.md:77-79]

## 顶层输出与重定向

非 void 的 `SessionEvent::Value` 打印为 `Value: <display>`；void（空元组）不打印。`ReportLine` 和 `ReportBytes` 按当前包含深度缩进，每层两空格；`ReportBytes` 经 `SessionEvent::output` 保留字节。参见 [[SessionEvent 与字节保留输出]]。^[atlas-core-session-frame.md:52-58]

[[单命令输出重定向的执行纪律]] 要求 `>file EXPR` 和 `>>file EXPR` 的命令体先按表达式解析，解析成功后、求值之前才打开 sink。语法错误不会创建文件，求值失败仍会留下文件，其中可能已有部分输出。^[atlas-core-session-frame.md:23-25, atlas-core-session-frame.md:59-62]

重定向打开失败只输出一行裸 stderr：`Failed to open {name}`，会话保持 clean。命令的 `Output` / `OutputBytes` 事件写入 sink，其余事件照常上行；`sink.close()` 无条件执行。^[atlas-core-session-frame.md:63-65]

## 测试与证据边界

文件内的 18 个测试覆盖包含框架行、once 与 force、解析顺序、缺失文件中止、由内向外的级联、循环静默跳过、quit 传播，以及重定向、字节保留、深度缩进、void 抑制、续行、尾 token 语法错误和物理行报告。这些属于单元级行为锚点；端到端文件命令兼容仍以 HPC 语料门为准，结构性阅读不构成语言或数学验收。^[atlas-core-session-frame.md:9-16, atlas-core-session-frame.md:81-90]

会话外层循环与 `SessionEvent` 定义属于会话层，表达式解析位于 `syntax.rs`，执行位于 `typed.rs`，可结合 [[会话循环与文件会话帧的职责边界]] 阅读。来源中的上游行号属于实现方的移植陈述，端到端可见行为仍以 HPC 差分语料为依据。^[atlas-core-session-frame.md:94-97]

## Sources

- [会话帧：文件包含、输出重定向与顶层输出面（session_frame.rs）](../../sources/atlas-core-session-frame.md)

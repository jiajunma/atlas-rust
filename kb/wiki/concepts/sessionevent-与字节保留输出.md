---
title: SessionEvent 与字节保留输出
summary: SessionEvent 承载值、文本、原始字节与诊断；输出构造器按 UTF-8 合法性分流并保留非法字节，值事件另保存 void 类型标志。
sources:
  - atlas-core-session.md
kind: concept
createdAt: "2026-10-09T14:34:38.036Z"
updatedAt: "2026-10-09T21:16:23.662Z"
tags:
  - 会话事件
  - 字节保真
aliases:
  - sessionevent-与字节保留输出
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: SessionEvent 与字节保留输出
summary: SessionEvent 承载会话执行产生的值、文本、原始字节与诊断；输出构造器按 UTF-8 有效性分流，保留非法 UTF-8 的原始字节，并在运行时失败时先发送已打印内容再发送诊断。
sources:
  - atlas-core-session.md
kind: concept
tags:
  - 事件模型
  - 字节串
  - 输出保真
aliases:
  - sessionevent-与字节保留输出
---

# SessionEvent 与字节保留输出

`SessionEvent` 是 `session.rs` 的会话层输出类型，承载逐命令执行产生的值、文本、原始字节与诊断。其字节保留机制使非法 UTF-8 输出保持原始内容，构成字节串从值到输出边界的会话端点。^[atlas-core-session.md:16-27]

## 事件类型

`SessionEvent` 有六个变体，全部携带 `span: SourceSpan`。`Value { value, is_void_type, .. }` 是唯一保留 void 类型标志的变体；转换函数 `session_event` 从 `TypedCommandEvent::Value` 的 `type_.is_void()` 提取该标志。^[atlas-core-session.md:18-21]

`Output { text: String, .. }` 与 `ReportLine { text: String, .. }` 承载文本；`OutputBytes { text: AtlasString, .. }` 与 `ReportBytes { .. }` 提供字节保留面。`Diagnostic(Diagnostic)` 则直接透传诊断。^[atlas-core-session.md:22-27]

## UTF-8 分流与字节保留

`SessionEvent::output()` 先尝试使用 `String::from_utf8` 转换输出内容：合法 UTF-8 进入 `Output`，非法 UTF-8 则将原始字节放入 `OutputBytes`，不使用替换字符改写原值。这是 [[AtlasString 字节保留串与原始字节打印]] 在会话输出边界的具体处理规则。^[atlas-core-session.md:23-26]

## 运行时失败时的输出顺序

执行命令时，如果 `context.execute` 返回错误，`execute_tokens` 会先调用 `drain_failed_printed(span)`，排空已经打印的内容，然后才发送诊断。因此，求值中途产生的 stdout 在后续运行时错误发生后仍被保留，事件顺序为“已打印内容在前、诊断在后”。参见 [[运行时失败后的已打印输出保留]]。^[atlas-core-session.md:54-56]

## 职责边界

会话外层循环负责[[有状态的逐命令会话执行]]，由于词法分类依赖先前命令留下的状态，不会预先切分整个源文件。转换与求值属于 `typed.rs`，文件包含与输出重定向属于 `session_frame.rs`。^[atlas-core-session.md:9-14, atlas-core-session.md:72-74]

会话循环本身遇到文件包含指令时，会产生 Io 诊断 `"file inclusion is only available through a session frame"`，要求通过会话帧处理。这一区分见 [[会话循环与文件会话帧的职责边界]]。^[atlas-core-session.md:37-39]

## 测试与证据范围

来源所述 `session.rs` 快照包含 201 个语言层回归测试，其中 `byte` 前缀族有 5 个测试。回归库采用 `include_str!` 载入 fixture 并逐字节比对 `.oracle.stdout` 与 `.oracle.stderr` 的范式，HPC 差分发现的差异须先形成原版背书的回归。相关主题见 [[原版背书的语言层回归测试]]。^[atlas-core-session.md:58-68]

源包属于结构性阅读，本身不构成语言或数学验收；测试的可执行正确性由 HPC 门验证。测试名称与计数只描述特定快照，哈希也不能证明字节正确；实现注释中的上游对应关系属于移植陈述，语义等值仍以 HPC 差分门为准。^[atlas-core-session.md:9-14, atlas-core-session.md:67-68, atlas-core-session.md:75-80]

## Sources

- [atlas-core-session.md](../../sources/atlas-core-session.md) — 会话外层循环与 SessionEvent 面（session.rs）。

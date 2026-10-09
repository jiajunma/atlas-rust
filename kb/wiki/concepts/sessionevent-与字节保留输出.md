---
title: SessionEvent 与字节保留输出
summary: SessionEvent 统一承载值、文本、原始字节和诊断；输出构造器将合法 UTF-8 转为文本，将非法 UTF-8 保留为 AtlasString，值事件另保留 void 类型标志。
sources:
  - atlas-core-session.md
kind: concept
createdAt: "2026-10-09T14:34:38.036Z"
updatedAt: "2026-10-09T14:34:38.036Z"
tags:
  - 事件模型
  - 字节串
  - 输出保真
aliases:
  - sessionevent-与字节保留输出
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# SessionEvent 与字节保留输出

`SessionEvent` 是 `session.rs` 的会话层输出类型，用于承载逐命令执行产生的值、文本、原始字节与诊断。它的字节保留机制使非法 UTF-8 输出仍能保持原始内容，构成字节串从值到输出边界的会话端点。^[atlas-core-session.md:16-27]

## 事件类型

`SessionEvent` 有六个变体，全部携带 `span: SourceSpan`。`Value` 是唯一保留 void 标志的变体；转换函数 `session_event` 从 `TypedCommandEvent::Value` 的 `type_.is_void()` 提取该标志。相关类型参见 [[TypedCommandEvent 类型化命令事件]]。^[atlas-core-session.md:18-21]

`Output` 与 `ReportLine` 承载 `String` 文本；`OutputBytes` 与 `ReportBytes` 提供字节保留面，其中 `OutputBytes` 的 `text` 类型为 `AtlasString`。`Diagnostic(Diagnostic)` 则直接透传诊断。^[atlas-core-session.md:22-27]

## UTF-8 分流与字节保留

`SessionEvent::output()` 先尝试通过 `String::from_utf8` 转换输出内容：合法 UTF-8 进入 `Output`，非法 UTF-8 则将原始字节保存在 `OutputBytes` 中，不使用替换字符改写原值。这一行为与 [[AtlasString 字节保留串与原始字节打印]] 共同构成字节保留输出的边界契约。^[atlas-core-session.md:23-26]

## 运行时错误前的输出顺序

执行命令时，如果 `context.execute` 返回错误，`execute_tokens` 会先调用 `drain_failed_printed(span)`，排空已经打印的内容，然后才发送诊断。这样，求值中途产生的 stdout 不会因后续运行时错误而丢失，事件顺序也保留为“已打印内容在前、诊断在后”。相关约定见 [[打印、字符串转换与运行时错误的输出契约]]。^[atlas-core-session.md:54-56]

## 职责与证据边界

`SessionEvent` 位于会话输出边界；转换与求值由 `typed.rs` 负责，文件包含与输出重定向由 `session_frame.rs` 负责。会话循环本身遇到文件包含指令时产生 Io 诊断，要求通过会话帧处理，参见 [[会话循环与文件会话帧的职责边界]]。^[atlas-core-session.md:37-39, atlas-core-session.md:72-74]

该源码快照包含 201 个语言层回归测试，其中 `byte` 前缀族有 5 个测试。回归库采用载入 fixture 并逐字节比对 `.oracle.stdout` 与 `.oracle.stderr` 的范式，可执行正确性由 HPC 门验证。源包的结构性阅读本身不构成语言或数学验收；测试计数与哈希也仅标识特定快照，不能单独证明正确性。^[atlas-core-session.md:9-14, atlas-core-session.md:58-68, atlas-core-session.md:78-80]

## Sources

- [atlas-core-session.md](atlas-core-session.md)

---
title: CLI 事件输出与源码诊断
summary: print_events 将文本和原始字节事件分别输出到 stdout，并通过 frame.describe_bytes 将含出处、源行和 caret 的诊断输出到 stderr。
sources:
  - atlas-cli-main.md
kind: concept
createdAt: "2026-10-09T14:25:20.435Z"
updatedAt: "2026-10-09T20:29:21.634Z"
tags:
  - CLI
  - 诊断
  - 字节保真
aliases:
  - cli-事件输出与源码诊断
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: CLI 事件输出与源码诊断
summary: print_events 将文本和原始字节事件输出到 stdout，通过 frame.describe_bytes 将带出处、源行和 caret 的诊断输出到 stderr；值通常已由会话帧渲染为文本。
sources:
  - atlas-cli-main.md
kind: concept
tags:
  - 事件处理
  - 字节输出
  - 诊断
aliases:
  - cli-事件输出与源码诊断
---

# CLI 事件输出与源码诊断

CLI 前端通过 `print_events` 将会话事件分别输出到 stdout 和 stderr：文本与原始字节走 stdout，诊断附带出处和源码定位信息写入 stderr。会话机器位于 `atlas-core`，CLI 负责这些事件的终端输出。^[atlas-cli-main.md:22-25, atlas-cli-main.md:41-43]

## 事件输出

`Output` 与 `ReportLine` 使用 `print!` 输出；`OutputBytes` 与 `ReportBytes` 则将原始字节写入 stdout。这一区分保留了字节事件的原始内容，相关概念见 [[SessionEvent 与字节保留输出]]。^[atlas-cli-main.md:22-25]

顶层值以 `Value: …` 的形式打印。会话帧已将值渲染为 `Output` 文本，因此 `print_events` 中的 `Value` 分支属于防御性处理。定义与包含操作的框架文字逐字输出，会话以 `Bye.` 结束。^[atlas-cli-main.md:10-13, atlas-cli-main.md:24-25]

## 源码诊断

`Diagnostic` 事件通过 `frame.describe_bytes` 提供出处、源码行和 caret 定位标记，再输出到 stderr。诊断所需的源码上下文由会话帧提供，可结合 [[SessionFrame 会话帧与文件包含语义]] 阅读。^[atlas-cli-main.md:22-25]

退出状态遵循会话的 clean 标志：语法、类型和运行时错误会导致非零退出状态，但缺少包含文件本身不会将 clean 标志置为不干净；`quit` 可以提前结束会话。相关规则见 [[clean 标志与 CLI 退出状态]]。^[atlas-cli-main.md:35-37]

## 证据边界

本页依据对 `crates/atlas-cli/src/main.rs` 全部 175 行的结构性阅读，不代表语言验收。该文件没有自身测试，行为由 HPC 语料门覆盖；CLI 兼容性以该语料门为准。来源中的上游行号引用属于实现方的移植陈述，快照字节数与哈希仅用于标识 Git base `964f0033` 对应的快照字节。^[atlas-cli-main.md:9-14, atlas-cli-main.md:41-45]

## Sources

- [atlas-cli-main.md](../../sources/atlas-cli-main.md) — CLI 前端（atlas-cli/main.rs）——会话帧驱动、--path 解析与 clean 退出状态。

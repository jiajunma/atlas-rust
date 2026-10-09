---
title: CLI 事件输出与源码诊断
summary: print_events 将文本和原始字节事件分别输出到 stdout，并通过 frame.describe_bytes 将含出处、源行和 caret 的诊断输出到 stderr；值通常已由帧渲染为文本。
sources:
  - atlas-cli-main.md
kind: concept
createdAt: "2026-10-09T14:25:20.435Z"
updatedAt: "2026-10-09T14:25:20.435Z"
tags:
  - 事件处理
  - 字节输出
  - 诊断
aliases:
  - cli-事件输出与源码诊断
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# CLI 事件输出与源码诊断

CLI 前端通过 `print_events` 将会话事件输出到 stdout 或 stderr：普通文本和原始字节输出走 stdout，诊断则附带来源与源码定位信息写入 stderr。会话机器位于 `atlas-core`，CLI 承担事件的终端输出职责。^[atlas-cli-main.md:22-25, atlas-cli-main.md:41-43]

## 事件输出

`Output` 与 `ReportLine` 使用 `print!` 输出；`OutputBytes` 与 `ReportBytes` 则将原始字节直接写入 stdout。这一区分保留了字节事件的原始内容，可结合 [[SessionEvent 与字节保留输出]] 理解。^[atlas-cli-main.md:22-25]

顶层值以 `Value: …` 的形式打印，但会话帧已将值渲染为 `Output` 文本，因此 `print_events` 中的 `Value` 分支仅作防御性处理。定义与包含操作的框架文字按原样输出，会话以 `Bye.` 结束。^[atlas-cli-main.md:10-13, atlas-cli-main.md:24-25]

## 源码诊断

`Diagnostic` 事件通过 `frame.describe_bytes` 生成诊断内容，其中包含出处、源码行及 caret 定位标记，再写入 stderr。诊断格式依赖会话帧提供的源码上下文，相关机制见 [[SessionFrame 会话帧与文件包含语义]]。^[atlas-cli-main.md:22-25]

诊断输出与退出状态还需结合 [[clean 标志与 CLI 退出状态]] 理解：语法、类型及运行时错误会使会话以非零状态退出，但缺少包含文件本身不设置这一错误状态；`quit` 可以提前结束会话。^[atlas-cli-main.md:35-37]

## 证据边界

本页依据对 `crates/atlas-cli/src/main.rs` 全部 175 行的结构性阅读，不代表语言验收。该文件没有自身测试，行为由 HPC 语料门覆盖；CLI 兼容性应以该语料门为准，上游行号引用仅属于实现方的移植陈述。相关限制见 [[CLI 兼容性的证据边界]]。^[atlas-cli-main.md:9-14, atlas-cli-main.md:41-45]

## Sources

- [atlas-cli-main.md](atlas-cli-main.md) — CLI 前端（atlas-cli/main.rs）——会话帧驱动、--path 解析与 clean 退出状态。

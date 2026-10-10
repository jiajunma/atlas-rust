---
title: CLI 事件输出与源码诊断
summary: print_events 将文本及原始字节事件输出到 stdout，并通过 frame.describe_bytes 将含出处、源行和 caret 的诊断输出到 stderr。
sources:
  - atlas-cli-main.md
kind: concept
createdAt: "2026-10-09T14:25:20.435Z"
updatedAt: "2026-10-10T00:13:56.471Z"
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
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: CLI 事件输出与源码诊断
summary: print_events 将文本和原始字节事件输出到 stdout，通过 frame.describe_bytes 将带出处、源行和 caret 的诊断输出到 stderr；值通常已由会话帧渲染为文本。
sources:
  - atlas-cli-main.md
kind: concept
tags:
  - CLI
  - 诊断
  - 字节保真
aliases:
  - cli-事件输出与源码诊断
---

# CLI 事件输出与源码诊断

CLI 前端的 `print_events` 按事件类型分配输出：文本和原始字节写入 stdout，带源码定位信息的诊断写入 stderr。会话机器位于 `atlas-core`，CLI 承担事件输出职责。^[atlas-cli-main.md:22-25, atlas-cli-main.md:41-43]

## 文本与原始字节输出

`Output` 和 `ReportLine` 使用 `print!` 输出；`OutputBytes` 和 `ReportBytes` 则将原始字节写入 stdout。文本事件与字节事件采用不同的输出路径，相关概念见 [[SessionEvent 与字节保留输出]]。^[atlas-cli-main.md:22-25]

顶层值以 `Value: …` 的形式打印。会话帧已将值渲染为 `Output` 文本，因此 `print_events` 中的 `Value` 分支属于防御性处理。定义与包含操作的框架文字逐字输出，会话以 `Bye.` 结束。^[atlas-cli-main.md:10-13, atlas-cli-main.md:24-25]

## 源码诊断与退出状态

`Diagnostic` 事件经 `frame.describe_bytes` 生成包含出处、源行和 caret 定位标记的诊断，再写入 stderr。会话帧的相关机制见 [[SessionFrame 会话帧与文件包含语义]]。^[atlas-cli-main.md:22-25]

退出状态遵循会话的 clean 标志：语法、类型和运行时错误会导致非零退出状态，但缺少包含文件本身不会将 clean 标志置为不干净；`quit` 可以提前结束会话。相关规则见 [[clean 标志与 CLI 退出状态]]。^[atlas-cli-main.md:35-37]

## 文件输入的编码边界

文件输入使用 `FsProvider` 的有损 UTF-8 处理，游离的非 UTF-8 字节不得被当作打开失败；输出端的 `OutputBytes` 和 `ReportBytes` 则直接写出原始字节。这一输入与输出约定限定了此处“字节保真”的范围。^[atlas-cli-main.md:18-19, atlas-cli-main.md:22-25]

## 证据边界

来源覆盖 `crates/atlas-cli/src/main.rs` 全部 175 行，属于结构性阅读，不声称语言验收。该文件没有自身测试，行为由 HPC 语料门覆盖；CLI 兼容性以该语料门为准。来源中的上游行号引用属于实现方的移植陈述，字节数与哈希仅标识 Git base `964f0033` 对应的快照字节。参见 [[CLI 兼容性的证据边界]]。^[atlas-cli-main.md:9-14, atlas-cli-main.md:41-45]

## Sources

- [atlas-cli-main.md](../../sources/atlas-cli-main.md) — CLI 前端（atlas-cli/main.rs）——会话帧驱动、--path 解析与 clean 退出状态。

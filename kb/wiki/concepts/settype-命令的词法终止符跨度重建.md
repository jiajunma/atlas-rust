---
title: SetType 命令的词法终止符跨度重建
summary: SetType 的 SourceSpan 根据真实词法终止符重建，并在换行终止时将列号加一，以保留上游将命令换行计入跨度的行为。
sources:
  - atlas-core-session.md
kind: concept
createdAt: "2026-10-09T14:34:56.697Z"
updatedAt: "2026-10-09T14:34:56.697Z"
tags:
  - 源码定位
  - 语法兼容
  - 类型命令
aliases:
  - settype-命令的词法终止符跨度重建
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# SetType 命令的词法终止符跨度重建

`Command::SetType` 的源码跨度（span）在会话层的 `execute_tokens` 中按**真实词法终止符**重建。其兼容依据是上游 `parser.y` 将命令换行计入 `set_type` 的 `@$`；因此，跨度末端不能通过语义 token 之后的位置猜测。^[atlas-core-session.md:47-53]

## 重建规则

重建使用终止符的实际行、列位置；终止符为 `Newline` 时，列位置加 1，以体现命令换行计入跨度的约定。这使 `SetType` 的跨度与词法边界对应，而不是仅由命令中的语义 token 决定。^[atlas-core-session.md:51-53]

## 与逐命令执行的关系

会话外层循环遇到 `Newline` 时调用 `execute_tokens`，遇到 `Eof` 时执行剩余缓冲并退出。换行与文件结束因此都是命令缓冲进入执行路径的边界；`SetType` 的跨度重建在该路径中使用实际终止符。相关执行模型见 [[会话循环与文件会话帧的职责边界]]。^[atlas-core-session.md:29-39, atlas-core-session.md:47-53]

换行也可能只是尚未完整的命令片段的边界：当终止符为 `Newline` 时，`allow_more` 为真；若 `parse_command_fragment_in` 返回 `Ok(None)`，会话保留缓冲前缀，不求值、不发出诊断，继续累积多行命令。原始源输入与文件会话使用同一路径，参见 [[多行命令的增量解析]]。^[atlas-core-session.md:47-50]

## 证据边界

本页描述的是来源包结构性阅读记录的实现规则，不构成语言或数学验收。来源明确区分移植陈述与经 HPC 差分门验证的语义等值；其中引用的 Weyl owner/dual A1 限定验收也不能扩大解释为本规则的独立验收结论。^[atlas-core-session.md:9-14, atlas-core-session.md:75-80]

## Sources

- [atlas-core-session.md](atlas-core-session.md) — 会话外层循环与 SessionEvent 面（session.rs）。

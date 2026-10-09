---
title: SetType 命令的词法终止符跨度重建
summary: SetType 的 SourceSpan 根据真实词法终止符重建，并在换行终止时将列号加一，以保留上游将命令换行计入跨度的行为。
sources:
  - atlas-core-session.md
kind: concept
createdAt: "2026-10-09T14:34:56.697Z"
updatedAt: "2026-10-09T21:16:26.666Z"
tags:
  - 源码跨度
  - 语法兼容
aliases:
  - settype-命令的词法终止符跨度重建
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: SetType 命令的词法终止符跨度重建
summary: SetType 的源码跨度按真实词法终止符的行列位置重建，换行终止时列号加一，以保留上游将命令换行计入跨度的约定。
sources:
  - atlas-core-session.md
kind: concept
tags:
  - 源码定位
  - 语法兼容
  - 类型命令
aliases:
  - settype-命令的词法终止符跨度重建
---

# SetType 命令的词法终止符跨度重建

`Command::SetType` 的源码跨度（span）由会话层的 `execute_tokens` 按**真实词法终止符**重建。来源记录的兼容依据是：上游 `parser.y` 将命令换行计入 `set_type` 的 `@$`，因此实现使用终止符的位置，不猜测语义 token 之后的列号。^[atlas-core-session.md:47-53]

## 重建规则

重建依据终止符的实际行、列位置；当终止符为 `Newline` 时，列号加 1，以保留命令换行计入跨度的行为。这里的关键是使用词法终止符提供的边界信息。^[atlas-core-session.md:51-53]

## 与命令执行边界的关系

会话外层循环遇到 `Newline` 时调用 `execute_tokens`，遇到 `Eof` 时执行剩余命令缓冲并退出。`SetType` 的跨度重建属于这一逐命令处理路径，相关职责划分见 [[会话循环与文件会话帧的职责边界]]。^[atlas-core-session.md:29-39, atlas-core-session.md:47-53]

换行并不总意味着命令已经完整。当终止符为 `Newline` 时，`allow_more` 为真；若 `parse_command_fragment_in(.., context.types(), allow_more)` 返回 `Ok(None)`，会话保留已有前缀，不求值、不发出诊断，继续在同一缓冲中累积命令。原始源输入与文件会话共用这条路径，参见 [[多行命令的增量解析]]。^[atlas-core-session.md:47-50]

## 证据边界

上述规则来自 `session.rs` 的结构性阅读记录，不构成语言或数学验收。来源将上游位置及对应关系视为实现方的移植陈述，语义等值须以 HPC 差分门为准；其中列出的 Weyl owner/dual A1 限定验收不提供本规则的独立验收结论。^[atlas-core-session.md:9-14, atlas-core-session.md:75-80]

## Sources

- [atlas-core-session.md](../../sources/atlas-core-session.md) — 会话外层循环与 SessionEvent 面（session.rs）。

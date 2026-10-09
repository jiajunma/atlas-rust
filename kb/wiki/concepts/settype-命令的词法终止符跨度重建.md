---
title: SetType 命令的词法终止符跨度重建
summary: SetType 的源码跨度依据真实词法终止符重建，换行终止时列号加一，以保留上游将命令换行计入跨度的行为。
sources:
  - atlas-core-session.md
kind: concept
createdAt: "2026-10-09T14:34:56.697Z"
updatedAt: "2026-10-09T22:19:40.086Z"
tags:
  - 源码位置
  - 兼容契约
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
  - 源码跨度
  - 语法兼容
  - 类型命令
aliases:
  - settype-命令的词法终止符跨度重建
---

# SetType 命令的词法终止符跨度重建

`Command::SetType` 的源码跨度（span）由会话层的 `execute_tokens` 按**真实词法终止符**重建。兼容依据是上游 `parser.y` 将命令换行计入 `set_type` 的 `@$`；因此，实现使用终止符的实际位置，而不猜测语义 token 之后的列号。^[atlas-core-session.md:47-53]

## 重建规则

重建使用终止符的行、列位置；当终止符为 `Newline` 时，列号加 1，以保留命令换行计入跨度的行为。^[atlas-core-session.md:51-53]

## 与命令解析边界的关系

会话外层循环遇到 `Newline` 时调用 `execute_tokens`，遇到 `Eof` 时执行剩余命令缓冲并退出。跨度重建位于这一逐命令处理路径中；Atlas 的词法分类依赖先前命令留下的状态，因此会话层不预切分整个源文件。^[atlas-core-session.md:11-14, atlas-core-session.md:29-35, atlas-core-session.md:47-53]

换行触发解析，但并不保证命令已经完整。当终止符为 `Newline` 时，`allow_more` 为真；若 `parse_command_fragment_in(.., context.types(), allow_more)` 返回 `Ok(None)`，会话保留前缀，不求值、不发出诊断，继续在同一缓冲中累积命令。原始源输入与文件会话共用这条路径，参见 [[多行命令的增量解析]]。^[atlas-core-session.md:47-50]

文件包含由 `session_frame.rs` 负责，会话层本身遇到包含指令会发出 Io 诊断。该职责边界与 `execute_tokens` 内的跨度处理应分别理解，参见 [[会话循环与文件会话帧的职责边界]]。^[atlas-core-session.md:37-39, atlas-core-session.md:47-53, atlas-core-session.md:72-74]

## 证据边界

上述规则来自 `session.rs` 的结构性阅读记录，不构成语言或数学验收。来源将上游行号及对应关系归为实现方的移植陈述，语义等值仍以 [[HPC 验收证据链|HPC 差分门]] 为准；来源列出的验收范围是 Weyl owner/dual 语义的 A1 限定范围。^[atlas-core-session.md:9-14, atlas-core-session.md:75-80]

## Sources

- [atlas-core-session.md](../../sources/atlas-core-session.md) — 会话外层循环与 SessionEvent 面（session.rs）。

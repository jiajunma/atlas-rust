---
title: SetType 命令的词法终止符跨度重建
summary: SetType 的源码跨度依据真实词法终止符重建，换行终止时列号加一，以保留上游将命令换行计入跨度的行为。
sources:
  - atlas-core-session.md
kind: concept
createdAt: "2026-10-09T14:34:56.697Z"
updatedAt: "2026-10-10T00:22:44.812Z"
tags:
  - 源码位置
  - 语法分析
aliases:
  - settype-命令的词法终止符跨度重建
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: SetType 命令的词法终止符跨度重建
summary: SetType 的源码跨度按真实词法终止符的行列位置重建，换行终止时列号加一，以保留上游将命令换行计入跨度的约定。
sources:
  - atlas-core-session.md
kind: concept
tags:
  - 源码位置
  - 兼容契约
aliases:
  - settype-命令的词法终止符跨度重建
---

# SetType 命令的词法终止符跨度重建

`Command::SetType` 的源码跨度（span）由会话层的 `execute_tokens` 按**真实词法终止符**重建。上游 `parser.y` 将命令换行计入 `set_type` 的 `@$`；为保留这一行为，实现使用终止符的实际行列位置，不猜测语义 token 之后的列号。^[atlas-core-session.md:47-53]

## 重建规则

跨度重建依据终止符的行、列位置；当终止符为 `Newline` 时，列号加 1，使命令换行计入跨度。这里的关键是保留词法终止符的位置，而非仅依据命令中的语义 token 确定边界。^[atlas-core-session.md:51-53]

## 与逐命令解析的关系

会话外层循环遇到 `Newline` 时调用 `execute_tokens`，遇到 `Eof` 时执行剩余命令缓冲并退出。Atlas 的词法分类依赖先前命令留下的状态，因此会话层逐命令处理，永不预切分整个源文件。^[atlas-core-session.md:11-14, atlas-core-session.md:29-35]

换行触发解析，但不保证命令已经完整。当终止符为 `Newline` 时，`allow_more` 为真；若 `parse_command_fragment_in(.., context.types(), allow_more)` 返回 `Ok(None)`，会话保留前缀，不求值、不发出诊断，继续在同一缓冲中累积多行命令。原始源输入与文件会话共用这条路径，参见 [[多行命令的增量解析]]。^[atlas-core-session.md:47-50]

文件包含由 `session_frame.rs` 负责；会话层本身遇到包含指令会发出 Io 诊断。该职责划分见 [[会话循环与文件会话帧的职责边界]]。^[atlas-core-session.md:37-39, atlas-core-session.md:72-74]

## 证据边界

上述规则来自 `session.rs` 的结构性阅读记录，不构成语言或数学验收。来源将上游行号及对应关系视为实现方的移植陈述，语义等值仍以 [[HPC 验收证据链|HPC 差分门]] 为准。来源列出的具体验收范围是 Weyl owner/dual 语义的 A1 限定范围，未提供 SetType 跨度重建的专项验收结论。^[atlas-core-session.md:9-14, atlas-core-session.md:75-80]

## Sources

- [atlas-core-session.md](../../sources/atlas-core-session.md) — 会话外层循环与 SessionEvent 面（session.rs）：逐命令执行、字节保留输出与回归测试库。

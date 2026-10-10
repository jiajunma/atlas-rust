---
title: TokenCursor 单 Token 前瞻
summary: TokenCursor 通过 peek 与 bump 提供单 token 前瞻，并缓存词法错误以保证前瞻和消费观察到相同结果。
sources:
  - atlas-core-lex.md
kind: concept
createdAt: "2026-10-09T14:31:46.874Z"
updatedAt: "2026-10-10T00:20:35.554Z"
tags:
  - 词法分析
  - 解析接口
aliases:
  - tokencursor-单-token-前瞻
  - T单T前
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: TokenCursor 单 Token 前瞻
summary: TokenCursor 通过 peek 与 bump 提供单 token 前瞻，并缓存词法错误，保证前瞻与消费观察到同一结果。
sources:
  - atlas-core-lex.md
kind: concept
tags:
  - 词法分析
  - 解析接口
aliases:
  - tokencursor-单-token-前瞻
provenanceState: extracted
---

# TokenCursor 单 Token 前瞻

`TokenCursor` 通过 `peek` 与 `bump` 提供单 token 前瞻。其核心契约是：**词法错误像 token 一样被缓存**，使前瞻与消费观察到同一结果。^[atlas-core-lex.md:79-80]

## 有状态词法器中的位置

Atlas 词法器刻意采用有状态设计，支持[[可嵌套注释]]，允许解析器或 REPL 逐 token 消费输入；`tokenize` 则是获取完整 token 流的兼容便利接口。`TokenCursor` 提供逐 token 消费所需的前瞻接口。^[atlas-core-lex.md:9-12, atlas-core-lex.md:79-83]

输入流中的换行受[[换行抑制状态机]]约束：只有能终止当前命令的换行才成为可观察 token。`skip_space` 仅在嵌套栈与 `prevent_termination` 均为空时保留换行，否则将其消费。^[atlas-core-lex.md:38-46]

## 错误缓存与恢复

`TokenCursor` 缓存词法错误，保证 `peek` 与 `bump` 看到同一结果。词法器还通过 `pending` 暂存恢复出的 token：未闭合字符串会报告 Lexical warning `"Closing string denotation."`，并将恢复出的 `String` token 放入 `pending`，使该 token 与后续命令仍可供解析器使用。详见[[字符串转义与未闭合恢复]]。^[atlas-core-lex.md:74-80]

命令级恢复由 `recover_command` 处理：丢弃当前物理行的剩余部分，并重置续行、嵌套与命令首状态。^[atlas-core-lex.md:78-78]

## 与完整流接口的关系

`tokenize` 是全有或全无的兼容便利接口；`tokenize_with_diagnostics` 返回 `LexOutput { tokens, diagnostics }`，按遭遇顺序保留诊断，并保留所有可恢复 token。与这些完整流接口相比，`TokenCursor` 提供单 token 前瞻与消费接口。^[atlas-core-lex.md:79-83]

## 职责与证据边界

词法器本身不进行语义判定。文法适配位于 `syntax.rs`，使用 LALRPOP spanned-token 流；逐命令消费位于 `session.rs`；指令的包含与重定向语义位于 `session_frame.rs`。会话层背景可参见[[会话帧驱动的 CLI 执行模型]]。^[atlas-core-lex.md:87-89]

来源记录了覆盖词法状态机分支的 23 个测试，但材料属于结构性阅读，不声称完成语言验收。上游行号引用属于实现方的移植陈述；词法兼容仍以 HPC 语言语料门为准。^[atlas-core-lex.md:9-13, atlas-core-lex.md:85-91]

## Sources

- [atlas-core-lex.md](../../sources/atlas-core-lex.md) — 有状态词法器（lex.rs）：TokenCursor 契约、恢复机制与测试边界。

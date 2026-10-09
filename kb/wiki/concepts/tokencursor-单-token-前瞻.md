---
title: TokenCursor 单 Token 前瞻
summary: TokenCursor 通过 peek 与 bump 提供单 token 前瞻，并将词法错误一并缓存，保证前瞻与消费观察到同一结果。
sources:
  - atlas-core-lex.md
kind: concept
createdAt: "2026-10-09T14:31:46.874Z"
updatedAt: "2026-10-09T20:35:01.196Z"
tags:
  - 词法分析
  - 流式解析
aliases:
  - tokencursor-单-token-前瞻
  - T单T前
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: TokenCursor 单 Token 前瞻
summary: TokenCursor 通过 peek 与 bump 提供单 token 前瞻，并缓存词法错误，保证前瞻与消费观察到同一结果。
sources:
  - atlas-core-lex.md
kind: concept
tags:
  - 词法分析
  - 前瞻
  - 流式解析
aliases:
  - tokencursor-单-token-前瞻
provenanceState: extracted
---

# TokenCursor 单 Token 前瞻

`TokenCursor` 通过 `peek` 与 `bump` 提供单 token 前瞻。其核心契约是：**词法错误也像 token 一样被缓存**，因此前瞻与消费看到的是同一结果。^[atlas-core-lex.md:79-80]

## 有状态词法器中的位置

Atlas 词法器刻意采用有状态设计，支持[[可嵌套注释]]，并允许解析器或 REPL 逐 token 消费输入。`TokenCursor` 提供这种消费方式所需的前瞻接口；需要完整 token 流时，可使用兼容便利接口 `tokenize`。^[atlas-core-lex.md:9-12, atlas-core-lex.md:79-83]

前瞻面对的 token 流受[[换行抑制状态机]]约束：只有能够终止当前命令的换行才成为可观察 token，分组构造与续行 token 会抑制换行。具体而言，换行仅在嵌套栈为空且 `prevent_termination` 为空时透出。^[atlas-core-lex.md:38-46]

## 错误缓存与恢复

游标缓存词法错误，使错误也遵循 `peek` 与 `bump` 的一致性契约。词法器另有恢复 token 的暂存机制：未闭合字符串会产生 Lexical warning `"Closing string denotation."`，并将恢复出的 `String` token 放入 `pending`，让该 token 与后续命令仍可供解析器使用。游标的结果缓存与词法器的恢复暂存应分别理解，相关细节见[[字符串转义与未闭合恢复]]。^[atlas-core-lex.md:74-80]

命令级恢复由 `recover_command` 处理：它丢弃当前物理行的剩余部分，并重置续行、嵌套与命令首状态。^[atlas-core-lex.md:78-78]

## 与完整流接口的关系

`tokenize` 是全有或全无的兼容便利接口；`tokenize_with_diagnostics` 则返回 `LexOutput { tokens, diagnostics }`，按遭遇顺序保留诊断，同时保留所有可恢复 token。它们与 `TokenCursor` 的单 token 前瞻接口提供不同的结果交付方式。^[atlas-core-lex.md:79-83]

## 职责与证据边界

词法器本身不进行语义判定。文法适配位于 `syntax.rs`，使用 LALRPOP spanned-token 流；逐命令消费位于 `session.rs`；指令的包含与重定向语义位于 `session_frame.rs`。会话层背景可参见[[会话帧驱动的 CLI 执行模型]]。^[atlas-core-lex.md:87-89]

来源记录了覆盖词法状态机分支的 23 个测试，但其证据性质是结构性阅读，不声称完成语言验收。上游行号引用属于实现方的移植陈述，词法兼容仍以 HPC 语言语料门为准。^[atlas-core-lex.md:9-13, atlas-core-lex.md:85-91]

## Sources

- [atlas-core-lex.md](../../sources/atlas-core-lex.md) — 有状态词法器（lex.rs）：TokenCursor 契约、恢复机制与测试边界。

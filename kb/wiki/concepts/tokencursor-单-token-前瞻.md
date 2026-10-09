---
title: TokenCursor 单 Token 前瞻
summary: TokenCursor 提供 peek 与 bump，并将词法错误像 token 一样缓存，保证前瞻与消费观察到同一结果。
sources:
  - atlas-core-lex.md
kind: concept
createdAt: "2026-10-09T14:31:46.874Z"
updatedAt: "2026-10-09T14:31:46.874Z"
tags:
  - 词法分析
  - 前瞻
  - 流式解析
aliases:
  - tokencursor-单-token-前瞻
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# TokenCursor 单 Token 前瞻

`TokenCursor` 通过 `peek` 和 `bump` 提供单 token 前瞻。其关键契约是：**词法错误也像 token 一样被缓存**，因此 `peek` 与 `bump` 看到的是同一结果。^[atlas-core-lex.md:79-80]

## 有状态词法器中的位置

Atlas 词法器刻意采用有状态设计，以支持可嵌套注释，以及解析器或 REPL 逐 token 消费输入。`TokenCursor` 的单 token 前瞻接口服务于这种消费方式；需要完整 token 流时，则可使用兼容便利接口 `tokenize`。^[atlas-core-lex.md:9-12, atlas-core-lex.md:79-83]

前瞻所面对的 token 流已经受词法状态约束：只有能够终止当前命令的换行才成为可观察 token，分组构造与续行 token 会抑制换行。因此，单 token 前瞻观察的是词法器产生的结果，而不是逐个物理换行。相关 token 种类见 [[Atlas 词法 Token 模型]]。^[atlas-core-lex.md:17-23, atlas-core-lex.md:38-46]

## 错误缓存与恢复

错误缓存使词法失败也遵循 `peek` 与 `bump` 的一致性契约。词法器同时支持可恢复诊断：未闭合字符串会报告 Lexical warning，并将恢复出的 `String` token 放入 `pending`，使该 token 与后续命令仍可供解析器使用。这里需要区分游标对词法结果的缓存与词法器对恢复 token 的暂存；字符串恢复细节见 [[字符串转义与未闭合恢复]]。^[atlas-core-lex.md:74-80]

完整流接口采用不同的结果交付方式：`tokenize` 是全有或全无的便利接口；`tokenize_with_diagnostics` 返回 `LexOutput { tokens, diagnostics }`，按遭遇顺序保留诊断，并保留所有可恢复 token。^[atlas-core-lex.md:81-83]

## 职责与证据边界

词法器本身不进行语义判定。文法适配位于 `syntax.rs`，逐命令消费位于 `session.rs`，指令的包含与重定向语义位于 `session_frame.rs`；这些职责可结合 [[Atlas 语法前端与运算符优先级归约]] 与 [[会话帧驱动的 CLI 执行模型]] 阅读。^[atlas-core-lex.md:87-89]

本来源属于结构性阅读，记录了覆盖词法状态机分支的 23 个测试，但不声称完成语言验收；词法兼容仍以 HPC 语言语料门为准。^[atlas-core-lex.md:9-13, atlas-core-lex.md:85-91]

## Sources

- [atlas-core-lex.md](../../sources/atlas-core-lex.md) — 有状态词法器（lex.rs）：TokenCursor 契约、恢复机制与测试边界。

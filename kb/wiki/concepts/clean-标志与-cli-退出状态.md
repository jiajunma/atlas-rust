---
title: clean 标志与 CLI 退出状态
summary: CLI 根据会话 clean 标志确定退出状态：语法、类型及运行时错误导致非零退出，缺失包含文件本身不使 clean 失效。
sources:
  - atlas-cli-main.md
kind: concept
createdAt: "2026-10-09T14:25:27.501Z"
updatedAt: "2026-10-10T00:13:45.979Z"
tags:
  - CLI
  - 错误处理
aliases:
  - clean-标志与-cli-退出状态
  - C标C退
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: clean 标志与 CLI 退出状态
summary: CLI 依据会话 clean 标志确定退出状态；语法、类型和运行时错误导致非零退出，缺失包含文件本身不使 clean 失效。
sources:
  - atlas-cli-main.md
kind: concept
tags:
  - CLI
  - 错误处理
  - 会话管理
aliases:
  - clean-标志与-cli-退出状态
provenanceState: extracted
---

# clean 标志与 CLI 退出状态

`atlas-cli` 的退出状态遵循会话的 **clean 标志**：语法、类型或运行时错误会导致非零退出；缺少包含文件本身不会使 clean 标志失效。`quit` 用于提前结束会话。^[atlas-cli-main.md:35-37]

## 会话与 CLI 的职责

CLI 通过会话帧，在文件参数或 stdin 上运行一个共享会话；会话机器位于 `atlas-core`，CLI 根据会话的 clean 状态确定退出状态。相关机制见 [[会话 clean 标志与诊断分流]] 和 [[SessionFrame 会话帧与文件包含语义]]。^[atlas-cli-main.md:10-13, atlas-cli-main.md:35-43]

`Diagnostic` 事件经 `frame.describe_bytes` 生成包含出处、源行和 caret 标记的诊断，并写入 stderr。退出状态则遵循上述 clean 规则，包括缺失包含文件本身不使 clean 失效的例外。诊断输出的细节见 [[CLI 事件输出与源码诊断]]。^[atlas-cli-main.md:22-25, atlas-cli-main.md:35-37]

## 输入处理边界

文件参数被刻意作为普通命令流处理，上游文件参数的 prelude-capture 语义不在此实现目标内。stdin 非终端时读入后运行，终端输入则进入 `run_interactive`。参见 [[交互式与非交互式输入分流]]。^[atlas-cli-main.md:30-34]

## 证据范围

来源覆盖 `crates/atlas-cli/src/main.rs` 全部 175 行的结构性阅读，不构成语言验收结论。该文件没有自身测试，行为由 HPC 语料门覆盖；来源中的上游行号引用属于实现方的移植陈述，CLI 兼容性仍以 HPC 语料门为准。参见 [[CLI 兼容性的证据边界]]。^[atlas-cli-main.md:9-14, atlas-cli-main.md:41-45]

## Sources

- [atlas-cli-main.md — CLI 前端：会话帧驱动、--path 解析与 clean 退出状态](../../sources/atlas-cli-main.md)

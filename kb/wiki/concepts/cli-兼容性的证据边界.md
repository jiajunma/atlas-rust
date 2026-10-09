---
title: CLI 兼容性的证据边界
summary: 本包仅完成结构性阅读，不声称语言验收；上游行号引用属于移植方陈述，CLI 兼容性以 HPC 语料门为准，快照哈希仅标识字节。
sources:
  - atlas-cli-main.md
kind: concept
createdAt: "2026-10-09T14:25:34.957Z"
updatedAt: "2026-10-09T14:25:34.957Z"
tags:
  - 兼容性验证
  - 证据管理
  - HPC
aliases:
  - cli-兼容性的证据边界
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# CLI 兼容性的证据边界

CLI 兼容性的判断需要区分结构性阅读、实现方的移植陈述与 HPC 语料验证。现有源码包完整覆盖 `crates/atlas-cli/src/main.rs` 的 175 行，但明确不声称完成语言验收；CLI 兼容性以 HPC 语料门为准。^[atlas-cli-main.md:9-14, atlas-cli-main.md:41-45]

## 结构性阅读支持的结论

CLI 前端通过会话帧运行共享会话，处理文件参数或 stdin；包含文件按可重复的 `--path=DIR` 前缀再按工作目录解析。其输出约定包括顶层值的 `Value: …`、定义与包含的框架文字、发往 stderr 的带出处诊断，以及会话结束时的 `Bye.`。这些是源码包记录的实现行为，可关联[[会话帧驱动的 CLI 执行模型]]与[[CLI 事件输出与源码诊断]]。^[atlas-cli-main.md:10-14]

字节处理也是兼容性审查的一部分：`FsProvider` 使用有损 UTF-8，避免将游离的非 UTF-8 字节变成打开失败；`print_events` 将 `OutputBytes` 与 `ReportBytes` 的原始字节写入 stdout，诊断则经 `frame.describe_bytes` 生成出处、源行与 caret 后写入 stderr。^[atlas-cli-main.md:18-25]

退出状态遵循上游 clean 标志：语法、类型和运行时错误使退出状态非零，缺少包含文件本身不置该标志，`quit` 可提前结束会话。相关语义见[[clean 标志与 CLI 退出状态]]。^[atlas-cli-main.md:35-37]

## 有意保留的兼容范围限制

文件参数被刻意作为普通命令流输入；上游文件参数的 prelude-capture 语义明确属于非目标。因此，不能把文件参数入口的实现描述理解为对上游该项语义的完整复现。stdin 则按是否连接终端分流：非终端输入读入后运行，终端输入进入 `run_interactive`。^[atlas-cli-main.md:30-34]

## 验证证据的适用范围

会话机器位于 `atlas-core`，CLI 的 `main.rs` 本身没有测试，行为由 HPC 语料门覆盖。源码包中引用的上游行号属于实现方的移植陈述，不能单独作为 CLI 兼容性的验收依据；这一边界与[[HPC 验收证据链]]相关。^[atlas-cli-main.md:41-44]

源码包记录的字节数与哈希仅标识 git base `964f0033` 对应的本快照字节。它们界定所阅读的源码版本；该包同时明确保留“结构性阅读，不声称语言验收”的限制。^[atlas-cli-main.md:14-14, atlas-cli-main.md:45-45]

## Sources

- [atlas-cli-main.md](atlas-cli-main.md)

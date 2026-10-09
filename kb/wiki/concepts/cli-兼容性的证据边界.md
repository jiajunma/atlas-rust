---
title: CLI 兼容性的证据边界
summary: 本包仅完成 main.rs 的结构性阅读，文件无测试，上游引用属于移植方陈述，CLI 兼容性须由 HPC 语料门验证。
sources:
  - atlas-cli-main.md
kind: concept
createdAt: "2026-10-09T14:25:34.957Z"
updatedAt: "2026-10-09T22:12:07.081Z"
tags:
  - 证据边界
  - 兼容性验证
aliases:
  - cli-兼容性的证据边界
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: CLI 兼容性的证据边界
summary: CLI 源码包仅完成结构性阅读，不声称语言验收；上游行号属于实现方移植陈述，兼容性以 HPC 语料门为准，快照字节数与哈希仅标识源码字节。
sources:
  - atlas-cli-main.md
kind: concept
tags:
  - 兼容性验证
  - 证据管理
  - HPC
aliases:
  - cli-兼容性的证据边界
provenanceState: extracted
---

# CLI 兼容性的证据边界

CLI 兼容性的判断需要区分结构性阅读、实现方的移植陈述与 HPC 语料验证。源码包完整覆盖 `crates/atlas-cli/src/main.rs` 的 175 行，但明确不声称完成语言验收；CLI 兼容性以 HPC 语料门为准。^[atlas-cli-main.md:9-14, atlas-cli-main.md:41-45]

## 结构性阅读支持的实现描述

CLI 前端通过会话帧在文件参数或 stdin 上运行共享会话；`<file` 包含按可重复的 `--path=DIR` 前缀、再按工作目录解析。输出约定包括顶层值的 `Value: …`、定义与包含的框架文字、发往 stderr 的带出处和源摘诊断，以及会话结束时的 `Bye.`。相关主题见 [[会话帧驱动的 CLI 执行模型]]与[[包含文件的搜索路径解析]]。^[atlas-cli-main.md:10-13]

字节处理具有明确的输入与输出约定：`FsProvider` 使用有损 UTF-8，避免将游离的非 UTF-8 字节变成打开失败；`print_events` 将 `OutputBytes` 与 `ReportBytes` 的原始字节写入 stdout，诊断则经 `frame.describe_bytes` 生成出处、源行与 caret 后写入 stderr。详见 [[CLI 事件输出与源码诊断]]。^[atlas-cli-main.md:18-25]

退出状态遵循上游 clean 标志：语法、类型和运行时错误使退出状态非零，缺少包含文件本身不置该标志，`quit` 可提前结束会话。相关语义见 [[clean 标志与 CLI 退出状态]]。^[atlas-cli-main.md:35-37]

## 有意限定的兼容范围

文件参数被刻意作为普通命令流输入，上游文件参数的 prelude-capture 语义明确属于非目标。stdin 则按是否连接终端分流：非终端输入读入后运行，终端输入进入 `run_interactive`，显示横幅并执行 `atlas> ` 循环。文件参数入口的这一范围限制应与[[交互式与非交互式输入分流]]分别理解。^[atlas-cli-main.md:30-34]

## 验证与快照的证据范围

会话机器位于 `atlas-core`，CLI 的 `main.rs` 本身没有测试，来源说明其行为由 HPC 语料门覆盖。上游行号引用属于实现方的移植陈述，CLI 兼容性仍以 HPC 语料门为准；相关背景见 [[HPC 验收证据链]]。^[atlas-cli-main.md:41-44]

源码包记录的字节数与哈希仅标识本快照字节，所列 git base 为 `964f0033`。这些标识应与来源明确声明的“结构性阅读，不声称语言验收”一并保留。^[atlas-cli-main.md:14-14, atlas-cli-main.md:45-45]

## Sources

- [atlas-cli-main.md](../../sources/atlas-cli-main.md) — CLI 前端（atlas-cli/main.rs）：会话帧驱动、`--path` 解析与 clean 退出状态。

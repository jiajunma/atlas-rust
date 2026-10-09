---
title: 会话帧驱动的 CLI 执行模型
summary: CLI 通过 atlas-core 会话帧在共享会话中执行文件参数或 stdin；文件参数按普通命令流处理，不实现上游 prelude-capture 语义，quit 可提前结束。
sources:
  - atlas-cli-main.md
kind: concept
createdAt: "2026-10-09T14:25:08.442Z"
updatedAt: "2026-10-09T14:25:08.442Z"
tags:
  - CLI
  - 会话管理
  - 兼容性边界
aliases:
  - 会话帧驱动的-cli-执行模型
  - 会C执
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 会话帧驱动的 CLI 执行模型

Atlas 的 CLI 前端通过会话帧，在文件参数或标准输入上运行一个共享会话。会话机器位于 `atlas-core`，`crates/atlas-cli/src/main.rs` 负责输入接入、文件系统适配、事件输出与退出状态衔接。^[atlas-cli-main.md:9-14, atlas-cli-main.md:41-43]

## 输入与执行流程

`main` 将可重复出现的 `--path=DIR` 参数加入 `search_path`，其余参数作为文件参数。`<file` 包含依次按这些搜索路径前缀、再按工作目录解析，相关机制见 [[包含文件的搜索路径解析]]。^[atlas-cli-main.md:10-13, atlas-cli-main.md:26-29]

文件参数刻意作为普通命令流输入共享会话；上游文件参数的 prelude-capture 语义不在此实现目标内。标准输入不是终端时，前端读入并执行；标准输入是终端时，则进入 `run_interactive`，显示版本横幅并运行 `atlas> ` 提示符循环。参见 [[交互式与非交互式输入分流]]。^[atlas-cli-main.md:30-34]

执行遇到 `quit` 时提前结束，会话以 `Bye.` 收尾。^[atlas-cli-main.md:11-13, atlas-cli-main.md:35-37]

## 文件系统适配

`FsProvider` 实现文件系统 `FileProvider`，采用有损 UTF-8 解码，以避免游离的非 UTF-8 字节被误作文件打开失败。`FsSink` 使用 `OpenOptions` 以写入、创建及追加或截断模式打开输出文件，并提供 `write_bytes` 与 `close` 操作。^[atlas-cli-main.md:18-21]

## 会话事件与输出

`print_events` 将会话事件分派到输出通道：`Output` 与 `ReportLine` 经 `print!` 输出，`OutputBytes` 与 `ReportBytes` 将原始字节写入 stdout。`Diagnostic` 经 `frame.describe_bytes` 生成包含出处、源行和 caret 指示的诊断，写入 stderr。参见 [[SessionEvent 与字节保留输出]] 和 [[CLI 事件输出与源码诊断]]。^[atlas-cli-main.md:22-25]

顶层值以 `Value: …` 形式打印，定义与包含的框架文字逐字输出。会话帧已经将值渲染为 `Output` 文本，因此 `print_events` 中的 `Value` 分支只是防御性处理。^[atlas-cli-main.md:11-13, atlas-cli-main.md:22-25]

## clean 标志与退出状态

CLI 退出状态遵循上游的 clean 标志：语法、类型或运行时错误使退出状态非零；缺少包含文件本身不会将 clean 标志置为失败。因而，文件包含失败与语言执行错误在退出状态上的处理不同，具体纪律见 [[会话 clean 标志与诊断分流]]。^[atlas-cli-main.md:35-37]

## 实现与证据边界

该源包覆盖 `main.rs` 全部 175 行，但证据性质是结构性阅读，不代表语言验收。该文件自身没有测试，行为由 HPC 语料门覆盖；上游行号引用属于实现方的移植陈述，CLI 兼容性仍以 HPC 语料门为准。参见 [[CLI 兼容性的证据边界]]。^[atlas-cli-main.md:9-14, atlas-cli-main.md:41-44]

## Sources

- [atlas-cli-main.md](atlas-cli-main.md)

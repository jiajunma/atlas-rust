---
title: 会话帧驱动的 CLI 执行模型
summary: CLI 通过 atlas-core 会话帧在共享会话中执行文件参数或 stdin，文件参数按普通命令流处理，不实现 prelude-capture，quit 可提前结束。
sources:
  - atlas-cli-main.md
kind: concept
createdAt: "2026-10-09T14:25:08.442Z"
updatedAt: "2026-10-09T22:11:36.309Z"
tags:
  - CLI
  - 会话管理
aliases:
  - 会话帧驱动的-cli-执行模型
  - 会C执
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: 会话帧驱动的 CLI 执行模型
summary: CLI 通过 atlas-core 会话帧在共享会话中执行文件参数或 stdin；文件参数按普通命令流处理，退出状态遵循 clean 标志。
sources:
  - atlas-cli-main.md
kind: concept
tags:
  - CLI
  - 会话执行
  - 兼容性边界
aliases:
  - 会话帧驱动的-cli-执行模型
---

# 会话帧驱动的 CLI 执行模型

Atlas CLI 通过会话帧，在文件参数或标准输入上运行一个共享会话。会话机器位于 `atlas-core`，`crates/atlas-cli/src/main.rs` 负责输入接入、文件系统适配、事件输出和退出状态衔接。^[atlas-cli-main.md:9-13, atlas-cli-main.md:18-37, atlas-cli-main.md:41-43]

## 输入与执行流程

`main` 将可重复出现的 `--path=DIR` 参数加入 `search_path`，其余参数作为文件参数。对于 `<file` 包含，文件依次按这些搜索路径前缀、再按工作目录解析，参见 [[包含文件的搜索路径解析]]。^[atlas-cli-main.md:10-13, atlas-cli-main.md:26-29]

文件参数刻意按普通命令流输入共享会话；上游文件参数的 prelude-capture 语义不在此实现目标内。标准输入不是终端时，前端读入并执行；是终端时，则进入 `run_interactive`，显示横幅并运行 `atlas> ` 提示符循环。横幅标示 Atlas 版本 `1.1.1`、axis 语言版本 `1.1`，以及 Rust 编译和 readline 禁用状态。参见 [[交互式与非交互式输入分流]]。^[atlas-cli-main.md:30-34]

`quit` 可使执行提前结束，会话以 `Bye.` 收尾。^[atlas-cli-main.md:11-13, atlas-cli-main.md:35-37]

## 文件系统适配

`FsProvider` 实现文件系统 `FileProvider`，采用有损 UTF-8 解码，避免将游离的非 UTF-8 字节视为打开失败。`FsSink` 使用 `OpenOptions`，以写入、创建及追加或截断模式打开文件，并提供 `write_bytes` 与 `close` 操作。^[atlas-cli-main.md:18-21]

## 会话事件与输出

`print_events` 按事件类型分派输出：`Output` 和 `ReportLine` 经 `print!` 输出；`OutputBytes` 和 `ReportBytes` 将原始字节写入 stdout；`Diagnostic` 经 `frame.describe_bytes` 生成包含出处、源行和 caret 指示的诊断，写入 stderr。参见 [[SessionEvent 与字节保留输出]] 和 [[CLI 事件输出与源码诊断]]。^[atlas-cli-main.md:22-25]

顶层值以 `Value: …` 形式打印，定义与包含的框架文字逐字输出。会话帧已经将值渲染为 `Output` 文本，因此 `print_events` 中的 `Value` 分支属于防御性处理。^[atlas-cli-main.md:11-13, atlas-cli-main.md:22-25]

## clean 标志与退出状态

CLI 退出状态遵循上游 clean 标志：语法、类型或运行时错误使退出状态非零；缺少包含文件本身不会使 clean 标志变为失败。包含文件缺失与语言执行错误的退出状态处理因此有所区别，参见 [[会话 clean 标志与诊断分流]]。^[atlas-cli-main.md:35-37]

## 工作线程配置

`main` 通过 `rayon::ThreadPoolBuilder` 将工作线程栈设为 2 MiB。来源将这一设置解释为降低 RSS 的措施，并指出 Weyl 枚举、轨道共轭和 KGB BFS 等并行通道采用迭代实现。相关主题见 [[Rayon 工作线程栈配置]]。^[atlas-cli-main.md:26-29]

## 实现与证据边界

源包覆盖 `main.rs` 全部 175 行，证据性质为结构性阅读，不代表语言验收。该文件自身没有测试，行为由 HPC 语料门覆盖；上游行号引用属于实现方的移植陈述，CLI 兼容性仍以 HPC 语料门为准，参见 [[CLI 兼容性的证据边界]]。^[atlas-cli-main.md:9-14, atlas-cli-main.md:41-44]

## Sources

- [atlas-cli-main.md](../../sources/atlas-cli-main.md) — CLI 前端（`atlas-cli/main.rs`）——会话帧驱动、`--path` 解析与 clean 退出状态。

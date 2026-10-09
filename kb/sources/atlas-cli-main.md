---
title: CLI 前端（atlas-cli/main.rs）——会话帧驱动、--path 解析与 clean 退出状态
source: atlas-rust/atlas-cli-main
ingestedAt: 2026-10-09T18:30:00Z
---

# CLI 前端（atlas-cli/main.rs）

编辑状态：**结构性阅读完成；维护者直接撰写（无 Kimi 调用）**。本包覆盖
`crates/atlas-cli/src/main.rs`（175 行，全部）。文件自述：经会话帧把一个
共享会话跑在文件参数（或 stdin）上；`<file` 包含按可重复的
`--path=DIR` 前缀再按工作目录解析；顶层值打印 `Value: …`；定义与包含
框架文字逐字；诊断带出处与源摘到 stderr；会话以 `Bye.` 结束。
结构性阅读，不声称语言验收。

## 结构与纪律

- `FsProvider`：文件系统 `FileProvider`，**有损 UTF-8**——上游读字节，
  游离非 UTF-8 字节不得变成打开失败。
- `FsSink`：`OpenOptions`（write+create，append 或 truncate）、
  `write_bytes`、`close`。
- `print_events`：`Output`/`ReportLine` 走 print!；`OutputBytes`/
  `ReportBytes` 写**原始字节**到 stdout；`Diagnostic` 经
  `frame.describe_bytes`（出处+源行+caret）到 stderr；`Value` 分支是
  防御性的（帧已把值渲染成 Output 文本）。
- `main`：`rayon::ThreadPoolBuilder` 设 2MiB 工作栈（Weyl 枚举、轨道
  共轭、KGB BFS 这些并行通道是迭代的——适度工作栈压低 RSS：默认每
  worker 8MiB vs 实际约 1-2MiB）；`--path=DIR` 入 search_path，其余为
  文件参数。
- 文件参数**刻意按普通命令流喂**（上游主循环从 stdin 读；其文件参数的
  prelude-capture 语义此处是 non-goal）。stdin 非终端时读入并跑；
  终端时进 `run_interactive`（横幅 "This is 'atlas' (version 1.1.1,
  axis language version 1.1)…compiled with Rust, readline disabled" +
  `atlas> ` 循环）。
- 退出状态 = 上游 **clean 标志**：语法/类型/运行时错误使非零；**缺包含
  文件本身不置**（见 [会话帧包](atlas-core-session-frame.md)的 clean
  纪律）。`quit` 提前结束。

## 边界与限制

- 会话机器在 `atlas-core`（[会话帧包](atlas-core-session-frame.md) 与
  [会话包](atlas-core-session.md)）；本文件无测试（行为由 HPC 语料门
  覆盖）。
- 上游行号引用是**实现方移植陈述**；CLI 兼容以 HPC 语料门为准。
- 字节数/哈希只标识本快照字节（git base `964f0033`）。

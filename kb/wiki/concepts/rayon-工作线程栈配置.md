---
title: Rayon 工作线程栈配置
summary: CLI 将 Rayon 工作线程栈设为 2 MiB，源码以并行算法采用迭代实现及降低 RSS 为依据，但本包仅提供结构性阅读证据。
sources:
  - atlas-cli-main.md
kind: concept
createdAt: "2026-10-09T14:25:24.404Z"
updatedAt: "2026-10-09T20:29:28.647Z"
tags:
  - Rayon
  - 内存管理
aliases:
  - rayon-工作线程栈配置
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: Rayon 工作线程栈配置
summary: Atlas CLI 将 Rayon 工作线程栈设为 2 MiB，依据是相关并行通道采用迭代实现，目标是降低 RSS；来源未提供实测性能结论。
sources:
  - atlas-cli-main.md
kind: concept
tags:
  - Rayon
  - 内存管理
  - 并行计算
aliases:
  - rayon-工作线程栈配置
---

# Rayon 工作线程栈配置

Atlas CLI 在 `main` 中通过 `rayon::ThreadPoolBuilder` 将 Rayon 工作线程栈设为 **2 MiB**，目的是降低并行工作线程的内存占用（RSS）。^[atlas-cli-main.md:26-29]

## 配置依据

源材料说明，Weyl 枚举、轨道共轭和 KGB BFS 等并行通道采用迭代实现，因此使用适度大小的工作栈。材料将默认每个工作线程的 8 MiB 栈与实际约 1–2 MiB 的需求作对照，作为采用 2 MiB 配置的依据。相关算法背景可参见 [[Twisted involution 枚举与共轭轨道分区]] 与 [[KGB 图与弱实形式]]。^[atlas-cli-main.md:26-28]

## 证据边界

上述配置及其动机来自对 `crates/atlas-cli/src/main.rs` 的结构性阅读，来源明确不声称语言验收。材料未提供这一配置的独立内存测量或性能验收结果，因此其中的栈大小对比不能作为已测得 RSS 降幅的证据。^[atlas-cli-main.md:9-14, atlas-cli-main.md:26-28]

`main.rs` 本身没有测试，来源说明其行为由 HPC 语料门覆盖；上游行号引用属于实现方的移植陈述，CLI 兼容性仍以 HPC 语料门为准。相关证据要求可参见 [[HPC 验收证据链]]。^[atlas-cli-main.md:41-44]

## Sources

- [atlas-cli-main.md](../../sources/atlas-cli-main.md)：CLI 前端（`atlas-cli/main.rs`），工作线程栈配置见第 26–29 行。

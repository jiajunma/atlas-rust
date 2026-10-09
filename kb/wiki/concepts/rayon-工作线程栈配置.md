---
title: Rayon 工作线程栈配置
summary: CLI 将 Rayon 工作线程栈设为 2 MiB，源码说明其依据是相关并行算法采用迭代实现并意在降低 RSS；该说明不构成实测性能结论。
sources:
  - atlas-cli-main.md
kind: concept
createdAt: "2026-10-09T14:25:24.404Z"
updatedAt: "2026-10-09T14:25:24.404Z"
tags:
  - Rayon
  - 内存管理
  - 并行计算
aliases:
  - rayon-工作线程栈配置
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# Rayon 工作线程栈配置

Atlas CLI 在 `main` 中通过 `rayon::ThreadPoolBuilder` 将 Rayon 工作线程栈设为 **2 MiB**。这一配置旨在降低并行工作线程的内存占用（RSS）。^[atlas-cli-main.md:26-29]

## 配置依据

源码说明指出，Weyl 枚举、轨道共轭和 KGB BFS 等并行通道采用迭代实现，因此使用适度大小的工作栈。源材料将默认每个 worker 的 8 MiB 栈与实际约 1–2 MiB 的需求作对照，作为采用 2 MiB 配置的依据。相关算法可参见 [[Twisted involution 枚举与共轭轨道分区]] 与 [[KGB 图与弱实形式]]。^[atlas-cli-main.md:26-28]

## 证据边界

上述配置及其动机来自对 CLI 前端的结构性阅读；源材料未提供该配置的独立内存测量或性能验收结果。`main.rs` 本身没有测试，行为由 HPC 语料门覆盖，因此不应把栈大小对比理解为已经测得的 RSS 降幅。相关验收要求可参见 [[HPC 验收证据链]]。^[atlas-cli-main.md:9-14, atlas-cli-main.md:26-28, atlas-cli-main.md:41-44]

## Sources

- [atlas-cli-main.md](atlas-cli-main.md)：CLI 前端（`atlas-cli/main.rs`），重点为第 26–29 行的 Rayon 工作线程栈配置说明。

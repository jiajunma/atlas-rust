---
title: Rayon 工作线程栈配置
summary: CLI 将 Rayon 工作线程栈设为 2 MiB；来源以并行通道采用迭代算法及降低 RSS 为依据，但未提供性能测量证据。
sources:
  - atlas-cli-main.md
kind: concept
createdAt: "2026-10-09T14:25:24.404Z"
updatedAt: "2026-10-10T00:13:59.978Z"
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
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: Rayon 工作线程栈配置
summary: Atlas CLI 将 Rayon 工作线程栈设为 2 MiB，以迭代式并行通道的栈需求为依据，目标是降低 RSS；来源仅提供结构性阅读证据。
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

Atlas CLI 在 `main` 中通过 `rayon::ThreadPoolBuilder` 将 Rayon 工作线程栈大小设为 **2 MiB**，目的是通过适度的工作栈大小降低 RSS。^[atlas-cli-main.md:26-29]

## 配置依据

源材料说明，Weyl 枚举、轨道共轭和 KGB BFS 等并行通道采用迭代实现，并将默认每个工作线程的 8 MiB 栈与实际约 1–2 MiB 的栈需求作对照，作为设置 2 MiB 工作栈的依据。相关算法背景可参见 [[Twisted involution 枚举与共轭轨道分区]] 与 [[KGB 图与弱实形式]]。^[atlas-cli-main.md:26-28]

## 证据边界

这一配置说明来自对 `crates/atlas-cli/src/main.rs` 全部 175 行的结构性阅读，来源明确不声称语言验收。材料给出了配置值及降低 RSS 的动机，但未提供独立内存测量或性能验收结果，因此不能据此断言已测得具体 RSS 降幅。^[atlas-cli-main.md:9-14, atlas-cli-main.md:26-28]

`main.rs` 本身没有测试；来源说明其行为由 HPC 语料门覆盖，并强调上游行号引用属于实现方的移植陈述，CLI 兼容性以 HPC 语料门为准。相关说明见 [[CLI 兼容性的证据边界]] 与 [[HPC 验收证据链]]。^[atlas-cli-main.md:41-44]

## Sources

- [atlas-cli-main.md](../../sources/atlas-cli-main.md) — CLI 前端（`atlas-cli/main.rs`），工作线程栈配置见第 26–29 行。

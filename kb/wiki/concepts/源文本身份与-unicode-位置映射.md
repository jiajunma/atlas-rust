---
title: 源文本身份与 Unicode 位置映射
summary: SourceText 保存源身份、文本和行首字节索引，将字节偏移钳到文本范围并退到字符边界，计算从 1 开始的行号和 Unicode 标量列号，生成含头不含尾的源跨度。
sources:
  - atlas-core-support-layer.md
kind: concept
createdAt: "2026-10-09T14:35:24.486Z"
updatedAt: "2026-10-09T14:35:24.486Z"
tags:
  - 源位置
  - Unicode
  - 源文本
aliases:
  - 源文本身份与-unicode-位置映射
  - 源U位
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 源文本身份与 Unicode 位置映射

源文本身份与位置映射由 `SourceId`、`SourceText`、`SourcePosition` 和 `SourceSpan` 共同表达：身份用于区分源缓冲，源文本保存行起始字节索引，位置与跨度用于描述文本中的具体区域。用户可见的行号、列号均从 1 开始。^[atlas-core-support-layer.md:16-19, atlas-core-support-layer.md:32-38]

## 源文本身份与跨度

`SourceId(u64)` 是源缓冲的稳定身份；值 `0` 表示匿名来源，适用于无需区分的输入。`SourceText` 自持源文本，包含 `source_id`、`text` 和 `line_starts`，构造时预存每行起始字节。测试锚点覆盖匿名身份的稳定性，以及身份随跨度保留的行为。^[atlas-core-support-layer.md:16-16, atlas-core-support-layer.md:34-38]

`SourceSpan` 采用含头不含尾的区间约定，构造时断言 `byte_start <= byte_end`。`SourceText::span(start, end)` 负责组装跨度；因此，字节区间的边界约定与用户可见的行列位置属于同一套源位置支撑机制。^[atlas-core-support-layer.md:18-19, atlas-core-support-layer.md:34-38]

## Unicode 位置映射

`SourceText::position(byte)` 先将输入字节偏移钳到文本长度以内，再退到字符边界，随后对 `line_starts` 使用 `partition_point` 确定从 1 开始的行号。列号按行首到该字节位置之间的 **Unicode 标量数加 1** 计算。这里的列号计量单位是 Unicode 标量，而非字节。^[atlas-core-support-layer.md:34-38]

这一映射明确处理越界偏移和字符边界：越界位置钳到文本末尾，偏移经过字符边界调整后才参与位置计算。已有测试锚点覆盖 Unicode 标量计数、换行计数和越界钳制。^[atlas-core-support-layer.md:35-38]

## 诊断集成与证据边界

结构化 `Diagnostic` 包含 `span` 字段，将源位置纳入诊断信息；诊断的渲染与打印则位于 `session_frame.rs` 的 `describe_bytes`。相关职责可结合 [[CLI 事件输出与源码诊断]] 与 [[会话帧驱动的 CLI 执行模型]] 阅读。^[atlas-core-support-layer.md:23-27, atlas-core-support-layer.md:70-72]

本说明依据支撑层的结构性阅读材料。所列测试锚点不代表语言或数学验收；行为兼容仍以 HPC 语料门为准，可参阅 [[HPC 验收证据链]]。^[atlas-core-support-layer.md:9-12, atlas-core-support-layer.md:73-74]

## Sources

- [atlas-core-support-layer.md](atlas-core-support-layer.md)

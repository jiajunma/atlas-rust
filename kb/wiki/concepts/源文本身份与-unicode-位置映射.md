---
title: 源文本身份与 Unicode 位置映射
summary: SourceText 保存稳定源身份与行首字节索引，将偏移钳界并退至字符边界，以 Unicode 标量计算一基列号及含头不含尾的跨度。
sources:
  - atlas-core-support-layer.md
kind: concept
createdAt: "2026-10-09T14:35:24.486Z"
updatedAt: "2026-10-10T00:23:06.433Z"
tags:
  - 源位置
  - Unicode
aliases:
  - 源文本身份与-unicode-位置映射
  - 源U位
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: 源文本身份与 Unicode 位置映射
summary: SourceText 保存源身份、文本与行首字节索引，将字节偏移钳至文本范围并退至字符边界，以 Unicode 标量计算一基列号，组装含头不含尾的源跨度。
sources:
  - atlas-core-support-layer.md
kind: concept
tags:
  - 源位置
  - Unicode
  - 源文本
aliases:
  - 源文本身份与-unicode-位置映射
  - 源U位
---

# 源文本身份与 Unicode 位置映射

源文本身份与位置映射由 `SourceId`、`SourceText`、`SourcePosition` 和 `SourceSpan` 共同表达：源身份标识输入缓冲，源文本保存文本与行首字节索引，位置与跨度描述文本中的区域。用户可见的行号和列号均从 1 开始。^[atlas-core-support-layer.md:16-19, atlas-core-support-layer.md:34-38]

## 源身份与文本存储

`SourceId(u64)` 是源缓冲的稳定身份；值 `0` 表示匿名来源，适用于无需区分的输入。`SourceText { source_id, text, line_starts }` 自持源文本，并在构造时预存每行的起始字节位置。^[atlas-core-support-layer.md:16-16, atlas-core-support-layer.md:32-38]

## 字节偏移到 Unicode 位置

`SourceText::position(byte)` 先将字节偏移钳到文本长度以内，再退到字符边界，随后在 `line_starts` 上通过 `partition_point` 确定从 1 开始的行号。列号等于行首到调整后字节位置之间的 **Unicode 标量数加 1**，因此列的计量单位是 Unicode 标量，而非字节。^[atlas-core-support-layer.md:34-38]

越界偏移会钳到文本末尾，字符边界调整发生在行列计算之前。来源列出的测试锚点覆盖 Unicode 标量与换行计数、越界钳制、匿名身份稳定，以及源身份随跨度保留。^[atlas-core-support-layer.md:34-38]

## 源跨度与诊断集成

`SourceSpan` 使用**含头不含尾**的区间约定，构造时断言 `byte_start <= byte_end`。`SourceText::span(start, end)` 负责组装源跨度，将源文本的位置映射与跨度表示衔接起来。^[atlas-core-support-layer.md:18-19, atlas-core-support-layer.md:34-38]

结构化 `Diagnostic` 通过 `span` 字段携带源跨度；诊断的渲染与打印由 `session_frame.rs` 中的 `describe_bytes` 处理。相关会话层背景可参阅 [[会话帧驱动的 CLI 执行模型]]。^[atlas-core-support-layer.md:23-27, atlas-core-support-layer.md:70-72]

## 证据边界

本页依据支撑层的结构性源码阅读材料，不构成语言或数学验收。来源中的上游行号属于实现方的移植陈述，行为兼容仍以 HPC 语料门为准；相关背景见 [[HPC 验收证据链]]。^[atlas-core-support-layer.md:9-12, atlas-core-support-layer.md:70-74]

## Sources

- [atlas-core-support-layer.md](../../sources/atlas-core-support-layer.md)：支撑层（diagnostic.rs + source.rs + coercions.rs）——结构化诊断、源位置与强转表/邻近谓词。

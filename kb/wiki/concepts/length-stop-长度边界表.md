---
title: length-stop 长度边界表
summary: 在块元素长度非降的构造保证下，length_stop[l] 记录首个长度不小于 l 的元素位置，并在表末追加块大小 size。
sources:
  - kl-support.md
kind: concept
createdAt: "2026-10-09T14:55:48.741Z"
updatedAt: "2026-10-09T14:55:48.741Z"
tags:
  - 索引结构
  - 长度排序
aliases:
  - length-stop-长度边界表
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# length-stop 长度边界表

length-stop 是 [[KlSupport：逐块 KL 支撑数据]] 为块预计算的长度边界表。`length_stop[l]` 表示首个长度大于或等于 `l` 的块元素位置；表的末尾追加块大小 `size`。它与下降集、good-ascent 集及本原索引表共同组成逐块 KL 支撑数据。^[kl-support.md:16-21, kl-support.md:37-39]

## 构造与顺序不变量

长度边界表依赖块元素按长度非降排列。`KlSupport::new` 首先调用 `validate_topology`，检查每个元素的长度均存在，并强制验证长度顺序非降；因此，这一排序要求由构造门控保证。相关约束见 [[KlSupport 的拓扑构造门控]]。^[kl-support.md:32-39]

这里的边界采用“长度 ≥ `l`”的定义，而不是仅查找长度等于 `l` 的元素。构造过程中还会计算 `max_length`，但随后通过 `let _ = max_length` 显式丢弃；来源将其记录为清理候选。^[kl-support.md:37-39]

## 测试与证据边界

来源列出的单元测试包含对长度非降约束违规的构造拒绝测试，但 `new` 的成功路径没有单元测试覆盖，因此不能据此宣称 length-stop 的构造结果已有专项单元测试验证。来源说明相关成功路径经 KL 层的 HPC 门覆盖；整体材料仍属于结构性阅读，不构成 KL 支撑层的数学验收。参见 [[KL 支撑层的测试覆盖与证据边界]]。^[kl-support.md:9-12, kl-support.md:57-62]

## Sources

- [kl-support.md](../../sources/kl-support.md)：逐块 KL 支撑数据：KlSupport 与 RankFlags（kl_support.rs）。

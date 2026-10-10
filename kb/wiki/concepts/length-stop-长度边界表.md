---
title: length-stop 长度边界表
summary: 在块元素长度非降的前提下，length_stop[l] 保存首个长度至少为 l 的元素位置，并在表末追加块大小。
sources:
  - kl-support.md
kind: concept
createdAt: "2026-10-09T14:55:48.741Z"
updatedAt: "2026-10-10T00:39:15.468Z"
tags:
  - KL
  - 索引
aliases:
  - length-stop-长度边界表
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: length-stop 长度边界表
summary: length_stop[l] 记录首个长度至少为 l 的块元素位置，依赖构造期保证的长度非降顺序，表末追加块大小 size。
sources:
  - kl-support.md
kind: concept
tags:
  - 索引结构
  - 长度排序
aliases:
  - length-stop-长度边界表
---

# length-stop 长度边界表

length-stop 是 [[KlSupport：逐块 KL 支撑数据]] 为块预计算的长度边界表，与下降集、good-ascent 集及本原索引表共同组成逐块 KL 支撑数据。`length_stop[l]` 记录首个长度大于或等于 `l` 的块元素位置，表末追加块大小 `size`。^[kl-support.md:16-21, kl-support.md:37-39]

## 构造与顺序不变量

长度边界表依赖块元素按长度非降排列。`KlSupport::new` 首先调用 `validate_topology`，检查每个元素的长度均存在，并强制验证长度顺序非降；这一前提由 [[KlSupport 的拓扑构造门控]] 保证。^[kl-support.md:32-39]

边界条件是“长度 ≥ `l`”，并不要求边界元素的长度恰好等于 `l`。构造过程中还会计算 `max_length`，随后通过 `let _ = max_length` 显式丢弃；来源将其记录为清理候选。^[kl-support.md:37-39]

## 测试与证据边界

来源列出的单元测试包含长度顺序违反非降要求时的构造拒绝测试，但 `new` 的成功路径没有单元测试覆盖。来源同时说明相关路径经 KL 层的 HPC 门覆盖，详见 [[KL 支撑层的测试覆盖与证据边界]]；这不等于 length-stop 构造结果已有专项单元测试验证。^[kl-support.md:57-62]

本页依据 `crates/atlas-real-group/src/kl_support.rs` 的结构性阅读材料，不构成 KL 支撑层的数学验收。材料中的上游行号仅转录自代码注释，未核对上游文件字节；该次知识维护未执行 Atlas、Cargo、测试或 benchmark。^[kl-support.md:9-12, kl-support.md:75-79]

## Sources

- [kl-support.md](../../sources/kl-support.md)：逐块 KL 支撑数据：KlSupport 与 RankFlags（kl_support.rs）。

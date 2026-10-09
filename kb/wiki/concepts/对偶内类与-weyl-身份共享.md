---
title: 对偶内类与 Weyl 身份共享
summary: 对偶内类构造翻转余根偏好并逐字母对偶化 Lie 类型，内容相等的对偶 datum 经内容弱驻留共享 Weyl 身份。
sources:
  - atlas-core-domain-construction.md
kind: concept
createdAt: "2026-10-09T14:28:30.612Z"
updatedAt: "2026-10-09T20:32:19.538Z"
tags:
  - 对偶内类
  - Weyl群
  - 身份语义
aliases:
  - 对偶内类与-weyl-身份共享
  - 对W身
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: 对偶内类与 Weyl 身份共享
summary: 对偶内类构造结合余根偏好翻转、逐字母对偶的 Lie 类型及对偶 datum 句柄，使内容等值的对偶 datum 经内容弱驻留共享 Weyl 身份。
sources:
  - atlas-core-domain-construction.md
kind: concept
tags:
  - 对偶内类
  - Weyl身份
  - 弱驻留
---

# 对偶内类与 Weyl 身份共享

`build_dual_inner_class` 采用 dual-identity 路径：构造对偶内类及其根数据句柄，并使内容等值的对偶 datum 经内容弱驻留（weak interning）共享 Weyl 身份。^[atlas-core-domain-construction.md:50-54]

## 对偶构造路径

`build_dual_inner_class` 位于 `domain_builtins.rs:2276`，结合 `dual_inner_class` 与对偶 datum 的 handle 完成构造。对偶 datum 的处理包含**余根偏好翻转**和**逐字母对偶的 Lie 类型**；来源将余根偏好翻转对应到上游 `RootSystem DualTag`（`rootdata.cpp:341`）。相关背景见 [[对偶根数据与对偶内类构造]]。^[atlas-core-domain-construction.md:50-54]

Weyl 身份共享以对偶 datum 的内容等值为依据，由内容弱驻留实现。构造管线材料将身份机制的进一步说明交由领域值包，本页据此保留这一共享契约；相关主题见 [[WeylAction 的等值与 datum 身份语义]]。^[atlas-core-domain-construction.md:50-54, atlas-core-domain-construction.md:69-72]

## 内类装配中的对偶侧

`build_inner_class_context` 按固定顺序装配：先运行带分类预算的 `classification_cached`，再运行受 `FIBER_BUDGET` 约束的 `StrongRealClassification::build`、`ExternalFormOrder::build`，以及受 `INTEGER_BUDGET` 约束的 `InnerClassLayout::build`，之后才构建对偶侧。详见 [[内类上下文的装配顺序与预算门]]。^[atlas-core-domain-construction.md:35-43]

对偶侧**只构建一次**，包括 `dual_inner_class`、对偶分类、对偶弱实形计数 `dual_form_count` 和 `dual_cartan_correspondence`。随后执行 `build_presentations`，并建立 `canonical_forms`：一个每个实形式对应一个 `Weak` 槽的 Mutex 向量，用作规范实形弱缓存的槽位表。^[atlas-core-domain-construction.md:39-43]

装配中的“对偶侧只建一次”与内容弱驻留描述不同层面的行为：前者规定装配流程中的构建次数，后者说明内容等值的对偶 datum 如何共享 Weyl 身份。^[atlas-core-domain-construction.md:37-43, atlas-core-domain-construction.md:50-54]

## 证据边界

本页依据构造管线的结构性阅读，不代表数学验收。来源中的上游行号属于实现方的移植陈述，构造兼容性仍以 HPC 差分门为准；字节数与哈希仅标识快照字节。相关验证要求见 [[HPC 验收证据链]]。^[atlas-core-domain-construction.md:9-14, atlas-core-domain-construction.md:73-75]

## Sources

- [atlas-core-domain-construction.md](../../sources/atlas-core-domain-construction.md) — 领域构造管线（domain_builtins.rs 1628–2527）。

---
title: 对偶内类与 Weyl 身份共享
summary: 对偶内类构造翻转余根偏好并逐字母对偶化 Lie 类型，内容相等的对偶 datum 经内容弱驻留共享 Weyl 身份。
sources:
  - atlas-core-domain-construction.md
kind: concept
createdAt: "2026-10-09T14:28:30.612Z"
updatedAt: "2026-10-09T22:14:33.409Z"
tags:
  - 对偶内类
  - Weyl群
  - 身份管理
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
summary: 对偶内类构造翻转余根偏好并逐字母对偶化 Lie 类型，内容等值的对偶 datum 经内容弱驻留共享 Weyl 身份。
sources:
  - atlas-core-domain-construction.md
kind: concept
tags:
  - 对偶内类
  - Weyl身份
  - 弱驻留
aliases:
  - 对偶内类与-weyl-身份共享
---

# 对偶内类与 Weyl 身份共享

`build_dual_inner_class` 采用 dual-identity 路径：结合对偶内类与对偶根数据（datum）句柄，使内容等值的对偶 datum 经内容弱驻留（weak interning）共享 Weyl 身份。^[atlas-core-domain-construction.md:50-54]

## 对偶构造路径

`build_dual_inner_class` 位于 `crates/atlas-core/src/domain_builtins.rs:2276`，调用 `dual_inner_class` 并构造对偶 datum 的 handle。对偶数据的处理包含**余根偏好翻转**和**逐字母对偶的 Lie 类型**；来源将前者对应到上游 `RootSystem DualTag`（`rootdata.cpp:341`）。相关背景见 [[对偶根数据与对偶内类构造]]。^[atlas-core-domain-construction.md:50-54]

Weyl 身份共享以对偶 datum 的内容等值为依据，通过内容弱驻留实现。来源将这一身份机制的详细说明交由领域值包，因此本页仅记录构造管线明确给出的共享契约；相关概念见 [[RootDatum 弱驻留与规范活对象身份]] 与 [[WeylAction 的等值与 datum 身份语义]]。^[atlas-core-domain-construction.md:50-54, atlas-core-domain-construction.md:69-72]

## 内类装配中的对偶侧

`build_inner_class_context` 按固定顺序装配内类上下文：先执行带分类预算的 `classification_cached`，再执行受 `FIBER_BUDGET` 约束的 `StrongRealClassification::build`、`ExternalFormOrder::build`，以及受 `INTEGER_BUDGET` 约束的 `InnerClassLayout::build`，随后构建对偶侧。详见 [[内类上下文的装配顺序与预算门]]。^[atlas-core-domain-construction.md:35-43]

在这一装配过程中，对偶侧**只构建一次**，包括 `dual_inner_class`、对偶分类、对偶弱实形计数 `dual_form_count` 和 `dual_cartan_correspondence`。来源将该流程对应到上游对偶 `InnerClass` 构造器（`innerclass.cpp:435`）。^[atlas-core-domain-construction.md:39-42]

对偶侧完成后，流程继续执行 `build_presentations`，并建立 `canonical_forms`：每个实形式对应一个 `Weak` 槽的 Mutex 向量，作为规范实形弱缓存的槽位表。相关主题见 [[实形式的规范弱缓存与属主身份]]。^[atlas-core-domain-construction.md:42-43]

## 证据边界

本页依据构造管线的结构性阅读，不代表数学验收。来源中的上游行号属于实现方的移植陈述，构造兼容性仍以 HPC 差分门为准；快照字节数与哈希仅标识所检查的字节。相关要求见 [[HPC 验收证据链]]。^[atlas-core-domain-construction.md:9-14, atlas-core-domain-construction.md:73-75]

## Sources

- [atlas-core-domain-construction.md](../../sources/atlas-core-domain-construction.md) — 领域构造管线（domain_builtins.rs 1628–2527）。

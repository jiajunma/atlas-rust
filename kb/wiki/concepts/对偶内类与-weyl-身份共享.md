---
title: 对偶内类与 Weyl 身份共享
summary: 对偶内类构造翻转余根偏好并逐字母对偶化 Lie 类型，内容相等的对偶 datum 经内容弱驻留共享 Weyl 身份。
sources:
  - atlas-core-domain-construction.md
kind: concept
createdAt: "2026-10-09T14:28:30.612Z"
updatedAt: "2026-10-10T00:16:49.300Z"
tags:
  - 内类
  - 对偶
  - Weyl群
aliases:
  - 对偶内类与-weyl-身份共享
  - 对W身
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: 对偶内类与 Weyl 身份共享
summary: 对偶内类构造翻转余根偏好并逐字母对偶化 Lie 类型，内容等值的对偶 datum 经内容弱驻留共享 Weyl 身份。
sources:
  - atlas-core-domain-construction.md
kind: concept
tags:
  - 对偶内类
  - Weyl群
  - 身份管理
aliases:
  - 对偶内类与-weyl-身份共享
---

# 对偶内类与 Weyl 身份共享

`build_dual_inner_class` 采用 dual-identity 路径，将对偶内类与对偶根数据（datum）句柄结合起来。**内容等值的对偶 datum 经内容弱驻留（weak interning）共享 Weyl 身份**，这是来源明确给出的构造契约。^[atlas-core-domain-construction.md:50-54]

## 对偶构造路径

`build_dual_inner_class` 位于 `crates/atlas-core/src/domain_builtins.rs:2276`，通过 `dual_inner_class` 构造对偶内类，并取得对偶 datum 的 handle。对偶 datum 的处理包括余根偏好翻转，以及 Lie 类型的逐字母对偶化；来源将余根偏好翻转对应到上游 `RootSystem DualTag`（`rootdata.cpp:341`）。相关背景见 [[对偶根数据与对偶内类构造]]。^[atlas-core-domain-construction.md:50-54]

内容弱驻留使内容等值的对偶 datum 共享 Weyl 身份。来源将身份机制的详细说明交由领域值包，本页据此保留这一共享关系，不扩展其内部实现；相关主题见 [[RootDatum 弱驻留与规范活对象身份]]。^[atlas-core-domain-construction.md:50-54, atlas-core-domain-construction.md:69-72]

## 内类装配中的对偶侧

`build_inner_class_context` 按固定顺序装配上下文：先执行受分类预算约束的 `classification_cached`，再执行受 `FIBER_BUDGET` 约束的 `StrongRealClassification::build`、`ExternalFormOrder::build`，以及受 `INTEGER_BUDGET` 约束的 `InnerClassLayout::build`，随后构建对偶侧。详见 [[内类上下文的装配顺序与预算门]]。^[atlas-core-domain-construction.md:35-43]

在这次装配中，对偶侧**只构建一次**，包括 `dual_inner_class`、对偶分类、对偶弱实形计数 `dual_form_count` 和 `dual_cartan_correspondence`。来源将这一流程对应到上游对偶 `InnerClass` 构造器（`innerclass.cpp:435`）；计数主题可参见 [[对偶实形式的分层计数管线]]。^[atlas-core-domain-construction.md:39-42]

对偶侧完成后，流程继续执行 `build_presentations`，并建立 `canonical_forms`。后者是每个实形式对应一个 `Weak` 槽的 Mutex 向量，用作规范实形弱缓存的槽位表；`build_real_form` 命中该缓存时共享上下文，而自定义实形构造总是新建属主。^[atlas-core-domain-construction.md:42-43, atlas-core-domain-construction.md:58-60]

## 证据边界

本页依据构造管线的结构性阅读，不代表数学验收。来源中的上游行号属于实现方的移植陈述，构造兼容性仍以 HPC 差分门为准；快照字节数与哈希仅标识所检查的字节。相关要求见 [[HPC 验收证据链]]。^[atlas-core-domain-construction.md:9-14, atlas-core-domain-construction.md:73-75]

## Sources

- [atlas-core-domain-construction.md](../../sources/atlas-core-domain-construction.md) — 领域构造管线（domain_builtins.rs 1628–2527）。

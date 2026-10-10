---
title: dual 预热历史与 Weyl 群兼容性
summary: canonical dual 仅在目标尚冷时接收源抽象群身份，预热目标不被覆盖，因此构造历史会影响 Weyl 元素兼容性。
sources:
  - atlas-core-domain-values.md
kind: concept
createdAt: "2026-10-09T15:16:17.578Z"
updatedAt: "2026-10-10T01:53:04.930Z"
tags:
  - 对偶根数据
  - Weyl群
  - 惰性初始化
aliases:
  - dual-预热历史与-weyl-群兼容性
  - D预W群
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# dual 预热历史与 Weyl 群兼容性

Weyl 元素的兼容性由抽象 Weyl 群的 `Arc` 身份决定，而非根数据的结构等值或局部坐标缓存。`dual` 仅向尚未初始化抽象群身份的 canonical dual 目标共享源群身份；独立预热的目标不会被覆盖，因此预热历史会影响后续 Weyl 二元关系和乘积的兼容性。^[atlas-core-domain-values.md:28-33, atlas-core-domain-values.md:53-56, atlas-core-domain-values.md:89-91]

## canonical dual 的冷共享规则

`share_group_into_if_cold` 实现根数据 `dual` 的身份共享：目标 canonical dual 仍冷时，将源的抽象群装入目标；目标已经独立预热时，保留其已有身份，与源保持不兼容。并发共享发生竞争时，仅一个候选被发布，另一个被丢弃。^[atlas-core-domain-values.md:30-33]

共享抽象群身份与共享局部坐标内核是不同的机制。`DatumWeylIdentity` 包含惰性的 `DatumWeylKernel` 和抽象群身份：前者保存 owner 局部坐标中的枚举根系，后者提供 canonical-word 接口。两个单元均不回指根数据 handle，因而不形成所有权环，参见 [[Rust Weyl 内核与抽象群的无环所有权模型]]。^[atlas-core-domain-values.md:24-29]

## 身份的初始化与生命周期

`WeylIdentityCell<T>` 使用 `OnceLock` 和初始化互斥锁管理身份。初始化仅在成功时落定，失败不占用单元；双重检查与锁防止并发重复初始化，锁毒化或二次初始化报告 `StructureError::RepInvariantViolation`。相关机制见 [[可失败重试的惰性身份初始化]]。^[atlas-core-domain-values.md:21-23]

`DATUM_WEYL_IDENTITIES` 是按完整内容索引的全局弱引用表。等值 owner 仍存活时，新构造复用其身份单元；全部强引用消失后，槽位过期，后来等值构造重新开始。表超过 4096 项时才扫描死槽。唯一构造入口 `RootDatumHandle::interned` 保证等值活数据共享 Weyl 身份，参见 [[根数据的 Weyl 身份与弱驻留机制]]。^[atlas-core-domain-values.md:34-41]

根数据的结构等值本身不包含这些缓存状态。`RootDatumHandle::PartialEq` 刻意忽略 Weyl 身份缓存，只比较 `datum`、`lie_type`、`isogeny` 和 `prefers_coroots`；因此，结构比较不能替代抽象群身份的兼容检查。^[atlas-core-domain-values.md:41-43, atlas-core-domain-values.md:53-56]

## 兼容检查与跨坐标运算

`weyl_group_compatible` 使用 `Arc::ptr_eq(group)` 判定兼容性。`require_weyl_compatible` 在无值门之前检查二元关系和乘积，不兼容时报 `Weyl group mismatch`；即使调用结果不被使用，也必须执行该检查。^[atlas-core-domain-values.md:87-91, atlas-core-domain-values.md:123-124]

兼容元素仍可能使用不同的局部坐标。等值判断先确认抽象群 `Arc` 同一性，再把右元素的 canonical 外生成元词在左系统中重放后比较，允许辫子等价词跨坐标相等。跨坐标乘积同样要求重放外生成元词；外来根置换不得直接比较或复合，参见 [[Weyl 元素兼容性与跨坐标词重放]]。^[atlas-core-domain-values.md:80-82, atlas-core-domain-values.md:123-124]

`WeylEltContext` 携带的内部生成元重编号固定上游 canonical-word 的选择。`WeylEltValue` 在构造时计算并冻结 canonical 既约词，`Display` 与 `word` 只读取该词，参见 [[Weyl 元素值的规范词冻结]]。^[atlas-core-domain-values.md:53-58, atlas-core-domain-values.md:92-92]

## 证据边界

本页来源是对 `domain_builtins.rs` 相关区域的结构性阅读。来源引用的 Weyl owner/dual 修复语义验收限定于 A1，验收记录为 `tests/reference/hpc/math_weyl_context_core_after_v5_acceptance_2026_10_06.json`；派发表、测试及内类与实形管线实现不在本包覆盖范围内，不能据此认定更高秩行为已完成验收。^[atlas-core-domain-values.md:9-17, atlas-core-domain-values.md:121-126]

## Sources

- [atlas-core-domain-values.md](../../sources/atlas-core-domain-values.md) — 领域值与 Weyl 身份（domain_builtins.rs 上部）。

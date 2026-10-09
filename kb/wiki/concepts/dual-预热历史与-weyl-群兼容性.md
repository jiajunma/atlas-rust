---
title: dual 预热历史与 Weyl 群兼容性
summary: canonical dual 仅在目标抽象群尚未初始化时共享源身份，预热目标不会被覆盖，因此兼容性受构造历史影响。
sources:
  - atlas-core-domain-values.md
kind: concept
createdAt: "2026-10-09T15:16:17.578Z"
updatedAt: "2026-10-09T20:34:12.735Z"
tags:
  - Weyl群
  - 对偶
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
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

# dual 预热历史与 Weyl 群兼容性

Weyl 元素的兼容性由抽象 Weyl 群的 `Arc` 身份决定，而不是根数据句柄的结构相等或局部坐标缓存相同。`dual()` 是否共享这一身份，取决于 canonical dual 目标是否已经预热，因此构造历史会影响后续二元关系与乘积是否被接受。^[atlas-core-domain-values.md:28-33, atlas-core-domain-values.md:53-56, atlas-core-domain-values.md:89-91]

## canonical dual 的预热规则

`share_group_into_if_cold` 实现 `root_datum_value::dual` 的身份共享规则：仅当 canonical dual 目标仍冷时，才把源的抽象群身份装入目标；已预热的目标永不被覆盖，并保留与源不兼容的身份。并发共享发生竞争时，只有一个候选被发布，另一个被丢弃。^[atlas-core-domain-values.md:30-33]

这项共享针对抽象群，而非根数据的局部坐标。`DatumWeylIdentity` 将惰性的 `DatumWeylKernel` 与 `AbstractWeylGroup` 分开保存：前者缓存 owner 的枚举根系，后者提供 canonical-word 接口，并在上游共享 `WeylGroup` 指针的场合共享。两个单元都不回指 handle，因此不形成所有权环。^[atlas-core-domain-values.md:24-29]

## 弱驻留与生命周期

`DATUM_WEYL_IDENTITIES` 是按完整内容索引的全局弱引用表。等值 owner 仍存活时，新构造会复用其身份单元；所有强引用消失后，槽位过期，后续等值构造重新开始。表的大小超过 4096 时才清理死槽。`RootDatumHandle::interned` 是唯一构造入口，保证等值的活数据共享一个 Weyl 身份。^[atlas-core-domain-values.md:34-41]

根数据本身的结构相等与 Weyl 兼容性保持分离。`RootDatumHandle::PartialEq` 刻意忽略 Weyl 身份缓存，只比较 `datum`、`lie_type`、`isogeny` 和 `prefers_coroots`；`Debug` 同样不纳入身份缓存。^[atlas-core-domain-values.md:41-43]

身份单元由 `WeylIdentityCell<T>` 管理，使用 `OnceLock` 与初始化互斥锁。初始化仅在成功时落定，失败不会占用单元；双重检查与锁防止并发重复初始化，毒化或二次初始化则报告 `StructureError::RepInvariantViolation`。^[atlas-core-domain-values.md:21-23]

## 兼容检查与跨坐标比较

`weyl_group_compatible` 使用 `Arc::ptr_eq(group)` 判定兼容；`require_weyl_compatible` 在无值门之前检查二元关系与乘积，不兼容时报告 `Weyl group mismatch`。即使调用方不需要结果值，也不能省略这一检查。^[atlas-core-domain-values.md:89-91, atlas-core-domain-values.md:123-124]

共享抽象群身份后，元素仍可能采用不同的 owner 局部坐标。`weyl_elements_equal` 先检查抽象群身份，再将右元素的 canonical 外生成元词在左侧系统中重放后比较；辫子等价词可以跨坐标相等，外来根置换绝不直接比较或复合。跨坐标乘积也遵循外生成元词重放规则，参见 [[Weyl 元素兼容性与跨坐标词重放]]。^[atlas-core-domain-values.md:80-82, atlas-core-domain-values.md:123-124]

`WeylEltContext` 携带的内部生成元重编号固定上游 canonical-word 的选择。`WeylEltValue` 在构造时计算并冻结 canonical 既约词，因此 `Display` 与 `word` 都是纯读取操作。相关词表示见 [[Weyl 元素的规范词]]。^[atlas-core-domain-values.md:53-58, atlas-core-domain-values.md:92-92]

## 证据范围

本页依据 `domain_builtins.rs` 相关区域的结构性阅读。来源指出 owner/dual 修复具有 A1 限定的 HPC 语义验收，并引用 `tests/reference/hpc/math_weyl_context_core_after_v5_acceptance_2026_10_06.json`；这不构成对更高秩情形的验收声明。原始差异发现与 A1 回归金标位于 `tests/math/generics/weyl_context_core_*`。^[atlas-core-domain-values.md:9-17, atlas-core-domain-values.md:121-122]

来源未覆盖内类与实形管线实现、完整派发表及测试正文。因此，本页说明身份与预热机制，不据此补充内类构造触发 dual 的具体流程或更广泛的测试结论。^[atlas-core-domain-values.md:125-126]

## Sources

- [领域值与 Weyl 身份（domain_builtins.rs 上部）](../../sources/atlas-core-domain-values.md)

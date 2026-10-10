---
title: 根数据的 Weyl 身份与弱驻留机制
summary: DatumWeylIdentity 分离惰性坐标 kernel 与抽象群身份，按完整内容弱驻留；等值活 owner 共享身份，强引用消失后后续构造重新初始化。
sources:
  - atlas-core-domain-values.md
kind: concept
createdAt: "2026-10-09T14:30:32.475Z"
updatedAt: "2026-10-10T01:44:36.547Z"
tags:
  - Weyl群
  - 身份管理
  - 弱驻留
aliases:
  - 根数据的-weyl-身份与弱驻留机制
  - 根W身
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# 根数据的 Weyl 身份与弱驻留机制

`DatumWeylIdentity` 管理每个根数据属主的 Weyl 身份，将惰性的局部坐标缓存与抽象 Weyl 群身份分离。等值且仍存活的根数据通过弱驻留共享身份单元；Weyl 元素是否兼容，则由抽象群的 `Arc` 指针同一性决定，不能用根数据结构相等或坐标缓存相同替代。^[atlas-core-domain-values.md:24-37, atlas-core-domain-values.md:53-56]

## 身份组成与惰性初始化

`DatumWeylKernel` 保存该根数据的枚举根系，属于属主局部坐标缓存；克隆、别名及 `root_datum(WeylElt)` 往返共享此缓存。`AbstractWeylGroup` 提供规范词（canonical-word）接口，其共享范围对应上游共享 `WeylGroup` 指针的情形。`DatumWeylIdentity` 组合这两部分，两个单元均不回指 handle，因此不形成所有权环，参见 [[Rust Weyl 内核与抽象群的无环所有权模型]]。^[atlas-core-domain-values.md:24-29]

底层 `WeylIdentityCell<T>` 使用 `OnceLock` 与初始化互斥锁。`get_or_try_init` 仅在成功时保存结果，失败不占用单元；双重检查与锁防止并发重复初始化。锁毒化或二次初始化返回 `StructureError::RepInvariantViolation`，详见 [[可失败重试的惰性身份初始化]]。^[atlas-core-domain-values.md:21-23]

## 完整内容弱驻留与生命周期

`DATUM_WEYL_IDENTITIES` 是按完整内容组织的全局弱引用表，对应上游 `root_datum_value` 的弱驻留机制。等值属主仍存活时，新构造复用其身份单元；所有强引用消失后，槽位过期，之后的等值构造重新开始。表规模超过 4096 时才清扫失效槽位。^[atlas-core-domain-values.md:34-37]

`RootDatumHandle::interned` 是唯一构造入口，保证等值的活数据共享 Weyl 身份。其 `PartialEq` 刻意忽略 Weyl 身份缓存，只比较 `datum`、`lie_type`、`isogeny` 和 `prefers_coroots`；`Debug` 同样忽略缓存。相关机制见 [[RootDatum 弱驻留与规范活对象身份]]。^[atlas-core-domain-values.md:39-45]

## 对偶构造与预热历史

对偶构造通过 `share_group_into_if_cold` 共享抽象群：只有目标 canonical dual 仍处于冷状态时，才把源的抽象群装入目标。已预热的目标不会被覆盖，并与源保持不兼容；并发共享发生竞态时，只有一个候选被发布，另一个被丢弃。因此，对偶目标的预热历史会影响兼容性，详见 [[dual 预热历史与 Weyl 群兼容性]]。^[atlas-core-domain-values.md:30-33]

## 元素兼容与跨坐标操作

`WeylEltContext` 包含属主局部坐标 kernel，以及携带内部生成元重编号的抽象群身份；该重编号固定上游规范词的选择。`WeylEltValue` 在构造时计算并冻结规范既约词，使 `Display` 与 `word` 成为纯读取操作，参见 [[Weyl 元素值的规范词冻结]]。^[atlas-core-domain-values.md:53-58, atlas-core-domain-values.md:92-92]

`weyl_group_compatible` 使用 `Arc::ptr_eq(group)` 判断兼容性。二元关系与乘积通过 `require_weyl_compatible` 检查，不兼容时报告 `Weyl group mismatch`。该检查发生在无值门之前，即使求值不需要返回值，也必须检查兼容性。^[atlas-core-domain-values.md:87-91, atlas-core-domain-values.md:123-124]

Weyl 元素等值比较先检查抽象群 `Arc` 同一性，再将右元素的规范外生成元词在左侧系统中重放后比较，从而识别跨坐标的辫子等价词。外来根置换不会直接参与比较或复合；跨坐标乘积同样遵循外生成元词重放规则，详见 [[Weyl 元素兼容性与跨坐标词重放]]。^[atlas-core-domain-values.md:80-82, atlas-core-domain-values.md:123-124]

## 证据范围

本页依据来源对 `domain_builtins.rs` 相关区域的结构性阅读。来源将 Weyl owner/dual 修复的语义验收限定于 A1，指向 `tests/reference/hpc/math_weyl_context_core_after_v5_acceptance_2026_10_06.json`；v8 原始差异发现与 A1 回归金标位于 `tests/math/generics/weyl_context_core_*`。这些记录不构成更广范围的语义验收。^[atlas-core-domain-values.md:9-17, atlas-core-domain-values.md:119-124]

来源包未覆盖 `coerce`、内类与实形管线实现、派发表及测试；`weyl_subgroup.rs` 也待独立分包。^[atlas-core-domain-values.md:125-126]

## Sources

- [atlas-core-domain-values.md — 领域值与 Weyl 身份（domain_builtins.rs 上部）](../../sources/atlas-core-domain-values.md)

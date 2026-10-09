---
title: Weyl 元素兼容性与跨坐标词重放
summary: Weyl 兼容性由抽象群 Arc 身份决定，跨坐标等值在左系统重放右元素冻结的 canonical 外生成元词，二元关系及乘积在无值门前也检查兼容。
sources:
  - atlas-core-domain-values.md
kind: concept
createdAt: "2026-10-09T14:30:45.385Z"
updatedAt: "2026-10-09T22:16:38.434Z"
tags:
  - Weyl群
  - 兼容性
  - 规范词
aliases:
  - weyl-元素兼容性与跨坐标词重放
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: Weyl 元素兼容性与跨坐标词重放
summary: Weyl 兼容性由抽象群 Arc 同一性决定；跨坐标等值与乘积重放外生成元词，二元关系和乘积在无值门之前检查兼容。
sources:
  - atlas-core-domain-values.md
kind: concept
tags:
  - Weyl群
  - 兼容契约
  - 词重放
aliases:
  - weyl-元素兼容性与跨坐标词重放
provenanceState: extracted
---

# Weyl 元素兼容性与跨坐标词重放

Weyl 元素的兼容性由抽象 Weyl 群的 `Arc` 身份决定，不能以根数据句柄的结构相等或局部坐标 kernel 相同替代。兼容元素的跨坐标等值与乘积通过重放外生成元词实现，外来根置换不能直接比较或复合。^[atlas-core-domain-values.md:53-56, atlas-core-domain-values.md:80-82, atlas-core-domain-values.md:123-124]

## 局部坐标与抽象群身份

`WeylEltContext` 包含 owner 局部坐标 kernel 与抽象群身份。`DatumWeylKernel` 保存该根数据的枚举根系，由克隆、别名及 `root_datum(WeylElt)` 往返共享；抽象群携带内部生成元重编号，以固定上游 canonical-word 的选择。`DatumWeylIdentity` 惰性保存这两部分，两个单元都不回指 handle，因此不形成所有权环，详见 [[Rust Weyl 内核与抽象群的无环所有权模型]]。^[atlas-core-domain-values.md:24-29, atlas-core-domain-values.md:53-56]

`RootDatumHandle::interned` 是唯一构造入口，等值的活数据共享一个 Weyl 身份。不过，`RootDatumHandle::PartialEq` 刻意忽略身份缓存，只比较 `datum`、`lie_type`、`isogeny` 与 `prefers_coroots`。根数据的结构等值与 Weyl 群身份兼容因此是不同的判定。^[atlas-core-domain-values.md:41-43, atlas-core-domain-values.md:53-56]

## 兼容检查与失败顺序

`weyl_group_compatible` 使用 `Arc::ptr_eq(group)` 判断兼容性。`require_weyl_compatible` 在无值门之前检查二元关系与乘积，不兼容时报告 `Weyl group mismatch`。即使调用处不需要结果值，也必须执行这一检查；相关契约见 [[Weyl 元素的可失败关系与跨坐标运算]]。^[atlas-core-domain-values.md:89-91, atlas-core-domain-values.md:123-124]

## 构造时冻结的词与跨坐标重放

`WeylEltValue` 保存元素、构造上下文，以及构造时冻结的 canonical 既约 word。`weyl_elt_value` 只在构造时计算一次该词，后续 `Display` 与 `word` 访问都是纯读，详见 [[Weyl 元素值的规范词冻结]]。^[atlas-core-domain-values.md:57-58, atlas-core-domain-values.md:92-92]

`weyl_elements_equal` 先检查抽象群 `Arc` 同一性，再将右元素的 canonical 外生成元 word 在左侧系统中重放，最后比较元素。这使辫子等价词能够跨坐标判为相等，且不直接比较外来根置换。跨坐标乘积也遵循重放外生成元词的规则。^[atlas-core-domain-values.md:80-82, atlas-core-domain-values.md:123-124]

词输入另有合法性检查：`check_weyl_word` 先将条目转换为 unsigned，拒绝负数，再要求条目严格小于半单秩。诊断格式为 `Illegal Weyl word entry i (should be <r)`，相关主题见 [[Weyl 词与单生成元的整数校验差异]]。^[atlas-core-domain-values.md:93-95]

## 对偶预热与身份生命周期

对偶构造通过 `share_group_into_if_cold` 共享抽象群身份：只有目标 canonical dual 仍冷时，才将源的抽象群装入目标。已预热的目标永不被覆盖，并与源保持不兼容；并发共享竞争中只有一个结果发布，另一个被丢弃。预热历史因此影响后续兼容性，详见 [[dual 预热历史与 Weyl 群兼容性]]。^[atlas-core-domain-values.md:30-33]

全局 `DATUM_WEYL_IDENTITIES` 表按完整内容保存弱引用。等值 owner 仍存活时，新构造复用其身份单元；全部强引用消失后，槽位过期，后来等值构造重新开始。表大小超过 4096 时才扫描失效槽位，相关机制见 [[RootDatum 弱驻留与规范活对象身份]]。^[atlas-core-domain-values.md:34-37]

身份单元 `WeylIdentityCell<T>` 使用 `OnceLock` 与初始化互斥锁，只在初始化成功时落定，失败不占用单元。双重检查与锁防止并发重复初始化；毒化或二次初始化报告 `StructureError::RepInvariantViolation`，详见 [[可失败重试的惰性身份初始化]]。^[atlas-core-domain-values.md:21-23]

## 证据范围

本页依据 `domain_builtins.rs` 相关区域的结构性阅读。来源将 Weyl owner/dual 修复的语义验收限定为 A1，并指向 `tests/reference/hpc/math_weyl_context_core_after_v5_acceptance_2026_10_06.json`；v8 原始差异发现与 A1 回归金标位于 `tests/math/generics/weyl_context_core_*`。本来源未覆盖派发表与测试正文，不能据此扩大验收范围。^[atlas-core-domain-values.md:9-17, atlas-core-domain-values.md:121-126]

## Sources

- [atlas-core-domain-values.md](../../sources/atlas-core-domain-values.md)：领域值与 Weyl 身份（domain_builtins.rs 上部）。

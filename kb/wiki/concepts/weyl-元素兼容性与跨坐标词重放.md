---
title: Weyl 元素兼容性与跨坐标词重放
summary: 兼容性由抽象群 Arc 身份决定；跨坐标比较在左系统重放右元素冻结的外生成元规范词，禁止直接比较或复合外来根置换。
sources:
  - atlas-core-domain-values.md
kind: concept
createdAt: "2026-10-09T14:30:45.385Z"
updatedAt: "2026-10-10T01:53:27.459Z"
tags:
  - Weyl群
  - 规范词
  - 跨坐标运算
aliases:
  - weyl-元素兼容性与跨坐标词重放
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# Weyl 元素兼容性与跨坐标词重放

Weyl 元素的兼容性由抽象 Weyl 群的 `Arc` 身份决定，不能用根数据句柄的结构相等或局部坐标 kernel 相同替代。兼容元素的跨坐标等值与乘积通过重放外生成元词实现；外来根置换不得直接比较或复合。^[atlas-core-domain-values.md:53-56, atlas-core-domain-values.md:80-82, atlas-core-domain-values.md:123-124]

## 局部坐标与抽象群身份

`WeylEltContext` 包含属主局部坐标 kernel 和抽象群身份。`DatumWeylKernel` 保存该根数据的枚举根系，由克隆、别名及 `root_datum(WeylElt)` 往返共享；抽象群身份携带内部生成元重编号，用于固定上游规范词的选择。`DatumWeylIdentity` 惰性保存这两部分，两个单元均不回指句柄，因此不形成所有权环，参见 [[Rust Weyl 内核与抽象群的无环所有权模型]]。^[atlas-core-domain-values.md:24-29, atlas-core-domain-values.md:53-56]

`RootDatumHandle::interned` 是唯一构造入口，使等值的活数据共享 Weyl 身份。但句柄的 `PartialEq` 刻意忽略身份缓存，只比较 `datum`、`lie_type`、`isogeny` 与 `prefers_coroots`。因此，根数据的结构等值与 Weyl 元素的身份兼容属于不同判定，详见 [[领域值的结构等值与属主身份]]。^[atlas-core-domain-values.md:41-43, atlas-core-domain-values.md:53-56]

## 兼容检查与失败顺序

`weyl_group_compatible` 使用 `Arc::ptr_eq(group)` 判断兼容性。`require_weyl_compatible` 在二元关系与乘积的无值门之前检查，不兼容时报告 `Weyl group mismatch`。即使调用处不需要结果值，也必须检查兼容性，参见 [[Weyl 元素的可失败关系与跨坐标运算]]。^[atlas-core-domain-values.md:89-91, atlas-core-domain-values.md:123-124]

## 规范词冻结与跨坐标重放

`WeylEltValue` 保存元素、构造上下文及构造时冻结的规范既约词。构造函数 `weyl_elt_value` 只计算一次该词，后续 `Display` 与 `word` 访问均为纯读，详见 [[Weyl 元素值的规范词冻结]]。^[atlas-core-domain-values.md:57-58, atlas-core-domain-values.md:92-92]

`weyl_elements_equal` 先检查抽象群 `Arc` 同一性，再把右元素的外生成元规范词放到**左元素的坐标系统中重放**，随后比较。右元素提供词，左系统提供词的坐标解释；这一过程允许辫子等价词跨坐标判为相等，而不直接比较外来根置换。^[atlas-core-domain-values.md:80-82]

跨坐标乘积同样遵循重放外生成元词的规则，不能直接复合来自不同坐标系统的根置换。来源将“兼容性取决于抽象群身份、跨坐标等值与乘积重放外生成元词、无值级仍检查二元关系兼容性”列为共同的设计约束。^[atlas-core-domain-values.md:80-82, atlas-core-domain-values.md:123-124]

## 对偶预热与身份生命周期

对偶构造通过 `share_group_into_if_cold` 共享抽象群身份：只有目标 canonical dual 仍冷时，才将源的抽象群装入目标。已预热目标不会被覆盖，并与源保持不兼容；并发共享竞争中只有一个结果发布。因此，对偶预热历史会影响后续兼容性，详见 [[dual 预热历史与 Weyl 群兼容性]]。^[atlas-core-domain-values.md:30-33]

全局 `DATUM_WEYL_IDENTITIES` 表按完整内容保存弱引用。等值属主仍存活时，新构造复用其身份单元；全部强引用消失后，槽位过期，后来等值构造重新开始。表大小超过 4096 时才扫描失效槽位，参见 [[根数据的 Weyl 身份与弱驻留机制]]。^[atlas-core-domain-values.md:34-37]

身份单元 `WeylIdentityCell<T>` 使用 `OnceLock`、初始化互斥锁和双重检查避免并发重复初始化。只有成功初始化才会落定，失败不占用单元；毒化或二次初始化报告 `StructureError::RepInvariantViolation`，详见 [[可失败重试的惰性身份初始化]]。^[atlas-core-domain-values.md:21-23]

## 与词条目校验的区别

`check_weyl_word` 检查输入词条目的合法性：先转为无符号整数并拒绝负数，再要求条目小于半单秩，越界诊断为 `Illegal Weyl word entry i (should be <r)`。这是词输入的整数与范围约束，与基于抽象群身份的兼容检查不同，参见 [[Weyl 词条目的有序校验]]。^[atlas-core-domain-values.md:89-95]

## 证据范围

本页依据 `domain_builtins.rs` 相关区域的结构性阅读。来源将 Weyl 属主与对偶修复的语义验收限定为 A1，并指向 `tests/reference/hpc/math_weyl_context_core_after_v5_acceptance_2026_10_06.json`；派发表与测试正文不在该包覆盖范围内，不能据此扩大验收结论。^[atlas-core-domain-values.md:9-17, atlas-core-domain-values.md:121-126]

## Sources

- [atlas-core-domain-values.md](../../sources/atlas-core-domain-values.md) — 领域值与 Weyl 身份（domain_builtins.rs 上部）。

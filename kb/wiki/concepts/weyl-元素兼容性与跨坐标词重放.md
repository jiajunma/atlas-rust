---
title: Weyl 元素兼容性与跨坐标词重放
summary: Weyl 元素比较先检查抽象群 Arc 身份，再将右操作数的外生成元规范词在左属主坐标系重放后比较。
sources:
  - atlas-core-domain-seams.md
  - atlas-core-domain-values.md
kind: concept
createdAt: "2026-10-09T14:30:45.385Z"
updatedAt: "2026-10-10T01:44:41.660Z"
tags:
  - Weyl群
  - 身份语义
  - 坐标转换
aliases:
  - weyl-元素兼容性与跨坐标词重放
confidence: 1
provenanceState: merged
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# Weyl 元素兼容性与跨坐标词重放

Weyl 元素的兼容性由抽象 Weyl 群的 `Arc` 身份决定，不能用根数据句柄的结构相等或局部坐标 kernel 相同替代。兼容元素的跨坐标等值与乘积通过重放外生成元规范词实现；外来根置换不得直接比较或复合。^[atlas-core-domain-values.md:53-56, atlas-core-domain-values.md:80-82, atlas-core-domain-values.md:123-124]

## 局部坐标与群身份

`WeylEltContext` 同时包含属主局部坐标 kernel 和抽象群身份。kernel 保存该根数据的枚举根系，由克隆、别名及 `root_datum(WeylElt)` 往返共享；抽象群身份携带内部生成元重编号，用于固定规范词的选择。`DatumWeylIdentity` 惰性保存这两部分，两个单元均不回指句柄，因此不形成所有权环，详见 [[Rust Weyl 内核与抽象群的无环所有权模型]]。^[atlas-core-domain-values.md:24-29, atlas-core-domain-values.md:53-56]

`RootDatumHandle::interned` 是唯一构造入口，使等值的活数据共享 Weyl 身份。但句柄的 `PartialEq` 刻意忽略身份缓存，只比较 `datum`、`lie_type`、`isogeny` 与 `prefers_coroots`。根数据的结构等值与 Weyl 元素的身份兼容是不同判定，参见 [[领域值的结构等值与属主身份]]。^[atlas-core-domain-values.md:41-43, atlas-core-domain-values.md:53-56]

## 兼容检查与失败顺序

`weyl_group_compatible` 使用 `Arc::ptr_eq(group)` 判断兼容性。`require_weyl_compatible` 在二元关系与乘积的无值门之前执行检查，不兼容时报告运行时错误 `Weyl group mismatch`。因此，即使调用处不需要结果值，也不能省略兼容检查，详见 [[Weyl 元素的可失败关系与跨坐标运算]]。^[atlas-core-domain-values.md:89-91, atlas-core-domain-values.md:123-124, atlas-core-domain-seams.md:85-87]

## 规范词冻结与重放方向

`WeylEltValue` 保存元素、构造上下文及构造时冻结的规范约化词。冻结点 `weyl_elt_value` 一次性计算该词，后续 `Display` 与 `word` 访问均为纯读，参见 [[Weyl 元素值的规范词冻结]]。^[atlas-core-domain-values.md:57-58, atlas-core-domain-seams.md:93-95]

`weyl_replayed_in_left` 取右操作数的**外生成元规范词**，在左属主的坐标系中逐步调用 `right_multiply_simple` 重放。右元素提供词，左上下文提供解释该词的坐标系统；整个过程不直接比较或复合外来根置换。^[atlas-core-domain-seams.md:88-90]

`weyl_elements_equal` 先检查兼容性，再将重放结果与左元素的词级置换比较，其消费方是 Weyl `=` 关系臂。这使辫子等价词能够跨坐标判为相等。跨坐标乘积同样遵循重放外生成元词的规则。^[atlas-core-domain-seams.md:91-92, atlas-core-domain-values.md:80-82, atlas-core-domain-values.md:123-124]

## 对偶预热与身份生命周期

对偶构造通过 `share_group_into_if_cold` 共享抽象群身份：只有目标 canonical dual 尚未预热时，才将源的抽象群装入目标。已预热目标不会被覆盖，并与源保持不兼容；并发共享竞争中只有一个结果发布。因此，预热历史会影响后续兼容性，详见 [[dual 预热历史与 Weyl 群兼容性]]。^[atlas-core-domain-values.md:30-33]

全局 `DATUM_WEYL_IDENTITIES` 表按完整内容保存弱引用。等值属主仍存活时，新构造复用其身份单元；全部强引用消失后，槽位过期，后来等值构造重新开始。表大小超过 4096 时才扫描失效槽位，参见 [[根数据的 Weyl 身份与弱驻留机制]]。^[atlas-core-domain-values.md:34-37]

身份单元 `WeylIdentityCell<T>` 通过 `OnceLock`、初始化互斥锁与双重检查避免并发重复初始化。只有成功初始化才会落定，失败不占用单元；锁毒化或二次初始化报告 `StructureError::RepInvariantViolation`，详见 [[可失败重试的惰性身份初始化]]。^[atlas-core-domain-values.md:21-23]

## 与输入合法性校验的区别

词条目是否合法由 `check_weyl_word` 检查：逐条先经 `as_integer`，再拒绝负数、检查能否收窄到 `u64`，最后要求条目小于半单秩。相应诊断依次为 `Negative integer where unsigned is required`、`Integer value to big for conversion` 和 `Illegal Weyl word entry {i} (should be <{r})`；其中 `to big` 是来源记录的原始措辞，参见 [[Weyl 词条目的有序校验]]。^[atlas-core-domain-seams.md:96-102]

单生成元的 `check_weyl_generator` 则先收窄到 `i32`，再检查范围；负 `i32` 转为 `usize` 失败时，也报告 `Generator {g} out of range for Weyl group (should be <{r})`。这些整数与范围检查有别于抽象群身份兼容检查，详见 [[Weyl 词与单生成元的整数校验差异]]。^[atlas-core-domain-seams.md:103-107, atlas-core-domain-values.md:89-95]

## 证据范围

本页依据 `domain_builtins.rs` 相关区域的结构性阅读。领域值来源将 Weyl 属主与对偶修复的语义验收限定为 A1，并指向 `tests/reference/hpc/math_weyl_context_core_after_v5_acceptance_2026_10_06.json`；派发表与测试正文不在该包覆盖范围内，不能据此扩大验收结论。^[atlas-core-domain-values.md:9-17, atlas-core-domain-values.md:121-126]

领域接缝来源补充了重放助手、冻结点与校验顺序，但不声称语言或数学验收。上游 C++／CWEB 行号转述自 Rust 源码注释，未独立重读上游，可能随版本漂移；校验助手的诊断措辞以 HPC 语料门为行为权威。^[atlas-core-domain-seams.md:9-19, atlas-core-domain-seams.md:115-118]

## Sources

- [atlas-core-domain-values.md](../../sources/atlas-core-domain-values.md) — 领域值与 Weyl 身份。
- [atlas-core-domain-seams.md](../../sources/atlas-core-domain-seams.md) — 领域层接缝：值提取器、alcove 助手与 Weyl 词／生成元校验。

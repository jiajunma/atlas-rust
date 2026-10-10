---
title: Weyl 元素兼容性与跨坐标词重放
summary: Weyl 元素比较先检查抽象群 Arc 身份，再将右元素的外生成元规范词在左属主坐标系重放，避免直接比较外来根置换。
sources:
  - atlas-core-domain-seams.md
  - atlas-core-domain-values.md
kind: concept
createdAt: "2026-10-09T14:30:45.385Z"
updatedAt: "2026-10-10T00:18:21.234Z"
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
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: Weyl 元素兼容性与跨坐标词重放
summary: Weyl 兼容性由抽象群 Arc 身份决定；跨坐标运算重放外生成元规范词，二元关系与乘积在无值门之前检查兼容。
sources:
  - atlas-core-domain-values.md
  - atlas-core-domain-seams.md
kind: concept
tags:
  - Weyl群
  - 兼容性
  - 规范词
aliases:
  - weyl-元素兼容性与跨坐标词重放
provenanceState: extracted
---

# Weyl 元素兼容性与跨坐标词重放

Weyl 元素的兼容性由抽象 Weyl 群的 `Arc` 身份决定，不能用根数据句柄的结构相等或局部坐标 kernel 相同替代。兼容元素的跨坐标等值与乘积通过重放外生成元词实现；外来根置换不得直接比较或复合。^[atlas-core-domain-values.md:53-56, atlas-core-domain-values.md:80-82, atlas-core-domain-values.md:123-124]

## 局部坐标与抽象群身份

`WeylEltContext` 包含 owner 局部坐标 kernel 和抽象群身份；后者携带内部生成元重编号，以固定 canonical-word 的选择。`DatumWeylKernel` 保存该根数据的枚举根系，由克隆、别名及 `root_datum(WeylElt)` 往返共享。`DatumWeylIdentity` 惰性保存坐标 kernel 与抽象群身份，两个单元都不回指 handle，因此不形成所有权环，参见 [[Rust Weyl 内核与抽象群的无环所有权模型]]。^[atlas-core-domain-values.md:24-29, atlas-core-domain-values.md:53-56]

`RootDatumHandle::interned` 是唯一构造入口，等值的活数据共享一个 Weyl 身份。不过，`RootDatumHandle::PartialEq` 刻意忽略身份缓存，只比较 `datum`、`lie_type`、`isogeny` 与 `prefers_coroots`。因此，根数据的结构等值与 Weyl 群的身份兼容属于不同判定。^[atlas-core-domain-values.md:41-43, atlas-core-domain-values.md:53-56]

## 兼容检查与失败顺序

`weyl_group_compatible` 使用 `Arc::ptr_eq(group)` 判断兼容性。`require_weyl_compatible` 在无值门之前检查二元关系与乘积，不兼容时报告运行时错误 `Weyl group mismatch`。即使调用处不需要结果值，这一检查仍须执行，参见 [[Weyl 元素的可失败关系与跨坐标运算]]。^[atlas-core-domain-values.md:89-91, atlas-core-domain-values.md:123-124, atlas-core-domain-seams.md:85-87]

## 构造时冻结与跨坐标重放

`WeylEltValue` 保存元素、构造上下文，以及构造时冻结的 canonical 既约 word。冻结点 `weyl_elt_value` 一次性计算规范约化词，后续 `Display` 与 `word` 访问都是纯读，详见 [[Weyl 元素值的规范词冻结]]。^[atlas-core-domain-values.md:57-58, atlas-core-domain-seams.md:93-95]

`weyl_replayed_in_left` 将右操作数的外生成元规范词放入左 owner 的坐标系，逐步调用 `right_multiply_simple` 重放。这个方向是明确的：右元素提供词，左上下文提供解释该词的坐标系统，外来根置换不直接参与比较或复合。^[atlas-core-domain-seams.md:88-90]

`weyl_elements_equal` 先检查兼容性，再把重放结果与左元素的词级置换比较；其消费方是 Weyl `=` 关系臂。由此，辫子等价词能够跨坐标判为相等。跨坐标乘积也遵循重放外生成元词的规则。^[atlas-core-domain-seams.md:91-92, atlas-core-domain-values.md:80-82, atlas-core-domain-values.md:123-124]

## 词输入的校验边界

词输入的合法性由 `check_weyl_word` 检查：每个条目先经 `as_integer`，负数报 `Negative integer where unsigned is required`；收窄至 `u64` 失败时报 `Integer value to big for conversion`；条目大于或等于半单秩时，报 `Illegal Weyl word entry {i} (should be <{r})`。其中 `to big` 是来源记录的原始措辞。^[atlas-core-domain-seams.md:96-102]

单生成元使用不同的转换顺序：`check_weyl_generator` 先收窄到 `i32`，再检查范围；负 `i32` 转为 `usize` 失败时，同样报告 `Generator {g} out of range for Weyl group (should be <{r})`。这类整数校验应与抽象群身份兼容检查区分，详见 [[Weyl 词与单生成元的整数校验差异]]。^[atlas-core-domain-seams.md:103-107, atlas-core-domain-values.md:89-95]

## 对偶预热与身份生命周期

对偶构造通过 `share_group_into_if_cold` 共享抽象群身份：只有目标 canonical dual 仍冷时，才将源的抽象群装入目标。已预热目标不会被覆盖，并与源保持不兼容；并发共享竞争中只有一个结果发布，另一个被丢弃。因此，预热历史影响后续兼容性，详见 [[dual 预热历史与 Weyl 群兼容性]]。^[atlas-core-domain-values.md:30-33]

全局 `DATUM_WEYL_IDENTITIES` 表按完整内容保存弱引用。等值 owner 仍存活时，新构造复用其身份单元；全部强引用消失后，槽位过期，后来等值构造重新开始。表大小超过 4096 时才扫描失效槽位，参见 [[RootDatum 弱驻留与规范活对象身份]]。^[atlas-core-domain-values.md:34-37]

身份单元 `WeylIdentityCell<T>` 使用 `OnceLock` 与初始化互斥锁，只在初始化成功时落定，失败不占用单元。双重检查与锁防止并发重复初始化；毒化或二次初始化报告 `StructureError::RepInvariantViolation`，详见 [[可失败重试的惰性身份初始化]]。^[atlas-core-domain-values.md:21-23]

## 证据范围

本页依据 `domain_builtins.rs` 相关区域的结构性阅读。领域值来源将 Weyl owner/dual 修复的语义验收限定为 A1，并指向 `tests/reference/hpc/math_weyl_context_core_after_v5_acceptance_2026_10_06.json`；v8 原始差异发现与 A1 回归金标位于 `tests/math/generics/weyl_context_core_*`。该来源未覆盖派发表与测试正文，不能据此扩大验收范围。^[atlas-core-domain-values.md:9-17, atlas-core-domain-values.md:121-126]

领域接缝来源补充了重放助手、冻结点与校验顺序，但不声称语言或数学验收。其中上游 C++／CWEB 行号转述自 Rust 源码注释，未独立重读上游，可能随版本漂移；校验助手的诊断措辞仍以 HPC 语料门为行为权威。^[atlas-core-domain-seams.md:9-19, atlas-core-domain-seams.md:115-118]

## Sources

- [atlas-core-domain-values.md](../../sources/atlas-core-domain-values.md)：领域值与 Weyl 身份。
- [atlas-core-domain-seams.md](../../sources/atlas-core-domain-seams.md)：领域层接缝——值提取器、alcove 助手与 Weyl 词／生成元校验。

---
title: 根数据的 Weyl 身份与弱驻留机制
summary: DatumWeylIdentity 分离惰性坐标 kernel 与抽象群身份，完整内容弱驻留使等值活 owner 共享身份，且不形成所有权环。
sources:
  - atlas-core-domain-values.md
kind: concept
createdAt: "2026-10-09T14:30:32.475Z"
updatedAt: "2026-10-09T20:34:03.325Z"
tags:
  - Rust设计
  - Weyl群
  - 对象身份
aliases:
  - 根数据的-weyl-身份与弱驻留机制
  - 根W身
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: 根数据的 Weyl 身份与弱驻留机制
summary: DatumWeylIdentity 分离属主局部坐标缓存与抽象 Weyl 群身份；完整内容弱驻留使存活的等值根数据共享身份，对偶构造仅向冷目标共享抽象群。
sources:
  - atlas-core-domain-values.md
kind: concept
tags:
  - Weyl群
  - 身份管理
  - 弱引用
aliases:
  - 根数据的-weyl-身份与弱驻留机制
  - 根W身
---

# 根数据的 Weyl 身份与弱驻留机制

`DatumWeylIdentity` 管理每个根数据属主的 Weyl 身份，将属主局部坐标缓存与抽象 Weyl 群身份分离。等值且仍存活的根数据共享身份单元；Weyl 元素兼容性则取决于抽象群的 `Arc` 指针同一性，不能由根数据结构相等或坐标缓存相同替代。^[atlas-core-domain-values.md:24-37, atlas-core-domain-values.md:53-56]

## 身份组成与惰性初始化

`DatumWeylKernel` 保存根数据的枚举根系，作为属主局部坐标缓存；克隆、别名以及 `root_datum(WeylElt)` 往返均共享它。`AbstractWeylGroup` 提供 canonical-word 接口，其共享范围对应上游共享 `WeylGroup` 指针的情形。`DatumWeylIdentity` 组合这两部分，两个单元都不回指 handle，因此不会形成所有权环。^[atlas-core-domain-values.md:24-29]

底层 `WeylIdentityCell<T>` 使用 `OnceLock` 与初始化互斥锁。`get_or_try_init` 仅在初始化成功时落定值，失败不占用单元；双重检查与锁防止并发重复初始化。锁毒化或二次初始化报告 `StructureError::RepInvariantViolation`，可结合 [[StructureError 统一错误分类学]] 理解其错误归类。^[atlas-core-domain-values.md:21-23]

## 按完整内容弱驻留

`DATUM_WEYL_IDENTITIES` 是按完整内容组织的全局弱引用表，对应上游 `root_datum_value` 的弱驻留机制。只要等值属主仍存活，新构造就复用其身份单元；所有强引用消失后，槽位过期，之后的等值构造重新建立身份。表规模超过 4096 时才清扫失效槽位。^[atlas-core-domain-values.md:34-37]

`RootDatumHandle::interned` 是唯一构造入口，保证等值的活数据共享同一个 Weyl 身份。其 `PartialEq` 刻意忽略 Weyl 身份缓存，只比较 `datum`、`lie_type`、`isogeny` 和 `prefers_coroots`；`Debug` 同样忽略该缓存。结构等值与 Weyl 身份由此保持分离。^[atlas-core-domain-values.md:39-43]

## 对偶构造与预热历史

对偶构造通过 `share_group_into_if_cold` 共享抽象群：只有目标 canonical dual 仍处于冷状态时，才把源的抽象群装入目标。已经预热的目标永不被覆盖，并与源保持不兼容；并发共享发生竞态时，只有一个候选被发布，另一个被丢弃。对偶目标的预热历史因此会影响后续 Weyl 兼容性，参见 [[dual 预热历史与 Weyl 群兼容性]]。^[atlas-core-domain-values.md:30-33]

## 元素兼容与跨坐标操作

`WeylEltContext` 同时携带属主局部坐标 kernel 和抽象群身份；抽象群身份还携带内部生成元重编号，用于固定上游 canonical-word 选择。`WeylEltValue` 在构造时计算并冻结 canonical 既约词，使 `Display` 和 `word` 成为纯读取操作。^[atlas-core-domain-values.md:53-58, atlas-core-domain-values.md:92-92]

`weyl_group_compatible` 使用 `Arc::ptr_eq(group)` 判断兼容性。二元关系和乘积通过 `require_weyl_compatible` 检查这一条件，且检查发生在无值门之前；不兼容时报告 `Weyl group mismatch`。即使求值不需要返回值，也必须验证兼容性。^[atlas-core-domain-values.md:87-91, atlas-core-domain-values.md:123-124]

Weyl 元素等值比较先检查抽象群 `Arc` 同一性，再将右元素的 canonical 外生成元词在左侧系统中重放后比较，从而识别跨坐标的辫子等价词。外来根置换永不直接比较或复合；跨坐标乘积同样遵循外生成元词重放规则。详见 [[Weyl 元素兼容性与跨坐标词重放]]。^[atlas-core-domain-values.md:80-82, atlas-core-domain-values.md:123-124]

## 证据范围

本页依据 `domain_builtins.rs` 相关区域的结构性阅读。来源将 Weyl owner/dual 修复的语义验收限定于 A1，并指向记录 `tests/reference/hpc/math_weyl_context_core_after_v5_acceptance_2026_10_06.json`；v8 的原始差异发现与 A1 回归金标位于 `tests/math/generics/weyl_context_core_*`。该来源包未覆盖派发表、测试及内类／实形管线实现，因此其验收描述不能推广为更广范围的证明。^[atlas-core-domain-values.md:9-17, atlas-core-domain-values.md:119-126]

## Sources

- [领域值与 Weyl 身份（domain_builtins.rs 上部）](atlas-core-domain-values.md)

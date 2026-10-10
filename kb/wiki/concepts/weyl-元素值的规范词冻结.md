---
title: Weyl 元素值的规范词冻结
summary: weyl_elt_value 在构造 WeylEltValue 时一次性计算并保存规范约化词，为后续跨坐标重放提供稳定表示。
sources:
  - atlas-core-domain-seams.md
kind: concept
createdAt: "2026-10-09T22:15:53.336Z"
updatedAt: "2026-10-10T00:18:12.339Z"
tags:
  - Weyl群
  - 值表示
aliases:
  - weyl-元素值的规范词冻结
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: Weyl 元素值的规范词冻结
summary: weyl_elt_value 在构造 WeylEltValue 时一次性计算规范约化词，与跨坐标重放及抽象群身份检查配合使用。
sources:
  - atlas-core-domain-seams.md
kind: concept
tags:
  - Weyl群
  - 值表示
  - 规范化
aliases:
  - weyl-元素值的规范词冻结
---

# Weyl 元素值的规范词冻结

`weyl_elt_value` 是 `WeylEltValue` 的值冻结点：构造值时一次性计算规范约化词。该助手位于 `domain_builtins.rs:9642`，属于 AFTER-v5 Weyl 身份纪律在派发侧的配套实现。^[atlas-core-domain-seams.md:81-95]

## 冻结内容与跨坐标重放

冻结的内容是 Weyl 元素的规范约化词。源码注释将其对应到上游 `WeylGroup::word`（`weyl.cpp:944–957`）；该上游位置来自注释转述。相关概念见 [[Weyl 元素的规范词]]。^[atlas-core-domain-seams.md:93-95]

跨坐标操作使用右操作数的**外生成元规范词**。`weyl_replayed_in_left` 在左操作数 owner 的坐标系中，逐步调用 `right_multiply_simple` 重放该词；外来根置换不直接参与比较或复合。详见 [[Weyl 元素兼容性与跨坐标词重放]]。^[atlas-core-domain-seams.md:88-90]

## 兼容性与等值比较

规范词重放配合抽象群身份检查使用。`require_weyl_compatible` 以抽象群的 `Arc` 身份判断兼容性，不兼容时报运行时错误 `Weyl group mismatch`。^[atlas-core-domain-seams.md:85-87]

`weyl_elements_equal` 先检查兼容性，再将右元素在左坐标系中的重放结果与左元素的词级置换比较。该助手由 Weyl `=` 关系臂消费，因此等值判断包含兼容性检查、坐标重放和置换比较三个环节。^[atlas-core-domain-seams.md:88-92]

## 证据边界

本页依据当前工作区字节的结构性阅读，不声称语言或数学验收。来源中的上游 C++／CWEB 行号均转述自 Rust 源码注释，未独立重读上游，可能随版本漂移。^[atlas-core-domain-seams.md:9-19, atlas-core-domain-seams.md:115-118]

## Sources

- [atlas-core-domain-seams.md](../../sources/atlas-core-domain-seams.md) — 领域层接缝：值提取器、alcove 助手与 Weyl 词／生成元校验。

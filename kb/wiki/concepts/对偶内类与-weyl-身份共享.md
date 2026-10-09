---
title: 对偶内类与 Weyl 身份共享
summary: 对偶内类构造结合余根偏好翻转、逐字母对偶的 Lie 类型及对偶 datum 句柄，使内容相等的对偶 datum 经内容弱 interning 共享 Weyl 身份。
sources:
  - atlas-core-domain-construction.md
kind: concept
createdAt: "2026-10-09T14:28:30.612Z"
updatedAt: "2026-10-09T14:28:30.612Z"
tags:
  - 对偶内类
  - Weyl身份
  - 弱驻留
aliases:
  - 对偶内类与-weyl-身份共享
  - 对W身
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 对偶内类与 Weyl 身份共享

对偶内类的构造不仅涉及对偶根数据与 Lie 类型的转换，也涉及 Weyl 身份的共享：等值内容的对偶 datum 经内容弱驻留（weak interning）共享 Weyl 身份。这是 `build_dual_inner_class` 所采用的 dual-identity 路径。^[atlas-core-domain-construction.md:50-54]

## 对偶构造路径

`build_dual_inner_class`（`domain_builtins.rs:2276`）通过 `dual_inner_class` 构造对偶内类，并配合对偶 datum 的 handle。对偶 datum 的处理包括**余根偏好翻转**以及逐字母对偶的 Lie 类型；来源将前者关联到上游 `RootSystem DualTag`（`rootdata.cpp:341`）。相关概念见 [[对偶根数据与对偶内类构造]]。^[atlas-core-domain-construction.md:50-54]

身份共享的依据是对偶 datum 的内容等值，并通过内容弱驻留实现。理解这一路径时，可结合 [[RootDatum 弱驻留与规范活对象身份]] 与 [[WeylAction 的等值与 datum 身份语义]]，区分根数据的内容与其关联的 Weyl 身份。^[atlas-core-domain-construction.md:50-54]

## 内类装配中的对偶侧

`build_inner_class_context` 按固定顺序完成装配：先进行带分类预算的 `classification_cached`，再构造受 `FIBER_BUDGET` 约束的强实分类、外部实形顺序，以及受 `INTEGER_BUDGET` 约束的内类布局；随后**只构建一次对偶侧**。对偶侧包括 `dual_inner_class`、对偶分类、对偶弱实形计数 `dual_form_count` 和 `dual_cartan_correspondence`。之后才构造展示信息与规范实形弱缓存槽位表。详见 [[内类上下文的装配顺序与预算门]]。^[atlas-core-domain-construction.md:35-43]

这里有两个不同层面的机制：装配流程规定对偶侧只建一次；dual-identity 路径则规定等值内容的对偶 datum 通过内容弱驻留共享 Weyl 身份。前者约束构造顺序，后者描述身份共享的依据。^[atlas-core-domain-construction.md:37-43, atlas-core-domain-construction.md:50-54]

## 证据边界

本说明依据构造管线的结构性阅读，不代表数学验收。来源中的上游行号属于实现方的移植陈述，构造兼容性仍以 HPC 差分门为准；快照字节或哈希本身不能替代兼容性验证。相关验收背景见 [[HPC 验收证据链]]。^[atlas-core-domain-construction.md:9-14, atlas-core-domain-construction.md:73-75]

## Sources

- [atlas-core-domain-construction.md](../../sources/atlas-core-domain-construction.md)

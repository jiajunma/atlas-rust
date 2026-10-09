---
title: WeylAction 的等值与 datum 身份语义
summary: 派生等值逐字段比较并包含 datum 的内容值，因此矩阵相同但 datum 值不同的作用仍不相等。
sources:
  - weyl-layer.md
kind: concept
createdAt: "2026-10-09T15:17:27.163Z"
updatedAt: "2026-10-09T19:38:05.868Z"
tags:
  - Weyl群
  - 等值语义
aliases:
  - weylaction-的等值与-datum-身份语义
  - W的D身
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# WeylAction 的等值与 datum 身份语义

`WeylAction` 是 Weyl 群的矩阵级作用类型，同时保存权格（character lattice）与余权格（cocharacter lattice）上的作用及其根数据来源。它包含 `datum: Arc<BasedRootDatum>`、`weight_matrix` 和 `coweight_matrix` 三个字段；其等值判定同时涉及 datum 值与两份矩阵。^[weyl-layer.md:19-21, weyl-layer.md:30-30, weyl-layer.md:45-47]

## 等值判定

实现采用派生的逐字段比较，**包含 datum 的值比较**。`Arc` 的 `PartialEq` 将比较委派给内部值，因此 datum 比较不要求两个作用持有同一个 `Arc` 分配对象。即使矩阵相同，只要 datum 值不同，两个 `WeylAction` 仍不相等。^[weyl-layer.md:45-47]

因此，“相等即矩阵作用相等”的概述需要限定在 datum 值相同的情形：在此前提下，两份作用矩阵相等即可得到相等的作用值；跨 datum 比较时，仅有矩阵相同还不够。同一 datum 下，等价生成元词给出相同作用值，无需枚举 Weyl 群。^[weyl-layer.md:19-21, weyl-layer.md:45-47]

## 来源访问与兼容性

`WeylAction` 提供 `datum` 和 `datum_arc` 访问器，其中 `datum_arc` 增加 `Arc` 引用计数。保存 datum 使矩阵作用携带根数据来源；相关类型背景见 [[BasedRootDatum：带基根数据与构造不变量]]，双格作用见 [[WeylAction 的对偶全格作用]]。^[weyl-layer.md:19-21, weyl-layer.md:30-39]

来源列出的测试锚点包括“同秩异 datum”以 `DatumMismatch` 拒绝，表明相同的秩不足以保证 datum 兼容。这里应区分等值比较与兼容性检查：前者由逐字段比较决定，后者有拒绝不匹配 datum 的测试锚点。^[weyl-layer.md:45-47, weyl-layer.md:59-62]

## 与词级元素的区别

`WeylElement` 使用 ambient `RootSystem` 的枚举根置换表示元素，其操作能表达的来源检查仅为根数匹配；始终使用同一 ambient system 是调用方契约，由 KGB stages 负责。因此，不能将 `WeylAction` 携带 datum 的等值语义直接套用于词级元素。两层通过 `RootSystem::action_permutation` 与 `WeylElement::from_action` 衔接，详见 [[Weyl 群的矩阵作用与词级元素双层结构]]。^[weyl-layer.md:22-26, weyl-layer.md:66-70]

## 证据范围

上述结论来自来源包对实现的结构性阅读。包内列出了 A2 编织关系值相等、同秩异 datum 拒绝等测试锚点，但未执行构建、测试或原版运行，因此不构成新增的数学验收、性能或并行结论。^[weyl-layer.md:9-12, weyl-layer.md:59-62, weyl-layer.md:121-124]

## Sources

- [weyl-layer.md](weyl-layer.md) — Weyl 群层：矩阵作用与词级元素的双层结构

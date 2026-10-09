---
title: WeylAction 的等值与 datum 身份语义
summary: 源码 derive 等值逐字段比较且包含 datum 值，因此“相等即矩阵作用相等”的概述须限定于 datum 值相同的情形。
sources:
  - weyl-layer.md
kind: concept
createdAt: "2026-10-09T15:17:27.163Z"
updatedAt: "2026-10-09T15:17:27.163Z"
tags:
  - Rust设计
  - 等值语义
  - 根数据
aliases:
  - weylaction-的等值与-datum-身份语义
  - W的D身
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# WeylAction 的等值与 datum 身份语义

`WeylAction` 是 Weyl 群的矩阵级作用类型，同时保存 character 与 cocharacter 两个全格上的作用，以及根数据来源。其字段为 `datum: Arc<BasedRootDatum>`、`weight_matrix` 和 `coweight_matrix`；这一结构使作用值同时携带矩阵与 datum 信息。^[weyl-layer.md:19-21, weyl-layer.md:28-30]

## 等值判定

实现中的等值由派生的逐字段比较决定，**包含 datum 的值比较**。`Arc` 的 `PartialEq` 将比较委派给内部值，因此这里的 datum 等值并非要求两个作用持有同一个 `Arc` 分配对象。即使矩阵相同，只要 datum 值不同，两个 `WeylAction` 仍不相等。^[weyl-layer.md:45-47]

这限定了“相等即矩阵作用相等”的表述：在 datum 值相同的前提下，矩阵作用相等与实现的等值语义一致；跨 datum 比较时，不能仅凭矩阵相同认定作用相等。等价生成子词可得到相同的作用值，无需枚举 Weyl 群，但理解这一性质时必须保留 datum 条件。^[weyl-layer.md:19-21, weyl-layer.md:45-47]

## datum 来源与兼容性

`WeylAction` 提供 `datum` 与 `datum_arc` 访问器，其中 `datum_arc` 增加 `Arc` 引用计数。来源包列出的测试锚点包括“同秩异 datum”以 `DatumMismatch` 拒绝，说明相同的秩不足以保证 datum 兼容。相关根数据构造背景见 [[BasedRootDatum：带基根数据与构造不变量]]。^[weyl-layer.md:37-39, weyl-layer.md:59-62]

矩阵层与词级组合层的来源约束不同：`WeylElement` 用 ambient `RootSystem` 的枚举根置换表示元素，其操作可表达的 provenance 检查仅为根数匹配，使用同一 ambient system 的纪律由调用方承担。因而不能把矩阵层携带 datum 的语义直接套用到词级元素；两层分工见 [[Weyl 群的矩阵作用与词级元素双层结构]]。^[weyl-layer.md:19-26, weyl-layer.md:64-68]

## 证据范围

上述等值语义来自来源包对实现的结构性阅读。包内虽列出 A2 编织关系值相等、同秩异 datum 拒绝等测试锚点，但未执行构建、测试或原版运行；这些记录不构成新增的数学验收、性能或并行结论。^[weyl-layer.md:9-12, weyl-layer.md:59-62, weyl-layer.md:121-124]

## Sources

- [weyl-layer.md](weyl-layer.md) — Weyl 群层：矩阵作用与词级元素的双层结构

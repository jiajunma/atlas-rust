---
title: WeylAction 的等值与 datum 身份语义
summary: 派生等值逐字段比较两种作用矩阵及 datum 内容，因此矩阵相同但 datum 值不同的作用仍不相等。
sources:
  - weyl-layer.md
kind: concept
createdAt: "2026-10-09T15:17:27.163Z"
updatedAt: "2026-10-09T21:14:06.271Z"
tags:
  - Weyl群
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
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: WeylAction 的等值与 datum 身份语义
summary: WeylAction 的派生等值比较包含 datum 内容值及两份作用矩阵，不要求 Arc 分配身份相同；矩阵相同但 datum 值不同的作用仍不相等。
sources:
  - weyl-layer.md
kind: concept
tags:
  - Weyl群
  - 等值语义
---

# WeylAction 的等值与 datum 身份语义

`WeylAction` 是 Weyl 群的矩阵级作用类型，同时保存权格（character lattice）与余权格（cocharacter lattice）上的作用，并携带根数据来源。其字段为 `datum: Arc<BasedRootDatum>`、`weight_matrix` 和 `coweight_matrix`；等值判定涉及这三个字段。^[weyl-layer.md:19-21, weyl-layer.md:30-30, weyl-layer.md:45-47]

## 等值判定

实现采用派生的逐字段比较，**datum 按内容值比较**。`Arc` 的 `PartialEq` 将比较委派给内部值，因此两个作用不必持有同一个 `Arc` 分配对象。即使两份作用矩阵分别相同，只要 datum 值不同，两个 `WeylAction` 仍不相等。^[weyl-layer.md:45-47]

来源中“相等即矩阵作用相等”的概述需要限定在 datum 值相同的情形：此前提下，两份矩阵分别相等即可得到相等的作用值。同一 datum 下，等价生成元词给出相同作用值，无需枚举 Weyl 群。^[weyl-layer.md:19-21, weyl-layer.md:45-47]

## 来源访问与兼容性

`datum` 和 `datum_arc` 提供根数据访问，其中 `datum_arc` 会增加 `Arc` 引用计数。这使作用值保留其根数据来源；相关背景见 [[BasedRootDatum：带基根数据与构造不变量]] 与 [[WeylAction 的对偶全格作用]]。^[weyl-layer.md:19-21, weyl-layer.md:30-39]

来源列出的测试锚点包括“同秩异 datum”以 `DatumMismatch` 拒绝，说明相同的秩不足以保证 datum 兼容。等值比较与兼容性检查应分别理解：前者是包含 datum 值的逐字段比较，后者有拒绝不匹配 datum 的测试锚点。^[weyl-layer.md:45-47, weyl-layer.md:59-62]

## 与词级元素的区别

`WeylElement` 通过 ambient `RootSystem` 的枚举根置换表示元素，其每次操作能表达的来源检查仅为根数匹配。始终使用同一 ambient system 是调用方契约，由 KGB stages 负责；这与 `WeylAction` 显式携带 datum 的结构不同。^[weyl-layer.md:22-24, weyl-layer.md:66-68]

两层通过 `RootSystem::action_permutation` 与 `WeylElement::from_action` 衔接。整体分工见 [[Weyl 群的矩阵作用与词级元素双层结构]]，词级不变量见 [[WeylElement 的置换表示与长度下降不变量]]。^[weyl-layer.md:22-26, weyl-layer.md:64-74]

## 证据范围

上述说明来自来源包的结构性源码阅读。包内列出了 A2 编织关系值相等、同秩异 datum 拒绝等测试锚点，但没有执行构建、测试或原版运行，因此不构成新增的数学验收、性能或并行结论。Weyl 层的正确性仍属于其独立的 HPC 证据链。^[weyl-layer.md:9-12, weyl-layer.md:59-62, weyl-layer.md:121-124]

## Sources

- [weyl-layer.md](../../sources/weyl-layer.md) — Weyl 群层：矩阵作用与词级元素的双层结构

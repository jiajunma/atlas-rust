---
title: Torus 部分的模二矩阵传输
summary: 该实现使用记录中 WeylAction 的模二矩阵传输，替代上游 push_across 与 pull_across 的 word walks；源文档未据此给出性能结论。
sources:
  - tits-element.md
kind: concept
createdAt: "2026-10-09T15:13:10.435Z"
updatedAt: "2026-10-09T15:13:10.435Z"
tags:
  - Weyl作用
  - 模二运算
  - 算法设计
aliases:
  - torus-部分的模二矩阵传输
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# Torus 部分的模二矩阵传输

Torus 部分的模二矩阵传输是 KGB stage c 中处理 Tits 群操作的一项实现策略：利用 involution 记录中存储的 `WeylAction`，以 mod-2 矩阵传输替代上游 `push_across`／`pull_across` 的逐词行走（word walks）。^[tits-element.md:19-26]

## 元素表示与传输依据

`TitsElement` 采用 `(involution, torus bits)` 二元组表示。该形状对应上游持久化的逐元素表示，本实现也在构造期使用它；由于 stage (b) 的 `InvolutionTable` 已提供 O(1) 的 cross 链接与 Cayley 边，元素自身不携带逐元素 Weyl 数据。矩阵传输使用记录中的 `WeylAction`，相关结构见 [[TitsElement 的元素表示与正规形契约]]与 [[Twisted involution 表与 Cartan 轨道存储]]。^[tits-element.md:19-26]

## 上下文一致性

`TitsCoset::new` 从 inner class 一次性建表，其来源校验要求完整的 inner class 相等，不能仅检查 datum 相等。同一 datum 上具有不同 distinguished involution 的 inner class，其 twist 与 transport 也不同；只检查 datum 会破坏操作语义。twist 置换和 simple-reflection 根置换由 coset 保存为自身的派生副本，参见 [[TitsCoset 的 grading offset 与完整 inner-class 门控]]。^[tits-element.md:38-43]

## 传输与正规形

Torus bits 的表示需要遵守归约契约：`TitsElement::new` 检查 involution 编号和 torus 维数，但不会自动归约。派生序关系按 involution 分组并比较原始 bits，只有在归约代表元上才具有语义；`reduce` 产生表内正规形，且具有幂等性。^[tits-element.md:32-34, tits-element.md:63-63]

具体操作分别规定归约位置：`cross` 在目标处归约；`cayley` 执行裸 `sigma_mult` 后，在目标处扩大的 mod-space 中归约；`inverse_cayley` 则先执行裸 `sigma_inv_mult`，必要时使用源 mod-space 基向量修复重建出的 compact grading，最后在 imaginary target 处归约并复核 simple grading。相关操作见 [[Based cross action 的闭式实现]]与 [[Cayley 变换与目标模空间归约]]。^[tits-element.md:50-60]

## 证据边界

本页依据对 `tits_element.rs` 的结构性阅读，所读字节来自 dirty 工作区并记录于来源快照。来源中的上游 `tits.cpp` 行号转述自 Rust 源码注释，未独立重读上游；该材料未执行构建、测试或原版运行，因此不提供矩阵传输的性能结论，也不扩展其所属的 HPC 正确性证据链。^[tits-element.md:9-15, tits-element.md:67-72]

## Sources

- [tits-element.md](tits-element.md) — Tits 元素：torus 部分与 Tits 群操作（KGB stage c）。

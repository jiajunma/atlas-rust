---
title: Torus 部分的模二矩阵传输
summary: 环面部分的运输使用对合记录中 WeylAction 的模二矩阵，替代上游 push_across 与 pull_across 的词遍历；来源未提供性能验证。
sources:
  - tits-element.md
kind: concept
createdAt: "2026-10-09T15:13:10.435Z"
updatedAt: "2026-10-09T19:37:26.914Z"
tags:
  - 环面
  - Weyl作用
  - 模二线性代数
aliases:
  - torus-部分的模二矩阵传输
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# Torus 部分的模二矩阵传输

Torus 部分的模二矩阵传输是 KGB stage c 中实现 Tits 群操作的一项策略：利用 involution 记录中存储的 `WeylAction`，以模二矩阵传输替代上游 `push_across`／`pull_across` 的逐词行走（word walks）。来源将这一策略对应到上游注释中的 sophistication，但未据此给出性能结论。^[tits-element.md:19-26, tits-element.md:72-72]

## 元素表示与传输依据

`TitsElement` 采用 `(involution, torus bits)` 二元组表示，对应上游持久化的逐元素形状。本实现也在构造期采用这一表示，因为 stage (b) 的 `InvolutionTable` 已提供 O(1) 的 cross 链接与 Cayley 边，因此元素自身不携带逐元素 Weyl 数据。矩阵传输所需的 `WeylAction` 来自记录，相关结构见 [[TitsElement 的元素表示与正规形契约]]与 [[Twisted involution 表与 Cartan 轨道存储]]。^[tits-element.md:19-26]

## 上下文一致性

`TitsCoset::new` 从 inner class 一次性建表，其来源校验要求完整的 inner class 相等。同一 datum 上具有不同 distinguished involution 的 inner class，会具有不同的 twist 与 transport，因此仅检查 datum 相等不足以保证操作语义。twist 置换与 simple-reflection 根置换由 coset 保存为自身的派生副本，参见 [[TitsCoset 的 grading offset 与完整 inner-class 门控]]。^[tits-element.md:38-43]

## 传输与正规形契约

`TitsElement::new` 检查 involution 编号和 torus 维数，但不会自动归约。其派生序关系按 involution 分组并比较原始 bits，只有在归约代表元上才具有语义；`reduce` 产生 torus bits 的表内正规形，且具有幂等性。^[tits-element.md:32-34, tits-element.md:63-63]

各项操作分别规定目标处的归约方式。`cross` 实现目标处已归约的 based cross action；`cayley` 执行裸 `sigma_mult`，再在目标处扩大的 mod-space 中归约，若目标 Cartan 类尚未加入表则返回 `None`。相关说明见 [[Based cross action 的闭式实现]]与 [[Cayley 变换与目标模空间归约]]。^[tits-element.md:50-53]

`inverse_cayley` 先执行裸 `sigma_inv_mult`。由于 real source 处的模约化可能遗忘源侧 grading，若重建出的根为 compact，就用第一个与该根配对非平凡的源 mod-space 基向量修复；不存在这样的基向量时报告 `TitsCosetInvariantViolation`。随后在 imaginary target 处归约并复核 simple grading，详见 [[逆 Cayley 变换的 grading 修复]]。^[tits-element.md:54-60]

## 证据边界

来源属于对 `tits_element.rs` 的结构性阅读，所读字节来自 dirty 工作区并记录于快照。上游 `tits.cpp` 行号转述自源码注释，未独立重读上游；该材料未执行构建、测试或原版运行，不包含数学验收、性能或并行结论，也不扩展此实现所属的 [[HPC 验收证据链]]。^[tits-element.md:9-15, tits-element.md:67-72]

## Sources

- [tits-element.md](tits-element.md) — Tits 元素：torus 部分与 Tits 群操作（KGB stage c）。

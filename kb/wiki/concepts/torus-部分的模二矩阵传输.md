---
title: Torus 部分的模二矩阵传输
summary: 环面部分使用对合记录中 WeylAction 的模二矩阵进行传输，替代上游 push_across 与 pull_across 的词遍历；来源未提供性能验证。
sources:
  - tits-element.md
kind: concept
createdAt: "2026-10-09T15:13:10.435Z"
updatedAt: "2026-10-09T21:11:21.035Z"
tags:
  - 环面
  - 模二线性代数
  - Weyl作用
aliases:
  - torus-部分的模二矩阵传输
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: Torus 部分的模二矩阵传输
summary: 使用对合记录中的 WeylAction 进行环面部分的模二矩阵传输，替代上游 push_across 与 pull_across 的词遍历；操作依赖完整 inner class 一致性及正规形契约。
sources:
  - tits-element.md
kind: concept
tags:
  - 环面
  - Weyl作用
  - 模二线性代数
aliases:
  - torus-部分的模二矩阵传输
---

# Torus 部分的模二矩阵传输

Torus 部分的模二矩阵传输是 KGB stage c 实现 Tits 群操作的一项策略：利用对合记录中存储的 `WeylAction`，以模二矩阵传输替代上游 `push_across`／`pull_across` 的词遍历（word walks）。来源将这一策略对应到上游注释所称的 sophistication，但未提供性能验证。^[tits-element.md:19-26, tits-element.md:72-72]

## 元素表示与传输依据

`TitsElement` 使用 `(involution, torus bits)` 二元组，沿用上游持久化的逐元素形状。本实现在构造期也采用该表示，因为 stage (b) 的 `InvolutionTable` 提供 O(1) 的 cross 链接与 Cayley 边，因此无需在每个元素中携带 Weyl 数据；矩阵传输使用记录中的 `WeylAction`。相关结构见 [[TitsElement 的元素表示与正规形契约]] 与 [[Twisted involution 表与 Cartan 轨道存储]]。^[tits-element.md:19-26]

## 上下文一致性

`TitsCoset::new` 从 inner class 一次性建表，来源校验要求**完整 inner class 相等**，而非仅 datum 相等。同一 datum 上具有不同 distinguished involution 的 inner class，其 twist 与 transport 不同，仅检查 datum 会破坏操作语义。twist 置换与 simple-reflection 根置换由 coset 持有为派生副本，详见 [[TitsCoset 的 grading offset 与完整 inner-class 门控]]。^[tits-element.md:38-43]

## 传输与正规形契约

`TitsElement::new` 检查 involution 编号与 torus 维数，但不自动归约。其派生序关系按 involution 分组并比较原始 bits，只有对归约代表元才具有语义。`reduce` 产生元素 torus bits 的表内正规形，且具有幂等性。^[tits-element.md:32-34, tits-element.md:63-63]

`cross` 返回在目标处已归约的 based cross action；其闭式实现等价于先执行 `sigma_mult(s, .)`，再执行 `mult_sigma_inv(., twist(s))`，并加上 offset 修正。`cayley` 执行裸 `sigma_mult`，随后在目标处扩大的 mod-space 中归约；目标 Cartan 类尚未加入表时返回 `None`。参见 [[Based cross action 的闭式实现]] 与 [[Cayley 变换与目标模空间归约]]。^[tits-element.md:50-53]

`inverse_cayley` 先执行裸 `sigma_inv_mult`：反射左 torus 部分，并在左乘增加 Weyl 长度时加上 \(m_\alpha\)。由于 real source 处的模约化可能遗忘源侧 grading，若重建出的根为 compact，就用第一个与该根配对非平凡的 source mod-space 基向量修复；不存在这样的基向量时报告 `TitsCosetInvariantViolation`。随后在 imaginary target 处执行 `quotient_representative` 归约，并复核 simple grading。根不是 real，或向下的 Cartan 类尚未加入表时返回 `None`，详见 [[逆 Cayley 变换的 grading 修复]]。^[tits-element.md:54-60]

## 证据边界

来源属于对 `tits_element.rs` 的结构性阅读，所读字节来自 dirty 工作区并记录于快照。上游 `tits.cpp` 行号转述自源码注释，未独立重读上游，可能随版本演进而漂移。该材料未执行构建、测试或原版运行，不包含数学验收、性能或并行结论，也不扩展实现自身的 HPC 证据链。^[tits-element.md:9-15, tits-element.md:67-72]

## Sources

- [tits-element.md](../../sources/tits-element.md) — Tits 元素：torus 部分与 Tits 群操作（KGB stage c）。

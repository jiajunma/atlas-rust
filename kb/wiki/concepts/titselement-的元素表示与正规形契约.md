---
title: TitsElement 的元素表示与正规形契约
summary: TitsElement 保存对合编号与环面位向量；构造器检查编号和维数但不自动归约，reduce 幂等地产生正规形，原始位向量排序仅对已归约代表元具有语义。
sources:
  - tits-element.md
kind: concept
createdAt: "2026-10-09T15:13:06.117Z"
updatedAt: "2026-10-09T21:11:16.471Z"
tags:
  - Tits群
  - 数据表示
  - 正规形
aliases:
  - titselement-的元素表示与正规形契约
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: TitsElement 的元素表示与正规形契约
summary: TitsElement 保存对合编号与环面位向量；构造器检查编号和维数但不自动归约，reduce 幂等地产生正规形，派生序关系仅对已归约代表元具有语义。
sources:
  - tits-element.md
kind: concept
tags:
  - Tits元素
  - 正规形
  - KGB
---

# TitsElement 的元素表示与正规形契约

`TitsElement` 是 KGB stage c 中用于环面（torus）部分与 Tits 群操作的元素类型，表示为二元组 `(involution, torus bits)`。其核心契约是：构造器检查编号与维数，但不自动归约；`reduce` 产生正规形，派生序关系仅在已归约的代表元上具有语义。^[tits-element.md:19-22, tits-element.md:32-34]

## 元素表示

这一二元组沿用上游持久化时的逐元素形状，本实现也在构造期采用它。由于 stage (b) 的 `InvolutionTable` 已提供 O(1) 的 cross 链接与 Cayley 边，元素本身不携带逐元素 Weyl 数据。相关表结构见 [[Twisted involution 表与 Cartan 轨道存储]]。^[tits-element.md:19-22]

## 构造与正规形

`TitsElement::new` 是受门控的裸构造器，检查 involution 编号与 torus 维数，但**不自动执行归约**。因此，构造成功并不保证 torus bits 已是正规代表元。^[tits-element.md:32-34]

`reduce` 将元素的 torus bits 化为表内正规形，且操作幂等。派生序关系先按 involution 分组，再比较原始位串（RAW bits）；这种比较仅在已归约的代表元上具有语义。使用该序关系时须满足正规形前提，不能将裸构造器的检查视为归约保证。^[tits-element.md:32-34, tits-element.md:63-63]

## 群操作中的归约位置

`cross` 实现 [[Based cross action 的闭式实现|based cross action]]，并在目标处归约结果。其闭式映射等于先执行 `sigma_mult(s, .)`，再执行 `mult_sigma_inv(., twist(s))`，外加 offset 修正。^[tits-element.md:50-51]

`cayley` 执行裸的 `sigma_mult`，随后在目标处扩大的 mod-space 中归约；若目标 Cartan 类尚未加入表，则返回 `None`。详见 [[Cayley 变换与目标模空间归约]]。^[tits-element.md:52-53]

`inverse_cayley` 先执行裸的 `sigma_inv_mult`：反射左 torus 部分，并在左乘增加 Weyl 长度时加上 \(m_\alpha\)。由于 real 源处的模约化可能已遗失源侧 grading，若重建出的根为 compact，则使用第一个与该根配对非平凡的源 mod-space 基向量进行修复；找不到这样的基向量时，报告 `TitsCosetInvariantViolation`。最后在 imaginary 目标处通过 `quotient_representative` 归约，并复核目标的 simple grading。根不是 real，或向下的 Cartan 类未加入表时，返回 `None`。相关机制见 [[逆 Cayley 变换的 grading 修复]]。^[tits-element.md:54-60]

## 上下文与查询前提

`TitsCoset::new` 从 inner class 一次性建表，grading offset 由调用方选定。其来源校验要求**完整 inner class 相等**，仅 datum 相等不足以保证兼容：同一 datum 上，不同 distinguished involution 对应的 inner class 具有不同的 twist 与 transport。详见 [[TitsCoset 的 grading offset 与完整 inner-class 门控]]。^[tits-element.md:38-43]

`simple_grading(s, element)` 按 `offset[s] XOR <alpha_s mod 2, torus bits>` 计算，`true` 表示 noncompact。它仅在简单根 `s` 于该元素的 involution 处为 IMAGINARY 时有意义，调用方须使用 `InvolutionTable::simple_root_kind` 守卫。任意根的 `grading` 查询采用上游的 conjugate-to-simple 循环，在根非 imaginary 时返回 `None`。^[tits-element.md:47-49, tits-element.md:61-62]

## 证据范围

本页依据对 `tits_element.rs` 的结构性阅读材料，其阅读快照记录的是 dirty 工作区字节。材料未执行构建、测试或原版运行，不提供数学验收、性能或并行结论；操作正确性属于实现自身的 HPC 证据链，来源材料不重述或扩展这些结论。^[tits-element.md:9-15, tits-element.md:67-72]

来源中的上游 `tits.cpp` 行号转述自源码注释，未独立重读上游，可能随版本演进而漂移。^[tits-element.md:69-69]

## Sources

- [tits-element.md](../../sources/tits-element.md) — Tits 元素：torus 部分与 Tits 群操作（KGB stage c）

---
title: Cayley 变换与目标模空间归约
summary: cayley 执行裸 sigma_mult 后在目标增大的 mod-space 中归约，目标 Cartan 类尚未加入表时返回 None。
sources:
  - tits-element.md
kind: concept
createdAt: "2026-10-09T15:13:36.894Z"
updatedAt: "2026-10-09T19:37:13.912Z"
tags:
  - Cayley变换
  - 商空间
aliases:
  - cayley-变换与目标模空间归约
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# Cayley 变换与目标模空间归约

`TitsCoset` 中的 `cayley` 先执行裸的 `sigma_mult`，再在目标处扩大的模空间（mod-space）中归约。目标模空间的归约是该操作语义的一部分；若目标 Cartan 类尚未加入表，则返回 `None`。^[tits-element.md:52-53]

## 元素表示与正规形

`TitsElement` 以 `(involution, torus bits)` 二元组表示元素。构造期也采用这一表示，因为 `InvolutionTable` 已提供 O(1) 的 cross 链接与 Cayley 边，无须为每个元素携带 Weyl 数据。参见 [[TitsElement 的元素表示与正规形契约]]、[[Twisted involution 表与 Cartan 轨道存储]]。^[tits-element.md:19-22]

`TitsElement::new` 检查 involution 编号与 torus 维数，但不自动归约。派生序关系按 involution 分组并比较原始 bits，仅在已归约代表元上具有语义。`reduce` 负责产生 torus bits 的表内正规形，且是幂等操作。^[tits-element.md:32-34, tits-element.md:63-63]

## 正向 Cayley 与 cross 的区别

正向 `cayley` 的顺序是“裸 `sigma_mult`，随后在目标模空间中归约”。`cross` 也在目标处归约，但其 [[Based cross action 的闭式实现|based cross action 闭式映射]] 等价于先执行 `sigma_mult(s, .)`，再执行 `mult_sigma_inv(., twist(s))`，并加上 offset 修正。两者的裸操作不同。^[tits-element.md:50-53]

## 逆 Cayley：源处修复，目标处归约

`inverse_cayley` 先执行裸的 `sigma_inv_mult`：反射左 torus 部分，并在左乘增加 Weyl 长度时加上 $m_\alpha$。由于实根源处的模约化可能已经遗忘源侧 grading，重建出的根可能为 compact；此时使用第一个与该根配对非平凡的源模空间基向量修复 grading。若不存在这样的基向量，则报 `TitsCosetInvariantViolation`。^[tits-element.md:54-58]

修复后，操作在虚根目标处调用 `quotient_representative` 归约，并复核目标的 simple grading。若源根不是实根，或向下的 Cartan 类尚未加入表，则返回 `None`。因此，源模空间参与 grading 修复，目标模空间负责最终归约，详见 [[逆 Cayley 变换的 grading 修复]]。^[tits-element.md:54-60]

`simple_grading(s, element)` 的计算式为 `offset[s] XOR <alpha_s mod 2, torus bits>`，其中 `true` 表示 noncompact。该值仅在简单根对当前元素的 involution 为 IMAGINARY 时有意义，调用方须使用 `InvolutionTable::simple_root_kind` 守卫。^[tits-element.md:47-49]

## 上下文约束与证据边界

`TitsCoset::new` 从 inner class 一次性建表，来源门控要求完整 inner class 相等。同一 datum 上不同 distinguished involution 对应不同的 twist 与 transport，因此仅检查 datum 相等不足以保证操作所需的上下文一致性。参见 [[TitsCoset 的 grading offset 与完整 inner-class 门控]]。^[tits-element.md:38-43]

来源文档属于结构性源码阅读；其中上游 `tits.cpp` 行号转述自源码注释，未独立重读上游。该文档未执行构建、测试或原版运行，不提供数学验收、性能或并行结论。^[tits-element.md:9-11, tits-element.md:65-72]

## Sources

- [tits-element.md](tits-element.md) — Tits 元素：torus 部分与 Tits 群操作（KGB stage c）

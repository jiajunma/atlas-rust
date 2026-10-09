---
title: 虚根的 noncompact grading
summary: 简单虚根的非紧性由 offset 与根对环面位向量的模二配对异或计算，任意虚根通过共轭到单根的循环求值，非虚根返回 None。
sources:
  - tits-element.md
kind: concept
createdAt: "2026-10-09T15:13:14.160Z"
updatedAt: "2026-10-09T19:37:24.556Z"
tags:
  - 虚根
  - 分级
aliases:
  - 虚根的-noncompact-grading
  - 虚NG
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# 虚根的 noncompact grading

虚根的 noncompact grading 用于判定一个根在给定 `TitsElement` 处的非紧性。`TitsCoset` 提供简单虚根接口 `simple_grading` 和任意虚根接口 `grading`；判定以根在该元素的 involution 处属于 IMAGINARY 为前提。^[tits-element.md:47-49, tits-element.md:61-62]

## 简单虚根的计算

对简单根 $\alpha_s$ 和元素 $e$，`simple_grading` 使用 grading offset 与 torus bits 的模二配对计算，公式为 $\operatorname{simple\_grading}(s,e)=\operatorname{offset}[s]\operatorname{XOR}\langle\alpha_s\bmod 2,\operatorname{torus\ bits}(e)\rangle$，结果 `true` 表示 noncompact（非紧）。调用方必须先通过 `InvolutionTable::simple_root_kind` 守卫根类型，确认其为 IMAGINARY。^[tits-element.md:47-49]

torus bits 来自 [[TitsElement 的元素表示与正规形契约|TitsElement]] 的二元组表示 `(involution, torus bits)`。grading offset 是构造 `TitsCoset` 时由调用方选定的输入：stage (d) 从 square-class cocharacter 导出；adjoint 约定为 `offset[s] = (twist(s) == s)`。^[tits-element.md:19-22, tits-element.md:38-40]

## 上下文一致性

`TitsCoset` 的来源校验要求完整 inner class 相等，不能仅检查 datum 相等。同一 datum 上不同的 distinguished involution 对应不同的 twist 与 transport，因此 datum-only 校验不足以保证操作语义。相关约束见 [[TitsCoset 的 grading offset 与完整 inner-class 门控]]。^[tits-element.md:38-43]

## 任意虚根的判定

`grading` 采用上游的 conjugate-to-simple 循环，判定任意 IMAGINARY 根的 noncompactness。若根在该元素处不是 imaginary，则返回 `None`。^[tits-element.md:61-62]

## 逆 Cayley 变换中的修复

[[逆 Cayley 变换的 grading 修复|inverse Cayley]] 必须处理 real source 处的模约化可能遗忘源端 grading 的情况。操作先执行裸的 `sigma_inv_mult`：反射左 torus 部分，并在左乘增加 Weyl 长度时加入 $m_\alpha$。若重建的根为 compact，则使用第一个与该根配对非平凡的 source mod-space 基向量修复；不存在这样的基向量时，报 `TitsCosetInvariantViolation`。^[tits-element.md:54-58]

修复后，操作在 imaginary target 处通过 `quotient_representative` 归约，并复核目标的 simple grading。若源根不是 real，或向下的 Cartan 类尚未加入表，则返回 `None`。这也是 [[Cayley 变换与目标模空间归约]] 中需要保留的目标端检查。^[tits-element.md:58-60]

## 证据范围

本页依据 `tits_element.rs` 的结构性阅读说明。来源未执行构建、测试或原版运行，不提供数学验收、性能或并行结论；其中上游 `tits.cpp` 行号转述自源码注释，未独立重读上游，可能随版本变化。^[tits-element.md:9-15, tits-element.md:65-72]

## Sources

- [tits-element.md](tits-element.md) — Tits 元素：torus 部分与 Tits 群操作（KGB stage c）

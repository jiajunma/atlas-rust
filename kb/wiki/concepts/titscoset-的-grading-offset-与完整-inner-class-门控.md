---
title: TitsCoset 的 grading offset 与完整 inner-class 门控
summary: TitsCoset 从 inner class 一次性建表，接受调用方指定的 grading offset，并要求完整 inner-class 相等，以防同一 datum 下不同 distinguished involution 的 twist 与 transport 被混用。
sources:
  - tits-element.md
kind: concept
createdAt: "2026-10-09T15:13:07.101Z"
updatedAt: "2026-10-09T15:13:07.101Z"
tags:
  - Tits群
  - 分级
  - 来源校验
aliases:
  - titscoset-的-grading-offset-与完整-inner-class-门控
  - T的GO与I门
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# TitsCoset 的 grading offset 与完整 inner-class 门控

`TitsCoset::new` 从 inner class 一次性建表，并接收调用方选定的 grading offset。其来源一致性门控要求 **完整 inner class 相等**，仅根数据（datum）相等不足以保证操作兼容。^[tits-element.md:36-43]

## Grading offset 的来源与作用

grading offset 的取值依赖调用场景：stage (d) 从 square-class cocharacter 导出；adjoint 约定为 `offset[s] = (twist(s) == s)`，即生成元被 twist 固定时取 `true`。^[tits-element.md:38-40]

对元素 `element`，简单根的 grading 按下式计算，其中 `true` 表示 noncompact：^[tits-element.md:47-49]

\[
\operatorname{simple\_grading}(s,\mathrm{element})
=
\mathrm{offset}[s]\;\operatorname{XOR}\;
\langle \alpha_s\bmod 2,\mathrm{torus\ bits}\rangle .
\]

此值仅在简单根 \(s\) 对该元素的 involution 为 IMAGINARY 时具有意义；调用方必须使用 `InvolutionTable::simple_root_kind` 守卫。相关分类见[[对合下的虚根、实根与复根分类]]。^[tits-element.md:47-49]

offset 也参与 [[Based cross action 的闭式实现]]：`cross` 的闭式映射等价于先执行 `sigma_mult(s, .)`，再执行 `mult_sigma_inv(., twist(s))`，并加入 offset 修正；结果在目标处归约。^[tits-element.md:50-51]

## 为什么要求完整 inner class 相等

同一 datum 上，具有不同 distinguished involution 的两个 inner class 会有不同的 twist 与 transport。因此，datum-only 检查无法保证这些操作兼容；源码说明明确要求 FULL inner-class 相等，以避免静默破坏操作语义。^[tits-element.md:40-43]

twist 置换与 simple-reflection 根置换由 coset 自有的派生副本保存，来源是表的私有缓存。相关结构可参见 [[Twisted involution 表与 Cartan 轨道存储]]。^[tits-element.md:42-43]

## 元素与归约契约

`TitsElement` 保存 `(involution, torus bits)`，利用 `InvolutionTable` 提供的 O(1) cross 链接与 Cayley 边，无须逐元素携带 Weyl 数据。其裸构造器 `TitsElement::new` 检查 involution 编号与 torus 维数，但不自动归约；正规形由幂等的 `reduce` 产生。该契约见 [[TitsElement 的元素表示与正规形契约]]。^[tits-element.md:19-22, tits-element.md:32-34, tits-element.md:63-63]

## 证据边界

本文依据对 `tits_element.rs` 的结构性阅读材料，其快照记录的是 dirty 工作区字节。来源未执行构建、测试或原版运行，不提供数学验收、性能或并行结论；上游 `tits.cpp` 行号仅转述源码注释，未独立复核。^[tits-element.md:9-15, tits-element.md:65-72]

## Sources

- [tits-element.md](tits-element.md) — Tits 元素：torus 部分与 Tits 群操作（KGB stage c）

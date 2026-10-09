---
title: TitsCoset 的 grading offset 与完整 inner-class 门控
summary: TitsCoset 接受调用方指定的 grading offset，并以完整 inner class 相等性校验来源，避免混用同一 datum 下不同 distinguished involution 的 twist 与运输数据。
sources:
  - tits-element.md
kind: concept
createdAt: "2026-10-09T15:13:07.101Z"
updatedAt: "2026-10-09T19:37:04.977Z"
tags:
  - Tits陪集
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
---

# TitsCoset 的 grading offset 与完整 inner-class 门控

`TitsCoset::new` 从 inner class 一次性建表，接受调用方选定的 grading offset。其来源一致性门控要求**完整 inner class 相等**：仅根数据（datum）相等不足以保证 twist 与 transport 的兼容性。^[tits-element.md:38-43]

## Grading offset 的来源与作用

grading offset 的来源随调用场景而异：stage (d) 从 square-class cocharacter 导出；adjoint 约定为 `offset[s] = (twist(s) == s)`，即生成元被 twist 固定时取 `true`。^[tits-element.md:38-40]

对于简单根 \(s\) 与元素 `element`，`simple_grading` 使用 offset 与根对 torus bits 的模二配对作异或：`offset[s] XOR <alpha_s mod 2, torus bits>`，结果为 `true` 表示 noncompact。该值仅在 \(s\) 对元素的 involution 为 IMAGINARY 时具有意义，调用方必须先用 `InvolutionTable::simple_root_kind` 守卫；相关语义见 [[虚根的 noncompact grading]]。^[tits-element.md:47-49]

offset 也参与 [[Based cross action 的闭式实现]]。`cross` 的闭式映射等价于先执行 `sigma_mult(s, .)`，再执行 `mult_sigma_inv(., twist(s))`，并加入 offset 修正；所得元素在目标处归约。^[tits-element.md:50-51]

## 完整 inner-class 门控

同一 datum 上具有不同 distinguished involution 的两个 inner class，会有不同的 twist 与 transport。因此，datum-only 门控可能静默破坏操作语义；来源明确要求以 FULL inner-class 相等作为来源一致性条件。^[tits-element.md:40-43]

twist 置换与 simple-reflection 根置换由 coset 自有的派生副本保存，这些副本来自表的私有缓存。相关表结构见 [[Twisted involution 表与 Cartan 轨道存储]]。^[tits-element.md:42-43]

## 元素与归约契约

`TitsElement` 采用 `(involution, torus bits)` 表示。由于 `InvolutionTable` 提供 O(1) 的 cross 链接与 Cayley 边，构造期也无须逐元素携带 Weyl 数据。^[tits-element.md:19-22]

`TitsElement::new` 检查 involution 编号与 torus 维数，但不自动归约。派生序关系按 involution 分组并比较原始 bits，只有对归约后的代表元才具有语义；正规形由幂等的 `reduce` 产生，参见 [[TitsElement 的元素表示与正规形契约]]。^[tits-element.md:32-34, tits-element.md:63-63]

## 证据边界

本页依据 `tits_element.rs` 的结构性阅读材料，所记录快照来自 dirty 工作区。来源未执行构建、测试或原版运行，不提供数学验收、性能或并行结论；其中上游 `tits.cpp` 行号仅转述源码注释，未独立重读上游，可能随版本变化。^[tits-element.md:9-15, tits-element.md:67-72]

## Sources

- [tits-element.md](tits-element.md) — Tits 元素：torus 部分与 Tits 群操作（KGB stage c）

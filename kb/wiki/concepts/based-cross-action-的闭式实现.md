---
title: Based cross action 的闭式实现
summary: cross 使用单一逐元素闭式映射，实现 sigma_mult(s, .) 后接 mult_sigma_inv(., twist(s)) 并加入 offset 修正的作用，结果在目标处归约。
sources:
  - tits-element.md
kind: concept
createdAt: "2026-10-09T15:13:23.657Z"
updatedAt: "2026-10-09T15:13:23.657Z"
tags:
  - Tits群
  - 交叉作用
  - 算法设计
aliases:
  - based-cross-action-的闭式实现
  - BCA的
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# Based cross action 的闭式实现

Based cross action 是 `TitsCoset::cross` 对单个 `TitsElement` 实施的闭式映射。其作用等价于先执行 `sigma_mult(s, .)`，再执行 `mult_sigma_inv(., twist(s))`，并加上 grading offset 修正；最终在目标处归约，得到 reduced 代表元。^[tits-element.md:24-30, tits-element.md:50-51]

## 元素表示与实现策略

`TitsElement` 以 `(involution, torus bits)` 表示元素。构造期也采用这一形状，因为 stage (b) 的 `InvolutionTable` 已提供 O(1) 的 cross 链接与 Cayley 边，无须为每个元素携带 Weyl 数据。相关表示约定见 [[TitsElement 的元素表示与正规形契约]]。^[tits-element.md:19-22]

模块文档将 based cross action 的单次闭式映射列为相对上游的实现策略差异，并记载该映射已逐步对照四个 sigma multiplications 验证。同一模块还以记录中 `WeylAction` 的 mod-2 矩阵传输替代上游 `push_across`／`pull_across` 的词遍历，参见 [[Torus 部分的模二矩阵传输]]。^[tits-element.md:24-30]

## Grading offset 与上下文约束

闭式作用包含调用方选定的 grading offset 修正。`TitsCoset::new` 从 inner class 一次性建表；stage (d) 从 square-class cocharacter 导出 offset，而 adjoint 约定为 `offset[s] = (twist(s) == s)`。^[tits-element.md:38-40]

上下文来源校验要求完整 inner class 相等，不能仅检查 datum 相等：同一 datum 上具有不同 distinguished involution 的 inner class，其 twist 与 transport 可能不同。twist 置换及 simple-reflection 根置换由 coset 持有派生副本。这一约束见 [[TitsCoset 的 grading offset 与完整 inner-class 门控]]。^[tits-element.md:40-43]

`simple_grading(s, element)` 的公式为 `offset[s] XOR <alpha_s mod 2, torus bits>`，其中 `true` 表示 noncompact。该值仅在 `s` 对当前元素的 involution 为 IMAGINARY 时有意义，调用方必须用 `InvolutionTable::simple_root_kind` 守卫；这一公式描述 grading 的求值，来源没有给出 cross 闭式映射的完整坐标公式。^[tits-element.md:47-51]

## 目标归约与正规形

`cross` 返回在目标处 reduced 的元素。裸构造器 `TitsElement::new` 只检查 involution 编号与 torus 维数，不自动归约；派生序关系按 involution 分组并比较 RAW bits，因此只有对 REDUCED 代表元才有语义。`reduce` 产生表内正规形，且具有幂等性。^[tits-element.md:32-34, tits-element.md:50-51, tits-element.md:63-63]

## 证据范围

本页依据的是对 `tits_element.rs` 的结构性阅读，所读字节来自 dirty 工作区快照。来源包没有执行构建、测试或原版运行，也不提供数学验收、性能或并行结论；其上游 `tits.cpp` 行号转述自源码注释，未独立重读上游。因此，模块所述的逐步对照验证应与独立的 [[HPC 验收证据链]] 区分。^[tits-element.md:9-15, tits-element.md:24-30, tits-element.md:67-72]

## Sources

- [tits-element.md](tits-element.md) — Tits 元素：torus 部分与 Tits 群操作（KGB stage c）

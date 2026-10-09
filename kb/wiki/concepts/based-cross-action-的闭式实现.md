---
title: Based cross action 的闭式实现
summary: cross 使用逐元素闭式映射，实现先 sigma_mult 再 mult_sigma_inv 并加入 offset 修正的作用，最终在目标对合处归约。
sources:
  - tits-element.md
kind: concept
createdAt: "2026-10-09T15:13:23.657Z"
updatedAt: "2026-10-09T22:50:39.528Z"
tags:
  - Tits陪集
  - 交叉作用
  - 算法
aliases:
  - based-cross-action-的闭式实现
  - BCA的
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: Based cross action 的闭式实现
summary: cross 以单一逐元素闭式映射实现 sigma_mult、mult_sigma_inv 与 grading offset 修正，并在目标对合处归约。
sources:
  - tits-element.md
kind: concept
tags:
  - Tits群
  - 交叉作用
  - 算法
aliases:
  - based-cross-action-的闭式实现
provenanceState: extracted
---

# Based cross action 的闭式实现

Based cross action 是 `TitsCoset::cross` 对单个 `TitsElement` 实施的闭式映射。其作用等价于先执行 `sigma_mult(s, .)`，再执行 `mult_sigma_inv(., twist(s))`，外加 grading offset 修正；结果在目标对合处归约为 reduced 代表元。^[tits-element.md:24-30, tits-element.md:50-51]

## 元素表示与实现策略

`TitsElement` 以 `(involution, torus bits)` 二元组表示元素，构造期也采用这一形状。由于 `InvolutionTable` 已提供 O(1) 的 cross 链接与 Cayley 边，每个元素无须携带 Weyl 数据。相关背景见 [[TitsElement 的元素表示与正规形契约]] 与 [[Twisted involution 表与 Cartan 轨道存储]]。^[tits-element.md:19-22]

模块文档将单一逐元素闭式映射列为相对上游的实现策略差异，并记载该映射已逐步对照四个 sigma multiplications 验证。同一模块还用记录中 `WeylAction` 的模二矩阵传输替代上游 `push_across`／`pull_across` 的词遍历，详见 [[Torus 部分的模二矩阵传输]]。^[tits-element.md:24-30]

## Grading offset 与上下文约束

`TitsCoset::new` 从 inner class 一次性建表，grading offset 由调用方选定：stage (d) 从 square-class cocharacter 导出，adjoint 约定则为 `offset[s] = (twist(s) == s)`。^[tits-element.md:38-40]

来源一致性校验要求**完整 inner class 相等**，仅检查 datum 相等不足以保证操作语义。同一 datum 上具有不同 distinguished involution 的 inner class，其 twist 与 transport 不同；coset 自有 twist 置换和 simple-reflection 根置换的派生副本。参见 [[TitsCoset 的 grading offset 与完整 inner-class 门控]]。^[tits-element.md:40-43]

`simple_grading(s, element)` 的计算式为 `offset[s] XOR <alpha_s mod 2, torus bits>`，其中 `true` 表示 noncompact。该值仅在 `s` 对当前元素的 involution 为 IMAGINARY 时有意义，调用方须用 `InvolutionTable::simple_root_kind` 守卫。^[tits-element.md:47-49]

## 目标归约与正规形

`cross` 返回的结果已在目标处归约。裸构造器 `TitsElement::new` 只检查 involution 编号与 torus 维数，不自动归约；派生序关系按 involution 分组并比较 RAW bits，只有在 REDUCED 代表元上才具有语义。`reduce` 产生 torus bits 的表内正规形，且具有幂等性。^[tits-element.md:32-34, tits-element.md:50-51, tits-element.md:63-63]

## 证据范围

来源包属于对 `tits_element.rs` 的结构性阅读，依据 dirty 工作区字节及快照 `snapshots/2026-10-03-tits-element.json`，不能将这些字节视为已提交源码。草稿经 Kimi probe 起草后，由维护者逐条对照源码核对改写。^[tits-element.md:9-15]

模块记载的逐步对照验证不等于来源包执行了独立测试：该包未执行构建、测试或原版运行，不包含数学验收、性能或并行结论。实现正确性仍属于其自身的 HPC 证据链，包括 KGB gate；上游 `tits.cpp` 行号转述自源码注释，未独立重读上游，可能随版本演进而漂移。^[tits-element.md:10-11, tits-element.md:24-30, tits-element.md:65-72]

## Sources

- [tits-element.md](../../sources/tits-element.md) — Tits 元素：torus 部分与 Tits 群操作（KGB stage c）。

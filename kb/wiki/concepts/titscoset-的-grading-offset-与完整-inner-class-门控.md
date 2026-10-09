---
title: TitsCoset 的 grading offset 与完整 inner-class 门控
summary: TitsCoset 使用调用方指定的 grading offset，并检查完整 inner class 相等性，防止混用同一 datum 上不同 distinguished involution 的 twist 与运输数据。
sources:
  - tits-element.md
kind: concept
createdAt: "2026-10-09T15:13:07.101Z"
updatedAt: "2026-10-09T22:50:32.435Z"
tags:
  - Tits陪集
  - 来源校验
  - 分级
aliases:
  - titscoset-的-grading-offset-与完整-inner-class-门控
  - T的GO与I门
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: TitsCoset 的 grading offset 与完整 inner-class 门控
summary: TitsCoset 接受调用方指定的 grading offset，并以完整 inner class 相等性校验来源，避免混用同一 datum 上不同 distinguished involution 的 twist 与 transport。
sources:
  - tits-element.md
kind: concept
tags:
  - Tits群
  - 来源校验
  - 分级
aliases:
  - titscoset-的-grading-offset-与完整-inner-class-门控
  - T的GO与I门
provenanceState: extracted
---

# TitsCoset 的 grading offset 与完整 inner-class 门控

`TitsCoset::new` 从 inner class 一次性建表，使用调用方选定的 grading offset。其来源一致性检查要求**完整 inner class 相等**：仅根数据（datum）相等，不能保证 twist 与 transport 的兼容性。^[tits-element.md:38-43]

## Grading offset 的来源与作用

grading offset 随调用场景确定：stage (d) 从 square-class cocharacter 导出；adjoint 约定为 `offset[s] = (twist(s) == s)`，即生成元被 twist 固定时取 `true`。^[tits-element.md:38-40]

`simple_grading(s, element)` 使用公式 `offset[s] XOR <alpha_s mod 2, torus bits>`，结果为 `true` 表示 noncompact（非紧）。该值仅在简单根 `s` 对该元素的 involution 为 IMAGINARY（虚根）时有意义；调用方必须通过 `InvolutionTable::simple_root_kind` 检查这一前提。^[tits-element.md:47-49]

offset 也参与 [[Based cross action 的闭式实现]]：`cross` 的闭式映射等价于先执行 `sigma_mult(s, .)`，再执行 `mult_sigma_inv(., twist(s))`，外加 offset 修正；结果在目标处归约。^[tits-element.md:50-51]

## 完整 inner-class 门控

同一 datum 上具有不同 distinguished involution 的两个 inner class，会有不同的 twist 与 transport。仅检查 datum 相等会允许不兼容的数据被静默混用，因此 provenance 门控以 FULL inner-class 相等为条件。^[tits-element.md:40-43]

twist 置换与 simple-reflection 根置换由 coset 自有的派生副本保存，它们是对表内私有缓存的命名复制。相关表结构见 [[Twisted involution 表与 Cartan 轨道存储]]。^[tits-element.md:42-43]

## 元素表示与归约契约

`TitsElement` 采用 `(involution, torus bits)` 表示。由于 `InvolutionTable` 提供 O(1) 的 cross 链接与 Cayley 边，构造期也采用这一形状，无须逐元素携带 Weyl 数据。^[tits-element.md:19-22]

`TitsElement::new` 检查 involution 编号与 torus 维数，但不自动归约。派生序关系按 involution 分组并比较原始 bits，只有对归约后的代表元才具有语义；表内正规形由幂等的 `reduce` 产生。详见 [[TitsElement 的元素表示与正规形契约]]。^[tits-element.md:32-34, tits-element.md:63-63]

## 证据边界

本页依据 `tits_element.rs` 的结构性阅读材料，所记录的源码快照来自 dirty 工作区。来源包未执行构建、测试或原版运行，不提供数学验收、性能或并行结论；正确性属于独立的 HPC 证据链，本页不扩展其范围。^[tits-element.md:9-15, tits-element.md:72-72]

来源中的上游 `tits.cpp` 行号转述自源码注释，未独立重读上游，可能随版本演进而漂移。^[tits-element.md:69-69]

## Sources

- [tits-element.md](../../sources/tits-element.md) — Tits 元素：torus 部分与 Tits 群操作（KGB stage c）。

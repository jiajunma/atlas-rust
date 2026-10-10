---
title: 逆 Cayley 变换的 grading 修复
summary: inverse_cayley 在裸 sigma_inv_mult 后用首个配对非平凡的源 mod-space 基向量修复紧根分级，再于虚根目标归约并复核；缺少修复向量时报不变量错误。
sources:
  - tits-element.md
kind: concept
createdAt: "2026-10-09T15:13:37.309Z"
updatedAt: "2026-10-10T00:53:18.157Z"
tags:
  - Tits群
  - Cayley变换
  - 分级
aliases:
  - 逆-cayley-变换的-grading-修复
  - 逆C变G修
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: 逆 Cayley 变换的 grading 修复
summary: inverse_cayley 在裸逆变换后，用首个与重构紧根配对非平凡的源模空间基向量修复 grading，再于虚根目标处归约并复核；缺少修复向量时报不变量错误。
sources:
  - tits-element.md
kind: concept
tags:
  - Cayley变换
  - 分级
  - 不变量
aliases:
  - 逆-cayley-变换的-grading-修复
provenanceState: extracted
---

# 逆 Cayley 变换的 grading 修复

grading 修复是 `TitsCoset::inverse_cayley` 中处理源侧分级信息丢失的步骤。实根源处的模约化可能已经遗忘源侧 grading，使裸逆变换重构出紧根。实现利用源 involution 的模空间修复 grading，随后在虚根目标处归约并复核。^[tits-element.md:54-60]

## grading 的含义与上下文

`simple_grading(s, element)` 按 `offset[s] XOR <alpha_s mod 2, torus bits>` 计算，结果 `true` 表示非紧（noncompact）。它仅在简单根 `s` 对该元素的 involution 为虚根时有意义；调用方须通过 `InvolutionTable::simple_root_kind` 守卫此前提。^[tits-element.md:47-49]

grading offset 由调用方选定。`TitsCoset` 的来源门控要求完整 inner class 相等：同一 datum 上不同的 distinguished involution 对应不同的 twist 与 transport，仅检查 datum 不足以保证一致性。参见 [[TitsCoset 的 grading offset 与完整 inner-class 门控]]。^[tits-element.md:38-43]

## 修复流程

首先执行裸的 `sigma_inv_mult`：反射左 torus 部分，并在左乘增加 Weyl 长度时加上 $m_\alpha$。随后处理源处模约化可能遗忘的 grading 信息。^[tits-element.md:54-57]

若重构根为紧根，实现使用**第一个与该根配对非平凡的源模空间基向量**进行修复。这里使用的是源 involution 的模空间；若不存在这样的基向量，则报告 `TitsCosetInvariantViolation`。^[tits-element.md:28-30, tits-element.md:54-58]

修复完成后，在虚根目标处调用 `quotient_representative` 归约，并复核目标的 simple grading。源模空间用于修复，目标模空间用于归约，二者的角色与操作顺序须保留。相关背景见 [[Cayley 变换与目标模空间归约]]。^[tits-element.md:54-60]

## 返回行为与正规形

当源根不是实根，或向下的 Cartan 类尚未加入表时，`inverse_cayley` 返回 `None`。这与需要修复却找不到相应基向量时报告 `TitsCosetInvariantViolation` 的情形不同。^[tits-element.md:54-60]

目标归约与 [[TitsElement 的元素表示与正规形契约]] 相衔接：`TitsElement::new` 检查 involution 编号与 torus 维数，但不自动归约；派生序关系按 involution 分组并比较原始位，仅在归约代表元上具有语义。表内 `reduce` 操作产生正规形且具有幂等性。^[tits-element.md:32-34, tits-element.md:63-63]

## 证据范围

本页依据对 `tits_element.rs` 的结构性阅读，所读字节来自 dirty 工作区，并由快照记录。来源文档未执行构建、测试或原版运行，不提供数学验收、性能或并行结论；其中的上游 `tits.cpp` 行号转述自源码注释，未独立重读上游核对。^[tits-element.md:9-15, tits-element.md:65-72]

## Sources

- [tits-element.md](../../sources/tits-element.md) — Tits 元素：torus 部分与 Tits 群操作（KGB stage c）。

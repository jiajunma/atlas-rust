---
title: 逆 Cayley 变换的 grading 修复
summary: inverse_cayley 先执行 sigma_inv_mult，再以首个配对非平凡的源 mod-space 基向量修复重构出的紧根 grading，随后在虚根目标归约并复核；缺少修复向量时报不变量错误，源根非实或向下 Cartan 类缺失时返回 None。
sources:
  - tits-element.md
kind: concept
createdAt: "2026-10-09T15:13:37.309Z"
updatedAt: "2026-10-09T15:13:37.309Z"
tags:
  - 逆Cayley变换
  - 分级修复
  - 算法不变量
aliases:
  - 逆-cayley-变换的-grading-修复
  - 逆C变G修
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 逆 Cayley 变换的 grading 修复

逆 Cayley 变换的 grading 修复是 `TitsCoset::inverse_cayley` 中恢复目标根非紧性的一步。real 源处的模约化可能已经遗忘源侧 grading，因此裸的逆变换可能重建出 compact 根；实现利用源 involution 的 mod-space 修复它，再在 imaginary 目标处归约并复核 grading。^[tits-element.md:54-60]

## grading 的含义与使用前提

`simple_grading(s, element)` 按下式计算，结果 `true` 表示 noncompact；只有当简单根 `s` 在该元素的 involution 处为 IMAGINARY 时才有意义，调用方必须通过 `InvolutionTable::simple_root_kind` 检查此前提。^[tits-element.md:47-49]

```text
offset[s] XOR <alpha_s mod 2, torus bits>
```

grading offset 是调用方选定的输入。`TitsCoset` 要求完整 inner class 相等，而不只是 datum 相等，因为同一 datum 上不同 distinguished involution 对应的 twist 与 transport 可能不同；这也是理解修复所依赖上下文的前提，参见 [[TitsCoset 的 grading offset 与完整 inner-class 门控]]。^[tits-element.md:38-43]

## 修复流程

首先应用裸的 `sigma_inv_mult`：反射左 torus 部分，并在左乘增加 Weyl 长度时加上 $m_\alpha$。这一步之后，重建根的 grading 仍可能受源处模约化造成的信息丢失影响。^[tits-element.md:54-56]

如果重建根为 compact，实现选取**第一个与该根配对非平凡的 source mod-space 基向量**进行修复。这里使用的是源 involution 的模空间；若不存在这样的基向量，则报 `TitsCosetInvariantViolation`。^[tits-element.md:54-58]

修复后，实现在 imaginary target 处调用 `quotient_representative` 归约，并复核目标的 simple grading。因此，源模空间负责修复，目标模空间负责选取归约代表元；二者在流程中承担不同职责。相关背景见 [[Cayley 变换与目标模空间归约]]。^[tits-element.md:54-60]

## 返回行为与正规形

当源根不是 real，或向下的 Cartan 类尚未加入表时，`inverse_cayley` 返回 `None`。这与找不到修复基向量时报告不变量错误的情形不同。^[tits-element.md:54-60]

`TitsElement::new` 只检查 involution 编号与 torus 维数，不自动归约；其派生序关系比较 RAW bits，只有在 REDUCED 代表元上才有语义。逆 Cayley 变换末尾的目标归约应结合 [[TitsElement 的元素表示与正规形契约]] 理解，表内 `reduce` 操作本身是幂等的。^[tits-element.md:32-34, tits-element.md:58-63]

## 证据范围

本说明依据对 `tits_element.rs` 的结构性阅读，所读快照来自 dirty 工作区。源文档未执行构建、测试或原版运行，因此上述内容说明实现机制，不构成数学验收或性能结论；其中上游 `tits.cpp` 行号也仅转述自源码注释，未独立重读核对。^[tits-element.md:9-15, tits-element.md:65-72]

## Sources

- [tits-element.md](tits-element.md) — Tits 元素：torus 部分与 Tits 群操作（KGB stage c）

---
title: Tits 元素：torus 部分与 Tits 群操作（KGB stage c）
source: atlas-rust/tits-element
ingestedAt: 2026-10-03T10:32:42Z
---

# Tits 元素：torus 部分与 Tits 群操作（KGB stage c）

编辑状态：**结构性阅读，草稿经 Kimi probe 起草、维护者逐条对照源码核对后改写**。
本包解释 `tits_element.rs` 的元素形状与操作语义；其正确性属于它自己的 HPC
证据链（KGB gate 等），本包不重述也不扩展。所读字节见
[`snapshots/2026-10-03-tits-element.json`](snapshots/2026-10-03-tits-element.json)
（`tits_element.rs` SHA-256
`eae08af3f1d943b23f04e4530c5d64a8da4a4a72b79722a812bbfa3192990fa5`，dirty
工作区）。

## 定位与元素形状

KGB stage c：torus 部分与 Tits 群操作。`TitsElement` 是二元组
`(involution, torus bits)`——上游 PERSISTED 的 per-element 形状，本实现在
构造期也采用它，因为 stage (b) 的 `InvolutionTable` 提供 O(1) 的 cross
链接与 Cayley 边，故不携带 per-element Weyl 数据。

相对上游的三处实现策略差异（模块文档载明）：(1) 上游 `push_across`/
`pull_across` 的 word walks 被来自记录中存储的 `WeylAction` 的 mod-2
matrix transport 取代（上游自己注释命名的 sophistication，tits.cpp:425-432）；
(2) based cross action 是单一的 closed-form per-element map，已逐步对照四个
sigma multiplications 验证（tits.cpp:469-503）；(3) inverse Cayley 用源
involution 的 mod-space 修复 compact reconstructed grading，然后在目标处
reduce（tits.cpp:605-644）。

`TitsElement::new` 是受门控的裸构造器（检查 involution 编号与 torus 维数），
**不**自动 reduce；派生的序关系按 involution 分组，比较 RAW bits，仅在
REDUCED 代表元上有语义（正规形契约由 `reduce` 产生）。

## TitsCoset：表与 grading offset

`TitsCoset::new` 从 inner class 一次性建表。grading offset 是调用方选定的
输入：stage (d) 从 square-class cocharacter 导出；adjoint 约定为
`offset[s] = (twist(s) == s)`。**provenance 门是 FULL inner-class 相等**而非
仅 datum 相等：同一 datum 上不同 distinguished involution 的两个 inner class
有不同的 twist 与 transport，datum-only 门会静默破坏一切。twist 置换与
simple-reflection 根置换是 coset 自有的派生副本（对表的私有缓存的命名复制）。

## 操作语义

- `simple_grading(s, element)`：`offset[s] XOR <alpha_s mod 2, torus bits>`，
  `true` 表示 noncompact；仅当 `s` 在该元素的 involution 处为 IMAGINARY 时
  有意义——调用方须以 `InvolutionTable::simple_root_kind` 守卫。
- `cross`：在目标处 reduced 的 based cross action；其 closed-form 等于先
  `sigma_mult(s, .)` 再 `mult_sigma_inv(., twist(s))`，外加 offset 修正。
- `cayley`：裸的 `sigma_mult`，在目标处 grown 的 mod-space 中 reduce；目标
  Cartan 类尚未加入表时返回 `None`。
- `inverse_cayley`：先作用裸的 `sigma_inv_mult`（反射左 torus 部分，左乘增加
  Weyl 长度时加 $m_\alpha$）；随后是**修复**：real source 处的模约化可能已
  遗忘 source-side grading，一个 compact reconstructed root 由「第一个与它
  配对非平凡的 source mod-space 基向量」修复（无这样的基向量则报
  `TitsCosetInvariantViolation`）；最后在 imaginary target 处
  `quotient_representative` 归约，并复核目标的 simple grading。根不是 real
  或向下的 Cartan 类未加入表时返回 `None`。
- `grading`：任意 IMAGINARY 根的 noncompactness，用上游的
  conjugate-to-simple loop；根在该元素处非 imaginary 时返回 `None`。
- `reduce`：该元素 torus bits 的表内正规形（幂等）。

## 来源与限制

- 源码：[tits_element.rs](../../../crates/atlas-real-group/src/tits_element.rs)；
  阅读快照 [`2026-10-03-tits-element.json`](snapshots/2026-10-03-tits-element.json)。
- 上游行号均转述自源码注释（tits.cpp），未独立重读上游，随版本演进可能漂移。
- 关联：[Twisted involution 表](involution-table.md)、
  [KGB 图结构](kgb-graph-structure.md)、[Weyl 群层](weyl-layer.md)。
- 本包未执行任何构建、测试或原版运行，不含数学验收、性能或并行结论。
- 起草经由本地 Kimi probe（无工具 profile，`kimi-code/k3-256k`；exit 0，
  116.8s，420 秒期限）。草案由维护者对照源码逐条核对改写；其「待源码核对」
  项中涉及 `new` 的门控、`inverse_cayley` 的修复机制、`reduce` 的幂等性与
  `TitsCoset` 的 full-inner-class 门控的内容均已按源码落实，其余骨架内容
  未采用。调用记录见快照的 `kimi_assist`。

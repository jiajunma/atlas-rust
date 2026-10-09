---
title: KGB 种子 x0：stable_log、基本余权与 RealFormSeed
source: atlas-rust/real-form-seed
ingestedAt: 2026-10-03T10:32:42Z
---

# KGB 种子 x0：stable_log、基本余权与 RealFormSeed

编辑状态：**结构性阅读，草稿经 Kimi probe 起草、维护者逐条对照源码核对后改写**。
本包解释 `real_form_seed.rs` 的种子构造；其正确性属于它自己的 HPC 证据链
（fundamental-lattice 与 seed gate 等），本包不重述也不扩展。所读字节见
[`snapshots/2026-10-03-real-form-seed.json`](snapshots/2026-10-03-real-form-seed.json)
（`real_form_seed.rs` SHA-256
`3f6620bb048040a5525b2f2385cc239d9e7ac29f8cd23cd5cea7d8ad612ca4ab`，dirty
工作区）。

## 定位

KGB 种子 $x_0$（stage d）的数学基底。按 `SEED_X0_DESIGN.md` 分两步落地：先
精确有理数机制（`stable_log` 与 fundamental coweights），再 `RealFormSeed`
构建器。**此处的选举是 OBSERVABLE-BEARING**：`stable_log` 的 adapted-basis
代表元固定 `g_rho_check`，进而固定每一个下游 `torus_factor` 有理数。

## stable_log：选举的 $\xi^T$-稳定对数

上游 `stable_log`（y_values.cpp:155-166）：(1) 输入逐坐标 mod 1 归约（取非负
剩余）；(2) 取 $\xi + 1$ 的前 $d$ 个 adapted-basis 坐标；(3) 将这些坐标再
mod 1（即模整个 fixed lattice）；(4) 转换回原坐标。输出**恰好**落在
+1-eigenspace。**前置条件（已检查）**：归约后输入的尾部 adapted-basis 坐标
必须为整数——等价于输入模 $X_*$ 同余于一个 exactly-fixed 向量；`some_coch`
输入在结构上满足，一般 squares 不必满足。

## fundamental_coweights：基本余权

$\varpi_i^\vee = \sum_j (C^{-1})_{ji}\,\alpha_j^\vee$，以 full lattice-rank
坐标表示——即 $\langle\alpha_s, \varpi_i^\vee\rangle = \delta_{si}$ 的唯一
有理解，且 radical 分量为零（rootdata.cpp:850-853, 1015-1016）。实现用
`invert_rational` 的精确有理高斯消元求 $C^{-1}$，再把每列按**实际 simple
coroot** 展开（不是恒等填充的坐标轴）。rank-bounded 的有理消元不携带 budget
knob（crate 的记录纪律）。

辅助：`solve_mod_two`（$\mathbb{F}_2$ 上求解，用上游的 canonical section
选举——解使用前 $d$ 个**输入列**而非 full augmented-space reduction 的任意
preimage；输入在 image 之外时返回 `None`）、`fractional_part`（非负小数部分）、
`invert_rational`（first-nonzero pivot 的精确有理逆，返回逆矩阵的列）。

## RealFormSeed：种子与门控

一个弱实形式的 KGB 种子：选举的 base-grading offset、精确的 square-class
cocharacter、以及在 fundamental involution 处归约的种子元素——在同一个共享
表的编号下（调用方对每个 inner class 保持一个 append-only 表）。私有字段：
公开构造可能组装出不匹配的三元组；构造不变量 `grading_offset ==
grading_of_simples(cocharacter)`；种子验证是上游自己的 x0-compacts 断言
（innerclass.cpp:1090-1092）。

`build` 的门控链：(i) 表的 inner class 相等（TitsCoset idiom）；(ii)
classification 的 fundamental class 归一化为该 datum 与 delta 上的恒等
twisted involution（逐一比较 Weyl action 与根 involution 矩阵）；(iii)/(iv)
强层的计数一致性降级与 form id 越界检查。`custom` 对应上游
`real_form_value::build` 的 custom 分支（atlas-types.w:3534-3545）：显式
(cocharacter, torus part) 对，cocharacter 的 simple pairings 须为整数，torus
part 须复现该 form 的 compact pattern。

访问器：`form()`、`grading_offset()`、`square_class_cocharacter()`、
`element()`。

## 来源与限制

- 源码：[real_form_seed.rs](../../crates/atlas-real-group/src/real_form_seed.rs)；
  阅读快照
  [`2026-10-03-real-form-seed.json`](snapshots/2026-10-03-real-form-seed.json)。
- 上游行号均转述自源码注释（y_values.cpp/rootdata.cpp/atlas-types.w/
  innerclass.cpp/realredgp.cpp），未独立重读上游，随版本演进可能漂移。
- 关联：[KGB 图结构](kgb-graph-structure.md)、
  [Twisted involution 表](involution-table.md)、
  [Tits 元素](tits-element.md)、[Cartan 分类](cartan-classification.md)；
  `adapted_basis`、`IntegerLatticeBudget`、`StrongRealClassification` 的展开
  属于后续来源包。
- 本包未执行任何构建、测试或原版运行，不含数学验收、性能或并行结论。
- 起草经由本地 Kimi probe（无工具 profile，`kimi-code/k3-256k`；exit 0，
  158.0s，420 秒期限）。草案由维护者对照源码逐条核对改写；其「待源码核对」
  项中涉及 `stable_log` 前置条件、`fundamental_coweights` 的实际余根展开、
  `build` 门控链的内容均已按源码落实，其余骨架内容未采用。调用记录见快照的
  `kimi_assist`。

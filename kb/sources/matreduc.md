---
title: 精确整数矩阵约化：matreduc 的逐操作移植
source: atlas-rust/matreduc
ingestedAt: 2026-10-03T10:32:42Z
---

# 精确整数矩阵约化：matreduc 的逐操作移植

编辑状态：**结构性阅读，草稿经 Kimi probe 起草、维护者逐条对照源码核对后改写**。
本包解释 `matreduc.rs` 的移植策略与操作面；其逐位保真证据属于它自己的
HPC 证据链（模块文档记载已与编译版 oracle 逐位验证，本包不重跑）。所读字节见
[`snapshots/2026-10-03-matreduc.json`](snapshots/2026-10-03-matreduc.json)
（`matreduc.rs` SHA-256
`49897e12a2572dffa8d9ce34022308d871d2a9893836732d035563be6a97cb92`，dirty
工作区）。

## 移植策略：为什么逐操作复现

本模块移植上游 `utilities/matreduc.cpp`（`diagonalise`、`gcd`、
`has_solution`、`find_solution`）。动机（文档载明）：欠定系统 $A x = b$ 的
**被选解在下游可观测**——$\tau$/$t$ 坐标的奇偶性会进入
`ext_block::same_sign`——因此本移植复现 C++ 算法的确切幺模操作序列，包括
其行列式符号簿记。算术为 wrapping `i32`，精确镜像上游 C++ `int`（包括其
溢出行为域——在被选解中可观测）。本模块承诺的不是"任意正确解"，而是与上游
逐操作一致的那个解。

## IntMatrix 与对角化

`IntMatrix`（crate 内部）：行主序矩形整数矩阵。`diagonalise(m)` 返回
`(row, col, diagonal)`：幺模的 `row`、`col` 使 `row * m * col` 对角；对角
元素除第一个外均为正——这是对 `matreduc::diagonalise` 的 operation-faithful
移植，含其精确符号簿记。

## 求解与像判定

- `has_solution(a, b)`：判定 $a x = b$ 是否有整数解——先对角化，左乘
  `row` 后逐坐标检查可除性。
- `find_solution(a, b)`：返回一个解，无整数解时返回 `None`（上游此时抛
  异常；此处调用方预期已先运行 `has_solution`）。
- `in_left_image`/`in_right_image`：像判定（这两个的注释归属
  `ext_block.cpp` 的 `in_L_image`/`in_R_image`，与 matreduc.cpp 区分开）。
- `inverse_upper_triangular(m)`（上游 `matrix::inverse_upper_triangular`，
  matrix.cpp:420-440）：单位上三角矩阵的回代逆，wrapping `i32`；输入非方阵
  或某对角元不为 1 时报错。
- `exp_i(n)`（上游 `arithmetic::exp_i`，arithmetic.h:49-51）：对**偶数** $n$
  给出 $i^n$（即 $\pm 1$）；奇数前置条件在上游是 `assert`，此处为
  `debug_assert!`。

## 来源与限制

- 源码：[matreduc.rs](../../../crates/atlas-real-group/src/matreduc.rs)；
  阅读快照 [`2026-10-03-matreduc.json`](snapshots/2026-10-03-matreduc.json)。
- 上游行号均转述自源码注释（matreduc.cpp/matrix.cpp/arithmetic.h/
  ext_block.cpp），未独立重读上游，随版本演进可能漂移。
- 关联：[整数格](integer-lattice.md)、[ext_param/star 层](ext-param.md)、
  [图像基对](real-projection.md)。
- 本包未执行任何构建、测试或原版运行，不含数学验收、性能或并行结论。
- 起草经由本地 Kimi probe（无工具 profile，`kimi-code/k3-256k`；exit 0，
  75.6s，420 秒期限）。草案由维护者对照源码逐条核对改写；其「待源码核对」
  项中涉及 `diagonalise` 返回值、`find_solution` 的 None-vs-异常、
  `exp_i` 的 debug_assert 前置的内容均已按源码落实，其余骨架内容未采用。
  调用记录见快照的 `kimi_assist`。

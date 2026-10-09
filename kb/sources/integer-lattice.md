---
title: 精确整数格线性代数：预算、饱和核与可观测基
source: atlas-rust/integer-lattice
ingestedAt: 2026-10-03T10:32:42Z
---

# 精确整数格线性代数：预算、饱和核与可观测基

编辑状态：**结构性阅读，草稿经 Kimi probe 起草、维护者逐条对照源码核对后改写**。
本包解释 `integer_lattice.rs` 的精确算术基础设施；其正确性属于它自己的 HPC
证据链（fundamental-lattice 与 torus gate 等），本包不重述也不扩展。所读字节
见 [`snapshots/2026-10-03-integer-lattice.json`](snapshots/2026-10-03-integer-lattice.json)
（`integer_lattice.rs` SHA-256
`e8891446aacb9caa87c69e9b528bf71fbbf3b8ea0b29531ca58668d193f8368e`，dirty
工作区）。

## 定位

在 Malachite 大整数（`Integer`）上的精确格线性代数。Atlas 特定的分层：预算
类型先行，所有构造/收集入口先经形状或预算校验再触碰存储。

## IntegerLatticeBudget：计算预算而非秩限制

文档明确：这些是**计算预算**，不是数学秩限制——约束单次计算的每个矩阵维数
（`max_rank`）、存活工作条目总量（`max_entries`）、初等操作次数
（`max_steps`）与中间系数位长（`max_coefficient_bits`）。四个字段私有，
`new` 是 `pub const fn`。

## IntegerMatrix 与 BezoutTransform

`IntegerMatrix`：Malachite 整数上的行主序精确矩阵（`rows`/`columns`/
`entries` 均 `pub`）；`from_i32_entries` 先 `checked_shape` 校验、再分配、
再逐元素转换，形状不符报 `InvalidIntegerMatrixShape`。`BezoutTransform`（私有）
携带两个元素关联的幺模变换系数：由 `extended_gcd` 得 `s, t`，`u`、`v` 为
除以 gcd 的精确商。

## 核与归约

- `saturated_kernel(matrix, budget)`：饱和整数核的基——先过预算检查，再做
  保持幺模右因子 $V$ 的行列混合约化；矩阵对角化后，`V` 中属于零对角元的列
  构成完整整数核。**刻意不做**「有理行约化再通分」。
- `reduce_basis_mod_two`：把整数基模 2 归约，只保留其在 $Y/2Y$ 中的张成。
- `negative_coweight_eigenspace`：从 cocharacter 作用本身计算
  $\ker_{\mathbb Z}(I + \theta_Y)$——`LatticeInvolution::coweight_matrix()`
  已经存储余特征上的对偶作用，故本函数有意不再转置。

## 关系格封装

`RelationMatrix(IntegerMatrix)` 及同族构造器（`preflight_shape` 在分配或复制
任何条目之前校验形状；`from_i32_iter` 在迭代器被推进或存储被保留之前拒绝超大
形状）；`RelationBasis`（`factors()`/`into_parts()`）；
`RelationGenerator::try_collect` 仅在借用生成元描述符的计数及其蕴含的分子
矩阵通过关系格预算后才收集；`RelationError` 是该族的错误类型。

## adapted_basis：可观测基

上游 `matreduc::adapted_basis`（matreduc.cpp:262-336 含其 `gcd` 辅助函数，
matreduc.h:70-122）的忠实移植：伴随跟踪 LEFT 变换的逆而非事后求逆；主元策略
逐字复制（"minimal use of row operations"、首个最小 gcd 种子、
`find_small_remainder` 的 rotate 步骤、kept-rows-first 重排）。动机（文档
载明）：所选基是 **OBSERVABLE-BEARING**——它固定 `stable_log` 代表元，进而
固定 `g_rho_check` 与下游每个 `torus_factor` 有理量（见
[KGB 种子](real-form-seed.md)）。实现先预检条目总量（工作矩阵 + 基 + 逆 +
对角线），再逐行 `gcd_row_to_pivot` 并循环 `find_small_remainder` 消元。

派生函数族：`adapted_relation_basis`、`filter_relation_units`、
`replace_relation_generators`、`annihilator_modulo`、`quotient_relation_basis`
等。

## 来源与限制

- 源码：[integer_lattice.rs](../../crates/atlas-real-group/src/integer_lattice.rs)；
  阅读快照
  [`2026-10-03-integer-lattice.json`](snapshots/2026-10-03-integer-lattice.json)。
- 上游行号均转述自源码注释（matreduc.h/matreduc.cpp），未独立重读上游，随
  版本演进可能漂移。
- 关联：[KGB 种子](real-form-seed.md)、[Cartan 分类](cartan-classification.md)、
  [根坐标与格坐标](../wiki/math/root-coordinates.md)；`ModTwoSubspace` 与
  `ModTwoSubquotient` 的展开属于后续来源包。
- 本包未执行任何构建、测试或原版运行，不含数学验收、性能或并行结论。
- 起草经由本地 Kimi probe（无工具 profile，`kimi-code/k3-256k`；exit 0，
  161.7s，420 秒期限）。草案由维护者对照源码逐条核对改写；其「待源码核对」
  项中涉及 `saturated_kernel` 的幺模右因子、`adapted_basis` 的预算预检与
  主元策略的内容均已按源码落实，其余骨架内容未采用。调用记录见快照的
  `kimi_assist`。

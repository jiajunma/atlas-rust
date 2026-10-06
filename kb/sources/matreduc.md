---
title: 精确整数矩阵约化：matreduc 的逐操作移植
source: atlas-rust/matreduc
ingestedAt: 2026-10-03T10:32:42Z
---

# 精确整数矩阵约化：matreduc 的逐操作移植

编辑状态：**结构性阅读（两次：2026-10-03 初读、2026-10-06 重读，字节未变，
SHA-256 相同）；两份草案均经 Kimi probe 起草、维护者逐条对照源码核对后改写合并**。
本包解释 `matreduc.rs`（755 行）的移植策略与操作面；其逐位保真证据属于它自己的
HPC 证据链（模块文档记载已与编译版 oracle 逐位验证，本包不重跑）。所读字节见
[`snapshots/2026-10-03-matreduc.json`](snapshots/2026-10-03-matreduc.json)
（初读）与
[`snapshots/2026-10-06-real-projection-matreduc.json`](snapshots/2026-10-06-real-projection-matreduc.json)
（重读，同一 SHA-256 `49897e12…`，与 real_projection.rs 同包进行）。

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
移植，含其精确符号簿记。空形状（0×n 或 n×0）提前返回。

细节（2026-10-06 重读补充，含逐行追踪）：

- `divide(a, b)`：正除数下取整除法（`a >= 0` 直除，否则
  `-1 - ((-1 - a) / b)`，避开 `i32::MIN` 取负）；
- `gcd(row, &mut flip, dest)`：最小绝对值主元（`wrapping_abs`）；负主元取正
  时翻转 `flip` 并在记录矩阵置 `col.set(mindex, mindex, -1)`；`divide` 消元；
  末尾列交换到 `dest` 也翻 `flip`；
- `diagonalise` 的簿记怪癖：每列首个 gcd 的 `flip` 对 `row_minus` 是**覆盖**
  赋值；内层循环交替行/列 gcd，`flip` 分别 `^=` 进 `col_minus`/`row_minus`；
  退出后再 `row_minus ^= flip`（从行 gcd 的 break 退出时该 `flip` 计入两侧；
  从列 gcd 的 break 退出时净效果抵消）；主元列未左对齐时用稳定排列
  `pull_back_columns` 并异或置换符号（逆序对奇偶）；最后
  `row_minus != col_minus` 时 `diagonal[0]` 取负，`row_minus`/`col_minus`
  分别经第 0 行/列乘 −1 归一（注释口径不一致：一处说 "ensure det(row)=1"，
  测试注释说上游只强制 `det(col)==1`、`det(row)` 可为 −1——以测试为准）；
- panic 面：`from_entries` 形状、`apply_to`/`right_prod` 长度、`transpose`
  方阵、`row_apply`/`column_apply` 边界全部 `assert`——故 `has_solution`
  等对长度不匹配的 `b` 是 panic 而非 `Err`。

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
  `debug_assert!`（release 下奇数输入落 `-1` 分支，无防护）。

测试锚点（2026-10-06 重读补充，6 个）：11 用例重构（`|det(row)| == 1`、
`det(col) == 1`、逐对角核对、首项外为正）；`find_solution` 三形态（满秩、
秩亏、矩形）；像判定一维例；**oracle_reference_cases**——取自 C++ oracle
的逐字节锚定（`[[0,5],[0,0]]` → `diagonal [−5]` 与精确 `row`/`col`；
`[[−4]]` → `[−4]`；6×6 秩亏且主元列未左对齐的完整 `row`/`col` 字面量与解
`[6,14,5,−37,−421,345]`）；单位上三角逆两例 + 两拒绝；`exp_i` 五点。未测：
wrapping 溢出域（文档声明可观测）、`in_*_image` 的矩形/秩亏、空形状
`diagonalise`、`from_entries`/`apply_to` 的 panic 路径。

## 来源与限制

- 源码：[matreduc.rs](../../../crates/atlas-real-group/src/matreduc.rs)；
  阅读快照 [`2026-10-03-matreduc.json`](snapshots/2026-10-03-matreduc.json)
  （初读）与
  [`2026-10-06-real-projection-matreduc.json`](snapshots/2026-10-06-real-projection-matreduc.json)
  （重读，同一 SHA-256 `49897e12…`）。
- 上游行号均转述自源码注释（matreduc.cpp/matrix.cpp/arithmetic.h/
  ext_block.cpp），未独立重读上游，随版本演进可能漂移。
- 关联：[整数格](integer-lattice.md)、[ext_param/star 层](ext-param.md)、
  [图像基对](real-projection.md)。
- 本包未执行任何构建、测试或原版运行，不含数学验收、性能或并行结论。
- 初读起草经由本地 Kimi probe（exit 0，75.6s）；重读同样经 Kimi probe
  （1200s 期限，exit 0，455.9s），其 `diagonalise` 符号簿记逐行追踪与
  oracle_reference_cases 普查均精确，已并入正文。调用记录见两份快照的
  `kimi_assist`。

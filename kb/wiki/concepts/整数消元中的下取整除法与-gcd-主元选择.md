---
title: 整数消元中的下取整除法与 gcd 主元选择
summary: divide 对正除数实现下取整并避开 i32::MIN 取负，gcd 使用 wrapping_abs 选择主元并记录负主元取正及列交换的符号变化。
sources:
  - matreduc.md
kind: concept
createdAt: "2026-10-09T15:00:21.647Z"
updatedAt: "2026-10-10T00:42:59.473Z"
tags:
  - 整数消元
  - 整数算术
aliases:
  - 整数消元中的下取整除法与-gcd-主元选择
  - 整G主
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: 整数消元中的下取整除法与 gcd 主元选择
summary: divide 对正除数实现下取整并避开 i32::MIN 取负；gcd 使用 wrapping_abs 选择主元，以 flip 记录负主元取正和列交换的符号变化。
sources:
  - matreduc.md
kind: concept
tags:
  - 整数消元
  - 主元选择
aliases:
  - 整数消元中的下取整除法与-gcd-主元选择
provenanceState: extracted
---

# 整数消元中的下取整除法与 gcd 主元选择

`matreduc` 的下取整除法、gcd 主元选择与符号记录服务于逐操作移植。欠定整数系统 $Ax=b$ 的被选解在下游可观测：$\tau$/$t$ 坐标的奇偶性会进入 `ext_block::same_sign`。因此，移植需要复现上游 C++ 的确切幺模操作序列及其选出的解，包含行列式符号簿记。^[matreduc.md:20-26]

## 正除数下的下取整除法

`divide(a, b)` 要求除数 $b>0$。当 $a\ge 0$ 时直接使用整数除法；当 $a<0$ 时使用 `-1 - ((-1 - a) / b)`，实现向下取整，并避开直接对 `i32::MIN` 取负。^[matreduc.md:37-38]

模块采用 wrapping `i32` 算术。来源将其描述为对上游 C++ `int` 算术行为的镜像，包括可能影响被选解的溢出行为域；这一约定属于逐操作兼容目标，但来源列出的测试尚未覆盖 wrapping 溢出域。^[matreduc.md:24-26, matreduc.md:75-76]

## gcd 主元选择与符号记录

`gcd(row, &mut flip, dest)` 使用 `wrapping_abs` 选择最小绝对值主元，并通过 `divide` 消元。负主元取正时，算法翻转 `flip`，同时在记录矩阵中执行 `col.set(mindex, mindex, -1)`；末尾将主元列交换到 `dest` 时，也翻转 `flip`。这些符号变化随后参与对角化的行列式簿记。^[matreduc.md:39-44]

上述操作用于 `diagonalise(m)`，其返回值为 `(row, col, diagonal)`。其中 `row`、`col` 为幺模矩阵，使 `row * m * col` 对角化；对角元素除第一项外均为正。对于 $0\times n$ 或 $n\times0$ 的空形状，函数提前返回。^[matreduc.md:30-33]

## 与整体对角化的衔接

`flip` 的合并顺序是兼容性的一部分：每列首个 gcd 的 `flip` **覆盖赋值**给 `row_minus`；内层交替进行行、列 gcd，将 `flip` 分别异或累积到 `col_minus`、`row_minus`。退出循环后再执行 `row_minus ^= flip`：从行 gcd 的 `break` 退出时，该 `flip` 计入两侧；从列 gcd 的 `break` 退出时，其净效果抵消。^[matreduc.md:42-45]

主元列未左对齐时，算法使用稳定排列 `pull_back_columns` 调整列位置，并按逆序对奇偶性计入置换符号。最后，若 `row_minus != col_minus`，则将 `diagonal[0]` 取负，再分别按两个标志对第 0 行或列乘以 $-1$ 归一。详细背景见 [[对角化的行列式符号簿记]]。^[matreduc.md:45-49]

源码注释对归一化目标存在不同口径：一处写为确保 `det(row)=1`，测试注释则说明上游只强制 `det(col)==1`，而 `det(row)` 可以为 $-1$。来源明确以测试口径为准。^[matreduc.md:48-49]

## 求解接口与失败行为

`has_solution(a, b)` 先对角化，将 `b` 左乘 `row`，再逐坐标检查可除性。`find_solution(a, b)` 返回一个整数解，无整数解时返回 `None`；上游在此情形抛出异常，Rust 调用方预期已先运行 `has_solution`。^[matreduc.md:56-59]

长度不匹配由断言处理。`apply_to`、`right_prod` 等操作检查输入长度，因此 `has_solution` 等函数接收长度不匹配的 `b` 时会 panic，而非返回 `Err`。这一失败情形应与无整数解区分。^[matreduc.md:50-52]

## 测试与证据边界

来源记录了 11 个对角化重构用例，检查 `|det(row)| == 1`、`det(col) == 1`、逐对角结果及首项外为正；`find_solution` 的测试覆盖满秩、秩亏和矩形系统。^[matreduc.md:69-71]

`oracle_reference_cases` 固定了来自 C++ oracle 的精确结果，包括矩阵 $\begin{pmatrix}0&5\\0&0\end{pmatrix}$ 对应 `diagonal = [-5]` 及精确的 `row`、`col`，以及一个主元列未左对齐的 $6\times6$ 秩亏案例。后者记录了完整变换矩阵与被选解 `[6,14,5,-37,-421,345]`。^[matreduc.md:71-74]

测试缺口包括 wrapping 溢出域、像判定的矩形与秩亏输入、空形状对角化，以及 `from_entries`、`apply_to` 的 panic 路径。来源属于结构性阅读，未执行构建、测试或原版运行；模块文档记载的编译版 oracle 逐位验证属于其自身的 HPC 证据链，本包未重跑，也不提供数学验收、性能或并行结论。^[matreduc.md:9-16, matreduc.md:75-76, matreduc.md:89-89]

## Sources

- [matreduc.md](../../sources/matreduc.md) — 精确整数矩阵约化：matreduc 的逐操作移植。

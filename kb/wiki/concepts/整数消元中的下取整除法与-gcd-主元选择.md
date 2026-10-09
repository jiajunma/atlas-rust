---
title: 整数消元中的下取整除法与 gcd 主元选择
summary: divide 对正除数实现下取整并避开 i32::MIN 取负，gcd 使用 wrapping_abs 选择主元并记录负主元取正和列交换的符号变化。
sources:
  - matreduc.md
kind: concept
createdAt: "2026-10-09T15:00:21.647Z"
updatedAt: "2026-10-09T22:39:03.322Z"
tags:
  - 整数消元
  - 主元选择
aliases:
  - 整数消元中的下取整除法与-gcd-主元选择
  - 整G主
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: 整数消元中的下取整除法与 gcd 主元选择
summary: divide 对正除数实施下取整并避免直接取负 i32::MIN；gcd 使用 wrapping_abs 选择最小绝对值主元，记录主元取正和列交换的符号变化。
sources:
  - matreduc.md
kind: concept
tags:
  - 整数线性代数
  - 消元算法
aliases:
  - 整数消元中的下取整除法与-gcd-主元选择
provenanceState: extracted
---

# 整数消元中的下取整除法与 gcd 主元选择

`matreduc` 的下取整除法、gcd 主元选择和符号记录共同服务于逐操作移植：欠定整数系统 \(Ax=b\) 的被选解在下游可观测，\(\tau\)/\(t\) 坐标的奇偶性会进入 `ext_block::same_sign`。因此，移植目标是复现上游 C++ 的确切幺模操作序列及其选出的解，而不仅是找到任意整数解。^[matreduc.md:20-26]

## 正除数下的下取整除法

`divide(a, b)` 的前提是除数 \(b>0\)。当 \(a\ge 0\) 时直接使用整数除法；当 \(a<0\) 时使用 `-1 - ((-1 - a) / b)`，得到向下取整的商，同时避开直接对 `i32::MIN` 取负。^[matreduc.md:37-38]

模块采用 wrapping `i32` 算术。来源将这一选择描述为对上游 C++ `int` 行为的镜像，包括可能影响被选解的溢出行为域；这一算术约定也是逐操作兼容要求的一部分。^[matreduc.md:20-26]

## gcd 主元选择与符号记录

`gcd(row, &mut flip, dest)` 使用 `wrapping_abs` 选择最小绝对值主元，并通过 `divide` 进行消元。负主元取正时，算法翻转 `flip`，同时在记录矩阵中执行 `col.set(mindex, mindex, -1)`；末尾将主元列交换到 `dest` 时，也翻转 `flip`。这些变化记录了相应操作的行列式符号。^[matreduc.md:39-41]

这些局部操作用于 `diagonalise(m)`。该函数返回 `(row, col, diagonal)`，其中 `row`、`col` 为幺模矩阵，使 `row * m * col` 对角化；对角元素除第一个外均为正。对于 \(0\times n\) 或 \(n\times0\) 的空形状，函数提前返回。^[matreduc.md:30-33]

## 与整体对角化的衔接

`flip` 的合并方式具有明确的操作顺序：每列首个 gcd 的 `flip` **覆盖赋值**给 `row_minus`；内层交替进行行、列 gcd，将 `flip` 分别异或累积到 `col_minus`、`row_minus`。退出循环后还执行 `row_minus ^= flip`：从行 gcd 的 break 退出时，该 `flip` 计入两侧；从列 gcd 的 break 退出时，净效果抵消。^[matreduc.md:42-45]

主元列未左对齐时，算法通过稳定排列 `pull_back_columns` 调整列位置，并按逆序对奇偶性计入置换符号。最后，若 `row_minus != col_minus`，则将 `diagonal[0]` 取负，再分别按标志对第 0 行或列乘以 \(-1\)。完整背景见 [[对角化的行列式符号簿记]]。^[matreduc.md:45-49]

## 求解接口与失败行为

`has_solution(a, b)` 先对角化，将 `b` 左乘 `row`，再逐坐标检查可除性。`find_solution(a, b)` 返回一个整数解，无整数解时返回 `None`；上游在此情形抛出异常，而 Rust 调用方预期已先运行 `has_solution`。^[matreduc.md:56-59]

长度不匹配与无整数解属于不同失败情形。`apply_to`、`right_prod` 等操作通过断言检查长度，因此 `has_solution` 等函数接收长度不匹配的 `b` 时会 panic，而非返回 `Err`。^[matreduc.md:50-52]

## 测试与证据边界

来源列出的测试锚点包括 11 个对角化重构用例，检查 `|det(row)| == 1`、`det(col) == 1`、逐对角结果及首项外为正；求解测试覆盖满秩、秩亏和矩形系统。源码注释对行列式归一化的口径存在差异，来源明确以测试口径为准：`det(col) == 1`，而 `det(row)` 可以为 \(-1\)。^[matreduc.md:48-49, matreduc.md:69-71]

`oracle_reference_cases` 固定了来自 C++ oracle 的精确结果，包括矩阵 \(\begin{pmatrix}0&5\\0&0\end{pmatrix}\) 对应 `diagonal = [-5]` 及精确的 `row`、`col`，以及一个主元列未左对齐的 \(6\times6\) 秩亏案例。后者记录完整变换矩阵和被选解 `[6,14,5,-37,-421,345]`。^[matreduc.md:71-74]

列出的测试未覆盖 wrapping 溢出域、空形状对角化，以及 `from_entries`、`apply_to` 的 panic 路径。来源属于结构性阅读，未执行构建、测试或原版运行；模块文档记载的编译版 oracle 逐位验证属于其自身的 HPC 证据链，本包未重跑，也不提供数学验收或性能结论。^[matreduc.md:9-16, matreduc.md:75-76, matreduc.md:89-89]

## Sources

- [matreduc.md](../../sources/matreduc.md) — 精确整数矩阵约化：matreduc 的逐操作移植。

---
title: 整数消元中的下取整除法与 gcd 主元选择
summary: divide 对正除数实施下取整并避开 i32::MIN 取负，gcd 使用 wrapping_abs 选择最小绝对值主元并记录取正与列交换的符号变化。
sources:
  - matreduc.md
kind: concept
createdAt: "2026-10-09T15:00:21.647Z"
updatedAt: "2026-10-09T21:01:54.706Z"
tags:
  - 整数线性代数
  - 消元算法
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
summary: divide 对正除数实现下取整并避免直接取负 i32::MIN；gcd 使用 wrapping_abs 选择主元，并记录主元取正和列交换引起的符号变化。
sources:
  - matreduc.md
kind: concept
tags:
  - 整数消元
  - 整数算术
---

# 整数消元中的下取整除法与 gcd 主元选择

`matreduc` 通过下取整除法、主元选择及符号记录，复现上游 C++ 的确切幺模操作序列。其动机是欠定系统 \(Ax=b\) 的被选解在下游可观测：\(\tau\)/\(t\) 坐标的奇偶性会进入 `ext_block::same_sign`，因此移植需要保留上游选出的那个解。相关背景见 [[整数矩阵算法的逐操作保真移植]]。^[matreduc.md:20-26]

## 正除数下的下取整除法

`divide(a, b)` 在正除数 \(b>0\) 下实现下取整除法。当 \(a\ge 0\) 时直接使用整数除法；当 \(a<0\) 时使用 `-1 - ((-1 - a) / b)`。负数分支避免直接对 `i32::MIN` 取负，同时得到向下取整的商。^[matreduc.md:37-38]

模块采用 wrapping `i32` 算术；来源将其描述为镜像上游 C++ `int` 的运算行为，包括在被选解中可观测的溢出行为域。解释除法与消元规则时，应保留这一算术契约。^[matreduc.md:20-26]

## gcd 主元选择与符号记录

`gcd(row, &mut flip, dest)` 使用 `wrapping_abs` 选择最小绝对值主元，并调用 `divide` 进行消元。负主元取正时，算法翻转 `flip`，同时在记录矩阵中执行 `col.set(mindex, mindex, -1)`；末尾将主元列交换到 `dest` 时，也翻转 `flip`，记录相应的行列式符号变化。^[matreduc.md:39-41]

这些局部操作服务于 [[整数矩阵的幺模对角化]]。`diagonalise(m)` 返回 `(row, col, diagonal)`，其中 `row`、`col` 为幺模矩阵，使 `row * m * col` 对角化，对角元素除第一个外均为正；空形状矩阵提前返回。^[matreduc.md:30-33]

`flip` 在整体对角化中的合并方式必须逐操作保留：每列首个 gcd 的 `flip` **覆盖赋值**给 `row_minus`；内层交替的行、列 gcd 则分别将 `flip` 异或累积到 `col_minus`、`row_minus`。退出循环后还有一次 `row_minus ^= flip`，其净效果取决于退出路径；完整流程见 [[对角化的行列式符号簿记]]。^[matreduc.md:42-49]

## 与整数求解的衔接

[[整数线性系统求解与像判定]] 中的 `has_solution(a, b)` 先对角化，再将 `b` 左乘 `row`，逐坐标检查可除性。`find_solution(a, b)` 返回一个解，无整数解时返回 `None`；调用方预期已先运行 `has_solution`。除法规则、主元选择与幺模操作序列共同落实保留上游被选解的移植要求。^[matreduc.md:20-26, matreduc.md:56-59]

长度不匹配不属于“无整数解”的返回分支：`apply_to` 等操作通过断言检查长度，因此 `has_solution` 等函数接收长度不匹配的 `b` 时会 panic，而非返回 `Err`。^[matreduc.md:50-52]

## 测试与证据边界

来源列出的测试包含 11 个对角化重构用例，检查 `|det(row)| == 1`、`det(col) == 1`、逐对角结果及首项外为正；求解测试覆盖满秩、秩亏和矩形系统。`oracle_reference_cases` 固定了 C++ oracle 的精确结果，例如矩阵 \(\begin{pmatrix}0&5\\0&0\end{pmatrix}\) 对应 `diagonal = [-5]` 及精确的 `row`、`col`，另有主元列未左对齐的 \(6\times6\) 秩亏案例。^[matreduc.md:69-74]

这些测试锚点未覆盖 wrapping 溢出域、空形状对角化，以及 `from_entries`、`apply_to` 的 panic 路径。来源包属于结构性阅读，未执行构建、测试或原版运行；模块文档所述与编译版 oracle 的逐位验证属于模块自身的独立 HPC 证据链，本包未重跑，也不提供数学验收或性能结论。^[matreduc.md:9-16, matreduc.md:75-76, matreduc.md:89-89]

## Sources

- [matreduc.md](../../sources/matreduc.md) — 精确整数矩阵约化：matreduc 的逐操作移植

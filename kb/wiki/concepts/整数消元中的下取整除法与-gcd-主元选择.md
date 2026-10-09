---
title: 整数消元中的下取整除法与 gcd 主元选择
summary: divide 对正除数实现下取整并避免直接取负 i32::MIN；gcd 采用 wrapping_abs 选主元，并记录主元取正和列交换的符号变化。
sources:
  - matreduc.md
kind: concept
createdAt: "2026-10-09T15:00:21.647Z"
updatedAt: "2026-10-09T19:33:05.030Z"
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
---

# 整数消元中的下取整除法与 gcd 主元选择

`matreduc` 的整数消元通过下取整除法、最小绝对值主元选择及符号记录，复现上游 C++ 的确切幺模操作序列。这属于 [[整数矩阵算法的逐操作保真移植]]：欠定系统 $Ax=b$ 的被选解在下游可观测，$\tau$/$t$ 坐标的奇偶性会进入 `ext_block::same_sign`，因此移植要求保留上游选出的解。^[matreduc.md:20-26]

## 正除数下的下取整除法

`divide(a, b)` 的前提是除数 $b>0$。当 $a\ge 0$ 时直接进行整数除法；当 $a<0$ 时使用 `-1 - ((-1 - a) / b)`，实现下取整商并避免直接对 `i32::MIN` 取负。^[matreduc.md:37-38]

$$
\operatorname{divide}(a,b)=
\begin{cases}
a/b, & a\ge 0,\\
-1-\bigl((-1-a)/b\bigr), & a<0.
\end{cases}
$$

该辅助函数用于 `gcd` 消元。模块算术采用 wrapping `i32`，以镜像上游 C++ `int` 的运算行为；来源明确将被选解中可观测的溢出行为域纳入移植契约。^[matreduc.md:20-26, matreduc.md:37-41]

## gcd 主元选择与符号记录

`gcd(row, &mut flip, dest)` 使用 `wrapping_abs` 选择最小绝对值主元，并通过 `divide` 消元。负主元取正时，算法翻转 `flip`，同时在记录矩阵中执行 `col.set(mindex, mindex, -1)`；末尾将主元列交换到 `dest` 时，也会翻转 `flip`。这些记录保留了取负与列交换引起的行列式符号变化。^[matreduc.md:39-41]

这些局部操作服务于 [[整数矩阵的幺模对角化]]。`diagonalise(m)` 返回 `(row, col, diagonal)`，其中 `row`、`col` 为幺模矩阵，使 `row * m * col` 对角化，且对角元素除第一个外均为正。^[matreduc.md:30-33]

`gcd` 的局部符号还要按对角化流程汇入整体簿记：每列首个 gcd 的 `flip` 覆盖赋值给 `row_minus`，内层交替的行、列 gcd 则分别异或累积到相应标志。具体退出路径和最终归一化见 [[对角化的行列式符号簿记]]。^[matreduc.md:42-49]

## 与整数求解的衔接

在 [[整数线性系统求解与像判定]] 中，`has_solution(a, b)` 先对角化，再将 `b` 左乘 `row`，逐坐标检查可除性；`find_solution(a, b)` 返回一个解，无整数解时返回 `None`。除法规则和主元选择所确定的幺模操作序列，是保留上游被选解的实现基础。^[matreduc.md:20-26, matreduc.md:56-59]

## 测试与证据边界

来源列出的测试包括 11 个对角化重构用例，检查 `|det(row)| == 1`、`det(col) == 1`、逐对角结果及首项外为正；求解测试覆盖满秩、秩亏和矩形系统。`oracle_reference_cases` 固定了 C++ oracle 的精确结果，包括 `[[0,5],[0,0]]` 的对角结果 `[-5]` 及对应变换矩阵，以及一个主元列未左对齐的 6×6 秩亏案例。^[matreduc.md:69-74]

这些测试锚点未覆盖 wrapping 溢出域、空形状对角化以及部分断言失败路径。来源包本身仅作结构性阅读，未执行构建、测试或原版运行；模块文档所述与编译版 oracle 的逐位验证属于其独立的 [[HPC 验收证据链]]。^[matreduc.md:9-16, matreduc.md:75-76, matreduc.md:89-89]

## Sources

- [matreduc.md](matreduc.md) — 精确整数矩阵约化：matreduc 的逐操作移植

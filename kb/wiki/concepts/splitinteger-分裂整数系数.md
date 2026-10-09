---
title: SplitInteger 分裂整数系数
summary: SplitInteger 以两个 i32 表示满足 s²=1 的 a+b·s，采用 wrapping 算术，并以 Split(c,−c) 表示 c(1−s)。
sources:
  - deformation-drivers.md
kind: concept
createdAt: "2026-10-09T14:44:24.111Z"
updatedAt: "2026-10-09T20:49:52.371Z"
tags:
  - 系数代数
  - Rust实现
aliases:
  - splitinteger-分裂整数系数
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: SplitInteger 分裂整数系数
summary: SplitInteger 用两个 i32 表示满足 s²=1 的 a+bs，以 wrapping 算术实现分裂乘法及乘以 1−s 等操作，承载形变多项式系数。
sources:
  - deformation-drivers.md
kind: concept
tags:
  - 系数代数
  - Rust 类型
  - 环绕算术
aliases:
  - splitinteger-分裂整数系数
---

# SplitInteger 分裂整数系数

`SplitInteger { a, b }` 表示分裂整数 \(a+bs\)，其中两个字段均为 `i32`，乘法满足 \(s^2=1\)。它是形变公式 \(F(z)=L(z)+(1-s)D(z)\) 中承载 \(s\)-系数的类型。^[deformation-drivers.md:50-59]

## 算术规则

`add_int` 将整数加到常数部分；`times_s` 交换两个分量；`times_1_s` 计算与 \(1-s\) 的乘积，对应
\[
(a+bs)s=b+as,\qquad
(a+bs)(1-s)=(a-b)+(b-a)s.
\]
`negate` 与 `mul_int` 分别逐分量取负和乘以整数。这些操作与上游 `arithmetic.h` 的运算对应。^[deformation-drivers.md:54-59]

类型实现 `Add` 和 `Mul` trait，其中分裂乘法为
\[
(a+bs)(c+ds)=(ac+bd)+(ad+bc)s.
\]
此外，它支持与 `(i32, i32)` 之间的双向 `From` 转换。^[deformation-drivers.md:57-59]

所有算术均使用 `wrapping_*`，沿用上游的环绕语义；溢出防护不属于该类型的职责。因此，上述代数公式的实现受 `i32` 固定宽度环绕运算约束。^[deformation-drivers.md:52-59]

## 在形变计算中的作用

`Split(c, -c)` 表示 \(c(1-s)\)。`twisted_deformation_terms` 返回 `(StandardRepr, int)` 形变项后，wrapper 将每个整数系数 \(c\) 映射为 `Split(c, -c)`，再按 `SR_poly` 顺序排序。相关项的提取见 [[twisted 与 common-block 形变项提取]]。^[deformation-drivers.md:59-59, deformation-drivers.md:107-111]

块形变驱动 `block_deformation_to_height` 返回 `(StandardRepr, SplitInteger)` 项，按 downward（reversed）block order 排列，并另外返回与 `accumulator` 平行的已消费项 flags。^[deformation-drivers.md:115-120]

递归 `twisted_deformation` 返回 `(KType, SplitInteger)` 项，并另外报告 shrink-wrap 到最后一个 reducibility point 时记录的 net flip；没有 shrink-wrap 时，`flip == false`。其递归与取消行为见 [[递归 twisted deformation 与取消语义]]。^[deformation-drivers.md:124-134]

## 证据边界

本页依据 `deform.rs` 的结构性阅读；来源中的 `SplitInteger` 算术说明已由维护者逐条对照源码核对。来源未执行构建、测试或原版运行，不提供数学验收、性能或并行结论。形变计算的正确性属于独立的 HPC 证据链，本来源不重述或扩展其结论。^[deformation-drivers.md:9-16, deformation-drivers.md:146-151]

## Sources

- [deformation-drivers.md](../../sources/deformation-drivers.md) — 形变驱动：twisted 与 block 形变。

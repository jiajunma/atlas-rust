---
title: SplitInteger 分裂整数系数
summary: SplitInteger 用两个 i32 表示满足 s²=1 的 a+b·s，以 wrapping 算术实现分裂乘法及乘以 1−s 等操作，承载形变多项式系数。
sources:
  - deformation-drivers.md
kind: concept
createdAt: "2026-10-09T14:44:24.111Z"
updatedAt: "2026-10-09T14:44:24.111Z"
tags:
  - 系数代数
  - Rust 类型
  - 环绕算术
aliases:
  - splitinteger-分裂整数系数
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# SplitInteger 分裂整数系数

`SplitInteger { a, b }` 表示分裂整数 \(a+bs\)，是形变公式 \(F(z)=L(z)+(1-s)D(z)\) 中承载 \(s\)-系数的类型。两个字段均为 `i32`；乘法规则体现 \(s^2=1\)。^[deformation-drivers.md:50-59]

## 算术规则

`SplitInteger` 的运算与上游 `arithmetic.h` 对应：`add_int` 将整数加到常数部分；`times_s` 交换两个分量；`times_1_s` 计算与 \(1-s\) 的乘积；`negate` 与 `mul_int` 分别逐分量取负和乘以整数。具体关系为
\[
(a+bs)s=b+as,\qquad
(a+bs)(1-s)=(a-b)+(b-a)s.
\]
类型还实现 `Add`、`Mul`，其中分裂乘法为
\[
(a+bs)(c+ds)=(ac+bd)+(ad+bc)s,
\]
并支持与 `(i32, i32)` 之间的双向 `From` 转换。^[deformation-drivers.md:52-59]

所有算术使用 `wrapping_*`，沿用上游的环绕语义；溢出防护不属于该类型的职责。因此，理解这些代数公式时，必须同时保留其 `i32` 环绕运算的实现约束。^[deformation-drivers.md:53-54]

## 在形变计算中的作用

记号 `Split(c, -c)` 表示 \(c(1-s)\)。`twisted_deformation_terms` 返回 `(StandardRepr, int)` 形变项后，wrapper 将每个整数系数 \(c\) 映射为 `Split(c, -c)`，再按 `SR_poly` 顺序排序。这使整数形变项进入带 \(1-s\) 因子的系数表示；相关项的提取见 [[twisted 与 common-block 形变项提取]]。^[deformation-drivers.md:59-59, deformation-drivers.md:107-111]

块形变驱动 `block_deformation_to_height` 返回 `(StandardRepr, SplitInteger)` 项；[[扭曲全形变计算流程]]中的递归 `twisted_deformation` 则返回 `(KType, SplitInteger)` 项，并另外报告 shrink-wrap 所记录的 net flip。^[deformation-drivers.md:115-120, deformation-drivers.md:124-130]

## 证据边界

本页依据对 `deform.rs` 的结构性阅读；来源中的 `SplitInteger` 算术说明已由维护者对照源码核对。该来源未执行构建、测试或原版运行，不提供数学验收、性能或并行结论；形变计算的正确性仍属于其独立的 [[HPC 验收证据链]]。^[deformation-drivers.md:9-16, deformation-drivers.md:146-151]

## Sources

- [deformation-drivers.md](deformation-drivers.md) — 形变驱动：twisted 与 block 形变。

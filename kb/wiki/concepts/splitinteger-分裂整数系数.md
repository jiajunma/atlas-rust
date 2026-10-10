---
title: SplitInteger 分裂整数系数
summary: SplitInteger 以两个 i32 表示 a+b·s（s²=1），采用 wrapping 算术，并以 Split(c,−c) 表示形变系数 c(1−s)。
sources:
  - deformation-drivers.md
kind: concept
createdAt: "2026-10-09T14:44:24.111Z"
updatedAt: "2026-10-10T00:29:59.979Z"
tags:
  - 形变
  - 系数算术
aliases:
  - splitinteger-分裂整数系数
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: SplitInteger 分裂整数系数
summary: SplitInteger 用两个 i32 表示 a+bs，采用 wrapping 算术实现分裂乘法，并以 Split(c,−c) 承载形变系数 c(1−s)。
sources:
  - deformation-drivers.md
kind: concept
tags:
  - 系数算术
  - 形变计算
aliases:
  - splitinteger-分裂整数系数
provenanceState: extracted
---

# SplitInteger 分裂整数系数

`SplitInteger { a, b }` 表示分裂整数 \(a+bs\)，两个字段均为 `i32`，乘法规则满足 \(s^2=1\)。它是形变公式 \(F(z)=L(z)+(1-s)D(z)\) 中承载 \(s\)-系数的类型。^[deformation-drivers.md:50-59]

## 算术规则

`add_int` 将整数加到常数部分；`times_s` 交换两个分量，即 \((a+bs)s=b+as\)；`times_1_s` 计算 \((a+bs)(1-s)=(a-b)+(b-a)s\)。`negate` 与 `mul_int` 分别逐分量取负和乘以整数。这些操作对应上游 `arithmetic.h` 的运算。^[deformation-drivers.md:54-59]

类型实现 `Add` 和 `Mul` trait，其中分裂乘法为 \((a+bs)(c+ds)=(ac+bd)+(ad+bc)s\)。此外，它支持与 `(i32, i32)` 之间的双向 `From` 转换。^[deformation-drivers.md:57-59]

所有算术均使用 `wrapping_*`，沿用上游的环绕语义；溢出防护不属于该类型的职责。因此，实现中的系数运算受 `i32` 固定宽度环绕语义约束。^[deformation-drivers.md:52-59]

## 在形变计算中的作用

`Split(c, -c)` 表示 \(c(1-s)\)。`twisted_deformation_terms` 为 final、delta-fixed 的父块元素返回 `(StandardRepr, int)` 形变项，顺序为 reverse-accumulated finals；wrapper 将每个整数系数 \(c\) 映射为 `Split(c, -c)`，再按 `SR_poly` 顺序排序。^[deformation-drivers.md:59-59, deformation-drivers.md:107-111]

`block_deformation_to_height` 返回 full block 中 height 不超过 `height_bound` 的 `(StandardRepr, SplitInteger)` 项，按 downward（reversed）block order 排列，并另外返回与 `accumulator` 平行的已消费项 flags。`u32::MAX` 对应上游负界表示的 maximal level。^[deformation-drivers.md:115-120]

[[扭曲全形变计算流程|递归 twisted deformation]] 输出 `(KType, SplitInteger)` 项，并另外报告 shrink-wrap 到最后一个 reducibility point 时记录的 net flip；没有 shrink-wrap 时，`flip == false`。可取消变体在递归或块级操作之间检查取消 probe，取消时返回 `Ok(None)`，不发布部分多项式。^[deformation-drivers.md:124-134]

## 证据边界

本页依据 `deform.rs` 的结构性阅读，所读字节来自来源记录的 dirty 工作区快照；`SplitInteger` 算术说明已由维护者对照源码核对。来源未执行构建、测试或原版运行，不提供数学验收、性能或并行结论。形变计算的正确性属于独立的 [[HPC 验收证据链]]，本来源不重述或扩展其结论。^[deformation-drivers.md:9-16, deformation-drivers.md:146-151]

上游 `arithmetic.h` 等文件的位置与对应关系转述自源码注释，来源未独立重读上游文件，行号可能随版本演进而漂移。相关移植约定见 [[形变驱动的冻结移植契约]]。^[deformation-drivers.md:18-27, deformation-drivers.md:140-141]

## Sources

- [deformation-drivers.md](../../sources/deformation-drivers.md) — 形变驱动：twisted 与 block 形变。

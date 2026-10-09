---
title: 可约点与标准参数 final 化
summary: reducibility_points 返回按分子分母对升序排列的可约分数，finals_for 以栈驱动 dominant 检查及奇偶、长度下降，生成带整数系数的 final 参数。
sources:
  - rep-context.md
kind: concept
createdAt: "2026-10-09T15:08:54.760Z"
updatedAt: "2026-10-09T15:08:54.760Z"
tags:
  - 表示论
  - 可约性
  - 参数算法
aliases:
  - 可约点与标准参数-final-化
  - 可F化
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 可约点与标准参数 final 化

可约点查询与标准参数 final 化是 `RepContext` 参数层提供的两项操作：`reducibility_points(z)` 返回可约分数，`finals_for(z)` 则通过栈驱动的过程返回带整数系数的标准参数。两者均围绕 [[StandardRepr 标准表示参数]] 展开。^[rep-context.md:81-84]

## 参数与上下文

`StandardRepr` 包含 KGB 元素 `x`、打包挠部分 `y_bits`、无穷小特征 `gamma` 与派生高度 `height`。参数相等性比较前三者，不比较高度；对 undefined 参数的操作通过 `ensure_defined` 报出 `RepInvariantViolation`。^[rep-context.md:30-38]

[[RepContext 借用上下文与一致性约束|RepContext]] 借用 inner class、involution 表及对应实形式的 KGB 图，并保存根数据派生常量。构造时检查 inner class 以及表与图的 `Arc` 指针一致性，不一致时返回 `DatumMismatch`。^[rep-context.md:40-47]

## 可约点查询

`reducibility_points(z)` 按分子／分母对的升序返回可约分数。来源将该实现对应到上游 `repr.cpp:825-925`；此处应保留“分子／分母对”的排序描述，而不能据此认定输出按分数数值大小排序。^[rep-context.md:81-82]

## 标准参数 final 化

`finals_for(z)` 采用栈驱动的 final 化过程，结合 dominant 检查与奇偶／长度下降，返回 `(StandardRepr, i32)` 系数对。输出不仅包含标准参数，也保留每项的整数系数；相关主题见 [[finals_for 带号重数展开]]。^[rep-context.md:83-84]

参数层的奇偶谓词 `is_parity(s, x, lambda_rho, gamma)` 将生成元 `s` 在 `x` 处的 KGB 状态转运到父单根，比较 $\theta_1\lambda_\rho + 2\rho_{\text{non-real}}$ 与 $\langle\gamma,\alpha_s^\vee\rangle$ 的奇偶。来源将其标注为上游 `repr.cpp:249` 的补集。^[rep-context.md:71-73]

## 与形变项计算的关系

`deformation_terms(block, y, gamma, lambda_rho, kl_table)` 有两个明确的边界行为：当 `block.length(y) == 0` 时返回空；当奇异集为空时，每个元素均为 final，逆向累积的列表为 `[y, y-1, ..., 0]`。这些是形变项计算的约定，相关主题见 [[形变项计算的边界情形与输出顺序]]。^[rep-context.md:85-87]

## 证据范围

本页依据结构性源码阅读材料。该材料未执行构建、测试或原版运行，不提供数学验收、性能或并行结论；其中上游文件行号来自 Rust 源码注释，未独立重读上游，可能随版本变化而漂移。参数层正确性仍属于其自身的 [[HPC 验收证据链]]。^[rep-context.md:9-16, rep-context.md:93-99]

## Sources

- [rep-context.md](rep-context.md) — 表示参数上下文：StandardRepr 与 RepContext。

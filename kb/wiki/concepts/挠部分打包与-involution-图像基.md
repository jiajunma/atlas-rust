---
title: 挠部分打包与 involution 图像基
summary: y_pack 使用 M_real 坐标模 2 打包挠部分，y_lift 恢复 (1−theta)lambda_rho；精确图像基保存在 involution 表中并沿 cross-action BFS 传送。
sources:
  - rep-context.md
kind: concept
createdAt: "2026-10-09T15:08:42.145Z"
updatedAt: "2026-10-09T15:08:42.145Z"
tags:
  - 格计算
  - 挠部分
  - 对合
aliases:
  - 挠部分打包与-involution-图像基
  - 挠I图
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 挠部分打包与 involution 图像基

挠部分打包是 [[StandardRepr 标准表示参数]] 保存 `lambda_rho` 信息的机制：`y_pack` 将其在 involution 图像基上的 `M_real` 坐标模 2，得到 `ModTwoVector`；反向的 `y_lift` 从打包结果计算 $(1-\theta)\lambda_\rho$。这套表示依赖 involution 表中保存的精确图像基。^[rep-context.md:23-28, rep-context.md:61-63]

## 图像基的存储与传送

每个 involution 的 $(1-\theta)X^*$ 图像基对由 `lift_mat` 与 `M_real` 组成，存放于 involution 表的记录中，并沿 cross-action BFS 传送。[[RepContext 借用上下文与一致性约束|RepContext]] 只读这些数据；相关结构见 [[对合的 (1−θ)X* 图像基对]]。^[rep-context.md:23-26]

图像基的具体选择会影响当选的 `lambda-rho` 代表元。因此，阶梯归约逐步复刻上游 `matreduc::column_echelon` 及其 gcd sweep，以保留所需的精确图像基行为。^[rep-context.md:26-28]

## 打包、提升与参数重建

`StandardRepr` 用 `y_bits: ModTwoVector` 保存 `lambda` 的打包挠部分，对应上游的 `y()`。构造入口 `sr_gamma(x, lambda_rho, gamma)` 打包 `lambda_rho`，同时保存无穷小特征 `gamma` 与 $(1+\theta)\gamma$ 的 height。位向量表示可参见 [[F₂ 上的位打包向量（ModTwoVector）]]。^[rep-context.md:32-35, rep-context.md:51-52]

`y_pack` 取 `lambda_rho` 的 `M_real` 坐标并逐坐标模 2，生成图像基上的位向量；`y_lift` 则对打包挠部分计算 $(1-\theta)\lambda_\rho$。提升的结果不是完整的 `lambda_rho`，完整代表元还需结合 `gamma` 重建。^[rep-context.md:56-63]

`lambda_rho(z)` 的重建先将 `gamma - rho` 与其 $\theta$ 像相加，取整坐标，再逐坐标加上 `y_lift` 的挠提升，最后减半。若某个坐标和为奇数，则以 `"lambda-rho halving"` 报出 `RepInvariantViolation`。随后可由 $\lambda=\rho+\lambda_\rho$ 得到 `lambda(z)`。^[rep-context.md:56-60]

## 规范代表元与相等性

`lambda_unique`、`real_unique` 与 `gamma_lambda` 负责代表元归一化。其中，`lambda_unique` 使用欧几里得除法 `div_euclid(2)` 取半；若对负奇数采用向零截断的有符号除法，会选出同一陪集中的不同代表元，导致公式项无法合并。^[rep-context.md:64-67]

`StandardRepr` 的相等性比较 `x`、打包挠部分与 `gamma`，派生字段 `height` 不参与比较。因此，打包挠部分既是参数的存储内容，也是参数相等性判定的一部分。^[rep-context.md:32-38]

## 上下文一致性与证据边界

`RepContext` 借用 inner class、involution 表和该实形式的 KGB 图，要求它们来自图构建时的同一 substrate 三元组。构造器 `new` 在表的 inner class 不匹配，或表与图的 `Arc` 指针不一致时返回 `DatumMismatch`；`from_derived` 则通过 `debug_assert!` 复核三方 `Arc` 指针一致。^[rep-context.md:42-47]

本页依据结构性源码阅读材料。源材料中的上游行号转述自源码注释，未独立重读上游；材料也未执行构建、测试或原版运行，因此不构成数学验收、性能或并行结论。参数层的正确性仍属于其独立的 [[HPC 验收证据链]]。^[rep-context.md:9-16, rep-context.md:93-99]

## Sources

- [rep-context.md](rep-context.md) — 表示参数上下文：StandardRepr 与 RepContext

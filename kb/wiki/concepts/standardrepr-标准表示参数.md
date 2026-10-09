---
title: StandardRepr 标准表示参数
summary: 以 x、打包挠部分、无穷小特征 gamma 和派生 height 表示标准参数；相等性不比较 height，未定义参数的操作受不变量检查约束。
sources:
  - rep-context.md
kind: concept
createdAt: "2026-10-09T15:08:28.037Z"
updatedAt: "2026-10-09T15:08:28.037Z"
tags:
  - 表示论
  - 参数建模
  - Rust
aliases:
  - standardrepr-标准表示参数
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# StandardRepr 标准表示参数

`StandardRepr` 是表示参数层中的标准表示参数，对应上游 `repr.h` 的四元组 $(x,y,\gamma,\mathrm{height})$。它只能经 [[RepContext 借用上下文与一致性约束|RepContext]] 的 `sr_gamma` 或 `sr` 构造；该模块移植了上游 `gkmod/repr.cpp` 的参数层数学及 `structure/involutions.cpp` 的 `K_type` 归一化。^[rep-context.md:20-28]

## 字段与相等性

`StandardRepr` 保存以下字段：^[rep-context.md:32-38]

| 字段 | 类型 | 含义 |
| --- | --- | --- |
| `x` | `KgbId` | KGB 元素标识 |
| `y_bits` | `ModTwoVector` | `lambda` 的打包挠部分，对应上游 `y()` |
| `gamma` | `RationalWeight` | 经 gcd 归一化的无穷小特征 |
| `height` | `u32` | 派生高度 |

参数相等性只比较 `x`、打包挠部分与 `gamma`，派生字段 `height` 不参与比较。此外，只有 `UndefKGB` twist 携带 `undefined_print_weights` 缓存，使打印不依赖图索引；普通参数自行派生两个权重。对 undefined 参数的操作由 `ensure_defined` 报出 `RepInvariantViolation`。^[rep-context.md:32-38]

## 构造与 K 型转换

`sr_gamma(x, lambda_rho, gamma)` 将 `lambda_rho` 的挠部分打包，保存 `gamma`，并计算 $(1+\theta)\gamma$ 的 height。`sr(x, lambda_rho, nu)` 则先计算 `gamma`，再转入 `sr_gamma`。^[rep-context.md:51-54]

标准参数与 [[KType 表示参数与规范化构造|KType]] 之间通过两个入口转换：`sr_of_ktype` 取 $\nu=0$，将 K 型扩张为标准参数；`sr_k_of_standard` 从标准参数取得 K 型。^[rep-context.md:53-55]

## 挠部分与权重重建

挠部分的打包依赖每个 involution 记录中的 [[对合的 (1−θ)X* 图像基对|$(1-\theta)X^*$ 图像基对]] `lift_mat`、`M_real`。这些基沿 cross-action BFS 传送，`RepContext` 只读使用；选出的 `lambda-rho` 代表元依赖精确的图像基，因此阶梯归约逐步复刻上游 `column_echelon` 及其 gcd sweep。^[rep-context.md:23-28]

`y_pack` 取 `lambda_rho` 的 `M_real` 坐标模二，将结果保存为图像基上的 `ModTwoVector`。`y_lift` 则从打包挠部分计算 $(1-\theta)\lambda_\rho$，为权重重建提供挠提升。^[rep-context.md:61-63]

`lambda_rho(z)` 将 $\gamma-\rho$ 与其 $\theta$ 像相加，取整坐标后加上 `y_lift` 的挠提升，再逐坐标减半。若坐标和为奇数，则报出 `RepInvariantViolation`，错误上下文为 `"lambda-rho halving"`。^[rep-context.md:56-58]

其余权重由下式派生，其中 $\nu$ 是 $-\theta$-不动投影：
\[
\lambda(z)=\rho+\lambda_\rho(z),\qquad
\nu(z)=\frac{\gamma-\theta\gamma}{2}.
\]
^[rep-context.md:59-60]

## 代表元规范化

`lambda_unique`、`real_unique` 与 `gamma_lambda` 用于代表元归一化。`lambda_unique` 通过欧几里得除法 `div_euclid(2)` 取半；负奇数若改用向零截断的有符号除法，会选出同一陪集中的不同代表元，使公式项无法合并。^[rep-context.md:64-67]

`mod_reduce(z)` 将 $\gamma-\rho-\lambda_\rho$ 经 `real_unique` 归一化，返回 $(x,\gamma_\lambda)$，用于打印 wrapper 的种子计算。反向入口 `build_srm(x, gamma_lambda)` 在相应 involution 处执行 `real_unique` 并规范化。^[rep-context.md:77-80]

## 参数上的进一步运算

参数层还提供朝向数、不变性、可约点及 final 化操作。`orientation_number` 先执行 `made_dominant`，再利用实正根相关权重计算朝向数；`is_fixed` 与 `is_delta_fixed` 判断参数在相应 twist 下是否不变。`reducibility_points` 返回排序后的可约分数；[[finals_for 带号重数展开|finals_for]] 以栈驱动 dominant 检查及奇偶、长度下降步骤，输出 `(StandardRepr, i32)` 系数对。^[rep-context.md:74-84]

## 证据范围

本页依据对 `rep_context.rs` 的结构性阅读，所读字节属于 dirty 工作区，并记录于来源快照。来源未执行构建、测试或原版运行，因此不提供数学验收、性能或并行结论；其中上游位置来自源码注释，未独立重读上游，行号可能随版本变化。^[rep-context.md:9-16, rep-context.md:91-99]

## Sources

- [rep-context.md](rep-context.md) — 表示参数上下文：StandardRepr 与 RepContext

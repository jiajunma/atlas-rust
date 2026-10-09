---
title: StandardRepr 标准表示参数
summary: 以 x、打包挠部分、无穷小特征 gamma 和派生 height 表示参数；相等性忽略 height，未定义参数另存打印权重并通过 ensure_defined 限制操作。
sources:
  - rep-context.md
kind: concept
createdAt: "2026-10-09T15:08:28.037Z"
updatedAt: "2026-10-09T19:35:38.766Z"
tags:
  - 表示论
  - 参数模型
aliases:
  - standardrepr-标准表示参数
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# StandardRepr 标准表示参数

`StandardRepr` 是表示参数层中的标准表示参数，对应上游 `repr.h` 的四元组 $(x,y,\gamma,\mathrm{height})$，只能经 [[RepContext 借用上下文与一致性约束|RepContext]] 的 `sr_gamma` 或 `sr` 构造。所属模块移植上游 `gkmod/repr.cpp` 的参数层数学及 `structure/involutions.cpp` 的 `K_type` 归一化。^[rep-context.md:20-28]

## 字段与相等性

参数的四个主要字段如下，其中无穷小特征采用 gcd 归一化表示。^[rep-context.md:32-38]

| 字段 | 类型 | 含义 |
| --- | --- | --- |
| `x` | `KgbId` | KGB 元素标识 |
| `y_bits` | `ModTwoVector` | `lambda` 的打包挠部分，对应上游 `y()` |
| `gamma` | `RationalWeight` | 无穷小特征 |
| `height` | `u32` | 派生高度 |

相等性只比较 `x`、打包挠部分与 `gamma`，不比较派生的 `height`。此外，只有 `UndefKGB` twist 携带 `undefined_print_weights` 缓存，使打印不依赖图索引；普通参数自行派生两个权重。对未定义参数的操作经 `ensure_defined` 报出 `RepInvariantViolation`。^[rep-context.md:32-38]

## 构造与 K 型转换

`sr_gamma(x, lambda_rho, gamma)` 打包 `lambda_rho` 的挠部分，保存 `gamma`，并计算 $(1+\theta)\gamma$ 的 height。`sr(x, lambda_rho, nu)` 则先计算 `gamma`，再调用 `sr_gamma`。^[rep-context.md:51-54]

标准参数与 [[KType 表示参数与规范化构造|KType]] 之间有两个转换入口：`sr_of_ktype` 以 $\nu=0$ 将 K 型扩张为标准参数；`sr_k_of_standard` 从标准参数取得 K 型。^[rep-context.md:53-55]

## 挠部分与权重恢复

挠部分的表示依赖每个 involution 记录中的 [[对合的 (1−θ)X* 图像基对|$(1-\theta)X^*$ 图像基对]] `lift_mat`、`M_real`。这些基沿 cross-action BFS 传送，`RepContext` 只读使用。由于选出的 `lambda-rho` 代表元依赖精确图像基，阶梯归约逐步复刻上游 `column_echelon` 及其 gcd sweep。^[rep-context.md:23-28]

`y_pack` 将 `lambda_rho` 的 `M_real` 坐标模二，得到图像基上的 `ModTwoVector`；`y_lift` 从打包挠部分计算 $(1-\theta)\lambda_\rho$。这构成 [[挠部分打包与 involution 图像基]] 的参数层接口。^[rep-context.md:61-63]

`lambda_rho(z)` 将 $\gamma-\rho$ 与其 $\theta$ 像相加，取整坐标后，逐坐标加上 `y_lift` 的挠提升并除以二。若坐标和为奇数，则报出 `RepInvariantViolation`，错误上下文为 `"lambda-rho halving"`。其余权重为 $\lambda(z)=\rho+\lambda_\rho(z)$ 与 $\nu(z)=(\gamma-\theta\gamma)/2$，后者是 $-\theta$-不动投影。^[rep-context.md:56-60]

## 代表元归一化与约化参数

`lambda_unique`、`real_unique`、`gamma_lambda` 用于 [[表示参数代表元归一化]]。其中 `lambda_unique` 使用欧几里得除法 `div_euclid(2)` 取半；对负奇数使用向零截断的有符号除法，会选出同一陪集中的不同代表元，使公式项无法合并。^[rep-context.md:64-67]

`mod_reduce(z)` 将 $\gamma-\rho-\lambda_\rho$ 经 `real_unique` 归一化，返回 $(x,\gamma_\lambda)$，用于打印 wrapper 的种子计算。`build_srm(x, gamma_lambda)` 则使 `gamma_lambda` 在相应 involution 处满足 `real_unique` 并规范化；相关主题见 [[标准模参数的约化表示（StandardReprMod）]]。^[rep-context.md:77-80]

## 参数上的进一步运算

`is_parity` 将生成元在指定 KGB 元素处的状态转运到父单根，比较 $\theta_1\lambda_\rho+2\rho_{\text{non-real}}$ 与 $\langle\gamma,\alpha_s^\vee\rangle$ 的奇偶。`orientation_number` 先执行 `made_dominant`，再按实正根的 $2\rho_{\text{real}}$ 与 $\gamma-\rho+\rho_{\text{real}}$ 计算朝向数；`is_fixed` 和 `is_delta_fixed` 判断参数在相应 twist 下是否不变。^[rep-context.md:71-76]

`reducibility_points` 按分子／分母对升序返回可约分数。[[finals_for 带号重数展开|finals_for]] 以栈驱动 final 化，通过 dominant 检查及奇偶、长度下降步骤，返回 `(StandardRepr, i32)` 系数对。^[rep-context.md:81-84]

## 证据范围

来源属于对 `rep_context.rs` 的结构性阅读，记录的是 dirty 工作区中的源码字节。该材料未执行构建、测试或原版运行，不提供数学验收、性能或并行结论；参数层正确性属于其自身的 [[HPC 验收证据链]]。上游位置转述自源码注释，未独立重读上游，行号可能随版本演进而变化。^[rep-context.md:9-16, rep-context.md:91-99]

## Sources

- [rep-context.md](rep-context.md) — 表示参数上下文：StandardRepr 与 RepContext

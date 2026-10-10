---
title: 挠部分打包与 involution 图像基
summary: y_pack 将 lambda_rho 的 M_real 坐标模二打包，y_lift 恢复其 (1−theta) 像；精确图像基由对合表保存并沿 cross-action BFS 传送。
sources:
  - rep-context.md
kind: concept
createdAt: "2026-10-09T15:08:42.145Z"
updatedAt: "2026-10-10T00:49:02.369Z"
tags:
  - 表示参数
  - 整数格
  - 模二线性代数
aliases:
  - 挠部分打包与-involution-图像基
  - 挠I图
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: 挠部分打包与 involution 图像基
summary: y_pack 将 lambda_rho 的 M_real 坐标模二打包，y_lift 计算其 (1−θ) 像；involution 表保存的精确图像基决定参数代表元的重建与归一化。
sources:
  - rep-context.md
kind: concept
tags:
  - 整数格
  - 表示论
aliases:
  - 挠部分打包与-involution-图像基
  - 挠I图
provenanceState: extracted
---

# 挠部分打包与 involution 图像基

挠部分打包是 [[StandardRepr 标准表示参数]] 保存参数信息的机制。`y_pack` 将 `lambda_rho` 的 `M_real` 坐标模 2，得到图像基上的 `ModTwoVector`；`y_lift` 根据打包挠部分计算 $(1-\theta)\lambda_\rho$。两者依赖 involution 表保存的精确图像基。^[rep-context.md:23-28, rep-context.md:61-63]

## 图像基的存储与传送

每个 involution 的 $(1-\theta)X^*$ 图像基对由 `lift_mat` 与 `M_real` 组成，存放在 involution 表的记录中，并沿 cross-action BFS 传送。[[RepContext 借用上下文与一致性约束|RepContext]] 只读这些数据，相关结构见 [[对合的 (1−θ)X* 图像基对]]。^[rep-context.md:23-26]

图像基的具体选择影响当选的 `lambda-rho` 代表元。因此，阶梯归约逐步复刻上游 `matreduc::column_echelon` 及其 gcd sweep，以保留精确图像基所决定的代表元选择。^[rep-context.md:26-28]

## 打包与提升

`StandardRepr` 的 `y_bits: ModTwoVector` 字段保存 `lambda` 的打包挠部分，对应上游访问器 `y()`。构造入口 `sr_gamma(x, lambda_rho, gamma)` 打包 `lambda_rho` 的挠部分，同时保存无穷小特征 `gamma` 与 $(1+\theta)\gamma$ 的 height。位向量表示参见 [[F₂ 上的位打包向量（ModTwoVector）]]。^[rep-context.md:32-35, rep-context.md:51-52]

`y_pack` 取 `lambda_rho` 的 `M_real` 坐标并模 2；`y_lift` 对打包挠部分计算 $(1-\theta)\lambda_\rho$。完整的 `lambda_rho` 代表元需要结合 `gamma` 与挠提升重建。^[rep-context.md:56-63]

## 参数重建与整性检查

`lambda_rho(z)` 先将 `gamma - rho` 与其 $\theta$ 像相加，取整坐标，再逐坐标加上 `y_lift` 的挠提升，最后减半。若某个坐标和为奇数，则报出 `RepInvariantViolation`，诊断为 `"lambda-rho halving"`。重建后，`lambda(z)` 由 $\lambda=\rho+\lambda_\rho$ 得到。^[rep-context.md:56-60]

`sr(x, lambda_rho, nu)` 先计算 `gamma`，再调用 `sr_gamma`；反向恢复的连续参数为 $\nu=(\gamma-\theta\gamma)/2$，即 $-\theta$-不动投影。^[rep-context.md:53-60]

## 规范代表元与相等性

`lambda_unique`、`real_unique` 与 `gamma_lambda` 负责代表元归一化。其中，`lambda_unique` 使用欧几里得除法 `div_euclid(2)` 取半；对负奇数采用向零截断的有符号除法，会选出同一陪集中的不同代表元，使公式项无法合并。^[rep-context.md:64-67]

`StandardRepr` 的相等性比较 `x`、打包挠部分与 `gamma`，派生字段 `height` 不参与比较。打包挠部分因此也是参数相等性判定的组成部分。^[rep-context.md:32-38]

## 上下文一致性

`RepContext` 借用 inner class、involution 表及该实形式的 KGB 图，要求它们来自图构建时的同一 substrate 三元组。构造器 `new` 在表的 inner class 不匹配，或表与图的 `Arc` 指针不一致时返回 `DatumMismatch`；`from_derived` 用 `debug_assert!` 复核三方 `Arc` 指针一致。^[rep-context.md:42-47]

## 证据边界

本页依据结构性源码阅读材料，其快照对应 dirty 工作区字节。材料中的上游行号转述自源码注释，未独立重读上游；材料未执行构建、测试或原版运行，不包含数学验收、性能或并行结论。参数层正确性属于其自身的 HPC 证据链，本页不扩展其验收范围。^[rep-context.md:9-16, rep-context.md:93-99]

## Sources

- [rep-context.md](../../sources/rep-context.md) — 表示参数上下文：StandardRepr 与 RepContext。

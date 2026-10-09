---
title: RepContext 借用上下文与一致性约束
summary: 借用 inner class、involution 表与 KGB 图，共享根 datum 派生常量，并通过一致性检查确保上下文使用同一底层结构。
sources:
  - rep-context.md
kind: concept
createdAt: "2026-10-09T15:08:21.834Z"
updatedAt: "2026-10-09T15:08:21.834Z"
tags:
  - Rust
  - 上下文设计
  - 数据一致性
aliases:
  - repcontext-借用上下文与一致性约束
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# RepContext 借用上下文与一致性约束

`RepContext<'a>` 是表示参数层的借用视图：它借用 inner class、involution 表及对应实形式的 KGB 图，并要求三者与图构建时使用的底层上下文一致。它同时持有 `Arc<RepContextDerived>`，其中保存根数据派生常量 $2\rho$、$2\rho^\vee$ 和 $\rho$。^[rep-context.md:40-47]

## 构造时的一致性检查

`RepContext::new` 设置两道一致性检查：若 involution 表所属的 inner class 不同，返回 `DatumMismatch`；若表与图的 `Arc` 指针不同，也返回 `DatumMismatch`。这些检查将上下文的构造约束落实到所属 inner class 与共享对象身份。^[rep-context.md:42-47]

crate 内可见的 `from_derived` 使用 `debug_assert!` 复核三方 `Arc` 指针一致。它与 `new` 的检查机制不同：来源明确记载前者使用调试断言，后者在不一致时返回 `DatumMismatch`。^[rep-context.md:44-47]

## 对 involution 图像基的只读依赖

每个 involution 的 $(1-\theta)X^*$ 图像基对由 `lift_mat` 与 `M_real` 表示，存放在 involution 表记录中，并沿 cross-action BFS 传送；`RepContext` 只读取这些数据。相关结构见 [[对合的 (1−θ)X* 图像基对]]。^[rep-context.md:23-28]

图像基的具体选择会影响 `lambda-rho` 代表元，因此来源强调阶梯归约逐步复刻上游 `matreduc::column_echelon` 及其 gcd sweep。在参数的挠部分处理中，`y_pack` 将 `lambda_rho` 的 `M_real` 坐标模二打包为 `ModTwoVector`，而 `y_lift` 从打包挠部分计算 $(1-\theta)\lambda_\rho$。^[rep-context.md:26-28, rep-context.md:61-63]

## 表示参数的构造与恢复

[[StandardRepr 标准表示参数]] 只能经 `RepContext::sr_gamma` 或 `RepContext::sr` 构造。`sr_gamma(x, lambda_rho, gamma)` 打包 `lambda_rho` 的挠部分，存入 `gamma` 及由 $(1+\theta)\gamma$ 得到的 height；`sr(x, lambda_rho, nu)` 则先计算 `gamma`，再调用 `sr_gamma`。上下文还提供与 [[KType 表示参数与规范化构造]] 之间的转换入口。^[rep-context.md:20-23, rep-context.md:49-55]

恢复 `lambda_rho(z)` 时，上下文将 `gamma - rho` 与其 $\theta$ 像相加，取整坐标，再与 `y_lift` 的挠提升逐坐标相加并减半。若坐标和为奇数，则以 `"lambda-rho halving"` 报告 `RepInvariantViolation`。这项算术检查与构造时的对象一致性检查共同约束参数操作。^[rep-context.md:56-58]

代表元归一化还依赖精确的除法约定：`lambda_unique` 使用欧几里得除法 `div_euclid(2)`。对负奇数采用向零截断会选出同一陪集中的不同代表元，导致公式项无法合并。^[rep-context.md:64-67]

## 证据范围

本页依据的来源属于结构性源码阅读，记录的是 dirty 工作区中的源码字节。来源未执行构建、测试或原版运行，不提供数学验收、性能或并行结论；参数层的正确性仍属于其独立的 [[HPC 验收证据链]]。来源中的上游位置来自源码注释转述，未独立重读上游，可能随版本变化而漂移。^[rep-context.md:9-16, rep-context.md:89-99]

## Sources

- [rep-context.md](rep-context.md) — 表示参数上下文：StandardRepr 与 RepContext

---
title: RepContext 借用上下文与一致性约束
summary: RepContext 借用 inner class、对合表与 KGB 图，共享根数据派生常量，并检查 inner class 与表图 Arc 身份的一致性。
sources:
  - rep-context.md
kind: concept
createdAt: "2026-10-09T15:08:21.834Z"
updatedAt: "2026-10-10T00:48:51.348Z"
tags:
  - 表示参数
  - 上下文管理
aliases:
  - repcontext-借用上下文与一致性约束
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: RepContext 借用上下文与一致性约束
summary: RepContext 借用 inner class、involution 表与 KGB 图，共享根数据派生常量，并通过来源与 Arc 指针一致性检查约束上下文构造。
sources:
  - rep-context.md
kind: concept
tags:
  - Rust设计
  - 上下文管理
aliases:
  - repcontext-借用上下文与一致性约束
provenanceState: extracted
---

# RepContext 借用上下文与一致性约束

`RepContext<'a>` 是表示参数层的借用视图，借用 inner class、involution 表与对应实形式的 KGB 图；三者须与图构建时使用的底层结构一致。它同时持有 `Arc<RepContextDerived>`，共享根数据派生常量 $2\rho$、$2\rho^\vee$ 和 $\rho$。^[rep-context.md:40-47]

## 构造时的一致性检查

`RepContext::new` 设置两道检查：若 involution 表所属的 inner class 不同，返回 `DatumMismatch`；若表与图的 `Arc` 指针不同，同样返回 `DatumMismatch`。这些检查将共享对象的指针身份纳入一致性约束。^[rep-context.md:42-47]

crate 内可见的 `from_derived` 使用 `debug_assert!` 复核三方 `Arc` 指针一致。两种构造入口的检查机制应明确区分：`new` 对上述不一致返回错误，`from_derived` 使用调试断言。^[rep-context.md:44-47]

## 对 involution 图像基的只读依赖

每个 involution 的 $(1-\theta)X^*$ 图像基对由 `lift_mat` 与 `M_real` 表示，存于 involution 表记录中，并沿 cross-action BFS 传送；上下文只读取这些数据。相关结构见 [[对合的 (1−θ)X* 图像基对]]。^[rep-context.md:23-28]

当选的 `lambda-rho` 代表元依赖精确的图像基，因此阶梯归约逐步复刻上游 `matreduc::column_echelon` 及其 gcd sweep。挠部分处理中，`y_pack` 将 `lambda_rho` 的 `M_real` 坐标模二打包为 `ModTwoVector`，`y_lift` 则由打包挠部分计算 $(1-\theta)\lambda_\rho$。参见 [[挠部分打包与 involution 图像基]]。^[rep-context.md:26-28, rep-context.md:61-63]

## 表示参数的构造与恢复

[[StandardRepr 标准表示参数]] 只能经 `RepContext::sr_gamma` 或 `RepContext::sr` 构造。`sr_gamma(x, lambda_rho, gamma)` 打包 `lambda_rho` 的挠部分，存入 `gamma` 及由 $(1+\theta)\gamma$ 得到的 height；`sr(x, lambda_rho, nu)` 先计算 `gamma`，再转入 `sr_gamma`。^[rep-context.md:20-23, rep-context.md:51-54]

上下文还提供与 [[KType 表示参数与规范化构造]] 之间的转换：`sr_of_ktype` 以 $\nu=0$ 将 K-type 扩张为标准参数，`sr_k_of_standard` 则由标准参数得到 `KType`。^[rep-context.md:53-55]

恢复 `lambda_rho(z)` 时，上下文将 `gamma - rho` 与其 $\theta$ 像相加，取整坐标后，与 `y_lift` 的挠提升逐坐标相加并减半。若坐标和为奇数，则返回 `RepInvariantViolation`，错误标识为 `"lambda-rho halving"`。随后可恢复 $\lambda=\rho+\lambda_\rho$，以及 $\nu=(\gamma-\theta\gamma)/2$，后者是 $-\theta$-不动投影。^[rep-context.md:56-63]

`lambda_unique`、`real_unique` 和 `gamma_lambda` 用于代表元归一化。其中 `lambda_unique` 使用欧几里得除法 `div_euclid(2)` 取半；若对负奇数采用向零截断，会选出同一陪集中的不同代表元，使公式项无法合并。^[rep-context.md:64-67]

## 证据范围

本页依据 `rep_context.rs` 的结构性源码阅读，来源记录的字节属于 dirty 工作区，阅读快照为 `snapshots/2026-10-03-rep-context.json`。来源未执行构建、测试或原版运行，不提供数学验收、性能或并行结论；参数层正确性属于独立的 HPC 证据链，本页不扩展其结论。^[rep-context.md:9-16, rep-context.md:91-99]

来源中的上游行号转述自源码注释，未独立重读上游文件，可能随版本演进而漂移。^[rep-context.md:93-94]

## Sources

- [rep-context.md](../../sources/rep-context.md) — 表示参数上下文：StandardRepr 与 RepContext。

---
title: RepContext 借用上下文与一致性约束
summary: 借用 inner class、involution 表与 KGB 图，共享根数据派生常量，并通过 inner class 与 Arc 指针一致性检查约束资源来源。
sources:
  - rep-context.md
kind: concept
createdAt: "2026-10-09T15:08:21.834Z"
updatedAt: "2026-10-09T21:07:35.905Z"
tags:
  - Rust设计
  - 上下文管理
aliases:
  - repcontext-借用上下文与一致性约束
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: RepContext 借用上下文与一致性约束
summary: RepContext 借用 inner class、involution 表和 KGB 图，共享根数据派生常量，并以所属 inner class 与 Arc 指针身份约束资源一致性。
sources:
  - rep-context.md
kind: concept
createdAt: "2026-10-09T15:08:21.834Z"
updatedAt: "2026-10-09T19:35:50.281Z"
tags:
  - 表示论
  - 上下文管理
aliases:
  - repcontext-借用上下文与一致性约束
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# RepContext 借用上下文与一致性约束

`RepContext<'a>` 是表示参数层的借用视图，借用 inner class、involution 表及对应实形式的 KGB 图，要求三者与图构建时使用的底层结构一致。它同时持有 `Arc<RepContextDerived>`，共享根数据派生常量 $2\rho$、$2\rho^\vee$ 和 $\rho$。^[rep-context.md:40-47]

## 构造时的一致性检查

`RepContext::new` 设置两道一致性检查：若 involution 表所属的 inner class 不同，返回 `DatumMismatch`；若表与图的 `Arc` 指针不同，也返回 `DatumMismatch`。构造约束因此包含共享对象的指针身份。^[rep-context.md:42-47]

crate 内可见的 `from_derived` 使用 `debug_assert!` 复核三方 `Arc` 指针一致。它采用调试断言，而 `new` 在发现上述不一致时返回错误；两者的检查机制应予区分。^[rep-context.md:44-47]

## 对 involution 图像基的只读依赖

每个 involution 的 $(1-\theta)X^*$ 图像基对由 `lift_mat` 与 `M_real` 表示，存放在 involution 表记录中，并沿 cross-action BFS 传送；`RepContext` 只读取这些数据。相关结构见 [[对合的 (1−θ)X* 图像基对]]。^[rep-context.md:23-28]

当选的 `lambda-rho` 代表元依赖精确的图像基，因此阶梯归约逐步复刻上游 `matreduc::column_echelon` 及其 gcd sweep。挠部分处理中，`y_pack` 将 `lambda_rho` 的 `M_real` 坐标模二打包为 `ModTwoVector`，`y_lift` 则从打包挠部分计算 $(1-\theta)\lambda_\rho$。参见 [[挠部分打包与 involution 图像基]]。^[rep-context.md:26-28, rep-context.md:61-63]

## 表示参数的构造与恢复

[[StandardRepr 标准表示参数]] 只能经 `RepContext::sr_gamma` 或 `RepContext::sr` 构造。`sr_gamma(x, lambda_rho, gamma)` 打包 `lambda_rho` 的挠部分，存入 `gamma` 及由 $(1+\theta)\gamma$ 得到的 height；`sr(x, lambda_rho, nu)` 先计算 `gamma`，再转入 `sr_gamma`。^[rep-context.md:20-23, rep-context.md:51-54]

上下文还提供与 [[KType 表示参数与规范化构造]] 之间的转换：`sr_of_ktype` 以 $\nu=0$ 将 K-type 扩张为标准参数，`sr_k_of_standard` 则由标准参数得到 `KType`。^[rep-context.md:53-55]

恢复 `lambda_rho(z)` 时，上下文将 `gamma - rho` 与其 $\theta$ 像相加，取整坐标后，再与 `y_lift` 的挠提升逐坐标相加并减半。若坐标和为奇数，则返回 `RepInvariantViolation`，错误标识为 `"lambda-rho halving"`。随后可恢复 $\lambda=\rho+\lambda_\rho$，以及 $\nu=(\gamma-\theta\gamma)/2$。参见 [[表示参数构造与权重恢复]]。^[rep-context.md:56-63]

[[表示参数代表元归一化]] 由 `lambda_unique`、`real_unique` 和 `gamma_lambda` 等操作完成。其中 `lambda_unique` 使用欧几里得除法 `div_euclid(2)` 取半；若对负奇数采用向零截断，会选出同一陪集中的不同代表元，导致公式项无法合并。^[rep-context.md:64-67]

## 证据范围

本页依据 `rep_context.rs` 的结构性源码阅读，所记录的是 dirty 工作区中的源码字节，阅读快照为 `snapshots/2026-10-03-rep-context.json`。来源未执行构建、测试或原版运行，不提供数学验收、性能或并行结论；参数层正确性属于独立的 HPC 证据链，本页不扩展其结论。^[rep-context.md:9-16, rep-context.md:91-99]

来源中的上游行号转述自源码注释，未独立重读上游文件，可能随版本演进而漂移。^[rep-context.md:93-94]

## Sources

- [rep-context.md](../../sources/rep-context.md) — 表示参数上下文：StandardRepr 与 RepContext。

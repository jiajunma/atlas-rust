---
title: 扭对合与 Weyl 平移
summary: TwistedInvolution 保存 WeylAction 与根对合数据，依次校验 datum、秩及双矩阵复合后的对合性；分解和典范化由其他层负责。
sources:
  - twisted-involution-trio.md
kind: concept
createdAt: "2026-10-09T15:14:26.282Z"
updatedAt: "2026-10-09T22:51:36.152Z"
tags:
  - 根数据
  - 扭对合
  - Rust
aliases:
  - 扭对合与-weyl-平移
  - 扭W平
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: 扭对合与 Weyl 平移
summary: TwistedInvolution 保存 WeylAction 与根对合数据，按 datum、秩和矩阵复合顺序验证 (wθ)²=1，分解与典范化由其他层负责。
sources:
  - twisted-involution-trio.md
kind: concept
tags:
  - 根理论
  - Rust设计
  - 对合
aliases:
  - 扭对合与-weyl-平移
  - 扭W平
provenanceState: extracted
---

# 扭对合与 Weyl 平移

`TwistedInvolution` 表示杰出对合 \(\theta\) 的 Weyl 平移：给定 Weyl 作用 \(w\)，要求复合 \(w\theta\) 仍为对合，即 \((w\theta)^2=1\)。该类型只确立这一根论条件；[[扭曲对合的 Cayley/Cross 分解|Cayley/cross 分解]]由 `CayleyCrossDecomposition` 负责，典范化由 `InnerClass::canonicalize` 负责。^[twisted-involution-trio.md:15-20]

## 表示与存储

`TwistedInvolution` 包含两个私有字段：`weyl_action` 保存 `WeylAction`，`root_involution` 保存 `RootInvolutionData`。构造过程中使用的中间对象 `LatticeInvolution` 不保存在最终对象中。^[twisted-involution-trio.md:17-29]

## 构造与验证顺序

`new` 首先执行 datum 同一性的三项检查，涉及 `root_system`、`distinguished` 与 `weyl_action`；任一不符即返回 `DatumMismatch`。随后依次执行三个秩检查，不符则返回 `RankMismatch`。其中 `WeylAction` 使用 `rank()`，其余使用 `lattice_rank()`，错误中的 `actual` 分别记录实测值。^[twisted-involution-trio.md:22-25]

检查通过后，构造器调用 `compose_matrices`，分别复合 weight 与 coweight 矩阵，乘法顺序均为 **Weyl 矩阵 × distinguished 矩阵**。随后依次调用 `LatticeInvolution::new` 和 `RootInvolutionData::new`，由内部构造器检查对合性。`TwistedInvolution::new` 本身没有直接构造 `InvalidInvolution` 的位置；测试观测到的是内部错误的传播。^[twisted-involution-trio.md:25-29]

## 矩阵复合与算术边界

辅助函数 `compose_matrices` 具有 `pub(crate)` 可见性，被 `global_tits` 等模块复用，用于对合传输，可结合 [[全局 Tits 传输的上下文一致性校验]] 阅读。它采用 `i128` checked 累加，并通过 `i32::try_from` 收窄结果。形状不符时，`RankMismatch.actual` 恒报 `right.len()`，无论实际是哪一行长度不齐。^[twisted-involution-trio.md:31-33, twisted-involution-trio.md:79-80]

源材料将 `twisted_involution` 的算术策略概括为全 checked；同包的对合分类工具混用 checked、saturating 与普通算术，环境根反射工具则使用 wrapping 算术。材料未判定这些差异属于有意分层还是实现漂移。^[twisted-involution-trio.md:79-84]

## 测试与证据范围

源材料列出四个测试：A1 反射平移产生两个 Real 根；A2 的三阶元素 `s0·s1` 被拒绝，传播 `InvalidInvolution`；同秩但不同 datum 的 A2 与 B2 触发 `DatumMismatch`；带中心环面的用例覆盖 `diag(1,−1)` distinguished 复合。^[twisted-involution-trio.md:35-37]

上述记录来自结构性源码阅读，不构成数学验收。材料明确列出 `restricted_roots` 路径及 `compose_matrices` 直接测试的覆盖缺口；本次知识维护未执行 Atlas、Cargo、测试或 benchmark。精确读取身份由来源快照绑定 Git base、源文件字节 SHA-256 与起草调用记录。^[twisted-involution-trio.md:9-13, twisted-involution-trio.md:83-84, twisted-involution-trio.md:88-92]

## Sources

- [twisted-involution-trio.md](../../sources/twisted-involution-trio.md) — 扭对合、对合分类与环境根反射字（twisted_involution.rs / involution_classification.rs / root_reflection.rs）。

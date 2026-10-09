---
title: 扭对合与 Weyl 平移
summary: TwistedInvolution 表示满足 (wθ)²=1 的 Weyl 平移，按 datum 同一性、秩、矩阵复合及内部构造器验证顺序建立根论对合，不承担 Cayley/cross 分解或典范化。
sources:
  - twisted-involution-trio.md
kind: concept
createdAt: "2026-10-09T15:14:26.282Z"
updatedAt: "2026-10-09T15:14:26.282Z"
tags:
  - 根系
  - Weyl群
  - 扭对合
aliases:
  - 扭对合与-weyl-平移
  - 扭W平
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 扭对合与 Weyl 平移

`TwistedInvolution` 表示杰出对合 θ 的一个 Weyl 平移：给定 Weyl 作用 w，要求复合 wθ 仍为对合，即满足 `(wθ)² = 1`。该类型只确立这一根论条件；Cayley/cross 分解由 `CayleyCrossDecomposition` 负责，典范化由 `InnerClass::canonicalize` 负责。^[twisted-involution-trio.md:15-20]

## 表示与职责边界

`TwistedInvolution` 包含两个私有字段：`weyl_action` 和 `root_involution`，分别保存 `WeylAction` 与 `RootInvolutionData`。构造过程中使用的 `LatticeInvolution` 不会被保存。相关分解机制可参见 [[扭曲对合的 Cayley/Cross 分解]]。^[twisted-involution-trio.md:17-20, twisted-involution-trio.md:25-29]

## 构造与验证顺序

`new` 首先执行 datum 同一性的三项检查，涉及 `root_system`、`distinguished` 与 `weyl_action`；任一不符即返回 `DatumMismatch`。随后依次执行三个秩检查，不符则返回 `RankMismatch`。其中 `WeylAction` 使用 `rank()`，其余使用 `lattice_rank()`，错误中的 `actual` 分别记录实测值。^[twisted-involution-trio.md:22-25]

通过上述检查后，构造器分别复合 weight 与 coweight 矩阵，乘法顺序均为 **Weyl 矩阵 × distinguished 矩阵**，再依次调用 `LatticeInvolution::new` 和 `RootInvolutionData::new`。对合性拒绝发生在这两个内部构造器中；`TwistedInvolution::new` 本身没有直接构造 `InvalidInvolution` 的位置，而是传播内部错误。^[twisted-involution-trio.md:25-29]

## 矩阵复合与算术边界

矩阵复合辅助函数 `compose_matrices` 具有 `pub(crate)` 可见性，也被 `global_tits` 等模块复用。它使用 checked i128 累加，并通过 `i32::try_from` 收窄结果。形状不符时有一个诊断细节：`RankMismatch.actual` 总是报告 `right.len()`，无论实际是哪一行长度不齐。^[twisted-involution-trio.md:31-33]

这一复合函数也服务于对合传输，可结合 [[全局 Tits 传输的上下文一致性校验]] 阅读。源材料将 `twisted_involution` 的算术策略概括为全 checked，但没有据此判定其他相关模块采用不同算术策略的原因。^[twisted-involution-trio.md:79-84]

## 测试与证据范围

源材料列出四个测试：A1 反射平移产生两个 Real 根；A2 的三阶元素 `s0·s1` 被拒绝并传播 `InvalidInvolution`；同秩但不同 datum 的 A2 与 B2 触发 `DatumMismatch`；另一个测试覆盖带中心环面的 `diag(1,−1)` distinguished 复合。^[twisted-involution-trio.md:35-37]

这些记录来自源码的结构性阅读，不构成数学验收。源材料还明确列出 `restricted_roots` 与 `compose_matrices` 的直接测试缺口；本次知识维护未运行 Atlas、Cargo、测试或 benchmark。^[twisted-involution-trio.md:9-13, twisted-involution-trio.md:83-84, twisted-involution-trio.md:88-92]

## Sources

- [twisted-involution-trio.md](../../sources/twisted-involution-trio.md)

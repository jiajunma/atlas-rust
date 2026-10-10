---
title: 扭曲对合（TwistedInvolution）
summary: 将 Weyl 作用左乘 distinguished 对合后重新验证双格对合与根数据自同构，保存 Weyl 作用及根对合数据；分解与规范化由其他层承担。
sources:
  - involution-types.md
kind: concept
createdAt: "2026-10-09T14:53:47.508Z"
updatedAt: "2026-10-10T02:22:36.839Z"
tags:
  - Weyl群
  - 扭曲对合
  - 构造验证
aliases:
  - 扭曲对合twistedinvolution
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# 扭曲对合（TwistedInvolution）

`TwistedInvolution` 表示 distinguished 对合 \(\theta\) 经 Weyl 元 \(w\) 平移后得到的对合 \(w\theta\)。构造器验证 \((w\theta)^2=1\)，并验证合成作用在枚举根系上形成根置换、将每个余根运输到像根的余根。Atlas Cartan 类最终由 twisted 共轭轨道的 canonical 代表元编号；本类型负责建立其中的根论条件。^[involution-types.md:104-112]

## 类型结构与不变量

该类型建立在两层验证之上：[[格对合（LatticeInvolution）]] 分别存储权格与余权格作用，一起验证对合条件和配对保持；[[根对合数据（RootInvolutionData）]] 进一步验证根置换与余根运输。仅保持配对不足以保证根数据自同构，因为它可能容许固定所有根却移动余根中心环面坐标的作用。^[involution-types.md:63-65, involution-types.md:82-84]

`TwistedInvolution` 包含两个私有字段：`weyl_action` 和 `root_involution`。构造时传入的 distinguished 对合不被存储；Weyl 作用原样保留，通过 `weyl_action()` 暴露。`root_involution()` 返回合成作用的根对合数据；另有 `restricted_roots(&RootSystem)` 接口，返回 `Result<RestrictedRootSystem, _>`。^[involution-types.md:49-57, involution-types.md:108-112]

## 构造顺序与错误优先级

`new` 接收 `&BasedRootDatum`、`&RootSystem`、`&LatticeInvolution` 和 `WeylAction`。它先检查三个 datum 的一致性，任一不符即返回 `DatumMismatch`，优先于全部秩检查；随后按 `root_system`、`distinguished`、`weyl_action` 的顺序检查秩，再分别合成两格上的作用矩阵。乘法方向固定为 \(w\) 在左、\(\theta\) 在右。^[involution-types.md:52-53, involution-types.md:108-112]

合成结果完整重走 `LatticeInvolution::new` 的验证：两个矩阵须为格秩阶方阵，随后先验证权作用平方为单位矩阵，再验证余权作用平方为单位矩阵，最后验证配对保持条件 \(W^T C=I\)。形状或平方条件不满足时返回 `InvalidInvolution`，配对条件失败返回 `InvalidRootAutomorphism`；平方检验使用 `i128` 检查累加，溢出传播为 `ArithmeticOverflow`。^[involution-types.md:67-71, involution-types.md:110-112]

随后调用 `RootInvolutionData::new`，依次检查 datum、秩、单根像和逐根运输。单根级错误 `SimpleRootImageNotRoot`、`SimpleCorootImageMismatch` 先于主循环中的泛型错误；主循环中，像不是根返回 `InvalidRootAutomorphism`，余根运输不符返回 `InvalidRootDatumAutomorphism`。相关主题见 [[对合类型的分层构造验证与错误优先级]]。^[involution-types.md:86-92, involution-types.md:110-112]

内部辅助函数 `compose_matrices` 为 `pub(crate)`，采用标准三重循环，以 `i128` 检查算术并收窄至 `i32`。形状错误返回 `RankMismatch { expected: rank, actual: right.len() }`；即使实际问题是某行长度不符，`actual` 仍报告右矩阵的行数。详见 [[矩阵复合形状错误的诊断字段失真]]。^[involution-types.md:114-116]

## 根分类与职责边界

合成作用的根分类由 `RootInvolutionData` 提供：像等于自身为 `Imaginary`，否则像等于负根为 `Real`，其余为 `Complex`；负根比较使用逐坐标 `checked_neg`。根置换是否满足平方为恒等不另行独立检查，而是依赖 `LatticeInvolution` 已建立的代数对合条件。^[involution-types.md:91-92, involution-types.md:144-145]

[[扭曲对合的 Cayley/Cross 分解]] 由 `CayleyCrossDecomposition` 承担，规范化由 `InnerClass::canonicalize` 承担，均不属于本类型的职责。^[involution-types.md:104-106]

## 测试锚点与证据范围

来源记录的 twisted 测试包括：A1 的 `s0` 平移合法，得到两个实根；A2 的三阶元 `s0·s1` 返回 `InvalidInvolution`；来自不同但同秩 datum（A2 与 B2）的 Weyl 作用返回 `DatumMismatch`；带中心环面的 distinguished 对合 \(\operatorname{diag}(1,-1)\) 与 `s0` 合成合法。^[involution-types.md:130-132]

`compose_matrices` 的 `RankMismatch` 分支没有直接测试覆盖。来源指出，该分支在 `TwistedInvolution::new` 内受前置秩检查阻挡，但辅助函数仍具有 crate 内可见性。私有辅助函数的裸下标索引依赖调用点的前置方阵和秩检查；来源将其潜在 panic 路径标为阅读推断，并指出字段私有保证实例经过门控构造。^[involution-types.md:138-143]

本页依据结构性源码阅读，不构成对合层的数学验收。来源中的上游引用仅转录自代码文档注释，未核对上游字节；该次知识维护未执行 Atlas、Cargo、测试或 benchmark，因此上述测试锚点不是本次执行结果。^[involution-types.md:9-15, involution-types.md:149-156]

## Sources

- [involution-types.md](../../sources/involution-types.md) — 对合类型三件套：LatticeInvolution / RootInvolutionData / TwistedInvolution。

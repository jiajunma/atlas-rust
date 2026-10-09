---
title: 扭曲对合（TwistedInvolution）
summary: 以 Weyl 作用左乘 distinguished 对合构造 wθ，重新验证格对合及根数据自同构条件；分解与规范化由其他层承担。
sources:
  - involution-types.md
kind: concept
createdAt: "2026-10-09T14:53:47.508Z"
updatedAt: "2026-10-09T20:56:35.375Z"
tags:
  - 扭曲对合
  - Weyl群
  - Rust类型设计
aliases:
  - 扭曲对合twistedinvolution
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: 扭曲对合（TwistedInvolution）
summary: 以 w 左乘 distinguished 对合 θ 构造 wθ，重新验证格对合、根置换与余根运输；轨道规范化与 Cayley/cross 分解由其他层负责。
sources:
  - involution-types.md
kind: concept
tags:
  - 扭曲对合
  - Weyl群
  - 分层设计
aliases:
  - 扭曲对合twistedinvolution
---

# 扭曲对合（TwistedInvolution）

`TwistedInvolution` 表示 distinguished 对合 \(\theta\) 经 Weyl 元 \(w\) 平移后得到的对合 \(w\theta\)。它验证合成作用满足 \((w\theta)^2=1\)，并保持根置换及余根运输。Atlas Cartan 类最终通过 twisted 共轭轨道的 canonical 代表元编号，但本类型只建立相应的根论条件。^[involution-types.md:104-112]

## 类型结构与不变量

该类型建立在两层验证之上：[[格对合（LatticeInvolution）]] 同时验证权格、余权格上的对合及配对保持；[[根对合数据（RootInvolutionData）]] 进一步验证枚举根系上的根置换和余根运输。仅保持配对不足以保证根数据自同构，因为它可能容许固定所有根却移动余根中心环面坐标的作用。^[involution-types.md:63-65, involution-types.md:82-84]

`TwistedInvolution` 仅有两个私有字段：`weyl_action` 和 `root_involution`。构造时传入的 distinguished 对合不被存储；`WeylAction` 原样保留，可通过 `weyl_action()` 访问。`root_involution()` 返回合成作用的根对合数据，另有接收 `&RootSystem`、返回 `Result<RestrictedRootSystem, _>` 的 `restricted_roots()` 接口。^[involution-types.md:49-57, involution-types.md:108-112]

## 构造顺序与错误优先级

`new` 接收 `&BasedRootDatum`、`&RootSystem`、`&LatticeInvolution` 和 `WeylAction`。它首先检查三个 datum 的一致性，任一不符即返回 `DatumMismatch`，优先于全部秩检查；随后按 `root_system`、`distinguished`、`weyl_action` 的顺序检查秩，再分别合成权格和余权格上的矩阵。乘法方向固定为 \(w\) 在左、\(\theta\) 在右。^[involution-types.md:52-53, involution-types.md:108-112]

合成结果完整经过 `LatticeInvolution::new`：先确认两个矩阵都是格秩阶方阵，再依次检查权作用与余权作用的平方为单位矩阵，最后验证配对保持条件 \(W^T C=I\)。形状或平方条件失败返回 `InvalidInvolution`，配对条件失败返回 `InvalidRootAutomorphism`；检验使用 `i128` 检查算术，溢出传播为 `ArithmeticOverflow`。^[involution-types.md:67-71, involution-types.md:110-112]

随后，`RootInvolutionData::new` 按 datum、秩、单根像、逐根主循环的顺序验证。单根错误 `SimpleRootImageNotRoot` 和 `SimpleCorootImageMismatch` 先于主循环的泛型错误；主循环中，像不是根返回 `InvalidRootAutomorphism`，余根运输不符返回 `InvalidRootDatumAutomorphism`。这体现了[[对合类型的分层构造验证与错误优先级]]。^[involution-types.md:86-92, involution-types.md:110-112]

内部函数 `compose_matrices` 具有 `pub(crate)` 可见性，采用标准三重循环，以 `i128` 检查算术并收窄至 `i32`。形状不符时返回 `RankMismatch { expected: rank, actual: right.len() }`；即使实际问题是某行长度不符，`actual` 仍报告右矩阵的行数。^[involution-types.md:58-58, involution-types.md:114-116]

## 根分类与职责边界

合成作用的根分类由 `RootInvolutionData` 提供，判断顺序固定：像等于自身为 `Imaginary`，像等于负根为 `Real`，其余为 `Complex`。负根比较使用逐坐标 `checked_neg`，相关概念见[[对合下的虚根、实根与复根分类]]。^[involution-types.md:91-92]

[[扭曲对合的 Cayley/Cross 分解]] 由 `CayleyCrossDecomposition` 承担，规范化由 `InnerClass::canonicalize` 承担，均不属于本层。`RootInvolutionData::new` 也不独立检查根置换是否满足平方为恒等，而是依赖 `LatticeInvolution` 已建立的代数对合条件。^[involution-types.md:104-106, involution-types.md:144-145]

## 测试锚点与证据范围

来源记录了四类 twisted 测试：A1 的 `s0` 平移合法，得到两个实根；A2 的三阶元 `s0·s1` 返回 `InvalidInvolution`；来自不同但同秩 datum（A2 与 B2）的 Weyl 作用返回 `DatumMismatch`；带中心环面的 distinguished 对合 \(\operatorname{diag}(1,-1)\) 与 `s0` 合成合法。^[involution-types.md:130-132]

`compose_matrices` 的 `RankMismatch` 分支没有直接测试覆盖；来源指出，该分支在 `TwistedInvolution::new` 内受前置秩检查阻挡，但辅助函数本身仍具有 crate 内可见性。私有辅助函数的裸下标安全性依赖调用点的方阵和秩检查，这属于结构性阅读推断。^[involution-types.md:138-143]

本页依据结构性源码阅读，不构成对合层的数学验收。来源中的上游引用仅转录自代码文档注释，未核对上游字节；该次知识维护也未执行 Atlas、Cargo、测试或 benchmark。^[involution-types.md:9-15, involution-types.md:149-156]

## Sources

- [involution-types.md](../../sources/involution-types.md) — 对合类型三件套：LatticeInvolution / RootInvolutionData / TwistedInvolution。

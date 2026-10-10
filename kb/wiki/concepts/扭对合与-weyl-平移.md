---
title: 扭对合与 Weyl 平移
summary: TwistedInvolution 以 Weyl 作用左乘 distinguished 对合，并重新验证双格对合与根数据自同构条件；分解和规范化由其他层承担。
sources:
  - involution-types.md
  - twisted-involution-trio.md
kind: concept
createdAt: "2026-10-09T15:14:26.282Z"
updatedAt: "2026-10-10T00:37:14.687Z"
tags:
  - 对合
  - Weyl群
  - rust
aliases:
  - 扭对合与-weyl-平移
  - 扭W平
confidence: 1
provenanceState: merged
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: 扭对合与 Weyl 平移
summary: TwistedInvolution 保存 Weyl 作用及复合后的根对合数据，依次校验 datum、秩、双格对合性与根余根运输；分解和典范化由其他层负责。
sources:
  - involution-types.md
  - twisted-involution-trio.md
kind: concept
tags:
  - 根数据
  - 扭对合
  - Rust
aliases:
  - 扭对合与-weyl-平移
  - 扭W平
provenanceState: extracted
---

# 扭对合与 Weyl 平移

`TwistedInvolution` 表示杰出对合（distinguished involution）$\theta$ 的 Weyl 平移：给定 Weyl 作用 $w$，要求复合 $w\theta$ 仍为对合，即 $(w\theta)^2=1$。该类型建立这一根论条件；[[扭曲对合的 Cayley/Cross 分解|Cayley/cross 分解]]由 `CayleyCrossDecomposition` 负责，典范化由 `InnerClass::canonicalize` 负责。Atlas Cartan 类的编号依赖 twisted 共轭轨道的典范代表元，不由本类型单独完成。^[involution-types.md:102-106, twisted-involution-trio.md:15-20]

## 表示与接口

类型包含两个私有字段：`weyl_action` 原样保存输入的 `WeylAction`，`root_involution` 保存复合后的 `RootInvolutionData`，分别通过同名访问器借用。输入的 distinguished 对合不单独存储；复合后的 `LatticeInvolution` 则包含在 `RootInvolutionData` 内，并非被丢弃。类型还提供 `restricted_roots(&RootSystem)`，返回可失败的 `RestrictedRootSystem` 构造结果。^[involution-types.md:34-57, involution-types.md:108-112]

## 构造与验证顺序

`new` 接收 `BasedRootDatum`、`RootSystem`、distinguished `LatticeInvolution` 和 `WeylAction`。首先检查后三者与给定 datum 的一致性，任一不符即返回 `DatumMismatch`，优先于所有秩检查。随后按 `root_system`、`distinguished`、`weyl_action` 的顺序核对秩；前两者使用 `lattice_rank()`，Weyl 作用使用 `rank()`，`RankMismatch.actual` 记录对应的实测值。^[involution-types.md:49-53, involution-types.md:108-110, twisted-involution-trio.md:22-25]

检查通过后，分别复合权格与余权格作用矩阵，顺序均为 **Weyl 矩阵在左、distinguished 矩阵在右**。结果依次进入 `LatticeInvolution::new` 与 `RootInvolutionData::new`。`TwistedInvolution::new` 本体没有直接构造 `InvalidInvolution` 的位置；这类错误由内部构造器传播。^[involution-types.md:108-112, twisted-involution-trio.md:25-29]

`LatticeInvolution::new` 重新执行完整验证：两矩阵必须为格秩阶方阵，先检查权格矩阵平方为单位矩阵，再检查余权格矩阵平方为单位矩阵，最后验证配对保持 $W^{T}C=I$。对合条件不成立返回 `InvalidInvolution`，配对保持失败返回 `InvalidRootAutomorphism`；检验算术使用受检的 `i128` 累加，溢出传播 `ArithmeticOverflow`。相关顺序见 [[对合类型的分层构造验证与错误优先级]]。^[involution-types.md:63-71, involution-types.md:108-112]

`RootInvolutionData::new` 进一步验证复合确实置换根，并把每个存储余根运输到像根的余根。仅有配对保持不足以保证这一性质：固定所有根却移动余根中心环面坐标的作用仍须被排除。单根级错误优先于逐根主循环中的泛型错误；根置换的二阶性依赖 `LatticeInvolution` 的验证，不另作独立检查。^[involution-types.md:80-92, involution-types.md:144-145]

## 根类型数据

复合后的根对合按固定优先级分类：像等于自身时为 `Imaginary`，等于负根时为 `Real`，其余为 `Complex`。`RootInvolutionData` 同时提供根像、分类及虚根和实根子系统的单根；后者使用继承的正系，按 `RootId` 升序输出。相关概念见 [[对合下的虚根、实根与复根分类]]。^[involution-types.md:35-46, involution-types.md:91-100]

## 矩阵复合与算术边界

`compose_matrices` 是 `pub(crate)` 辅助函数，也被 `global_tits` 等模块复用。它采用标准三重循环，以受检的 `i128` 算术累加，再通过 `i32::try_from` 收窄。形状不符返回 `RankMismatch`，但其 `actual` 恒为 `right.len()`，即使真实问题是某行长度不齐。相关传输背景见 [[全局 Tits 传输的上下文一致性校验]]。^[involution-types.md:114-116, twisted-involution-trio.md:31-33, twisted-involution-trio.md:79-80]

来源将 `twisted_involution` 的算术策略概括为全 checked；同包对合分类工具混用 checked、saturating 与普通算术，环境根反射工具则使用 wrapping。来源未判定这种差异属于有意分层还是实现漂移。^[twisted-involution-trio.md:79-84]

## 测试与证据范围

来源列出四个测试锚点：A1 的 `s0` 平移合法，产生两个 Real 根；A2 的三阶元 `s0·s1` 被拒绝，返回 `InvalidInvolution`；同秩但不同 datum 的 A2 与 B2 触发 `DatumMismatch`；带中心环面的 `diag(1,-1)` distinguished 与 `s0` 合成合法。^[involution-types.md:130-132, twisted-involution-trio.md:35-37]

覆盖缺口包括 `restricted_roots` 路径及 `compose_matrices` 的直接测试。后者的形状错误在 `TwistedInvolution::new` 内因前置秩检查而不可达，但函数具有 crate 内可见性，仍存在其他调用入口。私有辅助函数的裸下标安全依赖前置方阵与秩检查，这属于阅读推断。^[involution-types.md:138-143, twisted-involution-trio.md:83-84]

两份来源均为维护者核对过的结构性源码阅读，不构成数学验收；上游引用仅转录自代码注释。来源快照记录 Git base、文件字节 SHA-256 与起草调用信息，本次知识维护未执行 Atlas、Cargo、测试或 benchmark，因此上述测试锚点不代表本次运行结果。^[involution-types.md:9-15, involution-types.md:147-156, twisted-involution-trio.md:9-13, twisted-involution-trio.md:86-92]

## Sources

- [involution-types.md](../../sources/involution-types.md) — 对合类型三件套：LatticeInvolution / RootInvolutionData / TwistedInvolution。
- [twisted-involution-trio.md](../../sources/twisted-involution-trio.md) — 扭对合、对合分类与环境根反射字。

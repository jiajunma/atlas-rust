---
title: InnerClass 的根理论状态与实现边界
summary: InnerClass 持有已验证根数据、有限根系及 distinguished 对合，为分解与来源绑定提供上下文，但不持有 Cartan fibers、实形式或 KGB 环面数据。
sources:
  - inner-class.md
kind: concept
createdAt: "2026-10-09T14:50:50.729Z"
updatedAt: "2026-10-10T00:34:58.960Z"
tags:
  - 根理论
  - 架构边界
aliases:
  - innerclass-的根理论状态与实现边界
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: InnerClass 的根理论状态与实现边界
summary: InnerClass 持有已验证的带基根数据、有限根系与 distinguished 对合，提供根理论枚举、分解和来源绑定上下文，但不持有 Cartan fibers、实形式数据或 KGB 所需环面数据。
sources:
  - inner-class.md
kind: concept
tags:
  - 内类
  - 系统设计
aliases:
  - innerclass-的根理论状态与实现边界
provenanceState: extracted
---

# InnerClass 的根理论状态与实现边界

`InnerClass` 是有意保持为部分实现的内类根理论层。它拥有经过验证的 [[BasedRootDatum：带基根数据与构造不变量|BasedRootDatum]]、对应的有限常根系 `RootSystem` 和一个 distinguished `RootInvolutionData`，能够枚举根理论上的 twisted-conjugacy 轨道。^[inner-class.md:17-25]

## 核心职责与边界

`InnerClass` 为 `CayleyCrossDecomposition` 提供 distinguished-involution 上下文，并为 `RealFormLabels` 锚定来源身份。它尚未构建 Atlas Cartan-class fibers，不持有实形式数据，也不包含构造 KGB 图所需的环面数据。相关上层主题见 [[Cartan 分类构造与共享分区]] 与 [[KGB 图与弱实形式]]。^[inner-class.md:19-25]

## 构造与验证保证

`InnerClass::new` 构建共享的根理论状态，根枚举预算由调用方显式提供。构造成功保证 distinguished lattice involution 置换该根系并传送其余根，但这不构成 Atlas 实形式兼容性的声明。^[inner-class.md:32-35]

`InnerClass::from_root_involution` 接受未带基根数据上的任意对合，验证其置换根系并传送余根，再左复合 `wrt_distinguished` 从反射后的单根像中读出的 Weyl 词，使之成为带基根数据的对合。该入口随后丢弃 Weyl 词；需要同时取得相对于所得 distinguished involution 的 Weyl 因子时，可使用委托给 `InnerClass::from_root_involution_with_factor` 的 `inner_class_with_twisted_involution`。参见 [[从根数据对合构造内类]]。^[inner-class.md:29-40]

`based_involution_twist` 要求对合置换本类根系、传送余根，并将每个单根映到单根。成功时返回诱导的单根置换，拒绝时返回 `StructureError::InvalidBasedAutomorphism`。`generator_twist()` 给出 distinguished involution 对简单生成元的置换：`twist[s]` 对应的单根是 $\alpha_s$ 的 distinguished image。参见 [[Based involution 验证与生成元 twist]]。^[inner-class.md:44-51]

## 成员判定与规范化

`twisted_from_involution` 在调用方已检查平方与对合性的前提下，验证输入属于当前内类，并返回分解 $\theta=w\cdot\delta$ 中的 Weyl 元素 $w$，其中 $\delta$ 为 distinguished involution。权格矩阵相等蕴含 twist 比较；拒绝由 `StructureError::InvalidBasedAutomorphism` 表达。参见 [[InnerClass 成员判定与 twisted 分解]]。^[inner-class.md:53-59]

`canonicalize` 采用三阶段算法：先使正实根之和与正虚根之和均占优，再限制到与两个和都正交的简单生成元，最后使实际对合在残余复根子系统中保持正性。返回的生成元按执行顺序排列，依次执行 $\sigma\leftarrow s\cdot\sigma\cdot\delta(s)$ 将输入传送到规范代表元。`canonicalize_with_generators` 将算法限制在指定的 `active` 生成元内，其第二阶段取 `active` 与上述正交生成元集合的交集。参见 [[Twisted involution 的三阶段规范化]]。^[inner-class.md:61-72]

`canonical_involution_expr` 输出 twisted involution 的 Weyl 部分的约化 twisted-involution 表达式，并按**外部生成元编号**选取字典序最小者。普通条目 `s` 表示 cross，即左乘 `s`；按位取反条目 `!s` 表示由 `s` 作 twisted conjugation。实现每步按外部生成元升序选取第一个 descent，再通过 `hasTwistedCommutation` 区分两种操作。调用方必须保证输入是当前内类某个 twisted involution 的 Weyl 部分；循环终止依赖这一前提下每步降低 twisted length。参见 [[Twisted involution 的规范约化表达式]]。^[inner-class.md:74-85]

## 枚举结果与资源约定

`twisted_involutions` 返回形如 `w after distinguished` 的根对合稳定列表，尚未经过 twisted conjugacy 或 Cayley transforms 取商，因此不是 Cartan 类列表。`twisted_conjugacy_classes` 返回确定性的 Weyl twisted-conjugacy 轨道，但其代表元不是 Atlas-canonical，也不构造 Cartan fibers、实形式或 Cartan 偏序。参见 [[Twisted involution 枚举与共轭轨道分区]]。^[inner-class.md:87-94]

`twisted_conjugacy_partition` 提供带成员查询的完整分区，`twisted_conjugacy_classes` 是其薄包装。`generated_twisted_conjugacy_partition` 通过生成元闭包构造完整分区：相较 legacy 版本，预算按 twisted involutions 数量计算，而非所有 Weyl 元素；每类仅实体化一个 lattice involution，成员查询使用紧凑根置换，以避免为每个候选克隆完整 lattice datum。外部 Cartan 编号仍由 `CartanClassification` 选举。参见 [[基于生成元闭包的 twisted 共轭分区]]。^[inner-class.md:95-101]

## 证据范围

本页依据 `inner_class.rs` 的结构性阅读。来源记录的是 dirty 工作区字节，阅读快照为 `snapshots/2026-10-03-inner-class.json`，所读文件 SHA-256 为 `1ee32f3194b13b4fe6d714018a55595358e1be6eae544c8ffc56fd461fcfd31e`。本来源不重述或扩展内类自身的 HPC 正确性证据链。^[inner-class.md:9-15]

来源中的上游行号转述自源码注释，未独立重读上游，可能随版本演进而漂移。本来源未执行构建、测试或原版运行，不提供数学验收、性能或并行结论。^[inner-class.md:103-114]

## Sources

- [inner-class.md](../../sources/inner-class.md) — Inner class 层：构造、验证门与 twisted 共轭枚举。

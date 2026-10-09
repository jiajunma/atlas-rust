---
title: 基于生成元闭包的 twisted 共轭分区
summary: 生成元闭包版本按 twisted involution 数量计预算，每类仅物化一个格对合，并用紧凑根置换查询成员，外部 Cartan 编号仍由 CartanClassification 选举。
sources:
  - inner-class.md
kind: concept
createdAt: "2026-10-09T14:51:15.008Z"
updatedAt: "2026-10-09T22:32:08.102Z"
tags:
  - 轨道枚举
  - 资源预算
  - 数据表示
aliases:
  - 基于生成元闭包的-twisted-共轭分区
  - 基T共
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: 基于生成元闭包的 twisted 共轭分区
summary: 通过生成元闭包构建完整 twisted 共轭分区，按 twisted involution 数量计预算，每类仅物化一个格对合，以紧凑根置换支持成员查询；外部 Cartan 编号仍由 CartanClassification 选举。
sources:
  - inner-class.md
kind: concept
tags:
  - 轨道枚举
  - 生成元闭包
  - 资源预算
  - 数据表示
aliases:
  - 基于生成元闭包的-twisted-共轭分区
---

# 基于生成元闭包的 twisted 共轭分区

`generated_twisted_conjugacy_partition` 通过生成元闭包构建完整的 twisted 共轭分区。相较于旧版实现，它按 twisted involutions 的数量计量预算，每个共轭类只物化一个格对合，并使用紧凑根置换进行成员查询，避免在大群构造中为每个候选项克隆完整格数据。^[inner-class.md:97-101]

## 分区对象与接口关系

`twisted_involutions` 枚举形如 `w after distinguished` 的根对合，产生稳定列表；该列表尚未按 twisted conjugacy 或 Cayley transforms 取商。`twisted_conjugacy_partition` 提供带成员查询的完整分区，`twisted_conjugacy_classes` 是其上的薄封装，返回确定性的 Weyl twisted-conjugacy 轨道。相关概念见 [[Twisted involution 枚举与共轭轨道分区]]。^[inner-class.md:89-96]

这些轨道的代表元不是 Atlas-canonical 代表元，轨道枚举也不构造 Cartan fibers、real forms 或 Cartan 偏序。应保留 [[TwistedConjugacyClass 与 CartanClass 的职责划分]]，不能将根理论轨道分区视为完整的 Cartan 类数据。^[inner-class.md:92-96]

## 预算与数据表示

生成元闭包版本将预算的计数对象从全部 Weyl elements 改为 twisted involutions。两种预算口径对应不同的枚举对象，来源明确将其列为新旧实现的差异。^[inner-class.md:97-99]

数据表示分为两个层次：每个类只物化一个 lattice involution，成员查询则使用 compact root permutations。这样避免了逐候选项克隆完整 lattice datum；来源没有给出量化的内存或速度测量，也不包含性能结论。^[inner-class.md:99-101, inner-class.md:114-114]

## 与 Cartan 编号的衔接

生成元闭包负责构建完整分区，外部 Cartan 编号仍由 `CartanClassification` 选举。该职责衔接可结合 [[Cartan 分类构造与共享分区]] 和 [[CartanId 的 Atlas 编号顺序]] 阅读。^[inner-class.md:97-101]

## 实现与证据边界

该功能属于 [[InnerClass 的根理论状态与实现边界]]。来源描述的 `InnerClass` 是部分实现，拥有已验证的 `BasedRootDatum`、有限常根系 `RootSystem` 和 distinguished `RootInvolutionData`。它能枚举根理论的 twisted-conjugacy 轨道，但尚未构建 Atlas Cartan-class fibers，不持有 real-form data，也不含构造 KGB graph 所需的 torus data。^[inner-class.md:19-25]

本页依据 `inner_class.rs` 的结构性源码阅读，所读字节来自 dirty 工作区，记录于快照 `snapshots/2026-10-03-inner-class.json`。inner class 的正确性另属其自身的 HPC 证据链，包括 capacity/rank6 等验证门；本来源不重述或扩展这些结论，且未执行构建、测试或原版运行，不包含数学验收、性能或并行结论。^[inner-class.md:9-15, inner-class.md:105-106, inner-class.md:114-114]

## Sources

- [inner-class.md](../../sources/inner-class.md) — Inner class 层：构造、验证门与 twisted 共轭枚举。

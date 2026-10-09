---
title: 基于生成元闭包的 twisted 共轭分区
summary: 生成元闭包版本按 twisted involution 数量计预算，每类仅实例化一个格对合并用紧凑根置换查询成员，外部 Cartan 编号仍由 CartanClassification 选举。
sources:
  - inner-class.md
kind: concept
createdAt: "2026-10-09T14:51:15.008Z"
updatedAt: "2026-10-09T19:30:09.515Z"
tags:
  - 扭曲共轭
  - 生成元闭包
  - 资源预算
aliases:
  - 基于生成元闭包的-twisted-共轭分区
  - 基T共
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# 基于生成元闭包的 twisted 共轭分区

`generated_twisted_conjugacy_partition` 通过生成元闭包构建完整的 Weyl twisted-conjugacy 分区。相较于旧版实现，它以 twisted involutions 数量计量预算，每个共轭类仅实例化一个格对合，并使用紧凑根置换进行成员查找，避免为每个候选项克隆完整格数据。^[inner-class.md:97-101]

## 分区对象与接口关系

`twisted_involutions` 枚举形如 `w after distinguished` 的根对合，得到稳定列表；该列表尚未按 twisted conjugacy 或 Cayley transforms 取商。`twisted_conjugacy_partition` 提供带成员查找的完整分区，`twisted_conjugacy_classes` 则是其上的薄封装，返回确定性的 Weyl twisted-conjugacy 轨道。相关背景见 [[Twisted involution 枚举与共轭轨道分区]]。^[inner-class.md:89-96]

这些轨道的代表元不是 Atlas-canonical 代表元，轨道枚举也不构造 Cartan fibers、real forms 或 Cartan 偏序。根理论轨道分区与完整 Cartan 分类的职责需要区分，参见 [[TwistedConjugacyClass 与 CartanClass 的职责划分]]。^[inner-class.md:92-96]

## 预算与数据表示

生成元闭包版本将预算的计数对象从全部 Weyl elements 改为 twisted involutions。这是它与旧版实现的明确差异：预算衡量的是 twisted involutions 的数量。^[inner-class.md:97-99]

在数据表示上，每个类只实例化一个 lattice involution，成员查找使用 compact root permutations。这避免了大群构造中为每个候选项克隆完整 lattice datum；来源未提供量化的内存或速度测量，也不包含性能结论。^[inner-class.md:99-101, inner-class.md:114-114]

## 与 Cartan 编号的衔接

生成元闭包负责构建完整分区，外部 Cartan 编号仍由 `CartanClassification` 选举。因此，分区构造与外部编号选举是不同职责，可结合 [[Cartan 分类构造与共享分区]] 和 [[CartanId 的 Atlas 编号顺序]] 阅读。^[inner-class.md:97-101]

## 实现与证据边界

该功能属于 [[InnerClass 的根理论状态与实现边界]]。来源描述的 `InnerClass` 是部分实现，拥有已验证的 `BasedRootDatum`、有限根系 `RootSystem` 与 distinguished `RootInvolutionData`；它能够枚举根理论的 twisted-conjugacy 轨道，但尚未构建 Atlas Cartan-class fibers，也不持有 real-form data 或构建 KGB graph 所需的 torus data。^[inner-class.md:19-25]

来源是基于 dirty 工作区快照的结构性源码阅读，未执行构建、测试或原版运行，不提供数学验收、性能或并行结论。Inner class 的正确性另属其自身的 [[HPC 验收证据链]]，包括 capacity/rank6 等验证门；本说明不扩展这些验收结论。^[inner-class.md:9-15, inner-class.md:114-114]

## Sources

- [inner-class.md](inner-class.md) — Inner class 层：构造、验证门与 twisted 共轭枚举。

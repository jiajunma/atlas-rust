---
title: 基于生成元闭包的 twisted 共轭分区
summary: generated_twisted_conjugacy_partition 通过生成元闭包构造完整分区，以 twisted involutions 数量计预算、每类仅实例化一个 lattice involution，并使用紧凑根置换查询成员；外部 Cartan 编号仍由 CartanClassification 选举。
sources:
  - inner-class.md
kind: concept
createdAt: "2026-10-09T14:51:15.008Z"
updatedAt: "2026-10-09T14:51:15.008Z"
tags:
  - 生成元闭包
  - 扭曲共轭
  - 数据表示
  - 预算控制
aliases:
  - 基于生成元闭包的-twisted-共轭分区
  - 基T共
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 基于生成元闭包的 twisted 共轭分区

`generated_twisted_conjugacy_partition` 通过生成元闭包构建完整的 Weyl twisted-conjugacy 分区。相较于 legacy 版本，它以 twisted involutions 而非全部 Weyl elements 计量预算，并采用紧凑的根置换表示成员归属，以减少大群构造中的完整格数据克隆。^[inner-class.md:97-101]

## 分区对象与接口关系

`InnerClass` 的 `twisted_involutions` 枚举形如 `w after distinguished` 的根对合，得到稳定列表；该列表尚未按 twisted conjugacy 或 Cayley transforms 取商。`twisted_conjugacy_partition` 则提供带成员查找的完整分区，`twisted_conjugacy_classes` 只是其上的薄封装，用于取得确定性的 Weyl twisted-conjugacy 轨道。^[inner-class.md:89-96]

这些轨道的代表元不是 Atlas-canonical 代表元，轨道枚举也不构造 Cartan fibers、real forms 或 Cartan partial order。因此，理解此接口时应区分根理论轨道分区与 [[Cartan 分类构造与共享分区]]，相关职责边界见 [[TwistedConjugacyClass 与 CartanClass 的职责划分]]。^[inner-class.md:92-101]

## 生成元闭包版本的设计

生成元闭包版本改变了预算的计数对象：预算约束的是 twisted involutions 数量，而不是所有 Weyl elements 的数量。该区别是它与 legacy 版本的重要接口语义差异。^[inner-class.md:97-99]

在存储上，每个共轭类只实体化一个 lattice involution；成员归属使用 compact root permutations 表示。这避免了在大群构造中为每个候选项克隆完整 lattice datum，但来源未给出量化的内存或速度测量。^[inner-class.md:99-101, inner-class.md:114-114]

完整分区的构建不决定外部 Cartan 编号：该编号仍由 `CartanClassification` 选举。分区中的成员归属与外部编号因此需要分别理解，可参见 [[CartanId 的 Atlas 编号顺序]]。^[inner-class.md:97-101]

## 上下文与证据边界

该功能属于 [[InnerClass 的根理论状态与实现边界]]：来源描述的 `InnerClass` 是部分实现，拥有已验证的 `BasedRootDatum`、有限根系 `RootSystem` 与 distinguished `RootInvolutionData`，能够枚举根理论的 twisted-conjugacy 轨道；它尚未构建 Atlas Cartan-class fibers，也不持有 real-form data 或构建 KGB graph 所需的 torus data。^[inner-class.md:19-25]

来源属于结构性源码阅读，所依据的是 dirty 工作区的阅读快照。该来源未执行构建、测试或原版运行，也不提供数学验收、性能或并行结论；生成元闭包设计的说明不能替代独立的 [[HPC 验收证据链]]。^[inner-class.md:9-15, inner-class.md:114-114]

## Sources

- [inner-class.md](inner-class.md) — Inner class 层：构造、验证门与 twisted 共轭枚举。

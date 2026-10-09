---
title: Twisted involution 枚举与共轭轨道分区
summary: 稳定的 twisted involution 列表与带成员查询的完整扭曲共轭分区职责不同，classes 是分区的薄封装，其代表元不保证 Atlas-canonical，结果也不等同于 Cartan classes。
sources:
  - inner-class.md
kind: concept
createdAt: "2026-10-09T14:51:14.369Z"
updatedAt: "2026-10-09T19:29:59.971Z"
tags:
  - 扭曲共轭
  - 轨道枚举
  - 分区
aliases:
  - twisted-involution-枚举与共轭轨道分区
  - TI枚
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# Twisted involution 枚举与共轭轨道分区

`InnerClass` 提供根理论层面的 twisted involution（扭曲对合）枚举与 Weyl twisted-conjugacy（扭曲共轭）轨道分区。枚举接口返回稳定的对合列表，分区接口则组织共轭轨道并提供成员归属查询。这些结果不包含 Cartan fibers、real forms 或 Cartan 偏序。^[inner-class.md:19-25, inner-class.md:89-101]

## 枚举对象与上下文

`InnerClass` 持有经过验证的 `BasedRootDatum`、有限常根系 `RootSystem` 和 distinguished `RootInvolutionData`，为扭曲共轭枚举提供根理论上下文。它属于有意保留的部分实现，不持有 real-form data，也不包含构造 KGB 图所需的 torus data；参见 [[InnerClass 的根理论状态与实现边界]]。^[inner-class.md:19-25]

以 distinguished involution 为 $\delta$，本内类中的对合可分解为 $\theta=w\cdot\delta$。`twisted_involutions` 枚举形如 `w after distinguished` 的 root involutions，返回稳定列表；该列表尚未对 twisted conjugacy 或 Cayley transforms 取商，不能直接视为 Cartan classes。成员判定及 Weyl 因子的提取见 [[InnerClass 成员判定与 twisted 分解]]。^[inner-class.md:55-59, inner-class.md:89-91]

## 共轭轨道与成员查询

`twisted_conjugacy_classes` 返回确定性的 Weyl 扭曲共轭轨道，但其代表元不是 Atlas-canonical representative。该接口不构造 Cartan fibers、real forms 或 Cartan partial order。^[inner-class.md:92-94]

`twisted_conjugacy_partition` 构建带成员归属查询（membership lookup）的完整分区，是这一接口族的唯一轨道实现；`twisted_conjugacy_classes` 只是其上的薄封装。^[inner-class.md:95-96]

Atlas 规范代表元由另一个算法处理：`canonicalize` 通过三阶段过程，将输入搬运到 canonical representative，并按执行顺序返回所用生成元。因此，轨道枚举的确定性代表元与 [[Twisted involution 的三阶段规范化]] 所求的规范代表元需要区分。^[inner-class.md:63-72, inner-class.md:92-94]

## 生成元闭包构造

`generated_twisted_conjugacy_partition` 通过生成元闭包构建完整分区。相较于 legacy 版本，其预算计量对象是 twisted involutions，而非全部 Weyl elements；相关构造见 [[基于生成元闭包的 twisted 共轭分区]]。^[inner-class.md:97-101]

该构造每个类只物化一个 lattice involution，并使用紧凑根置换进行成员归属查询，避免在大群构造中为每个候选克隆完整 lattice datum。外部 Cartan 编号仍由 `CartanClassification` 选举，参见 [[CartanId 的 Atlas 编号顺序]]。^[inner-class.md:97-101]

## 证据边界

本页依据的来源是对 `inner_class.rs` 的结构性阅读，绑定了 dirty 工作区的源码快照。来源未执行构建、测试或原版运行，不提供数学验收、性能或并行结论；inner class 的正确性属于独立的 [[HPC 验收证据链]]，该来源不重述或扩展其结论。^[inner-class.md:9-15, inner-class.md:114-114]

## Sources

- [inner-class.md](inner-class.md) — Inner class 层：构造、验证门与 twisted 共轭枚举。

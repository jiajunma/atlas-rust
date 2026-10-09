---
title: Twisted involution 枚举与共轭轨道分区
summary: 稳定的 twisted involution 列表与带成员查询的完整扭曲共轭分区职责不同，classes 是分区的薄封装，其代表元不保证 Atlas-canonical，结果不等同于 Cartan classes。
sources:
  - inner-class.md
kind: concept
createdAt: "2026-10-09T14:51:14.369Z"
updatedAt: "2026-10-09T20:55:03.047Z"
tags:
  - 对合
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
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: Twisted involution 枚举与共轭轨道分区
summary: InnerClass 提供稳定的扭曲对合列表与带成员查询的共轭轨道分区；轨道代表元不是 Atlas 规范代表元，结果也不包含 Cartan 纤维、实形式或 Cartan 偏序。
sources:
  - inner-class.md
kind: concept
tags:
  - 扭曲共轭
  - 轨道枚举
  - 分区
aliases:
  - twisted-involution-枚举与共轭轨道分区
  - TI枚
---

# Twisted involution 枚举与共轭轨道分区

`InnerClass` 提供根理论层面的 twisted involution（扭曲对合）枚举与 Weyl twisted-conjugacy（扭曲共轭）轨道分区。枚举接口返回稳定的对合列表，分区接口组织共轭轨道并提供成员归属查询；这些结果不构造 Cartan 纤维、实形式或 Cartan 偏序。^[inner-class.md:89-101]

## 枚举对象与上下文

`InnerClass` 持有经过验证的 `BasedRootDatum`、有限常根系 `RootSystem` 和 distinguished `RootInvolutionData`。这是有意保留的部分实现：它提供扭曲共轭枚举所需的根理论上下文，但不持有实形式数据，也不包含构造 KGB 图所需的环面数据，参见 [[InnerClass 的根理论状态与实现边界]]。^[inner-class.md:19-25]

设 distinguished involution 为 $\delta$，本内类中的对合可分解为 $\theta=w\cdot\delta$。`twisted_involutions` 枚举形如 `w after distinguished` 的根对合，返回稳定列表；该列表尚未对扭曲共轭或 Cayley 变换取商，不能直接视为 Cartan 类。成员判定与 Weyl 因子提取见 [[InnerClass 成员判定与 twisted 分解]]。^[inner-class.md:55-59, inner-class.md:89-91]

## 共轭轨道与成员查询

`twisted_conjugacy_partition` 构建带成员归属查询（membership lookup）的完整分区。来源将其明确为唯一的轨道实现；`twisted_conjugacy_classes` 是其上的薄封装，返回确定性的 Weyl 扭曲共轭轨道。^[inner-class.md:92-96]

这些轨道的代表元不是 Atlas 规范代表元，接口也不构造 Cartan 纤维、实形式或 Cartan 偏序。确定性的轨道输出与 Atlas 规范化是不同职责。^[inner-class.md:92-94]

Atlas 规范化由 `canonicalize` 的三阶段算法处理：先使正实根与正虚根之和均为优势，再限制到与两个和都正交的单生成元，最后使实际对合在残余复根子系统中保持正性。算法按执行顺序返回生成元，通过逐步执行 $\sigma\mapsto s\cdot\sigma\cdot\delta(s)$ 将输入搬运至规范代表元，详见 [[Twisted involution 的三阶段规范化]]。^[inner-class.md:63-72]

## 生成元闭包构造

`generated_twisted_conjugacy_partition` 通过生成元闭包构建完整分区。相较于 legacy 版本，其预算计量对象是扭曲对合，而非全部 Weyl 元素；相关主题见 [[基于生成元闭包的 twisted 共轭分区]]。^[inner-class.md:97-101]

该构造每个类只物化一个格对合，并使用紧凑根置换进行成员归属查询，避免在大群构造中为每个候选克隆完整的格数据。外部 Cartan 编号仍由 `CartanClassification` 选举，参见 [[CartanId 的 Atlas 编号顺序]]。^[inner-class.md:97-101]

## 证据边界

本页依据对 `inner_class.rs` 的结构性阅读，来源绑定的是 dirty 工作区源码快照。inner class 的正确性属于其独立的 HPC 证据链，来源不重述或扩展该证据链；本包也未执行构建、测试或原版运行，不提供数学验收、性能或并行结论。^[inner-class.md:9-15, inner-class.md:114-114]

来源中的上游位置转述自源码注释，未独立重读上游文件，行号可能随版本演进而漂移。^[inner-class.md:105-108]

## Sources

- [inner-class.md](../../sources/inner-class.md) — Inner class 层：构造、验证门与 twisted 共轭枚举。

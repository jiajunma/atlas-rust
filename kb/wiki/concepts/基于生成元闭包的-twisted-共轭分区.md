---
title: 基于生成元闭包的 twisted 共轭分区
summary: 生成元闭包版本按 twisted involution 数量计预算，每类仅物化一个格对合，并用紧凑根置换查询成员，外部 Cartan 编号仍由 CartanClassification 选举。
sources:
  - inner-class.md
kind: concept
createdAt: "2026-10-09T14:51:15.008Z"
updatedAt: "2026-10-10T00:35:22.618Z"
tags:
  - 轨道枚举
  - 资源预算
  - 算法
aliases:
  - 基于生成元闭包的-twisted-共轭分区
  - 基T共
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: 基于生成元闭包的 twisted 共轭分区
summary: 通过生成元闭包构建完整 twisted 共轭分区，按 twisted involution 数量计预算，每类仅物化一个格对合，以紧凑根置换支持成员查询；外部 Cartan 编号仍由 CartanClassification 选举。
sources:
  - inner-class.md
kind: concept
tags:
  - 轨道枚举
  - 资源预算
  - 数据表示
aliases:
  - 基于生成元闭包的-twisted-共轭分区
---

# 基于生成元闭包的 twisted 共轭分区

`generated_twisted_conjugacy_partition` 通过生成元闭包构建完整的 twisted 共轭分区。相较于旧版实现，它按 twisted involutions 的数量计量预算，每个共轭类仅物化一个格对合，并使用紧凑根置换进行成员查询，避免在大群构造中为每个候选项克隆完整格数据。^[inner-class.md:97-101]

## 分区对象与接口关系

`twisted_involutions` 枚举形如 `w after distinguished` 的根对合，得到稳定列表；该列表尚未按 twisted conjugacy 或 Cayley transforms 取商，不能直接视为 Cartan 类。`twisted_conjugacy_partition` 提供带成员查询的完整分区，`twisted_conjugacy_classes` 则是其上的薄封装，返回确定性的 Weyl twisted-conjugacy 轨道。参见 [[Twisted involution 枚举与共轭轨道分区]]。^[inner-class.md:89-96]

来源将 `twisted_conjugacy_partition` 描述为唯一的轨道实现，同时将 `generated_twisted_conjugacy_partition` 列为通过生成元闭包构建完整分区的接口。轨道代表元不是 Atlas-canonical 代表元，轨道枚举不构造 Cartan fibers、real forms 或 Cartan 偏序；这些职责应与 [[TwistedConjugacyClass 与 CartanClass 的职责划分]] 一并理解。^[inner-class.md:92-101]

## 预算与数据表示

生成元闭包版本将预算的计数对象从**全部 Weyl elements** 改为 **twisted involutions**。这是来源明确记载的新旧实现差异，预算口径应随枚举对象区分。^[inner-class.md:97-99]

数据表示分为两个层次：每个共轭类只物化一个 lattice involution，成员查询则使用 compact root permutations。该设计避免逐候选项克隆完整 lattice datum；来源不提供性能结论，因此不能据此声称已测得内存降幅或运行加速。^[inner-class.md:99-101, inner-class.md:114-114]

## 与 Cartan 分类的衔接

生成元闭包负责构建完整分区，**外部 Cartan 编号仍由 `CartanClassification` 选举**。相关职责可结合 [[Cartan 分类构造与共享分区]] 和 [[CartanId 的 Atlas 编号顺序]] 阅读。^[inner-class.md:97-101]

该功能属于 [[InnerClass 的根理论状态与实现边界]]。来源中的 `InnerClass` 是部分实现，拥有已验证的 `BasedRootDatum`、有限常根系 `RootSystem` 和 distinguished `RootInvolutionData`。它能够枚举根理论的 twisted-conjugacy 轨道，但尚未构建 Atlas Cartan-class fibers，不持有 real-form data，也不包含构造 KGB graph 所需的 torus data。^[inner-class.md:19-25]

## 证据边界

本页依据 `inner_class.rs` 的结构性阅读。所读字节来自 dirty 工作区，记录于快照 `snapshots/2026-10-03-inner-class.json`；来源草稿经维护者逐条对照源码核对后改写。inner class 的正确性另属其自身的 HPC 证据链，包括 capacity/rank6 等验证门，本来源不重述或扩展这些结论。^[inner-class.md:9-15]

来源未执行构建、测试或原版运行，不包含数学验收、性能或并行结论。文中上游行号转述自源码注释，未独立重读上游文件，可能随版本演进而漂移。^[inner-class.md:107-114]

## Sources

- [inner-class.md](../../sources/inner-class.md) — Inner class 层：构造、验证门与 twisted 共轭枚举。

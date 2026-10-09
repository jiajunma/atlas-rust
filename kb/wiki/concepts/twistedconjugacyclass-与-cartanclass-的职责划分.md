---
title: TwistedConjugacyClass 与 CartanClass 的职责划分
summary: TwistedConjugacyClass 保存轨道代表元与对合计数，CartanClass 持有该值并承载分解、纤维和实形式标签，分类阶段采用 Atlas 规范代表元。
sources:
  - cartan-classification.md
kind: concept
createdAt: "2026-10-09T14:42:24.674Z"
updatedAt: "2026-10-09T22:25:52.189Z"
tags:
  - Rust设计
  - 扭曲共轭
  - Cartan分类
aliases:
  - twistedconjugacyclass-与-cartanclass-的职责划分
  - T与C的
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: TwistedConjugacyClass 与 CartanClass 的职责划分
summary: TwistedConjugacyClass 保存扭曲共轭轨道的代表元与对合计数，CartanClass 持有该值并承载纤维、实形式归属及实 Cartan 分量数据；分解与标签对应同一代表元。
sources:
  - cartan-classification.md
kind: concept
tags:
  - 扭曲共轭
  - Cartan分类
  - 类型设计
---

# TwistedConjugacyClass 与 CartanClass 的职责划分

`TwistedConjugacyClass` 描述 Weyl 扭曲共轭下的一条确定性轨道，保存轨道代表元与扭曲对合数量。`CartanClass` 拥有一个 `TwistedConjugacyClass` 值，并承载纤维群（fiber groups）、实形式归属与实 Cartan 分量数据。两者分别负责轨道描述及其关联的 Cartan 结构。^[cartan-classification.md:80-90]

## 轨道表示与代表元来源

`TwistedConjugacyClass` 定义于 `cartan_class.rs`，包含 `representative: TwistedInvolution` 与 `twisted_involution_count: usize` 两个字段，分别记录代表元和轨道中的扭曲对合数量。代表元类型参见 [[扭曲对合（TwistedInvolution）]]。^[cartan-classification.md:80-82]

代表元有两种来源：`TwistedConjugacyPartition` 产出的类采用本 crate 确定性 Weyl 枚举中的第一个 action；`CartanClassification` 消费这些类时，则用 Atlas-canonical 代表元重建。因此，分区阶段的枚举代表元与分类阶段的规范代表元应予以区分。^[cartan-classification.md:82-85]

规范化也是 [[CartanId 的 Atlas 编号顺序]] 的一部分：fundamental class 编号为 `CartanId(0)`，其余类按 BFS 发现顺序编号；Cayley successor 在比较与存储之前先经过 `InnerClass::canonicalize`。^[cartan-classification.md:25-31]

## Cartan 结构与代表元一致性

无论采用哪种代表元，类都经 `CayleyCrossDecomposition` 分解，实形式标签通过 `RealFormLabels` 在同一代表元处关联。分解与标签的关联以所采用的代表元为共同依据，相关主题见 [[扭曲对合的 Cayley/Cross 分解]]。^[cartan-classification.md:85-87]

`CartanClass` 承载纤维群、实形式归属和实 Cartan 分量数据，其访问器包括 `representative`、`twisted_involution_count`、`decomposition`、`grading`、`partition` 与 `labels`，供消费者访问轨道描述及关联结构。^[cartan-classification.md:87-90]

## 与整体分类的关系

`CartanClassification::build` 构建一个 inner class 的全部 Cartan 类，并聚合每个实形式的 Cartan 集、most-split 类、twisted-involution 总数和[[严格 Cayley 偏序]]。内部的 `partition: Arc<TwistedConjugacyPartition>` 会被 dual correspondence 与 dual-side classification 复用而无需重建，详见 [[Cartan 分类构造与共享分区]]。^[cartan-classification.md:18-21]

外部消费者通过 `cartan_ids()` 按升序遍历编号，并通过 `cartan_class(id)` 获取 `Option<&CartanClass>`，从整体分类进入单个 Cartan 类的数据访问。^[cartan-classification.md:30-31]

## 证据边界

本页依据 `cartan_classification.rs` 与 `cartan_class.rs` 的结构性阅读，来源快照记录的是 dirty 工作区字节。来源包未执行构建、测试或原版运行，不提供数学验收、性能或并行结论，也未重述或扩展 Cartan 分类正确性的既有 HPC 证据链。^[cartan-classification.md:9-14, cartan-classification.md:105-105]

`CayleyCrossDecomposition`、`CartanGradingData`、`RealFormLabels`、`WeakRealFormPartition` 与 `TwistedConjugacyPartition` 的详细机制属于后续来源包的范围。本页仅说明这些结构在类型与分类流程中的职责关系。来源中的上游行号转述自源码注释，未独立重读上游，可能随版本演进而漂移。^[cartan-classification.md:98-104]

## Sources

- [cartan-classification.md](../../sources/cartan-classification.md) — Cartan 分类：编号、预算与实形式归属。

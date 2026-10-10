---
title: TwistedConjugacyClass 与 CartanClass 的职责划分
summary: TwistedConjugacyClass 保存代表元与轨道计数，CartanClass 持有该值并承载分解、纤维与实形式标签；分类阶段使用 Atlas 规范代表元。
sources:
  - cartan-classification.md
kind: concept
createdAt: "2026-10-09T14:42:24.674Z"
updatedAt: "2026-10-10T00:28:55.892Z"
tags:
  - 扭曲共轭
  - Cartan分类
  - 类型设计
aliases:
  - twistedconjugacyclass-与-cartanclass-的职责划分
  - T与C的
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: TwistedConjugacyClass 与 CartanClass 的职责划分
summary: TwistedConjugacyClass 保存扭曲共轭轨道的代表元与对合计数，CartanClass 持有该值并承载纤维、实形式归属及实 Cartan 分量数据；分解与标签对应同一代表元。
sources:
  - cartan-classification.md
kind: concept
tags:
  - Rust设计
  - 扭曲共轭
  - Cartan分类
---

# TwistedConjugacyClass 与 CartanClass 的职责划分

`TwistedConjugacyClass` 描述 Weyl 扭曲共轭下的一条确定性轨道，保存代表元与扭曲对合数量。`CartanClass` 拥有一个 `TwistedConjugacyClass` 值，并承载纤维群（fiber groups）、实形式归属与实 Cartan 分量数据。两者分别组织轨道描述及其关联的 Cartan 结构。^[cartan-classification.md:80-90]

## 轨道表示与代表元来源

`TwistedConjugacyClass` 定义于 `cartan_class.rs`，包含两个字段：`representative: TwistedInvolution` 与 `twisted_involution_count: usize`，分别记录轨道代表元和扭曲对合数量。^[cartan-classification.md:80-82]

代表元有两种来源。`TwistedConjugacyPartition` 产出的类采用本 crate 确定性 Weyl 枚举中的第一个 action；`CartanClassification` 消费这些类时，则用 Atlas-canonical 代表元重建。因此，分区阶段的枚举代表元与分类阶段的规范代表元需要区分。^[cartan-classification.md:82-85]

规范化发生在编号流程中：Cayley successor 在比较与存储之前先经过 `InnerClass::canonicalize`。fundamental class 编号为 `CartanId(0)`，其余类按 BFS 发现顺序编号；parent 按编号升序处理，其 positive imaginary roots 按上游 `RootNbr` 顺序遍历。详见 [[CartanId 的 Atlas 编号顺序]]。^[cartan-classification.md:25-31]

## Cartan 结构与代表元一致性

无论代表元来自哪种途径，类都经 `CayleyCrossDecomposition` 分解，实形式标签通过 `RealFormLabels` 在同一代表元处关联。分解与标签因此以所采用的代表元为共同依据，相关主题见 [[扭曲对合的 Cayley/Cross 分解]]。^[cartan-classification.md:85-87]

纤维群、实形式归属和实 Cartan 分量数据由 `CartanClass` 承载。其访问器包括 `representative`、`twisted_involution_count`、`decomposition`、`grading`、`partition` 与 `labels`，覆盖轨道描述及关联结构。^[cartan-classification.md:87-90]

## 与整体分类的关系

`CartanClassification::build` 构建一个 inner class 的全部 Cartan 类，聚合每个实形式的 Cartan 集、most-split 类、twisted-involution 总数与[[严格 Cayley 偏序]]。分类内部的 `partition: Arc<TwistedConjugacyPartition>` 由 dual correspondence 与 dual-side classification 复用，无需重建，参见 [[Cartan 分类构造与共享分区]]。^[cartan-classification.md:18-21]

外部消费者通过 `cartan_ids()` 按升序遍历编号，通过 `cartan_class(id)` 获取 `Option<&CartanClass>`，从整体分类访问单个 Cartan 类。^[cartan-classification.md:30-31]

## 证据边界

本页依据 `cartan_classification.rs` 与 `cartan_class.rs` 的结构性阅读，来源快照记录的是两个文件的 dirty 工作区字节。来源包未执行构建、测试或原版运行，不提供数学验收、性能或并行结论，也不重述或扩展 Cartan 分类正确性的既有 HPC 证据链。^[cartan-classification.md:9-14, cartan-classification.md:105-105]

`CayleyCrossDecomposition`、`CartanGradingData`、`RealFormLabels`、`WeakRealFormPartition` 与 `TwistedConjugacyPartition` 的详细机制属于后续来源包的范围。来源中的上游行号转述自源码注释，未独立重读上游，可能随版本演进而漂移。^[cartan-classification.md:98-104]

## Sources

- [cartan-classification.md](../../sources/cartan-classification.md) — Cartan 分类：编号、预算与实形式归属。

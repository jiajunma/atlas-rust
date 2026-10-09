---
title: TwistedConjugacyClass 与 CartanClass 的职责划分
summary: TwistedConjugacyClass 保存轨道代表元与对合计数，CartanClass 持有该值并承载分解、纤维、实形式归属和标签，分类阶段采用 Atlas 规范代表元。
sources:
  - cartan-classification.md
kind: concept
createdAt: "2026-10-09T14:42:24.674Z"
updatedAt: "2026-10-09T19:27:12.526Z"
tags:
  - 扭曲共轭
  - Cartan分类
  - 数据模型
aliases:
  - twistedconjugacyclass-与-cartanclass-的职责划分
  - T与C的
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# TwistedConjugacyClass 与 CartanClass 的职责划分

`TwistedConjugacyClass` 描述 Weyl 扭曲共轭下的一条确定性轨道，保存代表元和 twisted involution 数量；`CartanClass` 拥有一个 `TwistedConjugacyClass` 值，并承载 fiber groups、实形式归属与实 Cartan 分量数据。两者分别负责轨道描述与 Cartan 类的附加结构。^[cartan-classification.md:80-90]

## 轨道代表元与构造来源

`TwistedConjugacyClass` 包含两个字段：`representative: TwistedInvolution` 和 `twisted_involution_count: usize`。代表元有两种来源：`TwistedConjugacyPartition` 产出的类采用本 crate 确定性 Weyl 枚举中的第一个 action；`CartanClassification` 消费这些类时，则以 Atlas-canonical 代表元重建。^[cartan-classification.md:80-85]

规范代表元的选择与 [[CartanId 的 Atlas 编号顺序]] 相连：fundamental class 编号为 `CartanId(0)`，后续类按 BFS 发现顺序编号；Cayley successor 在比较与存储之前先经过 `InnerClass::canonicalize`。因此，分区枚举选取的代表元与分类阶段使用的规范代表元应予区分。^[cartan-classification.md:25-31, cartan-classification.md:83-85]

## Cartan 数据与代表元一致性

无论代表元来自哪种构造，类都经 `CayleyCrossDecomposition` 分解，实形式标签由 `RealFormLabels` 在同一代表元处关联。分解和标签因此与所采用的代表元保持对应关系。^[cartan-classification.md:85-87]

fiber groups、实形式归属与实 Cartan 分量数据由 `CartanClass` 承载。它提供 `representative`、`twisted_involution_count`、`decomposition`、`grading`、`partition` 与 `labels` 访问器，将轨道描述与 Cartan 层面的结构集中到同一个访问接口中。^[cartan-classification.md:87-90]

## 与整体分类的关系

`CartanClassification::build` 构建一个 inner class 的全部 Cartan 类，并聚合每个实形式的 Cartan 集、most-split 类、twisted-involution 总数以及[[严格 Cayley 偏序]]。分类对象内部的 `Arc<TwistedConjugacyPartition>` 可被 dual correspondence 与 dual-side classification 复用而不重建；这属于 [[Cartan 分类构造与共享分区]] 的整体职责。^[cartan-classification.md:18-21]

外部消费者通过 `cartan_ids()` 按升序遍历类编号，再通过 `cartan_class(id)` 获取 `Option<&CartanClass>`。这一接口将分类层的编号与单个 Cartan 类的数据访问衔接起来。^[cartan-classification.md:30-31]

## 证据边界

本页依据对 `cartan_classification.rs` 与 `cartan_class.rs` 的结构性阅读，来源快照记录的是 dirty 工作区字节。来源包未执行构建、测试或原版运行，不提供数学验收、性能或并行结论；分解、grading、标签及分区类型的进一步展开也属于后续来源包的范围。^[cartan-classification.md:9-14, cartan-classification.md:103-105]

## Sources

- [cartan-classification.md](cartan-classification.md) — Cartan 分类：编号、预算与实形式归属。

---
title: Cartan 分类构造与共享分区
summary: CartanClassification::build 构造 inner class 的全部 Cartan 类，聚合各实形式的 Cartan 集、most-split 类、twisted-involution 总数与严格偏序，并共享 twisted 共轭分区供对偶侧复用。
sources:
  - cartan-classification.md
kind: concept
createdAt: "2026-10-09T14:41:59.024Z"
updatedAt: "2026-10-09T14:41:59.024Z"
tags:
  - Cartan分类
  - Rust设计
  - 数据共享
aliases:
  - cartan-分类构造与共享分区
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# Cartan 分类构造与共享分区

`CartanClassification::build` 构造一个 inner class 的全部 Cartan 类，并聚合四类信息：每个弱实形式的 Cartan 集合、most-split 类、twisted-involution 总数，以及严格 Cartan 偏序。内部以 `Arc<TwistedConjugacyPartition>` 保存共享分区，供 dual correspondence 与 dual-side classification 复用，避免重建分区。^[cartan-classification.md:18-21]

## 构造顺序与代表元

Cartan 类编号遵循 Atlas 顺序：fundamental class 为 `CartanId(0)`，其余按 BFS 发现顺序编号。构造时按编号升序处理 parent，并按上游 `RootNbr` 顺序遍历其 positive imaginary roots，即先比较 height，再比较 simple 坐标的 reverse-lexicographic 顺序。每个 Cayley successor 在比较与存储前，都先经 `InnerClass::canonicalize` 规范化。相关编号契约见 [[CartanId 的 Atlas 编号顺序]]。^[cartan-classification.md:25-29]

分区代表元与分类代表元的来源不同：`TwistedConjugacyPartition` 以本 crate 确定性 Weyl 枚举中遇到的第一个 action 作为类代表；`CartanClassification` 消费这些类时，则用 Atlas-canonical 代表元重建。因此，分区的共享复用与分类层的代表元规范化是两个分别明确的构造环节。^[cartan-classification.md:18-21, cartan-classification.md:80-85]

## 分区与 Cartan 类的职责

`TwistedConjugacyClass` 表示 Weyl twisted conjugacy 下的一条确定性轨道，保存 `representative: TwistedInvolution` 与 `twisted_involution_count: usize`。`CartanClass` 拥有一个这样的值，并承载 fiber groups、实形式归属和实 Cartan 分量数据；两者的职责划分见 [[TwistedConjugacyClass 与 CartanClass 的职责划分]]。^[cartan-classification.md:80-90]

无论代表元来自分区枚举还是 Atlas 规范化，类都会经 `CayleyCrossDecomposition` 分解，并在同一代表元处通过 `RealFormLabels` 关联实形式标签。`CartanClass` 提供 `representative`、`twisted_involution_count`、`decomposition`、`grading`、`partition` 与 `labels` 访问器。^[cartan-classification.md:85-90]

## 分类结果与查询

外部消费者通过 `cartan_ids()` 按编号升序遍历，使用 `cartan_class(id)` 获取 `Option<&CartanClass>`。弱实形式数量由 fundamental partition 给出；`cartan_set(form)` 返回该形式所属的 Cartan 类，按升序排列；`most_split(form)` 返回其唯一的 most-split Cartan 类。相关性质见 [[弱实形式的 Cartan 集与唯一 most-split 类]]。^[cartan-classification.md:30-31, cartan-classification.md:53-55]

分类还构造[[严格 Cayley 偏序]]：`a` 严格位于 `b` 之下，意味着它位于一条进入 `b` 的非空 single-root Cayley links 链的 more-compact 端。fundamental class 位于其他每个类之下；不可反身性是构造不变量，故 `is_below(x, x)` 恒为 `Some(false)`。^[cartan-classification.md:43-49]

## 构造预算

`CartanClassificationBudget` 分别控制整数格运算、每个 Cartan 的 adjoint fiber chains、Weyl 枚举、可选的 involution 数量上限、fiber 元素数量和 peeling 步数，对应字段为 `integer_lattice`、`adjoint_fiber`、`weyl_budget`、`involution_budget`、`max_fiber_elements` 与 `max_peeling_steps`。这些预算值均属于 classification cache key 的组成部分，详见 [[Cartan 分类的分层预算]]。^[cartan-classification.md:35-41]

默认构造使用 legacy full-Weyl 枚举预算。调用 `with_generated_involutions(limit)` 可切换到 canonical representative discovery，即 direct classification；该模式使用独立的精确 twisted-involution 计数上限，计数包含 identity，且不改动 `weyl_budget` 与 legacy 构造模式。^[cartan-classification.md:38-41]

## 证据边界

本页依据的来源包属于结构性源码阅读，记录的是两个 Rust 文件的 dirty 工作区字节。它未执行构建、测试或原版运行，不提供数学验收、性能或并行结论；Cartan 分类的正确性属于独立的 [[HPC 验收证据链]]，此处不扩展其覆盖范围。^[cartan-classification.md:9-14, cartan-classification.md:105-105]

## Sources

- [cartan-classification.md](cartan-classification.md) — Cartan 分类：编号、预算与实形式归属。

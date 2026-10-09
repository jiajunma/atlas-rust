---
title: Cartan 分类构造与共享分区
summary: CartanClassification::build 聚合各实形式的 Cartan 集、唯一 most-split 类、扭曲对合总数及严格偏序，并共享 TwistedConjugacyPartition 供对偶计算复用。
sources:
  - cartan-classification.md
kind: concept
createdAt: "2026-10-09T14:41:59.024Z"
updatedAt: "2026-10-09T20:48:51.497Z"
tags:
  - Cartan分类
  - 系统设计
aliases:
  - cartan-分类构造与共享分区
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: Cartan 分类构造与共享分区
summary: CartanClassification::build 构造 inner class 的全部 Cartan 类，聚合弱实形式的 Cartan 集、most-split 类、twisted-involution 总数与严格偏序，并共享 twisted 共轭分区供对偶侧复用。
sources:
  - cartan-classification.md
kind: concept
tags:
  - Cartan分类
  - Rust设计
  - 数据共享
aliases:
  - cartan-分类构造与共享分区
---

# Cartan 分类构造与共享分区

`CartanClassification::build` 构造一个 inner class 的全部 Cartan 类，并聚合四类信息：各实形式的 Cartan 集合、most-split 类、twisted-involution 总数，以及严格 Cartan 偏序。内部字段 `partition: Arc<TwistedConjugacyPartition>` 保存共享分区，供对偶对应（dual correspondence）和对偶侧分类（dual-side classification）复用，无须重建分区。^[cartan-classification.md:18-21]

## 构造顺序与代表元

Cartan 类采用 [[CartanId 的 Atlas 编号顺序]]：fundamental class 编号为 `CartanId(0)`，其余按 BFS 发现顺序编号。构造按编号升序处理 parent，并按上游 `RootNbr` 顺序遍历其正虚根，即先按 height，再按单根坐标的逆字典序排列。每个 Cayley successor 在比较与存储之前，都先经 `InnerClass::canonicalize` 规范化。^[cartan-classification.md:25-29]

共享分区与分类层采用的代表元来源不同。`TwistedConjugacyPartition` 以本 crate 确定性 Weyl 枚举中遇到的第一个 action 为类代表；`CartanClassification` 消费这些类时，则用 Atlas-canonical 代表元重建。分区复用并不取消分类层的代表元规范化步骤。^[cartan-classification.md:18-21, cartan-classification.md:80-85]

## 分区与 Cartan 类的职责

`TwistedConjugacyClass` 表示 Weyl twisted conjugacy 下的一条确定性轨道，保存 `representative: TwistedInvolution` 与 `twisted_involution_count: usize`。`CartanClass` 拥有一个这样的值，并承载 fiber groups、实形式归属和实 Cartan 分量数据，详见 [[TwistedConjugacyClass 与 CartanClass 的职责划分]]。^[cartan-classification.md:80-90]

无论代表元来自分区枚举还是 Atlas 规范化，类都会经 `CayleyCrossDecomposition` 分解，并在同一代表元处通过 `RealFormLabels` 关联实形式标签。`CartanClass` 提供 `representative`、`twisted_involution_count`、`decomposition`、`grading`、`partition` 与 `labels` 访问器。^[cartan-classification.md:85-90]

## 分类结果与查询

外部消费者通过 `cartan_ids()` 按编号升序遍历，并用 `cartan_class(id)` 获取 `Option<&CartanClass>`。`weak_real_form_count()` 给出来自 fundamental partition 的弱实形式数量；`cartan_set(form)` 返回该形式所处的 Cartan 类，按升序排列；`most_split(form)` 返回其唯一的 most-split Cartan 类。相关性质见 [[弱实形式的 Cartan 集与唯一 most-split 类]]。^[cartan-classification.md:30-31, cartan-classification.md:53-55]

分类构造的 [[严格 Cayley 偏序]] 中，`a` 严格位于 `b` 之下，表示 `a != b`，且 `b` 的固定环面的单位连通分支可经 Weyl 共轭嵌入 `a` 的相应分支；等价地，`a` 位于一条进入 `b` 的非空单根 Cayley 链的 more-compact 端。fundamental class 位于其他每个类之下，不可反身性是构造不变量，因此有效类编号满足 `is_below(x, x) == Some(false)`。^[cartan-classification.md:43-49]

## 构造预算

`CartanClassificationBudget` 包含六个字段：`integer_lattice`、`adjoint_fiber`、`weyl_budget`、`involution_budget: Option<usize>`、`max_fiber_elements` 和 `max_peeling_steps`；其中 `adjoint_fiber` 用于每个 Cartan 的 fiber chains。各 getter 均注明相应预算值属于 classification cache key，详见 [[Cartan 分类的分层预算]]。^[cartan-classification.md:35-41]

默认构造使用 legacy full-Weyl 枚举预算。`with_generated_involutions(limit)` 可切换到 canonical representative discovery，即 direct classification；该模式采用独立的精确 twisted-involution 计数上限，计数包含 identity，同时不改动 `weyl_budget` 与 legacy 构造模式。^[cartan-classification.md:38-41]

## 证据边界

来源包属于 `cartan_classification.rs` 与 `cartan_class.rs` 的结构性源码阅读，快照记录的是两个文件的 dirty 工作区字节。Cartan 分类的正确性属于其独立的 HPC 证据链，本包不重述或扩展该证据范围，也未执行构建、测试或原版运行，不提供数学验收、性能或并行结论。^[cartan-classification.md:9-14, cartan-classification.md:105-105]

来源中的上游行号转述自源码注释，未独立重读上游，可能随版本变化而漂移；`CayleyCrossDecomposition`、`CartanGradingData`、`RealFormLabels`、`WeakRealFormPartition` 与 `TwistedConjugacyPartition` 的内部展开不在本来源包的覆盖范围内。^[cartan-classification.md:98-104]

## Sources

- [cartan-classification.md](../../sources/cartan-classification.md) — Cartan 分类：编号、预算与实形式归属。

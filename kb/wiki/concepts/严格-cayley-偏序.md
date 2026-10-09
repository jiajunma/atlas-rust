---
title: 严格 Cayley 偏序
summary: 非空单根 Cayley 链定义 Cartan 类的严格偏序，fundamental 类位于其他类之下，有效类与自身比较返回 Some(false)。
sources:
  - cartan-classification.md
kind: concept
createdAt: "2026-10-09T14:42:02.992Z"
updatedAt: "2026-10-09T20:48:57.023Z"
tags:
  - Cartan分类
  - 偏序
  - Cayley变换
aliases:
  - 严格-cayley-偏序
  - 严C偏
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: 严格 Cayley 偏序
summary: 非空单根 Cayley 链定义 Cartan 类的严格偏序，fundamental 类位于其他类之下，有效类的自比较为 Some(false)。
sources:
  - cartan-classification.md
kind: concept
tags:
  - Cartan分类
  - 偏序
aliases:
  - 严格-cayley-偏序
  - 严C偏
---

# 严格 Cayley 偏序

严格 Cayley 偏序描述同一 inner class 内 Cartan 类之间由非空单根 Cayley 链确定的关系。它由 `CartanClassification::build` 建立，是 [[Cartan 分类构造与共享分区]] 聚合的数据之一。^[cartan-classification.md:18-21, cartan-classification.md:43-49]

## 数学含义与方向

对于有效的 Cartan 类编号，`is_below(a, b)` 为真，当且仅当 \(a \ne b\)，且 \(b\) 的固定环面的单位连通分支可以经 Weyl 共轭嵌入 \(a\) 的相应单位连通分支。等价地，存在一条从 \(a\) 进入 \(b\) 的非空单根 Cayley 链，其中 \(a\) 位于较紧致（more-compact）的一端。^[cartan-classification.md:45-47]

fundamental class 位于其他每个 Cartan 类之下。关系的不可反身性是构造不变量，因此对于有效类编号，`is_below(x, x)` 恒为 `Some(false)`；自比较不属于严格偏序关系。^[cartan-classification.md:48-49]

## 查询接口与编号

`is_below(a, b)` 在 `a` 越界时返回 `None`；否则查询内部存储的 `below[b][a]`。存储下标先是目标类 `b`，再是被判定位于其下的类 `a`。^[cartan-classification.md:45-49]

类编号遵循 [[CartanId 的 Atlas 编号顺序]]：fundamental class 为 `CartanId(0)`，其余类按 BFS 发现顺序编号。parent 按编号升序处理，其正虚根按上游 `RootNbr` 顺序处理，即先按 height，再按单根坐标的逆字典序。Cayley successor 在比较与存储之前先经 `InnerClass::canonicalize` 规范化。^[cartan-classification.md:25-29]

外部消费者通过 `cartan_ids()` 按编号升序遍历，并用 `cartan_class(id)` 获取 `Option<&CartanClass>`。^[cartan-classification.md:30-31]

## 在实形式构造中的用途

Cartan 分类记录每个弱实形式所处的 Cartan 类集合，以及该形式唯一的 most-split Cartan 类，参见 [[弱实形式的 Cartan 集与唯一 most-split 类]]。^[cartan-classification.md:53-55]

`real_form_of_detailed` 在返回强对合数据所属弱实形式的同时，返回其所属的 Cartan 类。synthetic wrapper 利用该类，在调用 `minimal_torus_part` 前，将 involution table 扩展到它之下的每个类；相关归属机制见 [[强对合数据的弱实形式归属]]。^[cartan-classification.md:56-62, cartan-classification.md:74-76]

## 证据边界

本页依据对 `cartan_classification.rs` 与 `cartan_class.rs` 的结构性阅读，来源快照对应两个文件的 dirty 工作区字节。Cartan 分类正确性属于独立的 HPC 证据链，来源包不重述或扩展该证据链。^[cartan-classification.md:9-14]

来源包未执行构建、测试或原版运行，不包含数学验收、性能或并行结论。其上游行号转述自源码注释，未独立重读上游，可能随版本演进而漂移。^[cartan-classification.md:98-105]

## Sources

- [cartan-classification.md](../../sources/cartan-classification.md) — Cartan 分类：编号、预算与实形式归属。

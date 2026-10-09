---
title: 弱实形式的 Cartan 集与唯一 most-split 类
summary: 弱实形式数量来自 fundamental partition，每个形式关联升序排列的 Cartan 类集合及唯一的 most-split Cartan 类。
sources:
  - cartan-classification.md
kind: concept
createdAt: "2026-10-09T14:42:28.349Z"
updatedAt: "2026-10-09T22:25:41.045Z"
tags:
  - 弱实形式
  - Cartan分类
aliases:
  - 弱实形式的-cartan-集与唯一-most-split-类
  - 弱C集M类
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: 弱实形式的 Cartan 集与唯一 most-split 类
summary: 弱实形式数量来自 fundamental partition；每个形式关联按 CartanId 升序排列的 Cartan 集及唯一的 most-split Cartan 类。
sources:
  - cartan-classification.md
kind: concept
tags:
  - 弱实形式
  - Cartan分类
aliases:
  - 弱实形式的-cartan-集与唯一-most-split-类
  - 弱C集M类
---

# 弱实形式的 Cartan 集与唯一 most-split 类

`CartanClassification::build` 为一个内类（inner class）构建全部 Cartan 类，并汇总每个弱实形式的 Cartan 集及其唯一的 most-split（最分裂）Cartan 类。这些信息与 twisted-involution 总数、严格 Cartan 偏序共同构成分类结果。^[cartan-classification.md:18-21, cartan-classification.md:53-55]

## 查询接口

`weak_real_form_count()` 返回弱实形式数量，数量来自 fundamental partition。`cartan_set(form)` 返回给定弱实形式所处的 Cartan 类，按编号升序排列；`most_split(form)` 返回该形式唯一的最分裂 Cartan 类。^[cartan-classification.md:53-55]

## Cartan 集的编号顺序

Cartan 集遵循 [[CartanId 的 Atlas 编号顺序]]：fundamental class 编号为 `CartanId(0)`，其余类按 BFS 发现顺序编号。构造时按编号升序处理 parent，并按上游 `RootNbr` 顺序遍历各 parent 的正虚根，即先按 height，再按 simple 坐标的逆字典序。Cayley successor 在比较与存储之前先经 `InnerClass::canonicalize` 规范化。^[cartan-classification.md:25-29]

全局接口 `cartan_ids()` 按编号升序迭代，`cartan_class(id)` 返回 `Option<&CartanClass>`。按弱实形式查询得到的 Cartan 集也按这一编号升序排列。^[cartan-classification.md:30-31, cartan-classification.md:53-55]

## [[严格-cayley-偏序|严格 Cayley 偏序]]

[[严格 Cayley 偏序]] 描述 Cartan 类之间的方向关系：`is_below(a, b)` 为真，当且仅当 `a != b`，且 `b` 的 fixed torus 的单位连通分量可经 Weyl 共轭嵌入 `a` 的相应分量。等价地，`a` 位于一条通向 `b` 的非空 single-root Cayley links 链的更紧致端。fundamental class 位于其他每个类之下；该关系不可反身，有效类与自身比较得到 `Some(false)`。^[cartan-classification.md:45-49]

## 从强对合数据确定归属

`real_form_of(inner_class, twisted, factor)` 返回强对合数据 `(twisted, factor)` 所属的弱实形式，其中 `factor` 是调用方投影得到的 theta-fixed rational coweight。该接口将具体强对合数据与弱实形式归属联系起来，相关机制见 [[强对合数据的弱实形式归属]]。^[cartan-classification.md:56-62]

`real_form_of_detailed` 在上述结果之外，还返回 `twisted` 所属的 Cartan 类。synthetic wrapper 需要该类，以便在调用 `minimal_torus_part` 前，将 involution table 扩展到它之下的每个类。^[cartan-classification.md:74-76]

## 证据范围

本页依据 `cartan_classification.rs` 与 `cartan_class.rs` 的结构性阅读，所记录快照对应两个文件的 dirty 工作区字节。来源明确记录了每个弱实形式具有唯一 most-split 类这一接口事实，但不重述或扩展 Cartan 分类自身的 HPC 正确性证据链。该来源包未执行构建、测试或原版运行，不提供数学验收、性能或并行结论。^[cartan-classification.md:9-14, cartan-classification.md:53-55, cartan-classification.md:105-105]

## Sources

- [Cartan 分类：编号、预算与实形式归属](../../sources/cartan-classification.md)

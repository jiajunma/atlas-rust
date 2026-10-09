---
title: 严格 Cayley 偏序
summary: is_below 描述非空 single-root Cayley links 链的 more-compact 方向，fundamental class 位于其他类之下，有效类的自比较恒为 Some(false)。
sources:
  - cartan-classification.md
kind: concept
createdAt: "2026-10-09T14:42:02.992Z"
updatedAt: "2026-10-09T14:42:02.992Z"
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
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 严格 Cayley 偏序

严格 Cayley 偏序描述同一 inner class 内 Cartan 类之间由非空 single-root Cayley links 链确定的关系。它由 `CartanClassification::build` 建立，是 Cartan 分类聚合的数据之一。^[cartan-classification.md:18-21, cartan-classification.md:43-49]

## 数学含义

若 `is_below(a, b)` 为真，则 \(a \ne b\)，且 \(b\) 的固定环面的单位连通分支可以经 Weyl 共轭嵌入 \(a\) 的相应单位连通分支。等价地，存在一条从 \(a\) 进入 \(b\) 的非空单根 Cayley 链，\(a\) 位于较紧致（more-compact）的一端。^[cartan-classification.md:45-47]

fundamental class 位于其他每个 Cartan 类之下。该关系具有严格性：不可反身性是构造不变量，因此对于有效类编号，`is_below(x, x)` 恒为 `Some(false)`。^[cartan-classification.md:48-49]

## 查询与编号

`is_below(a, b)` 在 `a` 越界时返回 `None`；否则查询内部存储的 `below[b][a]`。存储下标的顺序是先目标类 `b`、再被判定位于其下的类 `a`。^[cartan-classification.md:45-49]

类编号遵循 [[CartanId 的 Atlas 编号顺序]]：fundamental class 为 `CartanId(0)`，其余类按 BFS 发现顺序编号；Cayley successor 在比较与存储之前先经 `InnerClass::canonicalize` 规范化。外部消费者通过 `cartan_ids()` 按编号升序遍历，并可用 `cartan_class(id)` 获取 `Option<&CartanClass>`。^[cartan-classification.md:25-31]

## 使用场景

弱实形式的分类数据包括其 Cartan 集合与唯一的 most-split Cartan 类，可参见 [[弱实形式的 Cartan 集与唯一 most-split 类]]。在 synthetic wrapper 中，`real_form_of_detailed` 还返回输入强对合数据所属的 Cartan 类，以便在调用 `minimal_torus_part` 前，将 involution table 扩展到该类之下的每个类。^[cartan-classification.md:53-62, cartan-classification.md:74-76]

## 证据边界

本页依据的来源是对 `cartan_classification.rs` 与 `cartan_class.rs` 的结构性阅读，快照对应 dirty 工作区字节。来源包未执行构建、测试或原版运行；Cartan 分类的正确性属于独立的 [[HPC 验收证据链]]，此处不据此扩展数学验收结论。^[cartan-classification.md:9-14, cartan-classification.md:105-105]

## Sources

- [cartan-classification.md](cartan-classification.md) — Cartan 分类：编号、预算与实形式归属。

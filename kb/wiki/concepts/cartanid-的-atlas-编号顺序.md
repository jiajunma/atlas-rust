---
title: CartanId 的 Atlas 编号顺序
summary: fundamental class 编为 0，其余按父类编号与上游正虚根顺序进行 BFS 发现；Cayley successor 在比较和存储前先 canonicalize，以保持 Atlas 编号语义。
sources:
  - cartan-classification.md
kind: concept
createdAt: "2026-10-09T14:41:59.420Z"
updatedAt: "2026-10-09T14:41:59.420Z"
tags:
  - Cartan分类
  - 确定性编号
  - 基线对齐
aliases:
  - cartanid-的-atlas-编号顺序
  - C的A编
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# CartanId 的 Atlas 编号顺序

`CartanId(usize)` 是 Cartan 分类中的类编号，遵循 Atlas Cartan 顺序：fundamental class 固定为 `CartanId(0)`，其余类按广度优先搜索（BFS）的发现顺序编号。编号过程同时规定父类遍历顺序、根的遍历顺序以及代表元规范化的时机。^[cartan-classification.md:25-31]

## 编号规则

BFS 按已有编号升序处理父类；对每个父类，按上游 `RootNbr` 顺序遍历正虚根，即先按 height 排序，再按 simple 坐标的逆字典序排序。由此产生的 Cayley successor 必须先经过 `InnerClass::canonicalize`，之后才能参与比较与存储。根编号的相关背景见 [[RootNumbering 根编号与 RootNbr 顺序]]。^[cartan-classification.md:25-29]

这一顺序对应源码注释所引用的 `innerclass.cpp:218-291`（task 1），其中 successor 的规范化对应 `innerclass.cpp:252-263`。这些上游位置来自源码注释的转述，来源包未独立重读上游文件，因此行号可能随版本变化。^[cartan-classification.md:25-29, cartan-classification.md:98-99]

## 与轨道代表元的关系

`TwistedConjugacyPartition` 产生的类，使用本 crate 确定性 Weyl 枚举中遇到的第一个 action 作为代表元；`CartanClassification` 消费这些类时，则使用 Atlas-canonical 代表元重建。因而理解 Cartan 编号时，需要区分分区中的初始代表元与分类所采用的规范代表元；后者与 Cayley successor 在编号前的规范化相联系。参见 [[TwistedConjugacyClass 与 CartanClass 的职责划分]]。^[cartan-classification.md:80-90]

## 查询与偏序

外部消费者通过 `cartan_ids()` 按编号升序迭代，并通过 `cartan_class(id)` 查询相应的类；查询返回类型为 `Option<&CartanClass>`。^[cartan-classification.md:30-31]

分类还维护独立的 [[严格 Cayley 偏序]]，由 `is_below(a, b)` 查询。fundamental class 位于其他每个类之下；该关系是严格的，对有效类编号有 `is_below(x, x) == Some(false)`。^[cartan-classification.md:43-49]

## 证据范围

本页依据来源包对 Rust 构造与编号机制的结构性阅读，所用快照记录的是两个源码文件的 dirty 工作区字节。来源包未执行构建、测试或原版运行，不提供新的数学验收、性能或并行结论；Cartan 分类正确性属于其独立的 [[HPC 验收证据链]]。^[cartan-classification.md:9-14, cartan-classification.md:105-105]

## Sources

- [cartan-classification.md](cartan-classification.md) — Cartan 分类：编号、预算与实形式归属。

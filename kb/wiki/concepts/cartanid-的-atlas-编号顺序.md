---
title: CartanId 的 Atlas 编号顺序
summary: fundamental 类编号为 0，其余按父类编号及上游正虚根顺序进行 BFS 发现，Cayley 后继在比较与存储前先规范化。
sources:
  - cartan-classification.md
kind: concept
createdAt: "2026-10-09T14:41:59.420Z"
updatedAt: "2026-10-09T19:26:48.636Z"
tags:
  - Cartan分类
  - 编号规则
aliases:
  - cartanid-的-atlas-编号顺序
  - C的A编
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# CartanId 的 Atlas 编号顺序

`CartanId(usize)` 是 Cartan 分类中的类编号，遵循 Atlas Cartan 顺序：fundamental class 固定为 `CartanId(0)`，其余类按广度优先搜索（BFS）的发现顺序编号。编号规则同时规定父类与根的遍历顺序，以及 Cayley successor 的规范化时机。^[cartan-classification.md:25-31]

## 编号规则

BFS 按已有编号升序处理父类。对每个父类，按上游 `RootNbr` 顺序遍历正虚根：先按 height 排序，再按 simple 坐标的逆字典序排序。相关根编号约定见 [[RootNumbering 根编号与 RootNbr 顺序]]。^[cartan-classification.md:25-28]

生成的 Cayley successor 在参与比较与存储之前，必须先经过 `InnerClass::canonicalize`。规范化因此是编号过程的一部分。源码注释将整体编号流程对应到 `innerclass.cpp:218-291` 的 task 1，将 successor 规范化对应到 `innerclass.cpp:252-263`。^[cartan-classification.md:25-29]

## 与轨道代表元的关系

`TwistedConjugacyPartition` 产生的类以本 crate 确定性 Weyl 枚举中遇到的第一个 action 为代表元；`CartanClassification` 消费这些类时，使用 Atlas-canonical 代表元重建。这一区别与 Cayley successor 在编号前执行规范化直接相关。^[cartan-classification.md:80-85]

无论代表元来自哪一种路径，类都经 `CayleyCrossDecomposition` 分解，并在同一代表元处关联 `RealFormLabels`。`CartanClass` 拥有一个 `TwistedConjugacyClass` 值，并承载 fiber groups、实形式归属及实 Cartan 分量数据，详见 [[TwistedConjugacyClass 与 CartanClass 的职责划分]]。^[cartan-classification.md:85-90]

## 查询与偏序

外部消费者通过 `cartan_ids()` 按编号升序迭代；`cartan_class(id)` 用于查询相应的类，返回 `Option<&CartanClass>`。^[cartan-classification.md:30-31]

分类还维护 [[严格 Cayley 偏序]]，通过 `is_below(a, b)` 查询。fundamental class 位于其他每个类之下；该关系不可反身，因此对有效类编号有 `is_below(x, x) == Some(false)`。^[cartan-classification.md:43-49]

## 证据范围

本页依据来源包对 `cartan_classification.rs` 与 `cartan_class.rs` 的结构性阅读，快照记录的是两个文件的 dirty 工作区字节。上游行号转述自源码注释，未独立重读上游文件，可能随版本变化。^[cartan-classification.md:9-14, cartan-classification.md:94-99]

来源包未执行构建、测试或原版运行，不提供数学验收、性能或并行结论。Cartan 分类正确性属于其独立的 [[HPC 验收证据链]]，来源提及 Cartan3868252 与 rank1 class/dual-incidence 覆盖，但不重述或扩展这些证据。^[cartan-classification.md:10-12, cartan-classification.md:105-105]

## Sources

- [cartan-classification.md](cartan-classification.md) — Cartan 分类：编号、预算与实形式归属。

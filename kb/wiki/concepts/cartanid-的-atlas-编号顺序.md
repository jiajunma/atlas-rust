---
title: CartanId 的 Atlas 编号顺序
summary: fundamental 类编号为零，其余按父类编号及正虚根的上游 RootNbr 顺序进行 BFS 发现，Cayley 后继在比较和存储前先规范化。
sources:
  - cartan-classification.md
kind: concept
createdAt: "2026-10-09T14:41:59.420Z"
updatedAt: "2026-10-10T00:28:13.643Z"
tags:
  - Cartan分类
  - 编号约定
aliases:
  - cartanid-的-atlas-编号顺序
  - C的A编
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: CartanId 的 Atlas 编号顺序
summary: 基本类编号为 0，其余类按父类编号与正虚根的上游 RootNbr 顺序进行 BFS 发现；Cayley 后继在比较和存储前先规范化。
sources:
  - cartan-classification.md
kind: concept
tags:
  - Cartan分类
  - 确定性编号
aliases:
  - cartanid-的-atlas-编号顺序
---

# CartanId 的 Atlas 编号顺序

`CartanId(usize)` 是 Cartan 分类中的类编号，遵循 Atlas Cartan 顺序：基本类（fundamental class）为 `CartanId(0)`，其余类按广度优先搜索（BFS）的发现顺序编号。该规则规定了父类与正虚根的遍历顺序，以及 Cayley 后继的规范化时机。^[cartan-classification.md:25-31]

## 编号规则

BFS 按编号升序处理父类。对每个父类，其正虚根按上游 `RootNbr` 顺序遍历：先按高度（height），再按单根坐标的逆字典序（reverse-lexicographic）排列。相关编号概念见 [[RootNumbering 根编号与 RootNbr 顺序]]。^[cartan-classification.md:25-28]

每个 Cayley 后继在参与比较与存储之前，先经过 `InnerClass::canonicalize` 规范化。来源将整体编号流程对应到上游 `innerclass.cpp:218-291` 的 task 1，将后继规范化对应到 `innerclass.cpp:252-263`；这些上游位置均转述自源码注释。^[cartan-classification.md:25-29, cartan-classification.md:98-99]

## 与轨道代表元的关系

`TwistedConjugacyPartition` 产生的类，以本 crate 确定性 Weyl 枚举中遇到的第一个 action 为代表元；`CartanClassification` 消费这些类时，使用 Atlas-canonical 代表元重建，其中 Cayley 后继在编号前完成规范化。两种构造路径因此采用不同的代表元选择规则。^[cartan-classification.md:80-85]

无论采用哪种代表元，类都经 `CayleyCrossDecomposition` 分解，实形式标签通过 `RealFormLabels` 在同一代表元处关联。`CartanClass` 拥有一个 `TwistedConjugacyClass` 值，并承载 fiber groups、实形式归属与实 Cartan 分量数据，详见 [[TwistedConjugacyClass 与 CartanClass 的职责划分]]。^[cartan-classification.md:85-90]

## 查询接口与严格偏序

外部消费者通过 `cartan_ids()` 按编号升序迭代；`cartan_class(id)` 查询对应的 Cartan 类，返回 `Option<&CartanClass>`。^[cartan-classification.md:30-31]

分类还维护 [[严格 Cayley 偏序]]。`is_below(a, b)` 为真表示 `a != b`，且 `a` 位于一条进入 `b` 的非空单根 Cayley 链的更紧（more-compact）一端；等价地，`b` 的固定环面单位分量可经 Weyl 共轭嵌入 `a` 的对应分量。基本类位于其他每个类之下；关系不可反身，因此对有效类编号有 `is_below(x, x) == Some(false)`。^[cartan-classification.md:43-49]

## 证据范围

本页依据来源包对 `cartan_classification.rs` 与 `cartan_class.rs` 的结构性阅读，所读快照记录的是两个文件的 dirty 工作区字节。上游行号转述自源码注释，未独立重读上游文件，可能随版本演进而漂移。^[cartan-classification.md:9-14, cartan-classification.md:94-99]

来源包未执行构建、测试或原版运行，不提供数学验收、性能或并行结论。Cartan 分类正确性属于独立的 HPC 证据链；来源提及 Cartan3868252 与 rank1 class/dual-incidence 覆盖，但不重述或扩展这些证据。^[cartan-classification.md:10-12, cartan-classification.md:105-105]

## Sources

- [cartan-classification.md](../../sources/cartan-classification.md) — Cartan 分类：编号、预算与实形式归属。

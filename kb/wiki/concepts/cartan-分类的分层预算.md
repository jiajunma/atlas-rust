---
title: Cartan 分类的分层预算
summary: 分类预算分别约束整数格、伴随纤维链、Weyl 枚举、对合数量、纤维元素和剥离步骤；直接分类的对合计数包含 identity，各预算值参与缓存键。
sources:
  - cartan-classification.md
kind: concept
createdAt: "2026-10-09T14:41:58.846Z"
updatedAt: "2026-10-09T20:48:44.684Z"
tags:
  - 资源预算
  - 缓存
  - Cartan分类
aliases:
  - cartan-分类的分层预算
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: Cartan 分类的分层预算
summary: Cartan 分类预算包含六个字段；默认采用全 Weyl 枚举，直接分类模式使用包含恒等元的精确 twisted involution 计数上限，各预算值参与分类缓存键。
sources:
  - cartan-classification.md
kind: concept
createdAt: "2026-10-09T14:41:58.846Z"
updatedAt: "2026-10-09T19:26:49.255Z"
tags:
  - Cartan分类
  - 资源预算
aliases:
  - cartan-分类的分层预算
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# Cartan 分类的分层预算

`CartanClassificationBudget` 为 [[Cartan 分类构造与共享分区|Cartan 分类构造]]提供分层资源预算，涉及整数格、逐 Cartan 纤维链、Weyl 枚举、twisted involution 计数、纤维元素数量与剥离步骤。各预算 getter 均注明，对应值属于分类缓存键的一部分。^[cartan-classification.md:35-41]

## 预算组成

`CartanClassificationBudget` 包含以下六个字段。其中，`adjoint_fiber` 用于每个 Cartan 的纤维链，`involution_budget` 的类型为 `Option<usize>`。^[cartan-classification.md:35-37]

| 字段 | 预算对象或含义 |
| --- | --- |
| `integer_lattice` | 整数格计算预算 |
| `adjoint_fiber` | 每个 Cartan 的纤维链预算 |
| `weyl_budget` | Weyl 枚举预算 |
| `involution_budget` | 可选的 twisted involution 计数上限 |
| `max_fiber_elements` | 纤维元素数量上限 |
| `max_peeling_steps` | 剥离步骤上限 |

## 默认模式与直接分类

默认采用 legacy full-Weyl 枚举预算。调用 `with_generated_involutions(limit)` 切换为 canonical representative discovery，即直接分类模式；该模式使用自己的精确 twisted involution 计数上限，计数包含恒等元（identity）。相关概念见 [[Twisted involution 枚举与共轭轨道分区]]。^[cartan-classification.md:38-41]

这一切换不修改 `weyl_budget`，也不改动 legacy 构造模式本身。因此，直接分类的对合计数上限与原有全 Weyl 枚举预算是需要分别理解的约束。^[cartan-classification.md:38-41]

## 缓存与说明范围

各预算值均参与分类缓存键。来源未给出六个字段的具体默认数值、预算检查位置或超限错误行为，本页的说明范围限于预算组成、构造模式及其缓存关系。^[cartan-classification.md:33-41]

## 证据边界

来源属于结构性源码阅读，所读 `cartan_classification.rs` 与 `cartan_class.rs` 均为 dirty 工作区字节。Cartan 分类正确性另属其 HPC 证据链，来源包不重述或扩展该证据；来源包也未执行构建、测试或原版运行，不提供数学验收、性能或并行结论。^[cartan-classification.md:9-14, cartan-classification.md:105-105]

## Sources

- [cartan-classification.md](../../sources/cartan-classification.md) — Cartan 分类：编号、预算与实形式归属。

---
title: Cartan 分类的分层预算
summary: 分类预算分别约束整数格、纤维链、Weyl 枚举、对合计数与剥离步骤，直接分类模式的对合计数包含 identity，各预算值参与缓存键。
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

`CartanClassificationBudget` 为 [[Cartan 分类构造与共享分区|Cartan 分类构造]]提供分层资源预算，分别涉及整数格、逐 Cartan 纤维链、Weyl 枚举、twisted involution 计数、纤维元素数量与剥离步骤。各预算 getter 均注明，对应值属于分类缓存键的一部分。^[cartan-classification.md:33-41]

## 预算组成

`CartanClassificationBudget` 包含以下六个字段。其中，`adjoint_fiber` 明确用于每个 Cartan 的纤维链，`involution_budget` 的类型为 `Option<usize>`。^[cartan-classification.md:35-37]

| 字段 | 预算对象或含义 |
| --- | --- |
| `integer_lattice` | 整数格计算预算 |
| `adjoint_fiber` | 每个 Cartan 的纤维链预算 |
| `weyl_budget` | Weyl 枚举预算 |
| `involution_budget` | 可选的 twisted involution 计数上限 |
| `max_fiber_elements` | 纤维元素数量上限 |
| `max_peeling_steps` | 剥离步骤上限 |

## 默认模式与直接分类

默认构造采用 legacy full-Weyl 枚举预算。调用 `with_generated_involutions(limit)` 切换为 canonical representative discovery，即直接分类模式；该模式使用自己的精确 twisted involution 计数上限，且计数包含恒等元。相关背景见 [[Twisted involution 枚举与共轭轨道分区]]。^[cartan-classification.md:38-41]

这一切换不修改 `weyl_budget`，也不改动 legacy 构造模式本身。直接分类的对合计数限制与原有全 Weyl 枚举预算因此需要分别理解。^[cartan-classification.md:38-41]

## 缓存与证据边界

各预算值均参与分类缓存键。来源未展开六个字段的具体默认数值、检查位置或超限错误行为，因此本页仅描述预算组成与构造模式的关系。^[cartan-classification.md:33-41]

来源属于结构性源码阅读，所读两个文件均为 dirty 工作区字节，且未执行构建、测试或原版运行，不提供数学验收、性能或并行结论。Cartan 分类的正确性另属其 [[HPC 验收证据链]]，本页不扩展该证据的范围。^[cartan-classification.md:9-14, cartan-classification.md:105-105]

## Sources

- [cartan-classification.md](cartan-classification.md) — Cartan 分类：编号、预算与实形式归属。

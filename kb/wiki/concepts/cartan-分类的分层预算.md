---
title: Cartan 分类的分层预算
summary: CartanClassificationBudget 分别约束整数格、fiber、Weyl 枚举、对合发现与 peeling；generated-involutions 模式以含 identity 的精确计数上限控制代表元发现，各预算值参与分类缓存键。
sources:
  - cartan-classification.md
kind: concept
createdAt: "2026-10-09T14:41:58.846Z"
updatedAt: "2026-10-09T14:41:58.846Z"
tags:
  - 资源预算
  - 分类算法
  - 缓存
aliases:
  - cartan-分类的分层预算
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# Cartan 分类的分层预算

`CartanClassificationBudget` 为 [[Cartan 分类构造与共享分区|Cartan 分类构造]]提供分层预算，分别涉及整数格、逐 Cartan 纤维链、Weyl 枚举、对合计数、纤维元素数与剥离步骤。各预算 getter 均注明，其对应值属于 classification cache key 的一部分。^[cartan-classification.md:33-41]

## 预算组成

`CartanClassificationBudget` 包含六个字段；其中 `adjoint_fiber` 明确用于 per-Cartan fiber chains，`involution_budget` 的类型为 `Option<usize>`。^[cartan-classification.md:35-37]

| 字段 | 预算对象或含义 |
| --- | --- |
| `integer_lattice` | 整数格预算 |
| `adjoint_fiber` | 每个 Cartan 的纤维链预算 |
| `weyl_budget` | Weyl 枚举预算 |
| `involution_budget: Option<usize>` | 可选的 twisted-involution 计数上限 |
| `max_fiber_elements` | 纤维元素数量上限 |
| `max_peeling_steps` | 剥离步骤上限 |

## 默认模式与直接分类

默认构造采用 legacy full-Weyl 枚举预算。调用 `with_generated_involutions(limit)` 则切换为 canonical representative discovery，即直接分类模式；该模式使用自己的精确 twisted-involution 计数上限，且计数包含 identity。^[cartan-classification.md:38-41]

这一切换不修改 `weyl_budget`，也不改动 legacy 构造模式本身。因此，直接分类的对合计数上限应与原有全 Weyl 枚举预算区分理解。相关结构可参见 [[Twisted involution 枚举与共轭轨道分区]]。^[cartan-classification.md:38-41]

## 缓存与证据边界

预算值参与分类缓存键，属于分类构造的缓存身份信息。来源未展开六个字段各自的默认数值、检查位置或超限错误行为，因而不能据此补充具体的失败顺序或资源消耗保证。^[cartan-classification.md:33-41]

本来源是对构造机制的结构性阅读，所读两个源码文件均为 dirty 工作区字节；未执行构建、测试或原版运行，不提供数学验收、性能或并行结论。Cartan 分类的正确性另有其 [[HPC 验收证据链]]，本页不扩展该证据的适用范围。^[cartan-classification.md:9-14, cartan-classification.md:105-105]

## Sources

- [cartan-classification.md](cartan-classification.md) — Cartan 分类：编号、预算与实形式归属。

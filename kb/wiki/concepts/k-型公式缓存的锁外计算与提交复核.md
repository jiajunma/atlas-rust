---
title: K 型公式缓存的锁外计算与提交复核
summary: 公式生成期间不持有共享互斥锁，提交时重新检查缓存；若并发调用已提交更大截断的公式，则保留该公式，但来源未提供性能或并行效果验证。
sources:
  - rep-table.md
kind: concept
createdAt: "2026-10-09T15:09:31.584Z"
updatedAt: "2026-10-09T15:09:31.584Z"
tags:
  - 并发控制
  - 缓存
  - 锁外计算
aliases:
  - k-型公式缓存的锁外计算与提交复核
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# K 型公式缓存的锁外计算与提交复核

`RepTableOwner::k_type_formula(ktype, max_level)` 为一个实形式提供记忆化的 K 型公式。其并发约定是：生成公式期间不持有共享互斥锁，在锁外完成计算后，提交时重新检查缓存；若另一调用方已提交更大截断的公式，则保留后者。^[rep-table.md:63-68]

## 缓存身份与截断复用

缓存键采用该实形式所有者内部的严格 K 型身份 `(x, lambda_rho)`。这一缓存位于 [[RepTableOwner 实形式资源所有者]] 内，其身份范围受所属实形式所有者约束。^[rep-table.md:63-65]

调用可能返回缓存中截断高度高于 `max_level` 的公式，因此调用方在导出前必须自行将各项截断到所请求的高度。缓存复用与输出截断是两个独立责任，详见 [[K 型公式的记忆化与截断复用]]。^[rep-table.md:65-68]

## 锁外计算与提交复核

公式生成在共享互斥锁之外进行。计算完成后的提交阶段会再次检查缓存，以处理计算期间其他调用方已经更新缓存的情况：若其已提交更大截断的公式，就保留该公式。这使提交决定依据复核时的缓存状态，而非仅依据开始计算时的状态。^[rep-table.md:66-68]

这一约定与 [[共享 KL 表的惰性构造与回调并发约定]] 不同：`with_kl_table` 在整个回调期间持有记录局部互斥锁，同一块的调用被串行化；K 型公式缓存则明确将生成工作放在共享锁之外，并在提交时复核。^[rep-table.md:50-54, rep-table.md:63-68]

## 证据边界

上述行为来自对 `rep_table.rs` 的结构性阅读。来源包未执行构建、测试或原版运行，因此这里描述的是已核对的实现约定，不构成数学验收、性能提升或并行效果的验证结论。^[rep-table.md:9-15, rep-table.md:80-85]

## Sources

- [rep-table.md](rep-table.md)：共享块存储：reduced 键控复用与 RepTableOwner。

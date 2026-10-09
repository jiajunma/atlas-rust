---
title: K 型公式缓存的锁外计算与提交复核
summary: 公式在共享锁外计算，提交时复核缓存并保留并发调用已提交的更大截断结果；来源不提供性能或并行效果验收。
sources:
  - rep-table.md
kind: concept
createdAt: "2026-10-09T15:09:31.584Z"
updatedAt: "2026-10-09T21:08:25.639Z"
tags:
  - K型
  - 并发控制
  - 缓存
aliases:
  - k-型公式缓存的锁外计算与提交复核
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: K 型公式缓存的锁外计算与提交复核
summary: K 型公式在共享互斥锁外生成，提交时复核缓存并保留其他调用方已提交的更大截断结果；调用方负责按请求高度截断输出。
sources:
  - rep-table.md
kind: concept
tags:
  - K型公式
  - 缓存
  - 并发控制
aliases:
  - k-型公式缓存的锁外计算与提交复核
---

# K 型公式缓存的锁外计算与提交复核

`RepTableOwner::k_type_formula(ktype, max_level)` 提供记忆化的 K 型公式。公式生成期间不持有共享互斥锁；计算完成后，在提交阶段复核缓存。若另一调用方已在此期间提交更大截断的公式，则保留后者。^[rep-table.md:63-68]

## 缓存身份与截断复用

缓存键是 [[RepTableOwner 实形式资源所有者]] 内部的严格 K 型身份 `(x, lambda_rho)`，其作用范围属于该实形式所有者。调用可能返回截断高度高于请求 `max_level` 的缓存公式，因此调用方在导出前必须自行将各项截断到所求高度。参见 [[K 型公式的记忆化与截断复用]]。^[rep-table.md:63-66]

## 锁外计算与提交复核

这一并发约定包含两个阶段：先在共享锁外计算公式，再在提交时检查缓存是否已由其他调用方更新。如果生成期间已有更大截断的公式提交，当前提交须保留该结果。来源明确规定了对更大截断结果的保留规则，但没有在此展开相同截断高度下的提交细节。^[rep-table.md:66-68]

## 与共享 KL 表的对照

[[共享 KL 表的惰性构造与回调并发约定]] 使用不同的锁持有规则：`with_kl_table(operation)` 在整个回调期间持有记录局部互斥锁，同一块上的调用因此串行化。KL 回调不得对任何块再次调用 `with_kl_table`；同线程嵌套会由 `ActiveKlCallback::enter()` 在获取另一记录锁之前返回 `RepInvariantViolation`。K 型公式缓存的约定则是锁外生成、提交时复核。^[rep-table.md:50-54, rep-table.md:63-68]

## 证据边界

上述说明来自对 `rep_table.rs` 的结构性阅读，所读字节记录为 dirty 工作区快照。来源记载，更高截断复用与锁外计算等内容已由维护者逐条对照源码核对；本包未执行构建、测试或原版运行，因此不提供数学验收、性能或并行效果的验证结论。^[rep-table.md:9-15, rep-table.md:80-85]

## Sources

- [rep-table.md](../../sources/rep-table.md)：共享块存储：reduced 键控复用与 RepTableOwner。

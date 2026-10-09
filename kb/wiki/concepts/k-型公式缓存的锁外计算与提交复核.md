---
title: K 型公式缓存的锁外计算与提交复核
summary: 公式在共享锁外生成，提交时复核缓存并保留并发调用已提交的更大截断结果；来源未提供性能或并行效果验收。
sources:
  - rep-table.md
kind: concept
createdAt: "2026-10-09T15:09:31.584Z"
updatedAt: "2026-10-09T22:46:38.717Z"
tags:
  - K型
  - 并发
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
summary: K 型公式在共享互斥锁外生成，提交时复核缓存并保留并发调用已提交的更大截断结果；调用方负责按请求高度截断输出。
sources:
  - rep-table.md
kind: concept
tags:
  - K型
  - 并发控制
  - 缓存
aliases:
  - k-型公式缓存的锁外计算与提交复核
---

# K 型公式缓存的锁外计算与提交复核

`RepTableOwner::k_type_formula(ktype, max_level)` 提供记忆化的 K 型公式。其并发约定是：**公式生成期间不持有共享互斥锁，提交时再复核缓存**；如果另一调用方在计算期间已提交更大截断的公式，则保留后者。^[rep-table.md:63-68]

## 缓存身份与截断复用

缓存键是 [[RepTableOwner 实形式资源所有者]] 内部的严格 K 型身份 `(x, lambda_rho)`，作用范围限于该实形式所有者。调用可能返回截断高度高于请求 `max_level` 的缓存公式，因此调用方在导出前必须自行将各项截断到所求高度，详见 [[K 型公式的记忆化与截断复用]]。^[rep-table.md:63-66]

## 锁外计算与提交复核

计算与提交分为两个阶段：先在共享锁外生成公式，再在提交阶段复核缓存。复核时必须考虑其他调用方在此期间提交的结果；若已有更大截断的公式，保留该公式，不能用本次较小截断的结果替换它。来源未展开相同截断高度下的提交细节。^[rep-table.md:66-68]

## 与共享 KL 表的对照

[[共享 KL 表的惰性构造与回调并发约定]] 采用另一种锁持有规则：`with_kl_table(operation)` 在整个回调期间持有记录局部互斥锁，因此同一块上的调用被串行化。KL 回调不得对任何块再次调用 `with_kl_table`；同线程嵌套会由 `ActiveKlCallback::enter()` 在获取另一记录锁之前返回 `RepInvariantViolation`。这项回调重入禁令应与 K 型公式缓存的锁外计算、提交复核约定分别理解。^[rep-table.md:50-54, rep-table.md:63-68]

## 证据边界

本页依据 `rep_table.rs` 的结构性阅读，所读字节记录在 dirty 工作区快照中。来源记载，更高截断复用、锁外计算以及 KL 回调的互斥与重入约定，均已由维护者逐条对照源码核对。^[rep-table.md:9-15, rep-table.md:81-85]

来源包未执行构建、测试或原版运行，不提供数学验收、性能或并行效果的验证结论。上述机制描述不能据此扩展为已测得的加速或并行验收结果。^[rep-table.md:80-80]

## Sources

- [rep-table.md](../../sources/rep-table.md)：共享块存储：reduced 键控复用与 RepTableOwner。

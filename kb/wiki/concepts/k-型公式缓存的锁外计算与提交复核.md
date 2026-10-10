---
title: K 型公式缓存的锁外计算与提交复核
summary: 公式在共享锁外生成，提交时复核缓存并保留并发调用已提交的更大截断结果；来源不提供性能或并行效果验收。
sources:
  - rep-table.md
kind: concept
createdAt: "2026-10-09T15:09:31.584Z"
updatedAt: "2026-10-10T00:49:48.039Z"
tags:
  - k型
  - 缓存
  - 并发控制
aliases:
  - k-型公式缓存的锁外计算与提交复核
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: K 型公式缓存的锁外计算与提交复核
summary: K 型公式在共享锁外生成，提交时复核缓存并保留并发调用已提交的更大截断结果；调用方负责按请求高度截断输出。
sources:
  - rep-table.md
kind: concept
tags:
  - K型
  - 并发
  - 缓存
aliases:
  - k-型公式缓存的锁外计算与提交复核
---

# K 型公式缓存的锁外计算与提交复核

`RepTableOwner::k_type_formula(ktype, max_level)` 提供记忆化的 K 型公式。其并发约定是：**公式生成期间不持有共享互斥锁，提交时再复核缓存**；如果另一调用方在计算期间已提交更大截断的公式，则保留后者。^[rep-table.md:63-68]

## 缓存身份与截断复用

缓存键是 [[RepTableOwner 实形式资源所有者]] 内部的严格 K 型身份 `(x, lambda_rho)`，缓存作用范围限于该实形式所有者。调用可能返回截断高度高于请求 `max_level` 的缓存公式，因此调用方在导出前必须自行将各项截断到所求高度，详见 [[K 型公式的记忆化与截断复用]]。^[rep-table.md:63-66]

## 锁外计算与提交复核

公式生成与缓存提交分为两个阶段：先在共享锁外计算，再在提交时复核缓存。复核处理计算期间其他调用方可能提交的结果；若另一调用方已提交更大截断的公式，则保留该公式，避免本次较小截断的结果将其替换。^[rep-table.md:66-68]

## 与共享 KL 表的并发约定对照

[[共享 KL 表的惰性构造与回调并发约定]] 使用不同的锁持有规则：`with_kl_table(operation)` 在整个回调期间持有记录局部互斥锁，同一块上的调用因此被串行化。KL 回调不得对任何块再次调用 `with_kl_table`；同线程嵌套会由 `ActiveKlCallback::enter()` 在获取另一记录锁之前返回不变量错误 `RepInvariantViolation`。^[rep-table.md:48-54]

这项 KL 回调重入禁令与 K 型公式缓存的锁外计算、提交复核是不同接口的约定；来源对 K 型公式描述的是生成阶段与提交阶段的锁使用方式。^[rep-table.md:50-54, rep-table.md:63-68]

## 证据边界

本页依据 `rep_table.rs` 的结构性阅读，所读字节记录在 dirty 工作区快照 `snapshots/2026-10-03-rep-table.json` 中。来源记载，更高截断复用、锁外计算及 KL 回调的互斥与重入约定，均已由维护者逐条对照源码核对。^[rep-table.md:9-15, rep-table.md:81-85]

来源包未执行构建、测试或原版运行，不含数学验收、性能或并行结论；因此，上述机制不能作为已测得加速或并行效果的证据。^[rep-table.md:80-80]

## Sources

- [rep-table.md](../../sources/rep-table.md)：共享块存储：reduced 键控复用与 RepTableOwner。

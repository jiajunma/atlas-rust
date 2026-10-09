---
title: 共享 KL 表的惰性构造与回调并发约定
summary: with_kl_table 在回调期间持有记录局部锁，并在获取另一记录锁前拒绝同线程对任何块的嵌套调用，返回 RepInvariantViolation。
sources:
  - rep-table.md
kind: concept
createdAt: "2026-10-09T15:09:09.345Z"
updatedAt: "2026-10-09T22:46:36.301Z"
tags:
  - KL多项式
  - 并发
  - 惰性初始化
aliases:
  - 共享-kl-表的惰性构造与回调并发约定
  - 共K表
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: 共享 KL 表的惰性构造与回调并发约定
summary: with_kl_table 使用按记录惰性构造的共享 KL 表，在整个回调期间持有记录局部互斥锁，并在获取另一记录锁之前拒绝同线程对任何块的嵌套调用。
sources:
  - rep-table.md
kind: concept
createdAt: "2026-10-09T15:09:09.345Z"
updatedAt: "2026-10-10"
tags:
  - KL多项式
  - 并发控制
  - 惰性初始化
aliases:
  - 共享-kl-表的惰性构造与回调并发约定
  - 共K表
provenanceState: extracted
---

# 共享 KL 表的惰性构造与回调并发约定

`with_kl_table(operation)` 使用块记录中惰性构造的共享 KL 表执行回调。这一接口属于为单个实形式提供部分与完整公共块存储的共享设施，相关所有者见 [[RepTableOwner 实形式资源所有者]]。^[rep-table.md:19-27, rep-table.md:48-61]

## 惰性构造与锁的范围

共享 KL 表按记录惰性构造，记录局部互斥锁在**整个回调期间**保持持有。因此，对同一块的调用被串行化；锁的保护范围不只涵盖初始化阶段。^[rep-table.md:50-54]

共享块存储可以复用在某个 Weyl 姿态下与查询匹配的已存块。消费者通过 [[LocatedBlock 稳定块句柄与查询相对姿态]] 获取稳定块句柄及查询相对姿态，而 reduced 键与 Smith codec 保持私有。^[rep-table.md:24-27, rep-table.md:36-46]

## 回调重入限制

KL 回调不得对**任何块**再次调用 `with_kl_table`，包括当前块和其他块。同线程发生嵌套调用时，`ActiveKlCallback::enter()` 会在获取另一记录锁**之前**返回稳定的不变量错误 `RepInvariantViolation`。更换目标块不能绕过这一限制。^[rep-table.md:50-54]

## 与 K 型公式缓存的区别

同一所有者中的 [[K 型公式缓存的锁外计算与提交复核]] 采用不同的并发约定：`k_type_formula` 在生成公式期间不持有共享互斥锁，而是先在锁外计算，提交时再复核；若另一调用方已提交更高截断的公式，则保留后者。`with_kl_table` 则在整个回调期间持锁，两者的锁持有范围不同。^[rep-table.md:50-54, rep-table.md:63-68]

## 证据范围

上述约定来自对 `rep_table.rs` 的结构性阅读，所读字节属于 dirty 工作区，并由来源包的阅读快照记录。互斥与重入约定已由维护者对照源码核实；`ActiveKlCallback` 的进一步展开仍属于后续来源包的范围。^[rep-table.md:9-15, rep-table.md:78-85]

来源包未执行构建、测试或原版运行，不提供数学验收、性能或并行效果结论。块存储正确性属于其自身的 [[HPC 验收证据链]]，本页的接口与锁约定说明不扩展该证据范围。^[rep-table.md:10-11, rep-table.md:80-80]

## Sources

- [rep-table.md](../../sources/rep-table.md) — 共享块存储：reduced 键控复用与 RepTableOwner。

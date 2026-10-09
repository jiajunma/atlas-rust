---
title: 共享 KL 表的惰性构造与回调并发约定
summary: with_kl_table 惰性构造共享 KL 表并在整个回调期间持有记录局部互斥锁；禁止同线程对任何块嵌套调用，重入在获取另一记录锁前返回 RepInvariantViolation。
sources:
  - rep-table.md
kind: concept
createdAt: "2026-10-09T15:09:09.345Z"
updatedAt: "2026-10-09T15:09:09.345Z"
tags:
  - KL表
  - 并发控制
  - 重入限制
aliases:
  - 共享-kl-表的惰性构造与回调并发约定
  - 共K表
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 共享 KL 表的惰性构造与回调并发约定

`with_kl_table(operation)` 使用块记录中惰性构造的共享 KL 表执行回调。它属于为单个实形式提供部分与完整公共块存储的机制；相关资源由 [[RepTableOwner 实形式资源所有者]] 管理，消费者通过 [[LocatedBlock 稳定块句柄与查询相对姿态]] 访问存储块。^[rep-table.md:19-27, rep-table.md:36-54, rep-table.md:56-62]

## 锁的持有范围

记录局部互斥锁在整个回调执行期间保持持有，因此针对同一块的 `with_kl_table` 调用会被串行化。锁的保护范围包含回调本身，而不只是共享 KL 表的惰性构造阶段。^[rep-table.md:48-54]

## 禁止回调重入

KL 回调不得对任何块再次调用 `with_kl_table`：这一约束同时覆盖同一块与其他块。同线程发生嵌套调用时，`ActiveKlCallback::enter()` 会在获取另一记录锁之前返回稳定的不变量错误 `RepInvariantViolation`。^[rep-table.md:50-54]

## 与 K 型公式缓存的区别

同一所有者中的 [[K 型公式缓存的锁外计算与提交复核]] 采用不同的并发约定：公式生成期间不持有共享互斥锁，提交时才复核缓存；如果另一调用方已经提交更高截断的公式，则保留后者。`with_kl_table` 则在整个回调期间持锁，两者的锁持有范围不能混同。^[rep-table.md:50-54, rep-table.md:63-68]

## 证据范围

上述行为来自对 `rep_table.rs` 的结构性阅读，所依据字节属于 dirty 工作区，并由来源包中的快照记录。来源包未执行构建、测试或原版运行，不提供数学验收、性能或并行效果结论；`ActiveKlCallback` 的进一步展开也被列为后续来源包的范围。^[rep-table.md:9-15, rep-table.md:78-80]

## Sources

- [rep-table.md](rep-table.md) — 共享块存储：reduced 键控复用与 RepTableOwner。

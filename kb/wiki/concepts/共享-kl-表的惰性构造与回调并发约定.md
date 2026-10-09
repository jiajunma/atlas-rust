---
title: 共享 KL 表的惰性构造与回调并发约定
summary: with_kl_table 在整个回调期间持有记录局部锁，并在获取嵌套记录锁前拒绝同线程对任何块的重入调用。
sources:
  - rep-table.md
kind: concept
createdAt: "2026-10-09T15:09:09.345Z"
updatedAt: "2026-10-09T19:36:16.304Z"
tags:
  - KL多项式
  - 并发控制
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
---

# 共享 KL 表的惰性构造与回调并发约定

`with_kl_table(operation)` 使用块记录中惰性构造的共享 KL 表执行回调，并在整个回调期间持有记录局部互斥锁。该机制属于为单个实形式提供部分与完整公共块存储的共享设施，相关入口见 [[RepTableOwner 实形式资源所有者]]。^[rep-table.md:19-27, rep-table.md:48-61]

## 惰性构造与锁的范围

共享 KL 表按记录惰性构造。记录局部互斥锁的持有范围覆盖整个回调，因此针对同一块的 `with_kl_table` 调用会被串行化；锁并非只保护初始化阶段。^[rep-table.md:48-54]

消费者通过 [[LocatedBlock 稳定块句柄与查询相对姿态]] 获得存储块句柄、查询对应的存储行号与相对姿态数据。共享存储可能复用不同 Weyl 姿态下的已存块，因此稳定块身份与查询相对姿态需要分别保留。^[rep-table.md:24-27, rep-table.md:36-46]

## 回调重入限制

KL 回调不得对**任何块**再次调用 `with_kl_table`，包括当前块和其他块。同线程发生嵌套调用时，`ActiveKlCallback::enter()` 会在获取另一记录锁之前返回稳定的不变量错误 `RepInvariantViolation`。因此，改为访问另一块也不能绕过重入限制。^[rep-table.md:50-54]

## 与 K 型公式缓存的区别

同一所有者中的 [[K 型公式缓存的锁外计算与提交复核]] 使用不同的锁持有范围：`k_type_formula` 在生成公式期间不持有共享互斥锁，先在锁外计算，提交时再复核；若另一调用方已提交更高截断的公式，则保留后者。`with_kl_table` 的约定则是整个回调期间持锁。^[rep-table.md:50-54, rep-table.md:63-68]

## 证据范围

上述约定来自对 `rep_table.rs` 的结构性阅读，所读字节属于 dirty 工作区，并由来源包的快照记录。互斥与重入约定已由维护者对照源码核实，但 `ActiveKlCallback` 的进一步展开被列为后续来源包的内容。^[rep-table.md:9-15, rep-table.md:78-85]

来源包未执行构建、测试或原版运行，不提供数学验收、性能或并行效果结论；块存储正确性属于其自身的 [[HPC 验收证据链]]，不能由这里的接口与锁约定说明替代。^[rep-table.md:10-11, rep-table.md:80-80]

## Sources

- [rep-table.md](rep-table.md) — 共享块存储：reduced 键控复用与 RepTableOwner

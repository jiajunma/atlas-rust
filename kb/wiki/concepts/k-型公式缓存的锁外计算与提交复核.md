---
title: K 型公式缓存的锁外计算与提交复核
summary: 在共享互斥锁外生成公式，提交时复核缓存并保留并发调用已提交的更大截断结果；来源未提供性能或并行效果验证。
sources:
  - rep-table.md
kind: concept
createdAt: "2026-10-09T15:09:31.584Z"
updatedAt: "2026-10-09T19:36:30.384Z"
tags:
  - K型公式
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
---

# K 型公式缓存的锁外计算与提交复核

`RepTableOwner::k_type_formula(ktype, max_level)` 提供记忆化的 K 型公式。其并发约定是：公式生成期间不持有共享互斥锁，计算完成后在提交时复核缓存；若另一调用方已在此期间提交更大截断的公式，则保留后者。^[rep-table.md:63-68]

## 缓存身份与截断复用

缓存键是 [[RepTableOwner 实形式资源所有者]] 内部的严格 K 型身份 `(x, lambda_rho)`，作用范围属于该实形式所有者。调用可能返回缓存中截断高度高于所请求 `max_level` 的公式，因此调用方在导出前必须自行将各项截断到所求高度。相关接口语义见 [[K 型公式的记忆化与截断复用]]。^[rep-table.md:63-68]

## 锁外计算与提交复核

生成公式的工作在共享互斥锁之外完成。提交阶段再次检查缓存，处理生成期间其他调用方已经提交结果的情况；如果已有更大截断的公式，就保留该公式。这一规则明确了并发提交时对更大截断结果的保留约定。^[rep-table.md:66-68]

## 与共享 KL 表的并发约定对照

[[共享 KL 表的惰性构造与回调并发约定]] 采用另一套锁持有规则：`with_kl_table(operation)` 在整个回调期间持有记录局部互斥锁，使同一块上的调用串行化。KL 回调不得对任何块再次调用 `with_kl_table`；同线程嵌套会在获取另一记录锁之前返回 `RepInvariantViolation`。K 型公式缓存则将公式生成放在共享锁外，并在提交时复核缓存。^[rep-table.md:50-54, rep-table.md:63-68]

## 证据边界

上述约定来自对 `rep_table.rs` 的结构性阅读；来源明确记载，更高截断复用与锁外计算等内容已经维护者逐条对照源码核对。来源包未执行构建、测试或原版运行，因此这些描述不构成数学验收、性能或并行效果的验证结论。^[rep-table.md:9-15, rep-table.md:80-85]

## Sources

- [rep-table.md](rep-table.md)：共享块存储：reduced 键控复用与 RepTableOwner。

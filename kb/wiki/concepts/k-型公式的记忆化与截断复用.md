---
title: K 型公式的记忆化与截断复用
summary: k_type_formula 以所有者内部严格 K 型身份 (x, lambda_rho) 缓存公式，可能返回更高截断的缓存结果，调用方导出前须自行截断到请求高度。
sources:
  - rep-table.md
kind: concept
createdAt: "2026-10-09T15:09:17.051Z"
updatedAt: "2026-10-09T15:09:17.051Z"
tags:
  - K型公式
  - 记忆化
  - 截断语义
aliases:
  - k-型公式的记忆化与截断复用
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# K 型公式的记忆化与截断复用

`RepTableOwner::k_type_formula(ktype, max_level)` 提供记忆化的 K 型公式计算，对应源码注释所指的上游 `Rep_table::K_type_formula`（K_repr.cpp:591–602）。缓存允许复用更高截断的公式，因此调用方在导出结果前必须将各项截断到所请求的高度。^[rep-table.md:63-68]

## 缓存身份与作用范围

[[RepTableOwner 实形式资源所有者|RepTableOwner]] 为一个实形式提供共享存储。K 型公式缓存以该所有者内部的严格 K 型身份 `(x, lambda_rho)` 为键；所请求的截断高度 `max_level` 与这一身份键分别承担截断需求和参数识别的职责。^[rep-table.md:19-20, rep-table.md:63-66]

## 更高截断的复用契约

调用 `k_type_formula(ktype, max_level)` 时，返回值可能来自已缓存的更高截断公式。接口并不保证返回公式恰好截断于 `max_level`；调用方应在导出前自行将各项截断到所求高度。这一要求是使用缓存结果时必须保持的调用契约。^[rep-table.md:63-66]

## 并发计算与提交复核

公式生成期间不持有共享互斥锁：调用先在锁外计算，提交时再复核缓存状态。如果另一调用方在此期间已经提交了更大截断的公式，则保留后者。该流程将生成工作放在锁外，并在提交阶段保留较大的截断结果，详见[[K 型公式缓存的锁外计算与提交复核]]。^[rep-table.md:66-68]

## 证据边界

上述说明来自对 `rep_table.rs` 的结构性阅读，所记录的源码字节属于 dirty 工作区快照。来源包未执行构建、测试或原版运行，因此这些接口与并发约定不构成数学验收、性能或并行效果的结论；上游位置也仅转述自源码注释，未独立重读核对。^[rep-table.md:9-15, rep-table.md:74-80]

## Sources

- [rep-table.md](rep-table.md) — 共享块存储：reduced 键控复用与 RepTableOwner。

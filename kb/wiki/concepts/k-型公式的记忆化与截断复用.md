---
title: K 型公式的记忆化与截断复用
summary: K 型公式以所有者内部严格身份 (x, lambda_rho) 缓存，可返回更高截断结果，调用方须在导出前按请求高度截断。
sources:
  - rep-table.md
kind: concept
createdAt: "2026-10-09T15:09:17.051Z"
updatedAt: "2026-10-09T22:46:35.679Z"
tags:
  - K型
  - 记忆化
  - 截断
aliases:
  - k-型公式的记忆化与截断复用
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: K 型公式的记忆化与截断复用
summary: 以实形式所有者内部的严格 K 型身份缓存公式，允许复用更高截断结果，调用方须在导出前按请求高度截断。
sources:
  - rep-table.md
kind: concept
tags:
  - K型
  - 记忆化
  - 截断
aliases:
  - k-型公式的记忆化与截断复用
provenanceState: extracted
---

# K 型公式的记忆化与截断复用

`RepTableOwner::k_type_formula(ktype, max_level)` 提供记忆化的 K 型公式计算。接口可能返回缓存中更高截断的公式，因此调用方必须在导出前自行将各项截断到所请求的高度。^[rep-table.md:63-68]

## 缓存身份与作用范围

[[RepTableOwner 实形式资源所有者|RepTableOwner]] 为一个实形式提供共享存储。K 型公式的缓存键是该所有者内部的**严格 K 型身份 `(x, lambda_rho)`**；请求高度 `max_level` 不属于这一身份键。^[rep-table.md:19-20, rep-table.md:63-66]

## 截断复用契约

缓存中公式的截断高度可以高于本次请求的 `max_level`。复用这类结果时，返回公式的范围不能直接视为本次请求的输出范围；调用方须在导出前按所求高度截断各项。这是接口明确要求调用方承担的处理步骤。^[rep-table.md:63-66]

## 并发计算与提交复核

公式生成期间不持有共享互斥锁：先在锁外计算，提交时再复核缓存。如果另一调用方在此期间已提交更大截断的公式，则保留后者。相关并发约定见 [[K 型公式缓存的锁外计算与提交复核]]。^[rep-table.md:66-68]

## 来源与证据边界

源码注释将该接口对应到上游 `Rep_table::K_type_formula`（`K_repr.cpp:591–602`）。这一上游位置由来源包转述自源码注释，未独立重读上游核对，行号可能随版本演进而漂移。^[rep-table.md:63-64, rep-table.md:74-75]

本页依据 `rep_table.rs` 的结构性阅读，所读字节来自 dirty 工作区快照。来源包未执行构建、测试或原版运行，不包含数学验收、性能或并行效果的结论。^[rep-table.md:9-15, rep-table.md:80-85]

## Sources

- [rep-table.md](../../sources/rep-table.md) — 共享块存储：reduced 键控复用与 RepTableOwner。

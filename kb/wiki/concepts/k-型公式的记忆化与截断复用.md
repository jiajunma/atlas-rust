---
title: K 型公式的记忆化与截断复用
summary: K 型公式按所有者内部严格身份 (x, lambda_rho) 缓存，可返回更高截断结果，调用方须在导出前按请求高度截断。
sources:
  - rep-table.md
kind: concept
createdAt: "2026-10-09T15:09:17.051Z"
updatedAt: "2026-10-10T00:49:35.838Z"
tags:
  - k型
  - 缓存
aliases:
  - k-型公式的记忆化与截断复用
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: K 型公式的记忆化与截断复用
summary: K 型公式按实形式所有者内部的严格身份缓存，可复用更高截断结果；调用方须在导出前按请求高度截断各项。
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

`RepTableOwner::k_type_formula(ktype, max_level)` 提供记忆化的 K 型公式计算。接口允许返回缓存中更高截断的公式，因此调用方必须在导出前自行将各项截断到所请求的高度。^[rep-table.md:63-68]

## 缓存身份与作用范围

[[RepTableOwner 实形式资源所有者|RepTableOwner]] 为一个实形式提供共享存储。K 型公式以该所有者内部的**严格 K 型身份 `(x, lambda_rho)`** 为缓存键，`max_level` 不属于这一身份键。这里的 K 型身份键与公共块复用所用的 `ReducedParamKey` 应区分，后者采用 `(x, int_sys, residue)`。^[rep-table.md:19-27, rep-table.md:31-34, rep-table.md:63-66]

## 截断复用契约

缓存公式的截断高度可以高于当前请求的 `max_level`。因此，取得缓存结果并不意味着结果已按本次请求裁剪；调用方在导出前仍须按所求高度截断各项。更高截断结果的复用与最终输出的截断是两个独立步骤。^[rep-table.md:63-66]

## 并发计算与提交复核

公式生成期间不持有共享互斥锁。实现先在锁外计算，提交时再复核缓存；如果另一调用方在此期间已经提交了更大截断的公式，则保留后者。详见 [[K 型公式缓存的锁外计算与提交复核]]。^[rep-table.md:66-68]

这一约定不同于 [[共享 KL 表的惰性构造与回调并发约定|共享 KL 表的回调约定]]：`with_kl_table` 在整个回调期间持有记录局部互斥锁，同一块上的调用被串行化，且禁止回调对任何块再次调用 `with_kl_table`。因此，不能将 KL 回调的持锁方式套用于 K 型公式生成。^[rep-table.md:48-54, rep-table.md:66-68]

## 来源与证据边界

来源包将此接口对应到上游 `Rep_table::K_type_formula`（`K_repr.cpp:591–602`）。该位置转述自源码注释，未独立重读上游文件，行号可能随版本演进而漂移。^[rep-table.md:63-64, rep-table.md:74-75]

本页依据 `rep_table.rs` 的结构性阅读，所读字节来自 dirty 工作区快照。来源包未执行构建、测试或原版运行，不提供数学验收、性能或并行效果的结论；上述缓存与并发契约是实现行为的说明。^[rep-table.md:9-15, rep-table.md:80-85]

## Sources

- [rep-table.md](../../sources/rep-table.md) — 共享块存储：reduced 键控复用与 RepTableOwner。

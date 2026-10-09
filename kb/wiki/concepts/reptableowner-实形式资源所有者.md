---
title: RepTableOwner 实形式资源所有者
summary: 为一个实形式绑定 involution 表与 KGB 图，借出 RepContext，并分别查找查询下方最小部分块及包含查询的完整公共块。
sources:
  - rep-table.md
kind: concept
createdAt: "2026-10-09T15:09:28.211Z"
updatedAt: "2026-10-09T19:36:21.784Z"
tags:
  - 实形式
  - 资源所有权
  - 块查找
aliases:
  - reptableowner-实形式资源所有者
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
---

# RepTableOwner 实形式资源所有者

`RepTableOwner` 为一个实形式提供共享的部分与完整公共块存储，并绑定 involution 表与 KGB 图。它对应上游 `Rep_table` 的相关切片，提供临时表示上下文、公共块查找和记忆化的 K 型公式接口。^[rep-table.md:19-27, rep-table.md:56-68]

## 资源绑定与上下文

`new(table, graph)` 校验并绑定自有的 involution 表与 KGB 图对；`from_shared` 绑定已经共享的底层基件。`context()` 借出临时的 [[RepContext 借用上下文与一致性约束|RepContext]]，而 `table()` 与 `graph()` 是过渡性访问器。^[rep-table.md:58-62]

## 公共块查找与复用

`lookup(query)` 解析或物化查询下方最小的部分块；`lookup_full_block(query)` 解析或物化包含查询的完整公共块。部分查找保存规范化（normalised）查询，完整查找保存 dominant 查询；reduced 键与块相对代表元均从保存的查询参数计算。^[rep-table.md:38-41, rep-table.md:60-61]

块复用由私有的 `ReducedParamKey { x, int_sys, residue }` 驱动：`x` 是经 locator attitude 传输后的 KGB 元素，`int_sys` 是规范整数据编号，`residue` 是规范 Smith codec 各赋值的混合进制打包。若查询的积分子系统在某个 Weyl 姿态下与已存块匹配，就复用该块；详见 [[ReducedParamKey 与 reduced 键控块复用]]。^[rep-table.md:21-34]

调用方获得 [[LocatedBlock 稳定块句柄与查询相对姿态|LocatedBlock]]，reduced 键及其 Smith codec 保持私有。句柄提供 `Arc<PartialBlock>`、查询在存储块中的行号及完整块标志，并通过 `block_modifier()`、`relative_shift()` 和 `adapted_representative()` 表达查询相对已存块的姿态。^[rep-table.md:25-27, rep-table.md:36-46]

`has_identity_generator_attitude()` 为真，当且仅当查询到存储的 block modifier 同时具有恒等 `w` 与恒等 `simple_pi`。只有满足此条件，消费者才可用平实中心位移直接读取存储行；仍假设恒等姿态的消费者受到显式门控。^[rep-table.md:25-26, rep-table.md:42-44]

## 共享 KL 表的并发约定

`with_kl_table(operation)` 使用记录中惰性构造的共享 KL 表执行回调。记录局部互斥锁在整个回调期间保持持有，因此同一块上的调用会串行化。^[rep-table.md:48-54]

KL 回调不得对任何块再次调用 `with_kl_table`。同线程嵌套会由 `ActiveKlCallback::enter()` 在获取另一记录锁之前返回稳定的不变量错误 `RepInvariantViolation`。详见 [[共享 KL 表的惰性构造与回调并发约定]]。^[rep-table.md:50-54]

## K 型公式缓存

`k_type_formula(ktype, max_level)` 以该实形式所有者内部的严格 K 型身份 `(x, lambda_rho)` 为键，记忆化 K 型公式。接口可能返回缓存中截断高度更大的公式，因此调用方必须在导出前将各项截断到所求高度；参见 [[K 型公式的记忆化与截断复用]]。^[rep-table.md:63-66]

公式生成期间不持有共享互斥锁：先在锁外计算，再在提交时复核缓存。如果另一调用方在此期间已提交截断高度更大的公式，则保留后者。该机制见 [[K 型公式缓存的锁外计算与提交复核]]。^[rep-table.md:66-68]

## 证据范围

本页依据 `rep_table.rs` 的结构性阅读，来源记录了 dirty 工作区的源码快照。该来源未执行构建、测试或原版运行，不提供数学验收、性能或并行效果结论；块存储正确性属于独立的 [[HPC 验收证据链]]。来源中的上游行号转述自源码注释，未独立重读上游，可能随版本演进而漂移。^[rep-table.md:9-15, rep-table.md:70-80]

## Sources

- [rep-table.md](rep-table.md) — 共享块存储：reduced 键控复用与 RepTableOwner

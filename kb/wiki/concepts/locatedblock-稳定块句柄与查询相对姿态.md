---
title: LocatedBlock 稳定块句柄与查询相对姿态
summary: LocatedBlock 提供共享块句柄、存储行号及查询相对姿态，仅在 modifier 的 w 与 simple_pi 均为恒等时允许通过中心位移直接读取存储行。
sources:
  - rep-table.md
kind: concept
createdAt: "2026-10-09T15:09:16.106Z"
updatedAt: "2026-10-10T00:49:29.291Z"
tags:
  - 块存储
  - 姿态变换
aliases:
  - locatedblock-稳定块句柄与查询相对姿态
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: LocatedBlock 稳定块句柄与查询相对姿态
summary: LocatedBlock 提供稳定的共享块句柄、存储行号与查询相对姿态；只有 modifier 的 w 和 simple_pi 均为恒等时，消费者才能用平实中心位移直接读取存储行。
sources:
  - rep-table.md
kind: concept
tags:
  - 块存储
  - 表示参数
  - 接口契约
aliases:
  - locatedblock-稳定块句柄与查询相对姿态
---

# LocatedBlock 稳定块句柄与查询相对姿态

`LocatedBlock` 是共享公共块存储向消费者提供的稳定块句柄，同时携带查询相对于已存块的姿态数据。存储面向一个实形式：查询的积分子系统若在某个 Weyl 姿态下与已存块匹配，就复用该块，并由 `block_modifier` 记录姿态差。reduced 键及其 Smith codec 保持私有，消费者获得稳定句柄与查询相对代表元。^[rep-table.md:19-27]

## 块对象与查询位置

`block()` 返回 `Arc<PartialBlock>`；`raw_row()` 给出查询在存储块编号中的行号；`is_full()` 表示句柄是否指向完整公共块。块的构造与编号背景见 [[公共块的构造与元素编号（PartialBlock）]]。^[rep-table.md:36-39]

`prepared_query()` 保存准备后的查询参数：部分查找保存规范化（normalised）查询，完整查找保存 dominant 查询。reduced 键与块相对代表元都从该参数计算。^[rep-table.md:40-41]

## 查询相对姿态与访问门控

`block_modifier()`、`relative_shift()` 和 `adapted_representative()` 提供查询相对于已存块的姿态数据。其中，`relative_shift()` 对应来源注释所指的 `repr.h:494` 中的 `shift`。相关概念见 [[BlockModifier 块修正子]]与[[块修正子的相对化与标准参数恢复]]。^[rep-table.md:45-46]

`has_identity_generator_attitude()` 为真，当且仅当查询到存储块的 block modifier 同时具有恒等 `w` 与恒等 `simple_pi`。**只有满足这一条件，消费者才能用平实中心位移直接读取存储行**；仍假设恒等姿态的消费者受到显式门控。^[rep-table.md:25-27, rep-table.md:42-44]

## 查找入口与块复用

[[RepTableOwner 实形式资源所有者]]提供两个查找入口：`lookup(query)` 解析或物化查询下方最小的部分块；`lookup_full_block(query)` 解析或物化包含查询的完整公共块。两者分别遵循 `prepared_query()` 保存规范化查询和 dominant 查询的约定，句柄通过 `is_full()` 暴露块的完整性。^[rep-table.md:38-41, rep-table.md:56-61]

块复用依赖规范化后的 reduced 参数身份。私有 `ReducedParamKey { x: KgbId, int_sys: u32, residue: u32 }` 包含经 locator attitude 传输后的 KGB 元素 `x`、规范整数据编号 `int_sys`，以及规范 codec 各赋值的混合进制打包 `residue`。该机制支持 Weyl 姿态匹配下的已存块复用，详见 [[ReducedParamKey 与 reduced 键控块复用]]。^[rep-table.md:19-34]

## 共享 KL 表的回调约定

`with_kl_table(operation)` 使用记录中惰性构造的共享 KL 表执行回调。记录局部互斥锁在整个回调期间保持持有，因此同一块上的调用被串行化。KL 回调不得对任何块再次调用 `with_kl_table`；同线程嵌套会由 `ActiveKlCallback::enter()` 在获取另一记录锁之前返回稳定的不变量错误 `RepInvariantViolation`。详见 [[共享 KL 表的惰性构造与回调并发约定]]。^[rep-table.md:48-54]

## 证据范围

本页依据 `rep_table.rs` 的结构性阅读材料，所读字节记录于 dirty 工作区快照 `snapshots/2026-10-03-rep-table.json`。块存储正确性属于独立的 HPC 证据链，本来源包不重述或扩展该证据。^[rep-table.md:9-15]

来源包未执行构建、测试或原版运行，不提供数学验收、性能或并行结论。上游文件行号转述自源码注释，未独立重读上游，可能随版本演进而漂移；`BlockLocator`、`IntegralDatumTable` 与 `ActiveKlCallback` 的进一步展开属于后续来源包。^[rep-table.md:70-80]

## Sources

- [rep-table.md](../../sources/rep-table.md) — 共享块存储：reduced 键控复用与 RepTableOwner。

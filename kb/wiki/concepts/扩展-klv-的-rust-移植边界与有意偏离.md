---
title: 扩展 KLV 的 Rust 移植边界与有意偏离
summary: 移植使用自有池、推迟共享池及 swallow、省略部分调试检查，并仅覆盖扩展块构建后的矩阵流程；结构性阅读不证明结果等价或数学正确性。
sources:
  - extended-kl.md
kind: concept
createdAt: "2026-10-09T14:48:49.122Z"
updatedAt: "2026-10-09T20:53:08.871Z"
tags:
  - Rust移植
  - 兼容性
  - 证据边界
aliases:
  - 扩展-klv-的-rust-移植边界与有意偏离
  - 扩K的R移
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=81ad51b115c49a37eb623781761b22803544a24ebca2c6ce25d8ff094a836c7b
---

---
title: 扩展 KLV 的 Rust 移植边界与有意偏离
summary: Rust 扩展 KLV 表采用自有多项式池，传播列填充错误，推迟共享池和 swallow，省略部分检查，并仅覆盖扩展块已构建后的矩阵流程；结构性阅读不构成数学验收或结果等价性的独立证据。
sources:
  - extended-kl.md
kind: concept
tags:
  - Rust移植
  - 兼容性
  - 证据边界
aliases:
  - 扩展-klv-的-rust-移植边界与有意偏离
---

# 扩展 KLV 的 Rust 移植边界与有意偏离

扩展（twisted）KLV 多项式表的 Rust 实现位于 `ext_kl.rs`，对应上游 `gkmod/ext_kl.{h,cpp}`。实现包含预计算下降集、good ascents 与 primitivisation 符号的 `DescentTable`，逐列保存多项式池索引的 `ExtKlTable`，以及 `condense`、`ext_kl_matrix`、`contributions` 等自由函数。其递归采用扩展块版本，存储结构参见 [[扩展 KLV 多项式表的逐列存储]]。^[extended-kl.md:19-24, extended-kl.md:108-109]

## 多项式池与符号表示

实现复用 `kl_polynomial.rs` 的 `KlHashTable`，池条目为系数类型是 `i32` 的 `KlPol`，对应上游 `Polynomial<int>`。索引不打包符号位；primitivisation 符号独立保存在 `prim_flip` bitmap 中，`kl_pol_index` 返回 `(KLIndex, bool)`。这种表示与上游一致，详见 [[多项式池与 primitivisation 符号分离]]。^[extended-kl.md:28-35]

Rust 的 `ExtKlTable` 始终拥有自己的多项式池。上游共享池指针模式，以及用于 partial-block 迁移的 `swallow`，被推迟到需要这些功能的 common-block 切片。^[extended-kl.md:93-95]

## 列填充与错误传播

`fill_columns(limit)` 计算所有 `y < limit` 的列，`limit == 0` 表示填满整块。计算失败时，Rust 清空出错列并传播错误；上游则通过 `catch(...)` 吞掉异常。这一实现保持“每列要么为空，要么完整”的不变式，参见 [[列填充的完整性不变式与错误传播]]。^[extended-kl.md:84-87]

来源将有意偏离概括为“不改变可观察结果”，同时明确记录了上述错误传播差异。^[extended-kl.md:84-98]

因此，该概括不能解释为失败路径的行为完全相同，也不能单独作为两种实现结果等价的验证证据。

## 未移植的检查与辅助路径

上游受 `#ifndef NDEBUG` 控制的辅助函数 `get_M` 和 down-set defect 检查未移植；生产路径使用 `get_Mp`。`check_polys` 也未移植，因为它需要 `ExtBlock` 不保留的 untwisted block。^[extended-kl.md:97-100]

## 矩阵构造的功能边界

`ext_kl_matrix` 的移植以已有 `ExtBlock` 为起点，覆盖其后的填充、展开、`condense`、parity 符号翻转与 pool-index 矩阵处理。从 `StandardRepr` 构建扩展块需要 `StandardReprMod`／`common_block`，这部分留给后续切片。扩展块背景参见 [[扩展块与 δ-不动部分]]。^[extended-kl.md:101-104]

## 证据范围

本页依据结构性源码阅读材料。来源记录的 `ext_kl.rs` 来自 dirty 工作区，其 SHA-256 为 `923c34ffcaae013a3ae9e9aad25b1b8daff69457c3d056e1dd1f6fca7543916b`，阅读快照为 `snapshots/2026-10-03-extended-kl.json`。上游行号转述自源码注释，未独立重读上游，可能随版本演进而漂移。^[extended-kl.md:9-15, extended-kl.md:108-111]

来源未执行构建、测试或原版运行，不提供数学验收、性能或并行结论。扩展 KL 的正确性属于独立的 HPC 证据链，包括来源提及的 unitarity gate；本材料不重述或扩展该证据链的结论。^[extended-kl.md:10-11, extended-kl.md:114-114]

## Sources

- [extended-kl.md](../../sources/extended-kl.md)：扩展 KLV 多项式表：primitivisation 符号与逐列存储。

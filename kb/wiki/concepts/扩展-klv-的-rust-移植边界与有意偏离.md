---
title: 扩展 KLV 的 Rust 移植边界与有意偏离
summary: 移植采用自有池，推迟共享池和 swallow，省略部分调试检查，并仅覆盖扩展块已构建后的矩阵流程；源码包不提供数学验收或结果等价性的独立证据。
sources:
  - extended-kl.md
kind: concept
createdAt: "2026-10-09T14:48:49.122Z"
updatedAt: "2026-10-09T14:48:49.122Z"
tags:
  - Rust移植
  - 兼容性
  - 证据边界
aliases:
  - 扩展-klv-的-rust-移植边界与有意偏离
confidence: 0.99
provenanceState: merged
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=c27066b97a40017b600b4a46bc3f8cbf26c795bc54fde472704cec6f888a58cb
---

# 扩展 KLV 的 Rust 移植边界与有意偏离

扩展（twisted）KLV 多项式表的 Rust 实现位于 `ext_kl.rs`，对应上游 `gkmod/ext_kl.{h,cpp}`。其结构包含预计算下降集、good ascents 和 primitivisation 符号的 `DescentTable`，以及逐列保存多项式池索引的 `ExtKlTable`；模块还提供 `condense`、`ext_kl_matrix` 和 `contributions` 等自由函数。递归采用扩展块版本，相关存储结构见 [[扩展 KLV 多项式表的逐列存储]]。^[extended-kl.md:19-24]

## 多项式池与符号表示

实现复用 `kl_polynomial.rs` 的 `KlHashTable`，池条目为系数类型是 `i32` 的 `KlPol`，对应上游 `Polynomial<int>`。索引不打包符号位；primitivisation 的符号翻转独立保存在 `prim_flip` bitmap 中，`kl_pol_index` 返回 `(KLIndex, bool)`。这种池与符号分离的表示与上游一致，并非移植偏离；详见 [[多项式池与 primitivisation 符号分离]]。^[extended-kl.md:28-35]

Rust 的 `ExtKlTable` 始终拥有自己的多项式池。上游的共享池指针模式和用于 partial-block 迁移的 `swallow` 尚未移植，来源将其推迟到需要这些功能的 common-block 切片。^[extended-kl.md:93-95]

## 列填充与错误传播

`fill_columns(limit)` 计算所有 `y < limit` 的列，`limit == 0` 表示填满整块。Rust 在计算失败时清空出错列并向调用方传播错误，而上游通过 `catch(...)` 吞掉异常。该策略保持“每列要么为空，要么完整”的不变式，相关说明见 [[列填充的完整性不变式与错误传播]]。^[extended-kl.md:84-87]

来源将这些差异列为“不改变可观察结果”的有意偏离，但同时明确记录了错误处理行为的变化：Rust 调用方会收到传播的错误。因此，这一概括不能理解为失败路径上的行为完全相同。^[extended-kl.md:84-98]

## 未移植的检查与辅助路径

上游受 `#ifndef NDEBUG` 控制的辅助函数 `get_M` 和 down-set defect 检查未移植，生产路径使用 `get_Mp`。`check_polys` 也未移植，因为它依赖 `ExtBlock` 不保留的 untwisted block。^[extended-kl.md:97-100]

## 矩阵构造的功能边界

`ext_kl_matrix` 的移植以已有 `ExtBlock` 为边界。当前覆盖扩展块创建之后的填充、展开、`condense`、parity 符号翻转和 pool-index 矩阵处理；从 `StandardRepr` 构建扩展块所需的 `StandardReprMod`／`common_block` 部分被留给后续切片。相关概念见 [[扩展块与 δ-不动部分]]。^[extended-kl.md:101-104]

## 证据范围

本页依据的是结构性源码阅读材料，其快照记录了 dirty 工作区中的 `ext_kl.rs` 字节。来源中的上游行号转述自源码注释，未独立重读上游，可能随版本变化而漂移。^[extended-kl.md:9-15, extended-kl.md:108-111]

该材料未执行构建、测试或原版运行，不提供数学验收、性能或并行结论。扩展 KL 的正确性由独立的 [[HPC 验收证据链]] 支撑，包括来源提及的 unitarity gate；这里的移植范围说明不扩展其验收结论。^[extended-kl.md:10-11, extended-kl.md:114-114]

## Sources

- [extended-kl.md](extended-kl.md)：扩展 KLV 多项式表：primitivisation 符号与逐列存储。

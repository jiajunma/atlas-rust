---
title: 扩展 KL 矩阵的块后处理管线
summary: ext_kl_matrix 在已有 ExtBlock 上执行填充、展开、condense、奇偶符号翻转及池索引矩阵生成，本切片不包含从 StandardRepr 构建扩展块的过程。
sources:
  - extended-kl.md
kind: concept
createdAt: "2026-10-09T19:29:03.406Z"
updatedAt: "2026-10-10T00:33:22.546Z"
tags:
  - 扩展KLV
  - 矩阵
  - 处理管线
aliases:
  - 扩展-kl-矩阵的块后处理管线
  - 扩K矩
confidence: 1
provenanceState: extracted
modelId: codex-cli-default
promptVersion: v6
promptModifiers:
  - lang=zh-CN
  - policy=64721d7a1a45edb7f094b26adcd835a9732563f7c9e12935cdd235fbb15ae06d
---

---
title: 扩展 KL 矩阵的块后处理管线
summary: ext_kl_matrix 在已有 ExtBlock 上执行填充、展开、condense、奇偶符号翻转与池索引矩阵生成；当前移植不包含从 StandardRepr 构建扩展块。
sources:
  - extended-kl.md
kind: concept
tags:
  - 扩展KLV
  - 矩阵构造
  - 移植边界
aliases:
  - 扩展-kl-矩阵的块后处理管线
  - 扩K矩
provenanceState: extracted
---

# 扩展 KL 矩阵的块后处理管线

扩展 KL 矩阵的块后处理管线指 `ext_kl_matrix` 在已有 `ExtBlock` 上执行的处理部分，包括填充（fill）、展开（expand）、`condense`、奇偶性（parity）符号翻转，以及多项式池索引矩阵的生成。当前移植以 `ExtBlock` 为边界，不包含从标准表示参数构建扩展块的过程。^[extended-kl.md:101-104]

## 输入边界与模块分工

从 [[StandardRepr 标准表示参数|StandardRepr]] 构建扩展块需要 `StandardReprMod` 与 `common_block`，来源将这部分划归后续切片。因此，本管线以扩展块已经存在为前提；扩展块的相关概念见 [[扩展块与 δ-不动部分]]。^[extended-kl.md:101-104]

扩展 KLV 模块分为三个层次：`DescentTable` 预计算下降集、good ascent 与带 primitivisation 符号翻转信息的 primitive 索引；`ExtKlTable` 按块元素 `y` 保存多项式池索引列；模块尾部提供 `condense`、`ext_kl_matrix`、`contributions` 等自由函数。^[extended-kl.md:19-24]

## 列填充与访问基础

`ExtKlTable` 为每个块元素 `y` 保存一列池索引，按照 `x` 相对于 `y` 的下降集所对应的 primitive 位置寻址。`kl_pol_index` 返回 `(KLIndex, bool)`；`p(x,y)` 则在翻转标志为真时应用 `scaled(-1)`，得到 twisted KLV 多项式 $P_{x,y}$。详见 [[扩展 KLV 多项式表的逐列存储]] 与 [[Primitivisation 索引与符号传播]]。^[extended-kl.md:72-80]

`fill_columns(limit)` 计算所有 `y < limit` 的列，`limit == 0` 时填满整块。填充失败时，Rust 实现清空出错列并传播错误，保持“每列要么为空、要么完整”的不变式；上游则通过 `catch(...)` 吞掉异常。这是明确记录的有意偏离，参见 [[列填充的完整性不变式与错误传播]]。^[extended-kl.md:84-87]

## 后处理与符号表示

来源列出的矩阵处理顺序为 fill、expand、`condense`、parity 符号翻转，最后形成 pool-index 矩阵。该材料只给出这些阶段及其移植边界，没有展开 `expand`、`condense` 的内部算法或 parity 判定公式。^[extended-kl.md:101-104]

多项式池复用 `KlHashTable`，条目为系数类型是 `i32` 的 `KlPol`。池索引不打包符号位；primitivisation 符号存储在独立的 `prim_flip` 位图中，查询时另行读取。`raw_ext_KL` 包装层将索引与翻转标志渲染为 `inx.second ? -inx.first : inx.first`。相关表示见 [[多项式池与 primitivisation 符号分离]]。^[extended-kl.md:28-35]

## 实现范围与证据边界

当前 `ExtKlTable` 始终拥有自己的多项式池。共享池指针模式与用于 partial-block 迁移的 `swallow` 被推迟到需要它们的 common-block 切片；`check_polys` 也未移植，因为它依赖 `ExtBlock` 未保留的 untwisted block。详见 [[扩展 KLV 的 Rust 移植边界与有意偏离]]。^[extended-kl.md:93-104]

本页依据对 `ext_kl.rs` 的结构性阅读，所读快照对应 dirty 工作区字节。来源未执行构建、测试或原版运行，不提供数学验收、性能或并行结论；扩展 KL 的正确性属于其独立的 [[HPC 验收证据链]]。来源中的上游行号转述自源码注释，未独立重读上游，可能随版本演进而漂移。^[extended-kl.md:9-15, extended-kl.md:108-114]

## Sources

- [extended-kl.md](../../sources/extended-kl.md) — 扩展 KLV 多项式表：primitivisation 符号与逐列存储。
